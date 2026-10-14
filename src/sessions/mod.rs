//! Launching the user's first process after a successful login. mitos-login's
//! job ends once this process is running — everything from that point on
//! (locking, idling, re-authentication) belongs to `mitos-session`, a
//! separate sibling daemon this project does not depend on directly (see
//! docs/architecture.md for how the two relate).
pub mod environment;
pub mod launcher;
pub mod manager;
pub mod selection;
pub mod session;

pub use manager::SessionManager;
pub use session::Session;
