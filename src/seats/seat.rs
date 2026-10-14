use std::fs::File;

/// Ownership of one virtual terminal: the open `/dev/ttyN` (kept open only
/// to hold VT ioctls and lock switching; not read from or written to) and
/// its number.
pub struct Seat {
    pub vt_number: u32,
    pub tty: File,
    pub name: String,
}

impl Seat {
    pub fn tty_name(&self) -> String {
        format!("tty{}", self.vt_number)
    }
}
