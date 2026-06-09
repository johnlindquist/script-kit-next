//! Source-level contract for AURP-10 actions-interceptor key intents.
//!
//! The actions interceptor must route keys through the shared actions dialog
//! before dispatching local main-window intents such as Cmd+K toggle and
//! embedded Agent Chat Cmd+W close.

const STARTUP: &str = include_str!("../src/app_impl/startup.rs");
const ACTIONS_DIALOG: &str = include_str!("../src/actions/dialog.rs");
const ACTIONS_DIALOG_IMPL: &str = include_str!("../src/app_impl/actions_dialog.rs");
const RENDER_SCRIPT_LIST: &str = include_str!("../src/render_script_list/mod.rs");

fn source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end: {end_pat}"));
    &source[start..start + end_rel]
}

#[test]
fn actions_chords_are_classified_as_named_main_window_intents() {
    assert!(
        STARTUP.contains("enum MainWindowActionsKeyIntent"),
        "startup.rs must declare the actions-interceptor key intent enum."
    );
    assert!(
        STARTUP.contains("ToggleActions"),
        "Cmd+K must be named by behavior instead of remaining an inline chord branch."
    );
    assert!(
        STARTUP.contains("CloseEmbeddedAgentChatWindow"),
        "Embedded Agent Chat Cmd+W must be named by behavior instead of remaining an inline chord branch."
    );

    let classifier = source_between(
        STARTUP,
        "fn main_window_actions_key_intent(",
        "\n}\n\nimpl ScriptListApp",
    );
    assert!(
        classifier.contains("key.eq_ignore_ascii_case(\"k\")")
            && classifier.contains("Some(MainWindowActionsKeyIntent::ToggleActions)"),
        "The classifier must preserve Cmd+K as the actions toggle intent."
    );
    assert!(
        classifier.contains("key.eq_ignore_ascii_case(\"w\")")
            && classifier.contains("matches!(current_view, AppView::AgentChatView { .. })")
            && classifier
                .contains("Some(MainWindowActionsKeyIntent::CloseEmbeddedAgentChatWindow)"),
        "The classifier must preserve embedded Agent Chat Cmd+W as a view-specific close intent."
    );
    assert!(
        classifier.contains("let has_shift = event.keystroke.modifiers.shift;")
            && classifier.contains("!has_shift"),
        "The classifier must preserve the existing non-shift modifier gate."
    );
}

#[test]
fn actions_interceptor_routes_shared_dialog_before_local_intents() {
    let interceptor = source_between(
        STARTUP,
        "let actions_interceptor = cx.intercept_keystrokes({",
        "app.gpui_input_subscriptions.push(actions_interceptor);",
    );
    let shared_dialog_route = interceptor
        .find("route_key_to_actions_dialog(")
        .expect("actions interceptor must route through the shared actions dialog");
    let displayed_action_shortcut_route = interceptor
        .find("try_execute_main_list_action_shortcut_from_display(")
        .expect("actions interceptor must route displayed action shortcuts");
    let local_intent = interceptor
        .find("main_window_actions_key_intent(&this.current_view, event)")
        .expect("actions interceptor must classify local key intent after dialog routing");

    assert!(
        shared_dialog_route < local_intent,
        "Shared actions-dialog routing must run before local key intent dispatch."
    );
    assert!(
        displayed_action_shortcut_route < local_intent,
        "Displayed action shortcut routing must run before local key intent dispatch."
    );
    assert!(
        interceptor.contains("this.handle_main_window_actions_key_intent(intent, window, cx)"),
        "The actions interceptor must delegate local behavior to the named intent handler."
    );
    assert!(
        !interceptor.contains("if has_cmd && key.eq_ignore_ascii_case(\"k\") && !has_shift"),
        "The old inline Cmd+K branch must not be reintroduced inside the interceptor."
    );
    assert!(
        !interceptor.contains("// Handle Cmd+W for AgentChatView"),
        "The old inline Agent Chat Cmd+W branch must not be reintroduced inside the interceptor."
    );
}

#[test]
fn closed_popup_cmd_shift_k_uses_displayed_action_shortcut_metadata() {
    let interceptor = source_between(
        STARTUP,
        "let actions_interceptor = cx.intercept_keystrokes({",
        "app.gpui_input_subscriptions.push(actions_interceptor);",
    );

    assert!(
        interceptor.contains("try_execute_main_list_action_shortcut_from_display("),
        "Closed-popup main-list action shortcuts must resolve through displayed Action.shortcut metadata."
    );
    assert!(
        !interceptor
            .contains("key.eq_ignore_ascii_case(\"k\")\n                            && has_shift"),
        "Closed-popup displayed action shortcut routing must not be limited to Cmd+Shift+K."
    );
    assert!(
        !interceptor.contains("handle_action(\"add_shortcut\".to_string(), window, cx)"),
        "Cmd+Shift+K must not hard-code add_shortcut in the main-window interceptor."
    );
    assert!(
        RENDER_SCRIPT_LIST.contains("try_execute_main_list_action_shortcut_from_display("),
        "Render-list closed-popup shortcut fallback must also resolve displayed Action.shortcut metadata."
    );
    assert!(
        RENDER_SCRIPT_LIST.contains("sync_main_list_displayed_action_shortcut_keybindings("),
        "Render-list must register displayed Action.shortcut metadata into GPUI keybindings."
    );
    assert!(
        RENDER_SCRIPT_LIST.contains("MainListDisplayedActionShortcut"),
        "Render-list must receive displayed shortcut keybindings through the generic GPUI action envelope."
    );
    assert!(
        ACTIONS_DIALOG.contains("#[action(namespace = script_kit, no_json, no_register)]"),
        "The displayed-shortcut payload action must avoid inventory registration because this package links lib and binary module trees."
    );
    assert!(
        !RENDER_SCRIPT_LIST.contains("Shortcut Cmd+Shift+K -> add_shortcut"),
        "Render-list fallback must not reintroduce the old hard-coded Cmd+Shift+K add_shortcut route."
    );
}

#[test]
fn render_list_displayed_shortcut_fallback_normalizes_shifted_key_case() {
    let handler = source_between(
        RENDER_SCRIPT_LIST,
        "let handle_key = cx.listener(",
        "\n        let handle_key_up =",
    );

    let key_match = handler
        .find("let key_match = key_str.to_ascii_lowercase();")
        .expect("render-list key handler must normalize shifted key names before branch matching");
    let cmd_match = handler
        .find("match key_match.as_str()")
        .expect("render-list key handler must match normalized key names");
    let displayed_route = handler
        .find("try_execute_main_list_action_shortcut_from_display(")
        .expect("render-list key handler must route displayed action shortcuts");

    assert!(
        key_match < cmd_match && cmd_match < displayed_route,
        "Cmd+Shift+K can arrive as uppercase K, so branch selection must normalize before routing displayed shortcuts."
    );
}

#[test]
fn detached_actions_window_keeps_parent_shortcut_router_active() {
    assert!(
        ACTIONS_DIALOG_IMPL
            .contains("!self.show_actions_popup && !crate::actions::is_actions_window_open()"),
        "Parent actions-dialog routing must stay active while the detached actions window is open."
    );

    let interceptor = source_between(
        STARTUP,
        "let actions_interceptor = cx.intercept_keystrokes({",
        "app.gpui_input_subscriptions.push(actions_interceptor);",
    );
    let actions_window_route = interceptor
        .find("if is_actions {")
        .expect("actions interceptor must explicitly handle actions-window events");
    let shared_route = interceptor[actions_window_route..]
        .find("this.route_key_to_actions_dialog(")
        .expect(
            "actions-window events must route through the shared actions dialog before returning",
        );
    let actions_window_fallthrough_return = interceptor[actions_window_route..]
        .find("if actions_key_routed {\n                        return;\n                    }\n                    return;")
        .expect("actions-window branch must still avoid falling through to main-window shortcuts");

    assert!(
        shared_route < actions_window_fallthrough_return,
        "Actions-window key events must not return before the shared shortcut router can handle visible Action.shortcut metadata."
    );
}

#[test]
fn actions_key_intent_handler_preserves_existing_effects() {
    let handler = source_between(
        STARTUP,
        "fn handle_main_window_actions_key_intent(",
        "\n    pub(crate) fn new(",
    );
    assert!(
        handler.contains("self.handle_cmd_k_actions_toggle(window, cx)"),
        "The ToggleActions intent must preserve the existing shared Cmd+K toggle helper."
    );
    assert!(
        handler.contains("self.close_tab_ai_harness_terminal_with_window(window, cx)")
            && handler.contains("self.close_and_reset_window(cx)"),
        "The embedded Agent Chat close intent must preserve terminal cleanup and main-window reset."
    );
}
