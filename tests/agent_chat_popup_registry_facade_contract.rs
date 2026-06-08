use std::{fs, path::Path};

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[test]
fn agent_chat_popup_registration_facade_registers_and_removes() {
    let source = read("src/ai/agent_chat/ui/popup_registry.rs");
    assert!(source.contains("pub(crate) struct AgentChatPopupRegistration"));
    assert!(source.contains("pub(crate) fn register"));
    assert!(source.contains("upsert_runtime_window_handle"));
    assert!(source.contains("remove_runtime_window_handle"));
    assert!(source.contains("remove_automation_window"));
    assert!(source.contains("impl Drop for AgentChatPopupRegistration"));
}

#[test]
fn all_agent_chat_prompt_popups_hold_scoped_registration() {
    for (path, id) in [
        (
            "src/ai/agent_chat/ui/picker_popup.rs",
            "agent_chat-mention-popup",
        ),
        (
            "src/ai/agent_chat/ui/history_popup.rs",
            "agent_chat-history-popup",
        ),
    ] {
        let source = read(path);
        assert!(source.contains(id), "{path} missing {id}");
        assert!(
            source.contains("AgentChatPopupRegistration::register"),
            "{path} must register through Agent Chat popup facade"
        );
        assert!(
            source.contains("_registration: super::popup_registry::AgentChatPopupRegistration"),
            "{path} must keep the Drop guard in the popup slot"
        );
    }
}
