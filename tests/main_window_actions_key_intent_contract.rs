//! Source-level contract for AURP-10 actions-interceptor key intents.
//!
//! The actions interceptor must route keys through the shared actions dialog
//! before dispatching local main-window intents such as Cmd+K toggle and
//! embedded Agent Chat Cmd+W close.

const STARTUP: &str = include_str!("../src/app_impl/startup.rs");
const ACTIONS_DIALOG_IMPL: &str = include_str!("../src/app_impl/actions_dialog.rs");

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
