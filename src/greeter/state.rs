//! All mutable UI state for one greeter run, independent of how it is
//! rendered or driven — [`super::screen`] reads this, [`super::input`]
//! mutates it.
use crate::accessibility::keyboard_navigation::Focus;
use crate::accessibility::Settings as AccessibilitySettings;
use crate::authentication::AuthMethod;
use crate::ipc::messages::{AuthOutcomeWire, SessionSummary, UserSummary};
use crate::localization::{Language, Translator};
use crate::security::Secret;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Login,
    Authenticating,
    PowerMenu,
}

pub struct State {
    pub screen: Screen,
    pub users: Vec<UserSummary>,
    pub sessions: Vec<SessionSummary>,
    pub selected_user: usize,
    pub typing_username: bool,
    pub typed_username: String,
    pub password: Vec<u8>,
    pub method: AuthMethod,
    pub focus: Focus,
    pub error_message: Option<String>,
    pub caps_lock: bool,
    pub can_suspend: bool,
    pub can_reboot: bool,
    pub can_poweroff: bool,
    pub locked_out_until: Option<Instant>,
    pub language: Language,
    pub translator: Translator,
    pub accessibility: AccessibilitySettings,
    pub shake_started: Option<Instant>,
    pub materialized_at: Instant,
    pub should_quit: bool,
    pub started_at: Instant,
    pub last_activity: Instant,
}

impl State {
    pub fn new(language: Language, accessibility: AccessibilitySettings) -> State {
        let now = Instant::now();
        State {
            screen: Screen::Login,
            users: Vec::new(),
            sessions: Vec::new(),
            selected_user: 0,
            typing_username: false,
            typed_username: String::new(),
            password: Vec::new(),
            method: AuthMethod::Password,
            focus: Focus::PasswordField,
            error_message: None,
            caps_lock: false,
            can_suspend: false,
            can_reboot: false,
            can_poweroff: false,
            locked_out_until: None,
            translator: Translator::new(language),
            language,
            accessibility,
            shake_started: None,
            materialized_at: now,
            should_quit: false,
            started_at: now,
            last_activity: now,
        }
    }

    pub fn current_user(&self) -> Option<&UserSummary> {
        self.users.get(self.selected_user)
    }

    pub fn clear_password(&mut self) {
        crate::sys::zeroize_vec(&mut self.password);
    }

    pub fn push_password_char(&mut self, c: char, max_len: usize) {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        if self.password.len() + s.len() <= max_len {
            self.password.extend_from_slice(s.as_bytes());
        }
        self.note_activity();
    }

    pub fn pop_password_char(&mut self) {
        // Remove one UTF-8 scalar value, not just one byte, so backspace
        // never leaves a truncated multi-byte sequence in the buffer.
        while let Some(&last) = self.password.last() {
            self.password.pop();
            if (last & 0b1100_0000) != 0b1000_0000 {
                break;
            }
        }
        self.note_activity();
    }

    pub fn note_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn idle_for(&self) -> std::time::Duration {
        self.last_activity.elapsed()
    }

    pub fn take_password_secret(&mut self) -> Secret {
        let bytes = std::mem::take(&mut self.password);
        Secret::new(bytes)
    }

    pub fn apply_outcome(&mut self, outcome: AuthOutcomeWire) {
        self.screen = Screen::Login;
        match outcome {
            AuthOutcomeWire::Success => {
                self.error_message = None;
                self.clear_password();
            }
            AuthOutcomeWire::Failure => {
                self.error_message = Some(self.translator.get("wrong_password").to_string());
                self.clear_password();
                self.shake_started = Some(Instant::now());
            }
            AuthOutcomeWire::LockedOut { retry_after_secs } => {
                self.locked_out_until = Some(Instant::now() + std::time::Duration::from_secs(retry_after_secs));
                self.error_message = Some(self.translator.get_with("too_many_attempts", &[("seconds", &retry_after_secs.to_string())]));
                self.clear_password();
                self.shake_started = Some(Instant::now());
            }
            AuthOutcomeWire::Error => {
                self.error_message = Some("Internal error".to_string());
                self.clear_password();
            }
        }
    }

    pub fn is_locked_out(&self) -> bool {
        self.locked_out_until.is_some_and(|t| Instant::now() < t)
    }
}

impl Drop for State {
    fn drop(&mut self) {
        self.clear_password();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> State {
        State::new(Language::EnUs, AccessibilitySettings::default())
    }

    #[test]
    fn push_and_pop_password_chars() {
        let mut s = test_state();
        s.push_password_char('a', 64);
        s.push_password_char('b', 64);
        assert_eq!(s.password, b"ab");
        s.pop_password_char();
        assert_eq!(s.password, b"a");
    }

    #[test]
    fn pop_removes_whole_utf8_scalar_not_one_byte() {
        let mut s = test_state();
        s.push_password_char('é', 64); // 2-byte UTF-8 sequence
        assert_eq!(s.password.len(), 2);
        s.pop_password_char();
        assert!(s.password.is_empty(), "backspace must remove the whole character, not one byte");
    }

    #[test]
    fn push_respects_max_len() {
        let mut s = test_state();
        s.push_password_char('a', 1);
        s.push_password_char('b', 1);
        assert_eq!(s.password, b"a");
    }

    #[test]
    fn take_password_secret_empties_the_buffer() {
        let mut s = test_state();
        s.push_password_char('x', 64);
        let secret = s.take_password_secret();
        assert_eq!(secret.expose(), b"x");
        assert!(s.password.is_empty());
    }

    #[test]
    fn locked_out_until_expires() {
        let mut s = test_state();
        s.apply_outcome(AuthOutcomeWire::LockedOut { retry_after_secs: 0 });
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert!(!s.is_locked_out());
    }
}
