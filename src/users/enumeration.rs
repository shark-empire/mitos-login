//! Direct `/etc/passwd` parsing for the greeter's visible user list.
use super::user::User;
use crate::errors::Result;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Parses one well-formed `/etc/passwd` line. A malformed line (wrong field
/// count, non-numeric uid/gid) is skipped, not treated as a fatal error —
/// one bad entry must never make the whole login screen fail to list anyone.
fn parse_line(line: &str) -> Option<User> {
    let mut f = line.splitn(7, ':');
    let name = f.next()?;
    let _passwd = f.next()?;
    let uid: u32 = f.next()?.parse().ok()?;
    let gid: u32 = f.next()?.parse().ok()?;
    let gecos = f.next().unwrap_or("");
    let home = f.next()?;
    let shell = f.next().unwrap_or("").trim_end();
    if name.is_empty() {
        return None;
    }
    Some(User::from_passwd_line(name, uid, gid, gecos, home, shell))
}

pub fn read_all(path: &Path) -> Result<Vec<User>> {
    let file = std::fs::File::open(path)?;
    let mut out = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(u) = parse_line(&line) {
            out.push(u);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_lines() {
        let u = parse_line("alice:x:1000:1000:Alice Rivera,,,:/home/alice:/bin/sh").unwrap();
        assert_eq!(u.name, "alice");
        assert_eq!(u.uid, 1000);
        assert_eq!(u.display_name, "Alice Rivera");
    }

    #[test]
    fn skips_malformed_lines_without_failing() {
        assert!(parse_line("not:enough:fields").is_none());
        assert!(parse_line("bob:x:notanumber:1000::/home/bob:/bin/sh").is_none());
        assert!(parse_line("").is_none());
    }

    #[test]
    fn read_all_skips_bad_lines_and_keeps_good_ones() {
        let p = std::env::temp_dir().join(format!("mitos-login-passwd-test-{}", std::process::id()));
        std::fs::write(&p, "root:x:0:0:root:/root:/bin/sh\nbroken:line\nalice:x:1000:1000:Alice,,,:/home/alice:/bin/sh\n").unwrap();
        let users = read_all(&p).unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[1].name, "alice");
        std::fs::remove_file(&p).ok();
    }
}
