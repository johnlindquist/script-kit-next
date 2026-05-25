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
const PROMPT_HANDLER_SOURCE: &str = include_str!("../src/prompt_handler/mod.rs");
const CONTEXT_PICKER_TYPES_SOURCE: &str = include_str!("../src/ai/window/context_picker/types.rs");
const CONTEXT_PICKER_SOURCE: &str = include_str!("../src/ai/window/context_picker/mod.rs");
const AUTOMATION_COLLECTOR_SOURCE: &str =
    include_str!("../src/windows/automation_surface_collector.rs");
const DETACHED_TRANSACTION_PROVIDER_SOURCE: &str =
    include_str!("../src/windows/automation_transaction_provider.rs");
const FOOTER_CHROME_SOURCE: &str = include_str!("../src/components/footer_chrome.rs");
const FOOTER_POPUP_SOURCE: &str = include_str!("../src/footer_popup.rs");

fn fn_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("signature must exist");
    let rest = &source[start..];
    let body_start = rest.find('{').expect("function body must start");
    let mut depth = 0usize;
    for (idx, ch) in rest[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &rest[..body_start + idx + 1];
                }
            }
            _ => {}
        }
    }
    panic!("function body must close");
}

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
    assert!(ACP_VIEW_SOURCE.contains("profile_left_info"));
    assert!(ACP_VIEW_SOURCE.contains("profile_name: Some(self.profile_display.clone())"));
    assert!(ACP_VIEW_SOURCE.contains("agent-chat-profile-display"));
}

#[test]
fn profile_selector_batch_routing_precedes_mention_picker() {
    let value_route = PROMPT_HANDLER_SOURCE
        .find("batch_select_profile_by_value")
        .expect("prompt popup batch routing must try profile selector");
    let mention_route = PROMPT_HANDLER_SOURCE
        .find("batch_select_mention_item_by_value")
        .expect("prompt popup batch routing must still support @ mentions");
    assert!(
        value_route < mention_route,
        "profile popup selection must be tried before @ mention selection"
    );

    assert!(PROMPT_HANDLER_SOURCE.contains("batch_select_profile_by_semantic_id"));
    assert!(AUTOMATION_COLLECTOR_SOURCE.contains("collect_profile_selector_snapshot"));
    assert!(AUTOMATION_COLLECTOR_SOURCE.contains("panel:profile-selector"));
    assert!(DETACHED_TRANSACTION_PROVIDER_SOURCE
        .contains("batch_select_profile_by_value(&value, self.cx)"));
    assert!(DETACHED_TRANSACTION_PROVIDER_SOURCE.contains("batch_select_profile_by_semantic_id("));
}

#[test]
fn footer_profile_affordance_is_merged_with_left_status_marker() {
    assert!(FOOTER_CHROME_SOURCE.contains("FOOTER_PROFILE_ICON_TOKEN"));
    assert!(FOOTER_CHROME_SOURCE.contains("FOOTER_PROFILE_ICON_PATH"));
    assert!(ACP_VIEW_SOURCE.contains("FOOTER_PROFILE_ICON_TOKEN"));
    assert!(ACP_VIEW_SOURCE.contains("render_profile_status_marker_from_snapshot"));
    assert!(ACP_VIEW_SOURCE.contains("FooterAction::Ai => self.toggle_profile_selector_popup"));
    assert!(!fn_body(ACP_VIEW_SOURCE, "fn footer_buttons_for_thread(")
        .contains("FOOTER_PROFILE_ICON_TOKEN"));
    assert!(!fn_body(ACP_VIEW_SOURCE, "fn footer_buttons_for_thread(").contains("FooterAction::Ai"));
    assert!(FOOTER_POPUP_SOURCE.contains("dispatch_acp_footer_action(action);"));
}

#[test]
fn quote_trigger_selects_agent_chat_profiles_without_context_attachment() {
    assert!(CONTEXT_PICKER_TYPES_SOURCE.contains("Profile"));
    assert!(CONTEXT_PICKER_TYPES_SOURCE.contains("AgentChatProfile"));
    assert!(CONTEXT_PICKER_SOURCE.contains("b'\\'' => ContextPickerTrigger::Profile"));
    let refresh_body = fn_body(ACP_VIEW_SOURCE, "fn refresh_mention_session(");
    assert!(refresh_body.contains("ContextPickerTrigger::Profile"));
    assert!(refresh_body.contains("self.build_profile_picker_items(&query)"));
    assert!(ACP_VIEW_SOURCE.contains("self.select_profile_from_popup(&profile_id, cx);"));
    let accept_body = fn_body(ACP_VIEW_SOURCE, "fn accept_mention_selection_impl(");
    assert!(accept_body.contains("ContextPickerTrigger::Profile"));
    assert!(accept_body.contains("ContextPickerItemKind::AgentChatProfile"));
    assert!(accept_body.contains("Self::replace_text_in_char_range("));
    assert!(accept_body.contains("session.trigger_range.clone()"));
    assert!(accept_body.contains("thread.input.set_text(next_text)"));
}
