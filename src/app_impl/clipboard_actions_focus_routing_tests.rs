// Regression tests for clipboard actions dialog keyboard routing.
//
// These are source-audit tests that guard against regressions where:
// - Up/Down arrows navigate clipboard list instead of actions list when popup is open
// - Typing/backspace/enter/escape are not routed to clipboard actions dialog

#[cfg(test)]
mod tests {
    use std::fs;

    fn read_app_impl_sources() -> String {
        let files = [
            "src/app_impl/startup_new_arrow.rs",
            "src/app_impl/startup_new_actions.rs",
        ];

        let mut content = String::new();
        for file in files {
            content.push_str(
                &fs::read_to_string(file).unwrap_or_else(|_| panic!("Failed to read {}", file)),
            );
            content.push('\n');
        }
        content
    }

    fn get_arrow_interceptor_section(content: &str) -> &str {
        let start = content
            .find("let arrow_interceptor = cx.intercept_keystrokes")
            .expect("Arrow interceptor not found");
        let after_start = &content[start..];
        let end = after_start
            .find("let actions_interceptor")
            .unwrap_or(after_start.len().min(5000));
        &content[start..start + end]
    }

    fn get_actions_interceptor_section(content: &str) -> &str {
        let start = content
            .find("let actions_interceptor = cx.intercept_keystrokes")
            .expect("actions_interceptor not found");
        let after_start = &content[start..];
        let end = after_start
            .find("app.gpui_input_subscriptions.push(actions_interceptor);")
            .unwrap_or(after_start.len());
        &content[start..start + end]
    }

    #[test]
    fn clipboard_arrow_routing_checks_actions_popup_first() {
        let content = read_app_impl_sources();
        let arrow_section = get_arrow_interceptor_section(&content);

        let clipboard_pos = arrow_section
            .find("AppView::ClipboardHistoryView {")
            .expect("ClipboardHistoryView case not found in arrow interceptor");
        let after_clipboard = &arrow_section[clipboard_pos..];
        let check_end = after_clipboard.len().min(1200);
        let clipboard_handler = &after_clipboard[..check_end];

        assert!(
            clipboard_handler.contains("show_actions_popup"),
            "ClipboardHistoryView arrow handler must check show_actions_popup before list navigation.\nSection:\n{}",
            clipboard_handler
        );
        assert!(
            clipboard_handler.contains("actions_dialog"),
            "ClipboardHistoryView arrow handler must route to actions_dialog when popup is open."
        );
        assert!(
            clipboard_handler.contains("notify_actions_window"),
            "ClipboardHistoryView arrow handler must notify actions window after dialog navigation."
        );
    }

    #[test]
    fn clipboard_actions_interceptor_routes_modal_keys_to_dialog() {
        let content = read_app_impl_sources();
        let actions_section = get_actions_interceptor_section(&content);

        assert!(
            actions_section.contains("AppView::ClipboardHistoryView { .. }"),
            "actions_interceptor must include ClipboardHistoryView in Cmd+K toggle handling."
        );
        assert!(
            actions_section.contains("toggle_clipboard_actions"),
            "actions_interceptor must call toggle_clipboard_actions for ClipboardHistoryView."
        );
        assert!(
            actions_section.contains("ActionsDialogHost::ClipboardHistory"),
            "actions_interceptor must route modal keys using ActionsDialogHost::ClipboardHistory."
        );
    }

    #[test]
    fn actions_interceptor_skips_arrow_keys_for_non_file_search_modal_routing() {
        let content = read_app_impl_sources();
        let actions_section = get_actions_interceptor_section(&content);

        let anchor = actions_section
            .find("if !matches!(this.current_view, AppView::FileSearchView { .. })")
            .expect("Non-FileSearch modal routing branch not found");
        let branch = &actions_section[anchor..(anchor + 700).min(actions_section.len())];

        assert!(
            branch.contains("key == \"up\"")
                && branch.contains("key == \"arrowup\"")
                && branch.contains("key == \"down\"")
                && branch.contains("key == \"arrowdown\""),
            "actions_interceptor must skip arrow keys in non-FileSearch modal routing to avoid double-processing with arrow_interceptor."
        );
    }
}
