//! A hand-rolled parser for the small TOML subset this project's config
//! files actually use: `[section]` headers, `key = value` pairs, string/
//! bool/integer/float/string-array values, `#` comments. No nested tables,
//! no dates, no multi-line strings, no inline tables — anything requiring
//! those is rejected with a clear error rather than silently misparsed.
use crate::errors::{Error, Result};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<Value>),
}

/// Keys in the unnamed section that appears before any `[section]` header
/// live under this key.
pub const TOP_LEVEL: &str = "";

#[derive(Debug, Default)]
pub struct Document {
    sections: HashMap<String, HashMap<String, Value>>,
}

impl Document {
    pub fn table(&self, section: &str) -> Option<&HashMap<String, Value>> {
        self.sections.get(section)
    }

    pub fn get_str(&self, section: &str, key: &str) -> Option<&str> {
        match self.table(section)?.get(key)? {
            Value::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }
    pub fn get_bool(&self, section: &str, key: &str) -> Option<bool> {
        match self.table(section)?.get(key)? {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn get_u32(&self, section: &str, key: &str) -> Option<u32> {
        match self.table(section)?.get(key)? {
            Value::Int(i) if *i >= 0 => u32::try_from(*i).ok(),
            _ => None,
        }
    }
    pub fn get_u64(&self, section: &str, key: &str) -> Option<u64> {
        match self.table(section)?.get(key)? {
            Value::Int(i) if *i >= 0 => Some(*i as u64),
            _ => None,
        }
    }
    pub fn get_f64(&self, section: &str, key: &str) -> Option<f64> {
        match self.table(section)?.get(key)? {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }
    pub fn get_str_list(&self, section: &str, key: &str) -> Option<Vec<String>> {
        match self.table(section)?.get(key)? {
            Value::Array(items) => Some(
                items
                    .iter()
                    .filter_map(|v| match v {
                        Value::Str(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect(),
            ),
            _ => None,
        }
    }
}

struct Parser<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Parser<'a> {
        Parser { chars: src.chars().peekable(), line: 1 }
    }

    fn err(&self, msg: &str) -> Error {
        Error::config(format!("line {}: {}", self.line, msg))
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next();
        if c == Some('\n') {
            self.line += 1;
        }
        c
    }

    fn skip_inline_whitespace(&mut self) {
        while matches!(self.chars.peek(), Some(' ') | Some('\t')) {
            self.bump();
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        let mut s = String::new();
        loop {
            match self.bump() {
                None => return Err(self.err("unterminated string")),
                Some('"') => return Ok(s),
                Some('\\') => match self.bump() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('r') => s.push('\r'),
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some(other) => return Err(self.err(&format!("unknown escape '\\{}'", other))),
                    None => return Err(self.err("unterminated escape at end of file")),
                },
                Some(c) => s.push(c),
            }
        }
    }

    fn parse_bare_token(&mut self) -> String {
        let mut s = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '+' {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        s
    }

    fn parse_value(&mut self) -> Result<Value> {
        self.skip_inline_whitespace();
        match self.chars.peek() {
            Some('"') => {
                self.bump();
                Ok(Value::Str(self.parse_string()?))
            }
            Some('[') => {
                self.bump();
                let mut items = Vec::new();
                loop {
                    self.skip_ws_and_comments_and_newlines();
                    if self.chars.peek() == Some(&']') {
                        self.bump();
                        break;
                    }
                    items.push(self.parse_value()?);
                    self.skip_ws_and_comments_and_newlines();
                    match self.chars.peek() {
                        Some(',') => {
                            self.bump();
                        }
                        Some(']') => {
                            self.bump();
                            break;
                        }
                        _ => return Err(self.err("expected ',' or ']' in array")),
                    }
                }
                Ok(Value::Array(items))
            }
            Some(_) => {
                let tok = self.parse_bare_token();
                if tok.is_empty() {
                    return Err(self.err("expected a value"));
                }
                parse_scalar_token(&tok).ok_or_else(|| self.err(&format!("invalid value '{}'", tok)))
            }
            None => Err(self.err("expected a value, found end of file")),
        }
    }

    fn skip_ws_and_comments_and_newlines(&mut self) {
        loop {
            match self.chars.peek() {
                Some(' ') | Some('\t') | Some('\n') | Some('\r') => {
                    self.bump();
                }
                Some('#') => {
                    while !matches!(self.chars.peek(), None | Some('\n')) {
                        self.bump();
                    }
                }
                _ => break,
            }
        }
    }

    fn parse(mut self) -> Result<Document> {
        let mut doc = Document::default();
        doc.sections.insert(TOP_LEVEL.to_string(), HashMap::new());
        let mut current_section = TOP_LEVEL.to_string();
        loop {
            self.skip_inline_whitespace();
            match self.chars.peek() {
                None => break,
                Some('\n') | Some('\r') => {
                    self.bump();
                    continue;
                }
                Some('#') => {
                    while !matches!(self.chars.peek(), None | Some('\n')) {
                        self.bump();
                    }
                    continue;
                }
                Some('[') => {
                    self.bump();
                    let name = self.parse_bare_token();
                    self.skip_inline_whitespace();
                    if self.bump() != Some(']') {
                        return Err(self.err("expected ']' after section name"));
                    }
                    if name.is_empty() {
                        return Err(self.err("empty section name"));
                    }
                    doc.sections.entry(name.clone()).or_default();
                    current_section = name;
                    self.expect_end_of_line()?;
                }
                Some(_) => {
                    let key = self.parse_bare_token();
                    if key.is_empty() {
                        return Err(self.err("expected a key, a section header, or a comment"));
                    }
                    self.skip_inline_whitespace();
                    if self.bump() != Some('=') {
                        return Err(self.err(&format!("expected '=' after key '{}'", key)));
                    }
                    let value = self.parse_value()?;
                    self.expect_end_of_line()?;
                    doc.sections.get_mut(&current_section).unwrap().insert(key, value);
                }
            }
        }
        Ok(doc)
    }

    fn expect_end_of_line(&mut self) -> Result<()> {
        self.skip_inline_whitespace();
        match self.chars.peek() {
            Some('#') => {
                while !matches!(self.chars.peek(), None | Some('\n')) {
                    self.bump();
                }
                Ok(())
            }
            Some('\n') | None => {
                self.bump();
                Ok(())
            }
            Some(c) => Err(self.err(&format!("unexpected trailing character '{}'", c))),
        }
    }
}

fn parse_scalar_token(tok: &str) -> Option<Value> {
    match tok {
        "true" => return Some(Value::Bool(true)),
        "false" => return Some(Value::Bool(false)),
        _ => {}
    }
    if let Ok(i) = tok.parse::<i64>() {
        return Some(Value::Int(i));
    }
    if tok.contains('.') {
        if let Ok(f) = tok.parse::<f64>() {
            return Some(Value::Float(f));
        }
    }
    None
}

pub fn parse(src: &str) -> Result<Document> {
    Parser::new(src).parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_top_level_and_sections() {
        let doc = parse(concat!(
            "# a comment\n",
            "answer = 42\n",
            "\n",
            "[network]\n",
            "enabled = true\n",
            "name = \"eth0\"\n",
            "ratio = 1.5\n",
        ))
        .unwrap();
        assert_eq!(doc.get_u32(TOP_LEVEL, "answer"), Some(42));
        assert_eq!(doc.get_bool("network", "enabled"), Some(true));
        assert_eq!(doc.get_str("network", "name"), Some("eth0"));
        assert_eq!(doc.get_f64("network", "ratio"), Some(1.5));
    }

    #[test]
    fn parses_string_array_with_trailing_comma() {
        let doc = parse("shells = [\"/bin/false\", \"/usr/sbin/nologin\",]\n").unwrap();
        assert_eq!(
            doc.get_str_list(TOP_LEVEL, "shells"),
            Some(vec!["/bin/false".to_string(), "/usr/sbin/nologin".to_string()])
        );
    }

    #[test]
    fn parses_multiline_array() {
        let doc = parse("names = [\n  \"a\",\n  \"b\",\n]\n").unwrap();
        assert_eq!(doc.get_str_list(TOP_LEVEL, "names"), Some(vec!["a".to_string(), "b".to_string()]));
    }

    #[test]
    fn string_escapes_round_trip() {
        let doc = parse("s = \"line1\\nline2\\t\\\"quoted\\\"\"\n").unwrap();
        assert_eq!(doc.get_str(TOP_LEVEL, "s"), Some("line1\nline2\t\"quoted\""));
    }

    #[test]
    fn rejects_unterminated_string() {
        assert!(parse("s = \"oops\n").is_err());
    }

    #[test]
    fn rejects_missing_equals() {
        assert!(parse("key value\n").is_err());
    }

    #[test]
    fn rejects_trailing_garbage_on_a_line() {
        assert!(parse("key = 1 garbage\n").is_err());
    }

    #[test]
    fn negative_integers_and_floats() {
        let doc = parse("a = -5\nb = -1.5\n").unwrap();
        assert_eq!(doc.table(TOP_LEVEL).unwrap().get("a"), Some(&Value::Int(-5)));
        assert_eq!(doc.table(TOP_LEVEL).unwrap().get("b"), Some(&Value::Float(-1.5)));
    }

    #[test]
    fn last_file_without_trailing_newline_still_parses() {
        let doc = parse("a = 1").unwrap();
        assert_eq!(doc.get_u32(TOP_LEVEL, "a"), Some(1));
    }
}
