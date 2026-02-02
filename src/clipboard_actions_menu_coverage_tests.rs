//! Regression tests for clipboard actions menu handler coverage.
//!
//! Ensures every clipboard action ID exposed in the actions menu has a handler
//! in `ScriptListApp::handle_action`.

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;

    fn clipboard_builder_ids() -> BTreeSet<String> {
        let builders =
            fs::read_to_string("src/actions/builders.rs").expect("Failed to read builders.rs");
        let start = builders
            .find("pub fn get_clipboard_history_context_actions")
            .expect("clipboard actions builder not found");
        let end = builders[start..]
            .find("pub struct ChatPromptInfo")
            .map(|i| start + i)
            .expect("clipboard builder end marker not found");
        let region = &builders[start..end];

        let mut ids = BTreeSet::new();
        let re = regex::Regex::new(r#"Action::new\(\s*"(clipboard_[a-z0-9_]+)""#)
            .expect("regex compile failed");
        for cap in re.captures_iter(region) {
            ids.insert(cap[1].to_string());
        }
        ids
    }

    fn app_action_handler_region() -> String {
        let app_actions =
            fs::read_to_string("src/app_actions.rs").expect("Failed to read app_actions.rs");
        let start = app_actions
            .find("fn handle_action(")
            .expect("handle_action not found");
        app_actions[start..].to_string()
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
