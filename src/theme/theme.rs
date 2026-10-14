use super::colors::{Rgb, ACCENT, ACCENT_HIGH_CONTRAST, DIM, TEXT, TEXT_HIGH_CONTRAST};
use super::layout::Layout;
use super::typography::{TypeScale, DEFAULT_SCALE, LARGE_TEXT_MULTIPLIER};

/// Everything a screen needs to draw itself: resolved colors (already
/// accounting for high-contrast mode), a type scale (already accounting for
/// large-text mode), and layout metrics for the real output size.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub layout: Layout,
    pub type_scale: TypeScale,
    pub text: Rgb,
    pub dim: Rgb,
    pub accent: Rgb,
    pub reduced_motion: bool,
}

impl Theme {
    pub fn new(height: u32, high_contrast: bool, large_text: bool, reduced_motion: bool) -> Theme {
        let mut type_scale = DEFAULT_SCALE;
        if large_text {
            type_scale.clock *= LARGE_TEXT_MULTIPLIER;
            type_scale.date *= LARGE_TEXT_MULTIPLIER;
            type_scale.display_name *= LARGE_TEXT_MULTIPLIER;
            type_scale.body *= LARGE_TEXT_MULTIPLIER;
            type_scale.label *= LARGE_TEXT_MULTIPLIER;
            type_scale.small *= LARGE_TEXT_MULTIPLIER;
        }
        Theme {
            layout: Layout::for_height(height),
            type_scale,
            text: if high_contrast { TEXT_HIGH_CONTRAST } else { TEXT },
            dim: if high_contrast { TEXT_HIGH_CONTRAST } else { DIM },
            accent: if high_contrast { ACCENT_HIGH_CONTRAST } else { ACCENT },
            reduced_motion,
        }
    }
}
