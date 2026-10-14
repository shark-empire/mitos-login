use super::login_form::CardLayout;
use crate::greeter::renderer::{Canvas, TextAlign};
use crate::greeter::state::State;
use crate::theme::colors::ACCENT;
use crate::theme::Theme;

/// The recessed field with a password dot per typed byte, capped visually so
/// a very long password doesn't overflow the card — the dot count is purely
/// cosmetic feedback, never a reliable indicator of exact length.
pub fn draw(canvas: &mut Canvas, theme: &Theme, state: &State, card: &CardLayout, focus: bool, pulse: f32, t: f32) {
    let l = &theme.layout;
    let field_y = card.cy - l.card_height / 2.0 + 70.0 * l.scale + 126.0 * l.scale;
    canvas.inset(card.cx, field_y, l.field_width, l.field_height, l.field_radius, if focus { 1.0 } else { 0.4 }, pulse, t * 0.6);

    let dot_count = state.password.len().min(24);
    if dot_count > 0 {
        let spacing = 17.0 * l.scale;
        let start_x = card.cx - (dot_count as f32 - 1.0) * spacing / 2.0;
        for i in 0..dot_count {
            let x = start_x + i as f32 * spacing;
            canvas.fill_circle(x, field_y, 4.2 * l.scale, theme.text, 1.0);
            canvas.disc_glow(x, field_y, 4.2 * l.scale, ACCENT, 0.30);
        }
    } else {
        let placeholder = theme.text.lerp(theme.dim, 0.5);
        canvas.text(
            state.translator.get("password_placeholder"),
            card.cx - l.field_width / 2.0 + 16.0 * l.scale,
            field_y + 4.0 * l.scale,
            theme.type_scale.body * l.scale * 0.8,
            placeholder,
            TextAlign::Left,
            0.0,
            0.0,
        );
    }

    if let Some(err) = &state.error_message {
        canvas.text(err, card.cx, field_y + 50.0 * l.scale, theme.type_scale.small * l.scale, crate::theme::colors::WARN, TextAlign::Center, 0.6, 0.6);
    } else if state.caps_lock {
        canvas.text(
            state.translator.get("caps_lock_on"),
            card.cx,
            field_y + 50.0 * l.scale,
            theme.type_scale.small * l.scale,
            crate::theme::colors::WARN,
            TextAlign::Center,
            0.6,
            0.6,
        );
    }
}
