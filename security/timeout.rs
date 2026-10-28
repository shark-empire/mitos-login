//! Small deadline helper shared by the daemon's poll loop.
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct Deadline {
    at: Instant,
}

impl Deadline {
    pub fn after(d: Duration) -> Deadline {
        Deadline { at: Instant::now() + d }
    }

    pub fn is_past(&self) -> bool {
        Instant::now() >= self.at
    }

    /// Milliseconds remaining, clamped to `[0, i32::MAX]`, ready to hand to
    /// `poll(2)`.
    pub fn remaining_ms(&self) -> i32 {
        let now = Instant::now();
        if now >= self.at {
            0
        } else {
            (self.at - now).as_millis().min(i32::MAX as u128) as i32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_duration_is_immediately_past() {
        let d = Deadline::after(Duration::ZERO);
        std::thread::sleep(Duration::from_millis(1));
        assert!(d.is_past());
        assert_eq!(d.remaining_ms(), 0);
    }

    #[test]
    fn future_deadline_has_positive_remaining() {
        let d = Deadline::after(Duration::from_secs(10));
        assert!(!d.is_past());
        assert!(d.remaining_ms() > 5000);
    }
}
