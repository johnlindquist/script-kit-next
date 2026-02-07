use super::*;

impl ChatPrompt {
    pub(super) fn handle_escape(&mut self, _cx: &mut Context<Self>) {
        logging::log("CHAT", "Escape pressed - closing chat");

        // Save conversation to database if save_history is enabled
        if self.save_history {
            self.save_to_database();
        }

        if let Some(ref callback) = self.on_escape {
            callback(self.id.clone());
        }
    }

    /// Save the current conversation to the AI chats database
    pub(super) fn save_to_database(&self) {
        // Only save if we have messages
        if self.messages.is_empty() {
            logging::log("CHAT", "No messages to save");
            return;
        }

        // Initialize the AI database if needed
        if let Err(e) = ai::init_ai_db() {
            logging::log("CHAT", &format!("Failed to init AI db: {}", e));
            return;
        }

        // Generate title from first user message
        let title = self
            .messages
            .iter()
            .find(|m| m.is_user())
            .map(|m| Chat::generate_title_from_content(m.get_content()))
            .unwrap_or_else(|| "Chat Prompt Conversation".to_string());

        // Determine the model and provider
        let model_id = self.model.clone().unwrap_or_else(|| "unknown".to_string());
        let provider = self
            .models
            .iter()
            .find(|m| m.name == model_id || m.id == model_id)
            .map(|m| m.provider.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Create the chat record with ChatPrompt source
        let chat = Chat::new(&model_id, &provider).with_source(ChatSource::ChatPrompt);
        let mut chat = chat;
        chat.set_title(&title);

        // Save the chat
        if let Err(e) = ai::create_chat(&chat) {
            logging::log("CHAT", &format!("Failed to save chat: {}", e));
            return;
        }

        // Save all messages
        for (i, msg) in self.messages.iter().enumerate() {
            let role = if msg.is_user() {
                MessageRole::User
            } else {
                MessageRole::Assistant
            };

            let message = Message::new(chat.id, role, msg.get_content());
            if let Err(e) = ai::save_message(&message) {
                logging::log("CHAT", &format!("Failed to save message {}: {}", i, e));
            }
        }

        logging::log(
            "CHAT",
            &format!(
                "Saved conversation with {} messages (id: {})",
                self.messages.len(),
                chat.id
            ),
        );
    }

    pub fn handle_continue_in_chat(&mut self, cx: &mut Context<Self>) {
        logging::log("CHAT", "Continue in Chat - opening AI window");

        // Collect conversation history from messages
        let messages: Vec<(MessageRole, String)> = self
            .messages
            .iter()
            .map(|m| {
                let role = if m.is_user() {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                };
                (role, m.get_content().to_string())
            })
            .collect();

        logging::log(
            "CHAT",
            &format!("Transferring {} messages to AI window", messages.len()),
        );

        // Open AI window with the chat history
        if let Err(e) = ai::open_ai_window_with_chat(cx, messages) {
            logging::log("CHAT", &format!("Failed to open AI window: {}", e));
        }

        // Close this prompt by calling the escape callback
        if let Some(ref callback) = self.on_escape {
            callback(self.id.clone());
        }
    }

    pub fn handle_copy_last_response(&mut self, cx: &mut Context<Self>) {
        // Find the last assistant message
        if let Some(last_assistant) = self.messages.iter().rev().find(|m| !m.is_user()) {
            let content = last_assistant.get_content().to_string();
            self.last_copied_response = Some(content.clone());
            logging::log("CHAT", &format!("Copied response: {} chars", content.len()));
            // Copy to clipboard via cx
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(content));
        }
    }

    pub(super) fn handle_clear(&mut self, cx: &mut Context<Self>) {
        logging::log("CHAT", "Clearing conversation (⌘+⌫)");
        self.clear_messages(cx);
    }

    // ============================================
    // Actions Menu Methods
    // ============================================

    pub(super) fn toggle_actions_menu(&mut self, _cx: &mut Context<Self>) {
        // Delegate to parent via callback to open standard ActionsDialog
        if let Some(ref callback) = self.on_show_actions {
            logging::log("CHAT", "Requesting actions dialog via callback");
            callback(self.id.clone());
        } else {
            logging::log("CHAT", "No on_show_actions callback set");
        }
    }

    pub(super) fn close_actions_menu(&mut self, _cx: &mut Context<Self>) {
        // Actions menu is now handled by parent - nothing to do here
    }

    /// Get the list of action items for the menu
    pub(super) fn get_actions(&self) -> Vec<ChatAction> {
        vec![
            ChatAction::new("continue", "Continue in Chat", Some("⌘ ↵")),
            ChatAction::new("copy", "Copy Last Response", Some("⌘ C")),
            ChatAction::new("clear", "Clear Conversation", Some("⌘ ⌫")),
        ]
    }

    /// Get total selectable items (models + actions)
    pub(super) fn get_menu_item_count(&self) -> usize {
        self.models.len() + self.get_actions().len()
    }

    pub(super) fn actions_menu_up(&mut self, cx: &mut Context<Self>) {
        if self.actions_menu_selected > 0 {
            self.actions_menu_selected -= 1;
            cx.notify();
        }
    }

    pub(super) fn actions_menu_down(&mut self, cx: &mut Context<Self>) {
        let max = self.get_menu_item_count().saturating_sub(1);
        if self.actions_menu_selected < max {
            self.actions_menu_selected += 1;
            cx.notify();
        }
    }

    pub(super) fn actions_menu_select(&mut self, cx: &mut Context<Self>) {
        let selected = self.actions_menu_selected;
        let model_count = self.models.len();

        if selected < model_count {
            // Selected a model
            let model = &self.models[selected];
            self.model = Some(model.name.clone());
            logging::log("CHAT", &format!("Selected model: {}", model.name));
            self.close_actions_menu(cx);
        } else {
            // Selected an action
            let action_idx = selected - model_count;
            let actions = self.get_actions();
            if action_idx < actions.len() {
                let action = &actions[action_idx];
                logging::log("CHAT", &format!("Selected action: {}", action.id));
                match action.id.as_str() {
                    "continue" => {
                        self.close_actions_menu(cx);
                        self.handle_continue_in_chat(cx);
                    }
                    "copy" => {
                        self.handle_copy_last_response(cx);
                        self.close_actions_menu(cx);
                    }
                    "clear" => {
                        self.handle_clear(cx);
                        self.close_actions_menu(cx);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Handle clicking on a specific model in the menu
    pub(super) fn select_model_by_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.models.len() {
            let model = &self.models[index];
            self.model = Some(model.name.clone());
            logging::log("CHAT", &format!("Selected model: {}", model.name));
            self.close_actions_menu(cx);
        }
    }

    /// Handle clicking on a specific action in the menu
    pub(super) fn select_action_by_id(&mut self, action_id: &str, cx: &mut Context<Self>) {
        match action_id {
            "continue" => {
                self.close_actions_menu(cx);
                self.handle_continue_in_chat(cx);
            }
            "copy" => {
                self.handle_copy_last_response(cx);
                self.close_actions_menu(cx);
            }
            "clear" => {
                self.handle_clear(cx);
                self.close_actions_menu(cx);
            }
            _ => {}
        }
    }

    /// Render the actions menu overlay
    pub(super) fn render_actions_menu(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = &self.prompt_colors;
        let model_count = self.models.len();
        let current_model = self.model.clone().unwrap_or_default();
        // Check vibrancy to conditionally apply shadow
        // Uses cached theme to avoid file I/O on every render
        let vibrancy_enabled = crate::theme::get_cached_theme().is_vibrancy_enabled();

        let menu_bg = rgba((colors.code_bg << 8) | 0xF0);
        let hover_bg = rgba((colors.accent_color << 8) | 0x20);
        let selected_bg = rgba((colors.accent_color << 8) | 0x40);
        let border_color = rgba((colors.quote_border << 8) | 0x60);

        let mut menu = div()
            .absolute()
            .bottom(px(50.0)) // Position above footer
            .left(px(12.0))
            .right(px(12.0))
            .bg(menu_bg)
            .border_1()
            .border_color(border_color)
            .rounded(px(8.0))
            // Only apply shadow when vibrancy is disabled - shadows block blur
            .when(!vibrancy_enabled, |d| d.shadow_lg())
            .flex()
            .flex_col()
            .overflow_hidden();

        // Header
        menu = menu.child(
            div()
                .w_full()
                .px(px(12.0))
                .py(px(8.0))
                .border_b_1()
                .border_color(border_color)
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(colors.text_secondary))
                        .child("Actions"),
                )
                .child(
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .bg(rgba((colors.code_bg << 8) | 0x80))
                        .rounded(px(4.0))
                        .text_xs()
                        .text_color(rgb(colors.text_tertiary))
                        .child("⌘ K"),
                ),
        );

        // Models section
        for (i, model) in self.models.iter().enumerate() {
            let is_selected = i == self.actions_menu_selected;
            let is_current = model.name == current_model;

            let row_bg = if is_selected { Some(selected_bg) } else { None };

            let model_name = model.name.clone();
            let index = i;

            menu = menu.child(
                div()
                    .id(format!("model-{}", i))
                    .w_full()
                    .px(px(12.0))
                    .py(px(8.0))
                    .when_some(row_bg, |d, bg| d.bg(bg))
                    .hover(|s| s.bg(hover_bg))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _window, cx| {
                        this.select_model_by_index(index, cx);
                    }))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                // Radio button
                                div()
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .rounded_full()
                                    .border_1()
                                    .border_color(if is_current {
                                        rgb(colors.accent_color)
                                    } else {
                                        rgb(colors.text_tertiary)
                                    })
                                    .when(is_current, |d| {
                                        d.child(
                                            div()
                                                .w(px(8.0))
                                                .h(px(8.0))
                                                .m(px(2.0))
                                                .rounded_full()
                                                .bg(rgb(colors.accent_color)),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(colors.text_primary))
                                    .child(model_name),
                            ),
                    )
                    .when(is_current, |d| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(rgb(colors.text_tertiary))
                                .child("✓"),
                        )
                    }),
            );
        }

        // Separator
        menu = menu.child(div().w_full().h(px(1.0)).bg(border_color));

        // Actions section
        let actions = self.get_actions();
        for (i, action) in actions.iter().enumerate() {
            if action.is_separator {
                menu = menu.child(div().w_full().h(px(1.0)).bg(border_color));
                continue;
            }

            let menu_index = model_count + i;
            let is_selected = menu_index == self.actions_menu_selected;

            let row_bg = if is_selected { Some(selected_bg) } else { None };

            let action_id = action.id.clone();
            let action_label = action.label.clone();
            let shortcut = action.shortcut.clone();

            menu = menu.child(
                div()
                    .id(format!("action-{}", i))
                    .w_full()
                    .px(px(12.0))
                    .py(px(8.0))
                    .when_some(row_bg, |d, bg| d.bg(bg))
                    .hover(|s| s.bg(hover_bg))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _window, cx| {
                        this.select_action_by_id(&action_id, cx);
                    }))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(colors.text_primary))
                            .child(action_label),
                    )
                    .when_some(shortcut, |d, s| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(rgb(colors.text_tertiary))
                                .child(s),
                        )
                    }),
            );
        }

        menu
    }
}
