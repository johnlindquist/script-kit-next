#![cfg(unix)]
use serde_json::{Value, json};
use sk_one_window::personal::{Edit, SelectionAction, TextRole, Weight};
use sk_personal::{Author, Change, Error, Request, State, Store, api};
use std::{fs, path::Path};
use uuid::Uuid;
fn ask(view: &State, change: Change) -> Request {
    Request {
        workspace: view.workspace.clone(),
        expected_revision: view.revision.clone(),
        request_id: Uuid::new_v4(),
        change,
    }
}
fn api(root: &Path, q: Value) -> sk_personal::Result<Value> {
    api::answer(root, q.to_string().as_bytes(), false)
}
#[test]
fn an_external_author_discovers_then_applies_the_same_rules_the_launcher_reads_and_undoes() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path();
    let store = Store::open(p).unwrap();
    assert!(
        api(p, json!({"op":"describe"})).unwrap()["targets"]
            .as_array()
            .unwrap()
            .len()
            >= 4
    );
    let stock = store.inspect().unwrap();
    assert!(!p.join("db").exists() && !p.join("plugins").exists());
    let bright = ask(
        &stock,
        Change::Edit {
            edit: Edit::PinText(TextRole::Primary),
        },
    );
    let saved = api(p, json!({"op":"apply","request":bright})).unwrap();
    let a = store.inspect().unwrap();
    assert_eq!(a.effective.pin(false).text_role, TextRole::Primary);
    assert_eq!(a.revision, saved["result"]["appliedRevision"]);
    let b = store
        .apply(
            ask(
                &a,
                Change::Edit {
                    edit: Edit::PinWeight(Weight::SemiboldWhileDictating),
                },
            ),
            Author::Launcher,
        )
        .unwrap()
        .state;
    let edit = Edit::SelectionFirst(Some(SelectionAction::Note));
    let c = store
        .apply(ask(&b, Change::Edit { edit }), Author::Launcher)
        .unwrap()
        .state;
    assert_eq!(c.effective.pin(false).text_role, TextRole::Primary);
    assert!(!c.effective.pin(false).semibold && c.effective.pin(true).semibold);
    assert!(
        store
            .apply(bright.clone(), Author::LocalAgent)
            .unwrap()
            .replayed
    );
    assert_eq!(
        store
            .apply(bright.clone(), Author::LocalAgent)
            .unwrap()
            .state,
        c
    );
    let mut mismatch = bright.clone();
    mismatch.change = Change::Disable;
    assert_eq!(
        store.apply(mismatch, Author::LocalAgent).unwrap_err(),
        Error::RequestConflict
    );
    assert_eq!(
        store
            .apply(ask(&a, Change::Disable), Author::Launcher)
            .unwrap_err(),
        Error::StaleRevision
    );
    assert_eq!(
        store.why(Edit::PinText(TextRole::Muted)).unwrap()["requestId"],
        bright.request_id.to_string()
    );
    let restarted = Store::open(p).unwrap();
    assert_eq!(restarted.inspect().unwrap(), c);
    assert_eq!(
        restarted
            .apply(
                ask(
                    &c,
                    Change::Undo {
                        of: bright.request_id
                    }
                ),
                Author::Launcher
            )
            .unwrap_err(),
        Error::NothingToUndo
    );
    let off = restarted
        .apply(ask(&c, Change::Disable), Author::Launcher)
        .unwrap()
        .state;
    assert_ne!(off.declared, off.effective);
    assert_eq!(off.effective, stock.effective);
    let mut cur = off;
    while let Some(of) = cur.undo {
        cur = Store::open(p)
            .unwrap()
            .apply(ask(&cur, Change::Undo { of }), Author::Launcher)
            .unwrap()
            .state;
    }
    assert_eq!(cur.effective, stock.effective);
    assert_eq!(cur.declared, stock.declared);
    assert_eq!(
        api(p, json!({"op":"status","requestId":bright.request_id})).unwrap()["status"],
        "committed"
    );
    assert_eq!(
        api(
            p,
            json!({"op":"why","target":"launcher.header.pinned-label.textRole"})
        )
        .unwrap()["origin"],
        "stock"
    );
    let active = fs::read_to_string(p.join("plugins/main/customizations/launcher.json")).unwrap();
    assert!(!active.contains("localAgent") && !active.contains("expectedRevision"));
    assert_eq!(
        api::answer(
            p,
            json!({"op":"apply","request":ask(&cur,Change::Disable)})
                .to_string()
                .as_bytes(),
            true
        ),
        Err(Error::SafeMode)
    );
}
#[test]
fn a_stale_writer_or_unsafe_path_cannot_edit_its_neighbor() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let s = Store::open(a.path()).unwrap();
    let before = s.inspect().unwrap();
    let request = ask(
        &before,
        Change::Edit {
            edit: Edit::PinText(TextRole::Primary),
        },
    );
    assert_eq!(
        Store::open(b.path())
            .unwrap()
            .apply(request.clone(), Author::Launcher)
            .unwrap_err(),
        Error::WrongWorkspace
    );
    std::os::unix::fs::symlink(b.path(), a.path().join("plugins")).unwrap();
    assert_eq!(
        s.apply(request, Author::Launcher).unwrap_err(),
        Error::UnsafePath
    );
    assert_eq!(fs::read_dir(b.path()).unwrap().count(), 0);
    fs::remove_file(a.path().join("plugins")).unwrap();
    let mut wins = 0;
    std::thread::scope(|threads| {
        let mut jobs = vec![];
        for _ in 0..4 {
            let p = a.path();
            let q = ask(&before, Change::Disable);
            jobs.push(threads.spawn(move || Store::open(p).unwrap().apply(q, Author::LocalAgent)));
        }
        for j in jobs {
            match j.join().unwrap() {
                Ok(_) => wins += 1,
                Err(Error::WriterBusy | Error::StaleRevision) => {}
                res => panic!("unexpected outcome {res:?}"),
            }
        }
    });
    assert_eq!(wins, 1);
    let restarted = s.inspect().unwrap();
    let file = a.path().join("plugins/main/customizations/launcher.json");
    let outside = b.path().join("must-stay");
    fs::write(&outside, b"original").unwrap();
    fs::remove_file(&file).unwrap();
    fs::hard_link(&outside, &file).unwrap();
    assert_eq!(
        s.apply(ask(&restarted, Change::Enable), Author::LocalAgent)
            .unwrap_err(),
        Error::UnsafePath
    );
    assert_eq!(fs::read(&outside).unwrap(), b"original");
}
#[test]
fn an_unpublished_journal_record_is_not_a_committed_request() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path();
    let s = Store::open(p).unwrap();
    let stock = s.inspect().unwrap();
    let req = ask(&stock, Change::Disable);
    let success = s.apply(req.clone(), Author::Launcher).unwrap();
    let active = p.join("plugins/main/customizations/launcher.json");
    fs::remove_file(&active).unwrap(); // simulate a death after the private record, before pointer publication
    assert_eq!(
        s.status(req.request_id).unwrap()["status"],
        "notInActiveHistory"
    );
    let resumed = s.apply(req.clone(), Author::Launcher).unwrap();
    assert!(!resumed.replayed);
    assert_eq!(resumed.applied_revision, success.applied_revision);
    let entry = p
        .join("db/personal-changes")
        .join(format!("{}.json", req.request_id));
    let mut tampered: Value = serde_json::from_slice(&fs::read(&entry).unwrap()).unwrap();
    tampered["author"] = json!("localAgent");
    fs::write(entry, tampered.to_string()).unwrap();
    assert_eq!(s.inspect().unwrap_err(), Error::InvalidDocument);
}
#[test]
fn a_minimal_home_is_a_reversible_workspace_preference_not_a_search() {
    use sk_personal::speech::{Reading, read};
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path();
    let store = Store::open(p).unwrap();
    let stock = store.inspect().unwrap();
    let Some(Reading::Change(change)) = read("Kit, please keep my launcher minimal!") else {
        panic!("no scoped edit")
    };
    let request = ask(&stock, change);
    store.apply(request.clone(), Author::Launcher).unwrap();
    drop(store);
    let reopened = Store::open(p).unwrap();
    let current = reopened.inspect().unwrap();
    assert_eq!(
        api(p, json!({"op":"inspect"})).unwrap()["state"]["effective"]["homeMode"],
        "minimal"
    );
    assert_eq!(
        api(p, json!({"op":"why","target":"launcher.home.mode"})).unwrap()["requestId"],
        request.request_id.to_string()
    );
    let off = reopened
        .apply(ask(&current, Change::Disable), Author::Launcher)
        .unwrap()
        .state;
    assert_eq!(off.effective, stock.effective);
    assert_eq!(off.declared, current.declared);
    let on = reopened
        .apply(
            ask(
                &off,
                Change::Undo {
                    of: off.undo.unwrap(),
                },
            ),
            Author::Launcher,
        )
        .unwrap()
        .state;
    assert_eq!(on.effective, current.effective);
    let undone = reopened
        .apply(
            ask(
                &on,
                Change::Undo {
                    of: request.request_id,
                },
            ),
            Author::Launcher,
        )
        .unwrap()
        .state;
    assert_eq!(undone.effective, stock.effective);
    assert_eq!(
        api(p, json!({"op":"inspect"})).unwrap()["state"]["effective"]["homeMode"],
        "standard"
    );
    assert!(read("write a note: keep my launcher minimal").is_none());
}

#[test]
fn local_readings_require_a_complete_scoped_request() {
    use sk_personal::speech::{Reading, read};
    assert_eq!(
        read("Make ‘Stays open’ brighter"),
        Some(Reading::Change(Change::Edit {
            edit: Edit::PinText(TextRole::Primary)
        }))
    );
    assert!(matches!(
        read("For highlighted text, put save to today first"),
        Some(Reading::Change(Change::Edit {
            edit: Edit::SelectionFirst(Some(SelectionAction::Note))
        }))
    ));
    for s in [
        "do not make the pinned label brighter",
        "send Pat: make the pinned label brighter",
        "make the pinned label brighter and email my history",
        "put open first for selected text",
    ] {
        assert_eq!(read(s), None);
    }
}
