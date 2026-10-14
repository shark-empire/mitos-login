//! Enumerating `/dev/input/event*`, classified purely by reported
//! capability bits (never by name — see [`crate::sys::evdev_capabilities`]).
use crate::errors::Result;
use crate::sys;
use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::PathBuf;

pub struct InputDevice {
    pub file: File,
    pub path: PathBuf,
    pub name: String,
    pub is_keyboard: bool,
    pub is_pointer: bool,
}

/// Opens and grabs (`EVIOCGRAB`) every keyboard/pointer-capable device under
/// `/dev/input`. Exclusive grab means key/button events stop reaching
/// anything else (a text VT, another process) while the greeter owns the
/// seat — the same reason a real compositor grabs its input devices.
pub fn open_all(dir: &std::path::Path) -> Result<Vec<InputDevice>> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(e.into()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) if n.starts_with("event") => n.to_string(),
            _ => continue,
        };
        let file = match OpenOptions::new().read(true).write(false).open(&path) {
            Ok(f) => f,
            Err(_) => continue, // a device we can't open (permissions, race) is simply skipped
        };
        let fd = file.as_raw_fd();
        let caps = sys::evdev_capabilities(fd);
        if !caps.has_letter_keys && !caps.has_mouse_buttons && !caps.has_rel {
            continue; // not a keyboard or pointer (e.g. a lid switch, a webcam's button)
        }
        let _ = sys::evdev_grab(fd, true);
        let device_name = sys::evdev_name(fd).unwrap_or(name);
        out.push(InputDevice {
            file,
            path,
            name: device_name,
            is_keyboard: caps.has_letter_keys,
            is_pointer: caps.has_rel || caps.has_mouse_buttons,
        });
    }
    Ok(out)
}
