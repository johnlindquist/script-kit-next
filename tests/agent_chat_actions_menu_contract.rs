//! Source-contract tests keeping Agent Chat Actions Menu labels and leaf ID prefixes stable.

#[test]
fn agent_chat_actions_menu_keeps_stable_root_labels() {
    let source = include_str!("../src/actions/builders/script_context.rs");
    assert!(
        source.contains("\"Profile picker\""),
        "Agent Chat Actions Menu root must say Profile picker"
    );
    assert!(
        source.contains("\"Change Model\""),
        "Agent Chat Actions Menu root must say Change Model"
    );
}

#[test]
fn agent_chat_profile_picker_warns_profile_switch_starts_new_chat() {
    let source = include_str!("../src/actions/builders/script_context.rs");
    assert!(
        source.contains("Starts a new chat when a conversation is already active"),
        "Profile switch rows must warn that switching profiles starts a new chat"
    );
}

#[test]
fn agent_chat_actions_menu_keeps_stable_leaf_prefixes_in_builder_and_handler() {
    let builder = include_str!("../src/actions/builders/script_context.rs");
    let handler = include_str!("../src/app_actions/handle_action/mod.rs");

    for needle in ["agent_chat_switch_profile:", "agent_chat_switch_model:"] {
        assert!(
            builder.contains(needle),
            "Agent Chat builder must emit stable leaf prefix: {needle}"
        );
    }

    // Handler dispatches via function imports rather than raw prefix strings.
    // Verify both dispatch functions are called.
    assert!(
        handler.contains("agent_chat_switch_profile_id_from_action"),
        "Agent Chat handler must dispatch Agent Chat profile leaves via agent_chat_switch_profile_id_from_action"
    );
    assert!(
        handler.contains("agent_chat_switch_model_id_from_action"),
        "Agent Chat handler must dispatch agent_chat_switch_model: leaves via agent_chat_switch_model_id_from_action"
    );
}

#[test]
fn agent_chat_actions_menu_has_host_aware_filtering() {
    let source = include_str!("../src/actions/builders/script_context.rs");
    assert!(
        source.contains("AgentChatActionsDialogHost"),
        "Agent Chat builder must define AgentChatActionsDialogHost enum for host-aware filtering"
    );
    assert!(
        source.contains("filter_agent_chat_actions_for_host"),
        "Agent Chat builder must use filter_agent_chat_actions_for_host for host-aware filtering"
    );
    assert!(
        source.contains("enum AgentChatHostActionPlan")
            && source.contains("IncludeWithShortcut")
            && source.contains("IncludeWithoutShortcut")
            && source.contains("Exclude")
            && source.contains("fn agent_chat_host_action_plan"),
        "Agent Chat host filtering should use a named host-action state plan instead of split conditionals"
    );
}

#[test]
fn agent_chat_actions_menu_emits_built_log() {
    let source = include_str!("../src/actions/builders/script_context.rs");
    assert!(
        source.contains("agent_chat_actions_menu_built"),
        "Agent Chat route builder must emit agent_chat_actions_menu_built structured log"
    );
}

#[test]
fn agent_chat_actions_menu_emits_selected_log_for_both_hosts() {
    let shared_handler = include_str!("../src/app_actions/handle_action/mod.rs");
    let detached_handler = include_str!("../src/ai/agent_chat/ui/chat_window.rs");

    assert!(
        shared_handler.contains("agent_chat_actions_menu_selected"),
        "Shared host handler must emit agent_chat_actions_menu_selected structured log"
    );
    assert!(
        detached_handler.contains("agent_chat_actions_menu_selected"),
        "Detached host handler must emit agent_chat_actions_menu_selected structured log"
    );
}

#[test]
fn agent_chat_actions_menu_emits_filtered_log() {
    let source = include_str!("../src/actions/builders/script_context.rs");
    assert!(
        source.contains("agent_chat_actions_menu_filtered"),
        "Agent Chat filter must emit agent_chat_actions_menu_filtered structured log for removed items"
    );
}

#[test]
fn agent_chat_save_as_note_has_updated_description() {
    let source = include_str!("../src/actions/builders/script_context.rs");
    assert!(
        source.contains("Create or update a note from the current Agent Chat content"),
        "Agent Chat Save as Note action must use the updated description"
    );
}
