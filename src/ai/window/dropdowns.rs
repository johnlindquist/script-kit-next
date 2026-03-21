use super::*;

impl AiApp {
    pub(super) fn show_presets_dropdown(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.presets_selected_index = 0;
        self.showing_presets_dropdown = true;
        cx.notify();
    }

    /// Hide the presets dropdown
    pub(super) fn hide_presets_dropdown(&mut self, cx: &mut Context<Self>) {
        self.showing_presets_dropdown = false;
        cx.notify();
    }

    /// Move selection up in presets dropdown
    pub(super) fn presets_select_prev(&mut self, cx: &mut Context<Self>) {
        if !self.presets.is_empty() {
            if self.presets_selected_index > 0 {
                self.presets_selected_index -= 1;
            } else {
                self.presets_selected_index = self.presets.len() - 1;
            }
            cx.notify();
        }
    }

    /// Move selection down in presets dropdown
    pub(super) fn presets_select_next(&mut self, cx: &mut Context<Self>) {
        if !self.presets.is_empty() {
            self.presets_selected_index = (self.presets_selected_index + 1) % self.presets.len();
            cx.notify();
        }
    }

    /// Create a new chat with the selected preset
    pub(super) fn create_chat_with_preset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(preset) = self.presets.get(self.presets_selected_index).cloned() {
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

    // === New Chat Dropdown Methods (Raycast-style) ===

    /// Show the new chat dropdown (Raycast-style with search, last used, presets, models)
    pub(super) fn show_new_chat_dropdown(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.hide_all_dropdowns(cx);
        self.new_chat_dropdown_filter.clear();
        self.new_chat_dropdown_section = 0;
        self.new_chat_dropdown_index = 0;
        self.showing_new_chat_dropdown = true;

        // Clear the input entity so the search field starts empty
        self.new_chat_dropdown_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        tracing::info!("new_chat_dropdown_shown");

        cx.notify();
    }

    /// Hide the new chat dropdown
    pub(super) fn hide_new_chat_dropdown(&mut self, cx: &mut Context<Self>) {
        self.showing_new_chat_dropdown = false;
        self.new_chat_dropdown_filter.clear();
        cx.notify();
    }

    /// Handle filter change in the new chat dropdown
    pub(super) fn on_new_chat_dropdown_filter_change(&mut self, cx: &mut Context<Self>) {
        let filter = self.new_chat_dropdown_input.read(cx).value().to_string();
        self.new_chat_dropdown_filter = filter;
        // Reset selection when filter changes
        self.new_chat_dropdown_section = 0;
        self.new_chat_dropdown_index = 0;
        cx.notify();
    }

    /// Get filtered items for the new chat dropdown
    /// Returns (last_used: Vec<&LastUsedSetting>, presets: Vec<&AiPreset>, models: Vec<&ModelInfo>)
    pub(super) fn get_filtered_new_chat_items(
        &self,
    ) -> (Vec<&LastUsedSetting>, Vec<&AiPreset>, Vec<&ModelInfo>) {
        let filter = self.new_chat_dropdown_filter.to_lowercase();

        let filtered_last_used: Vec<_> = if filter.is_empty() {
            self.last_used_settings.iter().collect()
        } else {
            self.last_used_settings
                .iter()
                .filter(|s| {
                    s.display_name.to_lowercase().contains(&filter)
                        || s.provider_display_name.to_lowercase().contains(&filter)
                })
                .collect()
        };

        let filtered_presets: Vec<_> = if filter.is_empty() {
            self.presets.iter().collect()
        } else {
            self.presets
                .iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&filter)
                        || p.description.to_lowercase().contains(&filter)
                })
                .collect()
        };

        let filtered_models: Vec<_> = if filter.is_empty() {
            self.available_models.iter().collect()
        } else {
            self.available_models
                .iter()
                .filter(|m| {
                    m.display_name.to_lowercase().contains(&filter)
                        || m.provider.to_lowercase().contains(&filter)
                })
                .collect()
        };

        (filtered_last_used, filtered_presets, filtered_models)
    }

    /// Move selection up in new chat dropdown
    pub(super) fn new_chat_dropdown_select_prev(&mut self, cx: &mut Context<Self>) {
        let (last_used, presets, models) = self.get_filtered_new_chat_items();
        let section_sizes = [last_used.len(), presets.len(), models.len()];

        if self.new_chat_dropdown_index > 0 {
            self.new_chat_dropdown_index -= 1;
        } else {
            // Move to previous section
            let mut prev_section = if self.new_chat_dropdown_section > 0 {
                self.new_chat_dropdown_section - 1
            } else {
                2 // wrap to last section
            };

            // Find a non-empty section
            for _ in 0..3 {
                if section_sizes[prev_section] > 0 {
                    self.new_chat_dropdown_section = prev_section;
                    self.new_chat_dropdown_index = section_sizes[prev_section] - 1;
                    break;
                }
                prev_section = if prev_section > 0 {
                    prev_section - 1
                } else {
                    2
                };
            }
        }
        cx.notify();
    }

    /// Move selection down in new chat dropdown
    pub(super) fn new_chat_dropdown_select_next(&mut self, cx: &mut Context<Self>) {
        let (last_used, presets, models) = self.get_filtered_new_chat_items();
        let section_sizes = [last_used.len(), presets.len(), models.len()];

        let current_section_size = section_sizes[self.new_chat_dropdown_section];
        if current_section_size > 0 && self.new_chat_dropdown_index < current_section_size - 1 {
            self.new_chat_dropdown_index += 1;
        } else {
            // Move to next section
            let mut next_section = (self.new_chat_dropdown_section + 1) % 3;

            // Find a non-empty section
            for _ in 0..3 {
                if section_sizes[next_section] > 0 {
                    self.new_chat_dropdown_section = next_section;
                    self.new_chat_dropdown_index = 0;
                    break;
                }
                next_section = (next_section + 1) % 3;
            }
        }
        cx.notify();
    }

    /// Select the current item in the new chat dropdown
    pub(super) fn select_from_new_chat_dropdown(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (last_used, presets, models) = self.get_filtered_new_chat_items();
        let section = self.new_chat_dropdown_section;
        let index = self.new_chat_dropdown_index;

        // Clone the data we need before mutable operations
        let action: Option<NewChatAction> = match section {
            0 => {
                // Last Used Settings
                last_used.get(index).map(|setting| NewChatAction::Model {
                    model_id: setting.model_id.clone(),
                    provider: setting.provider.clone(),
                })
            }
            1 => {
                // Presets - find the original index
                presets.get(index).and_then(|preset| {
                    self.presets
                        .iter()
                        .position(|p| p.id == preset.id)
                        .map(|idx| NewChatAction::Preset { index: idx })
                })
            }
            2 => {
                // Models
                models.get(index).map(|model| NewChatAction::Model {
                    model_id: model.id.clone(),
                    provider: model.provider.clone(),
                })
            }
            _ => None,
        };

        // Now perform the action (borrows released)
        match action {
            Some(NewChatAction::Model { model_id, provider }) => {
                self.hide_new_chat_dropdown(cx);
                self.create_chat_with_model(&model_id, &provider, window, cx);
            }
            Some(NewChatAction::Preset { index }) => {
                self.presets_selected_index = index;
                self.hide_new_chat_dropdown(cx);
                self.create_chat_with_preset(window, cx);
            }
            None => {
                self.hide_new_chat_dropdown(cx);
            }
        }
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

    /// Add a file attachment.
    ///
    /// Delegates to `add_context_part` for dedup and structured logging.
    pub(super) fn add_attachment(&mut self, path: String, cx: &mut Context<Self>) {
        let label = std::path::Path::new(&path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());

        self.add_context_part(
            crate::ai::message_parts::AiContextPart::FilePath { path, label },
            cx,
        );
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
    fn notify_context_parts_changed(&mut self, cx: &mut Context<Self>) {
        if self.pending_context_parts.is_empty() {
            self.clear_context_preflight(cx);
        } else {
            // Use empty string as raw_content — the preflight will pick up
            // any @mentions from the actual composer input at submit time.
            // For the preflight preview, we only care about the explicit
            // pending parts that are already attached.
            self.schedule_context_preflight(String::new(), cx);
        }
    }

    /// Hide all dropdowns (including closing the command bar vibrancy window)
    pub(super) fn hide_all_dropdowns(&mut self, cx: &mut Context<Self>) {
        // Close command bar vibrancy window if open
        self.command_bar.close_app(cx);
        self.showing_presets_dropdown = false;
        self.showing_new_chat_dropdown = false;
        self.new_chat_dropdown_filter.clear();
        cx.notify();
    }
}
