use script_kit_gpui::{builtins::get_builtin_entries, config::BuiltInConfig};

#[test]
fn agent_chat_destination_builtins_name_agent_chat_not_generic_ai() {
    let entries = get_builtin_entries(&BuiltInConfig::default());

    for (id, expected_name, expected_action) in [
        (
            "builtin/send-screen-to-ai",
            "Send Screen to Agent Chat",
            "Send Screen to Agent Chat",
        ),
        (
            "builtin/send-focused-window-to-ai",
            "Send Focused Window to Agent Chat",
            "Send Window to Agent Chat",
        ),
        (
            "builtin/send-selected-text-to-ai",
            "Send Selected Text to Agent Chat",
            "Send Selection to Agent Chat",
        ),
        (
            "builtin/send-browser-tab-to-ai",
            "Send Focused Browser Tab to Agent Chat",
            "Send Tab to Agent Chat",
        ),
        (
            "builtin/dictation-to-ai",
            "Dictate to Agent Chat",
            "Start Dictation to Agent Chat",
        ),
    ] {
        let entry = entries
            .iter()
            .find(|entry| entry.id == id)
            .unwrap_or_else(|| panic!("missing builtin entry {id}"));

        assert_eq!(entry.name, expected_name, "{id} name");
        assert_eq!(entry.default_action_text(), expected_action, "{id} action");
        assert!(
            !entry.name.contains(" to AI") && !entry.default_action_text().contains(" to AI"),
            "{id} should name the concrete Agent Chat destination"
        );
    }

    let dictation = entries
        .iter()
        .find(|entry| entry.id == "builtin/dictation-to-ai")
        .expect("dictation-to-ai entry should exist");
    assert_eq!(dictation.footer_action_text(), "Dictate Chat");
}

#[test]
fn acp_history_text_names_agent_chat_conversations() {
    let entries = get_builtin_entries(&BuiltInConfig::default());
    let acp_history = entries
        .iter()
        .find(|entry| entry.id == "builtin/acp-history")
        .expect("acp-history entry should exist");
    let root_actions = super::read_source("src/app_impl/root_unified_result_actions.rs");

    assert_eq!(acp_history.name, "Conversation History");
    assert_eq!(
        acp_history.description,
        "Browse and manage past Agent Chat conversations"
    );
    assert!(
        root_actions.contains("Self::AcpHistory(_) => \"Agent Chat Conversations\""),
        "root-unified ACP history action context should name Agent Chat conversations"
    );
    assert!(
        !acp_history.description.contains("AI conversations")
            && !root_actions.contains("Self::AcpHistory(_) => \"AI Conversations\""),
        "ACP history text should not use generic AI conversation wording"
    );
}
