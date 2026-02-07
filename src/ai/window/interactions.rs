use super::*;

impl AiApp {
    pub(super) fn hide_mouse_cursor(&mut self, cx: &mut Context<Self>) {
        if !self.mouse_cursor_hidden {
            self.mouse_cursor_hidden = true;
            crate::platform::hide_cursor_until_mouse_moves();
            cx.notify();
        }
    }

    /// Show the mouse cursor when mouse moves.
    pub(super) fn show_mouse_cursor(&mut self, cx: &mut Context<Self>) {
        if self.mouse_cursor_hidden {
            self.mouse_cursor_hidden = false;
            cx.notify();
        }
    }

    /// Check if a message was recently copied (within 2 seconds)
    pub(super) fn is_message_copied(&self, msg_id: &str) -> bool {
        if let (Some(ref copied_id), Some(copied_at)) = (&self.copied_message_id, self.copied_at) {
            copied_id == msg_id && copied_at.elapsed() < std::time::Duration::from_millis(2000)
        } else {
            false
        }
    }

    /// Copy message content and show checkmark feedback for 2 seconds
    pub(super) fn copy_message(&mut self, msg_id: String, content: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(content));
        self.copied_message_id = Some(msg_id);
        self.copied_at = Some(std::time::Instant::now());
        cx.notify();

        // Reset feedback after 2 seconds
        cx.spawn(async move |this, cx| {
            gpui::Timer::after(std::time::Duration::from_millis(2000)).await;
            let _ = cx.update(|cx| {
                this.update(cx, |this, cx| {
                    this.copied_message_id = None;
                    this.copied_at = None;
                    cx.notify();
                })
            });
        })
        .detach();
    }

    /// Copy the last assistant response to the clipboard (Cmd+Shift+C).
    pub(super) fn copy_last_assistant_response(&mut self, cx: &mut Context<Self>) {
        if let Some(last_assistant) = self
            .current_messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::Assistant)
        {
            let content = last_assistant.content.clone();
            let msg_id = last_assistant.id.clone();
            self.copy_message(msg_id, content, cx);
        }
    }

    // === UX Batch 5 Methods ===

    /// Toggle the keyboard shortcuts overlay (Cmd+/).
    pub(super) fn toggle_shortcuts_overlay(&mut self, cx: &mut Context<Self>) {
        self.showing_shortcuts_overlay = !self.showing_shortcuts_overlay;
        cx.notify();
    }

    /// Export the current chat as markdown to the clipboard (Cmd+Shift+E).
    pub(super) fn export_chat_to_clipboard(&mut self, cx: &mut Context<Self>) {
        let chat = match self.get_selected_chat() {
            Some(c) => c.clone(),
            None => return,
        };

        let title = if chat.title.is_empty() {
            "New Chat"
        } else {
            &chat.title
        };

        let mut md = format!("# {}\n\n", title);
        md.push_str(&format!(
            "_Model: {} | Provider: {} | Created: {}_\n\n---\n\n",
            chat.model_id,
            chat.provider,
            chat.created_at.format("%Y-%m-%d %H:%M")
        ));

        for msg in &self.current_messages {
            let role_label = match msg.role {
                MessageRole::User => "**You**",
                MessageRole::Assistant => "**Assistant**",
                MessageRole::System => "**System**",
            };
            md.push_str(&format!("{}\n\n{}\n\n---\n\n", role_label, msg.content));
        }

        cx.write_to_clipboard(gpui::ClipboardItem::new_string(md));
        self.export_copied_at = Some(std::time::Instant::now());
        cx.notify();

        // Reset feedback after 2 seconds
        cx.spawn(async move |this, cx| {
            gpui::Timer::after(std::time::Duration::from_millis(2000)).await;
            let _ = cx.update(|cx| {
                this.update(cx, |this, cx| {
                    this.export_copied_at = None;
                    cx.notify();
                })
            });
        })
        .detach();
    }

    /// Check if the export feedback is currently showing.
    pub(super) fn is_showing_export_feedback(&self) -> bool {
        self.export_copied_at
            .is_some_and(|at| at.elapsed() < std::time::Duration::from_millis(2000))
    }

    /// Toggle collapse state of a message.
    pub(super) fn toggle_message_collapse(&mut self, msg_id: String, cx: &mut Context<Self>) {
        if self.expanded_messages.contains(&msg_id) {
            self.expanded_messages.remove(&msg_id);
            self.collapsed_messages.insert(msg_id);
        } else if self.collapsed_messages.contains(&msg_id) {
            self.collapsed_messages.remove(&msg_id);
            self.expanded_messages.insert(msg_id);
        } else {
            // Message was auto-collapsed; expand it
            self.expanded_messages.insert(msg_id);
        }
        cx.notify();
    }

    /// Whether a message should be shown collapsed (auto-collapse long messages).
    /// Messages over 800 chars are auto-collapsed unless the user expanded them.
    pub(super) fn is_message_collapsed(&self, msg_id: &str, content_len: usize) -> bool {
        if self.expanded_messages.contains(msg_id) {
            return false;
        }
        if self.collapsed_messages.contains(msg_id) {
            return true;
        }
        // Auto-collapse messages longer than 800 chars
        content_len > 800
    }

    /// Build the visible message body when no collapse is applied.
    pub(super) fn message_body_content(content: &str) -> String {
        content.to_string()
    }

    /// Navigate to the previous (-1) or next (+1) chat in the sidebar list.
    pub(super) fn navigate_chat(
        &mut self,
        direction: i32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.chats.is_empty() {
            return;
        }

        let current_index = self
            .selected_chat_id
            .and_then(|id| self.chats.iter().position(|c| c.id == id))
            .unwrap_or(0);

        let new_index = if direction < 0 {
            // Navigate to previous (older) chat
            if current_index + 1 < self.chats.len() {
                current_index + 1
            } else {
                current_index // Already at the end
            }
        } else {
            // Navigate to next (newer) chat
            current_index.saturating_sub(1)
        };

        if new_index != current_index {
            let new_id = self.chats[new_index].id;
            self.select_chat(new_id, window, cx);
            cx.notify();
        }
    }

    /// Delete the currently selected chat (Cmd+Shift+Backspace).
    pub(super) fn delete_current_chat(&mut self, cx: &mut Context<Self>) {
        if let Some(chat_id) = self.selected_chat_id {
            self.delete_chat_by_id(chat_id, cx);
        }
    }

    /// Delete a specific chat by ID (for sidebar delete buttons)
    pub(super) fn delete_chat_by_id(&mut self, chat_id: ChatId, cx: &mut Context<Self>) {
        let deleted_messages = storage::get_chat_messages(&chat_id).unwrap_or_default();
        let deleted_message_ids: Vec<String> = deleted_messages
            .iter()
            .map(|message| message.id.clone())
            .collect();
        let deleted_message_id_set: std::collections::HashSet<String> =
            deleted_message_ids.iter().cloned().collect();
        let deleted_image_cache_keys: std::collections::HashSet<String> = deleted_messages
            .iter()
            .flat_map(|message| message.images.iter())
            .map(|image| Self::image_cache_key(&image.data))
            .collect();

        if self.streaming_chat_id == Some(chat_id) {
            self.suppress_orphan_save_for_current_stream("chat_deleted");
            self.is_streaming = false;
            self.streaming_content.clear();
            self.streaming_chat_id = None;
            self.streaming_started_at = None;
        }

        if let Err(e) = storage::delete_chat(&chat_id) {
            tracing::error!(error = %e, "Failed to delete chat");
            return;
        }

        // Remove from visible list
        self.chats.retain(|c| c.id != chat_id);
        self.message_previews.remove(&chat_id);
        self.message_counts.remove(&chat_id);
        self.chat_drafts.remove(&chat_id);

        if self.pending_delete_chat_id == Some(chat_id) {
            self.pending_delete_chat_id = None;
        }
        if self.renaming_chat_id == Some(chat_id) {
            self.renaming_chat_id = None;
        }
        if self
            .editing_message_id
            .as_ref()
            .is_some_and(|message_id| deleted_message_id_set.contains(message_id))
        {
            self.editing_message_id = None;
        }

        ai_window_prune_deleted_message_ui_state(
            &mut self.collapsed_messages,
            &mut self.expanded_messages,
            &deleted_message_ids,
        );

        for cache_key in deleted_image_cache_keys {
            self.image_cache.remove(&cache_key);
        }

        // If we deleted the selected chat, select next
        if self.selected_chat_id == Some(chat_id) {
            self.selected_chat_id = self.chats.first().map(|c| c.id);
            self.current_messages = self
                .selected_chat_id
                .and_then(|new_id| storage::get_chat_messages(&new_id).ok())
                .unwrap_or_default();
            self.cache_message_images(&self.current_messages.clone());
            self.force_scroll_to_bottom();
            self.streaming_error = None;
        }

        cx.notify();
    }

    // -- UX enhancement methods --

    /// Retry after a streaming error.
    /// Replays the last user turn directly when possible to avoid duplicate user messages.
    pub(super) fn retry_after_error(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.streaming_error = None;

        if !self.is_streaming {
            if let Some(chat_id) = self.selected_chat_id {
                if should_retry_existing_user_turn(&self.current_messages) {
                    info!(
                        chat_id = %chat_id,
                        message_count = self.current_messages.len(),
                        "Retrying last failed request without adding a duplicate user message"
                    );
                    self.start_streaming_response(chat_id, cx);
                    cx.notify();
                    return;
                }
            }

            if let Some(last_user) = self
                .current_messages
                .iter()
                .rev()
                .find(|m| m.role == MessageRole::User)
            {
                let content = last_user.content.clone();
                self.input_state.update(cx, |state, cx| {
                    state.set_value(content, window, cx);
                });
                self.submit_message(window, cx);
            }
        }
        cx.notify();
    }

    /// Begin editing a specific message (sets editing_message_id + populates input).
    pub(super) fn start_editing_message(
        &mut self,
        msg_id: String,
        content: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editing_message_id = Some(msg_id);
        self.input_state.update(cx, |state, cx| {
            state.set_value(content, window, cx);
        });
        cx.notify();
    }

    /// Submit the edited message: truncate history from the edit point and re-send.
    pub(super) fn submit_edited_message(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(edit_id) = self.editing_message_id.take() {
            if let Some(idx) = self.current_messages.iter().position(|m| m.id == edit_id) {
                let to_delete: Vec<String> = self.current_messages[idx..]
                    .iter()
                    .map(|m| m.id.clone())
                    .collect();
                for mid in &to_delete {
                    let _ = storage::delete_message(mid);
                }
                self.current_messages.truncate(idx);
            }
            self.submit_message(window, cx);
        }
    }

    /// Edit the last user message (triggered by Up arrow in empty input).
    pub(super) fn edit_last_user_message(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(last_user) = self
            .current_messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
            .cloned()
        {
            self.start_editing_message(last_user.id.clone(), last_user.content.clone(), window, cx);
        }
    }

    /// Save the current input text as a draft for the current chat.
    pub(super) fn save_draft(&mut self, cx: &mut Context<Self>) {
        if let Some(chat_id) = self.selected_chat_id {
            let text = self.input_state.read(cx).value().to_string();
            if text.is_empty() {
                self.chat_drafts.remove(&chat_id);
            } else {
                self.chat_drafts.insert(chat_id, text);
            }
        }
    }

    /// Restore a previously saved draft into the input field.
    pub(super) fn restore_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(chat_id) = self.selected_chat_id {
            let draft = self.chat_drafts.get(&chat_id).cloned().unwrap_or_default();
            self.input_state.update(cx, |state, cx| {
                state.set_value(draft, window, cx);
            });
        }
    }

    /// Start renaming a chat in the sidebar (double-click).
    pub(super) fn start_rename(
        &mut self,
        chat_id: ChatId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let title = self
            .chats
            .iter()
            .find(|c| c.id == chat_id)
            .map(|c| c.title.clone())
            .unwrap_or_default();
        self.renaming_chat_id = Some(chat_id);
        self.rename_input_state.update(cx, |state, cx| {
            state.set_value(title, window, cx);
        });
        cx.notify();
    }

    /// Commit the sidebar rename (Enter key).
    pub(super) fn commit_rename(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(chat_id) = self.renaming_chat_id.take() {
            let new_title = self.rename_input_state.read(cx).value().to_string();
            if !new_title.is_empty() {
                if let Some(chat) = self.chats.iter_mut().find(|c| c.id == chat_id) {
                    chat.set_title(new_title.clone());
                }
                let _ = storage::update_chat_title(&chat_id, &new_title);
            }
        }
        cx.notify();
    }

    /// Cancel the sidebar rename (Escape key).
    pub(super) fn cancel_rename(&mut self, cx: &mut Context<Self>) {
        self.renaming_chat_id = None;
        cx.notify();
    }

    /// Update cached theme-derived values if theme revision has changed.
    ///
    /// This is called during render to detect theme hot-reloads and recompute
    /// values like box shadows that are derived from the theme.
    pub(super) fn maybe_update_theme_cache(&mut self) {
        let current_rev = crate::theme::service::theme_revision();
        if self.theme_rev_seen != current_rev {
            self.theme_rev_seen = current_rev;
            self.cached_box_shadows = Self::compute_box_shadows();
        }
    }

    /// Persist window bounds if they've changed (debounced).
    ///
    /// This ensures bounds are saved even when the window is closed via traffic light
    /// (red close button) which doesn't go through our close handlers.
    pub(super) fn maybe_persist_bounds(&mut self, window: &gpui::Window) {
        let wb = window.window_bounds();

        // Skip if bounds haven't changed
        if self.last_persisted_bounds.as_ref() == Some(&wb) {
            return;
        }

        // Debounce to avoid too-frequent saves
        if self.last_bounds_save.elapsed()
            < std::time::Duration::from_millis(Self::BOUNDS_DEBOUNCE_MS)
        {
            return;
        }

        // Save bounds
        self.last_persisted_bounds = Some(wb);
        self.last_bounds_save = std::time::Instant::now();
        crate::window_state::save_window_from_gpui(crate::window_state::WindowRole::Ai, wb);
    }
}
