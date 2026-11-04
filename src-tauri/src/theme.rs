use crate::catalog::{bounded, home};
use serde::Serialize;
use std::{collections::BTreeMap, path::PathBuf, sync::Mutex};
#[derive(Clone, Serialize, PartialEq, Debug)]
pub struct Theme {
    pub generation: u64,
    pub roles: BTreeMap<String, String>,
    pub mode: String,
    pub source: String,
    pub fallback: bool,
    pub warnings: Vec<String>,
}
pub static LAST: Mutex<Option<Theme>> = Mutex::new(None);
fn hex(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|c| c.is_ascii_hexdigit())
}
fn luminance(s: &str) -> f64 {
    let c: Vec<f64> = (1..7)
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap_or(0) as f64 / 255.)
        .map(|v| {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        })
        .collect();
    c[0] * 0.2126 + c[1] * 0.7152 + c[2] * 0.0722
}
fn ink(s: &str) -> String {
    if luminance(s) > 0.179 {
        "#000000"
    } else {
        "#ffffff"
    }
    .into()
}
pub fn decode(s: &str) -> Result<Theme, String> {
    decode_marker(s, false)
}
fn decode_marker(s: &str, marker: bool) -> Result<Theme, String> {
    let mut v = s.parse::<toml::Table>().map_err(|_| "TOML incomplet")?;
    for (key, aliases) in [
        ("background", &["bg", "color0"][..]),
        ("foreground", &["fg", "color7"][..]),
        ("accent", &["blue", "color4"][..]),
        ("red", &["color1"][..]),
        (
            "selection",
            &["selection_background", "color8", "color0", "background"][..],
        ),
        (
            "muted",
            &["color8", "dark_foreground", "dark_fg", "foreground"][..],
        ),
    ] {
        if !v.contains_key(key) {
            if let Some(value) = aliases.iter().find_map(|k| v.get(*k)).cloned() {
                v.insert(key.into(), value);
            }
        }
    }
    let inferred = if marker
        || v.get("background")
            .and_then(|v| v.as_str())
            .filter(|s| hex(s))
            .is_some_and(|s| {
                (1..7)
                    .step_by(2)
                    .map(|i| u16::from_str_radix(&s[i..i + 2], 16).unwrap())
                    .sum::<u16>()
                    > 382
            }) {
        "light"
    } else {
        "dark"
    };
    let mode = v
        .get("mode")
        .or_else(|| v.get("theme_type"))
        .and_then(|v| v.as_str())
        .unwrap_or(inferred);
    if mode != "dark" && mode != "light" {
        return Err("mode invalide".into());
    }
    let light = mode == "light";
    let mut warnings = Vec::new();
    let mut get = |k: &str, default: &str| -> Result<String, String> {
        match v.get(k) {
            Some(v) => {
                let s = v.as_str().ok_or("couleur invalide")?;
                if !hex(s) {
                    return Err("couleur invalide".into());
                }
                Ok(s.into())
            }
            None => {
                warnings.push(format!("Rôle {k} absent"));
                Ok(default.into())
            }
        }
    };
    let bg = get("background", if light { "#f4f2ec" } else { "#10151b" })?;
    let fg = get("foreground", if light { "#18252c" } else { "#f0f4f7" })?;
    let accent = get("accent", if light { "#245b72" } else { "#a5d5ef" })?;
    let selection = get("selection", &bg)?;
    let muted = get("muted", &fg)?;
    let error = get("red", if light { "#a72b33" } else { "#ffa8a8" })?;
    let contrast =
        (luminance(&fg).max(luminance(&bg)) + 0.05) / (luminance(&fg).min(luminance(&bg)) + 0.05);
    let fg = if contrast >= 4.5 {
        fg
    } else {
        warnings.push("Contraste corrigé".into());
        ink(&bg)
    };
    let mut roles = BTreeMap::new();
    roles.insert("background".into(), bg.clone());
    // Keep surface hierarchy while preserving readable ink on arbitrary themes.
    for (k, amount) in [
        ("surface", 0.03),
        ("elevated", 0.06),
        ("hover", 0.08),
        ("selection", 0.10),
    ] {
        let channels: Vec<u8> = (1..7)
            .step_by(2)
            .map(|i| {
                let b = u8::from_str_radix(&bg[i..i + 2], 16).unwrap() as f64;
                let f = u8::from_str_radix(&fg[i..i + 2], 16).unwrap() as f64;
                (b * (1.0 - amount) + f * amount).round() as u8
            })
            .collect();
        let surface = format!("#{:02x}{:02x}{:02x}", channels[0], channels[1], channels[2]);
        let contrast = (luminance(&fg).max(luminance(&surface)) + 0.05)
            / (luminance(&fg).min(luminance(&surface)) + 0.05);
        roles.insert(k.into(), if contrast >= 4.5 { surface } else { bg.clone() });
    }
    for (k, dark, light_color) in [
        ("success", "#9fe3b3", "#246438"),
        ("warning", "#ffcf86", "#765000"),
        ("destructive", "#ffa8a8", "#a72b33"),
    ] {
        let color = if light { light_color } else { dark };
        let contrast = (luminance(color).max(luminance(&bg)) + 0.05)
            / (luminance(color).min(luminance(&bg)) + 0.05);
        roles.insert(
            k.into(),
            if contrast >= 4.5 {
                color.into()
            } else {
                ink(&bg)
            },
        );
    }
    for k in ["foreground", "muted", "focus", "border"] {
        roles.insert(k.into(), fg.clone());
    }
    roles.insert("selection".into(), selection.clone());
    roles.insert("selection-foreground".into(), ink(&selection));
    roles.insert("muted".into(), muted);
    roles.insert("error".into(), error.clone());
    roles.insert("destructive".into(), error);
    roles.insert("overlay".into(), "#080d13".into());
    roles.insert("overlay-foreground".into(), "#ffffff".into());
    roles.insert("accent-foreground".into(), ink(&accent));
    roles.insert("accent".into(), accent);
    Ok(Theme {
        generation: 0,
        roles,
        mode: mode.into(),
        source: "Omarchy".into(),
        fallback: !warnings.is_empty(),
        warnings,
    })
}
pub fn paths() -> Vec<(PathBuf, &'static str)> {
    let h = home();
    let state = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| h.join(".local/state"));
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| h.join(".config"));
    let mut paths = vec![
        (
            state.join("omarchy/current/theme/colors.toml"),
            "Omarchy actuel",
        ),
        (
            config.join("omarchy/current/theme/colors.toml"),
            "Omarchy ancien",
        ),
    ];
    let real = h.join(".local/state/omarchy/current/theme/colors.toml");
    if paths[0].0 != real {
        paths.insert(1, (real, "Omarchy HOME upstream"));
    }
    paths
}
pub fn read() -> Theme {
    read_paths(&paths())
}
fn read_paths(paths: &[(PathBuf, &str)]) -> Theme {
    for (path, source) in paths {
        if path.exists() {
            let marker = path.with_file_name("light.mode").is_file();
            match bounded(path, 65536)
                .and_then(|bytes| {
                    if bounded(path, 65536)? != bytes
                        || marker != path.with_file_name("light.mode").is_file()
                    {
                        return Err("Thème en remplacement".into());
                    }
                    Ok(bytes)
                })
                .and_then(|v| String::from_utf8(v).map_err(|_| "UTF8".into()))
                .and_then(|s| {
                    let v = s.parse::<toml::Table>().map_err(|_| "TOML incomplet")?;
                    if !["background", "bg", "color0"]
                        .iter()
                        .any(|k| v.contains_key(*k))
                        || !["foreground", "fg", "color7"]
                            .iter()
                            .any(|k| v.contains_key(*k))
                    {
                        return Err("Palette incomplète".into());
                    }
                    decode_marker(&s, marker)
                }) {
                Ok(mut t) => {
                    t.source = (*source).into();
                    *LAST.lock().unwrap() = Some(t.clone());
                    return t;
                }
                Err(_) => {
                    if let Some(mut t) = LAST.lock().unwrap().clone() {
                        t.warnings = vec!["Thème invalide : dernière palette conservée".into()];
                        return t;
                    }
                }
            }
        }
    }
    if let Some(mut t) = LAST.lock().unwrap().clone() {
        t.warnings = vec!["Source temporairement absente : dernière palette conservée".into()];
        return t;
    }
    let mut t = decode("").unwrap();
    t.source = "Palette de secours".into();
    t
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn themes() {
        assert!(
            decode("mode='light'\nbackground='#ffffff'\nforeground='#ffffff'")
                .unwrap()
                .warnings
                .contains(&"Contraste corrigé".into())
        );
        for background in ["#ffffff", "#000000", "#777777", "#10151b"] {
            let t = decode(&format!("background='{background}'")).unwrap();
            for surface in ["background", "surface", "elevated", "hover"] {
                let a = luminance(&t.roles["foreground"]);
                let b = luminance(&t.roles[surface]);
                assert!((a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5);
            }
            for role in ["success", "warning", "destructive"] {
                assert!(t.roles.contains_key(role));
            }
        }
        assert!(decode("blue='invalid'").is_err());
        assert!(decode("mode='").is_err());
        assert!(decode("blue='#123456'").unwrap().fallback);
    }
}

#[cfg(test)]
mod reload_tests {
    use super::*;
    #[test]
    fn hot_reload_and_last_valid() {
        let dir = std::env::temp_dir().join(format!("omarchy-theme-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("colors.toml");
        let paths = [(path.clone(), "Fixture")];
        std::fs::write(
            &path,
            "mode='dark'\nbackground='#10151b'\nforeground='#ffffff'\nblue='#a5d5ef'",
        )
        .unwrap();
        let dark = read_paths(&paths);
        assert_eq!(dark.mode, "dark");
        std::fs::write(&path, "mode='").unwrap();
        assert_eq!(read_paths(&paths).roles, dark.roles);
        std::fs::remove_file(&path).unwrap();
        assert_eq!(read_paths(&paths).roles, dark.roles);
        let tmp = dir.join("next");
        std::fs::write(
            &tmp,
            "mode='light'\nbackground='#ffffff'\nforeground='#18252c'\nblue='#245b72'",
        )
        .unwrap();
        std::fs::rename(tmp, path).unwrap();
        assert_eq!(read_paths(&paths).mode, "light");
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(test)]
mod contracts {
    use super::*;
    #[test]
    fn rc3_precedence_and_mode() {
        for fixture in [
            include_str!("../../tests/fixtures/themes/stable-4.0.6.toml"),
            include_str!("../../tests/fixtures/themes/rc3.toml"),
        ] {
            assert!(decode(fixture).is_ok());
        }
        let s = "background='#ffffff'\nforeground='#18252c'\naccent='#123456'\nblue='#abcdef'\nselection='#223344'\nmuted='#556677'";
        let t = decode(s).unwrap();
        assert_eq!(t.mode, "light");
        assert_eq!(t.roles["accent"], "#123456");
        assert_eq!(t.roles["selection"], "#223344");
        assert_eq!(t.roles["muted"], "#556677");
        assert_eq!(
            decode_marker("background='#10151b'", true).unwrap().mode,
            "light"
        );
        assert_eq!(decode_marker("mode='dark'", true).unwrap().mode, "dark");
        assert_eq!(
            decode_marker("theme_type='dark'", true).unwrap().mode,
            "dark"
        );
        let t = decode("bg='#10151b'\nfg='#ffffff'\ncolor4='#123456'").unwrap();
        assert_eq!(t.roles["accent"], "#123456");
        assert_eq!(t.roles["background"], "#10151b");
    }
}
