//! Console ownership (virtual terminal) and input devices. mitos-login opens
//! these directly as root rather than depending on `libseat`/logind: since
//! the daemon already *is* root, the permission-brokering `libseat` exists
//! to provide for an unprivileged compositor is not needed here (see
//! docs/architecture.md).
pub mod devices;
pub mod input;
pub mod manager;
pub mod seat;

pub use manager::SeatManager;
pub use seat::Seat;
