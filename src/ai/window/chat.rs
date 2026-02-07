use super::window_api::get_pending_chat;
use super::*;

impl AiApp {
    pub(super) fn initialize_with_pending_chat(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Take the pending messages from the global state
        let pending_messages = get_pending_chat()
            .lock()
            .ok()
            .and_then(|mut pending| pending.take());

        let messages = match pending_messages {
            Some(msgs) if !msgs.is_empty() => msgs,
            _ => {
                crate::logging::log("AI", "No pending messages to initialize chat with");
                return;
            }
        };

        crate::logging::log(
            "AI",
            &format!("Initializing chat with {} messages", messages.len()),
        );

        // Get model and provider from selected model, or use defaults
        let (model_id, provider) = self
            .selected_model
            .as_ref()
            .map(|m| (m.id.clone(), m.provider.clone()))
            .unwrap_or_else(|| {
                (
                    "claude-3-5-sonnet-20241022".to_string(),
                    "anthropic".to_string(),
                )
            });

        // Create a new chat with the ChatPrompt source
        let mut chat = Chat::new(&model_id, &provider);
        chat.source = ChatSource::ChatPrompt;
        let chat_id = chat.id;

        // Generate title from the first user message (if any)
        if let Some((_, content)) = messages.iter().find(|(role, _)| *role == MessageRole::User) {
            let title = Chat::generate_title_from_content(content);
            chat.set_title(&title);
        }

        // Save chat to storage
        if let Err(e) = storage::create_chat(&chat) {
            tracing::error!(error = %e, "Failed to create chat for transferred conversation");
            return;
        }

        // Save all messages to storage and build the current_messages list
        let mut saved_messages = Vec::new();
        for (role, content) in messages {
            let message = Message::new(chat_id, role, content);
            if let Err(e) = storage::save_message(&message) {
                tracing::error!(error = %e, "Failed to save message in transferred conversation");
                continue;
            }
            saved_messages.push(message);
        }

        // Update message preview and count with the last message
        if let Some(last_msg) = saved_messages.last() {
            let preview: String = last_msg.content.chars().take(60).collect();
            let preview = if preview.len() < last_msg.content.len() {
                format!("{}...", preview.trim())
            } else {
                preview
            };
            self.message_previews.insert(chat_id, preview);
        }
        self.message_counts.insert(chat_id, saved_messages.len());

        // Add chat to the list and select it
        self.chats.insert(0, chat);
        self.selected_chat_id = Some(chat_id);
        self.cache_message_images(&saved_messages);
        self.current_messages = saved_messages;

        // Force scroll to bottom when initializing with a transferred conversation
        self.force_scroll_to_bottom();

        info!(
            chat_id = %chat_id,
            message_count = self.current_messages.len(),
            "Chat initialized with transferred conversation"
        );

        cx.notify();
    }

    /// Create a new chat
    pub(super) fn create_chat(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ChatId> {
        // Get model and provider from selected model, or use defaults
        let (model_id, provider) = self
            .selected_model
            .as_ref()
            .map(|m| (m.id.clone(), m.provider.clone()))
            .unwrap_or_else(|| {
                (
                    "claude-3-5-sonnet-20241022".to_string(),
                    "anthropic".to_string(),
                )
            });

        // Create a new chat with selected model
        let chat = Chat::new(&model_id, &provider);
        let id = chat.id;

        // Save to storage
        if let Err(e) = storage::create_chat(&chat) {
            tracing::error!(error = %e, "Failed to create chat");
            return None;
        }

        // Add to cache and select it
        self.chats.insert(0, chat);
        self.select_chat(id, window, cx);

        info!(chat_id = %id, model = model_id, "New chat created");
        Some(id)
    }

    /// Select a chat
    pub(super) fn select_chat(&mut self, id: ChatId, window: &mut Window, cx: &mut Context<Self>) {
        // Save draft for outgoing chat
        self.save_draft(cx);

        // Clear any pending delete confirmation
        self.pending_delete_chat_id = None;

        self.selected_chat_id = Some(id);

        // Load messages for this chat
        self.current_messages = storage::get_chat_messages(&id).unwrap_or_default();
        self.cache_message_images(&self.current_messages.clone());

        // Sync selected_model with the chat's stored model (BYOK per chat)
        if let Some(chat) = self.chats.iter().find(|c| c.id == id) {
            // Find the model in available_models that matches the chat's model_id
            self.selected_model = self
                .available_models
                .iter()
                .find(|m| m.id == chat.model_id)
                .cloned();

            if self.selected_model.is_none() && !chat.model_id.is_empty() {
                // Chat has a model_id but it's not in our available models
                // (provider may not be configured). Log for debugging.
                tracing::debug!(
                    chat_id = %id,
                    model_id = %chat.model_id,
                    provider = %chat.provider,
                    "Chat's model not found in available models (provider may not be configured)"
                );
            }
        }

        // Force scroll to bottom when switching chats (always scroll)
        self.force_scroll_to_bottom();

        // Clear streaming state for display purposes, but don't clear streaming_chat_id/generation
        // The streaming task may still be running for the previous chat - it will be
        // ignored via the generation guard when it tries to update
        self.is_streaming = false;
        self.streaming_content.clear();
        // Note: streaming_chat_id and streaming_generation are NOT cleared here
        // This allows the background streaming to complete and save to DB correctly
        // while UI shows the newly selected chat's messages

        // Reset UX state for new chat
        self.editing_message_id = None;
        self.streaming_error = None;

        // Restore draft for incoming chat
        self.restore_draft(window, cx);

        // Update placeholder based on chat context
        self.update_input_placeholder(window, cx);

        cx.notify();
    }

    /// Update input placeholder text based on current context.
    /// Shows model name when in an active chat, generic text otherwise.
    pub(super) fn update_input_placeholder(&self, window: &mut Window, cx: &mut Context<Self>) {
        let placeholder = if !self.current_messages.is_empty() {
            if let Some(ref model) = self.selected_model {
                format!("Reply to {}...", model.display_name)
            } else {
                "Type a reply...".to_string()
            }
        } else if let Some(ref model) = self.selected_model {
            format!("Ask {}...", model.display_name)
        } else {
            "Ask anything...".to_string()
        };
        self.input_state.update(cx, |state, cx| {
            state.set_placeholder(placeholder, window, cx);
        });
    }

    /// Delete the currently selected chat (soft delete)
    pub(super) fn delete_selected_chat(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_chat_id {
            self.delete_chat_by_id(id, cx);
        }
    }
}
