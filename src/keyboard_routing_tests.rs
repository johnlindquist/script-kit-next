//! Regression tests for keyboard routing in various app views.
//!
//! These tests ensure that keyboard events are routed correctly based on:
//! - Current view (ScriptList, FileSearchView, etc.)
//! - Whether actions popup is open
//! - Modifier key state
//!
//! ## Background
//! A bug was discovered where arrow keys in ScriptList view were navigating
//! the script list instead of the actions dialog when the popup was open.
//! This was fixed by adding a `show_actions_popup` check before handling
//! arrow keys in the ScriptList case.
//!
//! ## Code Audit Tests
//! These tests verify that the keyboard routing patterns are consistent
//! across all views that support actions popups.

#[cfg(test)]
mod tests {
    use std::fs;

    fn read_app_impl_sources() -> String {
        let files = [
            "src/app_impl/startup_new_prelude.rs",
            "src/app_impl/startup_new_tab.rs",
            "src/app_impl/startup_new_arrow.rs",
            "src/app_impl/startup_new_actions.rs",
            "src/app_impl/actions_dialog.rs",
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

    /// Extract the arrow interceptor section from startup interceptor sources.
    /// This is the section between "let arrow_interceptor" and the next major interceptor.
    fn get_arrow_interceptor_section(content: &str) -> &str {
        let start = content
            .find("let arrow_interceptor = cx.intercept_keystrokes")
            .expect("Arrow interceptor not found");

        // Find the end - look for the next interceptor or a reasonable boundary
        let after_start = &content[start..];
        let end = after_start
            .find("let actions_interceptor")
            .unwrap_or(after_start.len().min(5000));

        &content[start..start + end]
    }

    fn assert_main_window_visibility_guard_precedes_key_parse(section: &str, section_name: &str) {
        let guard = "if !script_kit_gpui::is_main_window_visible()";
        let guard_pos = section.find(guard).unwrap_or_else(|| {
            panic!("{section_name} must skip handling when main window is hidden")
        });
        let key_parse_pos = section
            .find("let key = event.keystroke.key.as_str()")
            .unwrap_or_else(|| panic!("{section_name} key parsing not found"));

        assert!(
            guard_pos < key_parse_pos,
            "{section_name} visibility guard must run before key parsing"
        );
    }

    /// Verify that ScriptList arrow key handling checks for actions popup.
    ///
    /// This is a code audit test that ensures the fix for keyboard routing
    /// to actions dialog is present and won't regress.
    #[test]
    fn test_scriptlist_arrow_handler_checks_actions_popup() {
        let content = read_app_impl_sources();

        let arrow_section = get_arrow_interceptor_section(&content);

        // Find the ScriptList case within the arrow interceptor
        let scriptlist_pos = arrow_section
            .find("AppView::ScriptList =>")
            .expect("ScriptList case not found in arrow interceptor");

        // Get the section after ScriptList match (next ~600 chars should have the check)
        let after_scriptlist = &arrow_section[scriptlist_pos..];
        let check_end = after_scriptlist.len().min(600);
        let scriptlist_handler = &after_scriptlist[..check_end];

        // Verify the actions popup check exists early in the handler
        assert!(
            scriptlist_handler.contains("show_actions_popup"),
            "ScriptList arrow key handler must check show_actions_popup before navigation. \
             Found section:\n{}\n\n\
             This check was added to fix keyboard routing to the actions dialog.",
            scriptlist_handler
        );

        // Verify it routes to actions dialog when popup is open
        assert!(
            scriptlist_handler.contains("actions_dialog"),
            "ScriptList must route arrow keys to actions_dialog when popup is open"
        );
    }

    /// Verify that FileSearchView arrow key handling checks for actions popup.
    ///
    /// This serves as the reference implementation for how actions popup
    /// keyboard routing should work.
    #[test]
    fn test_filesearchview_arrow_handler_checks_actions_popup() {
        let content = read_app_impl_sources();

        let arrow_section = get_arrow_interceptor_section(&content);

        // Find the FileSearchView case within the arrow interceptor
        let filesearch_pos = arrow_section
            .find("AppView::FileSearchView {")
            .expect("FileSearchView case not found in arrow interceptor");

        // Get the section after FileSearchView match
        let after_filesearch = &arrow_section[filesearch_pos..];
        let check_end = after_filesearch.len().min(600);
        let filesearch_handler = &after_filesearch[..check_end];

        // Verify the actions popup check exists
        assert!(
            filesearch_handler.contains("show_actions_popup"),
            "FileSearchView arrow key handler must check show_actions_popup. \
             Found section:\n{}",
            filesearch_handler
        );
    }

    /// Verify global arrow interception skips secondary windows.
    ///
    /// intercept_keystrokes hooks are app-wide, so this guard prevents the
    /// main list from consuming notes/AI window arrow navigation.
    #[test]
    fn test_arrow_interceptor_skips_notes_and_ai_windows() {
        let content = read_app_impl_sources();
        let arrow_section = get_arrow_interceptor_section(&content);

        let guard = "if crate::notes::is_notes_window(window) || crate::ai::is_ai_window(window)";
        let guard_pos = arrow_section
            .find(guard)
            .expect("arrow interceptor must skip notes/AI windows");
        let key_parse_pos = arrow_section
            .find("let key = event.keystroke.key.as_str()")
            .expect("arrow interceptor key parsing not found");

        assert!(
            guard_pos < key_parse_pos,
            "Notes/AI window guard should run before key routing in arrow interceptor"
        );
    }

    #[test]
    fn test_arrow_interceptor_skips_when_main_window_hidden() {
        let content =
            fs::read_to_string("src/app_impl/startup_new_arrow.rs").expect("Failed to read");
        let section = get_arrow_interceptor_section(&content);
        assert_main_window_visibility_guard_precedes_key_parse(section, "arrow interceptor");
    }

    #[test]
    fn test_home_end_interceptor_skips_when_main_window_hidden() {
        let content =
            fs::read_to_string("src/app_impl/startup_new_arrow.rs").expect("Failed to read");
        let start = content
            .find("let home_end_interceptor = cx.intercept_keystrokes")
            .expect("home_end interceptor not found");
        let after_start = &content[start..];
        let end = after_start
            .find("app.gpui_input_subscriptions.push(home_end_interceptor);")
            .expect("home_end interceptor end not found");
        let section = &content[start..start + end];

        assert_main_window_visibility_guard_precedes_key_parse(section, "home_end interceptor");
    }

    #[test]
    fn test_actions_interceptor_skips_when_main_window_hidden() {
        let content =
            fs::read_to_string("src/app_impl/startup_new_actions.rs").expect("Failed to read");
        let start = content
            .find("let actions_interceptor = cx.intercept_keystrokes")
            .expect("actions interceptor not found");
        let after_start = &content[start..];
        let end = after_start
            .find("app.gpui_input_subscriptions.push(actions_interceptor);")
            .expect("actions interceptor end not found");
        let section = &content[start..start + end];

        assert_main_window_visibility_guard_precedes_key_parse(section, "actions interceptor");
    }

    #[test]
    fn test_tab_interceptor_skips_when_main_window_hidden() {
        let content =
            fs::read_to_string("src/app_impl/startup_new_tab.rs").expect("Failed to read");
        let start = content
            .find("let tab_interceptor = cx.intercept_keystrokes")
            .expect("tab interceptor not found");
        let after_start = &content[start..];
        let end = after_start
            .find("app.gpui_input_subscriptions.push(tab_interceptor);")
            .expect("tab interceptor end not found");
        let section = &content[start..start + end];

        assert_main_window_visibility_guard_precedes_key_parse(section, "tab interceptor");
    }

    /// Verify that render_script_list.rs handles keyboard events for actions popup.
    ///
    /// The render handler should route Enter, Escape, Backspace, and character
    /// input to the actions dialog when the popup is open.
    #[test]
    fn test_render_script_list_handles_actions_keyboard() {
        let content = fs::read_to_string("src/render_script_list/mod.rs")
            .or_else(|_| fs::read_to_string("src/render_script_list.rs"))
            .expect("Failed to read render_script_list source");

        // Verify actions popup keyboard handling exists
        assert!(
            content.contains("show_actions_popup"),
            "render_script_list.rs must check show_actions_popup for keyboard routing"
        );

        // Verify Enter key routes to actions dialog
        assert!(
            content.contains("enter") && content.contains("get_selected_action_id"),
            "Enter key should execute the selected action from actions dialog"
        );

        // Verify Escape closes actions popup
        assert!(
            content.contains("escape")
                && (content.contains("close_actions_popup")
                    || content.contains("close_actions_window")),
            "Escape should close the actions popup"
        );
    }

    /// Verify that actions dialog move_up/move_down methods exist.
    ///
    /// These are called by the keyboard interceptors when routing to the dialog.
    #[test]
    fn test_actions_dialog_has_navigation_methods() {
        let content =
            fs::read_to_string("src/actions/dialog.rs").expect("Failed to read actions/dialog.rs");

        assert!(
            content.contains("fn move_up"),
            "ActionsDialog must have move_up method for keyboard navigation"
        );

        assert!(
            content.contains("fn move_down"),
            "ActionsDialog must have move_down method for keyboard navigation"
        );
    }

    /// Verify that notify_actions_window is called after navigation.
    ///
    /// This ensures the separate actions window re-renders after state changes.
    #[test]
    fn test_actions_navigation_triggers_window_notify() {
        let content = read_app_impl_sources();

        // When we route arrow keys to actions dialog, we should notify the window
        let notify_count = content.matches("notify_actions_window").count();

        // Should be called at least twice (once for ScriptList, once for FileSearchView)
        assert!(
            notify_count >= 2,
            "notify_actions_window should be called after routing arrow keys to actions dialog. \
             Found {} occurrences, expected at least 2.",
            notify_count
        );
    }

    /// Verify main input focus closes actions popup through shared close path.
    ///
    /// This prevents focus overlay state from desynchronizing.
    #[test]
    fn test_main_input_focus_uses_shared_actions_close_path() {
        let content = read_app_impl_sources();

        assert!(
            content.contains(
                "Main input focused while actions open - closing actions via shared close path"
            ),
            "Main input focus should log shared close path usage"
        );
        assert!(
            content.contains("this.close_actions_popup(ActionsDialogHost::MainList, window, cx);"),
            "Main input focus should call close_actions_popup to keep focus overlays in sync"
        );
    }

    /// Verify actions dialog routing handles jump keys.
    ///
    /// Home/End/PageUp/PageDown parity is required for power-user keyboard navigation.
    #[test]
    fn test_actions_dialog_route_handles_jump_keys() {
        let content = read_app_impl_sources();

        assert!(
            content.contains("let is_home = key.eq_ignore_ascii_case(\"home\")"),
            "route_key_to_actions_dialog should recognize Home key"
        );
        assert!(
            content.contains("let is_end = key.eq_ignore_ascii_case(\"end\")"),
            "route_key_to_actions_dialog should recognize End key"
        );
        assert!(
            content.contains("let is_page_up = key.eq_ignore_ascii_case(\"pageup\")"),
            "route_key_to_actions_dialog should recognize PageUp key"
        );
        assert!(
            content.contains("let is_page_down = key.eq_ignore_ascii_case(\"pagedown\")"),
            "route_key_to_actions_dialog should recognize PageDown key"
        );
    }

    /// Verify all views with actions support have consistent keyboard routing.
    ///
    /// This is a comprehensive check that all views handle the actions popup
    /// keyboard routing pattern consistently.
    #[test]
    fn test_all_views_have_consistent_actions_keyboard_routing() {
        let content = read_app_impl_sources();

        let arrow_section = get_arrow_interceptor_section(&content);

        // Views that support actions popup should all check show_actions_popup
        // in their arrow key handlers within the arrow interceptor
        let views_with_actions = ["AppView::ScriptList", "AppView::FileSearchView"];

        for view in &views_with_actions {
            // Find the view case within the arrow interceptor
            let view_pos = arrow_section.find(view);
            assert!(
                view_pos.is_some(),
                "View {} not found in arrow interceptor",
                view
            );

            // After the view match, should have show_actions_popup check within 600 chars
            let after_view = &arrow_section[view_pos.unwrap()..];
            let section_end = after_view.len().min(600);
            let view_section = &after_view[..section_end];

            // Count occurrences of the pattern - should have the check
            let has_popup_check = view_section.contains("show_actions_popup");

            assert!(
                has_popup_check,
                "{} must check show_actions_popup in arrow key handler. \
                 This ensures arrow keys route to actions dialog when popup is open.\n\
                 Section:\n{}",
                view, view_section
            );
        }
    }

    /// Verify stop_propagation is called when routing to actions dialog.
    ///
    /// This prevents the key event from being handled by other components.
    #[test]
    fn test_actions_routing_stops_propagation() {
        let content = read_app_impl_sources();

        // After routing to actions dialog, we should stop propagation
        // Look for the pattern: show_actions_popup check followed by stop_propagation
        assert!(
            content.contains("stop_propagation"),
            "Arrow key handlers must call stop_propagation after routing to actions dialog"
        );
    }

    /// The startup interceptor must delegate to the canonical actions_support_for_view()
    /// resolver instead of maintaining its own match on AppView.
    #[test]
    fn startup_interceptor_uses_canonical_actions_support_resolver() {
        let content = read_app_impl_sources();
        assert!(
            content.contains("actions_support_for_view()"),
            "startup actions routing should delegate to the canonical actions support resolver"
        );
    }

    /// Focus restore host mappings must cover the same surfaces the canonical
    /// resolver maps to SharedDialog.
    #[test]
    fn focus_restore_hosts_match_shared_actions_surfaces() {
        let content = read_app_impl_sources();
        for host in [
            "ActionsDialogHost::MainList",
            "ActionsDialogHost::ClipboardHistory",
            "ActionsDialogHost::EmojiPicker",
            "ActionsDialogHost::FileSearch",
        ] {
            assert!(
                content.contains(host),
                "shared actions focus restore should still cover {host}"
            );
        }
    }

    /// All global keystroke interceptors must guard against the actions popup window.
    ///
    /// The actions window (ActionsWindow) handles its own Escape, Enter, arrows, etc.
    /// If the main window's interceptors also process those keystrokes, they pass the
    /// WRONG window reference to focus restoration code, breaking Escape-to-close.
    ///
    /// This test ensures every interceptor file that checks for Notes/AI windows
    /// also checks for the actions window, preventing the class of bugs where
    /// interceptors steal keystrokes from the actions popup.
    #[test]
    fn test_all_interceptors_guard_against_actions_window_keystrokes() {
        let interceptor_files = [
            "src/app_impl/startup.rs",
            "src/app_impl/startup_new_actions.rs",
            "src/app_impl/startup_new_arrow.rs",
            "src/app_impl/startup_new_tab.rs",
        ];

        for file_path in &interceptor_files {
            let source = fs::read_to_string(file_path)
                .unwrap_or_else(|_| panic!("Failed to read {}", file_path));

            // Count how many interceptor guards check for notes/ai windows
            let notes_guard_count = source.matches("is_notes_window(window)").count();
            let actions_guard_count = source.matches("is_actions_window(window)").count();

            // Every interceptor that guards against notes windows must also guard
            // against the actions window. The actions window manages its own keys.
            assert!(
                actions_guard_count >= notes_guard_count,
                "{}: has {} notes window guard(s) but only {} actions window guard(s). \
                 Every intercept_keystrokes guard that checks is_notes_window must also \
                 check is_actions_window to prevent stealing keystrokes from the actions popup.",
                file_path,
                notes_guard_count,
                actions_guard_count
            );
        }
    }
}
