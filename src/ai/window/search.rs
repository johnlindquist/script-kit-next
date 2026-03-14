use super::*;

impl AiApp {
    pub(super) fn on_model_change(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(model) = self.available_models.get(index) {
            info!(
                model_id = model.id,
                model_name = model.display_name,
                provider = model.provider,
                "Model selected"
            );
            self.selected_model = Some(model.clone());

            // Update the current chat's model in storage (BYOK per-chat)
            if let Some(chat_id) = self.selected_chat_id {
                if let Some(chat) = self.chats.iter_mut().find(|c| c.id == chat_id) {
                    chat.model_id = model.id.clone();
                    chat.provider = model.provider.clone();
                    chat.touch(); // Update updated_at

                    // Persist to database
                    if let Err(e) = storage::update_chat(chat) {
                        tracing::error!(error = %e, chat_id = %chat_id, "Failed to persist model change to chat");
                    }
                }
            }

            cx.notify();
        }
    }

    /// Update a chat's timestamp and move it to the top of the list
    ///
    /// Called after message activity to keep the chat list sorted by recency.
    pub(super) fn touch_and_reorder_chat(&mut self, chat_id: ChatId) {
        // Find the chat and update its timestamp
        if let Some(chat) = self.chats.iter_mut().find(|c| c.id == chat_id) {
            chat.touch(); // Updates updated_at to now

            // Persist the timestamp update to storage
            if let Err(e) = storage::update_chat(chat) {
                tracing::error!(error = %e, chat_id = %chat_id, "Failed to persist chat timestamp");
            }
        }

        // Reorder: move the active chat to the top
        if let Some(pos) = self.chats.iter().position(|c| c.id == chat_id) {
            if pos > 0 {
                let chat = self.chats.remove(pos);
                self.chats.insert(0, chat);
            }
        }
    }

    pub(super) fn current_streaming_session_key(&self) -> Option<StreamingSessionKey> {
        self.streaming_chat_id
            .map(|active_chat_id| StreamingSessionKey {
                chat_id: active_chat_id,
                generation: self.streaming_generation,
            })
    }

    pub(super) fn suppress_orphan_save_for_current_stream(&mut self, reason: &'static str) {
        if let Some(session_key) = self.current_streaming_session_key() {
            let correlation_id = format!(
                "ai-stream-{}-{}",
                session_key.chat_id, session_key.generation
            );
            self.suppressed_orphan_sessions.insert(session_key);
            info!(
                correlation_id = %correlation_id,
                chat_id = %session_key.chat_id,
                generation = session_key.generation,
                reason = reason,
                "Suppressing stale stream completion persistence"
            );
        }
    }

    pub(super) fn should_persist_orphaned_completion(
        &mut self,
        chat_id: ChatId,
        generation: u64,
    ) -> bool {
        let session_key = StreamingSessionKey {
            chat_id,
            generation,
        };
        should_persist_stale_completion(&mut self.suppressed_orphan_sessions, session_key)
    }

    /// Debounce delay for search input (milliseconds).
    /// Delays the DB query so rapid keystrokes don't fire a query per character.
    pub(crate) const SEARCH_DEBOUNCE_MS: u64 = 150;

    /// Handle search query changes - filters chats asynchronously as user types.
    ///
    /// Uses a 150ms debounce: each keystroke cancels the previous timer and starts
    /// a new one. Empty queries bypass the debounce for instant feedback.
    /// A generation counter discards stale results when the user types faster
    /// than the search completes.
    pub(super) fn on_search_change(&mut self, cx: &mut Context<Self>) {
        let query = self.search_state.read(cx).value().to_string();
        self.search_query = query.clone();
        self.search_generation += 1;
        let generation = self.search_generation;

        info!(
            query = %query,
            generation = generation,
            "search_query_changed"
        );

        // Empty query: synchronous clear, no debounce, instant UX
        if query.trim().is_empty() {
            // Cancel any pending debounce task
            self.search_debounce_task = None;

            self.chats = storage::get_all_chats().unwrap_or_default();
            self.search_snippets.clear();
            self.search_matched_title.clear();

            // Keep current selection if it still exists, otherwise select first
            if let Some(id) = self.selected_chat_id {
                if !self.chats.iter().any(|c| c.id == id) {
                    self.selected_chat_id = self.chats.first().map(|c| c.id);
                    if let Some(new_id) = self.selected_chat_id {
                        self.current_messages =
                            storage::get_chat_messages(&new_id).unwrap_or_default();
                        self.cache_message_images(&self.current_messages.clone());
                    }
                }
            }

            info!(generation = generation, "search_cleared_synchronous");

            cx.notify();
            return;
        }

        let search_query = query.trim().to_string();

        // Debounce: cancel previous task (by replacing it) and start a new timer.
        // Dropping the old Task cancels it via GPUI's task cancellation.
        let debounce_task = cx.spawn(async move |this, cx| {
            // Wait for the debounce interval
            cx.background_executor()
                .timer(std::time::Duration::from_millis(Self::SEARCH_DEBOUNCE_MS))
                .await;

            info!(
                generation = generation,
                query = %search_query,
                debounce_ms = Self::SEARCH_DEBOUNCE_MS,
                "search_debounce_fired"
            );

            // Run the actual DB search on background executor
            let query_for_search = search_query.clone();
            let results: anyhow::Result<Vec<storage::ChatSearchResult>> = cx
                .background_executor()
                .spawn(async move { storage::search_chats_with_snippets(&query_for_search) })
                .await;

            this.update(cx, |this, cx| {
                // Clear the debounce task handle now that we're executing
                this.search_debounce_task = None;

                // Discard stale results: only apply if generation still matches
                if this.search_generation != generation {
                    info!(
                        expected_generation = generation,
                        current_generation = this.search_generation,
                        "search_results_discarded_stale"
                    );
                    return;
                }

                match results {
                    Ok(search_results) => {
                        info!(
                            generation = generation,
                            count = search_results.len(),
                            "search_results_applied"
                        );

                        // Extract snippets and chats
                        this.search_snippets.clear();
                        this.search_matched_title.clear();
                        let mut chats = Vec::with_capacity(search_results.len());

                        for result in search_results {
                            let chat_id = result.chat.id;
                            if let Some(snippet) = result.match_snippet {
                                this.search_snippets.insert(chat_id, snippet);
                            }
                            this.search_matched_title
                                .insert(chat_id, result.matched_title);
                            chats.push(result.chat);
                        }

                        this.chats = chats;

                        // Select first result
                        if !this.chats.is_empty() {
                            let first_id = this.chats[0].id;
                            if this.selected_chat_id != Some(first_id) {
                                this.selected_chat_id = Some(first_id);
                                this.current_messages =
                                    storage::get_chat_messages(&first_id).unwrap_or_default();
                                this.cache_message_images(&this.current_messages.clone());
                            }
                        } else {
                            this.selected_chat_id = None;
                            this.current_messages = Vec::new();
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            generation = generation,
                            "search_failed"
                        );
                    }
                }

                cx.notify();
            })
            .ok();
        });

        // Store the task — this drops (cancels) any previously pending debounce
        self.search_debounce_task = Some(debounce_task);
    }
}
