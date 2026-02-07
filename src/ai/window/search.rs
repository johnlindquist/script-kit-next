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

    /// Handle search query changes - filters chats in real-time as user types
    pub(super) fn on_search_change(&mut self, cx: &mut Context<Self>) {
        let query = self.search_state.read(cx).value().to_string();
        self.search_query = query.clone();

        debug!(query = %query, "Search query changed");

        // If search is not empty, filter chats
        if !query.trim().is_empty() {
            let search_query = query.trim();
            let mut used_fallback = false;

            self.chats = match storage::search_chats(search_query) {
                Ok(chats) => chats,
                Err(error) => {
                    used_fallback = true;
                    tracing::warn!(
                        error = %error,
                        query = %search_query,
                        "Storage-backed chat search failed, falling back to title filter"
                    );
                    let query_lower = search_query.to_lowercase();
                    storage::get_all_chats()
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|chat| chat.title.to_lowercase().contains(&query_lower))
                        .collect()
                }
            };

            debug!(
                results = self.chats.len(),
                used_fallback = used_fallback,
                "Search filtered chats"
            );

            // Always select first result when filtering
            if !self.chats.is_empty() {
                let first_id = self.chats[0].id;
                if self.selected_chat_id != Some(first_id) {
                    self.selected_chat_id = Some(first_id);
                    // Load messages for the selected chat
                    self.current_messages =
                        storage::get_chat_messages(&first_id).unwrap_or_default();
                    self.cache_message_images(&self.current_messages.clone());
                }
            } else {
                self.selected_chat_id = None;
                self.current_messages = Vec::new();
            }
        } else {
            // Reload all chats when search is cleared
            self.chats = storage::get_all_chats().unwrap_or_default();
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
        }

        cx.notify();
    }
}
