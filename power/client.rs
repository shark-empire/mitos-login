//! Maps [`crate::ipc::messages::PowerAction`] (what the greeter asked for)
//! onto this module's functions, applying the one policy rule that matters
//! before any session exists: every action here runs with no user
//! authenticated, so it must be available to anyone standing at the
//! keyboard — exactly like every other login screen's power button — and
//! carries no further authorization of its own.
use super::Capabilities;
use crate::errors::Result;
use crate::ipc::messages::PowerAction;

pub fn dispatch(action: PowerAction, caps: &Capabilities) -> Result<()> {
    match action {
        PowerAction::Suspend if caps.can_suspend => super::suspend(),
        PowerAction::Reboot if caps.can_reboot => super::reboot(),
        PowerAction::PowerOff if caps.can_poweroff => super::poweroff(),
        _ => Err(crate::errors::Error::Unsupported("that power action is not available")),
    }
}
