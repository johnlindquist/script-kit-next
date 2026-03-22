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

        // Collect conversation history from messages, including image attachments
        let messages: Vec<ai::PendingChatMessage> = self
            .messages
            .iter()
            .map(|m| {
                let role = if m.is_user() {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                };
                ai::PendingChatMessage {
                    role,
                    content: m.get_content().to_string(),
                    image_base64: m.image.clone(),
                }
            })
            .collect();

        let message_count = messages.len();
        let image_count = messages.iter().filter(|m| m.image_base64.is_some()).count();
        tracing::info!(
            action = "continue_in_chat",
            message_count = message_count,
            image_count = image_count,
            "Transferring conversation to AI window"
        );

        // Save conversation before clearing inline state
        if self.save_history {
            self.save_to_database();
        }

        // Reset the inline prompt to empty state BEFORE the deferred AI open
        self.messages.clear();
        self.streaming_message_id = None;
        self.user_has_scrolled_up = false;
        self.input.clear();
        self.pending_image = None;
        self.pending_image_render = None;
        self.image_render_cache.clear();
        self.mark_conversation_turns_dirty();
        self.ensure_conversation_turns_cache();
        cx.notify();

        // Dismiss the main prompt window via escape callback
        if let Some(ref callback) = self.on_escape {
            callback(self.id.clone());
        }

        // Defer AI window open so the inline prompt dismisses first,
        // avoiding synchronous image transfer work on the original prompt path.
        cx.spawn(async move |_this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1))
                .await;

            let open_result = cx.update(|cx| {
                ai::open_ai_window(cx).map_err(|error| error.to_string())?;
                ai::set_ai_pending_chat(cx, messages)?;
                Ok::<(), String>(())
            });

            match open_result {
                Ok(()) => {
                    tracing::info!(
                        action = "continue_in_chat",
                        "AI window opened with deferred pending chat"
                    );
                }
                Err(error) => {
                    tracing::error!(error = %error, "Failed to open AI window for continue-in-chat");
                }
            }
        })
        .detach();
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

        // Notify parent to show CreationFeedback panel
        if let Some(ref callback) = self.on_script_saved {
            callback(script_path, cx);
        }
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
