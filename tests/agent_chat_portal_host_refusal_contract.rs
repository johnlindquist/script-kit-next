use std::{fs, path::Path};

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[test]
fn portal_open_result_models_host_refusal() {
    let source = read("src/ai/agent_chat/ui/view.rs");
    for needle in [
        "pub(crate) enum PortalOpenResult",
        "Refused(PortalRefusal)",
        "PortalRefusal::NoHost",
        "PortalRefusal::UnsupportedByHost",
        "open_portal_contract_result",
    ] {
        assert!(source.contains(needle), "missing {needle}");
    }
}

#[test]
fn portal_session_is_not_staged_before_host_decision() {
    let source = read("src/ai/agent_chat/ui/view.rs");
    let body = source
        .split("fn open_portal_contract_result")
        .nth(1)
        .expect("open_portal_contract_result exists");
    let decide_idx = body.find("decide_portal_open").unwrap();
    let callback_idx = body.find("let Some(callback)").unwrap();
    let stage_idx = body.find("self.stage_pending_portal_session").unwrap();
    assert!(decide_idx < callback_idx && callback_idx < stage_idx);
}

#[test]
fn detached_and_notes_hosts_advertise_history_only_and_cancel_refusals() {
    let detached = read("src/ai/agent_chat/ui/chat_window.rs");
    assert!(
        detached
            .contains("view.set_allowed_portal_kinds(vec![ContextPortalKind::AgentChatHistory])")
            || detached
                .contains("view.set_allowed_portal_kinds(vec![PortalKind::AgentChatHistory])"),
        "detached host must advertise only AgentChatHistory"
    );
    assert!(
        detached.contains("cancel_portal_session_in_detached_chat_window(kind, cx)"),
        "detached host must cancel staged sessions when history open fails"
    );

    let notes = read("src/notes/window/agent_chat_host.rs");
    assert!(
        notes.contains("ContextPortalKind::AgentChatHistory")
            || notes.contains("PortalKind::AgentChatHistory")
    );
    assert!(notes.contains("cancel_pending_portal_session(kind, cx)"));
}
