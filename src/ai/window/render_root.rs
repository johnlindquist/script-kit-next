use super::*;
use crate::theme::opacity::{OPACITY_HOVER, OPACITY_SELECTED};

impl AiApp {
    /// Add a file attachment by enqueuing a `FilePath` context part with dedup.
    pub(super) fn add_attachment(&mut self, path: String, cx: &mut Context<Self>) {
        let label = std::path::Path::new(&path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        let part = crate::ai::message_parts::AiContextPart::FilePath { path, label };
        let already_present = self
            .pending_context_parts
            .iter()
            .any(|existing| existing == &part);

        if already_present {
            tracing::info!(
                target: "ai",
                label = %part.label(),
                source = %part.source(),
                "ai_context_part_add_skipped_duplicate"
            );
            return;
        }

        tracing::info!(
            target: "ai",
            label = %part.label(),
            source = %part.source(),
            count_before = self.pending_context_parts.len(),
            "Enqueued context part"
        );

        self.pending_context_parts.push(part);
        self.notify_context_parts_changed(cx);
    }

    pub(super) fn toggle_mini_history_overlay(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.showing_mini_history_overlay = !self.showing_mini_history_overlay;
        if self.showing_mini_history_overlay {
            // Focus the search input so typing immediately filters chats
            self.focus_search(window, cx);
        }
        cx.notify();
    }

    fn render_mini_history_overlay(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // Transparent backdrop catches clicks outside the overlay to dismiss it.
        div()
            .id("ai-mini-history-backdrop")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.showing_mini_history_overlay = false;
                    cx.notify();
                }),
            )
            .child(
                div()
                    .id("ai-mini-history-overlay")
                    .absolute()
                    .top(px(48.))
                    .left(S3)
                    .w(px(320.))
                    .max_h(px(420.))
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded(R_LG)
                    .shadow_lg()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    // Stop propagation so clicks inside the overlay don't dismiss it
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(self.render_sidebar_body(cx)),
            )
    }

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
            tracing::debug!(target: "ai", "Applied command bar focus in render");
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
                tracing::debug!(target: "ai", "Applied setup mode focus in render (main focus handle)");
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
            let command_name = cmd.name();
            let started_at = std::time::Instant::now();

            tracing::info!(
                category = "AI",
                event = "ai_command_apply_start",
                command = command_name,
                "Applying AI command"
            );

            match cmd {
                AiCommand::SetWindowMode(window_mode) => {
                    // Save current bounds under the old role before switching
                    let wb = window.window_bounds();
                    crate::window_state::save_window_from_gpui(
                        super::window_api::window_role_for_mode(self.window_mode),
                        wb,
                    );
                    self.window_mode = window_mode;
                    self.showing_mini_history_overlay = false;
                    window.set_window_title(window_mode.title());
                    // Restore saved bounds for target mode, falling back to defaults
                    let target_role = super::window_api::window_role_for_mode(window_mode);
                    let saved = crate::window_state::load_window_bounds(target_role);
                    if let Some(persisted) = saved {
                        let bounds = persisted.to_gpui().get_bounds();
                        window.resize(bounds.size);
                    } else {
                        window.resize(size(
                            px(window_mode.default_width()),
                            px(window_mode.default_height()),
                        ));
                    }
                    cx.notify();
                    tracing::info!(
                        target: "ai",
                        window_mode = ?window_mode,
                        restored_saved_bounds = saved.is_some(),
                        "AI window mode set"
                    );
                }
                AiCommand::SetSearch(query) => {
                    self.search_state.update(cx, |state, cx| {
                        state.set_value(query.clone(), window, cx);
                    });
                    self.on_search_change(cx);
                    tracing::info!(target: "ai", query = %query, "Search filter set");
                }
                AiCommand::SetInput { text, submit } => {
                    self.set_composer_value(&text, window, cx);
                    tracing::info!(target: "ai", input_len = text.len(), "Input set");
                    if submit {
                        self.submit_message(window, cx);
                        tracing::info!(target: "ai", "Message submitted - streaming started");
                    }
                }
                AiCommand::SetInputWithImage {
                    text,
                    image_base64,
                    submit,
                } => {
                    self.set_composer_value(&text, window, cx);
                    // Move the original String into pending_image and clone only once for
                    // deferred cache work. The old code cloned twice.
                    let cache_image = image_base64.clone();
                    self.pending_image = Some(image_base64);
                    self.defer_cache_pending_image(cache_image, cx);

                    tracing::info!(
                        category = "AI",
                        event = "ai_pending_image_received",
                        text_len = text.len(),
                        submit,
                        "Input set with image (cache deferred)"
                    );

                    if submit {
                        self.submit_message(window, cx);
                        tracing::info!(target: "ai", "Message with image submitted - streaming started");
                    }
                }
                AiCommand::AddAttachment { path } => {
                    self.add_attachment(path.clone(), cx);
                }
                AiCommand::InitializeWithPendingChat => {
                    self.initialize_with_pending_chat(window, cx);
                }
                AiCommand::ShowCommandBar => {
                    self.show_command_bar(window, cx);
                    tracing::info!(target: "ai", "Command bar shown via stdin command");
                }
                AiCommand::ApplyPreset { preset_id } => {
                    if let Some(idx) = self.presets.iter().position(|p| p.id == preset_id) {
                        self.presets_selected_index = idx;
                        self.create_chat_with_preset(window, cx);
                        tracing::info!(
                            preset_id = %preset_id,
                            action = "apply_preset",
                            "Applied AI preset via command"
                        );
                    } else {
                        tracing::warn!(
                            preset_id = %preset_id,
                            action = "apply_preset_not_found",
                            "Preset not found"
                        );
                    }
                }
                AiCommand::ReloadPresets => {
                    self.presets = AiPreset::load_all_presets();
                    tracing::info!(
                        count = self.presets.len(),
                        action = "reload_presets",
                        "Reloaded AI presets"
                    );
                    cx.notify();
                }
                AiCommand::SimulateKey { key, modifiers } => {
                    self.handle_simulated_key(&key, &modifiers, window, cx);
                }
                AiCommand::StartChat {
                    chat_id,
                    message,
                    parts,
                    image,
                    system_prompt,
                    model_id,
                    provider,
                    on_created,
                    submit,
                } => {
                    self.handle_start_chat(
                        chat_id,
                        message,
                        parts,
                        image,
                        system_prompt,
                        model_id,
                        provider,
                        on_created,
                        submit,
                        window,
                        cx,
                    );
                }
            }

            tracing::info!(
                category = "AI",
                event = "ai_command_apply_finish",
                command = command_name,
                duration_ms = started_at.elapsed().as_millis() as u64,
                "Applied AI command"
            );
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

        // Get vibrancy background - None when vibrancy enabled (let native blur show through).
        let vibrancy_bg =
            crate::ui_foundation::get_vibrancy_background(&crate::theme::get_cached_theme());

        // Capture mouse_cursor_hidden for use in div builder.
        let mouse_cursor_hidden = self.mouse_cursor_hidden;
        let title_text = if let Some(chat) = self.get_selected_chat() {
            if chat.title.is_empty() {
                "New Chat".to_string()
            } else {
                chat.title.clone()
            }
        } else {
            "AI Chat".to_string()
        };
        let mini_model_display_name = self
            .selected_model
            .as_ref()
            .map(|model| model.display_name.clone())
            .unwrap_or_else(|| "Select Model".to_string());

        div()
            .relative() // Keep relative positioning for overlay dropdowns
            .flex()
            .flex_col()
            .size_full()
            // Apply background only when vibrancy is disabled (same as main window)
            .when_some(vibrancy_bg, |d, bg| d.bg(bg))
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
            .text_color(cx.theme().foreground)
            .track_focus(&self.focus_handle)
            // Hide mouse cursor on keyboard interaction
            .when(mouse_cursor_hidden, |d| d.cursor(CursorStyle::None))
            // Show cursor when mouse moves
            .on_mouse_move(cx.listener(|this, _: &MouseMoveEvent, _window, cx| {
                this.show_mouse_cursor(cx);
            }))
            // Close popups when clicking on the AI window surface
            .on_any_mouse_down(cx.listener(|this, _, _window, cx| {
                if this.command_bar.is_open() {
                    this.hide_command_bar(cx);
                }
                if crate::confirm::is_confirm_window_open() {
                    crate::confirm::route_key_to_confirm_popup("escape", cx);
                }
            }))
            // CRITICAL: Use capture_key_down to intercept keys BEFORE Input component handles them.
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                this.handle_root_key_down(event, window, cx);
            }))
            .child(if self.window_mode.is_mini() {
                let muted_fg = cx.theme().muted_foreground;
                let label_color = muted_fg.opacity(OPACITY_SELECTED);
                div()
                    .id("ai-titlebar-mini")
                    .w_full()
                    .h(px(44.))
                    .pl(TITLEBAR_LEFT_PADDING)
                    .pr(S3)
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    // Left: "AI" title + clickable model name
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S2)
                            .min_w_0()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("AI"),
                            )
                            .child(
                                div()
                                    .id("ai-mini-model-name")
                                    .text_xs()
                                    .text_color(label_color)
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .cursor_pointer()
                                    .hover(|el| el.text_color(cx.theme().foreground))
                                    .tooltip(|window, cx| {
                                        Tooltip::new("Switch model").build(window, cx)
                                    })
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, window, cx| {
                                            this.show_command_bar(window, cx);
                                        }),
                                    )
                                    .child(mini_model_display_name),
                            ),
                    )
                    // Right: Recent (⌘J), New (⌘N), Actions (⌘K)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S1)
                            // Recent button
                            .child(
                                div()
                                    .id("ai-mini-recent")
                                    .px(S2)
                                    .py(S1)
                                    .rounded(R_SM)
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .gap(SP_2)
                                    .text_xs()
                                    .text_color(if self.showing_mini_history_overlay {
                                        cx.theme().foreground
                                    } else {
                                        label_color
                                    })
                                    .when(self.showing_mini_history_overlay, |el| {
                                        el.bg(cx.theme().muted.opacity(OPACITY_HOVER))
                                    })
                                    .hover(|el| {
                                        el.bg(cx.theme().muted.opacity(OPACITY_HOVER))
                                            .text_color(cx.theme().foreground)
                                    })
                                    .tooltip(|window, cx| {
                                        Tooltip::new("Recent chats")
                                            .key_binding(
                                                gpui::Keystroke::parse("cmd-j").ok().map(Kbd::new),
                                            )
                                            .build(window, cx)
                                    })
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.toggle_mini_history_overlay(window, cx);
                                    }))
                                    .child("Recent"),
                            )
                            // New chat button
                            .child(
                                div()
                                    .id("ai-mini-new")
                                    .px(S2)
                                    .py(S1)
                                    .rounded(R_SM)
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .gap(SP_2)
                                    .text_xs()
                                    .text_color(label_color)
                                    .hover(|el| {
                                        el.bg(cx.theme().muted.opacity(OPACITY_HOVER))
                                            .text_color(cx.theme().foreground)
                                    })
                                    .tooltip(|window, cx| {
                                        Tooltip::new("New chat")
                                            .key_binding(
                                                gpui::Keystroke::parse("cmd-n").ok().map(Kbd::new),
                                            )
                                            .build(window, cx)
                                    })
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.show_new_chat_command_bar(window, cx);
                                    }))
                                    .child("New"),
                            )
                            // Actions button
                            .child(
                                div()
                                    .id("ai-mini-actions")
                                    .px(S2)
                                    .py(S1)
                                    .rounded(R_SM)
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .gap(SP_2)
                                    .text_xs()
                                    .text_color(label_color)
                                    .hover(|el| {
                                        el.bg(cx.theme().muted.opacity(OPACITY_HOVER))
                                            .text_color(cx.theme().foreground)
                                    })
                                    .tooltip(|window, cx| {
                                        Tooltip::new("Actions")
                                            .key_binding(
                                                gpui::Keystroke::parse("cmd-k").ok().map(Kbd::new),
                                            )
                                            .build(window, cx)
                                    })
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.show_command_bar(window, cx);
                                    }))
                                    .child("Actions"),
                            ),
                    )
                    .into_any_element()
            } else {
                div()
                    .id("ai-titlebar")
                    .w_full()
                    .h(TITLEBAR_H)
                    .flex()
                    .flex_row()
                    .items_center()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .w(TITLEBAR_TRAFFIC_LIGHT_ZONE_W)
                            .h_full()
                            .flex()
                            .items_center()
                            // left padding clears macOS traffic lights (~56px) + tight gap
                            .pl(TITLEBAR_LEFT_PADDING)
                            .child(self.render_sidebar_toggle(cx)),
                    )
                    .child(
                        div()
                            .id("ai-centered-title")
                            .flex_1()
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(
                                        cx.theme().muted_foreground.opacity(OPACITY_SELECTED),
                                    )
                                    .child(title_text),
                            ),
                    )
                    .child(div().w(TITLEBAR_TRAFFIC_LIGHT_ZONE_W))
                    .into_any_element()
            })
            .child(if self.window_mode.is_mini() {
                div()
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    .child(self.render_main_panel(cx))
                    .into_any_element()
            } else {
                div()
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    .child(self.render_sidebar(cx))
                    .child(self.render_main_panel(cx))
                    .into_any_element()
            })
            .when(
                self.window_mode.is_mini() && self.showing_mini_history_overlay,
                |el| el.child(self.render_mini_history_overlay(cx)),
            )
            // Overlay dropdowns (only one at a time)
            .when(self.showing_presets_dropdown, |el| {
                el.child(self.render_presets_dropdown(cx))
            })
            // Keyboard shortcuts overlay (Cmd+/)
            .when(self.showing_shortcuts_overlay, |el| {
                el.child(self.render_shortcuts_overlay(cx))
            })
    }
}
