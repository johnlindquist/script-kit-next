const ACTIONS: &str = include_str!("../src/render_builtins/actions.rs");

#[test]
fn builtin_actions_window_open_feedback_uses_named_state() {
    assert!(
        ACTIONS.contains("enum BuiltInActionsWindowFeedback")
            && ACTIONS.contains("FileSearch")
            && ACTIONS.contains("ClipboardHistory"),
        "built-in actions popup open feedback should be driven by named host states"
    );
    assert!(
        ACTIONS.contains("BuiltInActionsWindowFeedback::FileSearch")
            && ACTIONS.contains("BuiltInActionsWindowFeedback::ClipboardHistory")
            && ACTIONS.contains("actions_window_feedback.opened_log()")
            && ACTIONS.contains("actions_window_feedback.failure_log(e)"),
        "file-search and clipboard actions popup logs should derive from the named feedback state"
    );
    assert!(
        ACTIONS.contains("\"File search actions popup window opened\"")
            && ACTIONS.contains("\"Clipboard actions popup window opened\"")
            && ACTIONS.contains("format!(\"Failed to open actions window: {error}\")"),
        "built-in actions popup feedback should preserve existing log copy"
    );
}
