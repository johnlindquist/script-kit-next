const ACTION_HANDLER_SOURCE: &str = include_str!("../src/app_actions/handle_action/mod.rs");
const PROFILES_SOURCE: &str = include_str!("../src/ai/agent_chat/profiles.rs");
const ACP_LAUNCH_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/acp_launch.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const FILTER_INPUT_CORE_SOURCE: &str = include_str!("../src/app_impl/filter_input_core.rs");
const FILTER_INPUT_UPDATES_SOURCE: &str = include_str!("../src/app_impl/filter_input_updates.rs");
const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const ACP_THREAD_SOURCE: &str = include_str!("../src/ai/acp/thread.rs");
const ACP_MOD_SOURCE: &str = include_str!("../src/ai/acp/mod.rs");
const ACP_PICKER_POPUP_SOURCE: &str = include_str!("../src/ai/acp/picker_popup.rs");
const CHAT_WINDOW_SOURCE: &str = include_str!("../src/ai/acp/chat_window.rs");
const PROMPT_HANDLER_SOURCE: &str = include_str!("../src/prompt_handler/mod.rs");
const CONTEXT_PICKER_TYPES_SOURCE: &str = include_str!("../src/ai/window/context_picker/types.rs");
const CONTEXT_PICKER_SOURCE: &str = include_str!("../src/ai/window/context_picker/mod.rs");
const CONFIG_TYPES_SOURCE: &str = include_str!("../src/config/types.rs");
const AUTOMATION_COLLECTOR_SOURCE: &str =
    include_str!("../src/windows/automation_surface_collector.rs");
const DETACHED_TRANSACTION_PROVIDER_SOURCE: &str =
    include_str!("../src/windows/automation_transaction_provider.rs");
const FOOTER_CHROME_SOURCE: &str = include_str!("../src/components/footer_chrome.rs");
const FOOTER_POPUP_SOURCE: &str = include_str!("../src/footer_popup.rs");
const STDIN_COMMANDS_SOURCE: &str = include_str!("../src/stdin_commands/mod.rs");
const SPINE_PROFILE_SOURCE: &str = include_str!("../src/spine/catalog_profile.rs");
const STARTUP_SOURCE: &str = include_str!("../src/app_impl/startup.rs");
const STARTUP_NEW_TAB_SOURCE: &str = include_str!("../src/app_impl/startup_new_tab.rs");

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
fn profile_picker_is_main_menu_spine_not_deprecated_popup() {
    assert!(SPINE_PROFILE_SOURCE.contains("agent_chat_profile_picker_entries"));
    assert!(SPINE_PROFILE_SOURCE.contains("selected_agent_chat_profile_picker_id"));
    assert!(SPINE_PROFILE_SOURCE.contains("SpineListRowKind::Profile"));
    assert!(SPINE_PROFILE_SOURCE.contains("resolution_source: ss(\"profile\")"));

    assert!(!ACP_MOD_SOURCE.contains("profile_selector_popup"));
    assert!(!ACP_VIEW_SOURCE.contains("profile_selector_open"));
    assert!(!ACP_VIEW_SOURCE.contains("sync_profile_selector_popup_window_from_cached_parent"));
    assert!(!PROMPT_HANDLER_SOURCE.contains("batch_select_profile_by_value"));
    assert!(!PROMPT_HANDLER_SOURCE.contains("batch_select_profile_by_semantic_id"));
    assert!(!PROMPT_HANDLER_SOURCE.contains("is_profile_selector_popup_window_open"));
    assert!(!AUTOMATION_COLLECTOR_SOURCE.contains("collect_profile_selector_snapshot"));
    assert!(!DETACHED_TRANSACTION_PROVIDER_SOURCE.contains("batch_select_profile_by_value"));
    assert!(!DETACHED_TRANSACTION_PROVIDER_SOURCE.contains("batch_select_profile_by_semantic_id"));
    assert!(!STDIN_COMMANDS_SOURCE.contains("OpenAcpProfilePicker"));
    assert!(!STDIN_COMMANDS_SOURCE.contains("\"openAcpProfilePicker\""));
}

#[test]
fn bare_pipe_stays_in_main_menu_search_for_profile_rows() {
    assert!(!FILTER_INPUT_CORE_SOURCE.contains("AcpProfilePicker"));
    assert!(!FILTER_INPUT_UPDATES_SOURCE.contains("open_tab_ai_acp_with_profile_picker"));
    assert!(!TAB_AI_MODE_SOURCE.contains("open_tab_ai_acp_with_profile_picker"));
    assert!(!CHAT_WINDOW_SOURCE.contains("open_detached_profile_picker"));
}

#[test]
fn spine_profile_submission_persists_pi_profile_before_launch() {
    let body = fn_body(
        TAB_AI_MODE_SOURCE,
        "pub(crate) fn try_submit_spine_prompt_plan_from_enter(",
    );
    assert!(body.contains("persist_agent_chat_profile_selection"));
    assert!(body.contains("save_user_preferences(&prefs)"));
    assert!(body.contains("Agent Chat profile: {profile_name}"));
    assert!(PROFILES_SOURCE.contains("ai.selected_profile_id = Some(entry.id.clone());"));
    assert!(PROFILES_SOURCE.contains("ai.selected_backend = Some(AgentChatBackend::Pi);"));
}

#[test]
fn profile_selection_updates_live_agent_chat_footer_without_relaunch() {
    assert!(ACTION_HANDLER_SOURCE.contains("agent_chat_switch_profile_id_from_action"));
    assert!(ACTION_HANDLER_SOURCE.contains("persist_agent_chat_profile_selection"));
    assert!(ACP_VIEW_SOURCE.contains("set_on_profile_selected"));
    assert!(ACP_THREAD_SOURCE.contains("pub(crate) fn set_profile_display("));
    assert!(ACP_VIEW_SOURCE.contains("pub(crate) fn set_profile_display("));
    assert!(TAB_AI_MODE_SOURCE.contains("view.set_profile_display("));
    assert!(TAB_AI_MODE_SOURCE.contains("select_agent_chat_profile_and_relaunch"));
}

#[test]
fn acp_launch_uses_effective_profile_for_acp_agent_and_model() {
    assert!(ACP_LAUNCH_SOURCE.contains("resolve_selected_pi_launch_with_cwd_override"));
    assert!(ACP_LAUNCH_SOURCE.contains("resolve_focused_text_pi_launch"));
    assert!(ACP_LAUNCH_SOURCE
        .contains("profile_display_name: Some(pi_launch.profile.name.clone().into())"));
    assert!(ACP_LAUNCH_SOURCE.contains("profile_icon_name: pi_launch.profile.icon_name.clone()"));
}

#[test]
fn provider_scoped_pi_model_selection_is_split_before_launch() {
    assert!(PROFILES_SOURCE.contains("parse_provider_model_selection"));
    assert!(PROFILES_SOURCE.contains("profile.provider = Some(provider);"));
    assert!(PROFILES_SOURCE.contains("profile.model = Some(model);"));
}

#[test]
fn primary_pi_provider_catalog_is_codex_only_with_advanced_alternatives() {
    let primary = fn_body(PROFILES_SOURCE, "pub fn pi_provider_model_catalog()");
    assert!(primary.contains(r#"id: "openai-codex""#));
    assert!(primary.contains(r#"display_name: "Codex""#));
    assert!(
        !primary.contains(r#"id: "anthropic""#) && !primary.contains(r#"id: "google""#),
        "primary Agent Chat provider catalog must not surface alternate providers"
    );

    let advanced = fn_body(
        PROFILES_SOURCE,
        "pub fn advanced_pi_provider_model_catalog()",
    );
    assert!(advanced.contains(r#"id: "anthropic""#));
    assert!(advanced.contains(r#"id: "google""#));
}

#[test]
fn profile_display_flows_through_thread_and_footer() {
    assert!(ACP_THREAD_SOURCE.contains("profile_display_name"));
    assert!(ACP_THREAD_SOURCE.contains("pub(crate) fn profile_display"));
    assert!(ACP_VIEW_SOURCE.contains("profile_display: thread.profile_display().to_string()"));
    assert!(ACP_VIEW_SOURCE.contains("profile_left_info"));
    assert!(ACP_VIEW_SOURCE.contains("profile_name: Some(self.profile_display.clone())"));
    assert!(ACP_VIEW_SOURCE.contains("agent-chat-profile-display"));
    assert!(ACP_VIEW_SOURCE.contains("snapshot.profile_display.clone()"));
    assert!(ACP_VIEW_SOURCE.contains(".id(\"acp-profile-display\")"));
    assert!(ACP_VIEW_SOURCE.contains(".id(\"acp-model-display\")"));
}

#[test]
fn shift_tab_routes_to_profile_switcher_copy() {
    assert!(STARTUP_SOURCE.contains("profile_switcher_open_shift_tab"));
    assert!(STARTUP_SOURCE.contains("acp_shift_tab_profile_switcher"));
    assert!(STARTUP_NEW_TAB_SOURCE.contains("acp_shift_tab_profile_switcher"));
    assert!(!STARTUP_SOURCE.contains("Shift+Tab → Agent & Model picker"));
    assert!(!STARTUP_NEW_TAB_SOURCE.contains("Shift+Tab → Agent & Model picker"));
}

#[test]
fn footer_profile_affordance_is_merged_into_left_status_marker() {
    assert!(FOOTER_CHROME_SOURCE.contains("FOOTER_PROFILE_ICON_TOKEN"));
    assert!(FOOTER_CHROME_SOURCE.contains("FOOTER_PROFILE_ICON_PATH"));

    let footer_buttons = fn_body(ACP_VIEW_SOURCE, "fn footer_buttons_for_thread");
    assert!(!footer_buttons.contains("FooterAction::Ai"));
    assert!(!footer_buttons.contains("FOOTER_PROFILE_ICON_TOKEN"));

    assert!(ACP_VIEW_SOURCE.contains("profile_left_info"));
    assert!(ACP_VIEW_SOURCE.contains("render_profile_status_marker_from_snapshot"));
    assert!(
        ACP_VIEW_SOURCE.contains("FooterAction::Ai => self.open_profile_trigger_picker_in_window")
    );

    assert!(FOOTER_POPUP_SOURCE.contains("pub action: Option<FooterAction>"));
    assert!(FOOTER_POPUP_SOURCE.contains("pub icon_token: Option<String>"));
    assert!(FOOTER_POPUP_SOURCE.contains("dispatch_acp_footer_action"));
}

#[test]
fn pipe_trigger_selects_agent_chat_profiles_without_context_attachment() {
    assert!(CONTEXT_PICKER_TYPES_SOURCE.contains("Profile"));
    assert!(CONTEXT_PICKER_TYPES_SOURCE.contains("PROFILE_TRIGGER_CHAR: char = '|'"));
    assert!(CONTEXT_PICKER_TYPES_SOURCE.contains("AgentChatProfile"));
    assert!(CONTEXT_PICKER_SOURCE.contains("b'|' => ContextPickerTrigger::Profile"));
    let accept_body = fn_body(ACP_VIEW_SOURCE, "fn accept_mention_selection_impl(");
    assert!(accept_body.contains("ContextPickerTrigger::Profile"));
    assert!(accept_body.contains("ContextPickerItemKind::AgentChatProfile"));
    assert!(accept_body.contains("Self::replace_text_in_char_range("));
    assert!(accept_body.contains("session.trigger_range.clone()"));
    assert!(accept_body.contains("thread.input.set_text(next_text)"));
    assert!(accept_body.contains("self.select_profile_from_popup(&profile_id, cx);"));
}

#[test]
fn composer_profile_trigger_rows_use_shared_icon_and_selected_chrome() {
    let row_body = fn_body(ACP_PICKER_POPUP_SOURCE, "fn render_picker_row(");
    assert!(row_body.contains("ContextPickerItemKind::AgentChatProfile"));
    assert!(row_body.contains("footer_icon_path_or_profile"));
    assert!(row_body.contains("FOOTER_PROFILE_ICON_TOKEN"));
    assert!(row_body.contains("gpui::svg()"));
    assert!(row_body.contains(".border_l(gpui::px(2.0))"));
}

#[test]
fn config_profile_icon_name_flows_to_footer_marker() {
    assert!(CONFIG_TYPES_SOURCE.contains("pub icon_name: Option<String>"));
    assert!(PROFILES_SOURCE.contains("pub icon_name: Option<String>"));
    assert!(PROFILES_SOURCE.contains("icon_name: profile.icon_name"));
    assert!(ACP_THREAD_SOURCE.contains("profile_icon_name: Option<String>"));
    assert!(ACP_THREAD_SOURCE.contains("pub(crate) fn profile_icon_name(&self)"));
    assert!(ACP_VIEW_SOURCE
        .contains("profile_icon_name: thread.profile_icon_name().map(str::to_string)"));
    assert!(ACP_VIEW_SOURCE.contains("footer_icon_path_or_profile"));
    assert!(FOOTER_POPUP_SOURCE.contains("pub icon_token: Option<String>"));
}
