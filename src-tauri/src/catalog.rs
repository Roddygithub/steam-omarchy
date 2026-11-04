use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone)]
pub enum V {
    Text(String),
    Object(BTreeMap<String, V>),
}
impl V {
    pub fn text(&self, k: &str) -> Option<&str> {
        if let V::Object(m) = self {
            if let Some(V::Text(s)) = m.get(k) {
                return Some(s);
            }
        }
        None
    }
    pub fn object(&self, k: &str) -> Option<&BTreeMap<String, V>> {
        if let V::Object(m) = self {
            if let Some(V::Object(v)) = m.get(k) {
                return Some(v);
            }
        }
        None
    }
}
pub fn parse(s: &str) -> Result<V, String> {
    let b = s.as_bytes();
    let mut i = 0;
    let mut tokens = Vec::new();
    while i < b.len() {
        match b[i] {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b'/' if b.get(i + 1) == Some(&b'/') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1
                }
            }
            b'{' | b'}' => {
                tokens.push((b[i] as char).to_string());
                i += 1
            }
            b'"' => {
                i += 1;
                let mut v = Vec::new();
                let mut closed = false;
                while i < b.len() {
                    if b[i] == b'"' {
                        i += 1;
                        closed = true;
                        break;
                    }
                    if b[i] == b'\\' {
                        i += 1;
                        if i >= b.len() {
                            return Err("escape tronqué".into());
                        }
                        match b[i] {
                            b'n' => v.push(b'\n'),
                            b't' => v.push(b'\t'),
                            b'"' | b'\\' => v.push(b[i]),
                            _ => {
                                v.push(b'\\');
                                v.push(b[i])
                            }
                        }
                    } else {
                        v.push(b[i])
                    }
                    i += 1
                }
                if !closed {
                    return Err("chaîne tronquée".into());
                }
                tokens.push(format!("s{}", String::from_utf8(v).map_err(|_| "UTF8")?));
            }
            _ => return Err("token invalide".into()),
        }
        if tokens.len() > 100000 {
            return Err("trop de tokens".into());
        }
    }
    fn obj(t: &[String], i: &mut usize, depth: usize, nested: bool) -> Result<V, String> {
        if depth > 32 {
            return Err("profondeur".into());
        }
        let mut m = BTreeMap::new();
        while *i < t.len() {
            if t[*i] == "}" {
                if !nested {
                    return Err("fermeture inattendue".into());
                }
                *i += 1;
                return Ok(V::Object(m));
            }
            let key = t[*i].strip_prefix('s').ok_or("clé invalide")?.to_string();
            *i += 1;
            let value = t.get(*i).ok_or("valeur absente")?;
            let v = if value == "{" {
                *i += 1;
                obj(t, i, depth + 1, true)?
            } else {
                *i += 1;
                V::Text(value.strip_prefix('s').ok_or("valeur invalide")?.into())
            };
            if m.insert(key, v).is_some() {
                return Err("clé dupliquée".into());
            }
        }
        if nested {
            return Err("objet tronqué".into());
        }
        Ok(V::Object(m))
    }
    obj(&tokens, &mut 0, 0, false)
}
pub fn bounded(p: &Path, max: u64) -> Result<Vec<u8>, String> {
    if !fs::metadata(p).map_err(|_| "métadonnées")?.is_file() {
        return Err("pas un fichier".into());
    }
    let f = fs::File::open(p).map_err(|_| "lecture impossible")?;
    if !f.metadata().map_err(|_| "métadonnées")?.is_file() {
        return Err("pas un fichier".into());
    }
    let mut v = Vec::new();
    f.take(max + 1).read_to_end(&mut v).map_err(|_| "lecture")?;
    if v.len() as u64 > max {
        return Err("taille dépassée".into());
    }
    Ok(v)
}
pub fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 10
        && s.bytes().all(|c| c.is_ascii_digit())
        && s.parse::<u32>().is_ok_and(|x| x > 0)
        && !s.starts_with('0')
}
pub fn home() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
}
pub fn roots() -> Vec<PathBuf> {
    let h = home();
    let mut seen = BTreeSet::new();
    [
        h.join(".local/share/Steam"),
        h.join(".steam/steam"),
        h.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
    ]
    .into_iter()
    .filter_map(|p| p.canonicalize().ok())
    .filter(|p| seen.insert(p.clone()))
    .collect()
}
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub app_id: String,
    pub name: String,
    pub library_ids: Vec<String>,
    pub installation: String,
    pub cover: bool,
    pub hero: bool,
    #[serde(skip)]
    pub process_dirs: Vec<PathBuf>,
}
#[derive(Serialize)]
pub struct Catalog {
    pub entries: Vec<Entry>,
    pub issues: Vec<String>,
    pub libraries: usize,
}
pub fn scan_from(roots: &[PathBuf]) -> Catalog {
    let mut libraries = BTreeSet::new();
    let mut issues = Vec::new();
    for root in roots {
        libraries.insert(root.clone());
        let path = root.join("steamapps/libraryfolders.vdf");
        if fs::symlink_metadata(&path).is_ok() {
            match bounded(&path, 1024 * 1024).and_then(|b| parse(&String::from_utf8_lossy(&b))) {
                Ok(v) => {
                    if let Some(m) = v.object("libraryfolders") {
                        for (i, (k, v)) in m.iter().enumerate() {
                            if i == 128 {
                                issues.push("Liste de bibliothèques tronquée (128 entrées)".into());
                                break;
                            }
                            if k.bytes().all(|c| c.is_ascii_digit()) {
                                let p = match v {
                                    V::Text(s) => Some(s.as_str()),
                                    _ => v.text("path"),
                                };
                                if let Some(p) = p {
                                    if Path::new(p).is_absolute() {
                                        libraries.insert(PathBuf::from(p));
                                    } else {
                                        issues.push("Chemin de bibliothèque non absolu".into());
                                    }
                                } else {
                                    issues.push("Chemin de bibliothèque absent ou invalide".into());
                                }
                            }
                        }
                    } else {
                        issues.push("Racine libraryfolders absente ou invalide".into());
                    }
                }
                Err(_) => issues.push("Liste de bibliothèques illisible".into()),
            }
        }
    }
    let mut entries: BTreeMap<String, Entry> = BTreeMap::new();
    if libraries.len() > 128 {
        issues.push("Scan limité à 128 bibliothèques".into());
    }
    let mut scanned = 0;
    for (index, lib) in libraries.iter().take(128).enumerate() {
        let Ok(dir) = fs::read_dir(lib.join("steamapps")) else {
            issues.push(format!("Bibliothèque {} illisible", index + 1));
            continue;
        };
        scanned += 1;
        for (i, file) in dir.enumerate() {
            if i == 20000 {
                issues.push(format!(
                    "Bibliothèque {} tronquée (20000 entrées)",
                    index + 1
                ));
                break;
            }
            let file = match file {
                Ok(f) => f,
                Err(_) => {
                    issues.push(format!("Erreur d’itération bibliothèque {}", index + 1));
                    continue;
                }
            };
            let name = file.file_name().to_string_lossy().into_owned();
            let Some(id) = name
                .strip_prefix("appmanifest_")
                .and_then(|s| s.strip_suffix(".acf"))
            else {
                continue;
            };
            if !valid_id(id) {
                issues.push("Nom de manifeste invalide".into());
                continue;
            }
            let parsed = bounded(&file.path(), 1024 * 1024)
                .and_then(|b| parse(&String::from_utf8_lossy(&b)));
            let Ok(v) = parsed else {
                issues.push(format!("Manifeste {id} invalide"));
                continue;
            };
            let Some(m) = v.object("AppState") else {
                issues.push(format!("Manifeste {id} incomplet"));
                continue;
            };
            let v = V::Object(m.clone());
            if v.text("appid") != Some(id) {
                issues.push(format!("AppID {id} incohérent"));
                continue;
            }
            let Some(title) = v
                .text("name")
                .filter(|n| !n.trim().is_empty() && n.len() <= 512)
            else {
                issues.push(format!("Titre du manifeste {id} invalide"));
                continue;
            };
            let flags = v.text("StateFlags").and_then(|s| s.parse::<u32>().ok());
            let safe_dir = v.text("installdir").filter(|s| {
                !s.is_empty()
                    && Path::new(s).components().count() == 1
                    && !s.contains(['/', '\\'])
                    && *s != ".."
                    && *s != "."
            });
            let complete = flags.is_some_and(|f| f & 4 != 0)
                && safe_dir.is_some_and(|s| lib.join("steamapps/common").join(s).is_dir());
            let state = if complete {
                "installed"
            } else if flags.is_some() {
                "partial"
            } else {
                "unknown"
            };
            let e = entries.entry(id.into()).or_insert_with(|| Entry {
                app_id: id.into(),
                name: title.into(),
                library_ids: vec![],
                installation: state.into(),
                cover: false,
                hero: false,
                process_dirs: vec![],
            });
            if let Some(d) = safe_dir {
                if let Ok(p) = lib.join("steamapps/common").join(d).canonicalize() {
                    e.process_dirs.push(p)
                }
            }
            e.library_ids.push(format!("Bibliothèque {}", index + 1));
            if complete {
                e.installation = "installed".into()
            }
        }
    }
    for e in entries.values_mut() {
        e.cover = art_path(roots, &e.app_id, "cover").is_some();
        e.hero = art_path(roots, &e.app_id, "hero").is_some()
    }
    Catalog {
        entries: entries.into_values().collect(),
        issues,
        libraries: scanned,
    }
}
pub fn art_path(roots: &[PathBuf], id: &str, kind: &str) -> Option<PathBuf> {
    if !valid_id(id) {
        return None;
    }
    let filename = match kind {
        "cover" => "library_600x900.jpg",
        "hero" => "library_hero.jpg",
        _ => return None,
    };
    for r in roots {
        let base = r.join("appcache/librarycache");
        let Ok(base) = base.canonicalize() else {
            continue;
        };
        let direct = base.join(id).join(filename);
        let old = base.join(format!("{id}_{filename}"));
        for p in [direct, old] {
            if let Ok(p) = p.canonicalize() {
                if p.starts_with(&base)
                    && fs::metadata(&p).is_ok_and(|m| m.is_file() && m.len() <= 8 * 1024 * 1024)
                {
                    return Some(p);
                }
            }
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vdf() {
        let v = parse("// comment\n\"AppState\" { \"name\" \"a\\\"b\\\\c\" }").unwrap();
        assert!(v.object("AppState").is_some());
        for s in [
            "\"x\" {",
            "\"x\" \"unterminated",
            "}",
            "\"x\" \"a\" \"x\" \"b\"",
        ] {
            assert!(parse(s).is_err())
        }
    }
    #[test]
    fn ids() {
        for s in ["0", "01", "1;id", "4294967296", "../440", ""] {
            assert!(!valid_id(s))
        }
        assert!(valid_id("440"));
    }
}
#[cfg(test)]
mod integration_tests {
    use super::*;
    #[test]
    fn multiple_libraries_partial_and_dedup() {
        let base = std::env::temp_dir().join(format!("steam-omarchy-test-{}", std::process::id()));
        let a = base.join("a");
        let b = base.join("b");
        for p in [&a, &b] {
            fs::create_dir_all(p.join("steamapps/common/Test")).unwrap();
            fs::write(p.join("steamapps/appmanifest_440.acf"),"\"AppState\" { \"appid\" \"440\" \"name\" \"Test\" \"StateFlags\" \"4\" \"installdir\" \"Test\" }").unwrap();
        }
        fs::write(
            a.join("steamapps/libraryfolders.vdf"),
            format!(
                "\"libraryfolders\" {{ \"1\" {{ \"path\" \"{}\" }} }}",
                b.display()
            ),
        )
        .unwrap();
        fs::write(
            b.join("steamapps/appmanifest_730.acf"),
            "\"AppState\" { \"appid\" \"730\" \"name\" \"Partial\" \"StateFlags\" \"2\" }",
        )
        .unwrap();
        fs::write(b.join("steamapps/appmanifest_570.acf"), "\"bad").unwrap();
        let c = scan_from(&[a]);
        assert_eq!(c.libraries, 2);
        assert_eq!(c.entries.len(), 2);
        assert_eq!(c.entries[0].library_ids.len(), 2);
        assert_eq!(c.entries[1].installation, "partial");
        assert_eq!(c.issues.len(), 1);
        fs::remove_dir_all(base).unwrap();
    }
}

#[cfg(test)]
mod review_tests {
    use super::*;
    #[test]
    fn partial_scan_reports_limits_structure_and_titles() {
        let base = std::env::temp_dir().join(format!("catalog-review-{}", std::process::id()));
        let apps = base.join("steamapps");
        fs::create_dir_all(&apps).unwrap();
        let list = apps.join("libraryfolders.vdf");
        fs::write(&list, "\"wrong_root\" {}").unwrap();
        assert!(scan_from(std::slice::from_ref(&base))
            .issues
            .iter()
            .any(|s| s.contains("Racine")));
        fs::write(
            &list,
            "\"libraryfolders\" { \"0\" { \"path\" \"relative\" } \"1\" {} }",
        )
        .unwrap();
        assert_eq!(scan_from(std::slice::from_ref(&base)).issues.len(), 2);
        fs::remove_file(list).unwrap();
        fs::write(
            apps.join("appmanifest_440.acf"),
            "\"AppState\" { \"appid\" \"440\" }",
        )
        .unwrap();
        assert!(scan_from(std::slice::from_ref(&base))
            .issues
            .iter()
            .any(|s| s.contains("Titre")));
        for i in 0..20001 {
            fs::write(apps.join(format!("ignored-{i}")), "").unwrap();
        }
        let c = scan_from(std::slice::from_ref(&base));
        assert_eq!(c.libraries, 1);
        assert!(c.issues.iter().any(|s| s.contains("tronquée")));
        let roots: Vec<_> = (0..129).map(|i| base.join(format!("lib-{i}"))).collect();
        for root in &roots {
            fs::create_dir_all(root.join("steamapps")).unwrap();
        }
        let c = scan_from(&roots);
        assert_eq!(c.libraries, 128);
        assert!(c.issues.iter().any(|s| s.contains("128")));
        fs::remove_dir_all(base).unwrap();
    }
}
