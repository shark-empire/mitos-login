//! One file per visual piece of the greeter, each a thin function over
//! [`crate::greeter::renderer::Canvas`]: given the canvas, the current
//! [`crate::theme::Theme`], and the relevant slice of
//! [`crate::greeter::state::State`], draw that piece. [`crate::greeter::screen`]
//! calls these in the right order for whichever [`crate::greeter::state::Screen`]
//! is active.
pub mod accessibility;
pub mod clock;
pub mod keyboard_menu;
pub mod language_menu;
pub mod login_form;
pub mod notifications;
pub mod password_input;
pub mod pin_input;
pub mod power_menu;
pub mod session_menu;
pub mod status_bar;
pub mod user_list;
