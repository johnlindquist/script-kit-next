use super::*;

impl ChatPrompt {
    pub(super) fn render_turn(
        &self,
        turn: &ConversationTurn,
        turn_index: usize,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let colors = &self.prompt_colors;

        // VIBRANCY: Use theme-aware overlay for subtle lift that lets blur show through
        // Dark mode: white overlay brightens; Light mode: much subtler black overlay
        let container_bg = if self.theme.is_dark_mode() {
            theme::hover_overlay_bg(&self.theme, 0x15) // ~8% white overlay for dark mode
        } else {
            theme::hover_overlay_bg(&self.theme, 0x08) // ~3% black overlay for light mode
        };
        let copy_hover_bg = theme::hover_overlay_bg(&self.theme, 0x28); // ~16% for hover
        let error_color = self.theme.colors.ui.error;
        let error_bg = rgba((error_color << 8) | 0x40); // Theme error with transparency
        let retry_hover_bg = rgba((colors.accent_color << 8) | 0x40);
        let has_retry_callback = self.on_retry.is_some();

        let mut content = div().flex().flex_col().gap(px(4.0)).w_full().min_w_0();
        // Note: removed overflow_hidden() to allow text to wrap naturally

        // User prompt (small, bold) - only if not empty
        if !turn.user_prompt.is_empty() {
            content = content.child(
                div()
                    .w_full()
                    .min_w_0()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(colors.text_secondary))
                    .child(turn.user_prompt.clone()),
            );
        }

        // User image thumbnail (if attached)
        if let Some(ref user_image) = turn.user_image {
            let render_img = user_image.clone();
            content = content.child(
                img(move |_window: &mut Window, _cx: &mut App| Some(Ok(render_img.clone())))
                    .w(px(64.))
                    .h(px(64.))
                    .rounded_sm(),
            );
        }

        // Error state - show error message with optional retry button
        if let Some(ref error_str) = turn.error {
            let error_type = ChatErrorType::from_error_string(error_str);
            let error_message = error_type.display_message();
            let can_retry = error_type.can_retry() && has_retry_callback;

            let mut error_row = div().flex().flex_row().items_center().gap(px(8.0)).child(
                div()
                    .text_sm()
                    .text_color(rgb(error_color))
                    .child(error_message.to_string()),
            );

            // Add retry button if applicable
            if can_retry {
                let message_id = turn.message_id.clone();
                error_row = error_row.child(
                    div()
                        .id(format!("retry-turn-{}", turn_index))
                        .px(px(8.0))
                        .py(px(4.0))
                        .bg(error_bg)
                        .rounded(px(4.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(retry_hover_bg))
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(colors.text_primary))
                        .child("Retry")
                        .on_click(cx.listener(move |this, _, _window, _cx| {
                            if let Some(msg_id) = &message_id {
                                this.handle_retry(msg_id.clone());
                            }
                        })),
                );
            }

            content = content.child(error_row);

            // Show raw error detail so the actual cause is visible
            let detail = error_str.trim();
            if !detail.is_empty() && detail != error_message {
                // Truncate very long error strings for display
                let truncated = if detail.len() > 200 {
                    format!("{}…", &detail[..200])
                } else {
                    detail.to_string()
                };
                content = content.child(
                    div()
                        .text_xs()
                        .opacity(0.5)
                        .text_color(rgb(error_color))
                        .child(truncated),
                );
            }
        }
        // AI response (only show if no error, or show partial if stream interrupted)
        else if let Some(ref response) = turn.assistant_response {
            let markdown_response = super::types::assistant_response_markdown_source(
                self.script_generation_mode,
                response,
            );

            // Use markdown rendering for assistant responses
            if turn.streaming && response.is_empty() {
                // Empty streaming state
                content = content.child(div().text_xs().opacity(0.6).child("Thinking..."));
            } else if turn.streaming {
                // Streaming with content - render markdown separately from cursor
                // to avoid invalidating the markdown cache on every frame
                content = content.child(
                    div()
                        .w_full()
                        .min_w_0()
                        .overflow_x_hidden()
                        .child(render_markdown(markdown_response.as_ref(), colors))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(colors.accent_color))
                                .child("▌"),
                        ),
                );
            } else {
                // Complete response - full markdown rendering (with container for proper wrapping)
                content = content.child(
                    div()
                        .w_full()
                        .min_w_0()
                        .overflow_x_hidden()
                        .child(render_markdown(markdown_response.as_ref(), colors)),
                );
            }
        }

        // Copy button (appears on right side) - copies assistant response
        let copy_button = div()
            .id(format!("copy-turn-{}", turn_index))
            .flex()
            .items_center()
            .justify_center()
            .w(px(24.0))
            .h(px(24.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .opacity(0.7)
            .hover(|s| s.opacity(1.0).bg(copy_hover_bg))
            .child(
                svg()
                    .external_path(IconName::Copy.external_path())
                    .size(px(16.))
                    .text_color(rgb(colors.text_secondary)),
            )
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.copy_turn_response(turn_index, cx);
            }));

        // The full-width container with copy button
        div()
            .w_full()
            .px(px(12.0))
            .py(px(10.0))
            .bg(container_bg)
            .rounded(px(8.0))
            .flex()
            .flex_row()
            .gap(px(8.0))
            .child(content.flex_1().min_w_0())
            .child(copy_button)
    }

    /// Handle retry for a failed message
    pub(super) fn handle_retry(&self, message_id: String) {
        logging::log(
            "CHAT",
            &format!("Retry requested for message: {}", message_id),
        );
        if let Some(ref callback) = self.on_retry {
            callback(self.id.clone(), message_id);
        }
    }

    /// Copy the assistant response from a specific turn
    pub(super) fn copy_turn_response(&mut self, turn_index: usize, cx: &mut Context<Self>) {
        self.ensure_conversation_turns_cache();
        if let Some(turn) = self.conversation_turns_cache.get(turn_index) {
            if let Some(ref response) = turn.assistant_response {
                let content = response.clone();
                logging::log(
                    "CHAT",
                    &format!(
                        "Copied turn {} response: {} chars",
                        turn_index,
                        content.len()
                    ),
                );
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(content));
            } else if !turn.user_prompt.is_empty() {
                // If no assistant response, copy the user prompt
                let content = turn.user_prompt.clone();
                logging::log(
                    "CHAT",
                    &format!(
                        "Copied turn {} user prompt: {} chars",
                        turn_index,
                        content.len()
                    ),
                );
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(content));
            }
        }
    }
}
