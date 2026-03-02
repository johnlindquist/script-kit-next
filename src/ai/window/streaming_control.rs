use super::*;

fn ai_streaming_terminal_refresh_splice_range(
    previous_item_count: usize,
    next_item_count: usize,
) -> Option<std::ops::Range<usize>> {
    if next_item_count == 0 {
        return None;
    }

    if previous_item_count == next_item_count {
        let last_ix = next_item_count - 1;
        Some(last_ix..next_item_count)
    } else {
        Some(0..previous_item_count)
    }
}

impl AiApp {
    fn refresh_messages_list_after_streaming_state_change(&mut self) {
        let next_item_count = self.messages_list_item_count();
        let previous_item_count = self.messages_list_state.item_count();
        let Some(splice_range) =
            ai_streaming_terminal_refresh_splice_range(previous_item_count, next_item_count)
        else {
            return;
        };

        if previous_item_count == next_item_count {
            self.messages_list_state.splice(splice_range, 1);
        } else {
            self.messages_list_state
                .splice(splice_range, next_item_count);
        }
    }

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
        self.streaming_cancel = None;
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
            let mut accumulated = String::new();
            let mut delay_counter = 0u64;

            for word in words {
                // Vary delay slightly based on word position (30-60ms range)
                delay_counter = delay_counter.wrapping_add(17); // Simple pseudo-variation
                let delay = 30 + (delay_counter % 30);
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(delay))
                    .await;

                accumulated.push_str(&word);

                let current_content = accumulated.clone();
                let Some(this_entity) = this.upgrade() else {
                    return;
                };
                let (should_break, should_persist_orphan) = cx.update(|cx| {
                    this_entity.update(cx, |app, cx| {
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
                });

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
            cx.background_executor()
                .timer(std::time::Duration::from_millis(100))
                .await;

            // Finish streaming
            if let Some(this_entity) = this.upgrade() {
                cx.update(|cx| {
                    this_entity.update(cx, |app, cx| {
                        app.finish_streaming(chat_id, generation, cx);
                    })
                });
            }
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
            if self.selected_chat_id == Some(chat_id) {
                self.message_counts
                    .insert(chat_id, self.current_messages.len());
            } else {
                let current = self.message_counts.get(&chat_id).copied().unwrap_or(0);
                self.message_counts.insert(chat_id, current + 1);
            }

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
        self.streaming_cancel = None;
        self.streaming_started_at = None;
        self.refresh_messages_list_after_streaming_state_change();
        cx.notify();
    }

    /// Stop the current streaming response.
    pub(super) fn stop_streaming(&mut self, cx: &mut Context<Self>) {
        if !self.is_streaming {
            return;
        }

        if let Some(cancelled) = self.streaming_cancel.take() {
            cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
        }

        let chat_id = match self.streaming_chat_id {
            Some(id) => id,
            None => {
                self.is_streaming = false;
                self.streaming_content.clear();
                self.streaming_started_at = None;
                self.refresh_messages_list_after_streaming_state_change();
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
            if self.selected_chat_id == Some(chat_id) {
                self.message_counts
                    .insert(chat_id, self.current_messages.len());
            } else {
                let current = self.message_counts.get(&chat_id).copied().unwrap_or(0);
                self.message_counts.insert(chat_id, current + 1);
            }
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

            self.sync_chat_derived_state_from_current_messages(chat_id);
            cx.notify();
            self.force_scroll_to_bottom();
            info!(chat_id = %chat_id, "Regenerating response");
            self.start_streaming_response(chat_id, cx);
            cx.notify();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_message_count_uses_current_messages_len_for_selected_chat() {
        let chat_id = ChatId::new();
        let selected_chat_id = Some(chat_id);
        let current_messages_len = 4usize;
        let mut message_counts = std::collections::HashMap::new();
        message_counts.insert(chat_id, 10usize);

        if selected_chat_id == Some(chat_id) {
            message_counts.insert(chat_id, current_messages_len);
        } else {
            let current = message_counts.get(&chat_id).copied().unwrap_or(0);
            message_counts.insert(chat_id, current + 1);
        }

        assert_eq!(
            message_counts.get(&chat_id).copied(),
            Some(current_messages_len),
            "Selected chat should use in-memory current_messages length"
        );
    }

    #[test]
    fn test_streaming_message_count_increments_cached_count_for_non_selected_chat() {
        let chat_id = ChatId::new();
        let selected_chat_id = Some(ChatId::new());
        let current_messages_len = 99usize;
        let mut message_counts = std::collections::HashMap::new();
        message_counts.insert(chat_id, 3usize);

        if selected_chat_id == Some(chat_id) {
            message_counts.insert(chat_id, current_messages_len);
        } else {
            let current = message_counts.get(&chat_id).copied().unwrap_or(0);
            message_counts.insert(chat_id, current + 1);
        }

        assert_eq!(
            message_counts.get(&chat_id).copied(),
            Some(4usize),
            "Background chat should increment cached count instead of using selected chat length"
        );
    }

    #[test]
    fn test_streaming_message_count_starts_at_one_for_non_selected_chat_without_cache() {
        let chat_id = ChatId::new();
        let selected_chat_id = Some(ChatId::new());
        let current_messages_len = 0usize;
        let mut message_counts = std::collections::HashMap::new();

        if selected_chat_id == Some(chat_id) {
            message_counts.insert(chat_id, current_messages_len);
        } else {
            let current = message_counts.get(&chat_id).copied().unwrap_or(0);
            message_counts.insert(chat_id, current + 1);
        }

        assert_eq!(
            message_counts.get(&chat_id).copied(),
            Some(1usize),
            "Missing cache entry should initialize to one message for background completion"
        );
    }

    #[test]
    fn test_ai_streaming_terminal_refresh_splice_range_replaces_all_items_when_count_changes() {
        assert_eq!(ai_streaming_terminal_refresh_splice_range(4, 5), Some(0..4));
        assert_eq!(ai_streaming_terminal_refresh_splice_range(2, 1), Some(0..2));
    }

    #[test]
    fn test_ai_streaming_terminal_refresh_splice_range_invalidates_last_item_when_count_unchanged()
    {
        assert_eq!(ai_streaming_terminal_refresh_splice_range(5, 5), Some(4..5));
    }

    #[test]
    fn test_ai_streaming_terminal_refresh_splice_range_returns_none_when_list_becomes_empty() {
        assert_eq!(ai_streaming_terminal_refresh_splice_range(1, 0), None);
    }

    #[test]
    fn test_regenerate_removal_recomputes_preview_and_count_from_remaining_messages() {
        let chat_id = ChatId::new();
        let mut current_messages = vec![
            Message::user(chat_id, "question"),
            Message::assistant(chat_id, "answer"),
        ];
        let mut message_counts = std::collections::HashMap::new();
        let mut message_previews = std::collections::HashMap::new();

        let assistant_idx = current_messages
            .iter()
            .rposition(|message| message.role == MessageRole::Assistant);
        if let Some(index) = assistant_idx {
            current_messages.remove(index);
        }

        super::super::interactions::ai_window_recompute_chat_derived_state_from_messages(
            chat_id,
            &current_messages,
            &mut message_counts,
            &mut message_previews,
        );

        assert_eq!(
            message_counts.get(&chat_id).copied(),
            Some(1usize),
            "After regeneration removal, count should match the in-memory remaining messages"
        );
        assert_eq!(
            message_previews.get(&chat_id).map(String::as_str),
            Some("question"),
            "After regeneration removal, preview should point to the latest remaining message"
        );
    }
}
