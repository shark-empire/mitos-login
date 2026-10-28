//! A tiny token-bucket limiter, used to cap how fast a single connection can
//! retry a request over IPC — independent from [`super::lockout`], which is
//! the actual authentication-attempt policy. This just stops someone from
//! busy-looping a socket.
use std::time::{Duration, Instant};

pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_per_sec: f64,
    last: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_per_sec: f64) -> TokenBucket {
        TokenBucket { capacity: capacity as f64, tokens: capacity as f64, refill_per_sec, last: Instant::now() }
    }

    fn refill(&mut self, now: Instant) {
        let elapsed = now.saturating_duration_since(self.last).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        self.last = now;
    }

    /// Consumes one token if available. `false` means "reject this request".
    pub fn take(&mut self) -> bool {
        self.take_at(Instant::now())
    }

    pub fn take_at(&mut self, now: Instant) -> bool {
        self.refill(now);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausts_then_refills() {
        let mut b = TokenBucket::new(3, 1.0);
        let t0 = Instant::now();
        assert!(b.take_at(t0));
        assert!(b.take_at(t0));
        assert!(b.take_at(t0));
        assert!(!b.take_at(t0), "bucket should be empty");
        assert!(b.take_at(t0 + Duration::from_secs(2)), "should have refilled by now");
    }
}
