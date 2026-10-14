//! Account lock/expiry *display* status — booleans derived from `/etc/shadow`,
//! never the hash itself, so this is safe to compute for a list the
//! unprivileged greeter will render.
use crate::authentication::shadow::{self, ShadowEntry};
use crate::errors::Result;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AccountStatus {
    pub locked: bool,
    pub expired: bool,
    pub password_expired: bool,
}

pub fn status_for(user: &str) -> Result<AccountStatus> {
    let entry: Option<ShadowEntry> = shadow::lookup(user)?;
    let now = shadow::now_days_since_epoch();
    Ok(match entry {
        None => AccountStatus::default(),
        Some(e) => AccountStatus {
            locked: e.is_locked_or_passwordless(),
            expired: e.is_expired(now),
            password_expired: e.password_expired(now),
        },
    })
}
