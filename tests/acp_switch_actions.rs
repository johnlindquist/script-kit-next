//! Source-level contract tests for ACP agent switching from the chat actions menu.

const ACTIONS_TOGGLE_SOURCE: &str = include_str!("../src/app_impl/actions_toggle.rs");
const ACTION_HANDLER_SOURCE: &str = include_str!("../src/app_actions/handle_action/mod.rs");
const ACTION_BUILDER_SOURCE: &str = include_str!("../src/actions/builders/script_context.rs");
const DIALOG_SOURCE: &str = include_str!("../src/actions/dialog.rs");
const CHAT_WINDOW_SOURCE: &str = include_str!("../src/ai/acp/chat_window.rs");
const ACTIONS_DIALOG_SOURCE: &str = include_str!("../src/app_impl/actions_dialog.rs");
const ACTIONS_WINDOW_SOURCE: &str = include_str!("../src/actions/window.rs");

#[test]
fn acp_actions_popup_uses_dynamic_agent_actions() {
    assert!(
        ACTIONS_TOGGLE_SOURCE.contains("acp_actions_context_built"),
        "ACP actions popup must log when it builds ACP actions context from the active session"
    );
    assert!(
        ACTIONS_TOGGLE_SOURCE.contains("thread.available_agents().to_vec()"),
        "ACP actions popup must source available agents from the live ACP thread"
    );
    assert!(
        ACTIONS_TOGGLE_SOURCE.contains("thread.available_models().to_vec()"),
        "ACP actions popup must source available models from the live ACP thread"
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
        ACTION_HANDLER_SOURCE.contains("self.open_tab_ai_acp_with_entry_intent(None, cx);"),
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

// ── Notes-hosted ACP agent switching contract tests ─────────────────────────

const NOTES_ACP_HOST_SOURCE: &str = include_str!("../src/notes/window/acp_host.rs");

#[test]
fn notes_acp_dispatch_handles_switch_agent_actions() {
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("acp_switch_agent_id_from_action"),
        "Notes-hosted ACP must detect switch-agent action IDs"
    );
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("persist_preferred_acp_agent_id_sync"),
        "Notes-hosted ACP switch-agent flow must persist the selected agent synchronously"
    );
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("notes_acp_switch_agent_relaunched"),
        "Notes-hosted ACP switch-agent flow must emit relaunch tracing"
    );
}

#[test]
fn notes_acp_switch_agent_preserves_draft_input() {
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("current_notes_acp_draft_input"),
        "Notes-hosted ACP switch-agent flow must extract draft input before relaunch"
    );
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("has_draft_input"),
        "Notes-hosted ACP switch-agent tracing must include draft input status"
    );
}

#[test]
fn notes_acp_switch_agent_tears_down_before_relaunch() {
    let hide_pos = NOTES_ACP_HOST_SOURCE
        .find("notes_acp_switch_agent_requested")
        .and_then(|start| {
            NOTES_ACP_HOST_SOURCE[start..]
                .find("prepare_for_host_hide")
                .map(|offset| start + offset)
        })
        .expect("prepare_for_host_hide must appear after switch-agent-requested");
    let drop_pos = NOTES_ACP_HOST_SOURCE[hide_pos..]
        .find("embedded_acp_chat = None")
        .map(|offset| hide_pos + offset)
        .expect("embedded_acp_chat = None must appear after prepare_for_host_hide");
    let relaunch_pos = NOTES_ACP_HOST_SOURCE[drop_pos..]
        .find("open_or_focus_embedded_acp")
        .map(|offset| drop_pos + offset)
        .expect("open_or_focus_embedded_acp must appear after dropping cached view");
    assert!(
        hide_pos < drop_pos && drop_pos < relaunch_pos,
        "Agent switch must: hide popups -> drop cached view -> relaunch"
    );
}

// ── Host-aware ACP unification contract tests ───────────────────────────────

#[test]
fn acp_builder_exposes_host_aware_route_api() {
    assert!(
        ACTION_BUILDER_SOURCE.contains("enum AcpActionsDialogHost"),
        "ACP builder must define AcpActionsDialogHost enum"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("AcpActionsDialogHost::Shared"),
        "AcpActionsDialogHost must have a Shared variant"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("AcpActionsDialogHost::Detached"),
        "AcpActionsDialogHost must have a Detached variant"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("get_acp_chat_root_route_for_host"),
        "Host-aware ACP root route builder must exist"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("get_acp_agent_picker_route_for_host"),
        "Host-aware ACP agent picker route builder must exist"
    );
}

#[test]
fn actions_window_routes_focused_popup_shortcuts_through_shared_matcher() {
    assert!(
        ACTIONS_WINDOW_SOURCE.contains("matching_action_id_for_keystroke"),
        "ActionsWindow must reuse the shared dialog shortcut matcher for focused popup fallback"
    );
    assert!(
        ACTIONS_DIALOG_SOURCE.contains("activate_action_id"),
        "Shared actions dialog routing must expose activation by explicit action id"
    );
}

#[test]
fn actions_window_defers_activation_to_host_callback_when_present() {
    assert!(
        ACTIONS_WINDOW_SOURCE.contains("on_activation_callback"),
        "ActionsWindow must read the dialog's activation callback"
    );
    assert!(
        ACTIONS_WINDOW_SOURCE.contains("callback(activation, window, cx);"),
        "ActionsWindow must defer focused-popup activation back to the host callback"
    );
}

#[test]
fn detached_acp_uses_host_aware_route_builder() {
    // Detached ACP must NOT use the old flat DETACHED_SUPPORTED_ACTIONS constant
    assert!(
        !CHAT_WINDOW_SOURCE.contains("const DETACHED_SUPPORTED_ACTIONS"),
        "Detached ACP must not define a local DETACHED_SUPPORTED_ACTIONS allowlist"
    );
    // Detached ACP must use the host-aware dialog constructor
    assert!(
        CHAT_WINDOW_SOURCE.contains("with_acp_chat_for_host"),
        "Detached ACP must use with_acp_chat_for_host"
    );
    assert!(
        CHAT_WINDOW_SOURCE.contains("AcpActionsDialogHost::Detached")
            || CHAT_WINDOW_SOURCE.contains("builders::AcpActionsDialogHost::Detached"),
        "Detached ACP must specify Detached host"
    );
}

#[test]
fn dialog_exposes_host_aware_constructor() {
    assert!(
        DIALOG_SOURCE.contains("with_acp_chat_for_host"),
        "ActionsDialog must expose with_acp_chat_for_host"
    );
}

#[test]
fn detached_host_excludes_unsupported_actions() {
    // The detached host filter must reject panel-only actions
    assert!(
        ACTION_BUILDER_SOURCE.contains("acp_action_supported_in_host"),
        "ACP builder must have a host action filter function"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("filter_acp_actions_for_host"),
        "ACP builder must have a host action filter"
    );
}

#[test]
fn route_visibility_logs_include_depth_and_escape_hint() {
    // Both shared and detached log sites must include route_depth and escape_hint
    assert!(
        ACTIONS_DIALOG_SOURCE.contains("route_depth"),
        "Shared actions dialog route logs must include route_depth"
    );
    assert!(
        ACTIONS_DIALOG_SOURCE.contains("escape_hint"),
        "Shared actions dialog route logs must include escape_hint"
    );
    assert!(
        ACTIONS_WINDOW_SOURCE.contains("route_depth"),
        "Detached actions window route logs must include route_depth"
    );
    assert!(
        ACTIONS_WINDOW_SOURCE.contains("escape_hint"),
        "Detached actions window route logs must include escape_hint"
    );
}
