const ACTION_BUILDER_SOURCE: &str = include_str!("../src/actions/builders/script_context.rs");
const ACTION_DIALOG_SOURCE: &str = include_str!("../src/actions/dialog.rs");
const ACTION_HANDLER_SOURCE: &str = include_str!("../src/app_actions/handle_action/mod.rs");
const PROFILES_SOURCE: &str = include_str!("../src/ai/agent_chat/profiles.rs");
const ACP_LAUNCH_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/acp_launch.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const FILTER_INPUT_UPDATES_SOURCE: &str = include_str!("../src/app_impl/filter_input_updates.rs");
const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const ACP_THREAD_SOURCE: &str = include_str!("../src/ai/acp/thread.rs");
const ACP_MOD_SOURCE: &str = include_str!("../src/ai/acp/mod.rs");
const ACP_PICKER_POPUP_SOURCE: &str = include_str!("../src/ai/acp/picker_popup.rs");
const PROFILE_POPUP_SOURCE: &str = include_str!("../src/ai/acp/profile_selector_popup.rs");
const CHAT_WINDOW_SOURCE: &str = include_str!("../src/ai/acp/chat_window.rs");
const PROMPT_HANDLER_SOURCE: &str = include_str!("../src/prompt_handler/mod.rs");
const CONTEXT_PICKER_TYPES_SOURCE: &str = include_str!("../src/ai/window/context_picker/types.rs");
const CONTEXT_PICKER_SOURCE: &str = include_str!("../src/ai/window/context_picker/mod.rs");
const CONFIG_TYPES_SOURCE: &str = include_str!("../src/config/types.rs");
const ACP_LAUNCH_IMPL_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/acp_launch.rs");
const AUTOMATION_COLLECTOR_SOURCE: &str =
    include_str!("../src/windows/automation_surface_collector.rs");
const DETACHED_TRANSACTION_PROVIDER_SOURCE: &str =
    include_str!("../src/windows/automation_transaction_provider.rs");
const FOOTER_CHROME_SOURCE: &str = include_str!("../src/components/footer_chrome.rs");
const FOOTER_POPUP_SOURCE: &str = include_str!("../src/footer_popup.rs");
const DEVTOOLS_ACT_SOURCE: &str = include_str!("../scripts/devtools/act.ts");

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
fn profile_selection_persistence_uses_stable_ids_and_pi_backend() {
    assert!(PROFILES_SOURCE.contains("persist_agent_chat_profile_selection"));
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
fn profile_picker_selection_defers_host_relaunch_out_of_acp_update() {
    let body = fn_body(ACP_VIEW_SOURCE, "pub(crate) fn select_profile_from_popup(");
    assert!(
        body.contains("let selected_profile_id = profile_id.to_string();"),
        "profile selection must own the selected profile id before deferring the host callback"
    );
    let defer_pos = body
        .find("cx.defer(move |cx|")
        .expect("profile selection must defer host callback out of the ACP update lease");
    let callback_pos = body
        .find("callback(selected_profile_id.clone(), cx);")
        .expect("deferred profile callback must still invoke the host profile switch");
    assert!(
        defer_pos < callback_pos,
        "host profile relaunch callback must run inside cx.defer"
    );
    assert!(
        !body.contains("callback(profile_id.to_string(), cx);"),
        "profile selection must not synchronously invoke the host relaunch callback"
    );
}

#[test]
fn acp_launch_uses_effective_profile_for_acp_agent_and_model() {
    assert!(ACP_LAUNCH_SOURCE.contains("resolve_effective_profile"));
    assert!(ACP_LAUNCH_SOURCE.contains("PiAgentChatLaunch::from_profile"));
    assert!(ACP_LAUNCH_SOURCE.contains("effective_profile.clone()"));
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
    assert!(PROFILE_POPUP_SOURCE.contains("icon_name: Option<String>"));
    assert!(PROFILE_POPUP_SOURCE
        .contains("render_dense_monoline_picker_row_with_leading_visual_and_accessory"));
    assert!(PROFILE_POPUP_SOURCE.contains("footer_icon_path_or_profile"));
    assert!(PROFILE_POPUP_SOURCE.contains("InlineDropdownColors::popup_from_theme"));
    assert!(PROFILE_POPUP_SOURCE.contains(".external_path(icon_path)"));
}

#[test]
fn toolbar_profile_selector_carries_parent_window_for_detached_popup() {
    assert!(ACP_VIEW_SOURCE.contains("AcpToolbarEvent::ToggleProfileSelector(parent)"));
    assert!(ACP_VIEW_SOURCE.contains("this.mention_popup_parent_window = Some(*parent);"));
    assert!(ACP_VIEW_SOURCE.contains("if this.is_setup_mode()"));
    assert!(ACP_VIEW_SOURCE.contains("this.open_profile_picker(cx);"));
    assert!(ACP_VIEW_SOURCE.contains("this.open_profile_trigger_picker(cx);"));
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
    assert!(PROMPT_HANDLER_SOURCE.contains("is_profile_selector_popup_window_open"));
    assert!(AUTOMATION_COLLECTOR_SOURCE.contains("collect_profile_selector_snapshot"));
    assert!(AUTOMATION_COLLECTOR_SOURCE.contains("panel:profile-selector"));
    assert!(DETACHED_TRANSACTION_PROVIDER_SOURCE
        .contains("batch_select_profile_by_value(&value, self.cx)"));
    assert!(DETACHED_TRANSACTION_PROVIDER_SOURCE.contains("batch_select_profile_by_semantic_id("));
}

#[test]
fn footer_profile_affordance_is_merged_into_left_status_marker() {
    assert!(FOOTER_CHROME_SOURCE.contains("FOOTER_PROFILE_ICON_TOKEN"));
    assert!(FOOTER_CHROME_SOURCE.contains("FOOTER_PROFILE_ICON_PATH"));

    let footer_buttons = fn_body(ACP_VIEW_SOURCE, "fn footer_buttons_for_thread");
    assert!(
        !footer_buttons.contains("FooterAction::Ai"),
        "ACP must not add profile as a standalone footer rail button"
    );
    assert!(
        !footer_buttons.contains("FOOTER_PROFILE_ICON_TOKEN"),
        "profile icon token must not be used by ACP footer button specs"
    );

    assert!(ACP_VIEW_SOURCE.contains("profile_selector_open: self.profile_selector_open"));
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
fn profile_affordance_clicks_use_shared_pipe_profile_picker() {
    let render_icon_body = fn_body(ACP_VIEW_SOURCE, "fn render_input_profile_icon(");
    let footer_dispatch_body = fn_body(ACP_VIEW_SOURCE, "pub(crate) fn dispatch_footer_button(");
    let main_window_source = include_str!("../src/app_impl/ui_window.rs");

    assert!(render_icon_body.contains("open_profile_trigger_picker_in_window(window, cx)"));
    assert!(!render_icon_body.contains("toggle_profile_selector_popup(window, cx)"));
    assert!(footer_dispatch_body
        .contains("FooterAction::Ai => self.open_profile_trigger_picker_in_window(window, cx)"));
    assert!(main_window_source.contains("chat.open_profile_trigger_picker_in_window(window, cx)"));
    assert!(ACP_PICKER_POPUP_SOURCE.contains("acp-mention-popup"));
}

#[test]
fn pipe_trigger_selects_agent_chat_profiles_without_context_attachment() {
    assert!(CONTEXT_PICKER_TYPES_SOURCE.contains("Profile"));
    assert!(CONTEXT_PICKER_TYPES_SOURCE.contains("PROFILE_TRIGGER_CHAR: char = '|'"));
    assert!(CONTEXT_PICKER_TYPES_SOURCE.contains("AgentChatProfile"));
    assert!(CONTEXT_PICKER_SOURCE.contains("b'|' => ContextPickerTrigger::Profile"));
    assert!(!CONTEXT_PICKER_SOURCE.contains("b'\\'' => ContextPickerTrigger::Profile"));
    assert!(CONTEXT_PICKER_SOURCE.contains("display: \"|general\""));
    let refresh_body = fn_body(ACP_VIEW_SOURCE, "fn refresh_mention_session(");
    assert!(refresh_body.contains("ContextPickerTrigger::Profile"));
    assert!(refresh_body.contains("self.build_profile_picker_items(&query)"));
    assert!(ACP_VIEW_SOURCE.contains("self.select_profile_from_popup(&profile_id, cx);"));
    assert!(ACP_VIEW_SOURCE.contains("icon_name: entry.icon_name"));
    assert!(CONTEXT_PICKER_TYPES_SOURCE.contains("icon_name: Option<String>"));
    let trigger_body = fn_body(
        ACP_VIEW_SOURCE,
        "pub(crate) fn open_profile_trigger_picker(",
    );
    assert!(
        trigger_body.contains("open_picker_trigger(PROFILE_TRIGGER_STR"),
        "main-menu pipe launch should open the composer profile trigger picker"
    );
    assert!(
        !trigger_body.contains("profile_selector_open = true"),
        "composer pipe trigger must not open the footer profile selector"
    );
    let open_picker_body = fn_body(ACP_VIEW_SOURCE, "fn open_picker_trigger(");
    assert!(open_picker_body.contains("self.set_input(trigger.to_string(), cx);"));
    assert!(open_picker_body.contains("self.refresh_mention_session(cx);"));
    assert!(open_picker_body.contains("self.profile_selector_open = false;"));
    assert!(open_picker_body.contains("close_profile_selector_popup_window"));
    let accept_body = fn_body(ACP_VIEW_SOURCE, "fn accept_mention_selection_impl(");
    assert!(accept_body.contains("ContextPickerTrigger::Profile"));
    assert!(accept_body.contains("ContextPickerItemKind::AgentChatProfile"));
    assert!(accept_body.contains("Self::replace_text_in_char_range("));
    assert!(accept_body.contains("session.trigger_range.clone()"));
    assert!(accept_body.contains("thread.input.set_text(next_text)"));
}

#[test]
fn pipe_trigger_routes_from_main_menu_to_composer_profile_trigger_picker() {
    assert!(TAB_AI_MODE_SOURCE.contains("open_tab_ai_acp_with_profile_picker"));
    assert!(TAB_AI_MODE_SOURCE.contains("view.open_profile_trigger_picker_in_window(window, cx)"));
    assert!(!TAB_AI_MODE_SOURCE.contains("'|' => view.open_profile_picker_in_window(window, cx)"));
    assert!(ACP_VIEW_SOURCE.contains("pub(crate) fn open_profile_trigger_picker_in_window("));
    assert!(CHAT_WINDOW_SOURCE.contains("view.open_profile_trigger_picker_in_window(window, cx)"));
    assert!(!CHAT_WINDOW_SOURCE.contains("view.open_profile_picker_in_window(window, cx)"));
    let initial_input_body = fn_body(
        ACP_LAUNCH_IMPL_SOURCE,
        "pub(super) fn tab_ai_acp_initial_input_for_launch(",
    );
    assert!(initial_input_body.contains("Some(trigger @ ('/' | '@' | '|'))"));
}

#[test]
fn composer_profile_trigger_rows_use_shared_icon_and_selected_chrome() {
    let row_body = fn_body(ACP_PICKER_POPUP_SOURCE, "fn render_picker_row(");
    assert!(row_body.contains("ContextPickerItemKind::AgentChatProfile"));
    assert!(row_body.contains("footer_icon_path_or_profile"));
    assert!(row_body.contains("FOOTER_PROFILE_ICON_TOKEN"));
    assert!(row_body.contains("gpui::svg()"));
    assert!(row_body.contains(".border_l(gpui::px(2.0))"));
    let outer_row_tail = row_body
        .split(".id(SharedString::from(format!(\"acp-mention-popup-row-{idx}\")))")
        .nth(1)
        .expect("outer row body must be present");
    assert!(
        !outer_row_tail.contains(".border_l(gpui::px(2.0))"),
        "composer picker accent bar should live inside the selected item background"
    );
}

#[test]
fn pipe_set_filter_routes_before_menu_syntax_trigger_popup() {
    let body = fn_body(
        FILTER_INPUT_UPDATES_SOURCE,
        "pub(crate) fn set_filter_text_immediate(",
    );
    let special_route = body
        .find("ScriptListSpecialEntry::AcpProfilePicker")
        .expect("set_filter_text_immediate must explicitly route bare pipe");
    let menu_popup = body
        .find("run_menu_syntax_trigger_popup_state_machine")
        .expect("menu syntax trigger popup state machine exists");
    assert!(
        special_route < menu_popup,
        "programmatic setFilter('|') must route to Agent Chat profile picker before generic menu-syntax popup can open"
    );
    assert!(body.contains("open_tab_ai_acp_with_profile_picker(window, cx)"));
    assert!(body.contains("close_menu_syntax_trigger_popup_window"));
}

#[test]
fn devtools_profile_popup_select_is_lifecycle_sensitive() {
    assert!(DEVTOOLS_ACT_SOURCE.contains("function isPromptPopupTargetReceipt("));
    assert!(DEVTOOLS_ACT_SOURCE.contains("targetInfo(receipt)"));
    assert!(DEVTOOLS_ACT_SOURCE.contains("function isSuccessfulPromptPopupSelect("));
    assert!(DEVTOOLS_ACT_SOURCE.contains("isPromptPopupTargetReceipt(targetReceipt)"));
    assert!(DEVTOOLS_ACT_SOURCE.contains("promptPopupSelectClosedSource"));
    assert!(DEVTOOLS_ACT_SOURCE.contains("resolvePostActionLifecycle("));
    assert!(DEVTOOLS_ACT_SOURCE.contains("(agent-chat-profile:)?(general|text|script-kit|acp)"));
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

#[test]
fn setup_mode_does_not_call_live_thread_for_profile_picker_or_footer_response() {
    let pastable_response = fn_body(ACP_VIEW_SOURCE, "fn pastable_response_text(");
    assert!(
        pastable_response.contains("if self.is_setup_mode()")
            && pastable_response.contains("return None;"),
        "setup-mode ACP footer response lookup must not dereference live_thread"
    );

    let composer_transition = fn_body(ACP_VIEW_SOURCE, "fn apply_composer_picker_transition(");
    assert!(
        composer_transition.contains("if !self.is_setup_mode()")
            && composer_transition.contains("clear_slash_input")
            && composer_transition.contains("insert_slash_input"),
        "setup-mode picker transitions must not mutate composer input through live_thread"
    );
    assert!(ACP_VIEW_SOURCE.contains("event = \"acp_setup_profile_selector_key_handled\""));
    assert!(ACP_VIEW_SOURCE.contains("event = \"acp_popup_sync_setup_mode_profile_only\""));
    assert!(ACP_VIEW_SOURCE.contains("event = \"acp_footer_snapshot_hidden_setup_mode\""));
    assert!(ACP_VIEW_SOURCE.contains("event = \"acp_footer_action_ignored_setup_mode\""));
    assert!(
        TAB_AI_MODE_SOURCE.contains("event = \"agent_chat_profile_selection_relaunch_from_setup\"")
    );
    let collect_state = fn_body(ACP_VIEW_SOURCE, "pub(crate) fn collect_acp_state_snapshot(");
    assert!(
        collect_state.contains("self.is_setup_mode() || setup_snapshot.is_some()"),
        "automation state collection must treat runtime setup recovery as setup before live_thread"
    );
    let focused_elements = fn_body(
        ACP_VIEW_SOURCE,
        "pub(crate) fn collect_focused_text_mini_elements(",
    );
    assert!(
        focused_elements
            .contains("self.is_setup_mode() || self.build_setup_protocol_snapshot(cx).is_some()"),
        "focused-text automation elements must not read live_thread while setup recovery is active"
    );
}
