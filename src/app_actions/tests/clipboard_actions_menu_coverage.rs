// Regression tests for clipboard actions menu handler coverage.
//
// Ensures every clipboard action ID exposed in the actions menu has a handler
// in `ScriptListApp::handle_action`.

#[cfg(test)]
mod tests {
    use crate::test_utils::read_source;
    use std::collections::BTreeSet;

    fn clipboard_builder_ids() -> BTreeSet<String> {
        let region = read_source("src/actions/builders/clipboard.rs");

        let mut ids = BTreeSet::new();
        let re = regex::Regex::new(r#"Action::new\(\s*"(clipboard_[a-z0-9_]+)""#)
            .expect("regex compile failed");
        for cap in re.captures_iter(&region) {
            ids.insert(cap[1].to_string());
        }
        ids
    }

    fn app_action_handler_region() -> String {
        // All clipboard action handlers live in the modular handle_action/ directory.
        crate::test_utils::read_all_handle_action_sources()
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
}
