use evdev::{Device, EventType, KeyCode};
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Codes below BTN_MISC are keyboard keys; above are mouse/joystick buttons.
const BTN_MISC: u16 = 0x100;

/// An /dev/input/eventN node as seen through sysfs (works even without read access).
#[derive(Clone, Debug)]
pub struct Node {
    pub path: PathBuf,
    pub name: String,
    pub vendor: String,
    pub product: String,
}

fn read_trim(path: PathBuf) -> String {
    fs::read_to_string(path).map(|s| s.trim().to_string()).unwrap_or_default()
}

pub fn list_nodes() -> Vec<Node> {
    let Ok(entries) = fs::read_dir("/sys/class/input") else {
        return vec![];
    };
    let mut nodes: Vec<Node> = entries
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("event"))
        .map(|e| {
            let dev = e.path().join("device");
            Node {
                path: PathBuf::from("/dev/input").join(e.file_name()),
                name: read_trim(dev.join("name")),
                vendor: read_trim(dev.join("id/vendor")),
                product: read_trim(dev.join("id/product")),
            }
        })
        .collect();
    nodes.sort_by(|a, b| a.name.cmp(&b.name));
    nodes
}

pub fn matching_nodes(prefix: &str) -> Vec<Node> {
    list_nodes()
        .into_iter()
        .filter(|n| !prefix.is_empty() && n.name.starts_with(prefix))
        .collect()
}

fn has_keyboard_keys(dev: &Device) -> bool {
    dev.supported_keys()
        .is_some_and(|keys| keys.iter().any(|k| k.code() < BTN_MISC))
}

/// Opens the keyboard-like interfaces of the named device (skips e.g. its mouse interface).
pub fn open_matching(prefix: &str) -> Vec<Device> {
    matching_nodes(prefix)
        .into_iter()
        .filter_map(|n| Device::open(&n.path).ok())
        .filter(has_keyboard_keys)
        .collect()
}

pub fn key_name(code: u16) -> String {
    format!("{:?}", KeyCode::new(code))
}

pub fn key_down(ev: &evdev::InputEvent) -> Option<u16> {
    (ev.event_type() == EventType::KEY && ev.value() == 1).then(|| ev.code())
}

/// Waits for the next key press on the device and returns its code.
pub fn learn(prefix: &str, timeout: Duration) -> Result<Option<u16>, String> {
    let mut devices = open_matching(prefix);
    if devices.is_empty() {
        return Err(format!("No readable device named \"{prefix}\""));
    }
    for d in &mut devices {
        d.set_nonblocking(true).map_err(|e| e.to_string())?;
        // Grab so the press doesn't also type into the focused window.
        let _ = d.grab();
    }
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        for d in &mut devices {
            match d.fetch_events() {
                Ok(events) => {
                    if let Some(code) = events.into_iter().find_map(|ev| key_down(&ev)) {
                        return Ok(Some(code));
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok(None)
}
