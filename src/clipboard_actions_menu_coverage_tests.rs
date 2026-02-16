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

    fn section_between<'a>(content: &'a str, start: &str, end: &str) -> &'a str {
        let start_index = content
            .find(start)
            .unwrap_or_else(|| panic!("Missing start marker: {start}"));
        let tail = &content[start_index..];
        let end_offset = tail
            .find(end)
            .unwrap_or_else(|| panic!("Missing end marker: {end}"));
        &tail[..end_offset]
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
    fn clipboard_attach_to_ai_defers_open_until_after_main_hide() {
        let handler = app_action_handler_region();
        let section = section_between(
            &handler,
            "\"clipboard_attach_to_ai\" => {",
            "// Copy to clipboard without pasting (Cmd+Enter)",
        );

        assert!(
            section.contains(
                "self.open_ai_window_after_main_hide(deferred_action, \"Attached to AI\", cx);"
            ),
            "clipboard_attach_to_ai should defer AI open until after main hide"
        );
        assert!(
            !section.contains("ai::open_ai_window(cx)"),
            "clipboard_attach_to_ai should not open AI synchronously in the action branch"
        );
        assert!(
            section.contains("DeferredAiWindowAction::AddAttachment"),
            "clipboard file attachments should queue as AI attachments"
        );
    }

    #[test]
    fn file_attach_to_ai_defers_open_until_after_main_hide() {
        let handler = app_action_handler_region();
        let section = section_between(
            &handler,
            "| \"attach_to_ai\" => {",
            "\"copy_filename\" => {",
        );

        assert!(
            section.contains("if action_id == \"attach_to_ai\" {"),
            "file attach action should have a dedicated deferred-open branch"
        );
        assert!(
            section.contains("self.open_ai_window_after_main_hide("),
            "file attach action should defer AI window open until after main hide"
        );
    }
}
