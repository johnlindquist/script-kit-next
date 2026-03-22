use super::*;
use crate::theme::opacity::OPACITY_SELECTED;

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
                    // Store the pending image immediately but defer the expensive
                    // base64-decode + PNG-to-RenderImage conversion so the AI
                    // window can paint its first frame without blocking.
                    self.pending_image = Some(image_base64.clone());
                    self.defer_cache_pending_image(image_base64.clone(), cx);
                    tracing::info!(target: "ai", text_len = text.len(), image_base64_len = image_base64.len(), "Input set with image (cache deferred)");
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

        // Get vibrancy background - tints the blur effect with theme color.
        let vibrancy_bg = crate::ui_foundation::get_window_vibrancy_background();

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

        div()
            .relative() // Keep relative positioning for overlay dropdowns
            .flex()
            .flex_col()
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
            .child(
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
                    .child(div().w(TITLEBAR_TRAFFIC_LIGHT_ZONE_W)),
            )
            .child(
                div()
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    .child(self.render_sidebar(cx))
                    .child(self.render_main_panel(cx)),
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
