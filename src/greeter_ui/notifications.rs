//! A single transient toast (used today for "config reloaded"-style
//! feedback and nothing else — the login screen otherwise has no
//! notification source of its own).
use crate::greeter::renderer::{Canvas, GlassSpec, TextAlign};
use crate::theme::Theme;

pub fn draw(canvas: &mut Canvas, theme: &Theme, message: &str, age_secs: f32) {
    if age_secs > 4.0 {
        return;
    }
    let l = &theme.layout;
    let fade = (1.0 - (age_secs / 4.0)).clamp(0.0, 1.0);
    let cx = canvas.w as f32 / 2.0;
    let cy = l.status_bar_margin + l.status_bar_height + 30.0 * l.scale;
    let w = canvas.text_width(message, theme.type_scale.small * l.scale, 0.3) + 48.0 * l.scale;
    let mut spec = GlassSpec::new(cx, cy, w, 36.0 * l.scale, 18.0 * l.scale);
    spec.tint = 0.10 * fade;
    spec.glow = 0.4 * fade;
    canvas.glass(spec);
    canvas.text(message, cx, cy + 4.0 * l.scale, theme.type_scale.small * l.scale, theme.text, TextAlign::Center, 0.0, 0.3);
}
