//! Picks a [`super::Language`] from environment variables, the same
//! `LC_ALL`/`LC_MESSAGES`/`LANG` precedence glibc itself uses, falling back
//! to US English rather than failing if nothing matches.
use super::Language;

pub fn detect_from_env() -> Language {
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(v) = std::env::var(var) {
            if let Some(lang) = parse_locale(&v) {
                return lang;
            }
        }
    }
    Language::EnUs
}

fn parse_locale(v: &str) -> Option<Language> {
    let head = v.split(['.', '@']).next().unwrap_or(v);
    let normalized = head.replace('_', "-");
    Language::from_code(&normalized).or_else(|| Language::from_code(normalized.split('-').next().unwrap_or("")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_locale_string_shapes() {
        assert_eq!(parse_locale("en_US.UTF-8"), Some(Language::EnUs));
        assert_eq!(parse_locale("es_ES.UTF-8"), Some(Language::EsEs));
        assert_eq!(parse_locale("es"), Some(Language::EsEs));
        assert_eq!(parse_locale("fr_FR.UTF-8"), None);
        assert_eq!(parse_locale("C"), None);
    }
}
