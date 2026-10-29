//! The always-running listener: `easy-vial --daemon`.

use crate::config::{self, Config};
use crate::device;
use std::fs::{self, OpenOptions};
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};

fn read_pid() -> Option<u32> {
    fs::read_to_string(config::pid_path()).ok()?.trim().parse().ok()
}

pub fn running_pid() -> Option<u32> {
    let pid = read_pid()?;
    let cmdline = fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    let cmdline = String::from_utf8_lossy(&cmdline);
    (cmdline.contains("easy-vial") && cmdline.contains("--daemon")).then_some(pid)
}

pub fn is_running() -> bool {
    running_pid().is_some()
}

/// Starts the daemon detached from the GUI, so it keeps running after the window closes.
pub fn start() -> Result<(), String> {
    if is_running() {
        return Ok(());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let log_path = config::log_path();
    fs::create_dir_all(log_path.parent().unwrap()).map_err(|e| e.to_string())?;
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| e.to_string())?;
    Command::new("setsid")
        .arg("-f")
        .arg(exe)
        .arg("--daemon")
        .stdin(Stdio::null())
        .stdout(log.try_clone().map_err(|e| e.to_string())?)
        .stderr(log)
        .status()
        .map_err(|e| e.to_string())?;
    for _ in 0..20 {
        if is_running() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(format!("Daemon did not start, see {}", log_path.display()))
}

pub fn stop() -> Result<(), String> {
    let Some(pid) = running_pid() else {
        return Ok(());
    };
    Command::new("kill")
        .arg(pid.to_string())
        .status()
        .map_err(|e| e.to_string())?;
    for _ in 0..40 {
        if !is_running() {
            let _ = fs::remove_file(config::pid_path());
            // Give the kernel a moment to release the device grab.
            thread::sleep(Duration::from_millis(50));
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err("Daemon did not stop".into())
}

pub fn is_executable(path: &str) -> bool {
    fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

/// Same as `chmod +x` (adds execute wherever read is allowed).
pub fn make_executable(path: &str) -> Result<(), String> {
    let mut perms = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
    let mode = perms.mode();
    perms.set_mode(mode | ((mode & 0o444) >> 2));
    fs::set_permissions(path, perms).map_err(|e| e.to_string())
}

pub fn run_script(path: &str) -> Result<(), String> {
    if !fs::metadata(path).is_ok_and(|m| m.is_file()) {
        return Err(format!("{path} does not exist"));
    }
    // Executable scripts run directly so their shebang is honored; others fall back to bash.
    let mut cmd = Command::new("setsid");
    cmd.arg("-f");
    if is_executable(path) {
        cmd.arg(path);
    } else {
        cmd.args(["bash", path]);
    }
    cmd
        .current_dir(config::home())
        .stdin(Stdio::null())
        .status()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn log(msg: impl AsRef<str>) {
    eprintln!("[easy-vial] {}", msg.as_ref());
}

fn config_mtime() -> Option<SystemTime> {
    fs::metadata(config::config_path()).ok()?.modified().ok()
}

pub fn run() {
    if let Some(pid) = running_pid() {
        log(format!("already running (pid {pid})"));
        return;
    }
    let _ = fs::write(config::pid_path(), std::process::id().to_string());

    let cfg = Arc::new(RwLock::new(Config::load()));

    // Pick up config saves from the GUI without a restart.
    {
        let cfg = cfg.clone();
        thread::spawn(move || {
            let mut last = config_mtime();
            loop {
                thread::sleep(Duration::from_millis(500));
                let now = config_mtime();
                if now != last {
                    last = now;
                    *cfg.write().unwrap() = Config::load();
                    log("config reloaded");
                }
            }
        });
    }

    log("started");
    loop {
        let name = cfg.read().unwrap().device.clone();
        let devices = device::open_matching(&name);
        if devices.is_empty() {
            // Not plugged in (or no access); check again shortly.
            thread::sleep(Duration::from_secs(2));
            continue;
        }
        log(format!("listening on {} interface(s) of \"{name}\"", devices.len()));
        let handles: Vec<_> = devices
            .into_iter()
            .map(|mut dev| {
                let cfg = cfg.clone();
                thread::spawn(move || {
                    // Exclusive: pad keys no longer type into apps, they only run scripts.
                    if let Err(e) = dev.grab() {
                        log(format!("grab failed: {e}"));
                    }
                    loop {
                        let events = match dev.fetch_events() {
                            Ok(events) => events.collect::<Vec<_>>(),
                            Err(e) => {
                                log(format!("device lost: {e}"));
                                return;
                            }
                        };
                        for code in events.iter().filter_map(device::key_down) {
                            let script = cfg.read().unwrap().script_for(code).map(String::from);
                            match script {
                                Some(script) => {
                                    log(format!("{} -> {script}", device::key_name(code)));
                                    if let Err(e) = run_script(&script) {
                                        log(format!("failed to run {script}: {e}"));
                                    }
                                }
                                None => log(format!("{} (unmapped)", device::key_name(code))),
                            }
                        }
                    }
                })
            })
            .collect();
        for h in handles {
            let _ = h.join();
        }
        thread::sleep(Duration::from_secs(1));
    }
}
