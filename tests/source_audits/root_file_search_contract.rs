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
    fn root_unified_search_config_gates_provider_and_grouping() {
        let config_source =
            fs::read_to_string("src/config/types.rs").expect("read src/config/types.rs");
        let root_source = fs::read_to_string("src/app_impl/root_file_search.rs")
            .expect("read src/app_impl/root_file_search.rs");
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let filtering_source = fs::read_to_string("src/app_impl/filtering_cache.rs")
            .expect("read src/app_impl/filtering_cache.rs");

        assert!(
            config_source.contains("pub struct UnifiedSearchConfig")
                && config_source.contains("pub struct UnifiedSearchFilesConfig")
                && config_source.contains("pub enum RootFilePromotionConfig")
                && config_source.contains("pub fn root_file_section_options(&self)"),
            "config.ts should expose real unifiedSearch.files controls that map to root file section options"
        );
        assert!(
            root_source.contains("root_file_options.files_enabled")
                && root_source.contains("root_file_options.global_search_enabled")
                && root_source.contains("root_file_options.directory_browse_enabled")
                && root_source.contains("options.recent_files_enabled"),
            "root provider startup and recent hydration should honor unifiedSearch.files gates before launching work"
        );
        assert!(
            filtering_source.contains("let unified_search = self.config.get_unified_search();")
                && filtering_source.contains("unified_search.root_file_section_options()")
                && filtering_source.contains(
                    "get_grouped_results_with_validation_query_and_root_files_with_options"
                ),
            "main menu grouping should pass config-derived root file options into the Files section"
        );
        assert!(
            grouping_source.contains("!options.files_enabled")
                && grouping_source.contains("!options.recent_files_enabled")
                && grouping_source.contains("!options.global_search_enabled")
                && grouping_source.contains("!options.directory_browse_enabled"),
            "grouping should keep a defensive config gate even if provider state is stale"
        );
    }

    #[test]
    fn async_root_file_updates_preserve_selection_by_stable_key() {
        let state_source =
            fs::read_to_string("src/main_sections/app_state.rs").expect("read app_state.rs");
        let root_source = fs::read_to_string("src/app_impl/root_file_search.rs")
            .expect("read src/app_impl/root_file_search.rs");

        assert!(
            state_source.contains("struct MainMenuSelectionSnapshot")
                && state_source.contains("main_menu_selection_snapshot")
                && state_source.contains("restore_main_menu_selection_from_snapshot")
                && state_source.contains("stable_selection_key()")
                && state_source.contains("grouped_index_for_stable_selection_key"),
            "main menu selection preservation should snapshot and restore stable result keys, not grouped indexes"
        );
        assert!(
            root_source.contains("let selection_before =")
                && root_source.contains("self.main_menu_selection_snapshot()")
                && root_source.contains("self.restore_main_menu_selection_from_snapshot(snapshot)")
                && root_source.contains("self.sync_list_state_for_filter_replacement();")
                && root_source.contains("self.validate_selection_bounds(cx);"),
            "async root file publish path should rebuild rows, restore the previous key, then validate bounds"
        );
    }

    #[test]
    fn root_global_file_search_caches_results_without_active_frame_publish() {
        let source = fs::read_to_string("src/app_impl/root_file_search.rs")
            .expect("read src/app_impl/root_file_search.rs");
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            !normalized.contains("publish_partial_results"),
            "root global file search should not stream partial batches into the active root frame"
        );
        assert!(
            normalized.contains("Ok(crate::file_search::SearchEvent::Result(result)) =>")
                && !normalized.contains("app.apply_root_file_search_results_for_generation( generation, snapshot, true, false, cx, );"),
            "SearchEvent::Result should collect into the batch without publishing a visible partial frame"
        );
        assert!(
            normalized.contains("Ok(crate::file_search::SearchEvent::Done) => break")
                && normalized.contains("app.cache_root_file_search_results_for_generation( generation, request_cache_key, batch, true, );"),
            "final global root file update should warm the cache instead of mutating the active frame"
        );
        assert!(
            normalized.contains("let publish_active_results =")
                && normalized.contains(
                    "matches!(&request, RootFileSearchRequest::DirectoryBrowse { .. })"
                )
                && normalized.contains("if publish_active_results"),
            "only explicit directory browse may publish its final collected rows into the active frame"
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
    fn root_file_directory_context_ranking_stays_bounded_and_below_filename_first() {
        let source =
            fs::read_to_string("src/file_search/mod.rs").expect("read src/file_search/mod.rs");
        let production = production_source(&source);
        let provider_source = production
            .split("pub fn root_file_provider_query_for_user_query(")
            .nth(1)
            .and_then(|section| {
                section
                    .split("/// Returns true when the root launcher")
                    .next()
            })
            .expect("root provider query builder should be present");
        let rank_source = production
            .split("pub fn rank_root_file_results(")
            .nth(1)
            .and_then(|section| section.split("/// Payload for file drag-out").next())
            .expect("rank_root_file_results source should be present");

        assert!(
            production.contains("const ROOT_FILE_PATH_CONTEXT_TIER: i32 = 3")
                && production.contains("const ROOT_FILE_PATH_CONTEXT_MAX_TERMS: usize = 4"),
            "directory-context ranking should stay below filename token tier 4 and bounded by term count"
        );
        assert!(
            provider_source.contains("root_file_path_context_mdquery_branches(&terms)")
                && provider_source.contains("kMDItemPath ==")
                && provider_source.contains("kMDItemFSName ==")
                && provider_source.contains("root_file_query_has_safe_global_length(term)"),
            "provider query should add bounded path+filename branches only for safe parent terms"
        );
        assert!(
            rank_source.contains("root_file_path_context_matches_query(file, &q)")
                && rank_source.contains("ROOT_FILE_PATH_CONTEXT_TIER")
                && rank_source.contains("root_file_name_relevance_tier(&file.name, &q, name_matched).max"),
            "root ranking should apply directory-context relevance without replacing filename-first relevance"
        );
    }

    #[test]
    fn root_file_renderer_uses_file_type_specific_svg_icons() {
        let source = fs::read_to_string("src/designs/core/render.rs")
            .expect("read src/designs/core/render.rs");
        let file_arm = source
            .split("SearchResult::File(fm) =>")
            .nth(1)
            .and_then(|section| section.split("SearchResult::Skill").next())
            .expect("SearchResult::File arm should be present");

        assert!(
            source.contains("fn root_file_type_svg_icon(")
                && source.contains("FileType::Directory => \"FolderOpen\"")
                && source.contains("FileType::Image => \"file-image\"")
                && source.contains("FileType::Document => \"file-text\"")
                && source.contains("FileType::Audio => \"file-audio\"")
                && source.contains("FileType::Video => \"file-video\"")
                && source.contains("FileType::Application => \"package\""),
            "root file SVG icon mapping should live in a small named helper with type-specific icons"
        );
        assert!(
            file_arm.contains("root_file_type_svg_icon(fm.file.file_type)"),
            "root file rows should derive their SVG icon from FileResult.file_type"
        );
        assert!(
            !file_arm.contains("IconKind::Svg(\"File\".to_string())"),
            "root file rows should no longer hardcode the generic File icon"
        );
        assert!(
            !file_arm.contains("IconKind::Image")
                && !file_arm.contains("is_thumbnail_preview_supported"),
            "root launcher file rows should stay on static SVG icons, not thumbnails"
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
            .find("root_file_section_title(mode, root_file_search_loading).to_string()")
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
            append_source
                .contains("let handoff = root_file_search_handoff_result(filter_text, mode);")
                && append_source.contains("files.is_empty() && handoff.is_none()"),
            "Files section should still appear with the handoff row when Spotlight returns zero file rows"
        );
    }

    #[test]
    fn root_global_file_promotion_is_policy_gated_and_grouping_only() {
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let production = production_source(&grouping_source);
        let helper = production
            .split("fn root_file_section_should_promote(")
            .nth(1)
            .and_then(|section| {
                section
                    .split("fn root_file_section_insertion_index(")
                    .next()
            })
            .expect("root file promotion helper should be present");

        assert!(
            helper.contains("RootFilePromotionPolicy::Never")
                && helper.contains("RootFileSectionMode::GlobalQuery")
                && helper.contains("filter_text.trim()")
                && helper.contains("root_file_global_query_is_eligible(query)")
                && helper.contains("flat_results.iter().any(is_primary_launcher_result)")
                && helper.contains("files.first()")
                && helper.contains("RootFilePromotionPolicy::ExactFilenameOnly")
                && helper.contains("root_file_name_exact_or_stem_matches_query("),
            "root file promotion should default to never and require exact filename/stem opt-in without primary launcher rows"
        );
        assert!(
            production.contains("root_file_section_insertion_index(grouped, flat_results, promote)")
                && production.contains("SearchResult::ScriptIssue(_)"),
            "promotion should route through a grouping insertion helper while preserving the script issue row"
        );
        for forbidden in [
            "mdfind",
            "search_files(",
            "search_files_streaming",
            "std::process::Command",
            "std::fs::read_dir",
            "list_directory",
        ] {
            assert!(
                !helper.contains(forbidden),
                "root file promotion must not start providers: {forbidden}"
            );
        }
    }

    #[test]
    fn root_global_promotion_uses_exact_gate_while_recent_seeds_keep_token_gate() {
        let file_search_source =
            fs::read_to_string("src/file_search/mod.rs").expect("read src/file_search/mod.rs");
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let file_search_production = production_source(&file_search_source);
        let grouping_production = production_source(&grouping_source);

        assert!(
            file_search_production.contains("pub fn root_file_name_token_matches_query(")
                && file_search_production.contains("root_file_name_seed_matches_query")
                && file_search_production.contains("root_file_recent_seed_matches_query")
                && file_search_production
                    .contains("root_file_name_token_matches_query(name, query)"),
            "recent seed eligibility should preserve the shared filename-token gate"
        );

        let promote_helper = grouping_production
            .split("fn root_file_section_should_promote(")
            .nth(1)
            .and_then(|section| {
                section
                    .split("fn root_file_section_insertion_index(")
                    .next()
            })
            .expect("root file promotion helper should be present");
        assert!(
            promote_helper.contains("RootFileSectionMode::GlobalQuery")
                && promote_helper.contains("root_file_global_query_is_eligible(query)")
                && promote_helper.contains("flat_results.iter().any(is_primary_launcher_result)")
                && promote_helper.contains("root_file_name_exact_or_stem_matches_query("),
            "promotion should be stricter than recent seeds: exact filename/stem only and blocked by primary launcher rows"
        );
        assert!(
            !promote_helper.contains("ROOT_FILE_MIN_QUERY_CHARS"),
            "promotion should share root file global query eligibility instead of keeping a stale raw length check"
        );
        for forbidden in [
            "mdfind",
            "search_files(",
            "search_files_streaming",
            "std::process::Command",
            "std::fs::read_dir",
            "list_directory",
        ] {
            assert!(
                !promote_helper.contains(forbidden),
                "root file promotion must not start providers: {forbidden}"
            );
        }
    }

    #[test]
    fn root_recent_files_share_global_file_eligibility() {
        let file_search_source =
            fs::read_to_string("src/file_search/mod.rs").expect("read src/file_search/mod.rs");
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let file_search_production = production_source(&file_search_source);
        let grouping_production = production_source(&grouping_source);

        let hydrate_body = file_search_production
            .split("pub fn file_result_from_existing_path(")
            .nth(1)
            .and_then(|section| section.split("/// Convert directory browse results").next())
            .expect("file_result_from_existing_path body should be present");
        assert!(
            hydrate_body.contains("root_global_file_result_is_eligible(&result)"),
            "recent file hydration should use the shared global root file eligibility gate"
        );

        let recent_body = grouping_production
            .split("fn append_recent_root_file_section(")
            .nth(1)
            .and_then(|section| section.split("fn append_root_file_section(").next())
            .expect("append_recent_root_file_section body should be present");
        assert!(
            recent_body.contains("root_global_file_result_is_eligible(file)"),
            "empty-root Recent Files should defensively filter app bundles and bundle contents"
        );
    }

    #[test]
    fn root_file_token_gate_supports_developer_filename_boundaries() {
        let file_search_source =
            fs::read_to_string("src/file_search/mod.rs").expect("read src/file_search/mod.rs");
        let file_search_production = production_source(&file_search_source);
        let token_helper = file_search_production
            .split("fn contains_at_root_file_token_boundary(")
            .nth(1)
            .and_then(|section| section.split("fn is_root_file_boundary_char(").next())
            .expect("root filename token-boundary helper should be present");
        let boundary_helper = file_search_production
            .split("fn is_root_file_token_boundary_at(")
            .nth(1)
            .and_then(|section| section.split("fn is_root_file_boundary_char(").next())
            .expect("root token boundary classifier should be present");

        assert!(
            file_search_production.contains("contains_at_root_file_token_boundary(name, query)")
                && file_search_production
                    .contains("fn root_file_name_token_matches_single_term(")
                && file_search_production
                    .contains("root_file_name_token_matches_query(name, query)"),
            "ranking, promotion, and recent seed gates should all share the developer filename token helper"
        );
        assert!(
            token_helper.contains("haystack.to_lowercase()")
                && token_helper.contains("needle.to_lowercase()")
                && token_helper.contains("is_root_file_token_boundary_at(haystack, idx)"),
            "token matching should stay case-insensitive while preserving original casing for boundary classification"
        );
        assert!(
            boundary_helper.contains("previous.is_ascii_lowercase() && current.is_ascii_uppercase()")
                && boundary_helper.contains("previous.is_ascii_digit() && current.is_ascii_alphabetic()")
                && boundary_helper
                    .contains("previous.is_ascii_uppercase()")
                && boundary_helper
                    .contains("next.is_some_and(|ch| ch.is_ascii_lowercase())"),
            "root file token boundaries should include camel-case, digit-to-word, and acronym-to-word filename transitions"
        );
        for forbidden in [
            "mdfind",
            "search_files(",
            "search_files_streaming",
            "std::process::Command",
            "std::fs::read_dir",
            "list_directory",
        ] {
            assert!(
                !token_helper.contains(forbidden) && !boundary_helper.contains(forbidden),
                "developer filename token matching must not start providers: {forbidden}"
            );
        }
    }

    #[test]
    fn root_global_multiword_provider_and_token_gate_stay_shared() {
        let file_search_source =
            fs::read_to_string("src/file_search/mod.rs").expect("read src/file_search/mod.rs");
        let root_source = fs::read_to_string("src/app_impl/root_file_search.rs")
            .expect("read src/app_impl/root_file_search.rs");
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let file_search_production = production_source(&file_search_source);
        let root_production = production_source(&root_source);
        let root_normalized = root_production
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let grouping_production = production_source(&grouping_source);

        assert!(
            file_search_production.contains("pub fn root_file_provider_query_for_user_query(")
                && file_search_production.contains("fn root_file_query_terms(")
                && file_search_production.contains("root_file_text_matches_terms_in_order")
                && file_search_production
                    .contains("root_file_name_token_matches_single_term(name, &terms[0])"),
            "root file token matching should share one query-term parser across single and multi-word gates"
        );
        assert!(
            root_normalized.contains("root_file_provider_query_for_user_query(&query)")
                && root_normalized.contains("search_files_streaming_with_options( &provider_query,"),
            "root global provider search should expand safe multi-word filename queries before launching Spotlight"
        );

        let promote_helper = grouping_production
            .split("fn root_file_section_should_promote(")
            .nth(1)
            .and_then(|section| {
                section
                    .split("fn root_file_section_insertion_index(")
                    .next()
            })
            .expect("root file promotion helper should be present");
        assert!(
            promote_helper.contains("root_file_name_exact_or_stem_matches_query("),
            "multi-word promotion should stay exact-only instead of using the broader filename-token gate"
        );
        assert!(
            file_search_production.contains("root_file_name_token_matches_query(name, query)"),
            "recent seeds should continue to delegate to the shared filename-token gate"
        );
    }

    #[test]
    fn root_global_short_digit_queries_share_search_and_promotion_eligibility() {
        let file_search_source =
            fs::read_to_string("src/file_search/mod.rs").expect("read src/file_search/mod.rs");
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let file_search_production = production_source(&file_search_source);
        let grouping_production = production_source(&grouping_source);

        assert!(
            file_search_production.contains("pub fn root_file_global_query_is_eligible(")
                && file_search_production.contains("fn root_file_query_has_safe_global_length(")
                && file_search_production.contains("fn root_file_short_digit_token_query(")
                && file_search_production.contains("root_file_global_query_is_eligible(query)"),
            "root file search should expose one shared global eligibility helper with a narrow short-digit exception"
        );

        let promote_helper = grouping_production
            .split("fn root_file_section_should_promote(")
            .nth(1)
            .and_then(|section| {
                section
                    .split("fn root_file_section_insertion_index(")
                    .next()
            })
            .expect("root file promotion helper should be present");
        assert!(
            promote_helper.contains("root_file_global_query_is_eligible(query)")
                && promote_helper.contains("flat_results.iter().any(is_primary_launcher_result)")
                && promote_helper.contains("root_file_name_exact_or_stem_matches_query("),
            "promotion should share root global search eligibility before applying exact filename-only promotion"
        );
        assert!(
            !promote_helper.contains("ROOT_FILE_MIN_QUERY_CHARS"),
            "promotion must not keep a stale raw min-length gate after short digit tokens become eligible"
        );
    }

    #[test]
    fn root_global_file_promotion_respects_primary_launcher_rows() {
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let production = production_source(&grouping_source);
        let launcher_helper = production
            .split("fn is_primary_launcher_result(")
            .nth(1)
            .and_then(|section| {
                section
                    .split("fn root_file_section_insertion_index(")
                    .next()
            })
            .expect("primary launcher helper should be present");

        assert!(
            launcher_helper.contains("SearchResult::Script(_)")
                && launcher_helper.contains("SearchResult::Scriptlet(_)")
                && launcher_helper.contains("SearchResult::Skill(_)")
                && launcher_helper.contains("SearchResult::BuiltIn(_)")
                && launcher_helper.contains("SearchResult::App(_)")
                && launcher_helper.contains("SearchResult::Window(_)"),
            "file promotion should be blocked by any primary launcher row"
        );
        for forbidden in [
            "mdfind",
            "search_files(",
            "search_files_streaming",
            "std::process::Command",
            "std::fs::read_dir",
            "list_directory",
        ] {
            assert!(
                !launcher_helper.contains(forbidden),
                "primary launcher guard must stay grouping-only: {forbidden}"
            );
        }
    }

    #[test]
    fn root_directory_browse_provider_stays_in_app_layer() {
        let root_source = fs::read_to_string("src/app_impl/root_file_search.rs")
            .expect("read src/app_impl/root_file_search.rs");
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let filtering_source = fs::read_to_string("src/app_impl/filtering_cache.rs")
            .expect("read src/app_impl/filtering_cache.rs");
        let grouping_production = production_source(&grouping_source);

        assert!(
            root_source.contains("RootFileSearchRequest::DirectoryBrowse")
                && root_source.contains("list_directory_with_options(")
                && root_source.contains("ROOT_FILE_BROWSE_SOURCE_LIMIT"),
            "root directory browsing should collect directory rows in the root file app/provider layer"
        );
        assert!(
            grouping_production.contains("RootFileSectionMode::DirectoryBrowse")
                && grouping_production.contains("root_directory_file_matches(")
                && grouping_production.contains("root_directory_browse_child_filter(filter_text)")
                && grouping_production.contains("ROOT_FILE_BROWSE_RENDER_LIMIT"),
            "grouping should render already-collected directory rows without starting providers"
        );
        for forbidden in ["std::fs::read_dir", "list_directory_with_options("] {
            assert!(
                !grouping_production.contains(forbidden),
                "grouping should not start directory providers directly: {forbidden}"
            );
        }
        assert!(
            filtering_source.contains("self.root_file_search_mode")
                && filtering_source.contains("self.root_file_search_loading")
                && filtering_source.contains("&self.root_file_results"),
            "filtering cache should pass the root file source mode, loading state, and collected rows"
        );
    }

    #[test]
    fn root_file_loading_state_reaches_files_section_header() {
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let filtering_source = fs::read_to_string("src/app_impl/filtering_cache.rs")
            .expect("read src/app_impl/filtering_cache.rs");

        assert!(
            grouping_source.contains("root_file_search_loading: bool"),
            "root grouping should accept the root file provider loading state"
        );
        assert!(
            grouping_source.contains("fn root_file_section_title(")
                && grouping_source.contains("\"Files · Searching...\"")
                && grouping_source.contains("\"Files · Loading folder...\""),
            "root Files section should expose distinct loading copy for global and directory modes"
        );
        assert!(
            grouping_source
                .contains("root_file_section_title(mode, root_file_search_loading).to_string()"),
            "root Files section header should be computed from mode and loading state"
        );
        assert!(
            filtering_source.contains("self.root_file_search_loading"),
            "filtering cache should pass root provider loading state into grouping"
        );
    }

    #[test]
    fn root_global_recent_file_seed_stays_grouping_only() {
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let production = production_source(&grouping_source);
        let helper = production
            .split("fn merge_root_global_file_results_with_recent(")
            .nth(1)
            .and_then(|section| section.split("fn root_file_section_title(").next())
            .expect("recent merge helper should be present");

        for forbidden in [
            "mdfind",
            "search_files(",
            "search_files_streaming",
            "std::process::Command",
            "std::fs::read_dir",
            "list_directory",
        ] {
            assert!(
                !helper.contains(forbidden),
                "recent seed merge must not start providers: {forbidden}"
            );
        }
        assert!(
            production.contains("root_recent_file_results")
                && production.contains("merge_root_global_file_results_with_recent(")
                && production.contains("root_file_recent_seed_matches_query")
                && production.contains("RootFileSectionMode::DirectoryBrowse"),
            "root grouping should pass eligible recent files into global ranking without changing directory browse mode"
        );
    }

    #[test]
    fn root_global_recent_file_seed_filters_path_only_matches() {
        let file_search_source =
            fs::read_to_string("src/file_search/mod.rs").expect("read src/file_search/mod.rs");
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let file_search_production = production_source(&file_search_source);
        let production = production_source(&grouping_source);
        let helper = production
            .split("fn merge_root_global_file_results_with_recent(")
            .nth(1)
            .and_then(|section| section.split("fn root_file_section_title(").next())
            .expect("recent merge helper should be present");
        let recent_seed_helper = file_search_production
            .split("pub fn root_file_recent_seed_matches_query(")
            .nth(1)
            .and_then(|section| {
                section
                    .split("fn contains_at_root_file_token_boundary(")
                    .next()
            })
            .expect("recent seed helper should be present");

        assert!(
            helper.contains("filter_text: &str")
                && helper.contains("provider_results")
                && helper.contains("recent_results")
                && helper.contains("root_file_recent_seed_matches_query(file, filter_text)"),
            "recent root file seeds should be filtered by the shared recent-seed predicate while provider rows stay unfiltered"
        );
        assert!(
            recent_seed_helper.contains("root_file_global_query_is_eligible(query)")
                && recent_seed_helper.contains("root_file_name_seed_matches_query(&file.name, query)")
                && recent_seed_helper.contains("root_file_path_context_matches_query(file, query)"),
            "recent seed eligibility should require a global query and allow filename-token or ordered directory-context matches"
        );
        for forbidden in [
            "mdfind",
            "search_files(",
            "search_files_streaming",
            "std::process::Command",
            "std::fs::read_dir",
            "list_directory",
        ] {
            assert!(
                !helper.contains(forbidden),
                "recent seed filename filter must not start providers: {forbidden}"
            );
        }
    }

    #[test]
    fn root_global_file_rows_filter_app_bundles_without_affecting_directory_browse() {
        let file_search_source =
            fs::read_to_string("src/file_search/mod.rs").expect("read src/file_search/mod.rs");
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let file_search_production = production_source(&file_search_source);
        let grouping_production = production_source(&grouping_source);

        assert!(
            file_search_production.contains("pub fn root_global_file_result_is_eligible(")
                && file_search_production.contains("file.file_type != FileType::Application"),
            "root global file result eligibility should keep app bundles out of global Files"
        );

        let merge_helper = grouping_production
            .split("fn merge_root_global_file_results_with_recent(")
            .nth(1)
            .and_then(|section| section.split("fn root_file_section_title(").next())
            .expect("recent/global merge helper should be present");
        assert!(
            merge_helper.contains("root_global_file_result_is_eligible(file)")
                && merge_helper
                    .contains("root_file_recent_seed_matches_query(file, filter_text)"),
            "global Files should filter app bundles before ranking while preserving recent seed gating"
        );
        assert!(
            grouping_production.contains("RootFileSectionMode::DirectoryBrowse")
                && grouping_production.contains("root_directory_file_matches("),
            "directory browse should keep rendering already-collected direct children, including app bundles"
        );

        for forbidden in [
            "mdfind",
            "search_files(",
            "search_files_streaming",
            "std::process::Command",
            "std::fs::read_dir",
            "list_directory",
        ] {
            assert!(
                !merge_helper.contains(forbidden),
                "global app-bundle filtering must stay grouping-only: {forbidden}"
            );
        }
    }

    #[test]
    fn root_global_file_rows_filter_app_bundle_contents_without_affecting_directory_browse() {
        let file_search_source =
            fs::read_to_string("src/file_search/mod.rs").expect("read src/file_search/mod.rs");
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let file_search_production = production_source(&file_search_source);
        let grouping_production = production_source(&grouping_source);

        assert!(
            file_search_production.contains("fn path_contains_application_bundle_component(")
                && file_search_production.contains("eq_ignore_ascii_case(\"app\")")
                && file_search_production.contains("!path_contains_application_bundle_component(&file.path)"),
            "global root file eligibility should reject rows nested under .app bundle path components"
        );

        let merge_helper = grouping_production
            .split("fn merge_root_global_file_results_with_recent(")
            .nth(1)
            .and_then(|section| section.split("fn root_file_section_title(").next())
            .expect("recent/global merge helper should be present");
        assert!(
            merge_helper.contains("root_global_file_result_is_eligible(file)"),
            "global provider and recent rows should share the app-bundle-content eligibility gate"
        );
        assert!(
            grouping_production.contains("RootFileSectionMode::DirectoryBrowse")
                && grouping_production.contains("root_directory_file_matches("),
            "explicit directory browse should stay outside the global app-bundle-content filter"
        );

        for forbidden in [
            "mdfind",
            "search_files(",
            "search_files_streaming",
            "std::process::Command",
            "std::fs::read_dir",
            "list_directory",
        ] {
            assert!(
                !merge_helper.contains(forbidden),
                "app-bundle-content filtering must stay grouping-only: {forbidden}"
            );
        }
    }

    #[test]
    fn nonempty_global_root_search_refreshes_recent_file_snapshot() {
        let source = fs::read_to_string("src/app_impl/filtering_cache.rs")
            .expect("read src/app_impl/filtering_cache.rs");
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            normalized.contains("RootFileSectionMode::GlobalQuery")
                && normalized.contains("self.refresh_root_recent_file_results();"),
            "non-empty global root file search should refresh the frecency-backed recent file snapshot"
        );
    }

    #[test]
    fn root_file_direct_shortcuts_route_through_shared_action_executor() {
        let selection_source = fs::read_to_string("src/app_impl/selection_fallback.rs")
            .expect("read src/app_impl/selection_fallback.rs");
        let normalized = selection_source
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        assert!(
            normalized.contains("pub(crate) fn try_execute_root_file_action_shortcut")
                && normalized.contains("ROOT_FILE_QUICK_LOOK_ACTION_ID")
                && normalized.contains("ROOT_FILE_COPY_PATH_ACTION_ID")
                && normalized.contains("ROOT_FILE_REVEAL_IN_FINDER_ACTION_ID")
                && normalized.contains("selected_root_file_result_owned()")
                && normalized.contains("self.execute_root_file_action(action_id, &file, window, cx)"),
            "root file direct shortcuts should route selected file rows through the shared root-file action executor"
        );

        for path in [
            "src/app_impl/startup.rs",
            "src/app_impl/startup_new_tab.rs",
            "src/main_entry/runtime_stdin_match_simulate_key.rs",
            "src/main_entry/app_run_setup.rs",
        ] {
            let source = fs::read_to_string(path).unwrap_or_else(|_| panic!("read {path}"));
            assert!(
                source.contains("try_execute_root_file_action_shortcut"),
                "{path} should offer direct root-file shortcuts on the same ScriptList key path as root directory navigation"
            );
        }
    }

    #[test]
    fn root_directory_child_fragment_filtering_stays_direct_child_only() {
        let file_search_source =
            fs::read_to_string("src/file_search/mod.rs").expect("read src/file_search/mod.rs");
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let file_search_production = production_source(&file_search_source);
        let grouping_production = production_source(&grouping_source);

        assert!(
            file_search_production.contains("pub fn root_directory_query_base(")
                && file_search_production.contains("root_file_section_mode_for_query")
                && file_search_production.contains("RootFileSectionMode::DirectoryBrowse")
                && file_search_production.contains("root_directory_file_matches("),
            "root directory child-fragment queries should stay in directory-browse mode"
        );
        assert!(
            grouping_production.contains("fn root_directory_browse_child_filter(")
                && grouping_production.contains("root_directory_query_base(query)")
                && grouping_production.contains("child_filter.as_deref()"),
            "grouping should derive only a child-name filter and pass it to already-collected rows"
        );
        for forbidden in [
            "std::fs::read_dir",
            "list_directory_with_options(",
            "search_files_streaming",
            "mdfind",
        ] {
            assert!(
                !grouping_production.contains(forbidden),
                "filtered root directory grouping should not start providers directly: {forbidden}"
            );
        }
    }

    #[test]
    fn root_directory_child_fragment_edits_reuse_active_provider() {
        let root_source = fs::read_to_string("src/app_impl/root_file_search.rs")
            .expect("read src/app_impl/root_file_search.rs");
        let file_search_source =
            fs::read_to_string("src/file_search/mod.rs").expect("read src/file_search/mod.rs");
        let normalized = root_source.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            file_search_source.contains("pub fn root_directory_browse_source_key(")
                && file_search_source.contains("parse_directory_path(query)?")
                && file_search_source.contains("Some((parsed.directory, parsed.show_hidden))"),
            "file search should expose a provider key based on directory plus hidden-file mode"
        );
        assert!(
            root_source.contains("fn active_root_directory_browse_source_matches(")
                && root_source.contains(
                    "root_directory_browse_source_key(&self.root_file_search_query)"
                ),
            "root app layer should compare active directory-browse provider identity separately from the visible query"
        );
        assert!(
            normalized.contains(
                "RootFileSearchRequest::DirectoryBrowse { query, directory, show_hidden, } if self.active_root_directory_browse_source_matches(directory, *show_hidden)"
            ) && normalized.contains("self.root_file_search_query = query.clone();")
                && normalized.contains("self.refresh_root_file_grouping_after_query_only_change(cx);"),
            "directory child-fragment edits should only update the visible query and regroup cached rows"
        );
        assert!(
            !normalized.contains("app.root_file_search_query != query_for_task"),
            "directory listings should be allowed to complete after the visible child fragment changes"
        );
    }

    #[test]
    fn root_directory_tab_navigation_precedes_plain_tab_acp_routing() {
        for path in [
            "src/app_impl/startup.rs",
            "src/app_impl/startup_new_tab.rs",
            "src/main_entry/runtime_stdin_match_simulate_key.rs",
            "src/main_entry/app_run_setup.rs",
        ] {
            let source = fs::read_to_string(path).unwrap_or_else(|_| panic!("read {path}"));
            let nav_offset = source
                .find("try_navigate_root_file_directory_with_tab")
                .unwrap_or_else(|| panic!("{path} should route root file directory Tab"));
            let acp_offset = source
                .find("try_route_plain_tab_to_acp_context_capture")
                .unwrap_or_else(|| panic!("{path} should still preserve ACP Tab routing"));

            assert!(
                nav_offset < acp_offset,
                "{path} should try root directory navigation before plain Tab ACP routing"
            );
        }

        let selection_source = fs::read_to_string("src/app_impl/selection_fallback.rs")
            .expect("read src/app_impl/selection_fallback.rs");
        assert!(
            selection_source.contains("pub(crate) fn try_navigate_root_file_directory_with_tab")
                && selection_source.contains("selected_root_directory_query_owned")
                && selection_source.contains("root_file_parent_query_for_filter")
                && selection_source.contains("set_filter_text_immediate"),
            "ScriptList Tab navigation should be centralized in selection_fallback.rs"
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
    fn root_file_open_records_frecency_after_success_through_shared_open_helper() {
        let source = fs::read_to_string("src/app_impl/selection_fallback.rs")
            .expect("read src/app_impl/selection_fallback.rs");
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            normalized.contains("fn record_root_file_open_use(")
                && normalized.contains("record_use(&format!(\"file/{}\", file.path))")
                && normalized.contains("self.frecency_store.save()")
                && normalized.contains("self.invalidate_grouped_cache();"),
            "root file frecency should be centralized in a helper that records file/<path>, saves, and invalidates grouping"
        );

        let open_body = source
            .split("pub(crate) fn execute_root_file_open(")
            .nth(1)
            .and_then(|section| {
                section
                    .split("pub(crate) fn root_file_search_in_folder_query")
                    .next()
            })
            .expect("execute_root_file_open source should be present");
        let open_call = open_body
            .find("crate::file_search::open_file(&file.path)")
            .expect("execute_root_file_open should call open_file");
        let record_call = open_body
            .find("self.record_root_file_open_use(file);")
            .expect("execute_root_file_open should record frecency after successful open");
        let close_call = open_body
            .find("self.close_and_reset_window(cx);")
            .expect("execute_root_file_open should close after recording");

        assert!(
            open_call < record_call && record_call < close_call,
            "root file frecency should record after a successful OS open and before closing"
        );
        assert!(
            normalized.contains("scripts::SearchResult::File(_) => None"),
            "execute_selected should not pre-record root file frecency before open success"
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
    fn root_recent_files_keep_grouping_search_pure() {
        for path in [
            "src/scripts/grouping.rs",
            "src/scripts/grouping/search_mode.rs",
            "src/scripts/types.rs",
        ] {
            let source = fs::read_to_string(path).unwrap_or_else(|_| panic!("read {path}"));
            let production = production_source(&source);
            for forbidden in [
                "mdfind",
                "search_files(",
                "search_files_streaming",
                "std::process::Command",
                "std::fs::read_dir",
                "list_directory",
            ] {
                assert!(
                    !production.contains(forbidden),
                    "{path} should not start filesystem providers while grouping or ranking: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn root_recent_files_are_file_rows_not_fallbacks() {
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let recent_source = grouping_source
            .split("fn append_recent_root_file_section(")
            .nth(1)
            .and_then(|section| section.split("fn append_root_file_section(").next())
            .expect("append_recent_root_file_section source should be present");

        assert!(
            recent_source.contains("\"Recent Files\""),
            "empty root recent files should render under a Recent Files section"
        );
        assert!(
            recent_source.contains("flat_results.push(SearchResult::File(")
                && !recent_source.contains("SearchResult::Fallback("),
            "recent root files should be real file rows, not fallback rows"
        );
        assert!(
            !recent_source.contains("root_file_search_handoff_result"),
            "empty recent files should not create the Search Files continuation row"
        );
    }

    #[test]
    fn root_recent_files_hydrate_from_frecency_in_app_layer() {
        let file_search_source =
            fs::read_to_string("src/file_search/mod.rs").expect("read src/file_search/mod.rs");
        let root_source = fs::read_to_string("src/app_impl/root_file_search.rs")
            .expect("read src/app_impl/root_file_search.rs");
        let filtering_source = fs::read_to_string("src/app_impl/filtering_cache.rs")
            .expect("read src/app_impl/filtering_cache.rs");
        let root_normalized = root_source.split_whitespace().collect::<Vec<_>>().join(" ");
        let filtering_normalized = filtering_source
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        assert!(
            file_search_source.contains("ROOT_FILE_RECENT_RENDER_LIMIT")
                && file_search_source.contains("ROOT_FILE_RECENT_SEED_LIMIT")
                && file_search_source.contains("ROOT_FILE_RECENT_HYDRATE_LIMIT"),
            "root recent files should separate visible render cap from searchable seed pool"
        );
        assert!(
            root_normalized
                .contains("top_file_paths(crate::file_search::ROOT_FILE_RECENT_HYDRATE_LIMIT)")
                && root_normalized.contains("file_result_from_existing_path(&path)")
                && root_normalized.contains("take(crate::file_search::ROOT_FILE_RECENT_SEED_LIMIT)")
                && root_normalized.contains("self.root_recent_file_results = next_results"),
            "recent root files should hydrate known frecency paths into a deeper seed pool in the app layer"
        );
        assert!(
            filtering_normalized.contains("RootFileSectionMode::GlobalQuery")
                && filtering_normalized.contains("self.refresh_root_recent_file_results();")
                && filtering_normalized.contains("&self.root_recent_file_results"),
            "empty and non-empty global root grouping should refresh and pass recent file rows explicitly"
        );
    }

    #[test]
    fn root_recent_file_seed_pool_exceeds_empty_render_cap() {
        let grouping_source =
            fs::read_to_string("src/scripts/grouping.rs").expect("read src/scripts/grouping.rs");
        let production = production_source(&grouping_source);
        let recent_source = production
            .split("fn append_recent_root_file_section(")
            .nth(1)
            .and_then(|section| section.split("fn append_root_file_section(").next())
            .expect("append_recent_root_file_section source should be present");
        let merge_helper = production
            .split("fn merge_root_global_file_results_with_recent(")
            .nth(1)
            .and_then(|section| section.split("fn root_file_section_title(").next())
            .expect("merge_root_global_file_results_with_recent source should be present");

        assert!(
            recent_source.contains("take(crate::file_search::ROOT_FILE_RECENT_RENDER_LIMIT)"),
            "empty-root Recent Files should stay visually capped even when the cached seed pool is deeper"
        );
        assert!(
            !merge_helper.contains("ROOT_FILE_RECENT_RENDER_LIMIT"),
            "non-empty global recent seeds should search the deeper recent pool, not just visible empty-root rows"
        );
        assert!(
            merge_helper.contains("root_file_recent_seed_matches_query(file, filter_text)"),
            "non-empty global recent seeds should keep bounded recent-seed eligibility"
        );
    }

    #[test]
    fn root_launcher_directory_browse_does_not_open_dedicated_file_search_directly() {
        let source = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("read src/render_script_list/mod.rs");
        let selection_source = fs::read_to_string("src/app_impl/selection_fallback.rs")
            .expect("read src/app_impl/selection_fallback.rs");

        assert!(
            !source.contains("ROOT_FILE_SEARCH_IN_FOLDER_ACTION_ID"),
            "root folder handoff should live in the action executor, not ScriptList key handling"
        );
        assert!(
            !source.contains("open_file_search("),
            "ScriptList render/key handling should not directly open File Search for root folder rows"
        );
        assert!(
            selection_source.contains("try_navigate_root_file_directory_with_tab")
                && selection_source.contains("set_filter_text_immediate"),
            "root directory Tab should update the ScriptList query inline instead of opening dedicated File Search"
        );
    }
}
