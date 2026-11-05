use crate::catalog;
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

// Internal seams only: neither command nor proc root is accepted over IPC.
pub fn request(
    id: &str,
    catalog: &catalog::Catalog,
    runner: impl FnOnce(&str) -> Result<(), String>,
) -> Result<String, String> {
    if !catalog::valid_id(id)
        || !catalog
            .entries
            .iter()
            .any(|e| e.app_id == id && e.installation == "installed")
    {
        return Err("AppID absent ou installation incomplète".into());
    }
    runner(&format!("steam://run/{id}"))?;
    Ok("Demande remise à Steam · exécution non confirmée".into())
}

pub fn open(uri: &str) -> Result<(), String> {
    let mut command = Command::new("xdg-open");
    command.arg(uri);
    run_bounded(&mut command, Duration::from_secs(5))
}

fn run_bounded(command: &mut Command, timeout: Duration) -> Result<(), String> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "Gestionnaire Steam indisponible")?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Ok(())
                } else {
                    Err("Gestionnaire Steam en échec".into())
                }
            }
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            result => {
                // Cancel only our handler child, never Steam or any game descendants.
                let _ = child.kill();
                let _ = child.wait();
                return Err(if result.is_err() {
                    "Observation du gestionnaire impossible"
                } else {
                    "Délai du gestionnaire Steam dépassé (5 s) · exécution non confirmée"
                }
                .into());
            }
        }
    }
}

pub fn status(app_id: &str, catalog: &catalog::Catalog, proc_root: &Path) -> Result<bool, String> {
    if !catalog::valid_id(app_id) {
        return Err("AppID invalide".into());
    }
    let Some(game) = catalog.entries.iter().find(|e| e.app_id == app_id) else {
        return Ok(false);
    };
    let Ok(dir) = fs::read_dir(proc_root) else {
        return Ok(false);
    };
    let deadline = Instant::now() + Duration::from_secs(1);
    for e in dir.flatten().take(32768) {
        if Instant::now() >= deadline {
            break;
        }
        if !e
            .file_name()
            .to_string_lossy()
            .bytes()
            .all(|b| b.is_ascii_digit())
        {
            continue;
        }
        let p = e.path();
        let Ok(cmd) = catalog::bounded(&p.join("cmdline"), 65536) else {
            continue;
        };
        let args: Vec<&[u8]> = cmd.split(|b| *b == 0).collect();
        if args
            .windows(2)
            .any(|a| a[0] == b"-applaunch" && a[1] == app_id.as_bytes())
        {
            continue;
        }
        let inside = |path: &std::path::Path| {
            path.canonicalize()
                .ok()
                .is_some_and(|p| game.process_dirs.iter().any(|d| p.starts_with(d)))
        };
        let executable_matches = fs::read_link(p.join("exe"))
            .ok()
            .is_some_and(|exe| inside(&exe));
        if !executable_matches
            && !args.iter().any(|a| {
                std::str::from_utf8(a).ok().is_some_and(|s| {
                    if s.is_empty() {
                        return false;
                    }
                    let path = std::path::Path::new(s);
                    if path.is_absolute() {
                        inside(path)
                    } else {
                        fs::read_link(p.join("cwd"))
                            .ok()
                            .is_some_and(|cwd| inside(&cwd.join(path)))
                    }
                })
            })
        {
            continue;
        }
        let Ok(env) = catalog::bounded(&p.join("environ"), 262144) else {
            continue;
        };
        if matching_app_id(&env, app_id) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn matching_app_id(env: &[u8], app_id: &str) -> bool {
    env.split(|b| *b == 0).any(|s| {
        s == format!("SteamAppId={app_id}").as_bytes()
            || s == format!("SteamGameId={app_id}").as_bytes()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    #[test]
    fn synthetic_backend_flow() {
        let root = std::env::temp_dir().join(format!("launch-fixture-{}", std::process::id()));
        let steam = root.join("steam");
        let game = steam.join("steamapps/common/Fixture");
        let proc = root.join("proc");
        fs::create_dir_all(&game).unwrap();
        fs::create_dir_all(&proc).unwrap();
        fs::write(game.join("game.exe"), b"synthetic executable").unwrap();
        fs::write(steam.join("steamapps/appmanifest_440.acf"),
            r#""AppState" { "appid" "440" "name" "Fixture" "StateFlags" "4" "installdir" "Fixture" }"#).unwrap();
        let mut catalog = catalog::scan_from(&[steam]);
        assert_eq!(catalog.entries.len(), 1);
        for id in ["", "0440", "440;id", "4400", "4294967296"] {
            assert!(request(id, &catalog, |_| panic!("must not run")).is_err());
        }
        catalog.entries[0].installation = "partial".into();
        assert!(request("440", &catalog, |_| panic!("must not install")).is_err());
        catalog.entries[0].installation = "installed".into();
        assert!(!status("440", &catalog, &proc).unwrap());
        // Initially closed simulation: handler creates a background-process record.
        let start = || {
            let pid = proc.join("100");
            fs::create_dir_all(&pid).unwrap();
            fs::write(pid.join("cmdline"), b"wine\0./game.exe\0").unwrap();
            fs::write(
                pid.join("environ"),
                b"SECRET=not exported\0SteamAppId=440\0",
            )
            .unwrap();
            symlink(&game, pid.join("cwd")).unwrap();
        };
        let message = request("440", &catalog, |uri| {
            assert_eq!(uri, "steam://run/440");
            start();
            Ok(())
        })
        .unwrap();
        assert!(message.contains("non confirmée"));
        assert!(status("440", &catalog, &proc).unwrap());
        // Already running simulation: observation cannot establish a new launch.
        request("440", &catalog, |_| Ok(())).unwrap();
        assert!(status("440", &catalog, &proc).unwrap());
        let pid = proc.join("100");
        fs::write(pid.join("environ"), b"SteamAppId=4400\0SteamGameId=4401\0").unwrap();
        assert!(!status("440", &catalog, &proc).unwrap());
        fs::write(pid.join("environ"), b"SteamGameId=440\0").unwrap();
        fs::write(pid.join("cmdline"), b"wine\0C:\\Fixture\\game.exe\0").unwrap();
        assert!(!status("440", &catalog, &proc).unwrap()); // No drive mapping guesses.
        symlink(game.join("game.exe"), pid.join("exe")).unwrap();
        assert!(status("440", &catalog, &proc).unwrap());
        fs::write(pid.join("cmdline"), b"steam\0-applaunch\x00440\0").unwrap();
        assert!(!status("440", &catalog, &proc).unwrap());
        fs::remove_dir_all(pid).unwrap();
        assert!(request("440", &catalog, |_| Err("failure".into())).is_err());
        request("440", &catalog, |_| Ok(())).unwrap(); // Handoff with no execution.
        assert!(!status("440", &catalog, &proc).unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn handler_exit_failure_and_timeout() {
        assert!(run_bounded(&mut Command::new("/usr/bin/true"), Duration::from_secs(1)).is_ok());
        assert!(run_bounded(&mut Command::new("/usr/bin/false"), Duration::from_secs(1)).is_err());
        assert!(run_bounded(
            &mut Command::new("/nonexistent-launch-fixture"),
            Duration::from_secs(1)
        )
        .is_err());
        let start = Instant::now();
        assert!(run_bounded(
            Command::new("/usr/bin/sleep").arg("30"),
            Duration::from_millis(40)
        )
        .unwrap_err()
        .contains("Délai"));
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}
