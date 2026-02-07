//! Regression tests for actions button visibility in the main script list.

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn test_has_actions_helper_exists_and_handles_clipboard() {
        let content = fs::read_to_string("src/app_actions/handle_action.rs")
            .expect("Failed to read src/app_actions/handle_action.rs");

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
        let content = fs::read_to_string("src/render_script_list/part_000.rs")
            .expect("Failed to read src/render_script_list/part_000.rs");

        assert!(
            content.contains("show_secondary(self.has_actions())"),
            "render_script_list footer must use show_secondary(self.has_actions())"
        );
    }

    #[test]
    fn test_cmd_k_requires_actions() {
        let content = fs::read_to_string("src/render_script_list/part_000.rs")
            .expect("Failed to read src/render_script_list/part_000.rs");

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

    #[test]
    fn test_ask_ai_hint_is_non_clickable_visual_hint() {
        let content = fs::read_to_string("src/render_script_list/part_000.rs")
            .expect("Failed to read src/render_script_list/part_000.rs");

        let ask_ai_pos = content
            .find(".id(\"ask-ai-button\")")
            .expect("Ask AI hint container not found in src/render_script_list/part_000.rs");
        let ask_ai_section = &content[ask_ai_pos..content.len().min(ask_ai_pos + 1200)];

        assert!(
            ask_ai_section.contains(".cursor_default()"),
            "Ask AI hint should be non-clickable (cursor_default). Section:\n{}",
            ask_ai_section
        );
        assert!(
            !ask_ai_section.contains(".cursor_pointer()"),
            "Ask AI hint should not imply clickability with cursor_pointer. Section:\n{}",
            ask_ai_section
        );
    }
}
