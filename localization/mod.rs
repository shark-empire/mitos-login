//! UI string translation. A hand-rolled flat key=value format (no serde —
//! matches mitos-gui's own `i18n.rs` convention), embedded at compile time
//! via `include_str!` so a broken install path can never leave the greeter
//! with no strings at all.
pub mod language;
pub mod locale;
pub mod translations;

pub use language::Language;
pub use translations::Translator;
