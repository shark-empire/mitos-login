use crate::greeter::renderer::{Canvas, TextAlign};
use crate::greeter::state::State;
use crate::theme::colors::ACCENT;
use crate::theme::Theme;

/// A single-line label + the current session's name, bottom-left of the
/// card — not a full dropdown in this version (cycling sessions is done via
/// [`crate::accessibility::keyboard_navigation::Focus::SessionMenu`] plus
/// Enter, handled in `greeter::input`); this just draws the current state.
pub fn draw(canvas: &mut Canvas, theme: &Theme, state: &State, card_cx: f32, card_bottom_y: f32) {
    let l = &theme.layout;
    let label = state.sessions.first().map(|s| s.name.as_str()).unwrap_or("—");
    canvas.text(
        &format!("{}  {}", state.translator.get("select_session"), label),
        card_cx - l.card_width / 2.0 + 20.0 * l.scale,
        card_bottom_y - 26.0 * l.scale,
        theme.type_scale.small * l.scale,
        theme.dim,
        TextAlign::Left,
        0.0,
        0.6,
    );
    let _ = ACCENT;
}
