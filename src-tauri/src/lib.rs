mod config;
pub mod daemon;
mod device;

use config::Config;
use serde::Serialize;
use std::fs;
use std::process::Command;
use std::time::Duration;

#[derive(Serialize)]
struct State {
    config: Config,
    /// Key name per slot, e.g. "KEY_1".
    key_names: Vec<Option<String>>,
    /// Whether each slot's script has the executable bit (false when unset).
    executable: Vec<bool>,
    devices: Vec<String>,
    device_present: bool,
    access_ok: bool,
    daemon_running: bool,
    autostart: bool,
    home: String,
    log_path: String,
}

#[derive(Serialize)]
struct Learned {
    code: u16,
    name: String,
}

#[tauri::command]
fn get_state() -> State {
    let config = Config::load();
    let nodes = device::matching_nodes(&config.device);
    let mut devices: Vec<String> = device::list_nodes().into_iter().map(|n| n.name).collect();
    devices.dedup();
    State {
        key_names: config
            .slots
            .iter()
            .map(|s| s.code.map(device::key_name))
            .collect(),
        executable: config
            .slots
            .iter()
            .map(|s| s.script.as_deref().is_some_and(daemon::is_executable))
            .collect(),
        devices,
        device_present: !nodes.is_empty(),
        access_ok: !nodes.is_empty() && nodes.iter().all(|n| fs::File::open(&n.path).is_ok()),
        daemon_running: daemon::is_running(),
        autostart: config::autostart_path().exists(),
        home: config::home().to_string_lossy().into(),
        log_path: config::log_path().to_string_lossy().into(),
        config,
    }
}

#[tauri::command]
fn save_config(config: Config) -> Result<(), String> {
    let device_changed = Config::load().device != config.device;
    config.save().map_err(|e| e.to_string())?;
    // The daemon reloads mappings live, but only picks a device at startup.
    if device_changed && daemon::is_running() {
        daemon::stop()?;
        daemon::start()?;
    }
    Ok(())
}

#[tauri::command]
async fn learn_key() -> Result<Option<Learned>, String> {
    // The daemon holds an exclusive grab, so pause it while we listen.
    let was_running = daemon::is_running();
    if was_running {
        daemon::stop()?;
    }
    let device = Config::load().device;
    let result = tauri::async_runtime::spawn_blocking(move || {
        device::learn(&device, Duration::from_secs(10))
    })
    .await
    .map_err(|e| e.to_string())?;
    if was_running {
        daemon::start()?;
    }
    Ok(result?.map(|code| Learned {
        code,
        name: device::key_name(code),
    }))
}

#[tauri::command]
fn test_script(path: String) -> Result<(), String> {
    daemon::run_script(&path)
}

#[tauri::command]
fn is_executable(path: String) -> bool {
    daemon::is_executable(&path)
}

#[tauri::command]
fn make_executable(path: String) -> Result<(), String> {
    daemon::make_executable(&path)
}

#[tauri::command]
fn daemon_start() -> Result<(), String> {
    daemon::start()
}

#[tauri::command]
fn daemon_stop() -> Result<(), String> {
    daemon::stop()
}

#[tauri::command]
fn daemon_restart() -> Result<(), String> {
    daemon::stop()?;
    daemon::start()
}

#[tauri::command]
fn set_autostart(enabled: bool) -> Result<(), String> {
    let path = config::autostart_path();
    if !enabled {
        return match fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
            _ => Ok(()),
        };
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let entry = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=easy-vial daemon\n\
         Comment=Runs scripts from macro pad keys\n\
         Exec=\"{}\" --daemon\n\
         NoDisplay=true\n\
         X-GNOME-Autostart-enabled=true\n",
        exe.display()
    );
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::write(path, entry).map_err(|e| e.to_string())
}

/// Installs a udev rule (via a pkexec password prompt) so the pad is readable without sudo.
#[tauri::command]
fn grant_access() -> Result<(), String> {
    let nodes = device::matching_nodes(&Config::load().device);
    let node = nodes.first().ok_or("Device not found, is it plugged in?")?;
    let rule = format!(
        "SUBSYSTEM==\"input\", ATTRS{{idVendor}}==\"{}\", ATTRS{{idProduct}}==\"{}\", MODE=\"0660\", TAG+=\"uaccess\"",
        node.vendor, node.product
    );
    let script = format!(
        "echo '{rule}' > /etc/udev/rules.d/70-easy-vial.rules && udevadm control --reload && udevadm trigger --subsystem-match=input"
    );
    let status = Command::new("pkexec")
        .args(["sh", "-c", &script])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("Access was not granted".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_state,
            save_config,
            learn_key,
            test_script,
            is_executable,
            make_executable,
            daemon_start,
            daemon_stop,
            daemon_restart,
            set_autostart,
            grant_access
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
