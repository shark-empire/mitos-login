//! Single-output layout: everything is centered on the one framebuffer this
//! project supports. This module exists as the seam a future multi-output
//! implementation would extend, not because there is real layout math yet.
use super::resolution::Resolution;

pub fn center_of(res: Resolution) -> (f32, f32) {
    (res.width as f32 / 2.0, res.height as f32 / 2.0)
}
