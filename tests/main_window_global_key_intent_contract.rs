//! Source-level contract for AURP-08 named global key intents.
//!
//! The main-window interceptor should classify global shortcuts into named
//! intents before dispatch, so agents can reason about the behavior owner
//! without re-deriving modifier gates from inline branches.

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

// doc-anchor-removed: [[removed-docs Key Intent Routing]]
#[test]
fn cmd_enter_to_acp_is_classified_as_a_named_global_key_intent() {
    assert!(
        STARTUP.contains("enum MainWindowGlobalKeyIntent"),
        "startup.rs must declare the main-window global key intent enum."
    );
    assert!(
        STARTUP.contains("OpenAcpWithCurrentContext"),
        "Cmd+Enter ACP entry must be named by behavior, not by raw key chord."
    );

    let classifier = source_between(
        STARTUP,
        "fn main_window_global_key_intent(",
        "\n}\n\nimpl ScriptListApp",
    );
    assert!(
        classifier.contains("crate::ui_foundation::is_key_enter(key)")
            && classifier.contains("event.keystroke.modifiers.platform")
            && classifier.contains("let has_shift = event.keystroke.modifiers.shift;")
            && classifier.contains("&& !has_shift")
            && classifier.contains("&& !event.keystroke.modifiers.alt")
            && classifier.contains("&& !event.keystroke.modifiers.control"),
        "The classifier must preserve the exact Cmd+Enter modifier gate."
    );
    assert!(
        classifier.contains("Some(MainWindowGlobalKeyIntent::OpenAcpWithCurrentContext)"),
        "The classifier must return the behavior-named ACP context intent."
    );
}

#[test]
fn tab_interceptor_dispatches_the_named_global_key_intent() {
    let interceptor = source_between(
        STARTUP,
        "let tab_interceptor = cx.intercept_keystrokes({",
        "app.gpui_input_subscriptions.push(tab_interceptor);",
    );
    assert!(
        interceptor.contains("let global_key_intent = main_window_global_key_intent(event);"),
        "The interceptor must ask the classifier for global key intent."
    );
    assert!(
        interceptor.contains("if let Some(intent) = global_key_intent"),
        "The interceptor must dispatch by named intent presence."
    );
    assert!(
        interceptor.contains("this.handle_main_window_global_key_intent(intent, cx)"),
        "The interceptor must delegate behavior to the named intent handler."
    );
    assert!(
        !interceptor.contains("let is_global_ai_chord"),
        "The interceptor must not reintroduce the raw inline Cmd+Enter chord branch."
    );
}

#[test]
fn global_key_intent_handler_routes_to_the_shared_acp_context_helper() {
    let handler = source_between(
        STARTUP,
        "fn handle_main_window_global_key_intent(",
        "\n    pub(crate) fn new(",
    );
    assert!(
        handler.contains("MainWindowGlobalKeyIntent::OpenAcpWithCurrentContext =>"),
        "The handler must own the ACP context intent arm."
    );
    assert!(
        handler.contains("self.try_route_global_cmd_enter_to_acp_context_capture(cx)"),
        "The ACP context intent must preserve the shared routing helper."
    );
}
