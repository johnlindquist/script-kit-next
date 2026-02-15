impl Focusable for ScriptListApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ScriptListApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Track render timing for filter perf analysis
        let render_start = std::time::Instant::now();
        let filter_snapshot = self.filter_text.clone();

        // Always log render start for "gr" prefix filters to debug the issue
        if filter_snapshot.starts_with("gr") {
            crate::logging::log(
                "FILTER_PERF",
                &format!(
                    "[FRAME_START] filter='{}' selected_idx={} view={:?}",
                    filter_snapshot,
                    self.selected_index,
                    match &self.current_view {
                        AppView::ScriptList => "ScriptList",
                        _ => "Other",
                    }
                ),
            );
        }

        // Flush any pending toasts to gpui-component's NotificationList
        // This is needed because toast push sites don't have window access
        self.flush_pending_toasts(window, cx);

        // Check for API key configuration completion (from built-in commands)
        // The EnvPrompt callback signals completion via channel
        if let Ok((provider, success)) = self.api_key_completion_receiver.try_recv() {
            self.handle_api_key_completion(provider, success, window, cx);
        }

        // Check for builtin confirmation modal completion
        // When user confirms/cancels a dangerous action (Quit, Shut Down, etc.),
        // the modal callback sends the result through this channel
        if let Ok((entry_id, confirmed)) = self.builtin_confirm_receiver.try_recv() {
            self.handle_builtin_confirmation(entry_id, confirmed, cx);
        }

        // Check for inline chat escape (from built-in ChatPrompt)
        // The ChatPrompt escape callback signals via channel
        if self.inline_chat_escape_receiver.try_recv().is_ok() {
            crate::logging::log(
                "CHAT",
                "Inline chat escape received - returning to main menu",
            );
            self.go_back_or_close(window, cx);
        }

        // Check for inline chat configure (user wants to set up API key)
        // The ChatPrompt configure callback signals via channel
        if self.inline_chat_configure_receiver.try_recv().is_ok() {
            crate::logging::log(
                "CHAT",
                "Inline chat configure received - showing API key setup",
            );
            // First close the chat prompt
            self.go_back_or_close(window, cx);
            // Then show the Vercel API key configuration prompt
            self.show_api_key_prompt(
                "SCRIPT_KIT_VERCEL_API_KEY",
                "Enter your Vercel AI Gateway API key",
                "Vercel AI Gateway",
                cx,
            );
        }

        // Check for inline chat Claude Code (user wants to enable Claude Code)
        // The ChatPrompt Claude Code callback signals via channel
        if self.inline_chat_claude_code_receiver.try_recv().is_ok() {
            crate::logging::log(
                "CHAT",
                "Inline chat Claude Code received - enabling Claude Code",
            );
            // Enable Claude Code in config.ts
            self.enable_claude_code_in_config(window, cx);
        }

        // Check for naming dialog completion (submit or cancel)
        if let Ok(payload) = self.naming_submit_receiver.try_recv() {
            self.handle_naming_dialog_completion(payload, window, cx);
        }

        // Focus-lost auto-dismiss: Close dismissable prompts when the main window loses focus
        // This includes focus loss to other app windows like Notes/AI.
        // When is_pinned is true, the window stays open on blur (only closes via ESC/Cmd+W)
        let is_window_focused = platform::is_main_window_focused();
        if self.was_window_focused && !is_window_focused {
            // Window just lost focus (user clicked another window)
            // Only auto-dismiss if we're in a dismissable view AND window is visible AND not pinned
            // AND we're past the focus grace period (prevents race condition on window open)
            // AND the actions popup is not open (actions popup is a companion window, not "losing focus")
            if self.is_dismissable_view()
                && script_kit_gpui::is_main_window_visible()
                && !self.is_pinned
                && !script_kit_gpui::is_within_focus_grace_period()
                && !confirm::is_confirm_window_open()
                && !actions::is_actions_window_open()
            {
                logging::log(
                    "FOCUS",
                    "Main window lost focus while in dismissable view - closing",
                );
                self.close_and_reset_window(cx);
            } else if actions::is_actions_window_open() {
                logging::log(
                    "FOCUS",
                    "Main window lost focus but actions popup is open - staying open",
                );
            } else if confirm::is_confirm_window_open() {
                logging::log(
                    "FOCUS",
                    "Main window lost focus but confirm dialog is open - staying open",
                );
            } else if script_kit_gpui::is_within_focus_grace_period() {
                logging::log(
                    "FOCUS",
                    "Main window lost focus but within grace period - ignoring",
                );
            } else if self.is_pinned {
                logging::log(
                    "FOCUS",
                    "Main window lost focus but is pinned - staying open",
                );
            }
        }
        self.was_window_focused = is_window_focused;

        // Apply pending focus request (if any). This is the new "apply once" mechanism
        // that replaces the old "perpetually enforce focus in render()" pattern.
        // Focus is applied exactly once when pending_focus is set, then cleared.
        self.apply_pending_focus(window, cx);

        // Sync filter input if needed (views that use shared input)
        if matches!(
            self.current_view,
            AppView::ScriptList
                | AppView::ClipboardHistoryView { .. }
                | AppView::EmojiPickerView { .. }
                | AppView::AppLauncherView { .. }
                | AppView::WindowSwitcherView { .. }
                | AppView::DesignGalleryView { .. }
                | AppView::FileSearchView { .. }
                | AppView::ThemeChooserView { .. }
        ) {
            self.sync_filter_input_if_needed(window, cx);
        }

        // NOTE: Prompt messages are now handled via event-driven async_channel listener
        // spawned in execute_interactive() - no polling needed in render()

        // P0-4: Clone current_view only for dispatch (needed to call &mut self methods)
        // The clone is unavoidable due to borrow checker: we need &mut self for render methods
        // but also need to match on self.current_view. Future optimization: refactor render
        // methods to take &str/&[T] references instead of owned values.
        //
        // HUD is now handled by hud_manager as a separate floating window
        // No need to render it as part of this view
        let current_view = self.current_view.clone();
        let main_content: AnyElement = match current_view {
            AppView::ScriptList => self.render_script_list(cx).into_any_element(),
            AppView::ActionsDialog => self.render_actions_dialog(cx),
            AppView::ArgPrompt {
                id,
                placeholder,
                choices,
                actions,
            } => self
                .render_arg_prompt(id, placeholder, choices, actions, cx)
                .into_any_element(),
            AppView::DivPrompt { id, entity } => {
                self.render_div_prompt(id, entity, cx).into_any_element()
            }
            AppView::FormPrompt { entity, .. } => {
                self.render_form_prompt(entity, cx).into_any_element()
            }
            AppView::TermPrompt { entity, .. } => {
                self.render_term_prompt(entity, cx).into_any_element()
            }
            AppView::EditorPrompt { entity, .. } => {
                self.render_editor_prompt(entity, cx).into_any_element()
            }
            AppView::SelectPrompt { entity, .. } => {
                self.render_select_prompt(entity, cx).into_any_element()
            }
            AppView::PathPrompt { entity, .. } => {
                self.render_path_prompt(entity, cx).into_any_element()
            }
            AppView::EnvPrompt { entity, .. } => {
                self.render_env_prompt(entity, cx).into_any_element()
            }
            AppView::DropPrompt { entity, .. } => {
                self.render_drop_prompt(entity, cx).into_any_element()
            }
            AppView::TemplatePrompt { entity, .. } => {
                self.render_template_prompt(entity, cx).into_any_element()
            }
            AppView::ChatPrompt { entity, .. } => {
                self.render_chat_prompt(entity, cx).into_any_element()
            }
            // P0 FIX: View state only - data comes from self.cached_clipboard_entries
            AppView::ClipboardHistoryView {
                filter,
                selected_index,
            } => self
                .render_clipboard_history(filter, selected_index, cx)
                .into_any_element(),
            AppView::EmojiPickerView {
                filter,
                selected_index,
                selected_category,
            } => self
                .render_emoji_picker(filter, selected_index, selected_category, cx)
                .into_any_element(),
            // P0 FIX: View state only - data comes from self.apps
            AppView::AppLauncherView {
                filter,
                selected_index,
            } => self
                .render_app_launcher(filter, selected_index, cx)
                .into_any_element(),
            // P0 FIX: View state only - data comes from self.cached_windows
            AppView::WindowSwitcherView {
                filter,
                selected_index,
            } => self
                .render_window_switcher(filter, selected_index, cx)
                .into_any_element(),
            AppView::DesignGalleryView {
                filter,
                selected_index,
            } => self
                .render_design_gallery(filter, selected_index, cx)
                .into_any_element(),
            AppView::WebcamView { entity } => {
                self.render_webcam_prompt(entity, cx).into_any_element()
            }
            AppView::ScratchPadView { entity, .. } => {
                self.render_editor_prompt(entity, cx).into_any_element()
            }
            AppView::QuickTerminalView { entity, .. } => {
                self.render_term_prompt(entity, cx).into_any_element()
            }
            AppView::FileSearchView {
                ref query,
                selected_index,
            } => self
                .render_file_search(query, selected_index, cx)
                .into_any_element(),
            AppView::ThemeChooserView {
                ref filter,
                selected_index,
            } => self.render_theme_chooser(filter, selected_index, cx),
            AppView::NamingPrompt { entity, .. } => {
                self.render_naming_prompt(entity, cx).into_any_element()
            }
            AppView::CreationFeedback { ref path } => {
                self.render_creation_feedback(path.clone(), cx)
            }
        };

        // Wrap content in a container that can have the debug grid overlay
        let window_bounds = window.bounds();
        let window_size = gpui::size(window_bounds.size.width, window_bounds.size.height);

        // Clone grid_config for use in the closure
        let grid_config = self.grid_config.clone();

        // Build component bounds for the current view (for debug overlay)
        // P0 FIX: Only compute bounds when grid overlay is actually enabled
        // Previously this was computed unconditionally on every frame
        let component_bounds = if grid_config.is_some() {
            self.build_component_bounds(window_size)
        } else {
            Vec::new()
        };

        // Build warning banner if needed (bun not available)
        let warning_banner = if self.show_bun_warning {
            let banner_colors = WarningBannerColors::from_theme(&self.theme);
            let entity = cx.entity().downgrade();
            let entity_for_dismiss = entity.clone();

            Some(
                div().w_full().px(px(12.)).pt(px(8.)).child(
                    WarningBanner::new(
                        "bun is not installed. Click to download from bun.sh",
                        banner_colors,
                    )
                    .on_click(Box::new(move |_event, _window, cx| {
                        if let Some(app) = entity.upgrade() {
                            app.update(cx, |this, _cx| {
                                this.open_bun_website();
                            });
                        }
                    }))
                    .on_dismiss(Box::new(move |_event, _window, cx| {
                        if let Some(app) = entity_for_dismiss.upgrade() {
                            app.update(cx, |this, cx| {
                                this.dismiss_bun_warning(cx);
                            });
                        }
                    })),
                ),
            )
        } else {
            None
        };

        // Build shortcut recorder overlay if state is set
        let shortcut_recorder_overlay = self.render_shortcut_recorder_overlay(window, cx);

        // Build alias input overlay if state is set
        let alias_input_overlay = self.render_alias_input_overlay(window, cx);

        // Log render timing for filter perf analysis - always log for "gr" filters
        let render_elapsed = render_start.elapsed();
        if filter_snapshot.starts_with("gr") {
            crate::logging::log(
                "FILTER_PERF",
                &format!(
                    "[FRAME_END] filter='{}' total={:.2}ms",
                    filter_snapshot,
                    render_elapsed.as_secs_f64() * 1000.0
                ),
            );
        }

        // Clear perf tracking after one complete render cycle
        if self.filter_perf_start.is_some() && !self.filter_text.is_empty() {
            self.filter_perf_start = None;
        }

        // Get vibrancy background - tints the blur effect with theme color
        let vibrancy_bg = crate::ui_foundation::get_window_vibrancy_background();

        // Capture mouse_cursor_hidden for use in div builder
        let mouse_cursor_hidden = self.mouse_cursor_hidden;

        // Get themed border color with 25% opacity (0x40 = 64/255)
        let border_color = rgba((self.theme.colors.ui.border << 8) | 0x40);

        div()
            .w_full()
            .h_full()
            .relative()
            .flex()
            .flex_col()
            // Hide mouse cursor while typing - use cursor() modifier instead of window method
            .when(mouse_cursor_hidden, |d| d.cursor(CursorStyle::None))
            // Hide cursor and clear hover on any keyboard interaction
            .capture_key_down(cx.listener(|this, _: &KeyDownEvent, _window, cx| {
                this.input_mode = InputMode::Keyboard;
                this.hovered_index = None;
                this.hide_mouse_cursor(cx);
            }))
            // Show cursor when mouse moves.
            .on_mouse_move(cx.listener(|this, _: &MouseMoveEvent, _window, cx| {
                this.show_mouse_cursor(cx);
            }))
            // CRITICAL: Apply vibrancy background like POC does
            // This tints the blur effect with the theme color
            .bg(vibrancy_bg)
            // Visual styling from vibrancy POC - rounded corners, subtle border, clip content
            .rounded(px(12.))
            .border_1()
            .border_color(border_color)
            .overflow_hidden()
            // Warning banner appears at the top when bun is not available
            .when_some(warning_banner, |container, banner| container.child(banner))
            // Main content takes remaining space
            .child(div().flex_1().w_full().min_h(px(0.)).child(main_content))
            // Shortcut recorder overlay (on top of main content when recording)
            .when_some(shortcut_recorder_overlay, |container, overlay| {
                container.child(overlay)
            })
            // Alias input overlay (on top of main content when entering alias)
            .when_some(alias_input_overlay, |container, overlay| {
                container.child(overlay)
            })
            .when_some(grid_config, |container, config| {
                let overlay_bounds = gpui::Bounds {
                    origin: gpui::point(px(0.), px(0.)),
                    size: window_size,
                };
                container.child(debug_grid::render_grid_overlay(
                    &config,
                    overlay_bounds,
                    &component_bounds,
                ))
            })
    }
}
