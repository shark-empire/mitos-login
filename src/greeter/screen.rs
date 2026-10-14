//! Composes one full frame from [`super::state::State`] by calling the
//! [`crate::greeter_ui`] widgets in order.
use super::renderer::Canvas;
use super::state::{Screen, State};
use crate::greeter::animation::ambient_pulse;
use crate::greeter_ui::{clock, login_form, password_input, power_menu, session_menu, status_bar, user_list};
use crate::theme::Theme;

pub fn draw(canvas: &mut Canvas, theme: &Theme, state: &State, t: f32, time_str: &str, date_str: &str) {
    let pulse = if state.accessibility.reduced_motion { 0.5 } else { ambient_pulse(t, 4.0) };
    if !state.accessibility.reduced_motion {
        canvas.particles(t, 7, 46);
    }

    status_bar::draw(canvas, theme, t, pulse);
    crate::greeter_ui::language_menu::draw(canvas, theme, state.language, t, pulse);
    power_menu::draw_button(canvas, theme, t, pulse);
    clock::draw(canvas, theme, time_str, date_str);

    let card = login_form::draw(canvas, theme, state, t, pulse);
    let focus = matches!(state.focus, crate::accessibility::keyboard_navigation::Focus::PasswordField);
    password_input::draw(canvas, theme, state, &card, focus, pulse, t);
    session_menu::draw(canvas, theme, state, card.cx, card.bottom_y);
    user_list::draw(canvas, theme, state, t, pulse, card.bottom_y);

    if state.screen == Screen::PowerMenu {
        power_menu::draw_menu(canvas, theme, state);
    }
}
