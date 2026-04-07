//! Regression tests for actions button visibility in the main script list.

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn test_has_actions_helper_exists_and_handles_clipboard() {
        let dir = "src/app_actions/handle_action";
        let mut content = String::new();
        for entry in fs::read_dir(dir).expect("Failed to read handle_action directory") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs") {
                content.push_str(
                    &fs::read_to_string(&path)
                        .unwrap_or_else(|_| panic!("Failed to read {}", path.display())),
                );
                content.push('\n');
            }
        }

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
        let content = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("Failed to read src/render_script_list/mod.rs");

        assert!(
            content.contains("show_secondary(self.has_actions())"),
            "render_script_list footer must use show_secondary(self.has_actions())"
        );
    }

    #[test]
    fn test_cmd_k_requires_actions() {
        let content = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("Failed to read src/render_script_list/mod.rs");

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
        let content = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("Failed to read src/render_script_list/mod.rs");

        let ask_ai_pos = content
            .find(".id(\"ask-ai-button\")")
            .expect("Ask AI hint container not found in src/render_script_list/mod.rs");
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

    #[test]
    fn test_mini_mode_branch_hides_ask_ai_and_skips_preview_footer() {
        let content = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("Failed to read src/render_script_list/mod.rs");
        let render_impl = fs::read_to_string("src/main_sections/render_impl.rs")
            .expect("Failed to read src/main_sections/render_impl.rs");
        let footer_popup =
            fs::read_to_string("src/footer_popup.rs").expect("Failed to read src/footer_popup.rs");

        assert!(
            content.contains("let is_mini = self.main_window_mode == MainWindowMode::Mini;"),
            "mini mode flag should be computed from main_window_mode"
        );
        assert!(
            content.contains(".when(!is_mini, |d| {"),
            "Ask AI header hint should be hidden in mini mode"
        );
        assert!(
            content.contains("if is_mini {")
                && content
                    .contains("// Mini mode: single column, toggle between list and info panel")
                && content.contains("mode = \"mini\""),
            "mini mode branch should render the single-column layout and mini perf log"
        );
        assert!(
            render_impl.contains("self.sync_main_footer_popup(cx);"),
            "render loop should sync the popup footer when mini mode visibility changes"
        );
        assert!(
            footer_popup.contains("render_hint_icons(")
                && footer_popup.contains("\"↵ Run\"")
                && footer_popup.contains("\"⌘K Actions\"")
                && footer_popup.contains("\"Tab AI\""),
            "popup footer should render the three launcher affordance hints"
        );
    }
}
