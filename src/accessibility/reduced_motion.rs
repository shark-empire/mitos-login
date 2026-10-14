//! When [`super::Settings::reduced_motion`] is set, the greeter's render
//! loop skips [`crate::greeter::renderer::Canvas::particles`] and the
//! shake/materialize one-shots in [`crate::greeter::animation`], and treats
//! `ambient_pulse` as a fixed `0.5` rather than an oscillation — callers
//! check the flag themselves rather than this module doing anything active.
pub fn static_pulse() -> f32 {
    0.5
}
