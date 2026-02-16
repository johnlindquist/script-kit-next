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
    fn file_attach_to_ai_hides_main_before_opening_ai_window() {
        let handler = app_action_handler_region();
        let branch_start = handler
            .find("\"attach_to_ai\" => {")
            .expect("Expected attach_to_ai handler branch");
        let branch = &handler[branch_start..];

        let hide_index = branch
            .find("self.hide_main_and_reset(cx);")
            .expect("Expected attach_to_ai to hide main window first");
        let spawn_index = branch
            .find("cx.spawn(async move |this, cx| {")
            .expect("Expected attach_to_ai to defer AI open via spawn");
        let open_index = branch
            .find("ai::open_ai_window(cx)")
            .expect("Expected attach_to_ai to open AI window");

        assert!(
            hide_index < spawn_index && spawn_index < open_index,
            "attach_to_ai should hide main, then defer open_ai_window via cx.spawn"
        );
    }
}
