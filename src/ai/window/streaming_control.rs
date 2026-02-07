use super::*;

impl AiApp {
    pub(super) fn start_mock_streaming_response(
        &mut self,
        chat_id: ChatId,
        cx: &mut Context<Self>,
    ) {
        // Set streaming state with chat-scoping guards
        self.is_streaming = true;
        self.streaming_content.clear();
        self.streaming_chat_id = Some(chat_id);
        self.streaming_generation = self.streaming_generation.wrapping_add(1);
        self.streaming_started_at = Some(std::time::Instant::now());
        let generation = self.streaming_generation;

        // Get the last user message to generate a contextual mock response
        let user_message = self
            .current_messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();

        // Generate a mock response based on the user's message
        let mock_response = generate_mock_response(&user_message);

        info!(
            chat_id = %chat_id,
            generation = generation,
            user_message_len = user_message.len(),
            mock_response_len = mock_response.len(),
            "Starting mock streaming response"
        );

        // Simulate streaming by revealing the response word by word
        let words: Vec<String> = mock_response
            .split_inclusive(char::is_whitespace)
            .map(|s| s.to_string())
            .collect();

        cx.spawn(async move |this, cx| {
            use gpui::Timer;

            let mut accumulated = String::new();
            let mut delay_counter = 0u64;

            for word in words {
                // Vary delay slightly based on word position (30-60ms range)
                delay_counter = delay_counter.wrapping_add(17); // Simple pseudo-variation
                let delay = 30 + (delay_counter % 30);
                Timer::after(std::time::Duration::from_millis(delay)).await;

                accumulated.push_str(&word);

                let current_content = accumulated.clone();
                let (should_break, should_persist_orphan) = cx
                    .update(|cx| {
                        this.update(cx, |app, cx| {
                            // Guard: only update UI if this is the current streaming session
                            if app.streaming_generation != generation
                                || app.streaming_chat_id != Some(chat_id)
                            {
                                let should_persist =
                                    app.should_persist_orphaned_completion(chat_id, generation);
                                return (true, should_persist); // stale session
                            }
                            app.streaming_content = current_content;
                            // Auto-scroll to bottom as new content arrives
                            app.sync_messages_list_and_scroll_to_bottom();
                            cx.notify();
                            (false, false)
                        })
                        .unwrap_or((true, false))
                    })
                    .unwrap_or((true, false));

                if should_break {
                    // Session was superseded. Persist only when not explicitly suppressed.
                    if should_persist_orphan && !accumulated.is_empty() {
                        let assistant_message = Message::assistant(chat_id, &accumulated);
                        if let Err(e) = storage::save_message(&assistant_message) {
                            tracing::error!(error = %e, "Failed to save orphaned mock message");
                        } else {
                            tracing::info!(
                                chat_id = %chat_id,
                                content_len = accumulated.len(),
                                "Orphaned mock streaming saved to DB"
                            );
                        }
                    } else if !should_persist_orphan {
                        tracing::info!(
                            chat_id = %chat_id,
                            generation = generation,
                            "Dropping stale mock completion after explicit stop/delete"
                        );
                    }
                    return;
                }
            }

            // Small delay before finishing
            Timer::after(std::time::Duration::from_millis(100)).await;

            // Finish streaming
            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    app.finish_streaming(chat_id, generation, cx);
                })
            });
        })
        .detach();
    }

    /// Finish streaming and save the assistant message
    ///
    /// The `generation` parameter guards against stale completion calls.
    /// If the generation doesn't match, this is an orphaned streaming task
    /// and we should not update UI (message was already saved to DB by the guard).
    pub(super) fn finish_streaming(
        &mut self,
        chat_id: ChatId,
        generation: u64,
        cx: &mut Context<Self>,
    ) {
        // Guard: verify this is still the current streaming session
        if self.streaming_generation != generation || self.streaming_chat_id != Some(chat_id) {
            tracing::debug!(
                expected_gen = generation,
                actual_gen = self.streaming_generation,
                "finish_streaming called with stale generation, ignoring"
            );
            return;
        }

        if !self.streaming_content.is_empty() {
            // Create and save assistant message
            let assistant_message = Message::assistant(chat_id, &self.streaming_content);
            if let Err(e) = storage::save_message(&assistant_message) {
                tracing::error!(error = %e, "Failed to save assistant message");
            }

            // Add to current messages (only if viewing this chat)
            if self.selected_chat_id == Some(chat_id) {
                self.current_messages.push(assistant_message);
            }

            // Update message preview and count
            let preview: String = self.streaming_content.chars().take(60).collect();
            let preview = if preview.len() < self.streaming_content.len() {
                format!("{}...", preview.trim())
            } else {
                preview
            };
            self.message_previews.insert(chat_id, preview);
            self.message_counts
                .insert(chat_id, self.current_messages.len());

            // Update chat timestamp and move to top of list
            self.touch_and_reorder_chat(chat_id);

            info!(
                chat_id = %chat_id,
                content_len = self.streaming_content.len(),
                "Streaming response complete"
            );
        }

        // Capture streaming duration for "Generated in Xs" feedback
        if let Some(started) = self.streaming_started_at {
            self.last_streaming_duration = Some(started.elapsed());
            self.last_streaming_completed_at = Some(std::time::Instant::now());
        }

        self.is_streaming = false;
        self.streaming_content.clear();
        self.streaming_chat_id = None;
        self.streaming_started_at = None;
        cx.notify();
    }

    /// Stop the current streaming response.
    pub(super) fn stop_streaming(&mut self, cx: &mut Context<Self>) {
        if !self.is_streaming {
            return;
        }

        let chat_id = match self.streaming_chat_id {
            Some(id) => id,
            None => {
                self.is_streaming = false;
                self.streaming_content.clear();
                self.streaming_started_at = None;
                cx.notify();
                return;
            }
        };

        if !self.streaming_content.is_empty() {
            let assistant_message = Message::assistant(chat_id, &self.streaming_content);
            if let Err(e) = storage::save_message(&assistant_message) {
                tracing::error!(error = %e, "Failed to save partial assistant message on stop");
            }

            if self.selected_chat_id == Some(chat_id) {
                self.current_messages.push(assistant_message);
            }

            let preview: String = self.streaming_content.chars().take(60).collect();
            let preview = if preview.len() < self.streaming_content.len() {
                format!("{}...", preview.trim())
            } else {
                preview
            };
            self.message_previews.insert(chat_id, preview);
            self.message_counts
                .insert(chat_id, self.current_messages.len());
            self.touch_and_reorder_chat(chat_id);

            info!(
                chat_id = %chat_id,
                content_len = self.streaming_content.len(),
                "Streaming stopped by user - partial response saved"
            );
        } else {
            info!(chat_id = %chat_id, "Streaming stopped by user - no content to save");
        }

        // Capture streaming duration for "Generated in Xs" feedback
        if let Some(started) = self.streaming_started_at {
            self.last_streaming_duration = Some(started.elapsed());
            self.last_streaming_completed_at = Some(std::time::Instant::now());
        }

        self.suppress_orphan_save_for_current_stream("user_stop");
        self.streaming_generation = self.streaming_generation.wrapping_add(1);
        self.is_streaming = false;
        self.streaming_content.clear();
        self.streaming_chat_id = None;
        self.streaming_started_at = None;
        self.force_scroll_to_bottom();
        cx.notify();
    }

    /// Regenerate the last assistant response.
    pub(super) fn regenerate_response(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_streaming {
            return;
        }

        let chat_id = match self.selected_chat_id {
            Some(id) => id,
            None => return,
        };

        let last_assistant_idx = self
            .current_messages
            .iter()
            .rposition(|m| m.role == MessageRole::Assistant);

        if let Some(assistant_idx) = last_assistant_idx {
            let removed_msg = self.current_messages.remove(assistant_idx);
            if let Err(e) = storage::delete_message(&removed_msg.id) {
                tracing::error!(error = %e, "Failed to delete assistant message for regeneration");
            }

            self.force_scroll_to_bottom();
            info!(chat_id = %chat_id, "Regenerating response");
            self.start_streaming_response(chat_id, cx);
            cx.notify();
        }
    }
}
