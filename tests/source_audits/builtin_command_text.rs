use script_kit_gpui::{builtins::get_builtin_entries, config::BuiltInConfig};

#[test]
fn agent_chat_destination_builtins_name_agent_chat_not_generic_ai() {
    let entries = get_builtin_entries(&BuiltInConfig::default());
    let builtins_source = super::read_source("src/builtins/mod.rs");
    let execution_source = super::read_source("src/app_execute/builtin_execution.rs");
    let agent_chat = entries
        .iter()
        .find(|entry| entry.id == "builtin/ai-chat")
        .expect("agent chat entry should exist");
    assert_eq!(agent_chat.name, "Agent Chat");
    assert_eq!(agent_chat.default_action_text(), "Open Agent Chat");
    assert_eq!(agent_chat.footer_action_text(), "Agent Chat");
    assert!(
        builtins_source
            .contains("AiCommandType::SendScreenAreaToAi => \"Select Area for Agent Chat\"")
            && !builtins_source.contains("Select Area for AI"),
        "screen-area Agent Chat label should not regress to generic AI wording"
    );
    assert!(
        execution_source.contains("Send Screen Area to Agent Chat is unavailable")
            && execution_source.contains("Opening Dictation to Agent Chat")
            && !execution_source.contains("Send Screen Area to AI is unavailable")
            && !execution_source.contains("Dictation-to-AI"),
        "Agent Chat execution messages should not regress to generic AI wording"
    );

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
    let group_header = super::read_source("src/app_render/group_header_item.rs");
    let script_context_actions = super::read_source("src/actions/builders/script_context.rs");
    let source_heads = super::read_source("src/menu_syntax/source_heads.rs");
    let payload = super::read_source("src/menu_syntax/payload.rs");
    let action_helpers = super::read_source("src/action_helpers.rs");
    let focused_info = super::read_source("src/app_render/focused_info.rs");

    assert_eq!(acp_history.name, "Agent Chat History");
    assert_eq!(acp_history.default_action_text(), "Open Agent Chat History");
    assert_eq!(
        acp_history.description,
        "Browse and manage past Agent Chat conversations"
    );
    assert!(
        root_actions.contains("Self::AcpHistory(_) => \"Agent Chat Conversations\""),
        "root-unified ACP history action context should name Agent Chat conversations"
    );
    assert!(
        group_header.contains("BuiltInFeature::AcpHistory => \"Agent Chat History\""),
        "ACP history group header should match the visible command name"
    );
    assert!(
        script_context_actions.contains("\"acp_show_history\"")
            && script_context_actions.contains("\"Agent Chat History\"")
            && script_context_actions.contains("Browse and manage past Agent Chat conversations"),
        "Agent Chat actions should not expose generic conversation-history text"
    );
    assert!(
        source_heads.contains("label: \"Agent Chat Conversations\"")
            && payload.contains("Self::Conversations => \"Agent Chat Conversations\"")
            && action_helpers.contains("Cannot edit Agent Chat conversations"),
        "source filters and disabled action text should name Agent Chat conversations"
    );
    assert!(
        focused_info.contains("focused_info_type_indicator(\"Agent Chat Conversation\""),
        "ACP history info panel type indicator should name Agent Chat"
    );
    assert!(
        !acp_history.description.contains("AI conversations")
            && !root_actions.contains("Self::AcpHistory(_) => \"AI Conversations\""),
        "ACP history text should not use generic AI conversation wording"
    );
}

#[test]
fn permission_assistant_commands_do_not_claim_to_grant_permissions() {
    let entries = get_builtin_entries(&BuiltInConfig::default());
    let settings = super::read_source("src/render_builtins/settings.rs");

    for (id, expected_name, expected_action, expected_description) in [
        (
            "builtin/allow-accessibility",
            "Accessibility Permission Assistant",
            "Open Accessibility Assistant",
            "Open the Permission Assistant for Accessibility",
        ),
        (
            "builtin/allow-screen-recording",
            "Screen Recording Permission Assistant",
            "Open Screen Recording Assistant",
            "Open the Permission Assistant for Screen Recording",
        ),
    ] {
        let entry = entries
            .iter()
            .find(|entry| entry.id == id)
            .unwrap_or_else(|| panic!("missing permission assistant builtin {id}"));

        assert_eq!(entry.name, expected_name, "{id} name");
        assert_eq!(entry.default_action_text(), expected_action, "{id} action");
        assert_eq!(entry.description, expected_description, "{id} description");
        assert!(
            !entry.name.starts_with("Allow ") && !entry.default_action_text().starts_with("Allow "),
            "{id} should not imply Script Kit can directly grant macOS permission"
        );
        assert!(
            settings.contains(expected_name) && settings.contains(expected_description),
            "settings hub should use the same assistant wording for {id}"
        );
    }

    let accessibility_settings = entries
        .iter()
        .find(|entry| entry.id == "builtin/accessibility-settings")
        .expect("accessibility settings builtin should exist");
    assert_eq!(
        accessibility_settings.description,
        "Open Accessibility settings in macOS System Settings"
    );
    assert!(
        !accessibility_settings
            .description
            .contains("System Preferences")
            && settings.contains("Open Accessibility settings in macOS System Settings"),
        "Accessibility Settings text should use modern macOS System Settings wording"
    );
}

#[test]
fn force_quit_command_text_names_the_dialog_it_opens() {
    let entries = get_builtin_entries(&BuiltInConfig::default());
    let system_actions = super::read_source("src/system_actions/mod.rs");

    let force_quit = entries
        .iter()
        .find(|entry| entry.id == "builtin/force-quit")
        .expect("force quit builtin should exist");

    assert_eq!(force_quit.name, "Open Force Quit Apps");
    assert_eq!(force_quit.default_action_text(), "Open Force Quit");
    assert_eq!(
        force_quit.description,
        "Open the macOS Force Quit Applications dialog"
    );
    assert!(
        system_actions.contains("Opening Force Quit Applications dialog")
            && system_actions.contains("command down, option down"),
        "Force Quit Apps builtin should open the macOS dialog instead of directly terminating apps"
    );
}
