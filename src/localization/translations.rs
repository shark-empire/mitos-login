//! Parses the embedded `locales/*.txt` flat `key=value` files. Embedding via
//! `include_str!` means a translation can never go missing at runtime from a
//! broken install path — the strings are baked into the binary itself.
use super::Language;
use std::collections::HashMap;

const EN_US: &str = include_str!("locales/en-US.txt");
const ES_ES: &str = include_str!("locales/es-ES.txt");

fn parse(src: &str) -> HashMap<&str, &str> {
    let mut map = HashMap::new();
    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k, v);
        }
    }
    map
}

pub struct Translator {
    active: HashMap<&'static str, &'static str>,
    fallback: HashMap<&'static str, &'static str>,
}

impl Translator {
    pub fn new(lang: Language) -> Translator {
        let active = parse(match lang {
            Language::EnUs => EN_US,
            Language::EsEs => ES_ES,
        });
        Translator { active, fallback: parse(EN_US) }
    }

    /// Looks up `key`, falling back to US English and then to the key itself
    /// — a missing translation must never make the UI show nothing.
    pub fn get(&self, key: &str) -> &str {
        self.active.get(key).or_else(|| self.fallback.get(key)).copied().unwrap_or(key)
    }

    /// `get`, substituting `{name}` placeholders from `vars`.
    pub fn get_with(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let mut s = self.get(key).to_string();
        for (name, value) in vars {
            s = s.replace(&format!("{{{}}}", name), value);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_key_resolves_in_both_locales() {
        let en = Translator::new(Language::EnUs);
        let es = Translator::new(Language::EsEs);
        assert_eq!(en.get("sign_in"), "Sign In");
        assert_eq!(es.get("sign_in"), "Iniciar sesión");
    }

    #[test]
    fn unknown_key_falls_back_to_itself() {
        let en = Translator::new(Language::EnUs);
        assert_eq!(en.get("totally_unknown_key"), "totally_unknown_key");
    }

    #[test]
    fn placeholder_substitution_works() {
        let en = Translator::new(Language::EnUs);
        let s = en.get_with("too_many_attempts", &[("seconds", "42")]);
        assert_eq!(s, "Too many attempts. Try again in 42s.");
    }

    #[test]
    fn every_key_present_in_english_has_a_spanish_translation() {
        let en = parse(EN_US);
        let es = parse(ES_ES);
        let missing: Vec<&&str> = en.keys().filter(|k| !es.contains_key(*k)).collect();
        assert!(missing.is_empty(), "es-ES is missing keys: {:?}", missing);
    }
}
