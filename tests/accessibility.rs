use mitos_login::accessibility::keyboard_navigation::{next, previous, Focus};
use mitos_login::accessibility::Settings;
use mitos_login::theme::Theme;

#[test]
fn high_contrast_and_large_text_change_the_resolved_theme() {
    let normal = Theme::new(720, false, false, false);
    let hc = Theme::new(720, true, false, false);
    let large = Theme::new(720, false, true, false);
    assert_ne!(normal.text, hc.text, "high contrast must change the resolved text color");
    assert!(large.type_scale.body > normal.type_scale.body, "large text must scale type up");
}

#[test]
fn settings_default_to_everything_off() {
    let s = Settings::default();
    assert!(!s.high_contrast && !s.large_text && !s.reduced_motion && !s.screen_reader);
}

#[test]
fn focus_order_is_a_closed_loop_with_tiles() {
    let mut f = Focus::UsernameOrUserTile(0);
    let start = f;
    for _ in 0..8 {
        f = next(f, 3);
    }
    assert_eq!(f, start);
    assert_eq!(previous(next(Focus::SubmitButton, 2), 2), Focus::SubmitButton);
}
