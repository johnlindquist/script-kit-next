//! Regression tests for clipboard actions menu handler coverage.
//!
//! Ensures every clipboard action ID exposed in the actions menu has a handler
//! in `ScriptListApp::handle_action`.

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;

    fn clipboard_builder_ids() -> BTreeSet<String> {
        let region = fs::read_to_string("src/actions/builders/clipboard.rs")
            .expect("Failed to read src/actions/builders/clipboard.rs");

        let mut ids = BTreeSet::new();
        let re = regex::Regex::new(r#"Action::new\(\s*"(clipboard_[a-z0-9_]+)""#)
            .expect("regex compile failed");
        for cap in re.captures_iter(&region) {
            ids.insert(cap[1].to_string());
        }
        ids
    }

    fn app_action_handler_region() -> String {
        let mut files: Vec<PathBuf> = fs::read_dir("src/app_actions/handle_action")
            .expect("Failed to read src/app_actions/handle_action")
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
            .collect();
        files.sort();

        let mut content = fs::read_to_string("src/app_actions/handle_action.rs")
            .expect("Failed to read src/app_actions/handle_action.rs");
        for file in files {
            content.push('\n');
            content.push_str(
                &fs::read_to_string(&file)
                    .unwrap_or_else(|_| panic!("Failed to read {}", file.display())),
            );
        }

        content
    }

    #[test]
    fn all_clipboard_menu_action_ids_are_handled() {
        let ids = clipboard_builder_ids();
        let handler = app_action_handler_region();

        let missing: Vec<String> = ids
            .iter()
            .filter(|id| !handler.contains(&format!("\"{}\"", id)))
            .cloned()
            .collect();

        assert!(
            missing.is_empty(),
            "Clipboard menu actions missing handlers in app_actions.rs: {:?}",
            missing
        );
    }

    #[test]
    fn file_attach_to_ai_uses_deferred_helper_after_hiding_main_window() {
        let handler = app_action_handler_region();
        let branch_start = handler
            .find("\"attach_to_ai\" => {")
            .expect("Expected attach_to_ai handler branch");
        let branch = &handler[branch_start..];

        let helper_call_index = branch
            .find("self.open_ai_window_after_main_hide(")
            .expect("Expected attach_to_ai to delegate deferred AI open to helper");

        assert!(
            helper_call_index > 0,
            "attach_to_ai should route through open_ai_window_after_main_hide"
        );

        let helper_start = handler
            .find("fn open_ai_window_after_main_hide(")
            .expect("Expected deferred AI helper implementation");
        let helper = &handler[helper_start..];

        let hide_index = helper
            .find("self.hide_main_and_reset(cx);")
            .expect("Expected helper to hide main window first");
        let spawn_index = helper
            .find("cx.spawn(async move |this, cx| {")
            .expect("Expected helper to defer AI open via spawn");
        let open_index = helper
            .find("ai::open_ai_window(cx)")
            .expect("Expected helper to open AI window");

        assert!(
            hide_index < spawn_index && spawn_index < open_index,
            "helper should hide main, then defer open_ai_window via cx.spawn"
        );
    }

    #[test]
    fn clipboard_attach_to_ai_delegates_window_open_to_deferred_helper() {
        let handler = app_action_handler_region();
        let branch_start = handler
            .find("\"clipboard_attach_to_ai\" => {")
            .expect("Expected clipboard_attach_to_ai handler branch");
        let branch = &handler[branch_start..];

        let helper_call_index = branch
            .find("self.open_ai_window_after_main_hide(")
            .expect("Expected clipboard_attach_to_ai to use deferred AI helper");
        let before_helper = &branch[..helper_call_index];

        assert!(
            !before_helper.contains("ai::open_ai_window(cx)"),
            "clipboard_attach_to_ai should not open AI window synchronously before helper"
        );
    }

    #[test]
    fn clipboard_file_attach_to_ai_queues_attachment_instead_of_input_text() {
        let handler = app_action_handler_region();
        let branch_start = handler
            .find("\"clipboard_attach_to_ai\" => {")
            .expect("Expected clipboard_attach_to_ai handler branch");
        let branch = &handler[branch_start..];

        let file_case_start = branch
            .find("clipboard_history::ContentType::File => {")
            .expect("Expected clipboard file content branch");
        let image_case_start = branch[file_case_start..]
            .find("clipboard_history::ContentType::Image => {")
            .map(|idx| file_case_start + idx)
            .expect("Expected clipboard image content branch");
        let file_case = &branch[file_case_start..image_case_start];

        assert!(
            file_case.contains("DeferredAiWindowAction::AddAttachment"),
            "clipboard file content should queue an attachment for AI chat"
        );
    }
}
