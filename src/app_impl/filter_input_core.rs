use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ScriptListSpecialEntry {
    FileSearchMini { query: String },
    QuickTerminal,
    ActionsHelp,
}

impl ScriptListApp {
    /// Transient first-character launch triggers should not persist when the
    /// user returns to the ScriptList surface.
    pub(crate) fn is_transient_script_list_trigger(new_text: &str) -> bool {
        matches!(new_text, "~" | ">" | "?")
    }

    /// Parse `raw` through the menu-syntax classifier and store the result in
    /// [`ScriptListApp::menu_syntax_mode`]. Called from every input-change
    /// boundary (per-keystroke `handle_filter_input_change`, programmatic
    /// `set_filter_text_immediate`) so result grouping and execution see a
    /// snapshot tied to the current raw input instead of racing the filter
    /// coalescer's `computed_filter_text` field.
    pub(crate) fn set_menu_syntax_mode_from_filter(&mut self, raw: &str) {
        let capture_targets =
            crate::menu_syntax::registered_capture_targets_from_scripts(&self.scripts);
        self.menu_syntax_mode = crate::menu_syntax::MenuSyntaxMode::from_input_with_capture_targets(
            raw,
            &capture_targets,
        );
    }

    pub(crate) fn current_view_uses_shared_filter_input(&self) -> bool {
        matches!(
            self.current_view,
            AppView::ScriptList
                | AppView::ClipboardHistoryView { .. }
                | AppView::EmojiPickerView { .. }
                | AppView::AppLauncherView { .. }
                | AppView::WindowSwitcherView { .. }
                | AppView::BrowserTabsView { .. }
                | AppView::DesignGalleryView { .. }
                | AppView::FooterGalleryView { .. }
                | AppView::ThemeChooserView { .. }
                | AppView::FileSearchView { .. }
                | AppView::ProcessManagerView { .. }
                | AppView::SettingsView { .. }
                | AppView::CurrentAppCommandsView { .. }
                | AppView::SearchAiPresetsView { .. }
                | AppView::AcpHistoryView { .. }
                | AppView::BrowserHistoryView { .. }
                | AppView::DictationHistoryView { .. }
                | AppView::NotesBrowseView { .. }
        )
    }

    pub(crate) fn source_filter_mode_blocks_input_history_recall(&self) -> bool {
        matches!(self.current_view, AppView::ScriptList)
            && self
                .menu_syntax_mode
                .advanced_query_for(&self.filter_text)
                .is_some_and(|query| query.has_source_filters())
    }

    pub(crate) fn single_line_filter_input_contains_newline(new_text: &str) -> bool {
        new_text.contains('\n') || new_text.contains('\r')
    }

    pub(crate) fn sync_builtin_query_state(
        query: &mut String,
        selected_index: &mut usize,
        new_text: &str,
    ) -> bool {
        if query == new_text {
            return false;
        }

        *query = new_text.to_string();
        *selected_index = 0;
        true
    }

    pub(crate) fn clear_builtin_query_state(query: &mut String, selected_index: &mut usize) {
        query.clear();
        *selected_index = 0;
    }

    /// Returns `true` when the typed text should hand off from ScriptList into
    /// mini file search (the `~` trigger).
    pub(crate) fn should_enter_file_search_from_script_list(new_text: &str) -> bool {
        new_text == "~" || new_text.starts_with("~/")
    }

    /// Normalises the mini file-search query: bare `~` becomes `~/` so the
    /// directory listing starts immediately.
    pub(crate) fn normalize_mini_file_search_query(new_text: &str) -> String {
        if new_text == "~" {
            "~/".to_string()
        } else {
            new_text.to_string()
        }
    }

    /// Classify narrow, first-character ScriptList handoffs into dedicated
    /// surfaces so regular search queries are not hijacked.
    pub(crate) fn special_entry_from_script_list_filter(
        new_text: &str,
    ) -> Option<ScriptListSpecialEntry> {
        if Self::should_enter_file_search_from_script_list(new_text) {
            return Some(ScriptListSpecialEntry::FileSearchMini {
                query: Self::normalize_mini_file_search_query(new_text),
            });
        }

        match new_text {
            ">" => Some(ScriptListSpecialEntry::QuickTerminal),
            "?" => Some(ScriptListSpecialEntry::ActionsHelp),
            _ => None,
        }
    }

    /// Choose the correct resize path for file search based on presentation mode.
    pub(crate) fn resize_file_search_window_for_presentation(
        presentation: FileSearchPresentation,
        result_count: usize,
    ) {
        match presentation {
            FileSearchPresentation::Mini => {
                crate::window_resize::resize_to_mini_file_search_window_sync(result_count);
            }
            FileSearchPresentation::Full => resize_to_view_sync(ViewType::ExpandedMainWindow, 0),
        }
    }

    /// Resize only when the mini explorer's row count change should actually
    /// affect window height.  Full presentation is fixed-size and should not
    /// re-enter synchronous resize work on every stream batch.
    pub(crate) fn resize_file_search_window_after_results_change(
        presentation: FileSearchPresentation,
        result_count: usize,
        is_first_batch: bool,
        is_done: bool,
    ) {
        match presentation {
            FileSearchPresentation::Mini if is_first_batch || is_done => {
                crate::window_resize::resize_to_mini_file_search_window_sync(result_count);
            }
            FileSearchPresentation::Full if is_first_batch => {
                resize_to_view_sync(ViewType::ExpandedMainWindow, 0);
            }
            _ => {} // skip intermediate batch resizes
        }
    }

    // ── Spine projection helpers ──────────────────────────────────────

    /// Reparse the Spine model from the current filter text and cursor position.
    pub(crate) fn set_spine_parse_from_filter_and_cursor(&mut self, raw: &str, cursor: usize) {
        self.spine_parse = crate::spine::parse_spine(raw);
        self.spine_projection = Some(crate::spine::project_cursor(&self.spine_parse, cursor));
    }

    /// Whether the Spine projection currently owns the main list (i.e., a sigil
    /// segment is active and should replace the normal unified search results).
    pub(crate) fn spine_projection_owns_main_list(&self) -> bool {
        if !self.spine_enabled {
            return false;
        }
        match &self.spine_projection {
            Some(proj) => {
                !matches!(
                    proj.active_segment_kind,
                    crate::spine::SpineSegmentKind::FreeText
                ) || (proj.is_tail
                    && proj.has_prompt_segments
                    && crate::spine::parse_has_prompt_builder_segments(&self.spine_parse))
            }
            None => false,
        }
    }

    /// Shared helper that opens file search in the given presentation mode.
    /// Used by both the builtin "Search Files" entry (Full) and the `~`
    /// trigger from ScriptList (Mini).
    ///
    /// Directory mini-entry from ScriptList seeds its first rows before
    /// switching surfaces; other opens paint loading state and stream results
    /// asynchronously via `restart_file_search_stream_for_query`.
    pub(crate) fn open_file_search_view(
        &mut self,
        query: String,
        presentation: FileSearchPresentation,
        cx: &mut Context<Self>,
    ) {
        self.open_file_search_view_with_result_transition(query, presentation, false, cx);
    }

    /// Browse within file search without blanking the current directory view
    /// before the next directory stream has produced its first batch.
    pub(crate) fn open_file_search_view_preserving_current_results(
        &mut self,
        query: String,
        presentation: FileSearchPresentation,
        cx: &mut Context<Self>,
    ) {
        self.open_file_search_view_with_result_transition(query, presentation, true, cx);
    }

    fn seed_file_search_directory_results_for_first_paint(&mut self, query: &str) -> bool {
        let Some(parsed) = crate::file_search::parse_directory_path(query) else {
            return false;
        };

        let results = crate::file_search::list_directory_with_options(
            &parsed.directory,
            crate::file_search::DEFAULT_CACHE_LIMIT,
            parsed.show_hidden,
        );

        if results.is_empty() {
            tracing::info!(
                category = "FILE_SEARCH",
                query,
                directory = %parsed.directory,
                "Mini file-search first-paint seed found no directory rows"
            );
            return false;
        }

        self.cached_file_results = results;
        self.file_search_current_dir = Some(parsed.directory.clone());
        self.file_search_current_dir_show_hidden = parsed.show_hidden;
        self.file_search_frozen_filter = None;
        self.apply_file_search_sort_mode();
        self.recompute_file_search_display_indices();
        self.restore_file_search_selection_after_results_change(None);

        tracing::info!(
            category = "FILE_SEARCH",
            query,
            directory = %parsed.directory,
            cached_count = self.cached_file_results.len(),
            display_count = self.file_search_display_indices.len(),
            "Seeded mini file-search directory rows before first paint"
        );

        true
    }

    fn seed_file_search_default_results_for_first_paint(&mut self) -> bool {
        let mut results = self
            .recent_file_results_from_frecency(crate::file_search::ROOT_FILE_RECENT_HYDRATE_LIMIT);
        let source = if results.is_empty() {
            results = crate::file_search::list_directory_with_options(
                "~/",
                crate::file_search::DEFAULT_CACHE_LIMIT,
                false,
            );
            "home_directory"
        } else {
            "frecency_recent_files"
        };

        if results.is_empty() {
            tracing::info!(
                category = "FILE_SEARCH",
                "Default file-search first-paint seed found no recent or home-directory rows"
            );
            return false;
        }

        self.cached_file_results = results;
        self.file_search_current_dir = None;
        self.file_search_current_dir_show_hidden = false;
        self.file_search_frozen_filter = None;
        self.recompute_file_search_display_indices();
        self.restore_file_search_selection_after_results_change(None);
        self.file_search_loading = false;

        tracing::info!(
            category = "FILE_SEARCH",
            source,
            cached_count = self.cached_file_results.len(),
            display_count = self.file_search_display_indices.len(),
            "Seeded default file-search rows before first paint"
        );

        true
    }

    fn open_file_search_view_with_result_transition(
        &mut self,
        query: String,
        presentation: FileSearchPresentation,
        preserve_current_results_until_first_batch: bool,
        cx: &mut Context<Self>,
    ) {
        // Preserve sort mode when already inside file search (e.g. browsing
        // into a subdirectory) — only reset on fresh entry from outside.
        let preserve_sort_mode = matches!(self.current_view, AppView::FileSearchView { .. });

        tracing::info!(
            category = "FILE_SEARCH",
            %query,
            ?presentation,
            preserve_sort_mode,
            preserve_current_results_until_first_batch,
            "Opening file search view"
        );

        self.filter_text = query.clone();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some("Search files...".to_string());
        self.gpui_input_state.update(cx, |state, _cx| {
            state.clear_highlight_ranges();
        });

        self.current_view = AppView::FileSearchView {
            query: query.clone(),
            selected_index: 0,
            presentation,
        };
        self.hovered_index = None;
        self.opened_from_main_menu = true;

        self.pending_focus = Some(FocusTarget::MainFilter);
        self.focused_input = FocusedInput::MainFilter;

        if !preserve_sort_mode {
            self.file_search_sort_mode = crate::actions::FileSearchSortMode::default();
        }

        let stabilize_fresh_mini_directory_entry = !preserve_current_results_until_first_batch
            && !preserve_sort_mode
            && presentation == FileSearchPresentation::Mini;
        let seeded_initial_results = stabilize_fresh_mini_directory_entry
            && self.seed_file_search_directory_results_for_first_paint(&query);
        let seed_fresh_full_default = !preserve_current_results_until_first_batch
            && !preserve_sort_mode
            && presentation == FileSearchPresentation::Full
            && query.trim().is_empty();
        let seeded_default_results =
            seed_fresh_full_default && self.seed_file_search_default_results_for_first_paint();
        let seeded_initial_results = seeded_initial_results || seeded_default_results;

        if !preserve_current_results_until_first_batch && !seeded_initial_results {
            self.cached_file_results.clear();
            self.file_search_display_indices.clear();
            self.file_search_current_dir = None;
            self.file_search_current_dir_show_hidden = false;
        }
        self.file_search_frozen_filter = None;
        self.file_search_selection_mode = FileSearchSelectionMode::AutoFirst;

        // Fresh opens paint their empty/loading state immediately. Internal
        // directory browsing keeps the previous rows and size until the next
        // stream's first batch replaces them.
        if !preserve_current_results_until_first_batch {
            Self::resize_file_search_window_for_presentation(
                presentation,
                self.file_search_display_indices.len(),
            );
        }

        if seeded_default_results {
            let _ = self.begin_file_search_session();
            cx.notify();
            return;
        }

        let preserve_stream_results =
            preserve_current_results_until_first_batch || seeded_initial_results;
        self.restart_file_search_stream_for_query(
            query,
            presentation,
            None,
            preserve_stream_results,
            cx,
        );
    }

    pub(crate) fn accept_ghost_prediction(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> bool {
        let prediction = match self.ghost_prediction.take() {
            Some(p) => p,
            None => return false,
        };
        let full_label = prediction.full_label.clone();
        self.pending_programmatic_filter_echo = Some(full_label.clone());
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_value(full_label.clone().into(), window, cx);
        });
        self.ghost_prediction = None;
        cx.notify();
        true
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn test_current_view_uses_shared_filter_input_includes_script_list_and_builtin_views() {
        let source = fs::read_to_string("src/app_impl/filter_input_core.rs")
            .expect("Failed to read src/app_impl/filter_input_core.rs");
        let required_views = [
            "AppView::ScriptList",
            "AppView::ClipboardHistoryView",
            "AppView::EmojiPickerView",
            "AppView::AppLauncherView",
            "AppView::WindowSwitcherView",
            "AppView::BrowserTabsView",
            "AppView::DesignGalleryView",
            "AppView::FooterGalleryView",
            "AppView::ThemeChooserView",
            "AppView::FileSearchView",
            "AppView::SettingsView",
            "AppView::BrowserHistoryView",
        ];

        for view in required_views {
            assert!(
                source.contains(view),
                "current_view_uses_shared_filter_input must include {}",
                view
            );
        }
    }

    #[test]
    fn test_should_enter_file_search_from_script_list() {
        use super::ScriptListApp;
        assert!(ScriptListApp::should_enter_file_search_from_script_list(
            "~"
        ));
        assert!(ScriptListApp::should_enter_file_search_from_script_list(
            "~/src"
        ));
        assert!(!ScriptListApp::should_enter_file_search_from_script_list(
            "foo"
        ));
        assert!(!ScriptListApp::should_enter_file_search_from_script_list(
            "/tmp"
        ));
        assert!(!ScriptListApp::should_enter_file_search_from_script_list(
            ""
        ));
    }

    #[test]
    fn test_normalize_mini_file_search_query() {
        use super::ScriptListApp;
        assert_eq!(ScriptListApp::normalize_mini_file_search_query("~"), "~/");
        assert_eq!(
            ScriptListApp::normalize_mini_file_search_query("~/src"),
            "~/src"
        );
    }

    #[test]
    fn test_special_entry_from_script_list_filter() {
        use super::{ScriptListApp, ScriptListSpecialEntry};

        assert_eq!(
            ScriptListApp::special_entry_from_script_list_filter("~"),
            Some(ScriptListSpecialEntry::FileSearchMini {
                query: "~/".to_string()
            })
        );
        assert_eq!(
            ScriptListApp::special_entry_from_script_list_filter("~/src"),
            Some(ScriptListSpecialEntry::FileSearchMini {
                query: "~/src".to_string()
            })
        );
        assert_eq!(
            ScriptListApp::special_entry_from_script_list_filter("/"),
            None,
        );
        assert_eq!(
            ScriptListApp::special_entry_from_script_list_filter("@"),
            None,
        );
        assert_eq!(
            ScriptListApp::special_entry_from_script_list_filter(">"),
            Some(ScriptListSpecialEntry::QuickTerminal)
        );
        assert_eq!(
            ScriptListApp::special_entry_from_script_list_filter("?"),
            Some(ScriptListSpecialEntry::ActionsHelp)
        );
        assert_eq!(
            ScriptListApp::special_entry_from_script_list_filter("/tmp"),
            None
        );
        assert_eq!(
            ScriptListApp::special_entry_from_script_list_filter("@browser"),
            None
        );
        assert_eq!(
            ScriptListApp::special_entry_from_script_list_filter("foo"),
            None
        );
    }

    #[test]
    fn test_is_transient_script_list_trigger() {
        use super::ScriptListApp;

        for trigger in ["~", "/", "@", "|", ">", "?"] {
            assert!(
                ScriptListApp::is_transient_script_list_trigger(trigger),
                "expected '{trigger}' to be treated as a transient ScriptList trigger"
            );
        }

        for query in ["~/src", "@browser", "/tmp", "foo", ""] {
            assert!(
                !ScriptListApp::is_transient_script_list_trigger(query),
                "expected '{query}' to remain a real query"
            );
        }
    }

    #[test]
    fn test_power_syntax_prefixes_do_not_route_to_special_surfaces() {
        use super::ScriptListApp;

        for query in [
            ":type:script shortcut:true",
            ":",
            ";todo Renew passport tomorrow",
            "+",
            "+xyz unknown target",
            "todo: Renew passport tomorrow",
            "cal: Lunch next friday",
            ">deploy -- prod",
            "!",
            "#finance search",
        ] {
            assert_eq!(
                ScriptListApp::special_entry_from_script_list_filter(query),
                None,
                "power-user syntax '{query}' must not route through special_entry_from_script_list_filter"
            );
        }
    }

    #[test]
    fn test_power_syntax_prefixes_are_not_transient_triggers() {
        use super::ScriptListApp;

        for prefix in [":", "+", "!", "#"] {
            assert!(
                !ScriptListApp::is_transient_script_list_trigger(prefix),
                "power-user prefix '{prefix}' must not be classified as a transient trigger"
            );
        }
    }

    #[test]
    fn test_single_line_filter_input_contains_newline_detects_enter_leaks() {
        use super::ScriptListApp;

        assert!(!ScriptListApp::single_line_filter_input_contains_newline(
            "do in"
        ));
        assert!(ScriptListApp::single_line_filter_input_contains_newline(
            "do in\n"
        ));
        assert!(ScriptListApp::single_line_filter_input_contains_newline(
            "do in\r"
        ));
        assert!(ScriptListApp::single_line_filter_input_contains_newline(
            "do in\r\n"
        ));
    }
}
