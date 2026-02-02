//! Regression tests for actions button visibility in the main script list.

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn test_has_actions_helper_exists_and_handles_clipboard() {
        let content =
            fs::read_to_string("src/app_actions.rs").expect("Failed to read src/app_actions.rs");

        assert!(
            content.contains("fn has_actions"),
            "Expected ScriptListApp::has_actions() helper to exist"
        );

        assert!(
            content.contains("ClipboardHistoryView"),
            "has_actions() should account for ClipboardHistoryView selection"
        );

        assert!(
            content.contains("selected_clipboard_entry"),
            "has_actions() should use selected_clipboard_entry() for clipboard history"
        );
    }

    #[test]
    fn test_footer_secondary_visibility_uses_has_actions() {
        let content = fs::read_to_string("src/render_script_list.rs")
            .expect("Failed to read src/render_script_list.rs");

        assert!(
            content.contains("show_secondary(self.has_actions())"),
            "render_script_list footer must use show_secondary(self.has_actions())"
        );
    }

    #[test]
    fn test_cmd_k_requires_actions() {
        let content = fs::read_to_string("src/render_script_list.rs")
            .expect("Failed to read src/render_script_list.rs");

        let cmdk_pos = content
            .find("Cmd+K")
            .expect("Cmd+K handler not found in render_script_list.rs");

        let after_cmdk = &content[cmdk_pos..content.len().min(cmdk_pos + 240)];

        assert!(
            after_cmdk.contains("has_actions"),
            "Cmd+K handling should be gated by has_actions(). Found section:\n{}",
            after_cmdk
        );
    }
}
