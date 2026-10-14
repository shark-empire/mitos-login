//! A PIN field is visually identical to the password field (same recessed
//! inset, same dot feedback) — the only difference is semantic (which
//! `AuthMethod` gets sent), so this module intentionally delegates rather
//! than duplicating the drawing code.
pub use super::password_input::draw;
