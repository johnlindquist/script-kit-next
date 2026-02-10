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

    pub(super) fn handle_script_generation_action(
        &mut self,
        action: ScriptGenerationAction,
        cx: &mut Context<Self>,
    ) {
        let Some((prompt_description, raw_response)) = self.latest_script_generation_draft() else {
            self.set_script_generation_status(true, "No generated script to save yet.", cx);
            return;
        };

        logging::log(
            "CHAT_SCRIPT_GEN",
            &format!(
                "state=save_requested action={:?} prompt_len={} response_len={}",
                action,
                prompt_description.len(),
                raw_response.len()
            ),
        );

        let script_path = match crate::ai::script_generation::save_generated_script_from_response(
            &prompt_description,
            &raw_response,
        ) {
            Ok(path) => path,
            Err(error) => {
                self.set_script_generation_status(
                    true,
                    format!("Failed to save script: {}", error),
                    cx,
                );
                return;
            }
        };

        let script_name = script_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "generated script".to_string());

        if action.should_run_after_save() {
            self.set_script_generation_status(false, format!("Running {}...", script_name), cx);
            if let Some(ref callback) = self.on_run_script {
                callback(script_path.clone(), cx);
                self.set_script_generation_status(
                    false,
                    format!("Saved and running {}", script_name),
                    cx,
                );
                logging::log(
                    "CHAT_SCRIPT_GEN",
                    &format!(
                        "state=run_dispatched action={:?} path={}",
                        action,
                        script_path.display()
                    ),
                );
            } else {
                self.set_script_generation_status(
                    true,
                    format!("Saved {} but run action is unavailable", script_name),
                    cx,
                );
                logging::log(
                    "CHAT_SCRIPT_GEN",
                    &format!(
                        "state=run_dispatch_failed action={:?} path={} reason=missing_callback",
                        action,
                        script_path.display()
                    ),
                );
            }
            return;
        }

        self.set_script_generation_status(false, format!("Saved {}", script_name), cx);
        logging::log(
            "CHAT_SCRIPT_GEN",
            &format!(
                "state=saved_only action={:?} path={}",
                action,
                script_path.display()
            ),
        );
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
}
