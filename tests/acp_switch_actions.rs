//! Source-level contract tests for ACP agent switching from the chat actions menu.

const ACTIONS_TOGGLE_SOURCE: &str = include_str!("../src/app_impl/actions_toggle.rs");
const ACTION_HANDLER_SOURCE: &str = include_str!("../src/app_actions/handle_action/mod.rs");
const ACTION_BUILDER_SOURCE: &str = include_str!("../src/actions/builders/script_context.rs");

#[test]
fn acp_actions_popup_uses_dynamic_agent_actions() {
    assert!(
        ACTIONS_TOGGLE_SOURCE.contains("get_acp_chat_actions_with_agents"),
        "ACP actions popup must build agent-aware actions from the catalog"
    );
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
        ACTION_HANDLER_SOURCE.contains("acp_switch_agent_reopen_requested"),
        "switch-agent action must emit acp_switch_agent_reopen_requested tracing event"
    );
    // The retry payload staging must happen before the close+reopen sequence.
    let stage_pos = ACTION_HANDLER_SOURCE
        .find("stage_agent_switch_retry")
        .expect("stage_agent_switch_retry must exist");
    let close_pos = ACTION_HANDLER_SOURCE
        .find("close_tab_ai_harness_terminal")
        .expect("close_tab_ai_harness_terminal must exist");
    assert!(
        stage_pos < close_pos,
        "retry payload must be staged before closing the harness terminal"
    );
}

#[test]
fn acp_action_builder_exposes_agent_section_entries() {
    assert!(
        ACTION_BUILDER_SOURCE.contains("Current Agent:"),
        "ACP action builder must label the current agent in the actions menu"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains(".with_section(\"Agent\")"),
        "ACP action builder must place switch actions in an Agent section"
    );
}
