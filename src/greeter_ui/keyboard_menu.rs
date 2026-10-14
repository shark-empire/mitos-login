//! A keyboard-layout picker. Only one layout (US QWERTY) is implemented in
//! [`crate::seats::input`] today, so this draws the current layout's name as
//! a fixed label rather than a real picker — see
//! [`super::language_menu::draw`], which renders the same "US" pill in the
//! status bar; this module exists as the seam a second layout would extend
//! (a menu body, analogous to [`super::power_menu::draw_menu`]), not because
//! there is a working picker behind it yet.
pub const CURRENT_LAYOUT_LABEL: &str = "US QWERTY";
