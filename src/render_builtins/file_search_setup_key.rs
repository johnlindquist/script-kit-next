        use crate::file_search::{self, FileType};

        // Use design tokens for spacing/visual, theme for colors
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let _design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let _opacity = self.theme.get_opacity();
        // bg_with_alpha removed - let vibrancy show through from Root (matches main menu)
        // Removed: box_shadows - shadows on transparent elements block vibrancy
        let _box_shadows = self.create_box_shadows();

        // Color values for use in closures
        let text_primary = self.theme.colors.text.primary;
        let text_muted = self.theme.colors.text.muted;
        let text_dimmed = self.theme.colors.text.dimmed;
        let ui_border = self.theme.colors.ui.border;
        let _accent_color = self.theme.colors.accent.selected;
        let list_hover = self.theme.colors.accent.selected_subtle;
        let list_selected = self.theme.colors.accent.selected_subtle;
        // Use theme opacity for vibrancy-compatible selection/hover (matches main menu)
        let opacity = self.theme.get_opacity();
        let selected_alpha = (opacity.selected * 255.0) as u32;
        let hover_alpha = (opacity.hover * 255.0) as u32;

        // Use pre-computed display indices instead of running Nucleo in render
        // This is CRITICAL for animation performance - render must be cheap
        // The display_indices are computed in recompute_file_search_display_indices()
        // which is called when:
        // 1. Results change (new directory listing or search results)
        // 2. Filter pattern changes (user types in existing directory)
        // 3. Loading completes
        let display_indices = &self.file_search_display_indices;
        let filtered_len = display_indices.len();

        // Log render state (throttled - only when state changes meaningfully)
        static LAST_LOG: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let last = LAST_LOG.load(std::sync::atomic::Ordering::Relaxed);
        if now_ms.saturating_sub(last) > 500 {
            // Log at most every 500ms
            LAST_LOG.store(now_ms, std::sync::atomic::Ordering::Relaxed);
            logging::log(
                "SEARCH",
                &format!(
                    "render_file_search: query='{}' loading={} cached={} display={} selected={}",
                    query,
                    self.file_search_loading,
                    self.cached_file_results.len(),
                    filtered_len,
                    selected_index
                ),
            );
        }

        // Get selected file for preview (if any)
        // Use display_indices to map visible index -> actual result index
        let selected_file = display_indices
            .get(selected_index)
            .and_then(|&result_idx| self.cached_file_results.get(result_idx))
            .cloned();

        // Key handler for file search
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;

                let modifiers = &event.keystroke.modifiers;

                // Route keys to actions dialog first if it's open
                match this.route_key_to_actions_dialog(
                    &key_str,
                    key_char,
                    modifiers,
                    ActionsDialogHost::FileSearch,
                    window,
                    cx,
                ) {
                    ActionsRoute::NotHandled => {
                        // Actions dialog not open - continue to file search key handling
                    }
                    ActionsRoute::Handled => {
                        // Key was consumed by actions dialog
                        return;
                    }
                    ActionsRoute::Execute { action_id } => {
                        // User selected an action - execute it
                        // Use handle_action instead of trigger_action_by_name to support
                        // both built-in actions (open_file, quick_look, etc.) and SDK actions
                        this.handle_action(action_id, cx);
                        return;
                    }
                }

                // ESC: Clear query first if present, otherwise go back/close
                if key_str == "escape" {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }

                // Cmd+W closes window
                if has_cmd && key_str == "w" {
                    logging::log("KEY", "Cmd+W - closing window");
                    this.close_and_reset_window(cx);
                    return;
                }

                if let AppView::FileSearchView {
                    query: _,
                    selected_index,
                } = &mut this.current_view
                {
                    // Use pre-computed display_indices to get the selected file
                    // This is consistent with what render shows and avoids re-running Nucleo
                    let get_selected_file = || {
                        this.file_search_display_indices
                            .get(*selected_index)
                            .and_then(|&idx| this.cached_file_results.get(idx))
                            .cloned()
                    };

                    match key_str.as_str() {
                        // Arrow keys are handled by arrow_interceptor in app_impl.rs
                        // which calls stop_propagation(). This is the single source of truth
                        // for arrow key handling in FileSearchView.
                        "up" | "arrowup" | "down" | "arrowdown" => {
                            // Already handled by interceptor, no-op here
                        }
                        // Tab/Shift+Tab handled by intercept_keystrokes in app_impl.rs
                        // (interceptor fires BEFORE input component can capture Tab)
                        "enter" | "return" => {
                            // Check for Cmd+Enter (reveal in finder) first
                            if has_cmd {
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::reveal_in_finder(&file.path);
                                }
                            } else {
                                // Open file with default app
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::open_file(&file.path);
                                    // Close window after opening file
                                    this.close_and_reset_window(cx);
                                }
                            }
                        }
                        _ => {
                            // Handle Cmd+K (toggle actions)
                            if has_cmd && key_str == "k" {
                                if let Some(file) = get_selected_file() {
                                    this.toggle_file_search_actions(&file, window, cx);
                                }
                                return;
                            }
                            // Handle Cmd+Y (Quick Look) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "y" {
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::quick_look(&file.path);
                                }
                                return;
                            }
                            // Handle Cmd+I (Show Info) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "i" {
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::show_info(&file.path);
                                }
                            }
                            // Handle Cmd+O (Open With) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "o" {
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::open_with(&file.path);
                                }
                            }
                        }
                    }
                }
            },
        );
