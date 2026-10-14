//! Message shapes for both links in [`super`]. Field names deliberately
//! parallel `mitos-session`'s own IPC vocabulary (`AuthOutcome`, `Request`/
//! `Event`) so the two daemons read as one consistent design even though
//! they share no code and no wire format.
use crate::authentication::AuthMethod;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserSummary {
    pub name: String,
    pub display_name: String,
    pub uid: u32,
    pub has_avatar: bool,
    pub locked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionSummary {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthOutcomeWire {
    Success,
    Failure,
    LockedOut { retry_after_secs: u64 },
    Error,
}

impl From<&crate::authentication::AuthOutcome> for AuthOutcomeWire {
    fn from(o: &crate::authentication::AuthOutcome) -> Self {
        use crate::authentication::AuthOutcome as O;
        match o {
            O::Success => AuthOutcomeWire::Success,
            O::Failure { .. } => AuthOutcomeWire::Failure,
            O::LockedOut { retry_after_secs } => AuthOutcomeWire::LockedOut { retry_after_secs: *retry_after_secs },
            O::Error(_) => AuthOutcomeWire::Error,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerAction {
    Suspend,
    Reboot,
    PowerOff,
}

/// Daemon -> greeter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GreeterEvent {
    /// Sent once, right after `GreeterRequest::Ready`.
    Show { users: Vec<UserSummary>, sessions: Vec<SessionSummary>, last_user: Option<String> },
    AuthFeedback { outcome: AuthOutcomeWire },
    /// Purely cosmetic: play the "wrong credential" shake, independent from
    /// `AuthFeedback` so the greeter can react before the backend even
    /// finishes (e.g. an obviously-empty submit).
    Shake,
    PowerCapabilities { can_suspend: bool, can_reboot: bool, can_poweroff: bool },
    PowerActionFailed { message: String },
    /// The daemon is about to tear this greeter down (a session launched
    /// successfully, or it is shutting down) — a clean way to fade out
    /// rather than being killed mid-frame.
    Farewell,
}

/// Greeter -> daemon.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GreeterRequest {
    Ready,
    Authenticate { user: String, method: AuthMethod, secret: Vec<u8> },
    SelectSession { id: String },
    PowerAction(PowerAction),
    /// Liveness ping; the daemon restarts the greeter if these stop.
    Heartbeat,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlRequest {
    Status,
    ListSessions,
    Reload,
    VerifyAuditLog,
    Terminate { session_id: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlResponse {
    Status { version: String, active_sessions: u32, backend: String },
    Sessions(Vec<(u32, String, String)>), // (id, user, seat)
    Ok,
    AuditOk,
    AuditTampered,
    Error(String),
}
