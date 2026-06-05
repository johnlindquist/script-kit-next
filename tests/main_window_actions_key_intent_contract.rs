//! Source-level contract for AURP-10 actions-interceptor key intents.
//!
//! The actions interceptor must route keys through the shared actions dialog
//! before dispatching local main-window intents such as Cmd+K toggle and
//! embedded ACP Cmd+W close.

const STARTUP: &str = include_str!("../src/app_impl/startup.rs");

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
        STARTUP.contains("CloseEmbeddedAcpWindow"),
        "Embedded ACP Cmd+W must be named by behavior instead of remaining an inline chord branch."
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
            && classifier.contains("matches!(current_view, AppView::AcpChatView { .. })")
            && classifier.contains("Some(MainWindowActionsKeyIntent::CloseEmbeddedAcpWindow)"),
        "The classifier must preserve embedded ACP Cmd+W as a view-specific close intent."
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
    let local_intent = interceptor
        .find("main_window_actions_key_intent(&this.current_view, event)")
        .expect("actions interceptor must classify local key intent after dialog routing");

    assert!(
        shared_dialog_route < local_intent,
        "Shared actions-dialog routing must run before local key intent dispatch."
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
        !interceptor.contains("// Handle Cmd+W for AcpChatView"),
        "The old inline ACP Cmd+W branch must not be reintroduced inside the interceptor."
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
        "The embedded ACP close intent must preserve terminal cleanup and main-window reset."
    );
}
