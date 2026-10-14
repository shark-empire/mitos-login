//! Spacing and sizing constants, all expressed relative to `scale` (itself
//! derived from the real output height, see [`Layout::for_height`]), so the
//! same numbers that looked right at 720p still look right at 4K.
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub scale: f32,
    pub status_bar_height: f32,
    pub status_bar_margin: f32,
    pub card_width: f32,
    pub card_height: f32,
    pub avatar_size: f32,
    pub field_width: f32,
    pub field_height: f32,
    pub field_radius: f32,
    pub user_tile_size: f32,
    pub user_tile_gap: f32,
    pub corner_radius: f32,
    pub panel_glow_radius: f32,
}

impl Layout {
    /// `height` is the real output height in physical pixels; every constant
    /// below was tuned against a 720-pixel-tall reference frame.
    pub fn for_height(height: u32) -> Layout {
        let scale = (height as f32 / 720.0).max(0.5);
        Layout {
            scale,
            status_bar_height: 44.0 * scale,
            status_bar_margin: 14.0 * scale,
            card_width: 400.0 * scale,
            card_height: 340.0 * scale,
            avatar_size: 96.0 * scale,
            field_width: 320.0 * scale,
            field_height: 50.0 * scale,
            field_radius: 25.0 * scale,
            user_tile_size: 60.0 * scale,
            user_tile_gap: 50.0 * scale,
            corner_radius: 30.0 * scale,
            panel_glow_radius: 30.0 * scale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_is_one_at_reference_height() {
        let l = Layout::for_height(720);
        assert!((l.scale - 1.0).abs() < 1e-6);
        assert!((l.card_width - 400.0).abs() < 1e-3);
    }

    #[test]
    fn never_scales_below_half() {
        let l = Layout::for_height(1);
        assert_eq!(l.scale, 0.5);
    }
}
