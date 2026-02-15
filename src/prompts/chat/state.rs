use super::*;

impl ChatPrompt {
    pub(super) fn mark_conversation_turns_dirty(&mut self) {
        self.conversation_turns_dirty = true;
    }

    pub(super) fn sync_turns_list_state(&mut self) {
        let item_count = self.conversation_turns_cache.len();
        let old_count = self.turns_list_state.item_count();
        if old_count != item_count {
            self.turns_list_state.splice(0..old_count, item_count);
        }
    }

    pub(super) fn ensure_conversation_turns_cache(&mut self) {
        if !self.conversation_turns_dirty {
            return;
        }
        self.conversation_turns_cache = Arc::new(build_conversation_turns(
            &self.messages,
            &self.image_render_cache,
        ));
        self.conversation_turns_dirty = false;
        self.sync_turns_list_state();
    }

    pub(super) fn turns_list_is_at_bottom(&self) -> bool {
        let item_count = self.conversation_turns_cache.len();
        if item_count == 0 {
            return true;
        }

        // For bottom-aligned lists, GPUI reports `item_ix == item_count` when the
        // viewport is at the real bottom (logical_scroll_top == None internally).
        let scroll_top = self.turns_list_state.logical_scroll_top();
        scroll_top.item_ix >= item_count
    }

    pub(super) fn scroll_turns_to_bottom(&mut self) {
        self.ensure_conversation_turns_cache();
        let item_count = self.conversation_turns_cache.len();
        if item_count == 0 {
            return;
        }

        // If manual mode is active but the user has returned to the bottom,
        // resume auto-follow for subsequent stream chunks.
        if self.user_has_scrolled_up && self.turns_list_is_at_bottom() {
            logging::log(
                "CHAT",
                "Resuming chat auto-follow after returning to bottom",
            );
            self.user_has_scrolled_up = false;
        }

        if !self.user_has_scrolled_up {
            self.turns_list_state.scroll_to_reveal_item(item_count - 1);
        }
    }

    pub(super) fn force_scroll_turns_to_bottom(&mut self) {
        self.user_has_scrolled_up = false;
        self.ensure_conversation_turns_cache();
        let item_count = self.conversation_turns_cache.len();
        if item_count > 0 {
            self.turns_list_state.scroll_to_reveal_item(item_count - 1);
        }
    }

    pub fn add_message(&mut self, message: ChatPromptMessage, cx: &mut Context<Self>) {
        logging::log(
            "CHAT",
            &format!("Adding message: {:?}", message.get_position()),
        );
        self.messages.push(message);
        self.mark_conversation_turns_dirty();
        self.force_scroll_turns_to_bottom();
        cx.notify();
    }

    pub fn start_streaming(
        &mut self,
        message_id: String,
        position: ChatMessagePosition,
        cx: &mut Context<Self>,
    ) {
        let role = match position {
            ChatMessagePosition::Right => Some(ChatMessageRole::User),
            ChatMessagePosition::Left => Some(ChatMessageRole::Assistant),
        };

        let message = ChatPromptMessage {
            id: Some(message_id.clone()),
            role,
            content: Some(String::new()),
            text: String::new(),
            position,
            name: None,
            model: self.model.clone(),
            streaming: true,
            error: None,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            image: None,
        };
        self.messages.push(message);
        self.streaming_message_id = Some(message_id);
        self.mark_conversation_turns_dirty();
        self.force_scroll_turns_to_bottom();
        cx.notify();
    }

    pub fn append_chunk(&mut self, message_id: &str, chunk: &str, cx: &mut Context<Self>) {
        if self.streaming_message_id.as_deref() == Some(message_id) {
            if let Some(msg) = self
                .messages
                .iter_mut()
                .rev()
                .find(|m| m.id.as_deref() == Some(message_id))
            {
                msg.append_content(chunk);
                self.mark_conversation_turns_dirty();
                self.scroll_turns_to_bottom();
                cx.notify();
            }
        }
    }

    pub fn complete_streaming(&mut self, message_id: &str, cx: &mut Context<Self>) {
        if let Some(msg) = self
            .messages
            .iter_mut()
            .rev()
            .find(|m| m.id.as_deref() == Some(message_id))
        {
            msg.streaming = false;
        }
        if self.streaming_message_id.as_deref() == Some(message_id) {
            self.streaming_message_id = None;
        }
        self.mark_conversation_turns_dirty();
        self.ensure_conversation_turns_cache();
        cx.notify();
    }

    pub fn clear_messages(&mut self, cx: &mut Context<Self>) {
        self.messages.clear();
        self.streaming_message_id = None;
        self.user_has_scrolled_up = false;
        self.mark_conversation_turns_dirty();
        self.ensure_conversation_turns_cache();
        cx.notify();
    }

    /// Check if currently streaming a response
    pub fn is_streaming(&self) -> bool {
        self.builtin_is_streaming || self.streaming_message_id.is_some()
    }

    /// Stop streaming the current response (preserves partial content)
    /// Triggered by Cmd+. or Escape
    pub fn stop_streaming(&mut self, cx: &mut Context<Self>) {
        logging::log("CHAT", "Stop streaming requested (Cmd+. or Escape)");

        // Flush all accumulated content so the user sees everything received so far
        if let Some(msg_id) = self.streaming_message_id.take() {
            if let Some(msg) = self
                .messages
                .iter_mut()
                .find(|m| m.id.as_deref() == Some(&msg_id))
            {
                if !self.builtin_accumulated_content.is_empty() {
                    msg.set_content(&self.builtin_accumulated_content);
                }
                msg.streaming = false;
            }
        }

        self.builtin_is_streaming = false;
        self.builtin_streaming_content.clear();
        self.builtin_accumulated_content.clear();
        self.builtin_reveal_offset = 0;
        self.mark_conversation_turns_dirty();
        self.ensure_conversation_turns_cache();

        cx.notify();
    }

    /// Get context-aware conversation starters
    /// Shows different suggestions based on clipboard content
    pub(super) fn get_conversation_starters(&self, cx: &Context<Self>) -> Vec<ConversationStarter> {
        let mut starters = default_conversation_starters();

        // Check if clipboard has content - add "Summarize clipboard" if so
        if let Some(clipboard) = cx.read_from_clipboard() {
            if let Some(text) = clipboard.text() {
                if !text.is_empty() && text.len() < 50000 {
                    // Insert clipboard-aware suggestion at position 1
                    starters.insert(
                        1,
                        ConversationStarter::new(
                            "clipboard",
                            "Summarize clipboard",
                            format!("Summarize the following:\n\n{}", text),
                        ),
                    );
                }
            }
        }

        // Limit to 5 suggestions max
        starters.truncate(5);
        starters
    }

    /// Handle clicking a conversation starter
    pub(super) fn select_conversation_starter(
        &mut self,
        starter: &ConversationStarter,
        cx: &mut Context<Self>,
    ) {
        logging::log("CHAT", &format!("Selected starter: {}", starter.id));

        // Insert the prompt into the input
        self.input.clear();
        for ch in starter.prompt.chars() {
            self.input.insert_char(ch);
        }
        self.reset_cursor_blink();
        cx.notify();
    }

    /// Render conversation starters for empty state
    pub(super) fn render_conversation_starters(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = &self.prompt_colors;
        let starters = self.get_conversation_starters(cx);

        // Chip styling - use theme-aware overlays
        let chip_bg = theme::hover_overlay_bg(&self.theme, 0x20);
        let chip_hover_bg = theme::hover_overlay_bg(&self.theme, 0x35);

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .gap(px(16.))
            .child(
                div()
                    .text_color(rgb(colors.text_secondary))
                    .text_sm()
                    .child("What can I help you with?"),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_center()
                    .gap(px(8.))
                    .max_w(px(400.))
                    .children(starters.into_iter().enumerate().map(|(i, starter)| {
                        let starter_clone = starter.clone();
                        div()
                            .id(format!("starter-{}", i))
                            .px(px(12.))
                            .py(px(8.))
                            .bg(chip_bg)
                            .rounded(px(6.))
                            .cursor_pointer()
                            .hover(|s| s.bg(chip_hover_bg))
                            .text_sm()
                            .text_color(rgb(colors.text_primary))
                            .child(starter.label.clone())
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.select_conversation_starter(&starter_clone, cx);
                            }))
                    })),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_tertiary))
                    .mt(px(8.))
                    .child("or type your own question..."),
            )
    }

    /// Set an error on a message (typically on streaming failure)
    pub fn set_message_error(&mut self, message_id: &str, error: String, cx: &mut Context<Self>) {
        if let Some(msg) = self
            .messages
            .iter_mut()
            .rev()
            .find(|m| m.id.as_deref() == Some(message_id))
        {
            msg.error = Some(error);
            msg.streaming = false; // Stop streaming indicator
        }
        if self.streaming_message_id.as_deref() == Some(message_id) {
            self.streaming_message_id = None;
        }
        self.mark_conversation_turns_dirty();
        self.ensure_conversation_turns_cache();
        cx.notify();
    }

    /// Clear error from a message (before retry)
    pub fn clear_message_error(&mut self, message_id: &str, cx: &mut Context<Self>) {
        if let Some(msg) = self
            .messages
            .iter_mut()
            .rev()
            .find(|m| m.id.as_deref() == Some(message_id))
        {
            msg.error = None;
        }
        self.mark_conversation_turns_dirty();
        self.ensure_conversation_turns_cache();
        cx.notify();
    }

    /// Handle paste event - check for clipboard images.
    /// Returns true if an image was pasted (caller should not process text).
    pub(super) fn handle_paste_for_image(&mut self, cx: &mut Context<Self>) -> bool {
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if let Ok(image_data) = clipboard.get_image() {
                    match crate::clipboard_history::encode_image_as_png(&image_data) {
                        Ok(encoded) => {
                            let base64_data =
                                encoded.strip_prefix("png:").unwrap_or(&encoded).to_string();

                            // Decode to RenderImage for preview
                            use base64::Engine;
                            if let Ok(png_bytes) =
                                base64::engine::general_purpose::STANDARD.decode(&base64_data)
                            {
                                if let Ok(render_img) =
                                    crate::list_item::decode_png_to_render_image_with_bgra_conversion(
                                        &png_bytes,
                                    )
                                {
                                    self.pending_image_render = Some(render_img);
                                }
                            }

                            self.pending_image = Some(base64_data);
                            cx.notify();
                            return true;
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Failed to encode pasted image");
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to access clipboard");
            }
        }
        false
    }

    /// Handle file drop - if it's an image, set it as pending image
    pub(super) fn handle_file_drop(&mut self, paths: &ExternalPaths, cx: &mut Context<Self>) {
        let paths = paths.paths();
        if paths.is_empty() {
            return;
        }

        let path = &paths[0];
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        let is_image = matches!(
            extension.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp"
        );

        if !is_image {
            return;
        }

        match std::fs::read(path) {
            Ok(data) => {
                use base64::Engine;
                let base64_data = base64::engine::general_purpose::STANDARD.encode(&data);

                // Decode raw bytes to RenderImage for preview
                if let Ok(render_img) =
                    crate::list_item::decode_png_to_render_image_with_bgra_conversion(&data)
                {
                    self.pending_image_render = Some(render_img);
                }

                self.pending_image = Some(base64_data);
                cx.notify();
            }
            Err(e) => {
                logging::log("CHAT", &format!("Failed to read dropped image: {}", e));
            }
        }
    }

    /// Remove the pending image attachment
    pub(super) fn remove_pending_image(&mut self, cx: &mut Context<Self>) {
        self.pending_image = None;
        self.pending_image_render = None;
        cx.notify();
    }

    /// Render the pending image preview badge
    pub(super) fn render_pending_image_preview(&self, cx: &Context<Self>) -> impl IntoElement {
        let render_img = self.pending_image_render.clone();
        let colors = &self.theme.colors;
        let preview_bg = if self.theme.is_dark_mode() {
            theme::hover_overlay_bg(&self.theme, 0x24)
        } else {
            theme::hover_overlay_bg(&self.theme, 0x12)
        };
        let preview_border = rgba((colors.accent.selected << 8) | 0x55);
        let remove_hover_bg = rgba((colors.ui.error << 8) | 0x2d);

        div()
            .id("pending-image-preview")
            .flex()
            .items_center()
            .gap(px(8.0))
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(8.0))
            .bg(preview_bg)
            .border_1()
            .border_color(preview_border)
            // Thumbnail
            .when_some(render_img, |d, render_img| {
                d.child(
                    img(move |_window: &mut Window, _cx: &mut App| Some(Ok(render_img.clone())))
                        .w(px(48.))
                        .h(px(48.))
                        .rounded_sm(),
                )
            })
            // Label
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text.primary))
                    .child("Image attached"),
            )
            // Remove button
            .child(
                div()
                    .id("remove-image-btn")
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(16.))
                    .rounded_full()
                    .cursor_pointer()
                    .hover(move |s| s.bg(remove_hover_bg))
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.remove_pending_image(cx);
                        }),
                    )
                    .child(
                        svg()
                            .external_path(IconName::Close.external_path())
                            .size(px(10.))
                            .text_color(rgb(colors.text.muted)),
                    ),
            )
    }
}
