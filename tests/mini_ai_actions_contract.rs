const PROMPT_AI: &str = include_str!("../src/app_impl/prompt_ai.rs");
const APP_STATE: &str = include_str!("../src/main_sections/app_state.rs");
const RENDER_IMPL: &str = include_str!("../src/main_sections/render_impl.rs");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let end_index = source[start_index..]
        .find(end)
        .map(|offset| start_index + offset)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"));
    &source[start_index..end_index]
}

// @lat: [[lat.md/tests/mini-window-contract#Mini AI actions]]
#[test]
fn mini_ai_actions_callback_sends_typed_parent_request() {
    let body = source_between(
        PROMPT_AI,
        "chat.set_on_show_actions",
        "self.current_view = AppView::ChatPrompt",
    );
    assert!(
        !body.contains("let _ = &app_weak;"),
        "Mini AI actions callback must not be a log-only app_weak capture"
    );
    assert!(
        body.contains("MiniAiUiRequest::ToggleActions") && body.contains("actions_sender.try_send"),
        "Mini AI actions callback must send a typed ToggleActions request"
    );
    assert!(
        APP_STATE.contains("enum MiniAiUiRequest") && APP_STATE.contains("ToggleActions"),
        "app state must define the typed Mini AI UI request channel"
    );
}

// @lat: [[lat.md/tests/mini-window-contract#Mini AI actions receiver]]
#[test]
fn mini_ai_actions_receiver_dispatches_through_real_window() {
    let body = source_between(
        RENDER_IMPL,
        "while let Ok(request) = self.inline_chat_actions_receiver.try_recv()",
        "// Check for inline chat continue",
    );
    assert!(
        body.contains("dispatch_actions_toggle_for_current_view(window, cx, source)"),
        "Mini AI actions receiver must dispatch through the parent window"
    );
}
