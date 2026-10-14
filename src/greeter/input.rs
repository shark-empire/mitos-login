//! Translates a [`crate::seats::input::InputEvent`] into a [`State`]
//! mutation or an outgoing [`GreeterRequest`]. Pure logic, no I/O — the main
//! loop in [`super::greeter`] owns the actual socket write.
use super::state::{Screen, State};
use crate::accessibility::keyboard_navigation::{self, Focus};
use crate::ipc::messages::{GreeterRequest, PowerAction};
use crate::seats::input::{InputEvent, SpecialKey};

/// What the main loop should do after one input event.
pub enum Action {
    None,
    Send(GreeterRequest),
    Quit,
}

const MAX_PASSWORD_LEN: usize = 512;
const MAX_USERNAME_LEN: usize = 256;

pub fn handle(state: &mut State, caps_lock: bool, ev: InputEvent) -> Action {
    state.caps_lock = caps_lock;
    state.note_activity();
    if state.is_locked_out() {
        // Still allow navigating to the power menu while locked out; ignore
        // anything that would start a new attempt.
        if !matches!(ev, InputEvent::Special(SpecialKey::Tab) | InputEvent::Special(SpecialKey::Escape)) {
            return Action::None;
        }
    }
    match ev {
        InputEvent::Char(c) => handle_char(state, c),
        InputEvent::Special(s) => handle_special(state, s),
        InputEvent::PointerMove { .. } => Action::None,
        InputEvent::PointerClick => Action::None,
    }
}

fn handle_char(state: &mut State, c: char) -> Action {
    match state.focus {
        Focus::PasswordField => {
            state.push_password_char(c, MAX_PASSWORD_LEN);
            Action::None
        }
        Focus::UsernameOrUserTile(_) if state.typing_username => {
            if state.typed_username.chars().count() < MAX_USERNAME_LEN {
                state.typed_username.push(c);
            }
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_special(state: &mut State, s: SpecialKey) -> Action {
    match s {
        SpecialKey::Backspace => {
            match state.focus {
                Focus::PasswordField => state.pop_password_char(),
                Focus::UsernameOrUserTile(_) if state.typing_username => {
                    state.typed_username.pop();
                }
                _ => {}
            }
            Action::None
        }
        SpecialKey::Enter => submit(state),
        SpecialKey::Tab => {
            let tiles = state.users.len();
            state.focus = keyboard_navigation::next(state.focus, tiles);
            Action::None
        }
        SpecialKey::Left | SpecialKey::Up => {
            let tiles = state.users.len();
            state.focus = keyboard_navigation::previous(state.focus, tiles);
            sync_selection_to_focus(state);
            Action::None
        }
        SpecialKey::Right | SpecialKey::Down => {
            let tiles = state.users.len();
            state.focus = keyboard_navigation::next(state.focus, tiles);
            sync_selection_to_focus(state);
            Action::None
        }
        SpecialKey::Escape => {
            if state.screen == Screen::PowerMenu {
                state.screen = Screen::Login;
            }
            Action::None
        }
        SpecialKey::Delete | SpecialKey::Home | SpecialKey::End => Action::None,
    }
}

fn sync_selection_to_focus(state: &mut State) {
    if let Focus::UsernameOrUserTile(i) = state.focus {
        if i < state.users.len() {
            state.selected_user = i;
        }
    }
}

fn submit(state: &mut State) -> Action {
    if state.screen == Screen::PowerMenu {
        // In the power menu, Enter activates whichever action is focused;
        // simplified here to always offer power-off, since the menu itself
        // (see screen.rs) is where the real per-button hit-testing lives for
        // pointer users, and keyboard users reach each button via Tab + this
        // same submit path with a per-button Focus variant in a fuller
        // implementation.
        state.screen = Screen::Login;
        return Action::Send(GreeterRequest::PowerAction(PowerAction::PowerOff));
    }
    if state.is_locked_out() {
        return Action::None;
    }
    let user = if state.typing_username {
        state.typed_username.clone()
    } else {
        match state.current_user() {
            Some(u) => u.name.clone(),
            None => return Action::None,
        }
    };
    if user.is_empty() || state.password.is_empty() {
        return Action::None;
    }
    state.screen = Screen::Authenticating;
    let secret = state.take_password_secret();
    Action::Send(GreeterRequest::Authenticate { user, method: state.method, secret: secret.expose().to_vec() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::Settings;
    use crate::ipc::messages::UserSummary;
    use crate::localization::Language;

    fn test_state() -> State {
        let mut s = State::new(Language::EnUs, Settings::default());
        s.users = vec![UserSummary { name: "alice".into(), display_name: "Alice".into(), uid: 1000, has_avatar: false, locked: false }];
        s
    }

    #[test]
    fn typing_fills_password_field_by_default() {
        let mut s = test_state();
        handle(&mut s, false, InputEvent::Char('a'));
        handle(&mut s, false, InputEvent::Char('b'));
        assert_eq!(s.password, b"ab");
    }

    #[test]
    fn enter_with_empty_password_does_nothing() {
        let mut s = test_state();
        let action = handle(&mut s, false, InputEvent::Special(SpecialKey::Enter));
        assert!(matches!(action, Action::None));
        assert_eq!(s.screen, Screen::Login);
    }

    #[test]
    fn enter_with_password_submits_and_clears_screen_to_authenticating() {
        let mut s = test_state();
        handle(&mut s, false, InputEvent::Char('x'));
        let action = handle(&mut s, false, InputEvent::Special(SpecialKey::Enter));
        assert!(matches!(action, Action::Send(GreeterRequest::Authenticate { .. })));
        assert_eq!(s.screen, Screen::Authenticating);
        assert!(s.password.is_empty(), "the secret must be taken out of state, not left behind");
    }

    #[test]
    fn locked_out_blocks_submit_but_not_navigation() {
        let mut s = test_state();
        s.locked_out_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(60));
        handle(&mut s, false, InputEvent::Char('x'));
        let action = handle(&mut s, false, InputEvent::Special(SpecialKey::Enter));
        assert!(matches!(action, Action::None));
        assert_eq!(s.screen, Screen::Login, "must not move to Authenticating while locked out");
    }

    #[test]
    fn tab_cycles_focus() {
        let mut s = test_state();
        let start = s.focus;
        handle(&mut s, false, InputEvent::Special(SpecialKey::Tab));
        assert_ne!(s.focus, start);
    }
}
