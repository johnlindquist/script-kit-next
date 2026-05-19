use super::*;

impl ScriptListApp {
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
        self.rebuild_main_window_preflight_if_needed();
    }

    pub(crate) fn queue_filter_compute(&mut self, value: String, cx: &mut Context<Self>) {
        logging::log(
            "FILTER_PERF",
            &format!(
                "[2/5] APPLY_FILTER value='{}' len={}",
                value,
                value.len()
            ),
        );
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
            self.reconcile_script_list_after_filter_change("filter_immediate", cx);
            if let Some(snapshot) = selection_before.as_ref() {
                if self.restore_root_file_handoff_selection_from_snapshot(snapshot) {
                    self.scroll_to_selected_if_needed("filter_immediate_restore_root_file_handoff");
                    self.rebuild_main_window_preflight_if_needed();
                }
            }
            self.update_window_size();
            let update_elapsed = update_start.elapsed();
            logging::log(
                "FILTER_PERF",
                &format!(
                    "[3/5] APPLY_FILTER_DONE in {:.2}ms for '{}'",
                    update_elapsed.as_secs_f64() * 1000.0,
                    value
                ),
            );
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
        // Run 12 Pass 11 — clear any pending Cmd+Enter inline AI proposal so
        // it doesn't survive a filter change (otherwise the proposal would
        // appear stale against an unrelated input).
        self.pending_menu_syntax_ai_proposal = None;
        self.suppress_filter_events = true;
        self.filter_text = text.clone();
        self.pending_programmatic_filter_echo = Some(text.clone());
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_value(text.clone(), window, cx);
            // Ensure cursor is at end with no selection after programmatic set_value
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

        if !handled_by_subview && matches!(self.current_view, AppView::ScriptList) {
            self.set_menu_syntax_mode_from_filter(&text);
            self.run_menu_syntax_trigger_popup_state_machine(&text, window, cx);
            self.invalidate_grouped_cache();
        } else {
            self.menu_syntax_mode = crate::menu_syntax::MenuSyntaxMode::default();
        }

        if self.menu_syntax_mode.is_menu_syntax_for(&text)
            || self.menu_syntax_trigger_popup_state.snapshot.is_some()
        {
            // Menu syntax owns the result list entirely — clear any stale
            // fallback items so pressing Enter routes to execute_selected,
            // not execute_selected_fallback. Also clear when the trigger
            // popup is open for a partial trigger like `;t` (where
            // `is_menu_syntax_for` still returns false because the parser
            // doesn't yet recognize `;t` as a full target).
            self.main_menu_fallback_state.clear();
        }

        if !handled_by_subview
            && matches!(self.current_view, AppView::ScriptList)
            && matches!(
                Self::special_entry_from_script_list_filter(&text),
                Some(crate::filter_input_core::ScriptListSpecialEntry::AcpMentionPicker)
            )
        {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "script_list_special_entry_routed",
                filter_text = %text,
                entry_kind = "acp_mention_picker",
                current_view = ?self.current_view,
            );
            self.open_tab_ai_acp_with_mention_picker(window, cx);
            return;
        }

        self.computed_filter_text = text.clone();
        self.filter_coalescer.reset();
        self.maybe_start_root_file_search(&text, cx);
        self.reconcile_script_list_after_filter_change("set_filter_text_immediate", cx);

        // Update fallback state immediately based on filter results
        // This ensures SimulateKey commands can check fallback state correctly
        // NOTE: validate_selection_bounds already clears main_menu_fallback_state,
        // but we need special handling for legacy SimulateKey compatibility.
        // Skip when a subview handled the filter: `get_filtered_results_cached`
        // and `collect_fallbacks` are ScriptList-only and would incorrectly
        // flip a builtin subview into the script-list fallback mode.
        if !handled_by_subview
            && !text.is_empty()
            && !self.menu_syntax_mode.is_menu_syntax_for(&text)
            && self.menu_syntax_trigger_popup_state.snapshot.is_none()
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

        self.rebuild_main_window_preflight_if_needed();
        self.update_window_size_deferred(window, cx);
        cx.notify();
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
