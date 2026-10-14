//! Design tokens shared by every greeter screen: colors, a type scale, and
//! resolution-relative layout metrics, bundled into one [`Theme`] so a
//! screen never hard-codes a color or a pixel count itself.
pub mod colors;
pub mod layout;
pub mod theme;
pub mod typography;

pub use colors::Rgb;
pub use theme::Theme;
