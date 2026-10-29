use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const SLOT_COUNT: usize = 9;
pub const DEFAULT_DEVICE: &str = "Pabile P9v2 Macro Pad";

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Slot {
    /// evdev key code this pad key sends (learned from the device).
    pub code: Option<u16>,
    /// Script to run when the key is pressed.
    pub script: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Every input device whose name starts with this is read (and grabbed) by the daemon.
    pub device: String,
    pub slots: Vec<Slot>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device: DEFAULT_DEVICE.into(),
            slots: vec![Slot::default(); SLOT_COUNT],
        }
    }
}

pub fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".into()))
}

fn config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home().join(".config"))
}

pub fn config_path() -> PathBuf {
    config_dir().join("easy-vial").join("config.json")
}

pub fn autostart_path() -> PathBuf {
    config_dir().join("autostart").join("easy-vial.desktop")
}

pub fn log_path() -> PathBuf {
    home().join(".cache").join("easy-vial").join("daemon.log")
}

pub fn pid_path() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("easy-vial.pid")
}

impl Config {
    pub fn load() -> Self {
        let mut cfg: Config = fs::read_to_string(config_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        cfg.slots.resize(SLOT_COUNT, Slot::default());
        cfg
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = config_path();
        fs::create_dir_all(path.parent().unwrap())?;
        // Write then rename so the daemon never reads a half-written file.
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        fs::rename(tmp, path)
    }

    pub fn script_for(&self, code: u16) -> Option<&str> {
        self.slots
            .iter()
            .find(|s| s.code == Some(code))
            .and_then(|s| s.script.as_deref())
    }
}
