mod catalog;
mod launch;
mod theme;
mod theme_watch;
use base64::Engine;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::{fs, path::PathBuf};
use tauri::{Emitter, Manager};
#[tauri::command]
fn scan_catalog() -> catalog::Catalog {
    catalog::scan_from(&catalog::roots())
}
#[tauri::command]
fn read_theme() -> theme::Theme {
    theme_watch::snapshot()
}
#[tauri::command]
fn artwork(app_id: String, kind: String) -> Result<String, String> {
    let p = catalog::art_path(&catalog::roots(), &app_id, &kind).ok_or("Illustration absente")?;
    let b = catalog::bounded(&p, 8 * 1024 * 1024)?;
    if !b.starts_with(&[0xff, 0xd8, 0xff]) {
        return Err("Type image refusé".into());
    }
    Ok(format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(b)
    ))
}
fn prefs_path() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| catalog::home().join(".config"))
        .join("steam-omarchy/preferences.json")
}
#[tauri::command]
fn load_preferences() -> Result<serde_json::Value, String> {
    read_preferences(&prefs_path())
}
fn read_preferences(p: &std::path::Path) -> Result<serde_json::Value, String> {
    match fs::symlink_metadata(p) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(serde_json::json!({})),
        Err(_) => return Err("Métadonnées préférences".into()),
        Ok(_) => {}
    }
    let b = catalog::bounded(p, 65536)?;
    serde_json::from_slice(&b).map_err(|_| "JSON préférences invalide".into())
}
#[tauri::command]
fn save_preferences(value: serde_json::Value) -> Result<(), String> {
    write_preferences(&prefs_path(), &value)
}
fn write_preferences(p: &std::path::Path, value: &serde_json::Value) -> Result<(), String> {
    let b = serde_json::to_vec(value).map_err(|_| "JSON")?;
    if b.len() > 65536 {
        return Err("Préférences trop grandes".into());
    }
    fs::create_dir_all(p.parent().unwrap()).map_err(|_| "Dossier préférences")?;
    let tmp = p.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)
        .map_err(|_| "Création préférences temporaire")?;
    let result = file
        .write_all(&b)
        .and_then(|_| file.sync_all())
        .and_then(|_| fs::rename(&tmp, p));
    drop(file);
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result.map_err(|_| "Sauvegarde préférences".into())
}
#[tauri::command]
async fn request_launch(app_id: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        launch::request(&app_id, &scan_catalog(), launch::open)
    })
    .await
    .map_err(|_| "Traitement Steam interrompu")?
}
// Only exact destinations shipped by the installed official Steam desktop launcher.
fn steam_destination(action: &str, app_id: Option<&str>) -> Result<String, String> {
    if let Some(id) = app_id {
        if action == "store" && catalog::valid_id(id) {
            return Ok(format!(
                "steam://openurl/https://store.steampowered.com/app/{id}"
            ));
        }
        return Err("Destination Steam refusée".into());
    }
    match action {
        "settings" | "downloads" => Ok("steam://open/settings".into()),
        "friends" => Ok("steam://open/friends".into()),
        "library" => Ok("steam://open/games".into()),
        "store" => Ok("steam://store".into()),
        "community" => Ok("steam://url/CommunityHome/".into()),
        _ => Err("Destination Steam refusée".into()),
    }
}
#[tauri::command]
async fn open_steam(action: String, app_id: Option<String>) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let uri = steam_destination(&action, app_id.as_deref())?;
        if let Some(id) = app_id {
            if !scan_catalog().entries.iter().any(|e| e.app_id == id) {
                return Err("AppID hors catalogue".into());
            }
        }
        launch::open(&uri)?;
        Ok("Ouverture remise à Steam · affichage non confirmé".into())
    })
    .await
    .map_err(|_| "Traitement Steam interrompu")?
}

#[test]
fn destinations_are_closed() {
    assert!(steam_destination("steam://run/440", None).is_err());
    assert!(steam_destination("store", Some("440;id")).is_err());
    for action in [
        "settings",
        "downloads",
        "friends",
        "library",
        "store",
        "community",
    ] {
        assert!(steam_destination(action, None)
            .unwrap()
            .starts_with("steam://"));
    }
    for action in [
        "steam://run/440",
        "install",
        "run",
        "/bin/sh",
        "settings?x=1",
        "",
    ] {
        assert!(steam_destination(action, None).is_err());
    }
    for id in ["0", "../440", "440?x=1", "440;id", "https://x", ""] {
        assert!(steam_destination("store", Some(id)).is_err());
    }
    assert_eq!(
        steam_destination("store", Some("440")).unwrap(),
        "steam://openurl/https://store.steampowered.com/app/440"
    );
    assert!(steam_destination("settings", Some("440")).is_err());
}
#[tauri::command]
async fn launch_status(app_id: String) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        launch::status(&app_id, &scan_catalog(), std::path::Path::new("/proc"))
    })
    .await
    .map_err(|_| "Observation Steam interrompue")?
}
// Test-only fixed telemetry and close; no paths or evaluation accepted over IPC.
#[cfg(feature = "native-qa")]
#[tauri::command]
fn native_qa_report(value: serde_json::Value) -> Result<(), String> {
    let serialized = validate_native_qa_report(&value)?;
    println!("NATIVE_QA {}", serialized);
    Ok(())
}
#[cfg(feature = "native-qa")]
fn validate_native_qa_report(value: &serde_json::Value) -> Result<String, String> {
    if !value.is_object() {
        return Err("QA report must be an object".into());
    }
    let records = value
        .get("records")
        .and_then(serde_json::Value::as_array)
        .ok_or("QA report records must be an array")?;
    if records.len() > 64 {
        return Err("QA report exceeds 64 records".into());
    }
    let serialized = serde_json::to_string(value).map_err(|e| e.to_string())?;
    if serialized.len() > 65536 {
        return Err("QA report exceeds 64 KiB".into());
    }
    Ok(serialized)
}
#[cfg(feature = "native-qa")]
#[tauri::command]
fn native_qa_close(window: tauri::WebviewWindow) {
    let _ = window.close();
}
fn main() {
    // WebKitGTK DMA-BUF fails on the host NVIDIA/Wayland stack; override remains possible.
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
    if std::env::args().any(|a| a == "--theme-watch-probe") {
        let mut worker =
            theme_watch::Worker::start(|t| println!("{}", serde_json::to_string(&t).unwrap()));
        // stdin line or EOF requests cooperative shutdown, without a native window.
        let _ = std::io::stdin().read_line(&mut String::new());
        assert!(
            worker.shutdown(std::time::Duration::from_millis(500)),
            "Theme worker shutdown exceeded budget"
        );
        return;
    }
    if std::env::args().any(|a| a == "--diagnose") {
        println!(
            "{}",
            serde_json::json!({"catalog":scan_catalog(),"theme":theme::read()})
        );
        return;
    }
    let builder = tauri::Builder::default();
    #[cfg(feature = "native-qa")]
    let builder = builder.append_invoke_initialization_script(include_str!("native_qa.js"));
    let builder = builder.setup(|app| {
        let handle = app.app_handle().clone();
        app.manage(std::sync::Mutex::new(theme_watch::Worker::start(
            move |t| {
                let _ = handle.emit("theme-changed", t);
            },
        )));
        Ok(())
    });
    #[cfg(not(feature = "native-qa"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        scan_catalog,
        read_theme,
        artwork,
        load_preferences,
        save_preferences,
        request_launch,
        launch_status,
        open_steam
    ]);
    #[cfg(feature = "native-qa")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        scan_catalog,
        read_theme,
        artwork,
        load_preferences,
        save_preferences,
        request_launch,
        launch_status,
        open_steam,
        native_qa_report,
        native_qa_close
    ]);
    builder
        .build(tauri::generate_context!())
        .expect("Impossible de démarrer Steam Omarchy")
        .run(|app, event| {
            if matches!(
                event,
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
            ) {
                let state = app.state::<std::sync::Mutex<theme_watch::Worker>>();
                let mut worker = state.lock().unwrap();
                worker.request_stop();
                if matches!(event, tauri::RunEvent::Exit)
                    && !worker.shutdown(std::time::Duration::from_millis(500))
                {
                    eprintln!("Theme worker cleanup exceeded 500 ms budget");
                }
            }
        });
}

#[cfg(test)]
mod preference_tests {
    use super::*;
    #[test]
    fn preserve_bad_files_and_retry_failed_save() {
        let dir = std::env::temp_dir().join(format!("prefs-regression-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("preferences.json");
        assert_eq!(read_preferences(&p).unwrap(), serde_json::json!({}));
        for bytes in [b"{broken".to_vec(), vec![b' '; 65537], vec![255]] {
            fs::write(&p, &bytes).unwrap();
            assert!(read_preferences(&p).is_err());
            assert_eq!(fs::read(&p).unwrap(), bytes);
        }
        fs::remove_file(&p).unwrap();
        fs::create_dir(&p).unwrap(); // rename fails after temporary creation
        assert!(write_preferences(&p, &serde_json::json!({})).is_err());
        assert!(!p
            .with_extension(format!("{}.tmp", std::process::id()))
            .exists());
        fs::remove_dir(&p).unwrap();
        write_preferences(&p, &serde_json::json!({"favorites":["440"]})).unwrap();
        assert_eq!(read_preferences(&p).unwrap()["favorites"][0], "440");
        fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(all(test, feature = "native-qa"))]
mod native_qa_tests {
    use super::validate_native_qa_report;
    use serde_json::json;
    #[test]
    fn report_bounds() {
        assert!(validate_native_qa_report(&json!({"records": []})).is_ok());
        assert!(validate_native_qa_report(&json!([])).is_err());
        assert!(validate_native_qa_report(&json!({})).is_err());
        assert!(validate_native_qa_report(&json!({"records": vec![0; 64]})).is_ok());
        assert!(validate_native_qa_report(&json!({"records": vec![0; 65]})).is_err());
        assert!(
            validate_native_qa_report(&json!({"records": [], "text": "x".repeat(65536)})).is_err()
        );
    }
}
