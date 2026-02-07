use super::*;

impl ScriptListApp {
    fn cancel_script_execution(&mut self, cx: &mut Context<Self>) {
        logging::log("EXEC", "=== Canceling script execution ===");

        // Send cancel message to script (Exit with cancel code)
        // Use try_send to avoid blocking UI thread during cancellation
        if let Some(ref sender) = self.response_sender {
            // Try to send Exit message to terminate the script cleanly
            let exit_msg = Message::Exit {
                code: Some(1), // Non-zero code indicates cancellation
                message: Some("Cancelled by user".to_string()),
            };
            match sender.try_send(exit_msg) {
                Ok(()) => logging::log("EXEC", "Sent Exit message to script"),
                Err(std::sync::mpsc::TrySendError::Full(_)) => logging::log(
                    "EXEC",
                    "Exit message dropped - channel full (script may be stuck)",
                ),
                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                    logging::log("EXEC", "Exit message dropped - script already exited")
                }
            }
        } else {
            logging::log("EXEC", "No response_sender - script may not be running");
        }

        // Belt-and-suspenders: Force-kill the process group using stored PID
        // This ensures cleanup even if Drop doesn't fire properly
        if let Some(pid) = self.current_script_pid.take() {
            logging::log(
                "CLEANUP",
                &format!("Force-killing script process group {}", pid),
            );
            #[cfg(unix)]
            {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &format!("-{}", pid)])
                    .output();
            }
        }

        // Abort script session if it exists
        {
            let mut session_guard = self.script_session.lock();
            if let Some(_session) = session_guard.take() {
                logging::log("EXEC", "Cleared script session");
            }
        }

        // Reset to script list view
        self.reset_to_script_list(cx);
        logging::log("EXEC", "=== Script cancellation complete ===");
    }

    /// Flush pending toasts from ToastManager to gpui-component's NotificationList
    ///
    /// This should be called at the start of render() where we have window access.
    /// The ToastManager acts as a staging queue for toasts pushed from callbacks
    /// that don't have window access.
    fn flush_pending_toasts(&mut self, window: &mut gpui::Window, cx: &mut gpui::App) {
        use gpui_component::WindowExt;

        let pending = self.toast_manager.drain_pending();
        let count = pending.len();
        if count > 0 {
            logging::log(
                "UI",
                &format!("Flushing {} pending toast(s) to NotificationList", count),
            );
        }
        for toast in pending {
            logging::log("UI", &format!("Pushing notification: {}", toast.message));
            let notification = pending_toast_to_notification(&toast);
            window.push_notification(notification, cx);
        }
    }

    /// Close window and reset to default state (Cmd+W global handler)
    ///
    /// This method handles the global Cmd+W shortcut which should work
    /// regardless of what prompt or view is currently active. It:
    /// 1. Cancels any running script
    /// 2. Resets state to the default script list
    /// 3. Hides the window
    fn close_and_reset_window(&mut self, cx: &mut Context<Self>) {
        logging::log("VISIBILITY", "=== Close and reset window ===");

        // Reset pin state when window is closed
        self.is_pinned = false;

        // Close child windows FIRST if open (they are children of main window)
        // Actions window
        if self.show_actions_popup || is_actions_window_open() {
            self.show_actions_popup = false;
            self.actions_dialog = None;
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                })
                .ok();
            })
            .detach();
            logging::log("VISIBILITY", "Closed actions window before hiding main");
        }

        // Confirm window (modal)
        if crate::confirm::is_confirm_window_open() {
            crate::confirm::close_confirm_window(cx);
            logging::log("VISIBILITY", "Closed confirm window before hiding main");
        }

        // Save window position BEFORE hiding (main window is hidden, not closed)
        if let Some((x, y, w, h)) = crate::platform::get_main_window_bounds() {
            let bounds = crate::window_state::PersistedWindowBounds::new(x, y, w, h);
            let displays = crate::platform::get_macos_displays();
            let _ =
                crate::window_state::save_main_position_with_display_detection(bounds, &displays);
        }

        // Update visibility state FIRST to prevent race conditions
        script_kit_gpui::set_main_window_visible(false);
        logging::log("VISIBILITY", "WINDOW_VISIBLE set to: false");

        // If in a prompt, cancel the script execution
        if self.is_in_prompt() {
            logging::log(
                "VISIBILITY",
                "In prompt mode - canceling script before hiding",
            );
            self.cancel_script_execution(cx);
        } else {
            // Just reset to script list (clears filter, selection, scroll)
            self.reset_to_script_list(cx);
        }

        // Check if Notes or AI windows are open BEFORE hiding
        let notes_open = notes::is_notes_window_open();
        let ai_open = ai::is_ai_window_open();
        logging::log(
            "VISIBILITY",
            &format!(
                "Secondary windows: notes_open={}, ai_open={}",
                notes_open, ai_open
            ),
        );

        // CRITICAL: Only hide main window if Notes/AI are open
        // cx.hide() hides the ENTIRE app (all windows), so we use
        // platform::hide_main_window() to hide only the main window
        if notes_open || ai_open {
            logging::log(
                "VISIBILITY",
                "Using hide_main_window() - secondary windows are open",
            );
            platform::hide_main_window();
        } else {
            logging::log("VISIBILITY", "Using cx.hide() - no secondary windows");
            cx.hide();
        }
        logging::log("VISIBILITY", "=== Window closed ===");
    }

    /// Clear the current built-in view's filter/query text if non-empty.
    ///
    /// Returns `true` if the filter was cleared (caller should stop processing ESC).
    /// Returns `false` if the filter was already empty (caller should proceed with go_back_or_close).
    ///
    /// This implements the "ESC clears filter first" UX pattern that matches the main menu behavior.
    fn clear_builtin_view_filter(&mut self, cx: &mut Context<Self>) -> bool {
        let cleared = match &self.current_view {
            AppView::ClipboardHistoryView { filter, .. } if !filter.is_empty() => {
                Some("ClipboardHistory filter")
            }
            AppView::AppLauncherView { filter, .. } if !filter.is_empty() => {
                Some("AppLauncher filter")
            }
            AppView::WindowSwitcherView { filter, .. } if !filter.is_empty() => {
                Some("WindowSwitcher filter")
            }
            AppView::DesignGalleryView { filter, .. } if !filter.is_empty() => {
                Some("DesignGallery filter")
            }
            AppView::ThemeChooserView { filter, .. } if !filter.is_empty() => {
                Some("ThemeChooser filter")
            }
            AppView::FileSearchView { query, .. } if !query.is_empty() => Some("FileSearch query"),
            _ => None,
        };
        let Some(cleared) = cleared else {
            return false;
        };
        logging::log("KEY", &format!("ESC - clearing {}", cleared));

        // Clear shared filter state (for views using the shared input component)
        self.filter_text.clear();
        self.pending_filter_sync = true;

        // Clear view-specific filter and reset selection
        match &mut self.current_view {
            AppView::ClipboardHistoryView {
                filter,
                selected_index,
            } => {
                Self::clear_builtin_query_state(filter, selected_index);
                self.clipboard_list_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
                // Update focused entry to first entry (filter cleared = show all)
                self.focused_clipboard_entry_id =
                    self.cached_clipboard_entries.first().map(|e| e.id.clone());
            }
            AppView::AppLauncherView {
                filter,
                selected_index,
            } => {
                Self::clear_builtin_query_state(filter, selected_index);
                self.list_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
            }
            AppView::WindowSwitcherView {
                filter,
                selected_index,
            } => {
                Self::clear_builtin_query_state(filter, selected_index);
                self.window_list_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
            }
            AppView::DesignGalleryView {
                filter,
                selected_index,
            } => {
                Self::clear_builtin_query_state(filter, selected_index);
                self.design_gallery_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
            }
            AppView::ThemeChooserView {
                filter,
                selected_index,
            } => {
                Self::clear_builtin_query_state(filter, selected_index);
                self.theme_chooser_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
            }
            AppView::FileSearchView {
                query,
                selected_index,
            } => {
                Self::clear_builtin_query_state(query, selected_index);
                // Cancel any pending search
                self.file_search_debounce_task = None;
                self.file_search_loading = false;
                self.cached_file_results.clear();
                self.file_search_display_indices.clear();
                self.file_search_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
            }
            _ => {}
        }

        // Clear hover state to prevent stale highlights after filter change
        self.hovered_index = None;

        cx.notify();
        true
    }

    fn reset_script_list_filter_state(&mut self) {
        self.filter_text.clear();
        self.computed_filter_text.clear();
        self.filter_coalescer.reset();
        self.pending_filter_sync = true;
    }

    fn reset_script_list_selection_state(&mut self, cx: &mut Context<Self>) {
        self.invalidate_grouped_cache();
        self.sync_list_state();
        self.selected_index = 0;
        self.hovered_index = None;
        self.validate_selection_bounds(cx);
        self.main_list_state.scroll_to(ListOffset {
            item_ix: 0,
            offset_in_item: px(0.),
        });
        self.last_scrolled_index = Some(0);
    }

    fn reset_script_list_filter_and_selection_state(&mut self, cx: &mut Context<Self>) {
        self.reset_script_list_filter_state();
        self.reset_script_list_selection_state(cx);
    }

    fn request_script_list_main_filter_focus(&mut self, cx: &mut Context<Self>) {
        self.focused_input = FocusedInput::MainFilter;
        self.request_focus(FocusTarget::MainFilter, cx);
    }

    /// Go back to main menu or close window depending on how the view was opened.
    ///
    /// If the current built-in view was opened from the main menu, this returns to the
    /// main menu (ScriptList). If it was opened directly via hotkey or protocol command,
    /// this closes the window entirely.
    ///
    /// This provides consistent UX: pressing ESC always "goes back" one step.
    fn go_back_or_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.opened_from_main_menu {
            logging::log(
                "KEY",
                "ESC - returning to main menu (opened from main menu)",
            );
            // Return to main menu
            self.current_view = AppView::ScriptList;
            // Reset the flag since we're now in main menu
            self.opened_from_main_menu = false;

            self.reset_script_list_filter_and_selection_state(cx);

            // Sync input and reset placeholder to default
            self.gpui_input_state.update(cx, |state, cx| {
                state.set_value("", window, cx);
                state.set_selection(0, 0, window, cx);
                state.set_placeholder(DEFAULT_PLACEHOLDER.to_string(), window, cx);
            });

            // Clear actions popup state (prevents stale overlay on return to menu)
            self.show_actions_popup = false;
            self.actions_dialog = None;

            self.update_window_size_deferred(window, cx);
            self.request_script_list_main_filter_focus(cx);
        } else {
            logging::log(
                "KEY",
                "ESC - closing window (opened directly via hotkey/protocol)",
            );
            self.close_and_reset_window(cx);
        }
    }

    /// Handle global keyboard shortcuts with configurable dismissability
    ///
    /// Returns `true` if the shortcut was handled (caller should return early)
    ///
    /// # Arguments
    /// * `event` - The key down event to check
    /// * `is_dismissable` - If true, ESC key will also close the window (for prompts like arg, div, form, etc.)
    ///   If false, only Cmd+W closes the window (for prompts like term, editor)
    /// * `cx` - The context
    ///
    /// # Handled shortcuts
    /// - Cmd+W: Always closes window and resets to default state
    /// - Escape: Only closes window if `is_dismissable` is true AND actions popup is not showing
    /// - Cmd+Shift+M: Cycle vibrancy material (for debugging)
    #[tracing::instrument(skip(self, event, cx), fields(key = %event.keystroke.key, modifiers = ?event.keystroke.modifiers, is_dismissable))]
}
