//! Accessibility settings. These are plain data ([`Settings`]) consumed by
//! [`crate::theme::Theme`] (high contrast, large text) and the greeter's own
//! render/input loop (reduced motion, screen reader, sticky keys) — this
//! module has no rendering code of its own.
pub mod high_contrast;
pub mod keyboard_navigation;
pub mod large_text;
pub mod reduced_motion;
pub mod screen_reader;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    pub high_contrast: bool,
    pub large_text: bool,
    pub reduced_motion: bool,
    pub screen_reader: bool,
    /// Full keyboard navigation between user tiles/buttons (Tab/arrows) is
    /// always on in this greeter — there is no pointer-only interaction to
    /// begin with — so this only toggles *visible* focus rings for people
    /// who specifically rely on seeing where keyboard focus is.
    pub visible_focus: bool,
}
