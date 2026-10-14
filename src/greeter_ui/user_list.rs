use crate::greeter::renderer::{Canvas, GlassSpec, TextAlign};
use crate::greeter::state::State;
use crate::theme::Theme;

/// The row of user tiles below the login card. Hidden entirely when there is
/// only one account, matching how most login screens behave.
pub fn draw(canvas: &mut Canvas, theme: &Theme, state: &State, t: f32, pulse: f32, card_bottom_y: f32) {
    if state.users.len() < 2 {
        return;
    }
    let l = &theme.layout;
    let cx0 = canvas.w as f32 / 2.0;
    let n = state.users.len();
    let span = (n as f32 - 1.0) * l.user_tile_gap;
    let y = card_bottom_y + l.user_tile_size;
    for (i, user) in state.users.iter().enumerate() {
        let x = cx0 - span / 2.0 + i as f32 * l.user_tile_gap;
        let selected = i == state.selected_user;
        let mut spec = GlassSpec::new(x, y, l.user_tile_size, l.user_tile_size, l.user_tile_size / 2.0);
        spec.tint = if selected { 0.14 } else { 0.08 };
        spec.glow = if selected { 0.9 } else { 0.25 };
        spec.glow_radius = 12.0 * l.scale;
        spec.border = if selected { 1.4 } else { 0.8 };
        spec.ring_phase = t * 0.4 + i as f32;
        spec.pulse = pulse;
        canvas.glass(spec);
        let initial = user.display_name.chars().next().unwrap_or('?').to_ascii_uppercase();
        let mut buf = [0u8; 4];
        canvas.text(initial.encode_utf8(&mut buf), x, y + 9.0 * l.scale, 22.0 * l.scale, theme.text, TextAlign::Center, if selected { 0.6 } else { 0.0 }, 0.0);
        canvas.text(&user.display_name, x, y + 52.0 * l.scale, 10.0 * l.scale, if selected { theme.text } else { theme.dim }, TextAlign::Center, 0.0, 0.5);
    }
}
