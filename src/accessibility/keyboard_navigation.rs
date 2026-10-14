//! Focus order for Tab/Shift+Tab and the arrow keys between user tiles,
//! fields, and buttons. A single flat ring in a fixed, predictable order —
//! see [`crate::greeter::state`] for the actual focus index this cycles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    UsernameOrUserTile(usize),
    PasswordField,
    SubmitButton,
    SessionMenu,
    LanguageMenu,
    PowerButton,
}

pub fn next(current: Focus, tile_count: usize) -> Focus {
    match current {
        Focus::UsernameOrUserTile(i) if i + 1 < tile_count => Focus::UsernameOrUserTile(i + 1),
        Focus::UsernameOrUserTile(_) => Focus::PasswordField,
        Focus::PasswordField => Focus::SubmitButton,
        Focus::SubmitButton => Focus::SessionMenu,
        Focus::SessionMenu => Focus::LanguageMenu,
        Focus::LanguageMenu => Focus::PowerButton,
        Focus::PowerButton if tile_count > 0 => Focus::UsernameOrUserTile(0),
        Focus::PowerButton => Focus::PasswordField,
    }
}

pub fn previous(current: Focus, tile_count: usize) -> Focus {
    match current {
        Focus::UsernameOrUserTile(0) | Focus::PasswordField if tile_count == 0 => Focus::PowerButton,
        Focus::UsernameOrUserTile(0) => Focus::PowerButton,
        Focus::UsernameOrUserTile(i) => Focus::UsernameOrUserTile(i - 1),
        Focus::PasswordField if tile_count > 0 => Focus::UsernameOrUserTile(tile_count - 1),
        Focus::PasswordField => Focus::PowerButton,
        Focus::SubmitButton => Focus::PasswordField,
        Focus::SessionMenu => Focus::SubmitButton,
        Focus::LanguageMenu => Focus::SessionMenu,
        Focus::PowerButton => Focus::LanguageMenu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_all_the_way_around_with_tiles() {
        // 3 tiles + PasswordField/SubmitButton/SessionMenu/LanguageMenu/
        // PowerButton = 8 stops in the ring, so it takes exactly 8 calls to
        // `next` to return to the start.
        let mut f = Focus::UsernameOrUserTile(0);
        for _ in 0..8 {
            f = next(f, 3);
        }
        assert_eq!(f, Focus::UsernameOrUserTile(0));
    }

    #[test]
    fn next_then_previous_is_identity() {
        let start = Focus::SessionMenu;
        let n = next(start, 2);
        assert_eq!(previous(n, 2), start);
    }

    #[test]
    fn zero_tiles_skips_straight_to_password_field() {
        assert_eq!(next(Focus::PowerButton, 0), Focus::PasswordField);
        assert_eq!(previous(Focus::PasswordField, 0), Focus::PowerButton);
    }
}
