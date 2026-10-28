//! Suspend-to-RAM via the kernel's `/sys/power/state` interface — the same
//! mechanism every Linux power daemon (including systemd-logind) ultimately
//! uses; no extra privilege beyond root (already held) is needed.
use crate::errors::{Error, Result};
use std::io::Write;

const STATE_PATH: &str = "/sys/power/state";

pub fn is_supported() -> bool {
    std::fs::read_to_string(STATE_PATH).map(|s| s.split_whitespace().any(|w| w == "mem")).unwrap_or(false)
}

pub fn run() -> Result<()> {
    if !is_supported() {
        return Err(Error::Unsupported("suspend-to-RAM is not available on this system"));
    }
    let mut f = std::fs::OpenOptions::new().write(true).open(STATE_PATH)?;
    f.write_all(b"mem")?;
    Ok(())
}
