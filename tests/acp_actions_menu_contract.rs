//! Source-contract tests keeping ACP Actions Menu labels and leaf ID prefixes stable.

#[test]
fn acp_actions_menu_keeps_stable_root_labels() {
    let source = include_str!("../src/actions/builders/script_context.rs");
    assert!(
        source.contains("\"Change Agent\""),
        "ACP Actions Menu root must say Change Agent"
    );
    assert!(
        source.contains("\"Change Model\""),
        "ACP Actions Menu root must say Change Model"
    );
}

#[test]
fn acp_actions_menu_keeps_stable_leaf_prefixes_in_builder_and_handler() {
    let builder = include_str!("../src/actions/builders/script_context.rs");
    let handler = include_str!("../src/app_actions/handle_action/mod.rs");

    for needle in ["acp_switch_agent:", "acp_switch_model:"] {
        assert!(
            builder.contains(needle),
            "ACP builder must emit stable leaf prefix: {needle}"
        );
    }

    // Handler dispatches via function imports rather than raw prefix strings.
    // Verify both dispatch functions are called.
    assert!(
        handler.contains("acp_switch_agent_id_from_action"),
        "ACP handler must dispatch acp_switch_agent: leaves via acp_switch_agent_id_from_action"
    );
    assert!(
        handler.contains("acp_switch_model_id_from_action"),
        "ACP handler must dispatch acp_switch_model: leaves via acp_switch_model_id_from_action"
    );
}

#[test]
fn acp_actions_menu_has_host_aware_filtering() {
    let source = include_str!("../src/actions/builders/script_context.rs");
    assert!(
        source.contains("AcpActionsDialogHost"),
        "ACP builder must define AcpActionsDialogHost enum for host-aware filtering"
    );
    assert!(
        source.contains("filter_acp_actions_for_host"),
        "ACP builder must use filter_acp_actions_for_host for host-aware filtering"
    );
}

#[test]
fn acp_actions_menu_emits_built_log() {
    let source = include_str!("../src/actions/builders/script_context.rs");
    assert!(
        source.contains("acp_actions_menu_built"),
        "ACP route builder must emit acp_actions_menu_built structured log"
    );
}

#[test]
fn acp_actions_menu_emits_selected_log_for_both_hosts() {
    let shared_handler = include_str!("../src/app_actions/handle_action/mod.rs");
    let detached_handler = include_str!("../src/ai/acp/chat_window.rs");

    assert!(
        shared_handler.contains("acp_actions_menu_selected"),
        "Shared host handler must emit acp_actions_menu_selected structured log"
    );
    assert!(
        detached_handler.contains("acp_actions_menu_selected"),
        "Detached host handler must emit acp_actions_menu_selected structured log"
    );
}

#[test]
fn acp_actions_menu_emits_filtered_log() {
    let source = include_str!("../src/actions/builders/script_context.rs");
    assert!(
        source.contains("acp_actions_menu_filtered"),
        "ACP filter must emit acp_actions_menu_filtered structured log for removed items"
    );
}

#[test]
fn acp_save_as_note_has_updated_description() {
    let source = include_str!("../src/actions/builders/script_context.rs");
    assert!(
        source.contains("Create or update a note from the current ACP content"),
        "ACP Save as Note action must use the updated description"
    );
}
