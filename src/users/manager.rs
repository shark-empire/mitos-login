//! The greeter-facing user list: system accounts filtered to plausible human
//! logins, using the same UID floor/shell-denylist convention distros use for
//! "don't show system accounts on the login screen".
use super::enumeration;
use super::user::User;
use crate::errors::Result;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct UserListPolicy {
    pub min_uid: u32,
    pub max_uid: u32,
    pub denied_shells: Vec<String>,
    /// Usernames always excluded regardless of UID (e.g. the daemon's own
    /// unprivileged greeter service account).
    pub always_hide: Vec<String>,
    /// If non-empty, only these usernames are ever shown (an allowlist,
    /// e.g. for a shared kiosk machine); empty means "show everyone the
    /// other rules allow".
    pub allow_only: Vec<String>,
}

impl Default for UserListPolicy {
    fn default() -> UserListPolicy {
        UserListPolicy {
            min_uid: 1000,
            max_uid: 60000,
            denied_shells: vec![
                "/usr/sbin/nologin".into(),
                "/sbin/nologin".into(),
                "/bin/false".into(),
                "/usr/bin/false".into(),
            ],
            always_hide: vec!["mitos-greeter".into()],
            allow_only: Vec::new(),
        }
    }
}

pub struct UserManager {
    passwd_path: PathBuf,
    policy: UserListPolicy,
}

impl UserManager {
    pub fn new(passwd_path: PathBuf, policy: UserListPolicy) -> UserManager {
        UserManager { passwd_path, policy }
    }

    /// The list the greeter shows. Sorted by display name for a stable,
    /// predictable UI.
    pub fn visible_users(&self) -> Result<Vec<User>> {
        let all = enumeration::read_all(&self.passwd_path)?;
        let mut out: Vec<User> = all
            .into_iter()
            .filter(|u| self.is_visible(u))
            .collect();
        out.sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));
        Ok(out)
    }

    fn is_visible(&self, u: &User) -> bool {
        if self.policy.always_hide.iter().any(|n| n == &u.name) {
            return false;
        }
        if !self.policy.allow_only.is_empty() {
            return self.policy.allow_only.iter().any(|n| n == &u.name);
        }
        if u.uid < self.policy.min_uid || u.uid > self.policy.max_uid {
            return false;
        }
        if self.policy.denied_shells.iter().any(|s| s == &u.shell) {
            return false;
        }
        true
    }

    /// Resolves any account by name for authentication purposes, including
    /// ones NSS provides that would never appear in [`Self::visible_users`]
    /// (e.g. `root`, typed manually) — this is intentionally more permissive
    /// than the visible list, matching how every mainstream login manager
    /// treats "shown by default" and "can type it and log in" as separate
    /// questions.
    pub fn resolve_for_login(&self, name: &str) -> Result<Option<User>> {
        match crate::sys::getpwnam(name)? {
            Some(p) => Ok(Some(User::from_passwd_line(&p.name, p.uid, p.gid, &p.gecos, &p.dir, &p.shell))),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_passwd(content: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "mitos-login-usermgr-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()
        ));
        std::fs::write(&p, content).unwrap();
        p
    }

    #[test]
    fn hides_system_and_nologin_accounts() {
        let p = write_passwd(concat!(
            "root:x:0:0:root:/root:/bin/bash\n",
            "daemon:x:1:1:daemon:/:/usr/sbin/nologin\n",
            "alice:x:1000:1000:Alice,,,:/home/alice:/bin/bash\n",
            "mitos-greeter:x:900:900:greeter:/var/lib/mitos-login:/usr/sbin/nologin\n",
            "bob:x:1001:1001:Bob,,,:/home/bob:/bin/false\n",
        ));
        let mgr = UserManager::new(p.clone(), UserListPolicy::default());
        let visible = mgr.visible_users().unwrap();
        let names: Vec<&str> = visible.iter().map(|u| u.name.as_str()).collect();
        assert_eq!(names, vec!["alice"]);
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn allow_only_overrides_uid_range() {
        let p = write_passwd(concat!(
            "root:x:0:0:root:/root:/bin/bash\n",
            "alice:x:1000:1000:Alice,,,:/home/alice:/bin/bash\n",
        ));
        let mut policy = UserListPolicy::default();
        policy.allow_only = vec!["root".to_string()];
        let mgr = UserManager::new(p.clone(), policy);
        let visible = mgr.visible_users().unwrap();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].name, "root");
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn sorted_case_insensitively_by_display_name() {
        let p = write_passwd(concat!(
            "zed:x:1002:1002:zed user,,,:/home/zed:/bin/bash\n",
            "amy:x:1000:1000:Amy,,,:/home/amy:/bin/bash\n",
            "bob:x:1001:1001:bob,,,:/home/bob:/bin/bash\n",
        ));
        let mgr = UserManager::new(p.clone(), UserListPolicy::default());
        let visible = mgr.visible_users().unwrap();
        let names: Vec<&str> = visible.iter().map(|u| u.name.as_str()).collect();
        assert_eq!(names, vec!["amy", "bob", "zed"]);
        std::fs::remove_file(&p).ok();
    }
}
