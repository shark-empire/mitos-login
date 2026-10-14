use mitos_login::localization::{Language, Translator};

#[test]
fn both_shipped_locales_translate_the_same_keys() {
    let en = Translator::new(Language::EnUs);
    let es = Translator::new(Language::EsEs);
    for key in ["sign_in", "password", "wrong_password", "power_off", "cancel"] {
        assert_ne!(en.get(key), key, "english is missing a translation for '{}'", key);
        assert_ne!(es.get(key), key, "spanish is missing a translation for '{}'", key);
    }
}

#[test]
fn language_round_trips_through_its_bcp47_code() {
    for lang in Language::all() {
        assert_eq!(Language::from_code(lang.code()), Some(*lang));
    }
    assert_eq!(Language::from_code("fr-FR"), None);
}

#[test]
fn placeholder_substitution_is_available_through_the_public_api() {
    let t = Translator::new(Language::EnUs);
    assert_eq!(t.get_with("too_many_attempts", &[("seconds", "10")]), "Too many attempts. Try again in 10s.");
}
