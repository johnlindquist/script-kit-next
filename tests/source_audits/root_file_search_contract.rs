//! Regression tests for the root launcher file-search boundary.
//!
//! Root search may append bounded file rows, but the unified matcher itself
//! must stay a pure in-memory ranker. Spotlight process ownership remains in
//! the file-search module, and the dedicated File Search view keeps its richer
//! directory-browser behavior.

#[cfg(test)]
mod tests {
    use std::fs;

    fn production_source(source: &str) -> &str {
        source.split("#[cfg(test)]").next().unwrap_or(source)
    }

    #[test]
    fn unified_search_module_does_not_call_file_search_processes() {
        let source = fs::read_to_string("src/scripts/search/unified.rs")
            .expect("read src/scripts/search/unified.rs");
        let production = production_source(&source);

        for forbidden in ["mdfind", "search_files(", "search_files_streaming"] {
            assert!(
                !production.contains(forbidden),
                "unified search should not call file search process APIs directly: {forbidden}"
            );
        }
    }

    #[test]
    fn dedicated_file_search_still_owns_file_search_view_navigation() {
        let list_source = fs::read_to_string("src/render_builtins/file_search_list.rs")
            .expect("read src/render_builtins/file_search_list.rs");
        let view_source = fs::read_to_string("src/render_builtins/file_search.rs")
            .expect("read src/render_builtins/file_search.rs");

        assert!(
            list_source.contains("AppView::FileSearchView"),
            "dedicated File Search view should remain a distinct browser surface"
        );
        assert!(
            view_source.contains("Double-click: browse directory inline or open file")
                && view_source.contains("Tab/Shift+Tab handled by intercept_keystrokes"),
            "dedicated File Search should keep directory browsing and parent navigation"
        );
    }

    #[test]
    fn root_streaming_search_disables_filesystem_fallback() {
        let source = fs::read_to_string("src/file_search/mdfind.rs")
            .expect("read src/file_search/mdfind.rs");
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            normalized.contains("pub fn root_search() -> Self { Self { skip_metadata: true, allow_filesystem_fallback: false"),
            "root search options should skip metadata and disable filesystem fallback"
        );
        assert!(
            normalized
                .contains("SearchFilesStreamingOptions::dedicated_file_search(skip_metadata)"),
            "existing streaming entry point should preserve dedicated File Search defaults"
        );
    }

    #[test]
    fn script_list_automation_reads_grouped_visible_rows() {
        let collect_source = fs::read_to_string("src/app_layout/collect_elements.rs")
            .expect("read src/app_layout/collect_elements.rs");
        let prompt_source = fs::read_to_string("src/prompt_handler/mod.rs")
            .expect("read src/prompt_handler/mod.rs");

        assert!(
            collect_source.contains("script_list_visible_row_labels_from_cache")
                && collect_source.contains("cached_grouped_results_snapshot()")
                && collect_source.contains("SearchResult::File"),
            "getElements should expose ScriptList grouped rows, including root file results"
        );
        assert!(
            prompt_source.contains("self.get_grouped_results_cached();")
                && prompt_source.contains("self.script_list_visible_row_labels_from_cache()"),
            "getState should refresh grouped rows before reporting ScriptList visible rows"
        );
    }

    #[test]
    fn root_file_search_receive_loop_handles_cancel_and_disconnect() {
        let source = fs::read_to_string("src/app_impl/root_file_search.rs")
            .expect("read src/app_impl/root_file_search.rs");
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            normalized.contains(
                "loop { if cancel.load(std::sync::atomic::Ordering::Relaxed) { return; }"
            ),
            "root file receive loop should keep honoring cancellation after the worker starts"
        );
        assert!(
            normalized.contains("Err(std::sync::mpsc::TryRecvError::Disconnected) => break"),
            "root file receive loop should exit if the worker channel disconnects before Done"
        );
    }

    #[test]
    fn root_file_actions_are_main_list_only() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("read src/app_impl/actions_dialog.rs");
        let simulate_key_source =
            fs::read_to_string("src/main_entry/runtime_stdin_match_simulate_key.rs")
                .expect("read src/main_entry/runtime_stdin_match_simulate_key.rs");
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");
        let simulate_key_normalized = simulate_key_source
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        assert!(
            normalized.contains(
                "ActionsDialogHost::MainList => { if let Some(file) = self.selected_root_file_result_owned()"
            ) && normalized.contains("self.toggle_root_file_actions(&file, window, cx);"),
            "MainList actions should branch to root-file actions for selected root file rows"
        );
        assert!(
            simulate_key_normalized
                .contains("if has_cmd && key_lower == \"k\" { logging::log( \"STDIN\", \"SimulateKey: Cmd+K - dispatch actions toggle\", ); view.handle_cmd_k_actions_toggle(window, ctx);"),
            "stdin simulateKey Cmd+K on ScriptList should use the shared dispatcher so root-file rows get their actions"
        );
        assert!(
            normalized.contains("ActionsDialogHost::FileSearch => { let selected = self.selected_file_search_result_owned();")
                && normalized.contains("self.toggle_file_search_actions("),
            "dedicated FileSearch actions should keep using the file-search action route"
        );
    }

    #[test]
    fn root_file_actions_do_not_expand_dedicated_file_search_browser() {
        let view_source = fs::read_to_string("src/render_builtins/file_search.rs")
            .expect("read src/render_builtins/file_search.rs");
        let list_source = fs::read_to_string("src/render_builtins/file_search_list.rs")
            .expect("read src/render_builtins/file_search_list.rs");
        let dedicated_file_search = format!("{view_source}\n{list_source}");

        for forbidden in [
            "root_file_open",
            "root_file_reveal_in_finder",
            "root_file_copy_path",
            "root_file_quick_look",
        ] {
            assert!(
                !dedicated_file_search.contains(forbidden),
                "root file action id should not be introduced into dedicated File Search render/navigation code: {forbidden}"
            );
        }
    }

    #[test]
    fn root_file_open_uses_shared_open_helper() {
        let source = fs::read_to_string("src/app_impl/selection_fallback.rs")
            .expect("read src/app_impl/selection_fallback.rs");
        let actions_dialog_source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("read src/app_impl/actions_dialog.rs");
        let actions_toggle_source = fs::read_to_string("src/app_impl/actions_toggle.rs")
            .expect("read src/app_impl/actions_toggle.rs");
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");
        let actions_dialog_normalized = actions_dialog_source
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let actions_toggle_normalized = actions_toggle_source
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        assert!(
            normalized.contains(
                "scripts::SearchResult::File(file_match) => { self.execute_root_file_open(&file_match.file, cx); }"
            ),
            "Enter on root file rows should call execute_root_file_open instead of inlining open_file"
        );
        assert!(
            normalized.contains("ROOT_FILE_OPEN_ACTION_ID => {")
                && normalized.contains("self.execute_root_file_open(file, cx);"),
            "root_file_open action should share execute_root_file_open with Enter"
        );
        assert!(
            actions_toggle_normalized
                .contains("self.pending_root_file_actions_file = Some(file.clone());"),
            "root file actions should capture the selected file when the palette opens"
        );
        assert!(
            actions_dialog_normalized.contains(
                "let root_file_context = if should_close && matches!(host, ActionsDialogHost::MainList) && crate::action_helpers::is_root_file_action_id(&action_id)"
            ) && actions_dialog_normalized.contains(
                "self.pending_root_file_actions_file .clone() .or_else(|| self.selected_root_file_result_owned())"
            ) && actions_dialog_normalized
                .contains("self.clear_actions_context_for_host(host);"),
            "root file action activation should capture context before close and clear it on MainList close"
        );
    }

    #[test]
    fn root_file_actions_prefer_captured_file_over_live_selection() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("read src/app_impl/actions_dialog.rs");
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            normalized.contains(
                "if crate::action_helpers::is_root_file_action_id(&action_id) { if let Some(file) = self .pending_root_file_actions_file .clone() .or_else(|| self.selected_root_file_result_owned())"
            ),
            "root file action execution should prefer the captured file over the current live selection"
        );
    }

    #[test]
    fn root_file_actions_context_cleared_by_detached_on_close() {
        let source = fs::read_to_string("src/app_impl/actions_toggle.rs")
            .expect("read src/app_impl/actions_toggle.rs");
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            normalized.contains(
                "app.mark_actions_popup_closed(); app.clear_actions_context_for_host(host); app.mark_filter_resync_after_actions_if_needed();"
            ),
            "detached actions-window on_close should clear any captured MainList root-file context"
        );
    }

    #[test]
    fn root_file_action_ids_are_reserved() {
        let source =
            fs::read_to_string("src/action_helpers.rs").expect("read src/action_helpers.rs");

        for action_id in [
            "root_file_open",
            "root_file_reveal_in_finder",
            "root_file_copy_path",
            "root_file_quick_look",
        ] {
            assert!(
                source.contains(action_id),
                "root file action id should be reserved: {action_id}"
            );
        }
    }
}
