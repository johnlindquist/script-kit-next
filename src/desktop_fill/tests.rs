use super::*;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
struct Fake {
    target: Mutex<Window>,
    text: Mutex<String>,
    tree: Mutex<Value>,
    writes: AtomicUsize,
    unverified: bool,
}
fn win() -> Window {
    Window {
        pid: 9,
        app: "Editor".into(),
        title: "Document".into(),
        window_id: "w-42".into(),
        process_instance: "start:123".into(),
    }
}
fn node() -> Value {
    json!({"role":"textfield","name":"Save As:","ref_id":"@s42:e2","available_actions":["SetValue","Click"]})
}
fn fixture() -> Service<Fake> {
    Service::new(Fake {
        target: Mutex::new(win()),
        text: Mutex::new("Untitled".into()),
        tree: Mutex::new(
            json!({"app":"Editor","window":{"id":"w-42"},"complete":true,"snapshot_id":"s42","tree":{"role":"sheet","children":[node(),{"role":"button","name":"Save","ref_id":"@s42:e3","available_actions":["Click"]}]}}),
        ),
        writes: 0.into(),
        unverified: false,
    })
}
impl Backend for Fake {
    fn windows(&self, _: &str) -> Result<Vec<Window>, Failure> {
        Ok(vec![self.target.lock().clone()])
    }
    fn sheet(&self, _: &Window) -> Result<Value, Failure> {
        Ok(self.tree.lock().clone())
    }
    fn value(&self, r: &str) -> Result<String, Failure> {
        assert_eq!(r, "@s42:e2");
        Ok(self.text.lock().clone())
    }
    fn set_value(&self, r: &str, v: &str) -> Result<(), Failure> {
        assert_eq!(r, "@s42:e2");
        self.writes.fetch_add(1, Ordering::SeqCst);
        *self.text.lock() = v.into();
        if self.unverified {
            Err("delivery_unverified")
        } else {
            Ok(())
        }
    }
}
fn proposal(s: &Service<Fake>) -> Uuid {
    s.propose("launcher", "Editor", 9, 42, None, "Workshop outline")
        .unwrap()
        .proposal_id
}
#[test]
fn scoped_agent_proposes_applies_and_reads_the_same_receipt() {
    // The owned service belongs to the library. A binary test links the production
    // library, where that fixture must not exist. Drive the same MCP adapter here.
    use super::owned as fixture;
    use crate::mcp_protocol::desktop_fill::call_with;
    const SCOPE: &str = "desktop:fill";
    const PROPOSE: &str = "desktop/filename_propose";
    const APPLY: &str = "desktop/filename_apply";
    const STATUS: &str = "desktop/filename_status";
    let invoke = |owner: &str, name: &str, args: Value| {
        let envelope = smol::block_on(call_with(
            fixture::api(),
            name,
            args,
            Some(owner.into()),
            &[SCOPE.into()],
        ));
        serde_json::from_str::<Value>(envelope["content"][0]["text"].as_str().unwrap()).unwrap()
    };
    let p = invoke(
        "agent-a",
        PROPOSE,
        json!({"app":fixture::APP,"pid":fixture::PID,"nativeWindowId":fixture::WINDOW,"filename":"Agent outline"}),
    );
    assert_eq!(p["state"], "proposed");
    let h = json!({"proposalId":p["proposal"]["proposalId"]});
    assert_eq!(
        invoke("agent-b", APPLY, h.clone())["error"]["code"],
        "unknown_or_expired_proposal"
    );
    assert_eq!(
        invoke("agent-a", STATUS, h.clone())["result"]["state"],
        "proposed"
    );
    assert_eq!(
        invoke("agent-a", APPLY, h.clone())["result"]["state"],
        "verified"
    );
    assert_eq!(invoke("agent-a", STATUS, h)["result"]["state"], "verified");
}

#[test]
fn fill_is_single_use_and_bound_to_caller() {
    let s = fixture();
    let id = proposal(&s);
    assert!(s.apply("different", id).is_err());
    assert!(matches!(s.status("launcher", id), Ok(Status::Proposed)));
    for _ in 0..2 {
        assert!(matches!(s.apply("launcher", id), Ok(Status::Verified)))
    }
    assert_eq!(s.backend.writes.load(Ordering::SeqCst), 1);
    assert_eq!(*s.backend.text.lock(), "Workshop outline");
}
#[test]
fn same_field_edited_before_apply_is_preserved() {
    let s = fixture();
    let id = proposal(&s);
    *s.backend.text.lock() = "My edit".into();
    assert!(matches!(
        s.apply("launcher", id),
        Ok(Status::Refused {
            code: "field_changed"
        })
    ));
    assert_eq!(s.backend.writes.load(Ordering::SeqCst), 0);
}
#[test]
fn recycled_window_or_expired_preview_is_refused() {
    let s = fixture();
    let id = proposal(&s);
    s.backend.target.lock().process_instance = "replacement".into();
    assert!(matches!(
        s.apply("launcher", id),
        Ok(Status::Refused {
            code: "window_changed"
        })
    ));
    let id = proposal(&s);
    s.entries.lock().get_mut(&id).unwrap().issued = Instant::now() - 2 * TTL;
    assert!(s.apply("launcher", id).is_err());
    assert_eq!(s.backend.writes.load(Ordering::SeqCst), 0);
}
#[test]
fn uncertain_delivery_is_not_retried() {
    let mut s = fixture();
    s.backend.unverified = true;
    let id = proposal(&s);
    for _ in 0..2 {
        assert!(matches!(
            s.apply("launcher", id),
            Ok(Status::Uncertain { .. })
        ))
    }
    assert_eq!(s.backend.writes.load(Ordering::SeqCst), 1);
}
#[test]
fn reject_wrong_window_incomplete_tree_hidden_ambiguous_or_non_field() {
    let s = fixture();
    let orig = s.backend.tree.lock().clone();
    for i in 0..6 {
        let mut snap = orig.clone();
        match i {
            0 => snap["window"]["id"] = json!("w-41"),
            1 => snap["complete"] = json!(false),
            2 => snap["tree"]["children"][0]["states"] = json!(["offscreen"]),
            3 => snap["tree"]["children"]
                .as_array_mut()
                .unwrap()
                .push(node()),
            4 => snap["tree"]["children"][0]["role"] = json!("button"),
            _ => snap["tree"]["children"][0]["ref_id"] = json!("@another:e2"),
        };
        assert!(sheet::filename_field(&snap, &win()).is_err());
    }
}
