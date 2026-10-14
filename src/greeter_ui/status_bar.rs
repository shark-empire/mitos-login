use crate::greeter::renderer::{Canvas, GlassSpec, TextAlign};
use crate::theme::Theme;

/// The top bar: MITOS wordmark on the left, language/keyboard/power buttons
/// on the right.
pub fn draw(canvas: &mut Canvas, theme: &Theme, t: f32, pulse: f32) {
    let l = &theme.layout;
    let cx = canvas.w as f32 / 2.0;
    let cy = l.status_bar_margin + l.status_bar_height / 2.0;
    let mut spec = GlassSpec::new(cx, cy, canvas.w as f32 - l.status_bar_margin * 2.0, l.status_bar_height, l.status_bar_height / 2.0);
    spec.tint = 0.07;
    spec.glow = 0.35;
    spec.glow_radius = 14.0 * l.scale;
    spec.border = 1.0;
    spec.ring_phase = t * 0.3;
    spec.pulse = pulse;
    canvas.glass(spec);
    canvas.text(
        "MITOS",
        l.status_bar_margin * 2.4,
        cy + 4.0 * l.scale,
        12.0 * l.scale,
        theme.text,
        TextAlign::Left,
        0.6,
        1.2,
    );
}
