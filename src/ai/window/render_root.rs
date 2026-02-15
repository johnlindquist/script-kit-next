use super::*;

impl AiApp {
    pub(super) fn process_render_focus_requests(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Process command bar focus request FIRST (set after vibrancy window opens)
        // This ensures keyboard events route to the window's key handler for command bar navigation.
        if self.needs_command_bar_focus {
            self.needs_command_bar_focus = false;
            self.focus_handle.focus(window, cx);
            crate::logging::log("AI", "Applied command bar focus in render");
        }
        // Process focus request flag (set by open_ai_window when bringing existing window to front).
        else if !self.command_bar.is_open()
            && (self.needs_focus_input
                || AI_FOCUS_REQUESTED.swap(false, std::sync::atomic::Ordering::SeqCst))
        {
            self.needs_focus_input = false;
            // In setup mode, focus main handle for keyboard navigation instead of input.
            let in_setup_mode = self.available_models.is_empty() && !self.showing_api_key_input;
            if in_setup_mode {
                self.focus_handle.focus(window, cx);
                crate::logging::log(
                    "AI",
                    "Applied setup mode focus in render (main focus handle)",
                );
            } else {
                self.focus_input(window, cx);
            }
        }
    }

    pub(super) fn process_render_pending_commands(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        for cmd in take_ai_commands() {
            match cmd {
                AiCommand::SetSearch(query) => {
                    self.search_state.update(cx, |state, cx| {
                        state.set_value(query.clone(), window, cx);
                    });
                    self.on_search_change(cx);
                    crate::logging::log("AI", &format!("Search filter set to: {}", query));
                }
                AiCommand::SetInput { text, submit } => {
                    // Sanitize newlines - single-line Input can't handle them
                    // (GPUI's shape_line panics on newlines)
                    let sanitized_text = text.replace('\n', " ");
                    self.input_state.update(cx, |state, cx| {
                        state.set_value(sanitized_text.clone(), window, cx);
                        // Ensure cursor is at end of text with proper focus for editing
                        let text_len = state.text().len();
                        state.set_selection(text_len, text_len, window, cx);
                    });
                    crate::logging::log("AI", &format!("Input set to: {}", sanitized_text));
                    if submit {
                        self.submit_message(window, cx);
                        crate::logging::log("AI", "Message submitted - streaming started");
                    }
                }
                AiCommand::SetInputWithImage {
                    text,
                    image_base64,
                    submit,
                } => {
                    // Sanitize newlines - single-line Input can't handle them
                    // (GPUI's shape_line panics on newlines)
                    let sanitized_text = text.replace('\n', " ");
                    self.input_state.update(cx, |state, cx| {
                        state.set_value(sanitized_text.clone(), window, cx);
                        // Ensure cursor is at end of text with proper focus for editing
                        let text_len = state.text().len();
                        state.set_selection(text_len, text_len, window, cx);
                    });
                    // Store the pending image to be included with the next message
                    self.cache_image_from_base64(&image_base64);
                    self.pending_image = Some(image_base64.clone());
                    crate::logging::log(
                        "AI",
                        &format!(
                            "Input set with image: {} chars text, {} chars base64",
                            text.len(),
                            image_base64.len()
                        ),
                    );
                    if submit {
                        self.submit_message(window, cx);
                        crate::logging::log(
                            "AI",
                            "Message with image submitted - streaming started",
                        );
                    }
                }
                AiCommand::AddAttachment { path } => {
                    self.add_attachment(path.clone(), cx);
                    crate::logging::log("AI", &format!("Added attachment: {}", path));
                }
                AiCommand::InitializeWithPendingChat => {
                    self.initialize_with_pending_chat(window, cx);
                }
                AiCommand::ShowCommandBar => {
                    self.show_command_bar(window, cx);
                    crate::logging::log("AI", "Command bar shown via stdin command");
                }
                AiCommand::SimulateKey { key, modifiers } => {
                    self.handle_simulated_key(&key, &modifiers, window, cx);
                }
            }
        }
    }
}

impl Render for AiApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Update cached theme values if theme has changed (hot-reload)
        self.maybe_update_theme_cache();

        // Persist bounds on change (ensures bounds saved even on traffic light close)
        self.maybe_persist_bounds(window);

        self.process_render_focus_requests(window, cx);
        self.process_render_pending_commands(window, cx);

        // NOTE: Shadow disabled for vibrancy - shadows on transparent elements cause gray fill.
        // The vibrancy effect requires no shadow on transparent elements.

        // Get vibrancy background - tints the blur effect with theme color.
        let vibrancy_bg = crate::ui_foundation::get_window_vibrancy_background();

        // Capture mouse_cursor_hidden for use in div builder.
        let mouse_cursor_hidden = self.mouse_cursor_hidden;

        div()
            .relative() // Required for absolutely positioned sidebar toggle
            .flex()
            .flex_row()
            .size_full()
            // Apply vibrancy background like POC does - Root no longer provides this
            .bg(vibrancy_bg)
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
            .text_color(cx.theme().foreground)
            .track_focus(&self.focus_handle)
            // Hide mouse cursor on keyboard interaction
            .when(mouse_cursor_hidden, |d| d.cursor(CursorStyle::None))
            // Show cursor when mouse moves
            .on_mouse_move(cx.listener(|this, _: &MouseMoveEvent, _window, cx| {
                this.show_mouse_cursor(cx);
            }))
            // CRITICAL: Use capture_key_down to intercept keys BEFORE Input component handles them.
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                this.handle_root_key_down(event, window, cx);
            }))
            .child(self.render_sidebar(cx))
            .child(self.render_main_panel(cx))
            // Absolutely positioned sidebar toggle - stays fixed regardless of sidebar state (like Raycast)
            .child(
                div()
                    .absolute()
                    .top(px(4.)) // Align with traffic lights (~8px) and title center
                    .left(px(78.)) // After traffic lights (~70px) + small gap
                    .child(self.render_sidebar_toggle(cx)),
            )
            // Absolutely positioned CENTERED title - centered within main panel area.
            .child(
                div()
                    .id("ai-centered-title")
                    .absolute()
                    .top_0()
                    // Offset left by sidebar width when sidebar is open
                    .when(self.sidebar_collapsed, |d| d.left_0())
                    .when(!self.sidebar_collapsed, |d| d.left(px(240.))) // Sidebar width
                    .right_0()
                    .h(px(36.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(cx.theme().muted_foreground.opacity(0.7))
                            .child(
                                self.get_selected_chat()
                                    .map(|c| {
                                        if c.title.is_empty() {
                                            "New Chat".to_string()
                                        } else {
                                            c.title.clone()
                                        }
                                    })
                                    .unwrap_or_else(|| "AI Chat".to_string()),
                            ),
                    ),
            )
            // Absolutely positioned right-side icons in header
            .child(
                div()
                    .absolute()
                    .top(px(10.)) // Vertically centered in 36px header
                    .right(px(12.))
                    .flex()
                    .items_center()
                    .gap_2()
                    // Plus icon for new chat (using SVG for reliable rendering)
                    .child(
                        div()
                            .id("ai-new-chat-icon-global")
                            .cursor_pointer()
                            .hover(|s| s.opacity(1.0))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.create_chat(window, cx);
                            }))
                            .child(
                                svg()
                                    .external_path(LocalIconName::Plus.external_path())
                                    .size(px(16.))
                                    .text_color(cx.theme().muted_foreground.opacity(0.7)),
                            ),
                    )
                    // Dropdown chevron icon (using SVG for reliable rendering)
                    .child(
                        div()
                            .id("ai-menu-icon-global")
                            .cursor_pointer()
                            .hover(|s| s.opacity(1.0))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.toggle_new_chat_command_bar(window, cx);
                            }))
                            .child(
                                svg()
                                    .external_path(LocalIconName::ChevronDown.external_path())
                                    .size(px(16.))
                                    .text_color(cx.theme().muted_foreground.opacity(0.7)),
                            ),
                    ),
            )
            // Overlay dropdowns (only one at a time)
            .when(self.showing_presets_dropdown, |el| {
                el.child(self.render_presets_dropdown(cx))
            })
            .when(self.showing_attachments_picker, |el| {
                el.child(self.render_attachments_picker(cx))
            })
            // Keyboard shortcuts overlay (Cmd+/)
            .when(self.showing_shortcuts_overlay, |el| {
                el.child(self.render_shortcuts_overlay(cx))
            })
    }
}
