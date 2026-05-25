const ACTION_BUILDER_SOURCE: &str = include_str!("../src/actions/builders/script_context.rs");
const ACTION_DIALOG_SOURCE: &str = include_str!("../src/actions/dialog.rs");
const ACTION_HANDLER_SOURCE: &str = include_str!("../src/app_actions/handle_action/mod.rs");
const PROFILES_SOURCE: &str = include_str!("../src/ai/agent_chat/profiles.rs");
const ACP_LAUNCH_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/acp_launch.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const ACP_THREAD_SOURCE: &str = include_str!("../src/ai/acp/thread.rs");
const ACP_MOD_SOURCE: &str = include_str!("../src/ai/acp/mod.rs");
const PROFILE_POPUP_SOURCE: &str = include_str!("../src/ai/acp/profile_selector_popup.rs");

#[test]
fn profile_selector_is_separate_from_model_selector() {
    assert!(ACTION_BUILDER_SOURCE.contains("AGENT_CHAT_CHANGE_PROFILE_ACTION_ID"));
    assert!(ACTION_BUILDER_SOURCE.contains("AGENT_CHAT_PROFILE_PICKER_ROUTE_ID"));
    assert!(ACTION_BUILDER_SOURCE.contains("agent_chat_switch_profile:"));
    assert!(ACTION_DIALOG_SOURCE.contains("get_agent_chat_profile_picker_route_for_host"));
    assert!(ACTION_DIALOG_SOURCE.contains("AGENT_CHAT_CHANGE_PROFILE_ACTION_ID"));
    assert!(ACTION_BUILDER_SOURCE.contains("ACP_CHANGE_MODEL_ACTION_ID"));
    assert!(ACTION_BUILDER_SOURCE.contains("acp_switch_model:"));
    assert!(ACP_VIEW_SOURCE.contains("profile_selector_open"));
    assert!(ACP_VIEW_SOURCE.contains("sync_profile_selector_popup_window_from_cached_parent"));
    assert!(ACP_VIEW_SOURCE.contains("model_selector_open"));
    assert!(ACP_VIEW_SOURCE.contains("sync_model_selector_popup_window_from_cached_parent"));
}

#[test]
fn profile_selection_persistence_preserves_acp_fallback_and_stable_ids() {
    assert!(PROFILES_SOURCE.contains("persist_agent_chat_profile_selection"));
    assert!(PROFILES_SOURCE.contains("BUILTIN_ACP_FALLBACK_PROFILE_ID"));
    assert!(PROFILES_SOURCE.contains("ai.selected_profile_id = None;"));
    assert!(PROFILES_SOURCE.contains("ai.selected_backend = Some(AgentChatBackend::Acp);"));
    assert!(PROFILES_SOURCE.contains("ai.selected_profile_id = Some(entry.id.clone());"));
}

#[test]
fn profile_selection_relaunches_fresh_in_shared_agent_chat_host() {
    assert!(ACTION_HANDLER_SOURCE.contains("agent_chat_switch_profile_id_from_action"));
    assert!(ACTION_HANDLER_SOURCE.contains("persist_agent_chat_profile_selection"));
    assert!(ACTION_HANDLER_SOURCE.contains("self.close_tab_ai_harness_terminal(cx);"));
    assert!(ACTION_HANDLER_SOURCE.contains("self.embedded_acp_chat = None;"));
    assert!(ACTION_HANDLER_SOURCE.contains("self.open_tab_ai_acp_with_entry_intent(None, cx);"));
    assert!(ACP_VIEW_SOURCE.contains("set_on_profile_selected"));
    assert!(TAB_AI_MODE_SOURCE.contains("select_agent_chat_profile_and_relaunch"));
}

#[test]
fn acp_launch_uses_effective_profile_for_acp_agent_and_model() {
    assert!(ACP_LAUNCH_SOURCE.contains("resolve_effective_profile"));
    assert!(ACP_LAUNCH_SOURCE.contains("PiAgentChatLaunch::from_profile"));
    assert!(ACP_LAUNCH_SOURCE.contains("effective_profile.agent.clone()"));
    assert!(ACP_LAUNCH_SOURCE.contains("effective_profile.model.clone()"));
    assert!(ACP_LAUNCH_SOURCE.contains("load_preferred_acp_agent_id"));
}

#[test]
fn provider_scoped_pi_model_selection_is_split_before_launch() {
    assert!(PROFILES_SOURCE.contains("parse_provider_model_selection"));
    assert!(PROFILES_SOURCE.contains("profile.provider = Some(provider);"));
    assert!(PROFILES_SOURCE.contains("profile.model = Some(model);"));
}

#[test]
fn profile_selector_popup_has_independent_module_and_automation_id() {
    assert!(ACP_MOD_SOURCE.contains("profile_selector_popup"));
    assert!(PROFILE_POPUP_SOURCE.contains("agent-chat-profile-selector-popup"));
    assert!(PROFILE_POPUP_SOURCE.contains("batch_select_profile_by_value"));
    assert!(PROFILE_POPUP_SOURCE.contains("batch_select_profile_by_semantic_id"));
}

#[test]
fn toolbar_profile_selector_carries_parent_window_for_detached_popup() {
    assert!(ACP_VIEW_SOURCE.contains("AcpToolbarEvent::ToggleProfileSelector(parent)"));
    assert!(ACP_VIEW_SOURCE.contains("this.mention_popup_parent_window = Some(*parent);"));
    assert!(ACP_VIEW_SOURCE.contains("this.selected_profile_popup_index(&entries)"));
}

#[test]
fn profile_display_flows_through_thread_and_footer() {
    assert!(ACP_THREAD_SOURCE.contains("profile_display_name"));
    assert!(ACP_THREAD_SOURCE.contains("pub(crate) fn profile_display"));
    assert!(ACP_VIEW_SOURCE.contains("profile_display: thread.profile_display().to_string()"));
    assert!(ACP_VIEW_SOURCE.contains("agent-chat-profile-display"));
}
