//! The accessibility quick-settings panel, reachable from the status bar.
//! Toggles [`crate::accessibility::Settings`] fields directly; the greeter's
//! main loop persists the result back to `greeter.toml` on change (see
//! `greeter::greeter::persist_accessibility`).
use crate::accessibility::Settings;
use crate::greeter::renderer::{Canvas, GlassSpec, TextAlign};
use crate::theme::Theme;

pub fn draw_menu(canvas: &mut Canvas, theme: &Theme, settings: &Settings) {
    let l = &theme.layout;
    let cx = canvas.w as f32 / 2.0;
    let cy = canvas.h as f32 / 2.0;
    let w = 300.0 * l.scale;
    let rows: [(&str, bool); 4] = [
        ("High Contrast", settings.high_contrast),
        ("Large Text", settings.large_text),
        ("Reduced Motion", settings.reduced_motion),
        ("Visible Focus Ring", settings.visible_focus),
    ];
    let h = 44.0 * l.scale * rows.len() as f32 + 30.0 * l.scale;
    let mut spec = GlassSpec::new(cx, cy, w, h, 18.0 * l.scale);
    spec.tint = 0.12;
    spec.glow = 0.7;
    canvas.glass(spec);
    let top = cy - h / 2.0 + 34.0 * l.scale;
    for (i, (label, on)) in rows.iter().enumerate() {
        let y = top + i as f32 * 44.0 * l.scale;
        canvas.text(label, cx - w / 2.0 + 20.0 * l.scale, y, theme.type_scale.small * l.scale, theme.text, TextAlign::Left, 0.0, 0.3);
        let dot_color = if *on { theme.accent } else { theme.dim };
        canvas.fill_circle(cx + w / 2.0 - 24.0 * l.scale, y - 4.0 * l.scale, 6.0 * l.scale, dot_color, 1.0);
    }
}
