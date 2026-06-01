use super::*;
impl ScriptListApp {
    #[inline]
    fn filter_change_can_affect_window_size(&self) -> bool {
        // Mini ScriptList height depends on grouped results.
        // Non-ScriptList views may also size from filtered item counts.
        // Full ScriptList uses the normal fixed launcher size, so typing
        // should not recalculate/defer resize every keystroke.
        !matches!(self.current_view, AppView::ScriptList) || self.main_window_mode == MainWindowMode::Mini
    }

    pub(crate) fn cancel_history_filter_render_pending_if_obsolete(&mut self, next_filter: &str) {
        if self
            .history_filter_render_pending
            .as_deref()
            .is_some_and(|pending| pending != next_filter)
        {
            tracing::info!(
                target: "script_kit::input_history",
                event = "history_filter_render_pending_cancelled_obsolete",
                next_filter_len = next_filter.len(),
                history_index = ?self.input_history.current_index(),
                selected_index = self.selected_index,
            );
            self.history_filter_render_pending = None;
        }
    }

    /// Single authoritative post-filter reconciliation path for ScriptList.
    ///
    /// Called after `computed_filter_text` changes (both debounced and immediate).
    /// Syncs the GPUI list model, resets selection to the first selectable row,
    /// reveals it, and rebuilds preflight — all outside `render()`.
    pub(crate) fn reconcile_script_list_after_filter_change(
        &mut self,
        reason: &'static str,
        cx: &mut Context<Self>,
    ) {
        if !matches!(self.current_view, AppView::ScriptList) {
            return;
        }

        // Keep GPUI's list model aligned with the newly computed grouped results.
        // Filter changes may replace every row while preserving the same count,
        // so force measured item rebuilding instead of only syncing count.
        self.sync_list_state_for_filter_replacement();

        // Filter changes intentionally restart from the first selectable row.
        self.selected_index = 0;
        self.validate_selection_bounds(cx);

        // Clear last_scrolled_index so the reveal is never skipped —
        // filter changes always need a fresh scroll even if selected_index == 0.
        self.last_scrolled_index = None;

        // Reveal the final selected row after selection coercion.
        self.scroll_to_selected_if_needed(reason);

        // Preflight depends on filter + selection and must stay out of render().
        // The immediate typing path restores root-file handoff selection after
        // reconciliation, so it rebuilds once after the final row is known.
        if reason != "filter_immediate" {
            self.rebuild_main_window_preflight_if_needed();
        }

        self.refresh_ghost_with_input(cx);
    }

    pub(crate) fn queue_filter_compute(&mut self, value: String, cx: &mut Context<Self>) {
        if self.computed_filter_text == value {
            tracing::debug!(
                target: "script_kit::filter",
                event = "queue_filter_compute_exact_query_noop",
                filter_len = value.len(),
            );
            return;
        }
        if logging::filter_perf_trace_enabled() {
            logging::log(
                "FILTER_PERF",
                &format!("[2/5] APPLY_FILTER value='{}' len={}", value, value.len()),
            );
        }
        if self.computed_filter_text != value {
            let update_start = std::time::Instant::now();
            let selection_before = if matches!(self.current_view, AppView::ScriptList) {
                Some(self.main_menu_selection_snapshot())
            } else {
                None
            };
            self.filter_coalescer.reset();
            self.computed_filter_text = value.clone();
            self.maybe_start_root_file_search(&value, cx);
            self.maybe_start_root_windows_refresh_for_query(&value, cx);
            self.maybe_start_root_browser_tabs_refresh_for_query(&value, cx);
            self.maybe_start_root_browser_history_refresh_for_query(&value, cx);
            self.reconcile_script_list_after_filter_change("filter_immediate", cx);
            if let Some(snapshot) = selection_before.as_ref() {
                if self.restore_root_file_handoff_selection_from_snapshot(snapshot) {
                    self.scroll_to_selected_if_needed("filter_immediate_restore_root_file_handoff");
                }
            }
            self.rebuild_main_window_preflight_if_needed();
            if self.filter_change_can_affect_window_size() {
                self.update_window_size();
            }
            let update_elapsed = update_start.elapsed();
            if logging::filter_perf_trace_enabled()
                || update_elapsed >= std::time::Duration::from_millis(8)
            {
                logging::log(
                    "FILTER_PERF",
                    &format!(
                        "[3/5] APPLY_FILTER_DONE in {:.2}ms for '{}'",
                        update_elapsed.as_secs_f64() * 1000.0,
                        value
                    ),
                );
            }
            cx.notify();
        }
    }

    /// Apply a filter text change synchronously, without coalescer delay.
    ///
    /// Verbatim-echo contract (Run 4 Pass #8 attacker probe
    /// `stdin-setfilter-inputvalue-unbounded`, closed Run 8 Pass #23):
    /// `text` is stored into `self.filter_text` with no length cap,
    /// truncation, or encoding transformation — whatever the stdin
    /// `setFilter` command supplied arrives in `getState.inputValue`
    /// byte-for-byte. The only enforced bound is the stdin line cap at
    /// `MAX_STDIN_COMMAND_BYTES` (16 * 1024 bytes), applied by
    /// `read_stdin_line_bounded` in `src/stdin_commands/mod.rs:1003`.
    /// Callers consuming `getState.inputValue` MUST handle payloads up
    /// to that cap. Pinned by
    /// `tests/stdin_setfilter_input_value_verbatim_contract.rs`.
    pub(crate) fn set_filter_text_immediate(
        &mut self,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // The filter input is single-line; GPUI's text shaper panics on
        // newlines (`vendor/gpui/src/text_system.rs:414`). Sanitize early so
        // pasted multi-line content cannot crash the app.
        let text = if text.chars().any(|c| matches!(c, '\n' | '\r')) {
            text.replace("\r\n", " ").replace(['\n', '\r'], " ")
        } else {
            text
        };

        self.pending_menu_syntax_ai_proposal = None;

        if let AppView::AcpChatView { entity } = &self.current_view {
            self.suppress_filter_events = true;
            self.filter_text = text.clone();
            self.pending_programmatic_filter_echo = Some(text.clone());
            self.gpui_input_state.update(cx, |state, cx| {
                state.set_highlight_ranges_with_roles(Vec::new());
                state.set_value(text.clone(), window, cx);
                let len = text.len();
                state.set_selection(len, len, window, cx);
            });
            self.suppress_filter_events = false;
            self.pending_filter_sync = false;
            entity.update(cx, |chat, cx| {
                chat.set_input(text.clone(), cx);
                chat.refresh_acp_spine_from_composer(cx);
            });
            cx.notify();
            return;
        }

        let input_already_matches = self.gpui_input_state.read(cx).value().to_string() == text;
        if matches!(self.current_view, AppView::ScriptList)
            && self.filter_text == text
            && self.computed_filter_text == text
            && input_already_matches
            && !self.pending_filter_sync
        {
            self.pending_programmatic_filter_echo = None;
            tracing::debug!(
                target: "script_kit::filter",
                event = "set_filter_text_immediate_exact_query_noop",
                filter_len = text.len(),
            );
            return;
        }

        self.suppress_filter_events = true;
        self.filter_text = text.clone();
        self.pending_programmatic_filter_echo = Some(text.clone());
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_highlight_ranges_with_roles(Vec::new());
            state.set_value(text.clone(), window, cx);
            let len = text.len();
            state.set_selection(len, len, window, cx);
        });
        self.suppress_filter_events = false;
        self.pending_filter_sync = false;

        // Route filter to the active subview's variant field when current_view
        // is a builtin subview (ClipboardHistoryView, EmojiPickerView, etc.).
        // Without this, stdin `setFilter` on a subview would only update
        // `self.filter_text` and leave the subview's own `filter` field stale,
        // so `getState.visibleChoiceCount` (computed from the variant's filter)
        // would never reflect the narrowed dataset. Sub-gap (2) of the
        // `empty-clipboard-state` story.
        let handled_by_subview = self.write_filter_to_current_subview(&text);

        // Menu bar items are now pre-fetched by frontmost_app_tracker
        // No lazy loading needed - items are already in cache when we open

        // stdin `setFilter` on FileSearchView needs to drive the file-search
        // stream the same way real keystrokes do (the GPUI handler at
        // `handle_filter_input_change` line ~511 is suppressed here). Open
        // the view at the new query so directory navigation works under
        // protocol automation.
        if !handled_by_subview
            && matches!(self.current_view, AppView::FileSearchView { .. })
        {
            let presentation = if let AppView::FileSearchView { presentation, .. } =
                &self.current_view
            {
                *presentation
            } else {
                FileSearchPresentation::Full
            };
            self.open_file_search_view_preserving_current_results(
                text.clone(),
                presentation,
                cx,
            );
            return;
        }

        let mut handler_form_owns_input = false;
        if !handled_by_subview && matches!(self.current_view, AppView::ScriptList) {
            if let Some(entry) = Self::special_entry_from_script_list_filter(&text) {
                if self.route_script_list_special_entry(entry, &text, window, cx) {
                    return;
                }
            }
            self.set_menu_syntax_mode_from_filter(&text);
            if self.spine_enabled {
                self.set_spine_parse_from_filter_and_cursor(&text, text.len());
                self.maybe_start_spine_file_subsearch_for_current_projection(cx);
                let has_cwd_segment = self.spine_parse.segments.iter().any(|s| {
                    matches!(s.kind, crate::spine::SpineSegmentKind::ProjectCwd { .. })
                        && matches!(
                            s.resolution,
                            crate::spine::SpineSegmentResolution::Resolved { .. }
                        )
                });
                // Note: CWD is no longer auto-cleared when the parsed input
                // lacks a `>:` segment. The CWD now lives in the footer chip
                // (set on Enter against a directory row) and is independent
                // of the input bar. The user changes it by typing `>` again
                // and picking a different directory, or by clicking the
                // chip.
                let _ = has_cwd_segment;
            }
            handler_form_owns_input = self.menu_syntax_capture_form_owns_input_for(&text);
            self.sync_menu_syntax_form_inputs_from_filter(window, cx);
            if handler_form_owns_input {
                self.menu_syntax_object_selector_state = Default::default();
                self.menu_syntax_trigger_popup_state = Default::default();
                crate::menu_syntax_object_selector_popup_window::close_menu_syntax_object_selector_popup_window(cx);
                crate::menu_syntax_trigger_popup_window::close_menu_syntax_trigger_popup_window(cx);
                self.sync_menu_syntax_form_inputs_from_filter(window, cx);
            } else {
                self.run_menu_syntax_object_selector_state_machine(&text, window, cx);
            }
            if !handler_form_owns_input && self.menu_syntax_object_selector_state.snapshot.is_none()
            {
                self.run_menu_syntax_trigger_popup_state_machine(&text, window, cx);
            }
            self.invalidate_grouped_cache();
        } else {
            self.menu_syntax_mode = crate::menu_syntax::MenuSyntaxMode::default();
            self.sync_menu_syntax_form_inputs_from_filter(window, cx);
        }

        if self.menu_syntax_mode.is_menu_syntax_for(&text)
            || self.menu_syntax_trigger_popup_state.snapshot.is_some()
            || self.menu_syntax_object_selector_state.snapshot.is_some()
            || self.menu_syntax_capture_form_owns_input_for(&text)
        {
            // Menu syntax owns the result list entirely — clear any stale
            // fallback items so pressing Enter routes to execute_selected,
            // not execute_selected_fallback. Also clear when the trigger
            // popup is open for a partial trigger like `;t` (where
            // `is_menu_syntax_for` still returns false because the parser
            // doesn't yet recognize `;t` as a full target).
            self.main_menu_fallback_state.clear();
        }

        self.computed_filter_text = text.clone();
        self.filter_coalescer.reset();
        self.maybe_start_root_file_search(&text, cx);
        self.maybe_start_root_windows_refresh_for_query(&text, cx);
        self.maybe_start_root_browser_tabs_refresh_for_query(&text, cx);
        self.maybe_start_root_browser_history_refresh_for_query(&text, cx);
        self.reconcile_script_list_after_filter_change("set_filter_text_immediate", cx);

        // Update fallback state immediately based on filter results
        // This ensures SimulateKey commands can check fallback state correctly
        // NOTE: validate_selection_bounds already clears main_menu_fallback_state,
        // but we need special handling for legacy SimulateKey compatibility.
        // Skip when a subview handled the filter: `get_filtered_results_cached`
        // and `collect_fallbacks` are ScriptList-only and would incorrectly
        // flip a builtin subview into the script-list fallback mode.
        if !handled_by_subview && !text.is_empty() {
            if !handler_form_owns_input
                && !self.menu_syntax_mode.is_menu_syntax_for(&text)
                && self.menu_syntax_trigger_popup_state.snapshot.is_none()
                && self.menu_syntax_object_selector_state.snapshot.is_none()
            {
                let results = self.get_filtered_results_cached();
                if results.is_empty() {
                    // No matches - check if we should enter fallback mode
                    use crate::fallbacks::collect_fallbacks;
                    let fallbacks = collect_fallbacks(&text, self.scripts.as_slice());
                    if !fallbacks.is_empty() {
                        self.main_menu_fallback_state.replace_items(fallbacks);
                    }
                }
            }
        }

        self.rebuild_main_window_preflight_if_needed();
        if self.filter_change_can_affect_window_size() {
            self.update_window_size_deferred(window, cx);
        }
        cx.notify();
    }

    pub(crate) fn handle_script_list_printable_simulate_key(
        &mut self,
        key_char: Option<&str>,
        modifiers: &gpui::Modifiers,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !matches!(self.current_view, AppView::ScriptList) {
            return false;
        }
        if modifiers.platform || modifiers.alt || modifiers.control {
            return false;
        }
        if self.menu_syntax_form_input_active && self.menu_syntax_capture_form_owns_input() {
            return false;
        }
        let Some(ch) = key_char else {
            return false;
        };
        if ch.is_empty() || ch.chars().count() != 1 {
            return false;
        }

        let mut next = self.filter_text.clone();
        next.push_str(ch);
        self.set_filter_text_immediate(next, window, cx);
        true
    }

    /// Write the given filter text into the current view's `filter` field
    /// when `current_view` is one of the shared-input builtin subviews.
    ///
    /// Returns `true` when a subview was handled — callers should skip any
    /// ScriptList-only bookkeeping (fallback mode, ranker, etc.) in that case.
    /// Returns `false` for `ScriptList`, `FileSearchView` (dedicated routing
    /// via `restart_file_search_stream_for_query`), and non-filter views.
    pub(crate) fn write_filter_to_current_subview(&mut self, text: &str) -> bool {
        match &mut self.current_view {
            AppView::ClipboardHistoryView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::AppLauncherView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::WindowSwitcherView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::BrowserTabsView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::DesignGalleryView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::FooterGalleryView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::ThemeChooserView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::ProcessManagerView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::SettingsView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::SearchAiPresetsView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::FavoritesBrowseView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::CurrentAppCommandsView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::AcpHistoryView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::BrowserHistoryView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::DictationHistoryView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::NotesBrowseView {
                filter,
                selected_index,
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            AppView::EmojiPickerView {
                filter,
                selected_index,
                ..
            } => {
                Self::sync_builtin_query_state(filter, selected_index, text);
                true
            }
            _ => false,
        }
    }

    pub(crate) fn clear_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.cancel_history_filter_render_pending_if_obsolete("");
        self.set_filter_text_immediate(String::new(), window, cx);
    }

    // ── Spine row acceptance ────────────────────────────────────────────

    /// Accept the currently selected Spine projection row (Enter / click).
    /// Returns `true` if the action was handled.
    pub(crate) fn accept_spine_projection_row(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.spine_projection_owns_main_list() {
            return false;
        }
        // Rich subsearch rows (SearchResult::File, ClipboardHistory) need
        // interception: resolve them into @file:path / @clipboard:id tokens
        // instead of executing default file-open / clipboard-paste behavior.
        if let Some(action) = self.selected_spine_rich_subsearch_action() {
            return self.apply_spine_list_action(action, window, cx);
        }
        let Some(row) = self.selected_spine_projection_row() else {
            tracing::debug!(
                target: "script_kit::spine",
                event = "accept_spine_projection_row_no_selection",
                selected_index = self.selected_index,
            );
            return false;
        };
        let action = row.action.clone();
        tracing::info!(
            target: "script_kit::spine",
            event = "accept_spine_projection_row",
            row_id = %row.id,
            row_title = %row.title,
            selected_index = self.selected_index,
        );
        self.apply_spine_list_action(action, window, cx)
    }

    fn selected_spine_rich_subsearch_action(
        &mut self,
    ) -> Option<crate::spine::SpineListAction> {
        let projection = self.spine_projection.as_ref()?;
        let crate::spine::SpineSegmentKind::ContextMention {
            context_type,
            sub_query,
        } = &projection.active_segment_kind
        else {
            return None;
        };
        let (source, _) = crate::spine::catalog_subsearch::parse_context_subsearch(
            context_type,
            sub_query.as_deref(),
        )?;
        let segment_index = projection.active_segment_index;
        let segment_byte_range = self
            .spine_parse
            .segments
            .get(segment_index)
            .map(|seg| seg.byte_range.clone())?;

        let (grouped, flat) = self.get_grouped_results_cached();
        let result_idx = match grouped.get(self.selected_index)? {
            GroupedListItem::Item(idx) => *idx,
            _ => return None,
        };
        let result = flat.get(result_idx)?;

        match (source, result) {
            (
                crate::spine::catalog_subsearch::ContextSubsearchSource::File,
                scripts::SearchResult::File(file_match),
            ) => {
                let short = crate::file_search::shorten_path(&file_match.file.path);
                let replacement = format!(
                    "@file:{}",
                    crate::spine::catalog_subsearch::escape_ref_component(&short),
                );
                Some(crate::spine::SpineListAction::ResolveSegment {
                    segment_index,
                    segment_byte_range,
                    replacement: replacement.into(),
                    resolution_id: format!("file/{}", file_match.file.path).into(),
                    resolution_label: file_match.file.name.clone().into(),
                    resolution_source: "file".into(),
                    trailing_space: true,
                })
            }
            (
                crate::spine::catalog_subsearch::ContextSubsearchSource::Clipboard,
                scripts::SearchResult::ClipboardHistory(clip_match),
            ) => {
                let replacement = format!(
                    "@clipboard:{}",
                    crate::spine::catalog_subsearch::escape_ref_component(
                        &clip_match.entry.id,
                    ),
                );
                Some(crate::spine::SpineListAction::ResolveSegment {
                    segment_index,
                    segment_byte_range,
                    replacement: replacement.into(),
                    resolution_id: format!("clipboard/{}", clip_match.entry.id).into(),
                    resolution_label: clip_match.title.clone().into(),
                    resolution_source: "clipboard".into(),
                    trailing_space: true,
                })
            }
            _ => None,
        }
    }

    /// Return the `SpineListRow` at the current `selected_index`, if any.
    pub(crate) fn selected_spine_projection_row(&mut self) -> Option<crate::spine::SpineListRow> {
        let (grouped, flat) = self.get_grouped_results_cached();
        let item = grouped.get(self.selected_index)?;
        match item {
            GroupedListItem::Item(result_idx) => {
                if let Some(crate::scripts::SearchResult::SpineProjection(row)) =
                    flat.get(*result_idx)
                {
                    if row.is_selectable {
                        Some(row.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Dispatch a `SpineListAction` from a selected row.
    pub(crate) fn apply_spine_list_action(
        &mut self,
        action: crate::spine::SpineListAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        use crate::spine::SpineListAction;
        match action {
            SpineListAction::InsertSegmentText {
                segment_index,
                segment_byte_range,
                text,
                trailing_space,
            } => {
                tracing::info!(
                    target: "script_kit::spine",
                    event = "apply_spine_action_insert_segment",
                    segment_index,
                    text = %text,
                    trailing_space,
                );
                self.replace_active_segment_text(
                    segment_index,
                    segment_byte_range,
                    text.as_ref(),
                    trailing_space,
                    window,
                    cx,
                )
            }
            SpineListAction::ResolveSegment {
                segment_index,
                segment_byte_range,
                replacement,
                resolution_id,
                resolution_label,
                resolution_source,
                trailing_space,
            } => {
                tracing::info!(
                    target: "script_kit::spine",
                    event = "apply_spine_action_resolve_segment",
                    segment_index,
                    replacement = %replacement,
                    resolution_id = %resolution_id,
                    resolution_label = %resolution_label,
                    resolution_source = %resolution_source,
                    trailing_space,
                );
                if resolution_source.as_ref() == "cwd" {
                    let path = std::path::PathBuf::from(resolution_id.as_ref());
                    self.spine_cwd = Some(path);
                    self.spine_cwd_label = Some(resolution_label.as_ref().to_string());
                    self.spine_cwd_revision = self.spine_cwd_revision.wrapping_add(1);
                    self.persist_spine_cwd();
                    self.invalidate_grouped_cache();
                    // CWD becomes a footer chip — strip the segment text from
                    // the input bar so the user sees a clean prompt builder.
                    self.replace_active_segment_text(
                        segment_index,
                        segment_byte_range,
                        "",
                        false,
                        window,
                        cx,
                    )
                } else {
                    self.replace_active_segment_text(
                        segment_index,
                        segment_byte_range,
                        replacement.as_ref(),
                        trailing_space,
                        window,
                        cx,
                    )
                }
            }
            SpineListAction::OpenModeExit { sigil, rest } => {
                tracing::info!(
                    target: "script_kit::spine",
                    event = "apply_spine_action_open_mode_exit",
                    sigil = %sigil,
                    rest = %rest,
                );
                match sigil {
                    '~' => {
                        self.open_file_search_view(
                            rest.to_string(),
                            FileSearchPresentation::Mini,
                            cx,
                        );
                        true
                    }
                    '!' => {
                        self.open_quick_terminal(None, cx);
                        true
                    }
                    '?' => {
                        if self.has_actions() {
                            self.toggle_actions(cx, window);
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            SpineListAction::OpenConversation { conversation_id } => {
                tracing::info!(
                    target: "script_kit::spine",
                    event = "apply_spine_action_open_conversation_not_wired",
                    conversation_id = %conversation_id,
                );
                false
            }
            SpineListAction::AwaitContextSubsearchInput { source } => {
                tracing::info!(
                    target: "script_kit::spine",
                    event = "empty_context_subsearch_enter_consumed",
                    source = %source,
                );
                true
            }
            SpineListAction::Noop => false,
        }
    }

    /// Replace the text of the active Spine segment in the filter input,
    /// optionally appending a trailing space, and reposition the cursor.
    pub(crate) fn replace_active_segment_text(
        &mut self,
        segment_index: usize,
        segment_byte_range: std::ops::Range<usize>,
        replacement: &str,
        trailing_space: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let current = self.filter_text.clone();

        // Validate byte range against current filter text.
        if !self.valid_filter_byte_range(&current, &segment_byte_range) {
            tracing::debug!(
                target: "script_kit::spine",
                event = "replace_segment_invalid_byte_range",
                range_start = segment_byte_range.start,
                range_end = segment_byte_range.end,
                filter_len = current.len(),
            );
            return false;
        }

        let Some(current_segment) = self.spine_parse.segments.get(segment_index) else {
            tracing::debug!(
                target: "script_kit::spine",
                event = "replace_segment_index_out_of_bounds",
                segment_index,
                segment_count = self.spine_parse.segments.len(),
            );
            return false;
        };

        if current_segment.byte_range != segment_byte_range {
            tracing::debug!(
                target: "script_kit::spine",
                event = "replace_segment_stale_range",
                segment_index,
                expected = ?current_segment.byte_range,
                got = ?segment_byte_range,
            );
            return false;
        }

        let prefix = &current[..segment_byte_range.start];
        let suffix = &current[segment_byte_range.end..];
        let add_space = trailing_space
            && !replacement.ends_with(char::is_whitespace)
            && !suffix.starts_with(char::is_whitespace);
        let space = if add_space { " " } else { "" };
        let new_text = format!("{prefix}{replacement}{space}{suffix}");
        let cursor = prefix.len() + replacement.len() + space.len();

        tracing::info!(
            target: "script_kit::spine",
            event = "replace_active_segment_text",
            segment_index,
            old_range = ?segment_byte_range,
            replacement,
            trailing_space,
            new_text_len = new_text.len(),
            cursor,
        );

        self.set_filter_text_and_cursor_immediate(new_text, cursor, window, cx);
        true
    }

    /// Set filter text and cursor position in one shot, then reparse spine.
    pub(crate) fn set_filter_text_and_cursor_immediate(
        &mut self,
        text: String,
        cursor_position: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Use existing set_filter_text_immediate to sync filter + input state.
        self.set_filter_text_immediate(text.clone(), window, cx);

        // Now reposition the cursor (set_filter_text_immediate places it at end).
        let cursor = self.clamp_filter_cursor_to_char_boundary(&text, cursor_position);
        self.suppress_filter_events = true;
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_selection(cursor, cursor, window, cx);
        });
        self.suppress_filter_events = false;

        // Reparse spine at the new cursor position and force tail projection
        // before invalidation/reconciliation so list state sees the correct projection.
        if self.spine_enabled {
            self.set_spine_parse_from_filter_and_cursor(&text, cursor);
            self.force_spine_tail_projection_after_trailing_space(&text, cursor);
            self.maybe_start_spine_file_subsearch_for_current_projection(cx);
        }

        self.main_menu_fallback_state.clear();
        self.invalidate_grouped_cache();
        self.reconcile_script_list_after_filter_change("spine_segment_replace", cx);

        cx.notify();
    }

    /// If cursor is at end of text and the last char is whitespace and prompt
    /// segments exist, force the spine projection to a synthetic empty-tail
    /// FreeText projection so the tail-hint section appears.
    fn force_spine_tail_projection_after_trailing_space(&mut self, raw: &str, cursor: usize) {
        if cursor != raw.len() {
            return;
        }
        if !raw.ends_with(char::is_whitespace) {
            return;
        }
        let has_prompt_segments =
            crate::spine::parse_has_prompt_builder_segments(&self.spine_parse);
        if !has_prompt_segments {
            return;
        }
        // Set projection to a synthetic tail FreeText.
        self.spine_projection = Some(crate::spine::SpineCursorProjection {
            active_segment_index: self.spine_parse.segments.len(),
            active_segment_kind: crate::spine::SpineSegmentKind::FreeText,
            active_query: String::new(),
            is_tail: true,
            has_prompt_segments: true,
        });
    }

    /// Check if a byte range is valid for the given filter text.
    fn valid_filter_byte_range(&self, text: &str, range: &std::ops::Range<usize>) -> bool {
        range.start <= range.end
            && range.end <= text.len()
            && text.is_char_boundary(range.start)
            && text.is_char_boundary(range.end)
    }

    /// Clamp a cursor position to the nearest char boundary.
    fn clamp_filter_cursor_to_char_boundary(&self, text: &str, pos: usize) -> usize {
        let clamped = pos.min(text.len());
        // Walk backwards to the nearest char boundary if needed.
        let mut p = clamped;
        while p > 0 && !text.is_char_boundary(p) {
            p -= 1;
        }
        p
    }

    pub(crate) fn sync_filter_input_if_needed(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Sync placeholder if pending
        if let Some(placeholder) = self.pending_placeholder.take() {
            self.gpui_input_state.update(cx, |state, cx| {
                state.set_placeholder(placeholder, window, cx);
            });
        }

        if !self.pending_filter_sync {
            return;
        }

        let desired = self.filter_text.clone();
        let current = self.gpui_input_state.read(cx).value().to_string();
        if current == desired {
            self.pending_filter_sync = false;
            return;
        }

        self.suppress_filter_events = true;
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_value(desired.clone(), window, cx);
            // Ensure cursor is at end with no selection after programmatic set_value
            let len = desired.len();
            state.set_selection(len, len, window, cx);
        });
        self.suppress_filter_events = false;
        self.pending_filter_sync = false;
    }
}
