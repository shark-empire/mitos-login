#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    EnUs,
    EsEs,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::EnUs => "en-US",
            Language::EsEs => "es-ES",
        }
    }

    pub fn from_code(code: &str) -> Option<Language> {
        match code {
            "en-US" | "en" => Some(Language::EnUs),
            "es-ES" | "es" => Some(Language::EsEs),
            _ => None,
        }
    }

    pub fn all() -> &'static [Language] {
        &[Language::EnUs, Language::EsEs]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::EnUs => "English",
            Language::EsEs => "Español",
        }
    }
}
