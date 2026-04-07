//! Regression tests for actions button visibility in the main script list.
//!
//! These source-audit tests ensure:
//! 1. The native footer bridge routes through canonical ScriptListApp dispatch.
//! 2. The full ScriptList footer uses the universal three-key hint strip.
//! 3. The Actions button click is gated by `has_actions()`.

#[cfg(test)]
mod tests {
    use std::fs;

    // -----------------------------------------------------------------------
    // Helper: read all .rs files from a directory into one string
    // -----------------------------------------------------------------------
    fn read_all_rs_in_dir(dir: &str) -> String {
        let mut content = String::new();
        for entry in fs::read_dir(dir).unwrap_or_else(|_| panic!("Failed to read dir {}", dir)) {
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
        content
    }

    // =======================================================================
    // Existing tests (preserved)
    // =======================================================================

    #[test]
    fn test_has_actions_helper_exists_and_handles_clipboard() {
        let content = read_all_rs_in_dir("src/app_actions/handle_action");

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
    fn test_footer_uses_universal_hint_strip() {
        let content = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("Failed to read src/render_script_list/mod.rs");

        assert!(
            content.contains("render_universal_prompt_hint_strip_clickable"),
            "render_script_list footer must use the universal three-key hint strip"
        );
        assert!(
            content.contains("emit_prompt_hint_audit(\"render_script_list::full\""),
            "render_script_list footer must emit a prompt hint audit"
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
            render_impl.contains("self.sync_main_footer_popup(window, cx);"),
            "render loop should sync the popup footer when mini mode visibility changes"
        );
        assert!(
            footer_popup.contains("FOOTER_HINTS: [(FooterAction, &str, &str); 3]")
                && footer_popup.contains("FooterAction::Run")
                && footer_popup.contains("FooterAction::Actions")
                && footer_popup.contains("FooterAction::Ai")
                && footer_popup.contains("(FooterAction::Run, \"↵\", \"Run\")")
                && footer_popup.contains("(FooterAction::Actions, \"⌘K\", \"Actions\")")
                && footer_popup.contains("(FooterAction::Ai, \"Tab\", \"AI\")"),
            "popup footer should match the universal launcher affordance contract"
        );
    }

    // =======================================================================
    // NEW: Native footer bridge dispatch regression tests
    // =======================================================================

    /// Fails if the native footer bridge listener is removed from ui_window.rs
    /// or stops draining the canonical `footer_action_channel()`.
    #[test]
    fn test_native_footer_bridge_exists_and_uses_canonical_channel() {
        let content = fs::read_to_string("src/app_impl/ui_window.rs")
            .expect("Failed to read src/app_impl/ui_window.rs");

        // The bridge must exist as a method on ScriptListApp
        assert!(
            content.contains("fn ensure_main_footer_action_listener"),
            "Native footer bridge listener must exist in ui_window.rs"
        );

        // It must drain the canonical footer_action_channel (not a duplicate channel)
        assert!(
            content.contains("footer_popup::footer_action_channel()"),
            "Native footer bridge must use the canonical footer_action_channel()"
        );

        // It must delegate to handle_main_footer_action (not inline dispatch)
        assert!(
            content.contains("handle_main_footer_action(action"),
            "Native footer bridge must route through handle_main_footer_action()"
        );
    }

    /// Fails if handle_main_footer_action stops dispatching all three actions
    /// through the canonical ScriptListApp methods.
    #[test]
    fn test_native_footer_dispatches_all_three_canonical_actions() {
        let content = fs::read_to_string("src/app_impl/ui_window.rs")
            .expect("Failed to read src/app_impl/ui_window.rs");

        // Locate handle_main_footer_action
        let handler_pos = content
            .find("fn handle_main_footer_action")
            .expect("handle_main_footer_action must exist in ui_window.rs");
        let handler_section = &content[handler_pos..content.len().min(handler_pos + 3000)];

        // Run dispatches to execute_selected
        assert!(
            handler_section.contains("FooterAction::Run")
                && handler_section.contains("execute_selected"),
            "FooterAction::Run must dispatch to execute_selected()"
        );

        // Actions dispatches through the shared actions toggle dispatcher
        assert!(
            handler_section.contains("FooterAction::Actions")
                && handler_section.contains("dispatch_actions_toggle_for_current_view"),
            "FooterAction::Actions must dispatch through dispatch_actions_toggle_for_current_view()"
        );

        // Ai dispatches to open_tab_ai_chat
        assert!(
            handler_section.contains("FooterAction::Ai")
                && handler_section.contains("open_tab_ai_chat"),
            "FooterAction::Ai must dispatch to open_tab_ai_chat()"
        );
    }

    /// Fails if the native footer Actions dispatch stops using the shared toggle
    /// dispatcher. The shared dispatcher (`dispatch_actions_toggle_for_current_view`)
    /// handles the has_actions() gating internally, so the native footer must
    /// delegate to it rather than duplicating its own gate.
    #[test]
    fn test_native_footer_actions_gated_by_has_actions() {
        let content = fs::read_to_string("src/app_impl/ui_window.rs")
            .expect("Failed to read src/app_impl/ui_window.rs");

        let handler_pos = content
            .find("fn handle_main_footer_action")
            .expect("handle_main_footer_action must exist in ui_window.rs");
        let handler_section = &content[handler_pos..content.len().min(handler_pos + 3000)];

        // The Actions arm must use the shared dispatcher which gates on has_actions() internally
        let actions_pos = handler_section
            .find("FooterAction::Actions")
            .expect("FooterAction::Actions arm must exist");
        let after_actions =
            &handler_section[actions_pos..handler_section.len().min(actions_pos + 600)];

        assert!(
            after_actions.contains("dispatch_actions_toggle_for_current_view"),
            "Native footer Actions dispatch must use the shared dispatcher (dispatch_actions_toggle_for_current_view). Found:\n{}",
            after_actions
        );
    }

    /// Fails if the native footer bridge stops guarding against non-mini or
    /// non-ScriptList views. The native AppKit footer only applies when the
    /// mini main window is showing the script list.
    #[test]
    fn test_native_footer_guards_mini_scriptlist_surface() {
        let content = fs::read_to_string("src/app_impl/ui_window.rs")
            .expect("Failed to read src/app_impl/ui_window.rs");

        let handler_pos = content
            .find("fn handle_main_footer_action")
            .expect("handle_main_footer_action must exist");
        let handler_section = &content[handler_pos..content.len().min(handler_pos + 1500)];

        assert!(
            handler_section.contains("AppView::ScriptList"),
            "Native footer handler must check for AppView::ScriptList"
        );
        assert!(
            handler_section.contains("MainWindowMode::Mini"),
            "Native footer handler must check for MainWindowMode::Mini"
        );
    }

    /// Fails if sync_main_footer_popup stops calling ensure_main_footer_action_listener.
    /// Without this, the async bridge is never started and native clicks are silently dropped.
    #[test]
    fn test_sync_footer_popup_starts_listener() {
        let content = fs::read_to_string("src/app_impl/ui_window.rs")
            .expect("Failed to read src/app_impl/ui_window.rs");

        let sync_pos = content
            .find("fn sync_main_footer_popup")
            .expect("sync_main_footer_popup must exist in ui_window.rs");
        let sync_section = &content[sync_pos..content.len().min(sync_pos + 400)];

        assert!(
            sync_section.contains("ensure_main_footer_action_listener"),
            "sync_main_footer_popup must call ensure_main_footer_action_listener to start the bridge"
        );
    }

    /// Fails if the full ScriptList footer clickable strip stops dispatching
    /// all three actions through the canonical ScriptListApp methods.
    /// Complements test_footer_uses_universal_hint_strip by verifying the
    /// actual callback wiring, not just the function name.
    #[test]
    fn test_full_footer_clickable_strip_dispatches_canonical_actions() {
        let content = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("Failed to read src/render_script_list/mod.rs");

        // Find the clickable strip section
        let strip_pos = content
            .find("render_universal_prompt_hint_strip_clickable")
            .expect("Clickable hint strip must exist in render_script_list");
        let strip_section = &content[strip_pos..content.len().min(strip_pos + 2000)];

        assert!(
            strip_section.contains("execute_selected"),
            "Full footer Run callback must dispatch to execute_selected()"
        );
        assert!(
            strip_section.contains("toggle_actions"),
            "Full footer Actions callback must dispatch to toggle_actions()"
        );
        assert!(
            strip_section.contains("open_tab_ai_chat"),
            "Full footer AI callback must dispatch to open_tab_ai_chat()"
        );
    }

    /// Fails if the full ScriptList footer Actions callback stops gating on
    /// has_actions(). Both the native mini footer and the GPUI full footer
    /// must share this contract.
    #[test]
    fn test_full_footer_actions_callback_gated_by_has_actions() {
        let content = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("Failed to read src/render_script_list/mod.rs");

        let strip_pos = content
            .find("render_universal_prompt_hint_strip_clickable")
            .expect("Clickable hint strip must exist in render_script_list");
        let strip_section = &content[strip_pos..content.len().min(strip_pos + 2000)];

        assert!(
            strip_section.contains("has_actions()"),
            "Full footer Actions callback must gate on has_actions(). Found:\n{}",
            strip_section
        );
    }
}
