//! Two independent wire protocols, both hand-rolled (zero dependencies, see
//! Cargo.toml) since the message set is small and stable:
//!
//! * The **greeter link** — an anonymous `socketpair(2)` the daemon creates
//!   before spawning the greeter and inherits into it across `exec` (see
//!   [`crate::sys::spawn`]). No path, no permission bits: only that one
//!   process ever holds the other end.
//! * The **control socket** (`/run/mitos-login/control.sock`) — a real,
//!   listening Unix socket `mitos-loginctl` connects to, gated by
//!   `SO_PEERCRED` rather than a password (see [`permissions`]).
pub mod messages;
pub mod permissions;
pub mod protocol;

pub mod client;
pub mod server;

pub use messages::{ControlRequest, ControlResponse, GreeterEvent, GreeterRequest};
