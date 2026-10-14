use crate::greeter::renderer::{Canvas, TextAlign};
use crate::theme::{colors, Theme};

/// The large centered clock and date, drawn from the wall clock directly
/// (no IPC round trip needed for this).
pub fn draw(canvas: &mut Canvas, theme: &Theme, time_str: &str, date_str: &str) {
    let cx = canvas.w as f32 / 2.0;
    let y = canvas.h as f32 * 0.235;
    canvas.text(time_str, cx, y, theme.type_scale.clock * theme.layout.scale / 1.15, theme.text, TextAlign::Center, 1.0, 2.0);
    canvas.text(date_str, cx, y + 40.0 * theme.layout.scale, theme.type_scale.date * theme.layout.scale, theme.dim, TextAlign::Center, 0.0, 1.0);
    let _ = colors::ACCENT;
}
