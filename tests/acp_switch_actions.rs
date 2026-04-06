//! Source-level contract tests for ACP agent switching from the chat actions menu.

const ACTIONS_TOGGLE_SOURCE: &str = include_str!("../src/app_impl/actions_toggle.rs");
const ACTION_HANDLER_SOURCE: &str = include_str!("../src/app_actions/handle_action/mod.rs");
const ACTION_BUILDER_SOURCE: &str = include_str!("../src/actions/builders/script_context.rs");
const DIALOG_SOURCE: &str = include_str!("../src/actions/dialog.rs");

#[test]
fn acp_actions_popup_uses_dynamic_agent_actions() {
    assert!(
        ACTIONS_TOGGLE_SOURCE.contains("acp_actions_agent_context_built"),
        "ACP actions popup must log when it builds ACP agent context from the active session"
    );
    assert!(
        ACTIONS_TOGGLE_SOURCE.contains("thread.available_agents().to_vec()"),
        "ACP actions popup must source available agents from the live ACP thread"
    );
}

#[test]
fn acp_action_handler_switches_agents_by_persisting_and_reopening() {
    assert!(
        ACTION_HANDLER_SOURCE.contains("acp_switch_agent_id_from_action"),
        "ACP action handler must detect switch-agent action IDs"
    );
    assert!(
        ACTION_HANDLER_SOURCE.contains("persist_preferred_acp_agent_id"),
        "switch-agent action must persist the selected ACP agent"
    );
    assert!(
        ACTION_HANDLER_SOURCE.contains("self.open_tab_ai_chat(cx);"),
        "switch-agent action must reopen ACP chat after changing the agent"
    );
}

#[test]
fn acp_action_handler_stages_retry_payload_before_reopen() {
    assert!(
        ACTION_HANDLER_SOURCE.contains("stage_agent_switch_retry"),
        "switch-agent action must stage a retry payload preserving capability requirements"
    );
    assert!(
        ACTION_HANDLER_SOURCE.contains("acp_switch_agent_relaunch_requested"),
        "switch-agent action must emit acp_switch_agent_relaunch_requested tracing event"
    );
    // The retry payload staging must happen before the close+reopen sequence.
    let stage_pos = ACTION_HANDLER_SOURCE
        .find("stage_agent_switch_retry")
        .expect("stage_agent_switch_retry must exist");
    let close_pos = ACTION_HANDLER_SOURCE[stage_pos..]
        .find("close_tab_ai_harness_terminal")
        .map(|offset| stage_pos + offset)
        .expect("close_tab_ai_harness_terminal must exist");
    assert!(
        stage_pos < close_pos,
        "retry payload must be staged before closing the harness terminal"
    );
}

#[test]
fn acp_action_builder_exposes_agent_section_entries() {
    assert!(
        ACTION_BUILDER_SOURCE.contains(".with_section(\"Agent\")"),
        "ACP action builder must place switch actions in an Agent section"
    );
}

// ── Route / back-stack contract tests ────────────────────────────────────────

#[test]
fn acp_root_route_uses_change_agent_entry() {
    assert!(
        ACTION_BUILDER_SOURCE.contains("ACP_CHANGE_AGENT_ACTION_ID"),
        "ACP actions must define ACP_CHANGE_AGENT_ACTION_ID"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("get_acp_chat_root_route"),
        "ACP root route builder must exist"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("get_acp_agent_picker_route"),
        "ACP agent picker route builder must exist"
    );
}

#[test]
fn acp_actions_dialog_registers_change_agent_drill_down() {
    assert!(
        DIALOG_SOURCE.contains("with_acp_chat"),
        "ActionsDialog must expose with_acp_chat"
    );
    assert!(
        DIALOG_SOURCE.contains("ACP_CHANGE_AGENT_ACTION_ID"),
        "with_acp_chat must register ACP_CHANGE_AGENT_ACTION_ID"
    );
    assert!(
        DIALOG_SOURCE.contains("register_drill_down_route"),
        "with_acp_chat must register the ACP drill-down route"
    );
}

#[test]
fn acp_picker_preserves_existing_switch_action_ids() {
    assert!(
        ACTION_BUILDER_SOURCE.contains("acp_switch_agent_action_id(entry.id.as_ref())"),
        "Second-level ACP picker must preserve acp_switch_agent:* IDs"
    );
}

#[test]
fn toggle_actions_uses_route_based_acp_dialog() {
    assert!(
        ACTIONS_TOGGLE_SOURCE.contains("ActionsDialog::with_acp_chat"),
        "ACP actions open path must build a route-based dialog"
    );
}

#[test]
fn dialog_exposes_route_stack_public_api() {
    // Verify the core route/back-stack types and methods exist
    assert!(
        DIALOG_SOURCE.contains("pub struct ActionsDialogRoute"),
        "ActionsDialogRoute must be a public struct"
    );
    assert!(
        DIALOG_SOURCE.contains("pub enum ActionsDialogActivation"),
        "ActionsDialogActivation must be a public enum"
    );
    assert!(
        DIALOG_SOURCE.contains("pub enum ActionsDialogEscapeOutcome"),
        "ActionsDialogEscapeOutcome must be a public enum"
    );
    assert!(
        DIALOG_SOURCE.contains("pub fn activate_selected"),
        "activate_selected must be a public method"
    );
    assert!(
        DIALOG_SOURCE.contains("pub fn handle_escape"),
        "handle_escape must be a public method"
    );
    assert!(
        DIALOG_SOURCE.contains("pub fn route_hint_label"),
        "route_hint_label must be a public method"
    );
}

#[test]
fn dialog_has_structured_route_tracing() {
    assert!(
        DIALOG_SOURCE.contains("actions_dialog_activation"),
        "activate_selected must emit actions_dialog_activation tracing"
    );
    assert!(
        DIALOG_SOURCE.contains("actions_dialog_route_push"),
        "push_route must emit actions_dialog_route_push tracing"
    );
    assert!(
        DIALOG_SOURCE.contains("actions_dialog_route_pop"),
        "pop_route must emit actions_dialog_route_pop tracing"
    );
    assert!(
        DIALOG_SOURCE.contains("actions_dialog_escape"),
        "handle_escape must emit actions_dialog_escape tracing"
    );
}
