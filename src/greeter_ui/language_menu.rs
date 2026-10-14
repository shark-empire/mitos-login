use crate::greeter::renderer::{Canvas, GlassSpec, TextAlign};
use crate::localization::Language;
use crate::theme::Theme;

/// Two small pill buttons in the status bar: current language, and a
/// keyboard-layout indicator (always "US" today — see docs/architecture.md's
/// roadmap on additional keyboard layouts).
pub fn draw(canvas: &mut Canvas, theme: &Theme, language: Language, t: f32, pulse: f32) {
    let l = &theme.layout;
    let cy = l.status_bar_margin + l.status_bar_height / 2.0;
    let mut x = canvas.w as f32 - l.status_bar_margin * 2.4;
    for label in ["US", short_code(language)] {
        let mut spec = GlassSpec::new(x, cy, 50.0 * l.scale, 30.0 * l.scale, 15.0 * l.scale);
        spec.tint = 0.06;
        spec.glow = 0.25;
        spec.glow_radius = 8.0 * l.scale;
        spec.border = 0.9;
        spec.ring_phase = t * 0.2;
        spec.pulse = pulse;
        canvas.glass(spec);
        canvas.text(label, x, cy + 4.5 * l.scale, 9.0 * l.scale, theme.text, TextAlign::Center, 0.0, 0.8);
        x -= 64.0 * l.scale;
    }
}

fn short_code(language: Language) -> &'static str {
    match language {
        Language::EnUs => "EN",
        Language::EsEs => "ES",
    }
}
