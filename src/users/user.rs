/// One account, as shown on the greeter and used to launch a session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub name: String,
    pub uid: u32,
    pub gid: u32,
    /// The comma-separated GECOS field's first component, if present and
    /// plausible; falls back to `name`.
    pub display_name: String,
    pub home: String,
    pub shell: String,
}

impl User {
    pub fn from_passwd_line(name: &str, uid: u32, gid: u32, gecos: &str, home: &str, shell: &str) -> User {
        let display_name = gecos
            .split(',')
            .next()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or(name)
            .to_string();
        User {
            name: name.to_string(),
            uid,
            gid,
            display_name,
            home: home.to_string(),
            shell: shell.to_string(),
        }
    }

    pub fn initial(&self) -> char {
        self.display_name.chars().next().unwrap_or('?').to_ascii_uppercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gecos_first_field_becomes_display_name() {
        let u = User::from_passwd_line("alice", 1000, 1000, "Alice Rivera,,,", "/home/alice", "/bin/sh");
        assert_eq!(u.display_name, "Alice Rivera");
        assert_eq!(u.initial(), 'A');
    }

    #[test]
    fn empty_gecos_falls_back_to_username() {
        let u = User::from_passwd_line("bob", 1001, 1001, "", "/home/bob", "/bin/sh");
        assert_eq!(u.display_name, "bob");
    }
}
