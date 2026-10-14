//! The main glass card: avatar, display name, then delegates to
//! [`super::password_input`]/[`super::pin_input`] for the field itself.
use crate::greeter::animation::shake_offset;
use crate::greeter::renderer::{Canvas, GlassSpec, TextAlign};
use crate::greeter::state::State;
use crate::theme::colors::{ACCENT, ACCENT_2};
use crate::theme::Theme;

pub struct CardLayout {
    pub cx: f32,
    pub cy: f32,
    pub bottom_y: f32,
}

pub fn draw(canvas: &mut Canvas, theme: &Theme, state: &State, t: f32, pulse: f32) -> CardLayout {
    let l = &theme.layout;
    let cx = canvas.w as f32 / 2.0;
    let cy = canvas.h as f32 * 0.30 + l.card_height / 2.0 + 10.0 * l.scale;
    let shake = state
        .shake_started
        .map(|start| {
            let one_shot = crate::greeter::animation::OneShot::from_start(start, std::time::Duration::from_millis(420));
            shake_offset(&one_shot, 10.0 * l.scale)
        })
        .unwrap_or(0.0);
    let cx = cx + shake;

    let mut spec = GlassSpec::new(cx, cy, l.card_width, l.card_height, l.corner_radius);
    spec.tint = 0.10;
    spec.glow = 1.0;
    spec.glow_radius = l.panel_glow_radius;
    spec.border = 1.5;
    spec.ring_phase = t * 0.35;
    spec.pulse = pulse;
    if state.error_message.is_some() {
        spec.accent = crate::theme::colors::DANGER;
        spec.accent2 = crate::theme::colors::WARN;
    }
    canvas.glass(spec);

    let avatar_y = cy - l.card_height / 2.0 + 70.0 * l.scale;
    let mut avatar_spec = GlassSpec::new(cx, avatar_y, l.avatar_size, l.avatar_size, l.avatar_size / 2.0);
    avatar_spec.tint = 0.12;
    avatar_spec.glow = 1.0;
    avatar_spec.glow_radius = 16.0 * l.scale;
    avatar_spec.border = 2.0;
    avatar_spec.ring_phase = -t * 0.9;
    avatar_spec.pulse = pulse;
    avatar_spec.accent = ACCENT;
    avatar_spec.accent2 = ACCENT_2;
    canvas.glass(avatar_spec);

    let name = state.current_user().map(|u| u.display_name.as_str()).unwrap_or("");
    let initial = name.chars().next().unwrap_or('?').to_ascii_uppercase();
    let mut buf = [0u8; 4];
    canvas.text(initial.encode_utf8(&mut buf), cx, avatar_y + 15.0 * l.scale, 40.0 * l.scale, theme.text, TextAlign::Center, 1.0, 0.0);
    canvas.text(name, cx, avatar_y + 82.0 * l.scale, theme.type_scale.display_name * l.scale, theme.text, TextAlign::Center, 0.5, 0.8);

    CardLayout { cx, cy, bottom_y: cy + l.card_height / 2.0 }
}
