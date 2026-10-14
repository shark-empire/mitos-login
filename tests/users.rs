use mitos_login::users::manager::{UserListPolicy, UserManager};

fn write_passwd(content: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "mitos-login-it-users-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()
    ));
    std::fs::write(&p, content).unwrap();
    p
}

#[test]
fn visible_users_respects_policy_through_the_public_api() {
    let p = write_passwd(concat!(
        "root:x:0:0:root:/root:/bin/bash\n",
        "bin:x:1:1:bin:/bin:/usr/sbin/nologin\n",
        "mitos-greeter:x:900:900:greeter:/var/lib/mitos-login:/usr/sbin/nologin\n",
        "alice:x:1000:1000:Alice Rivera,,,:/home/alice:/bin/bash\n",
        "service:x:70000:70000:service,,,:/nonexistent:/usr/sbin/nologin\n",
    ));
    let mgr = UserManager::new(p.clone(), UserListPolicy::default());
    let visible = mgr.visible_users().unwrap();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].name, "alice");
    assert_eq!(visible[0].display_name, "Alice Rivera");
    std::fs::remove_file(&p).ok();
}

#[test]
fn resolve_for_login_finds_root_even_though_it_is_never_visible() {
    let p = write_passwd("root:x:0:0:root:/root:/bin/bash\n");
    let mgr = UserManager::new(p.clone(), UserListPolicy::default());
    assert!(mgr.visible_users().unwrap().is_empty());
    let root = mgr.resolve_for_login("root").unwrap();
    assert_eq!(root.map(|u| u.uid), Some(0));
    std::fs::remove_file(&p).ok();
}
