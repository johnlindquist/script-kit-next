use super::*;

impl AiApp {
    fn selected_preset_dropdown_index(&self) -> usize {
        crate::components::inline_dropdown::inline_dropdown_clamp_selected_index(
            self.presets_selected_index,
            self.presets.len(),
        )
    }

    pub(super) fn show_presets_dropdown(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.presets_selected_index = 0;
        self.showing_presets_dropdown = true;
        tracing::info!(
            target: "ai",
            event = "ai_presets_dropdown_opened",
            preset_count = self.presets.len(),
            selected_index = self.presets_selected_index,
            "Opened AI presets inline dropdown"
        );
        cx.notify();
    }

    pub(super) fn hide_presets_dropdown(&mut self, cx: &mut Context<Self>) {
        if !self.showing_presets_dropdown {
            return;
        }
        self.showing_presets_dropdown = false;
        tracing::info!(
            target: "ai",
            event = "ai_presets_dropdown_closed",
            "Closed AI presets inline dropdown"
        );
        cx.notify();
    }

    pub(super) fn presets_select_prev(&mut self, cx: &mut Context<Self>) {
        let preset_count = self.presets.len();
        if preset_count == 0 {
            return;
        }
        self.presets_selected_index =
            crate::components::inline_dropdown::inline_dropdown_select_prev(
                self.selected_preset_dropdown_index(),
                preset_count,
            );
        let visible = crate::components::inline_dropdown::inline_dropdown_visible_range(
            self.presets_selected_index,
            preset_count,
            8,
        );
        tracing::info!(
            target: "ai",
            event = "ai_presets_dropdown_selection_moved",
            direction = "prev",
            selected_index = self.presets_selected_index,
            visible_start = visible.start,
            visible_end = visible.end,
            "Moved AI presets dropdown selection"
        );
        cx.notify();
    }

    pub(super) fn presets_select_next(&mut self, cx: &mut Context<Self>) {
        let preset_count = self.presets.len();
        if preset_count == 0 {
            return;
        }
        self.presets_selected_index =
            crate::components::inline_dropdown::inline_dropdown_select_next(
                self.selected_preset_dropdown_index(),
                preset_count,
            );
        let visible = crate::components::inline_dropdown::inline_dropdown_visible_range(
            self.presets_selected_index,
            preset_count,
            8,
        );
        tracing::info!(
            target: "ai",
            event = "ai_presets_dropdown_selection_moved",
            direction = "next",
            selected_index = self.presets_selected_index,
            visible_start = visible.start,
            visible_end = visible.end,
            "Moved AI presets dropdown selection"
        );
        cx.notify();
    }

    pub(super) fn confirm_presets_selection(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let selected_index = self.selected_preset_dropdown_index();
        let preset_name = self
            .presets
            .get(selected_index)
            .map(|preset| preset.name.to_string())
            .unwrap_or_default();
        tracing::info!(
            target: "ai",
            event = "ai_presets_dropdown_confirmed",
            selected_index,
            preset_name = %preset_name,
            "Confirmed AI presets inline dropdown selection"
        );
        self.presets_selected_index = selected_index;
        self.create_chat_with_preset(window, cx);
    }

    pub(super) fn create_chat_with_preset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(preset) = self
            .presets
            .get(self.selected_preset_dropdown_index())
            .cloned()
        {
            tracing::info!(
                target: "ai",
                event = "ai_preset_applied",
                preset_name = %preset.name,
                preferred_model = ?preset.preferred_model,
                "Applying AI preset from inline dropdown"
            );
            self.hide_presets_dropdown(cx);

            // Create new chat with system prompt
            let chat_id = self.new_conversation(window, cx);
            if let Some(chat_id) = chat_id {
                // Add system message from preset
                if !preset.system_prompt.is_empty() {
                    let system_msg = Message::new(
                        chat_id,
                        crate::ai::model::MessageRole::System,
                        preset.system_prompt,
                    );
                    if let Err(e) = storage::save_message(&system_msg) {
                        tracing::error!(error = %e, "Failed to save system message");
                    }
                    // Reload messages to include system prompt
                    self.current_messages =
                        storage::get_chat_messages(&chat_id).unwrap_or_default();
                    self.cache_message_images(&self.current_messages.clone());
                }

                // Set preferred model if specified
                if let Some(model_id) = preset.preferred_model {
                    if let Some(model) = self.available_models.iter().find(|m| m.id == model_id) {
                        self.selected_model = Some(model.clone());
                    }
                }

                cx.notify();
            }
        }
    }

    /// Canonical entry point for showing the new-chat picker surface.
    ///
    /// All UI affordances (header button, command bar action, future shortcuts)
    /// should call this instead of reaching for individual dropdown/command-bar
    /// helpers directly. Currently delegates to `show_new_chat_command_bar`.
    pub(super) fn show_canonical_new_chat_surface(
        &mut self,
        source: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.hide_all_dropdowns(cx);
        self.show_new_chat_command_bar(source, window, cx);
    }

    /// Create a new chat with a specific model
    pub(super) fn create_chat_with_model(
        &mut self,
        model_id: &str,
        provider: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Set the selected model
        if let Some(model) = self
            .available_models
            .iter()
            .find(|m| m.id == model_id && m.provider == provider)
        {
            self.selected_model = Some(model.clone());
        }

        // Update last used settings
        self.update_last_used_settings(model_id, provider);

        // Create the chat with full state reset
        self.new_conversation(window, cx);
    }

    /// Remove a file attachment by its index in the file-path subset of `pending_context_parts`.
    ///
    /// The index corresponds to the position within the `FilePath` entries only
    /// (matching the order returned by `file_path_parts()`), not the overall
    /// `pending_context_parts` index. This removes exactly one matching `FilePath`
    /// context part and leaves all other parts (including `ResourceUri`) intact.
    pub(super) fn remove_attachment(&mut self, file_index: usize, cx: &mut Context<Self>) {
        // Find the absolute index of the Nth FilePath entry
        let abs_index = self
            .pending_context_parts
            .iter()
            .enumerate()
            .filter(|(_, part)| {
                matches!(
                    part,
                    crate::ai::message_parts::AiContextPart::FilePath { .. }
                )
            })
            .nth(file_index)
            .map(|(i, _)| i);

        if let Some(idx) = abs_index {
            let removed = self.pending_context_parts.remove(idx);
            let remaining =
                crate::ai::message_parts::file_path_parts(&self.pending_context_parts).len();
            tracing::info!(
                file_index = file_index,
                abs_index = idx,
                path = %removed.source(),
                remaining_file_parts = remaining,
                "attachment_removed"
            );
            self.notify_context_parts_changed(cx);
        }
    }

    /// Remove a pending context part by index and schedule a preflight update.
    pub(super) fn remove_context_part(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.pending_context_parts.len() {
            return;
        }
        // Close preview if it references a stale index after removal
        if let Some(pi) = self.context_preview_index {
            if pi == index {
                self.context_preview_index = None;
            } else if pi > index {
                self.context_preview_index = Some(pi - 1);
            }
        }
        let removed = self.pending_context_parts.remove(index);
        tracing::info!(
            target: "ai",
            index,
            label = %removed.label(),
            source = %removed.source(),
            remaining = self.pending_context_parts.len(),
            "ai_context_part_removed"
        );
        self.notify_context_parts_changed(cx);
    }

    /// Add a context part with deterministic dedup.
    ///
    /// If an identical part (same variant, URI/path, and label) is already
    /// present, the call is a no-op and a structured log checkpoint is emitted.
    /// On successful add, schedules a context preflight so the UI shows an
    /// up-to-date budget/provenance summary.
    pub(super) fn add_context_part(
        &mut self,
        part: crate::ai::message_parts::AiContextPart,
        cx: &mut Context<Self>,
    ) {
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
            "ai_context_part_added"
        );

        self.pending_context_parts.push(part);
        self.notify_context_parts_changed(cx);
    }

    /// Toggle the full prepared-message inspector panel (⌥⌘I).
    pub(super) fn toggle_context_inspector(&mut self, cx: &mut Context<Self>) {
        self.show_context_inspector = !self.show_context_inspector;
        tracing::info!(
            target: "ai",
            visible = self.show_context_inspector,
            has_receipt = self.last_prepared_message_receipt.is_some(),
            "context_inspector_toggled"
        );
        cx.notify();
    }

    /// Clear all pending context parts (both ResourceUri and FilePath) and reset preflight.
    pub(super) fn clear_context_parts(&mut self, cx: &mut Context<Self>) {
        let cleared_count = self.pending_context_parts.len();
        if cleared_count == 0 {
            return;
        }

        self.context_preview_index = None;
        self.pending_context_parts.clear();
        self.inline_owned_context_tokens.clear();
        tracing::info!(
            target: "ai",
            cleared_count,
            "ai_context_parts_cleared"
        );
        self.clear_context_preflight(cx);
    }

    /// Clear all file-path attachments from `pending_context_parts`.
    ///
    /// Only `FilePath` entries are removed; `ResourceUri` entries are preserved.
    /// Schedules a preflight update if any resource URIs remain.
    pub(super) fn clear_attachments(&mut self, cx: &mut Context<Self>) {
        let before = crate::ai::message_parts::file_path_parts(&self.pending_context_parts).len();
        self.pending_context_parts.retain(|part| {
            !matches!(
                part,
                crate::ai::message_parts::AiContextPart::FilePath { .. }
            )
        });
        if before > 0 {
            tracing::info!(
                cleared_file_path_parts = before,
                remaining_context_parts = self.pending_context_parts.len(),
                "file_path_attachments_cleared"
            );
        }
        self.notify_context_parts_changed(cx);
    }

    /// Centralized notification after any context parts mutation.
    ///
    /// Either schedules a preflight (if parts remain) or clears the
    /// preflight state. Both paths call `cx.notify()` internally.
    /// Uses the current composer draft so recommendations stay accurate.
    pub(super) fn notify_context_parts_changed(&mut self, cx: &mut Context<Self>) {
        if self.pending_context_parts.is_empty() {
            self.clear_context_preflight(cx);
        } else {
            self.schedule_context_preflight_for_current_draft(cx);
        }
    }

    /// Hide all dropdowns (including closing the command bar vibrancy window)
    pub(super) fn hide_all_dropdowns(&mut self, cx: &mut Context<Self>) {
        // Close command bar vibrancy window if open
        self.command_bar.close_app(cx);
        // Close new-chat command bar (Raycast-style Cmd+N dropdown)
        self.new_chat_command_bar.close(cx);
        self.showing_presets_dropdown = false;
        // Close context picker if open
        self.context_picker = None;
        cx.notify();
    }
}
