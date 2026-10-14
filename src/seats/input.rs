//! Translates raw evdev key codes into characters (US QWERTY; a different
//! layout is a matter of swapping this table — see
//! [`crate::localization`] for where that hook belongs next) and tracks
//! modifier state. Deliberately independent of any specific device: the
//! daemon feeds every grabbed device's events through the same
//! [`Keymap::translate`].
use crate::sys::ffi;
use crate::sys::{RawInputEvent, EV_KEY, EV_REL};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialKey {
    Enter,
    Backspace,
    Delete,
    Tab,
    Escape,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent {
    Char(char),
    Special(SpecialKey),
    PointerMove { dx: f32, dy: f32 },
    PointerClick,
}

#[derive(Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
    pub caps_lock: bool,
}

pub struct Keymap {
    pub mods: Modifiers,
}

impl Keymap {
    pub fn new() -> Keymap {
        Keymap { mods: Modifiers::default() }
    }

    /// Feeds one raw evdev event; returns the higher-level event it produces,
    /// if any (most events — key releases, unmapped codes — produce none).
    pub fn translate(&mut self, ev: RawInputEvent) -> Option<InputEvent> {
        if ev.type_ == EV_REL {
            return match ev.code {
                ffi::REL_X => Some(InputEvent::PointerMove { dx: ev.value as f32, dy: 0.0 }),
                ffi::REL_Y => Some(InputEvent::PointerMove { dx: 0.0, dy: ev.value as f32 }),
                _ => None,
            };
        }
        if ev.type_ != EV_KEY {
            return None;
        }
        let pressed = ev.value != 0; // 1 = down, 2 = autorepeat, 0 = up
        let released = ev.value == 0;
        match ev.code {
            ffi::KEY_LEFTSHIFT | ffi::KEY_RIGHTSHIFT => {
                self.mods.shift = pressed;
                return None;
            }
            ffi::KEY_LEFTCTRL | ffi::KEY_RIGHTCTRL => {
                self.mods.ctrl = pressed;
                return None;
            }
            ffi::KEY_LEFTALT | ffi::KEY_RIGHTALT => {
                self.mods.alt = pressed;
                return None;
            }
            ffi::KEY_LEFTMETA | ffi::KEY_RIGHTMETA => {
                self.mods.meta = pressed;
                return None;
            }
            ffi::KEY_CAPSLOCK => {
                if ev.value == 1 {
                    self.mods.caps_lock = !self.mods.caps_lock;
                }
                return None;
            }
            ffi::BTN_LEFT if ev.value == 1 => return Some(InputEvent::PointerClick),
            _ => {}
        }
        if released {
            return None;
        }
        // Every modifier key already returned above, so anything reaching
        // here is an ordinary key: produce its char/special on both the
        // initial press (value == 1) and every autorepeat (value >= 2),
        // exactly like a normal keyboard typing into a text field.
        if let Some(special) = special_for(ev.code) {
            return Some(InputEvent::Special(special));
        }
        char_for(ev.code, &self.mods).map(InputEvent::Char)
    }
}

impl Default for Keymap {
    fn default() -> Self {
        Self::new()
    }
}

fn special_for(code: u16) -> Option<SpecialKey> {
    Some(match code {
        ffi::KEY_ENTER | ffi::KEY_KPENTER => SpecialKey::Enter,
        ffi::KEY_BACKSPACE => SpecialKey::Backspace,
        ffi::KEY_DELETE => SpecialKey::Delete,
        ffi::KEY_TAB => SpecialKey::Tab,
        ffi::KEY_ESC => SpecialKey::Escape,
        ffi::KEY_UP => SpecialKey::Up,
        ffi::KEY_DOWN => SpecialKey::Down,
        ffi::KEY_LEFT => SpecialKey::Left,
        ffi::KEY_RIGHT => SpecialKey::Right,
        ffi::KEY_HOME => SpecialKey::Home,
        ffi::KEY_END => SpecialKey::End,
        _ => return None,
    })
}

/// US QWERTY only. `shift` XORed with `caps_lock` affects letters; digit/
/// symbol keys respond to `shift` alone, matching every real keyboard.
fn char_for(code: u16, mods: &Modifiers) -> Option<char> {
    let letter_shift = mods.shift != mods.caps_lock;
    let letter = |lower: char| Some(if letter_shift { lower.to_ascii_uppercase() } else { lower });
    match code {
        ffi::KEY_A => letter('a'),
        ffi::KEY_B => letter('b'),
        ffi::KEY_C => letter('c'),
        ffi::KEY_D => letter('d'),
        ffi::KEY_E => letter('e'),
        ffi::KEY_F => letter('f'),
        ffi::KEY_G => letter('g'),
        ffi::KEY_H => letter('h'),
        ffi::KEY_I => letter('i'),
        ffi::KEY_J => letter('j'),
        ffi::KEY_K => letter('k'),
        ffi::KEY_L => letter('l'),
        ffi::KEY_M => letter('m'),
        ffi::KEY_N => letter('n'),
        ffi::KEY_O => letter('o'),
        ffi::KEY_P => letter('p'),
        ffi::KEY_Q => letter('q'),
        ffi::KEY_R => letter('r'),
        ffi::KEY_S => letter('s'),
        ffi::KEY_T => letter('t'),
        ffi::KEY_U => letter('u'),
        ffi::KEY_V => letter('v'),
        ffi::KEY_W => letter('w'),
        ffi::KEY_X => letter('x'),
        ffi::KEY_Y => letter('y'),
        ffi::KEY_Z => letter('z'),
        ffi::KEY_0 => Some(if mods.shift { ')' } else { '0' }),
        ffi::KEY_1 => Some(if mods.shift { '!' } else { '1' }),
        ffi::KEY_2 => Some(if mods.shift { '@' } else { '2' }),
        ffi::KEY_3 => Some(if mods.shift { '#' } else { '3' }),
        ffi::KEY_4 => Some(if mods.shift { '$' } else { '4' }),
        ffi::KEY_5 => Some(if mods.shift { '%' } else { '5' }),
        ffi::KEY_6 => Some(if mods.shift { '^' } else { '6' }),
        ffi::KEY_7 => Some(if mods.shift { '&' } else { '7' }),
        ffi::KEY_8 => Some(if mods.shift { '*' } else { '8' }),
        ffi::KEY_9 => Some(if mods.shift { '(' } else { '9' }),
        ffi::KEY_SPACE => Some(' '),
        ffi::KEY_MINUS => Some(if mods.shift { '_' } else { '-' }),
        ffi::KEY_EQUAL => Some(if mods.shift { '+' } else { '=' }),
        ffi::KEY_LEFTBRACE => Some(if mods.shift { '{' } else { '[' }),
        ffi::KEY_RIGHTBRACE => Some(if mods.shift { '}' } else { ']' }),
        ffi::KEY_BACKSLASH => Some(if mods.shift { '|' } else { '\\' }),
        ffi::KEY_SEMICOLON => Some(if mods.shift { ':' } else { ';' }),
        ffi::KEY_APOSTROPHE => Some(if mods.shift { '"' } else { '\'' }),
        ffi::KEY_GRAVE => Some(if mods.shift { '~' } else { '`' }),
        ffi::KEY_COMMA => Some(if mods.shift { '<' } else { ',' }),
        ffi::KEY_DOT => Some(if mods.shift { '>' } else { '.' }),
        ffi::KEY_SLASH => Some(if mods.shift { '?' } else { '/' }),
        ffi::KEY_KP0 => Some('0'),
        ffi::KEY_KP1 => Some('1'),
        ffi::KEY_KP2 => Some('2'),
        ffi::KEY_KP3 => Some('3'),
        ffi::KEY_KP4 => Some('4'),
        ffi::KEY_KP5 => Some('5'),
        ffi::KEY_KP6 => Some('6'),
        ffi::KEY_KP7 => Some('7'),
        ffi::KEY_KP8 => Some('8'),
        ffi::KEY_KP9 => Some('9'),
        ffi::KEY_KPDOT => Some('.'),
        ffi::KEY_KPPLUS => Some('+'),
        ffi::KEY_KPMINUS => Some('-'),
        ffi::KEY_KPASTERISK => Some('*'),
        ffi::KEY_KPSLASH => Some('/'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: u16, value: i32) -> RawInputEvent {
        RawInputEvent { type_: EV_KEY, code, value }
    }

    #[test]
    fn plain_letter_is_lowercase() {
        let mut km = Keymap::new();
        assert_eq!(km.translate(key(ffi::KEY_A, 1)), Some(InputEvent::Char('a')));
    }

    #[test]
    fn shift_makes_letters_uppercase_and_changes_symbols() {
        let mut km = Keymap::new();
        km.translate(key(ffi::KEY_LEFTSHIFT, 1));
        assert_eq!(km.translate(key(ffi::KEY_A, 1)), Some(InputEvent::Char('A')));
        assert_eq!(km.translate(key(ffi::KEY_1, 1)), Some(InputEvent::Char('!')));
        km.translate(key(ffi::KEY_LEFTSHIFT, 0));
        assert_eq!(km.translate(key(ffi::KEY_A, 1)), Some(InputEvent::Char('a')));
    }

    #[test]
    fn caps_lock_toggles_letters_only() {
        let mut km = Keymap::new();
        km.translate(key(ffi::KEY_CAPSLOCK, 1));
        km.translate(key(ffi::KEY_CAPSLOCK, 0));
        assert_eq!(km.translate(key(ffi::KEY_A, 1)), Some(InputEvent::Char('A')));
        assert_eq!(km.translate(key(ffi::KEY_1, 1)), Some(InputEvent::Char('1')), "caps lock must not affect digits");
        // shift + caps lock cancels back to lowercase, like a real keyboard
        km.translate(key(ffi::KEY_LEFTSHIFT, 1));
        assert_eq!(km.translate(key(ffi::KEY_A, 1)), Some(InputEvent::Char('a')));
    }

    #[test]
    fn key_release_produces_nothing() {
        let mut km = Keymap::new();
        assert_eq!(km.translate(key(ffi::KEY_A, 0)), None);
    }

    #[test]
    fn enter_and_backspace_are_special_not_char() {
        let mut km = Keymap::new();
        assert_eq!(km.translate(key(ffi::KEY_ENTER, 1)), Some(InputEvent::Special(SpecialKey::Enter)));
        assert_eq!(km.translate(key(ffi::KEY_BACKSPACE, 1)), Some(InputEvent::Special(SpecialKey::Backspace)));
    }

    #[test]
    fn modifier_keys_never_produce_a_char() {
        let mut km = Keymap::new();
        assert_eq!(km.translate(key(ffi::KEY_LEFTCTRL, 1)), None);
        assert_eq!(km.translate(key(ffi::KEY_LEFTALT, 1)), None);
        assert_eq!(km.translate(key(ffi::KEY_LEFTMETA, 1)), None);
    }
}
