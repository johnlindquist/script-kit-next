use super::*;

impl ChatPrompt {
    pub(super) fn render_input(&self, is_focused: bool) -> impl IntoElement {
        let colors = &self.prompt_colors;
        let theme_colors = &self.theme.colors;
        let text = self.input.text();
        let cursor_pos = self.input.cursor();
        let chars: Vec<char> = text.chars().collect();
        let cursor_visible = self.cursor_visible && is_focused;

        let mut input_content = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .text_size(px(14.0));

        // Text before cursor
        if cursor_pos > 0 {
            let before_raw: String = chars[..cursor_pos].iter().collect();
            let before = Self::input_display_text(&before_raw);
            input_content =
                input_content.child(div().text_color(rgb(colors.text_primary)).child(before));
        }

        // Cursor (blinking)
        let cursor = div()
            .w(px(2.0))
            .h(px(16.0))
            .when(cursor_visible, |d| d.bg(rgb(colors.accent_color)));
        input_content = input_content.child(cursor);

        // Text after cursor
        if cursor_pos < chars.len() {
            let after_raw: String = chars[cursor_pos..].iter().collect();
            let after = Self::input_display_text(&after_raw);
            input_content =
                input_content.child(div().text_color(rgb(colors.text_primary)).child(after));
        }

        // Placeholder if empty - cursor appears BEFORE placeholder text
        if text.is_empty() {
            let placeholder = self
                .placeholder
                .clone()
                .unwrap_or_else(|| "Ask follow-up...".into());
            let cursor = div()
                .w(px(2.0))
                .h(px(16.0))
                .when(cursor_visible, |d| d.bg(rgb(colors.accent_color)));
            input_content = div().flex().flex_row().items_center().child(cursor).child(
                div()
                    .text_color(rgb(colors.text_tertiary))
                    .child(placeholder),
            );
        }

        let input_bg_alpha = if is_focused { 0xD8 } else { 0xA8 };
        let input_border = if is_focused {
            theme_colors.accent.selected
        } else {
            theme_colors.ui.border
        };
        let input_border_alpha = if is_focused { 0xA0 } else { 0x60 };

        div()
            .id("chat-input-field")
            .w_full()
            .min_h(px(28.0))
            .px(px(10.0))
            .py(px(7.0))
            .flex()
            .items_center()
            .rounded(px(8.0))
            .bg(rgba(
                (theme_colors.background.search_box << 8) | input_bg_alpha as u32,
            ))
            .border_1()
            .border_color(rgba((input_border << 8) | input_border_alpha as u32))
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
            .px(px(12.0))
            .py(px(8.0))
            .border_b_1()
            .border_color(rgba((colors.quote_border << 8) | 0x40))
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
