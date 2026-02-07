use super::*;

impl AiApp {
    pub(super) fn show_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Open the command bar (CommandBar handles window creation internally)
        self.command_bar.open(window, cx);

        // CRITICAL: Focus main focus_handle so keyboard events route to us
        // The ActionsWindow is a visual-only popup - it does NOT take keyboard focus.
        // macOS popup windows often don't receive keyboard events properly.
        // This also unfocuses the Input component which would otherwise consume arrow keys.
        self.focus_handle.focus(window, cx);

        // Request command bar focus on next render for keyboard routing
        // This ensures the focus persists even if something else tries to steal it
        self.needs_command_bar_focus = true;

        // Log focus state for debugging - check both main handle AND input's focus state
        let main_focused = self.focus_handle.is_focused(window);
        let input_focused = self
            .input_state
            .read(cx)
            .focus_handle(cx)
            .is_focused(window);
        crate::logging::log(
            "AI",
            &format!(
                "show_command_bar: main_focus={} input_focus={} (input should be false for arrow keys to work)",
                main_focused, input_focused
            ),
        );

        cx.notify();
    }

    /// Hide the command bar (closes the vibrancy window) and refocus the input
    #[tracing::instrument(skip(self, cx))]
    pub(super) fn hide_command_bar(&mut self, cx: &mut Context<Self>) {
        self.command_bar.close(cx);
        // Refocus the chat input after closing the command bar
        self.request_focus(cx);
    }

    /// Handle character input in command bar
    pub(super) fn command_bar_handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.command_bar.handle_char(ch, cx);
    }

    /// Handle backspace in command bar
    pub(super) fn command_bar_handle_backspace(&mut self, cx: &mut Context<Self>) {
        self.command_bar.handle_backspace(cx);
    }

    /// Move selection up in command bar
    pub(super) fn command_bar_select_prev(&mut self, cx: &mut Context<Self>) {
        self.command_bar.select_prev(cx);
    }

    /// Move selection down in command bar
    pub(super) fn command_bar_select_next(&mut self, cx: &mut Context<Self>) {
        self.command_bar.select_next(cx);
    }

    /// Execute the selected command bar action
    pub(super) fn execute_command_bar_action(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(action_id) = self.command_bar.execute_selected_action(cx) {
            self.execute_action(&action_id, window, cx);
        }
    }

    // === New Chat Command Bar Methods ===
    // Raycast-style dropdown in the titlebar using CommandBar component

    /// Toggle the new chat command bar dropdown
    pub(super) fn toggle_new_chat_command_bar(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.new_chat_command_bar.is_open() {
            self.hide_new_chat_command_bar(cx);
        } else {
            self.show_new_chat_command_bar(window, cx);
        }
    }

    /// Show the new chat command bar with dynamically built actions
    pub(super) fn show_new_chat_command_bar(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use crate::actions::{
            get_new_chat_actions, NewChatModelInfo, NewChatPresetInfo, WindowPosition,
        };

        // Build last used settings from recent chats
        let last_used: Vec<NewChatModelInfo> = self
            .last_used_settings
            .iter()
            .map(|s| NewChatModelInfo {
                model_id: s.model_id.clone(),
                display_name: s.display_name.clone(),
                provider: s.provider.clone(),
                provider_display_name: s.provider_display_name.clone(),
            })
            .collect();

        // Build presets list
        let presets: Vec<NewChatPresetInfo> = self
            .presets
            .iter()
            .map(|p| {
                NewChatPresetInfo {
                    id: p.id.to_string(),
                    name: p.name.to_string(),
                    icon: p.icon, // Use the preset's icon
                }
            })
            .collect();

        // Build models list
        let models: Vec<NewChatModelInfo> = self
            .available_models
            .iter()
            .map(|m| {
                let provider_display = match m.provider.as_str() {
                    "anthropic" => "Anthropic",
                    "openai" => "OpenAI",
                    "google" => "Google",
                    "groq" => "Groq",
                    "openrouter" => "OpenRouter",
                    "vercel" => "Vercel",
                    _ => &m.provider,
                }
                .to_string();
                NewChatModelInfo {
                    model_id: m.id.clone(),
                    display_name: m.display_name.clone(),
                    provider: m.provider.clone(),
                    provider_display_name: provider_display,
                }
            })
            .collect();

        // Build actions and update the command bar
        let actions = get_new_chat_actions(&last_used, &presets, &models);
        self.new_chat_command_bar.set_actions(actions, cx);

        // Open at top-right position (below titlebar)
        self.new_chat_command_bar
            .open_at_position(window, cx, WindowPosition::TopRight);

        // Focus main handle for keyboard routing
        self.focus_handle.focus(window, cx);

        // Also hide other dropdowns
        self.hide_presets_dropdown(cx);
        self.hide_attachments_picker(cx);

        cx.notify();
    }

    /// Hide the new chat command bar
    pub(super) fn hide_new_chat_command_bar(&mut self, cx: &mut Context<Self>) {
        self.new_chat_command_bar.close(cx);
        self.request_focus(cx);
    }

    /// Execute the selected new chat action
    pub(super) fn execute_new_chat_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(action_id) = self.new_chat_command_bar.execute_selected_action(cx) {
            self.handle_new_chat_action(&action_id, window, cx);
        }
    }

    /// Handle action from the new chat dropdown
    pub(super) fn handle_new_chat_action(
        &mut self,
        action_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if action_id.starts_with("last_used_") {
            // Parse index from action ID
            if let Some(idx_str) = action_id.strip_prefix("last_used_") {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if let Some(setting) = self.last_used_settings.get(idx) {
                        let model_id = setting.model_id.clone();
                        let provider = setting.provider.clone();
                        self.create_chat_with_model(&model_id, &provider, window, cx);
                    }
                }
            }
        } else if action_id.starts_with("preset_") {
            // Parse preset ID
            if let Some(preset_id) = action_id.strip_prefix("preset_") {
                if let Some(idx) = self.presets.iter().position(|p| p.id == preset_id) {
                    self.presets_selected_index = idx;
                    self.create_chat_with_preset(window, cx);
                }
            }
        } else if action_id.starts_with("model_") {
            // Parse model index
            if let Some(idx_str) = action_id.strip_prefix("model_") {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if let Some(model) = self.available_models.get(idx) {
                        let model_id = model.id.clone();
                        let provider = model.provider.clone();
                        self.create_chat_with_model(&model_id, &provider, window, cx);
                    }
                }
            }
        }
    }

    /// Move selection up in new chat dropdown
    pub(super) fn new_chat_command_bar_select_prev(&mut self, cx: &mut Context<Self>) {
        self.new_chat_command_bar.select_prev(cx);
    }

    /// Move selection down in new chat dropdown
    pub(super) fn new_chat_command_bar_select_next(&mut self, cx: &mut Context<Self>) {
        self.new_chat_command_bar.select_next(cx);
    }

    /// Execute an action by ID
    pub(super) fn execute_action(
        &mut self,
        action_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action_id {
            "copy_response" => self.copy_last_response(cx),
            "copy_chat" => self.copy_entire_chat(cx),
            "copy_last_code" => self.copy_last_code_block(cx),
            "submit" => self.submit_message(window, cx),
            "new_chat" => {
                self.create_chat(window, cx);
            }
            "delete_chat" => {
                self.delete_selected_chat(cx);
            }
            "add_attachment" => {
                self.show_attachments_picker(window, cx);
            }
            "paste_image" => self.paste_image_from_clipboard(cx),
            "change_model" => {
                // Model selection now available via Actions (Cmd+K)
                // Cycle to next model as a convenience
                self.cycle_model(cx);
            }
            _ => {
                tracing::warn!(action = action_id, "Unknown action");
            }
        }
    }

    /// Handle a simulated key press (for testing via stdin)
    pub(super) fn handle_simulated_key(
        &mut self,
        key: &str,
        modifiers: &[KeyModifier],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let has_cmd = modifiers.contains(&KeyModifier::Cmd);
        let key_lower = key.to_lowercase();

        crate::logging::log(
            "AI",
            &format!(
                "SimulateKey: key='{}' modifiers={:?} command_bar_open={}",
                key_lower,
                modifiers,
                self.command_bar.is_open()
            ),
        );

        // Handle Cmd+K to toggle command bar
        if has_cmd && key_lower == "k" {
            crate::logging::log("AI", "SimulateKey: Cmd+K - toggling command bar");
            if self.command_bar.is_open() {
                self.hide_command_bar(cx);
            } else {
                self.hide_all_dropdowns(cx);
                self.show_command_bar(window, cx);
            }
            return;
        }

        // Handle command bar navigation when it's open
        if self.command_bar.is_open() {
            match key_lower.as_str() {
                "up" | "arrowup" => {
                    crate::logging::log("AI", "SimulateKey: Up in command bar");
                    self.command_bar_select_prev(cx);
                }
                "down" | "arrowdown" => {
                    crate::logging::log("AI", "SimulateKey: Down in command bar");
                    self.command_bar_select_next(cx);
                }
                "enter" | "return" => {
                    crate::logging::log("AI", "SimulateKey: Enter in command bar");
                    self.execute_command_bar_action(window, cx);
                }
                "escape" | "esc" => {
                    crate::logging::log("AI", "SimulateKey: Escape - closing command bar");
                    self.hide_command_bar(cx);
                }
                "backspace" | "delete" => {
                    crate::logging::log("AI", "SimulateKey: Backspace in command bar");
                    self.command_bar_handle_backspace(cx);
                }
                _ => {
                    // Handle printable characters for search
                    if let Some(ch) = key_lower.chars().next() {
                        if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_' {
                            crate::logging::log(
                                "AI",
                                &format!("SimulateKey: Typing '{}' in command bar search", ch),
                            );
                            self.command_bar_handle_char(ch, cx);
                        }
                    }
                }
            }
            return;
        }

        // Handle presets dropdown navigation
        if self.showing_presets_dropdown {
            match key_lower.as_str() {
                "up" | "arrowup" => self.presets_select_prev(cx),
                "down" | "arrowdown" => self.presets_select_next(cx),
                "enter" | "return" => self.create_chat_with_preset(window, cx),
                "escape" | "esc" => self.hide_presets_dropdown(cx),
                _ => {}
            }
            return;
        }

        // Handle setup mode navigation (when no providers configured)
        let in_setup_mode = self.available_models.is_empty() && !self.showing_api_key_input;
        if in_setup_mode {
            crate::logging::log(
                "AI",
                &format!(
                    "SimulateKey in setup mode: key='{}' focus_index={}",
                    key_lower, self.setup_button_focus_index
                ),
            );
            let has_shift = modifiers.contains(&KeyModifier::Shift);
            match key_lower.as_str() {
                "tab" => {
                    if has_shift {
                        self.move_setup_button_focus(-1, cx);
                    } else {
                        self.move_setup_button_focus(1, cx);
                    }
                    return;
                }
                "up" | "arrowup" => {
                    self.move_setup_button_focus(-1, cx);
                    return;
                }
                "down" | "arrowdown" => {
                    self.move_setup_button_focus(1, cx);
                    return;
                }
                "enter" | "return" => {
                    match self.setup_button_focus_index {
                        0 => self.show_api_key_input(window, cx),
                        1 => self.enable_claude_code(window, cx),
                        _ => {}
                    }
                    return;
                }
                _ => {}
            }
        }

        // Handle API key input escape
        if self.showing_api_key_input && key_lower == "escape" {
            self.hide_api_key_input(window, cx);
            return;
        }

        // Default key handling (when no overlays are open)
        match key_lower.as_str() {
            "escape" | "esc" => {
                if self.showing_attachments_picker {
                    self.hide_attachments_picker(cx);
                }
            }
            _ => {
                crate::logging::log(
                    "AI",
                    &format!("SimulateKey: Unhandled key '{}' in AI window", key_lower),
                );
            }
        }
    }

    /// Copy the last AI response to clipboard
    pub(super) fn copy_last_response(&self, cx: &mut Context<Self>) {
        // Find the last assistant message
        if let Some(last_response) = self
            .current_messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::Assistant)
        {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                last_response.content.clone(),
            ));
            info!("Copied last response to clipboard");
        }
    }

    /// Copy the entire chat to clipboard
    pub(super) fn copy_entire_chat(&self, cx: &mut Context<Self>) {
        let chat_text: String = self
            .current_messages
            .iter()
            .map(|m| {
                let role = if m.role == MessageRole::User {
                    "You"
                } else {
                    "AI"
                };
                format!("**{}**: {}\n\n", role, m.content)
            })
            .collect();
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(chat_text));
        info!("Copied entire chat to clipboard");
    }

    /// Copy the last code block from AI response
    pub(super) fn copy_last_code_block(&self, cx: &mut Context<Self>) {
        // Find the last assistant message with a code block
        for msg in self.current_messages.iter().rev() {
            if msg.role == MessageRole::Assistant {
                // Simple regex-like search for code blocks
                if let Some(start) = msg.content.find("```") {
                    let after_start = &msg.content[start + 3..];
                    // Find the end of the language identifier (newline)
                    if let Some(lang_end) = after_start.find('\n') {
                        let code_start = &after_start[lang_end + 1..];
                        if let Some(end) = code_start.find("```") {
                            let code = &code_start[..end];
                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                code.to_string(),
                            ));
                            info!("Copied last code block to clipboard");
                            return;
                        }
                    }
                }
            }
        }
        info!("No code block found to copy");
    }

    /// Paste image from clipboard as attachment
    pub(super) fn paste_image_from_clipboard(&mut self, cx: &mut Context<Self>) {
        // Get the current clipboard text or image
        // Note: GPUI's clipboard API may not support raw image data directly
        // For now, we'll use a placeholder that can be enhanced later
        info!("Paste image from clipboard - checking for image data");
        // TODO: Implement proper image clipboard support when GPUI supports it
        cx.notify();
    }
}
