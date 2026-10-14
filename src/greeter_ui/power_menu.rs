use crate::greeter::renderer::{Canvas, GlassSpec, TextAlign};
use crate::greeter::state::State;
use crate::theme::Theme;

/// The power button in the status bar; the actual menu (suspend/restart/
/// power off) is drawn as a small glass popup when `state.screen` is
/// `PowerMenu`.
pub fn draw_button(canvas: &mut Canvas, theme: &Theme, t: f32, pulse: f32) {
    let l = &theme.layout;
    let cy = l.status_bar_margin + l.status_bar_height / 2.0;
    let x = canvas.w as f32 - l.status_bar_margin * 2.4 - 128.0 * l.scale;
    let mut spec = GlassSpec::new(x, cy, 30.0 * l.scale, 30.0 * l.scale, 15.0 * l.scale);
    spec.tint = 0.06;
    spec.glow = 0.25;
    spec.glow_radius = 8.0 * l.scale;
    spec.border = 0.9;
    spec.ring_phase = t * 0.2;
    spec.pulse = pulse;
    canvas.glass(spec);
    let r = 7.0 * l.scale;
    let pts: Vec<(f32, f32)> = (0..=20)
        .map(|k| {
            let a = (-50.0 + k as f32 * (280.0 / 20.0)).to_radians();
            (x + a.sin() * r, cy + 1.0 * l.scale - a.cos() * r)
        })
        .collect();
    for pair in pts.windows(2) {
        canvas.line(pair[0].0, pair[0].1, pair[1].0, pair[1].1, 1.8 * l.scale, theme.text, 1.0);
    }
    canvas.line(x, cy - 10.0 * l.scale, x, cy - 1.0 * l.scale, 1.8 * l.scale, theme.text, 1.0);
}

pub fn draw_menu(canvas: &mut Canvas, theme: &Theme, state: &State) {
    let l = &theme.layout;
    let cx = canvas.w as f32 / 2.0;
    let cy = canvas.h as f32 / 2.0;
    let w = 260.0 * l.scale;
    let items: Vec<(&str, bool)> = vec![
        (state.translator.get("suspend"), state.can_suspend),
        (state.translator.get("restart"), state.can_reboot),
        (state.translator.get("power_off"), state.can_poweroff),
    ];
    let h = 56.0 * l.scale * items.len() as f32 + 20.0 * l.scale;
    let mut spec = GlassSpec::new(cx, cy, w, h, 20.0 * l.scale);
    spec.tint = 0.14;
    spec.glow = 0.9;
    spec.glow_radius = 20.0 * l.scale;
    canvas.glass(spec);
    let top = cy - h / 2.0 + 38.0 * l.scale;
    for (i, (label, enabled)) in items.iter().enumerate() {
        let y = top + i as f32 * 56.0 * l.scale;
        let color = if *enabled { theme.text } else { theme.dim };
        canvas.text(label, cx, y, theme.type_scale.body * l.scale, color, TextAlign::Center, if *enabled { 0.3 } else { 0.0 }, 0.5);
    }
}
