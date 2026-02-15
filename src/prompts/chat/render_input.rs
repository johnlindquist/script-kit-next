use super::*;

impl ChatPrompt {
    pub(super) fn render_input(&self, is_focused: bool) -> impl IntoElement {
        let theme_colors = &self.theme.colors;
        let text = self.input.text();
        let cursor_pos = self.input.cursor();
        let chars: Vec<char> = text.chars().collect();
        let cursor_visible = self.cursor_visible && is_focused;
        let input_text_color = rgb(theme_colors.text.primary);
        let placeholder_text_color = rgb(theme_colors.text.muted);
        let cursor_color = rgb(theme_colors.accent.selected);

        let mut input_content = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .text_size(px(14.0))
            .text_color(input_text_color);

        // Text before cursor
        if !text.is_empty() && cursor_pos > 0 {
            let before_raw: String = chars[..cursor_pos].iter().collect();
            let before = Self::input_display_text(&before_raw);
            input_content = input_content.child(before);
        }

        // Cursor (blinking)
        let cursor = div()
            .w(px(2.0))
            .h(px(16.0))
            .when(cursor_visible, |d| d.bg(cursor_color));
        input_content = input_content.child(cursor);

        // Placeholder if empty - cursor appears BEFORE placeholder text
        if text.is_empty() {
            let placeholder = self
                .placeholder
                .clone()
                .unwrap_or_else(|| "Ask follow-up...".into());
            input_content =
                input_content.child(div().text_color(placeholder_text_color).child(placeholder));
        } else if cursor_pos < chars.len() {
            // Text after cursor
            let after_raw: String = chars[cursor_pos..].iter().collect();
            let after = Self::input_display_text(&after_raw);
            input_content = input_content.child(after);
        }

        let input_bg_alpha = if is_focused {
            CHAT_LAYOUT_INPUT_BG_FOCUSED_ALPHA
        } else {
            CHAT_LAYOUT_INPUT_BG_IDLE_ALPHA
        };
        let input_border = if is_focused {
            theme_colors.accent.selected
        } else {
            theme_colors.ui.border
        };
        let input_border_alpha = if is_focused {
            CHAT_LAYOUT_INPUT_BORDER_FOCUSED_ALPHA
        } else {
            CHAT_LAYOUT_INPUT_BORDER_IDLE_ALPHA
        };

        div()
            .id("chat-input-field")
            .w_full()
            .min_h(px(28.0))
            .px(px(CHAT_LAYOUT_CARD_PADDING_X))
            .py(px(CHAT_LAYOUT_SECTION_PADDING_Y))
            .flex()
            .flex_row()
            .items_center()
            .rounded(px(8.0))
            .bg(rgba(
                (theme_colors.background.search_box << 8) | input_bg_alpha,
            ))
            .border_1()
            .border_color(rgba((input_border << 8) | input_border_alpha))
            .child(input_content)
    }

    /// Render the header with back button and title
    pub(super) fn render_header(&self) -> impl IntoElement {
        let colors = &self.prompt_colors;
        let title = self.title.clone().unwrap_or_else(|| "Chat".into());

        div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            .px(px(CHAT_LAYOUT_PADDING_X))
            .py(px(CHAT_LAYOUT_SECTION_PADDING_Y))
            .border_b_1()
            .border_color(rgba((colors.quote_border << 8) | CHAT_LAYOUT_BORDER_ALPHA))
            .child(
                // Back arrow
                div()
                    .text_sm()
                    .text_color(rgb(colors.text_secondary))
                    .child("←"),
            )
            .child(
                // Title
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_primary))
                    .child(title),
            )
    }
}
