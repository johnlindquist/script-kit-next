const PROMPT_AI: &str = include_str!("../src/app_impl/prompt_ai.rs");
const UI_WINDOW: &str = include_str!("../src/app_impl/ui_window.rs");
const WINDOW_RESIZE: &str = include_str!("../src/window_resize/mod.rs");

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

// @lat: [[lat.md/tests/mini-window-contract#Mini AI sizing]]
#[test]
fn inline_mini_ai_uses_mode_aware_resize_helper() {
    let body = source_between(
        PROMPT_AI,
        "pub fn show_inline_ai_chat(",
        "    /// Compatibility shim",
    );
    assert!(
        !body.contains("resize_to_view_sync(ViewType::DivPrompt, 0)"),
        "show_inline_ai_chat must not resize Mini AI through raw DivPrompt"
    );
    assert!(
        body.contains("compact_ai_view_type_for_mode(self.main_window_mode)"),
        "show_inline_ai_chat must size ChatPrompt through main-window mode"
    );
}

// @lat: [[lat.md/tests/mini-window-contract#MiniPrompt sizing]]
#[test]
fn mini_prompt_has_its_own_compact_view_type() {
    let calculate = source_between(
        UI_WINDOW,
        "pub(crate) fn calculate_window_size_params",
        "pub(crate) fn calculate_window_size_params_if_current_view",
    );
    let mini_prompt_arm = source_between(calculate, "AppView::MiniPrompt", "AppView::MicroPrompt");
    assert!(
        mini_prompt_arm.contains("mini_prompt_view_type()"),
        "AppView::MiniPrompt must return the compact MiniPrompt view type"
    );
    assert!(
        !mini_prompt_arm.contains("ViewType::ArgPromptWithChoices"),
        "AppView::MiniPrompt must not borrow full ArgPrompt width"
    );
    assert!(
        WINDOW_RESIZE.contains("MiniPrompt")
            && WINDOW_RESIZE.contains("ViewType::MiniPrompt")
            && WINDOW_RESIZE.contains("ViewType::MiniAiChat"),
        "window_resize must define MiniPrompt and MiniAiChat view types"
    );
}

// @lat: [[lat.md/tests/mini-window-contract#Chat and ACP mode sizing]]
#[test]
fn chat_and_acp_sizing_branch_on_main_window_mode() {
    let calculate = source_between(
        UI_WINDOW,
        "pub(crate) fn calculate_window_size_params",
        "pub(crate) fn calculate_window_size_params_if_current_view",
    );
    let chat_arm = source_between(calculate, "AppView::ChatPrompt", "AppView::TermPrompt");
    assert!(
        chat_arm.contains("compact_ai_view_type_for_mode(self.main_window_mode)"),
        "AppView::ChatPrompt must branch on main_window_mode"
    );
    let acp_arm = source_between(calculate, "AppView::AcpChatView", "AppView::ConfirmPrompt");
    assert!(
        acp_arm.contains("compact_ai_view_type_for_mode(self.main_window_mode)"),
        "AppView::AcpChatView must branch on main_window_mode"
    );
}
