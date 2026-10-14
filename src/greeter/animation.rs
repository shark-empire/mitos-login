//! Small time-based easing helpers, in the same spirit as mitos-gui's own
//! `animation.rs`: continuous functions of wall-clock time need no stored
//! per-frame state, and one-shot transitions are tracked as a start time
//! plus a duration.
use std::time::{Duration, Instant};

/// A slow, continuous breathing glow — used for focus rings and the auth
/// panel outline. Returns `0.0..1.0`.
pub fn ambient_pulse(t: f32, period_secs: f32) -> f32 {
    0.5 + 0.5 * (t * std::f32::consts::TAU / period_secs).sin()
}

pub fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    let t = t.clamp(0.0, 1.0) - 1.0;
    1.0 + c3 * t * t * t + c1 * t * t
}

pub fn ease_out_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

/// A one-shot animation with a start time and duration; `progress()` is
/// `0.0` before it starts (never, since it is only ever constructed at the
/// moment it starts) through `1.0` once finished, and `is_done()` says when
/// it is safe to drop.
#[derive(Clone, Copy, Debug)]
pub struct OneShot {
    start: Instant,
    duration: Duration,
}

impl OneShot {
    pub fn start_now(duration: Duration) -> OneShot {
        OneShot { start: Instant::now(), duration }
    }
    /// Rebuilds a `OneShot` from a start time recorded earlier (e.g. a
    /// `shake_started: Option<Instant>` field) plus the duration the caller
    /// has always used for it.
    pub fn from_start(start: Instant, duration: Duration) -> OneShot {
        OneShot { start, duration }
    }
    pub fn progress(&self) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }
        (self.start.elapsed().as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0)
    }
    pub fn is_done(&self) -> bool {
        self.start.elapsed() >= self.duration
    }
}

/// A short, decaying left-right shake for a rejected credential, expressed
/// as a horizontal pixel offset. Reduced-motion mode should skip calling
/// this and just flash color instead.
pub fn shake_offset(t: &OneShot, amplitude_px: f32) -> f32 {
    let p = t.progress();
    let decay = 1.0 - p;
    (p * std::f32::consts::TAU * 4.0).sin() * amplitude_px * decay * decay
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulse_stays_in_unit_range() {
        for i in 0..100 {
            let v = ambient_pulse(i as f32 * 0.13, 4.0);
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn ease_out_quad_endpoints() {
        assert!((ease_out_quad(0.0) - 0.0).abs() < 1e-6);
        assert!((ease_out_quad(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn oneshot_progress_reaches_one() {
        let o = OneShot { start: Instant::now() - Duration::from_secs(10), duration: Duration::from_secs(1) };
        assert_eq!(o.progress(), 1.0);
        assert!(o.is_done());
    }
}
