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
            let chat_id = self.create_chat(window, cx);
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
    pub(super) fn show_new_chat_dropdown(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.hide_all_dropdowns(cx);
        self.new_chat_dropdown_filter.clear();
        self.new_chat_dropdown_section = 0;
        self.new_chat_dropdown_index = 0;
        self.showing_new_chat_dropdown = true;

        // Clear the search input - InputState::set_value takes window and cx
        // For now just clear the filter; the input will be cleared on next type
        // Actually we just set needs_focus_input flag since we can't easily clear

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

        // Create the chat
        self.create_chat(window, cx);
    }

    // === Attachments Picker Methods ===

    /// Show the attachments picker
    pub(super) fn show_attachments_picker(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.showing_attachments_picker = true;
        cx.notify();
    }

    /// Hide the attachments picker
    pub(super) fn hide_attachments_picker(&mut self, cx: &mut Context<Self>) {
        self.showing_attachments_picker = false;
        cx.notify();
    }

    /// Add a file attachment
    pub(super) fn add_attachment(&mut self, path: String, cx: &mut Context<Self>) {
        if !self.pending_attachments.contains(&path) {
            self.pending_attachments.push(path);
            cx.notify();
        }
    }

    /// Remove a file attachment
    pub(super) fn remove_attachment(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.pending_attachments.len() {
            self.pending_attachments.remove(index);
            cx.notify();
        }
    }

    /// Clear all attachments
    pub(super) fn clear_attachments(&mut self, cx: &mut Context<Self>) {
        self.pending_attachments.clear();
        cx.notify();
    }

    /// Hide all dropdowns (including closing the command bar vibrancy window)
    pub(super) fn hide_all_dropdowns(&mut self, cx: &mut Context<Self>) {
        // Close command bar vibrancy window if open
        self.command_bar.close_app(cx);
        self.showing_presets_dropdown = false;
        self.showing_attachments_picker = false;
        self.showing_new_chat_dropdown = false;
        self.new_chat_dropdown_filter.clear();
        cx.notify();
    }
}
