//! Font-size scale. All sizes are in design pixels at the theme's reference
//! scale factor ([`super::layout::Layout::scale`]); the renderer multiplies
//! by that factor once, so every widget stays proportionate on any output
//! resolution instead of hard-coding pixel counts.
#[derive(Clone, Copy, Debug)]
pub struct TypeScale {
    pub clock: f32,
    pub date: f32,
    pub display_name: f32,
    pub body: f32,
    pub label: f32,
    pub small: f32,
}

pub const DEFAULT_SCALE: TypeScale = TypeScale {
    clock: 72.0,
    date: 15.0,
    display_name: 19.0,
    body: 15.0,
    label: 11.0,
    small: 10.0,
};

/// Applied when the person turns on "Large Text" accessibility.
pub const LARGE_TEXT_MULTIPLIER: f32 = 1.35;
