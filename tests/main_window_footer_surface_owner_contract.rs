//! Source-level contract for the native main-window footer surface owner.

const APP_VIEW_STATE_SOURCE: &str = include_str!("../src/main_sections/app_view_state.rs");
const UI_WINDOW_SOURCE: &str = include_str!("../src/app_impl/ui_window.rs");

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature: {signature}"));
    let after_start = &source[start..];
    let open = after_start
        .find('{')
        .unwrap_or_else(|| panic!("missing function body for: {signature}"));
    let mut depth = 0usize;
    for (offset, ch) in after_start[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &after_start[..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function body: {signature}");
}

// doc-anchor-removed: [[removed-docs contract]]
#[test]
fn app_view_owns_native_footer_surface_map() {
    let body = function_body(APP_VIEW_STATE_SOURCE, "pub(crate) fn native_footer_surface");
    for expected in [
        "AppView::ScriptList => Some(\"script_list\")",
        "AppView::QuickTerminalView { .. } => Some(\"quick_terminal\")",
        "AppView::AcpChatView { .. } => Some(\"acp_chat\")",
        "AppView::ConfirmPrompt { .. } => Some(\"confirm_prompt\")",
        "AppView::TermPrompt { .. }",
        "AppView::MicroPrompt { .. }",
    ] {
        assert!(
            body.contains(expected),
            "AppView::native_footer_surface must declare footer ownership for `{expected}`"
        );
    }
    assert!(
        !body.contains("_ => None"),
        "native footer ownership must remain explicit so new AppView variants cannot inherit footer behavior silently"
    );
}

// doc-anchor-removed: [[removed-docs contract]]
#[test]
fn ui_window_delegates_footer_surface_to_app_view_contract() {
    let body = function_body(UI_WINDOW_SOURCE, "fn main_window_footer_surface");
    assert!(
        body.contains("self.current_view.native_footer_surface()"),
        "ui_window must delegate footer surface identity to AppView::native_footer_surface"
    );
    assert!(
        !body.contains("match &self.current_view"),
        "ui_window must not duplicate the AppView footer surface map"
    );
}
