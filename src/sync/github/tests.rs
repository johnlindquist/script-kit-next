use super::*;
use std::cell::{Cell, RefCell};

#[test]
fn workspace_snapshot_never_exports_the_vault_staged_files_or_history() {
    let work = tempfile::tempdir().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let w = work.path();
    std::fs::create_dir_all(w.join("plugins/main/scripts")).unwrap();
    for (p, b) in [
        ("plugins/main/scripts/hello.ts", "export {}"),
        ("secrets.age", "fixture-ciphertext"),
        ("agent-token", "fixture"),
        ("plugins/main/scripts/private.ts", "private"),
        (".gitignore", "**/private.ts\n"),
    ] {
        std::fs::write(w.join(p), b).unwrap();
    }
    assert!(git(w, &["init", "--template=", "-q"]).unwrap().0);
    assert!(
        git(
            w,
            &[
                "add",
                "-f",
                "secrets.age",
                "agent-token",
                "plugins/main/scripts/private.ts"
            ]
        )
        .unwrap()
        .0
    );
    let before = git(w, &["ls-files", "-s"]).unwrap().1;
    let files = snapshot::collect(w, tmp.path()).unwrap();
    assert_eq!(
        files,
        vec![("plugins/main/scripts/hello.ts".into(), "export {}".into())]
    );
    assert_eq!(git(w, &["ls-files", "-s"]).unwrap().1, before);
    assert_eq!(
        std::fs::read_to_string(w.join(".gitignore")).unwrap(),
        "**/private.ts\n"
    );
}
#[test]
#[cfg(unix)]
fn allowed_looking_link_cannot_read_an_external_file() {
    let work = tempfile::tempdir().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(work.path().join("plugins/main/scripts")).unwrap();
    let secret = tmp.path().join("fixture");
    std::fs::write(&secret, "never export me").unwrap();
    std::os::unix::fs::symlink(secret, work.path().join("plugins/main/scripts/indirect.ts"))
        .unwrap();
    assert!(snapshot::collect(work.path(), tmp.path()).is_err());
}
#[test]
fn destination_has_one_exact_github_identity() {
    assert_eq!(
        github_name("git@github.com:example/personal.git").unwrap(),
        "example/personal"
    );
    for bad in [
        "https://elsewhere.example/u/r",
        "https://github.com/u/r?key=x",
        "https://u@github.com/o/r",
        "https://github.com/u/../r",
        "https://github.com/-u/r",
        "file:///local",
    ] {
        assert!(github_name(bad).is_err());
    }
}
fn repo(private: bool) -> Value {
    json!({"id":42,"full_name":"owner/kit","private":private,"visibility":if private{"private"}else{"public"},"permissions":{"push":true}})
}
#[test]
fn private_destination_is_rechecked_before_sending_any_file() {
    for first in [true, false] {
        let n = Cell::new(0);
        let result = publish(
            &[("plugins/main/scripts/s.ts".into(), "payload".into())],
            Some("owner/kit"),
            |m, p, b| {
                assert_eq!(m, "GET");
                assert!(b.is_none());
                if p == "repos/owner/kit" {
                    n.set(n.get() + 1);
                    Ok((200, repo(first && n.get() == 1)))
                } else {
                    Ok((404, Value::Null))
                }
            },
        );
        assert!(result.is_err());
    }
}
#[test]
fn private_snapshot_creates_only_the_backup_ref_and_requires_readback() {
    for right in [false, true] {
        let moved = Cell::new(false);
        let requests = RefCell::new(Vec::new());
        let id = "a".repeat(40);
        let tr = "b".repeat(40);
        let result = publish(
            &[("plugins/main/scripts/s.ts".into(), "payload".into())],
            Some("owner/kit"),
            |m, p, b| {
                requests.borrow_mut().push((m.to_string(), p.to_string()));
                if p == "repos/owner/kit" {
                    return Ok((200, repo(true)));
                }
                if p.ends_with("/trees") {
                    let t = b.unwrap();
                    assert_eq!(t["tree"][0]["content"], "payload");
                    return Ok((201, json!({"sha":tr})));
                }
                if p.ends_with("/commits") {
                    assert_eq!(b.unwrap()["parents"], json!([]));
                    return Ok((201, json!({"sha":id})));
                }
                if p.ends_with("/refs") {
                    assert_eq!(b.unwrap()["ref"], format!("refs/heads/{BRANCH}"));
                    moved.set(true);
                    return Ok((201, json!({})));
                }
                if p.ends_with(&format!("/ref/heads/{BRANCH}")) {
                    return if moved.get() {
                        Ok((200, json!({"object":{"sha":if right{&id}else{&tr}}})))
                    } else {
                        Ok((404, Value::Null))
                    };
                }
                panic!("unexpected request")
            },
        );
        assert_eq!(result.is_ok(), right);
        assert!(moved.get());
    }
}
