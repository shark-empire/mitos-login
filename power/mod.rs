//! Suspend/reboot/power-off, reachable from the greeter's power menu before
//! any login. `reboot`/`poweroff` call the kernel directly
//! ([`crate::sys::reboot_now`]/[`crate::sys::poweroff_now`]) — this project
//! has no dependency on a sibling init/supervisor, matching mitos-session's
//! own `power::DirectBackend`. If mitosOS's process supervisor is present, a
//! future `SupervisedBackend` here (mirroring mitos-session's own) could
//! route through it instead; not implemented, since this project has no
//! access to that daemon's control protocol to build and verify against.
pub mod client;
pub mod hibernate;
pub mod reboot;
pub mod shutdown;
pub mod suspend;

use crate::errors::Result;

#[derive(Clone, Copy, Debug, Default)]
pub struct Capabilities {
    pub can_suspend: bool,
    pub can_reboot: bool,
    pub can_poweroff: bool,
}

/// What this system can actually do right now, probed cheaply and safely
/// (read-only checks, no side effects).
pub fn detect_capabilities() -> Capabilities {
    Capabilities {
        can_suspend: suspend::is_supported(),
        can_reboot: true,
        can_poweroff: true,
    }
}

pub fn reboot() -> Result<()> {
    reboot::run()
}

pub fn poweroff() -> Result<()> {
    shutdown::run()
}

pub fn suspend() -> Result<()> {
    suspend::run()
}
