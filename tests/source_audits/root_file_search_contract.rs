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
    fn root_file_ranking_stays_local_and_does_not_start_searches() {
        let source =
            fs::read_to_string("src/file_search/mod.rs").expect("read src/file_search/mod.rs");
        let rank_source = source
            .split("pub fn rank_root_file_results(")
            .nth(1)
            .and_then(|section| section.split("/// Payload for file drag-out").next())
            .expect("rank_root_file_results source should be present");

        for forbidden in [
            "mdfind",
            "search_files(",
            "search_files_streaming",
            "std::process::Command",
            "std::fs::read_dir",
            "list_directory",
        ] {
            assert!(
                !rank_source.contains(forbidden),
                "root ranking should only rank already-returned rows, not start searches: {forbidden}"
            );
        }
        assert!(
            rank_source.contains("file.name") && rank_source.contains("file.path"),
            "root ranking should continue scoring the existing FileResult name/path fields"
        );
    }

    #[test]
    fn root_file_handoff_row_uses_existing_search_files_fallback() {
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let types_source =
            fs::read_to_string("src/scripts/types.rs").expect("read src/scripts/types.rs");
        let builtins_source = fs::read_to_string("src/fallbacks/builtins.rs")
            .expect("read src/fallbacks/builtins.rs");

        assert!(
            builtins_source.contains("pub const SEARCH_FILES_FALLBACK_ID: &str = \"search-files\""),
            "Search Files fallback id should be exported instead of repeated as an inline literal"
        );
        assert!(
            grouping_source.contains("fn root_file_search_handoff_result(")
                && grouping_source.contains("SEARCH_FILES_FALLBACK_ID")
                && grouping_source.contains("Search Files for \\\"{query}\\\"")
                && grouping_source.contains("Open full File Search")
                && grouping_source.contains("SearchResult::Fallback("),
            "root file grouping should append a synthetic fallback row that opens the dedicated File Search view"
        );
        assert!(
            types_source.contains("title_override: Option<String>")
                && types_source.contains("description_override: Option<String>")
                && types_source.contains("with_display_overrides(")
                && types_source.contains("pub fn display_label(&self) -> String")
                && types_source.contains("pub fn display_description(&self) -> String"),
            "fallback matches should support dynamic display text without leaking static strings"
        );
        assert!(
            fs::read_to_string("src/app_layout/collect_elements.rs")
                .expect("read src/app_layout/collect_elements.rs")
                .contains("scripts::SearchResult::Fallback(m) => m.display_label()"),
            "automation element labels should expose the handoff row's dynamic title"
        );
    }

    #[test]
    fn root_file_handoff_row_does_not_start_file_search_processes() {
        let source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let handoff_source = source
            .split("fn root_file_search_handoff_result(")
            .nth(1)
            .and_then(|section| section.split("/// Incomplete menu-syntax hint row.").next())
            .expect("root_file_search_handoff_result source should be present");

        for forbidden in [
            "mdfind",
            "search_files(",
            "search_files_streaming",
            "std::process::Command",
            "std::fs::read_dir",
            "list_directory",
        ] {
            assert!(
                !handoff_source.contains(forbidden),
                "root file handoff row should only reuse the fallback execution path, not start searches: {forbidden}"
            );
        }
    }

    #[test]
    fn root_file_handoff_row_groups_after_files_before_fallbacks() {
        let source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let append_source = source
            .split("fn append_root_file_section(")
            .nth(1)
            .and_then(|section| section.split("fn root_file_search_handoff_result(").next())
            .expect("append_root_file_section source should be present");

        let section_offset = append_source
            .find("GroupedListItem::SectionHeader(\"Files\".to_string(), None)")
            .expect("Files section header should be inserted");
        let file_offset = append_source
            .find("flat_results.push(SearchResult::File(file_match));")
            .expect("actual file rows should be inserted");
        let handoff_offset = append_source
            .find("flat_results.push(handoff);")
            .expect("handoff row should be inserted after actual file rows");
        let splice_offset = append_source
            .find("grouped.splice(insertion_index..insertion_index, file_group);")
            .expect("Files group should still be spliced before fallback rows");

        assert!(
            section_offset < file_offset && file_offset < handoff_offset && handoff_offset < splice_offset,
            "Files section should render real file rows, then the handoff row, before the group is inserted ahead of fallbacks"
        );
        assert!(
            append_source.contains("let handoff = root_file_search_handoff_result(filter_text);")
                && append_source.contains("files.is_empty() && handoff.is_none()"),
            "Files section should still appear with the handoff row when Spotlight returns zero file rows"
        );
    }

    #[test]
    fn fallback_keeps_window_open_uses_search_files_constant() {
        let source = fs::read_to_string("src/app_impl/selection_fallback.rs")
            .expect("read src/app_impl/selection_fallback.rs");
        let body = source
            .split("fn fallback_keeps_window_open(")
            .nth(1)
            .and_then(|section| {
                section
                    .split("fn should_ignore_main_menu_open_carryover_input")
                    .next()
            })
            .expect("fallback_keeps_window_open source should be present");

        assert!(
            body.contains("SEARCH_FILES_FALLBACK_ID"),
            "search-files fallback window behavior should use the exported id constant"
        );
        assert!(
            !body.contains("\"search-files\""),
            "search-files id should not be repeated as a literal in fallback_keeps_window_open"
        );
    }

    #[test]
    fn fallback_mode_enter_prefers_visible_grouped_fallback_selection() {
        let source = fs::read_to_string("src/app_impl/selection_fallback.rs")
            .expect("read src/app_impl/selection_fallback.rs");
        let body = source
            .split("pub fn execute_selected_fallback(")
            .nth(1)
            .and_then(|section| {
                section
                    .split("/// Execute a built-in fallback action without window reference")
                    .next()
            })
            .expect("execute_selected_fallback source should be present");

        assert!(
            body.contains("self.selected_main_list_search_result_owned()")
                && body.contains("scripts::SearchResult::Fallback(fallback_match)")
                && body.contains("self.execute_fallback_item(&fallback_match.fallback, cx);")
                && body.find("selected_main_list_search_result_owned()")
                    < body.find("main_menu_fallback_state.selected_item()"),
            "fallback-mode Enter should execute the visible grouped fallback row before consulting the legacy fallback cursor"
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
            "root_file_copy_name",
            "root_file_quick_look",
            "root_file_search_in_folder",
            "root_file_browse_parent_folder",
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
            "root_file_copy_name",
            "root_file_quick_look",
            "root_file_search_in_folder",
            "root_file_browse_parent_folder",
        ] {
            assert!(
                source.contains(action_id),
                "root file action id should be reserved: {action_id}"
            );
        }
        assert!(
            source.contains("ROOT_FILE_SEARCH_IN_FOLDER_ACTION_ID")
                && source.contains("| ROOT_FILE_SEARCH_IN_FOLDER_ACTION_ID"),
            "Search Inside Folder action id should be reserved and recognized as a captured root-file action"
        );
        assert!(
            source.contains("ROOT_FILE_BROWSE_PARENT_FOLDER_ACTION_ID")
                && source.contains("| ROOT_FILE_BROWSE_PARENT_FOLDER_ACTION_ID"),
            "Browse Parent Folder action id should be reserved and recognized as a captured root-file action"
        );
        assert!(
            source.contains("ROOT_FILE_COPY_NAME_ACTION_ID")
                && source.contains("| ROOT_FILE_COPY_NAME_ACTION_ID"),
            "Copy Name action id should be reserved and recognized as a captured root-file action"
        );
    }

    #[test]
    fn root_file_search_in_folder_action_is_directory_only() {
        let source = fs::read_to_string("src/app_impl/actions_toggle.rs")
            .expect("read src/app_impl/actions_toggle.rs");
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            normalized.contains("file.file_type == crate::file_search::FileType::Directory")
                && normalized.contains("if is_dir { actions.push( Action::new( crate::action_helpers::ROOT_FILE_SEARCH_IN_FOLDER_ACTION_ID"),
            "Search Inside Folder should only be added for directory root-file rows"
        );
    }

    #[test]
    fn root_file_search_in_folder_action_opens_dedicated_file_search() {
        let source = fs::read_to_string("src/app_impl/selection_fallback.rs")
            .expect("read src/app_impl/selection_fallback.rs");
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            normalized.contains("ROOT_FILE_SEARCH_IN_FOLDER_ACTION_ID")
                && normalized.contains("ensure_trailing_slash(&file.path)")
                && normalized.contains("self.open_file_search(query, cx);"),
            "root folder action should hand off to dedicated File Search with a trailing-slashed directory path"
        );
    }

    #[test]
    fn root_file_browse_parent_folder_action_is_file_only_and_opens_dedicated_file_search() {
        let actions_source = fs::read_to_string("src/app_impl/actions_toggle.rs")
            .expect("read src/app_impl/actions_toggle.rs");
        let selection_source = fs::read_to_string("src/app_impl/selection_fallback.rs")
            .expect("read src/app_impl/selection_fallback.rs");
        let actions_normalized = actions_source
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let selection_normalized = selection_source
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        assert!(
            actions_normalized.contains("ROOT_FILE_BROWSE_PARENT_FOLDER_ACTION_ID")
                && actions_normalized.contains("!is_dir"),
            "Browse Parent Folder should only be rendered for regular root file rows"
        );
        assert!(
            selection_normalized.contains("root_file_browse_parent_folder_query")
                && selection_normalized.contains("parent_folder_search_query(&file.path)")
                && selection_normalized.contains("self.open_file_search(query, cx);"),
            "Browse Parent Folder should hand off to dedicated File Search at the containing folder"
        );
    }

    #[test]
    fn root_file_copy_name_action_copies_basename_only() {
        let source = fs::read_to_string("src/app_impl/selection_fallback.rs")
            .expect("read src/app_impl/selection_fallback.rs");
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            normalized.contains("ROOT_FILE_COPY_NAME_ACTION_ID")
                && normalized.contains("gpui::ClipboardItem::new_string(file.name.clone())")
                && normalized.contains("format!(\"Copied name: {}\", file.name)"),
            "Copy Name should copy only FileResult.name and show basename-only HUD feedback"
        );
    }

    #[test]
    fn root_launcher_does_not_browse_directories_inline() {
        let source = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("read src/render_script_list/mod.rs");

        assert!(
            !source.contains("ROOT_FILE_SEARCH_IN_FOLDER_ACTION_ID"),
            "root folder handoff should live in the action executor, not ScriptList key handling"
        );
        assert!(
            !source.contains("open_file_search("),
            "ScriptList render/key handling should not directly open File Search for root folder rows"
        );
    }
}
