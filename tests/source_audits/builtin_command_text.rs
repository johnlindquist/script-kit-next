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
