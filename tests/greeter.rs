//! Exercises the greeter's pure state/input logic without any display,
//! socket, or real input device -- the whole point of keeping rendering and
//! I/O separate from state mutation (see src/greeter/input.rs's doc comment).
use mitos_login::accessibility::Settings;
use mitos_login::greeter::input::{handle, Action};
use mitos_login::greeter::state::{Screen, State};
use mitos_login::ipc::messages::{AuthOutcomeWire, GreeterRequest, UserSummary};
use mitos_login::localization::Language;
use mitos_login::seats::input::{InputEvent, SpecialKey};

fn state_with_one_user() -> State {
    let mut s = State::new(Language::EnUs, Settings::default());
    s.users = vec![UserSummary { name: "alice".into(), display_name: "Alice".into(), uid: 1000, has_avatar: false, locked: false }];
    s
}

#[test]
fn a_full_type_and_submit_cycle_produces_an_authenticate_request() {
    let mut s = state_with_one_user();
    for c in "correct horse".chars() {
        assert!(matches!(handle(&mut s, false, InputEvent::Char(c)), Action::None));
    }
    match handle(&mut s, false, InputEvent::Special(SpecialKey::Enter)) {
        Action::Send(GreeterRequest::Authenticate { user, secret, .. }) => {
            assert_eq!(user, "alice");
            assert_eq!(secret, b"correct horse");
        }
        _ => panic!("expected an Authenticate request, got a different action"),
    }
    assert_eq!(s.screen, Screen::Authenticating);
}

#[test]
fn a_failure_outcome_clears_the_password_and_shows_an_error() {
    let mut s = state_with_one_user();
    s.push_password_char('x', 64);
    s.apply_outcome(AuthOutcomeWire::Failure);
    assert!(s.password.is_empty());
    assert!(s.error_message.is_some());
    assert_eq!(s.screen, Screen::Login);
}

#[test]
fn lockout_outcome_blocks_further_submits_until_it_expires() {
    let mut s = state_with_one_user();
    s.apply_outcome(AuthOutcomeWire::LockedOut { retry_after_secs: 5 });
    assert!(s.is_locked_out());
    s.push_password_char('x', 64);
    let action = handle(&mut s, false, InputEvent::Special(SpecialKey::Enter));
    assert!(matches!(action, Action::None));
}
