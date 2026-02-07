use super::*;
use crate::ai::model::ImageAttachment;
use crate::ai::providers::{ProviderImage, ProviderMessage};

impl AiApp {
    pub(super) fn submit_message(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // If we are in editing mode, delegate to the edit-submit flow
        if self.editing_message_id.is_some() {
            self.submit_edited_message(window, cx);
            return;
        }

        let content = self.input_state.read(cx).value().to_string();
        let has_pending_image = self.pending_image.is_some();

        if !ai_window_can_submit_message(&content, has_pending_image) {
            return;
        }

        // Don't allow new messages while streaming for the CURRENT chat
        // (streaming for a different chat is fine - the guard handles it)
        if self.is_streaming && self.streaming_chat_id == self.selected_chat_id {
            return;
        }

        // If no chat selected, create a new one
        let chat_id = if let Some(id) = self.selected_chat_id {
            id
        } else {
            match self.create_chat(window, cx) {
                Some(id) => id,
                None => {
                    tracing::error!("Failed to create chat for message submission");
                    return;
                }
            }
        };

        // Capture pending image only after all early-return guards so we don't drop attachments.
        let pending_image = self.pending_image.take();
        let has_image = pending_image.is_some();

        if let Some(ref image_base64) = pending_image {
            // Calculate approximate image size for logging
            let image_size_kb = image_base64.len() / 1024;
            crate::logging::log(
                "AI",
                &format!("Message includes attached image (~{}KB)", image_size_kb),
            );
        }

        // Update chat title if this is the first message
        if let Some(chat) = self.chats.iter_mut().find(|c| c.id == chat_id) {
            if chat.title == "New Chat" {
                let new_title = if content.trim().is_empty() && has_image {
                    "Image attachment".to_string()
                } else {
                    Chat::generate_title_from_content(&content)
                };
                chat.set_title(&new_title);

                // Persist title update
                if let Err(e) = storage::update_chat_title(&chat_id, &new_title) {
                    tracing::error!(error = %e, "Failed to update chat title");
                }
            }
        }

        // Create and save user message with optional image
        let mut user_message = Message::user(chat_id, &content);

        // Attach image if present
        if let Some(image_base64) = pending_image {
            user_message.images.push(ImageAttachment::png(image_base64));
        }

        if let Err(e) = storage::save_message(&user_message) {
            tracing::error!(error = %e, "Failed to save user message");
            return;
        }

        // Add to current messages for display
        self.current_messages.push(user_message);

        // Force scroll to bottom when user sends a new message (always scroll, even if scrolled up)
        self.force_scroll_to_bottom();

        // Update message preview and count cache
        let preview_source = if content.trim().is_empty() && has_image {
            "Image attachment"
        } else {
            content.as_str()
        };
        let preview: String = preview_source.chars().take(60).collect();
        let preview = if preview.len() < preview_source.len() {
            format!("{}...", preview.trim())
        } else {
            preview
        };
        self.message_previews.insert(chat_id, preview);
        self.message_counts
            .insert(chat_id, self.current_messages.len());

        // Update chat timestamp and move to top of list
        self.touch_and_reorder_chat(chat_id);

        // Clear the input (pending image was already taken above)
        // Explicitly reset cursor to position 0 to fix cursor placement with placeholder
        self.input_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
            state.set_selection(0, 0, window, cx);
        });

        // Update placeholder to "Reply to..." now that we have messages
        self.update_input_placeholder(window, cx);

        info!(
            chat_id = %chat_id,
            content_len = content.len(),
            has_image = has_image,
            "User message submitted"
        );

        // Start streaming response
        self.start_streaming_response(chat_id, cx);

        cx.notify();
    }

    /// Start streaming an AI response (or mock response if no providers configured)
    pub(super) fn start_streaming_response(&mut self, chat_id: ChatId, cx: &mut Context<Self>) {
        // Check if we have a model selected - if not, use mock mode
        let use_mock_mode = self.selected_model.is_none() || self.available_models.is_empty();

        if use_mock_mode {
            info!(chat_id = %chat_id, "No AI providers configured - using mock mode");
            self.start_mock_streaming_response(chat_id, cx);
            return;
        }

        // Get the selected model
        let model = match &self.selected_model {
            Some(m) => m.clone(),
            None => {
                tracing::error!("No model selected for streaming");
                return;
            }
        };

        // Find the provider for this model
        let provider = match self.provider_registry.find_provider_for_model(&model.id) {
            Some(p) => p.clone(),
            None => {
                tracing::error!(model_id = model.id, "No provider found for model");
                return;
            }
        };

        // Build messages for the API call
        let api_messages: Vec<ProviderMessage> = self
            .current_messages
            .iter()
            .map(|m| ProviderMessage {
                role: m.role.to_string(),
                content: m.content.clone(),
                images: m
                    .images
                    .iter()
                    .map(|img| ProviderImage {
                        data: img.data.clone(),
                        media_type: img.media_type.clone(),
                    })
                    .collect(),
            })
            .collect();

        // Set streaming state with chat-scoping guards
        self.is_streaming = true;
        self.streaming_content.clear();
        self.streaming_error = None;
        self.streaming_chat_id = Some(chat_id);
        self.streaming_generation = self.streaming_generation.wrapping_add(1);
        self.streaming_started_at = Some(std::time::Instant::now());
        let generation = self.streaming_generation;

        info!(
            chat_id = %chat_id,
            generation = generation,
            model = model.id,
            provider = model.provider,
            message_count = api_messages.len(),
            "Starting AI streaming response"
        );

        // Use a shared buffer for streaming content
        let shared_content = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let shared_done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shared_error = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));

        let model_id = model.id.clone();
        let content_clone = shared_content.clone();
        let done_clone = shared_done.clone();
        let error_clone = shared_error.clone();
        // Use chat_id as session_id for Claude Code CLI conversation continuity
        let session_id = chat_id.to_string();

        // Spawn background thread for streaming
        std::thread::spawn(move || {
            let result = provider.stream_message(
                &api_messages,
                &model_id,
                Box::new(move |chunk| {
                    if let Ok(mut content) = content_clone.lock() {
                        content.push_str(&chunk);
                    }
                }),
                Some(&session_id),
            );

            match result {
                Ok(()) => {
                    done_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                Err(e) => {
                    if let Ok(mut err) = error_clone.lock() {
                        *err = Some(e.to_string());
                    }
                    done_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                }
            }
        });

        // Poll for streaming updates using background executor
        let content_for_poll = shared_content.clone();
        let done_for_poll = shared_done.clone();
        let error_for_poll = shared_error.clone();

        cx.spawn(async move |this, cx| {
            use gpui::Timer;
            loop {
                Timer::after(std::time::Duration::from_millis(50)).await;

                // Check if done or errored
                if done_for_poll.load(std::sync::atomic::Ordering::SeqCst) {
                    // Get final content
                    let final_content = content_for_poll.lock().ok().map(|c| c.clone());
                    let error = error_for_poll.lock().ok().and_then(|e| e.clone());

                    let _ = cx.update(|cx| {
                        this.update(cx, |app, cx| {
                            // CRITICAL: Guard against stale updates from chat-switch
                            // If generation doesn't match, this is an old streaming task
                            if app.streaming_generation != generation
                                || app.streaming_chat_id != Some(chat_id)
                            {
                                tracing::debug!(
                                    expected_gen = generation,
                                    actual_gen = app.streaming_generation,
                                    expected_chat = %chat_id,
                                    actual_chat = ?app.streaming_chat_id,
                                    "Ignoring stale streaming completion (user switched chats)"
                                );
                                let should_persist =
                                    app.should_persist_orphaned_completion(chat_id, generation);

                                if !should_persist {
                                    tracing::info!(
                                        chat_id = %chat_id,
                                        generation = generation,
                                        "Dropping stale completion after explicit stop/delete"
                                    );
                                    return;
                                }

                                // Persist stale completion for chat-switch continuity.
                                if let Some(err) = &error {
                                    tracing::error!(error = %err, chat_id = %chat_id, "Stale streaming error");
                                } else if let Some(content) = &final_content {
                                    // Save orphaned message to DB
                                    if !content.is_empty() {
                                        let assistant_message =
                                            Message::assistant(chat_id, content);
                                        if let Err(e) = storage::save_message(&assistant_message) {
                                            tracing::error!(error = %e, "Failed to save orphaned assistant message");
                                        } else {
                                            tracing::info!(
                                                chat_id = %chat_id,
                                                content_len = content.len(),
                                                "Orphaned streaming response saved to DB"
                                            );
                                        }
                                    }
                                }
                                return;
                            }

                            if let Some(err) = error {
                                tracing::error!(error = %err, "Streaming error");
                                app.streaming_error = Some(err);
                                app.streaming_started_at = None;
                                app.is_streaming = false;
                                app.streaming_content.clear();
                                app.streaming_chat_id = None;
                            } else if let Some(content) = final_content {
                                app.streaming_content = content;
                                app.finish_streaming(chat_id, generation, cx);
                            }
                            cx.notify();
                        })
                    });
                    break;
                }

                // Update with current content (only if generation matches)
                if let Ok(content) = content_for_poll.lock() {
                    if !content.is_empty() {
                        let current = content.clone();
                        let _ = cx.update(|cx| {
                            this.update(cx, |app, cx| {
                                // Guard: only update UI if this is the current streaming session
                                if app.streaming_generation != generation
                                    || app.streaming_chat_id != Some(chat_id)
                                {
                                    return; // Stale update, ignore
                                }
                                app.streaming_content = current;
                                // Auto-scroll to bottom as new content arrives
                                app.sync_messages_list_and_scroll_to_bottom();
                                cx.notify();
                            })
                        });
                    }
                }
            }
        })
        .detach();
    }
}
