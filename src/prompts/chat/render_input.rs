use super::*;

impl ChatPrompt {
    pub(super) fn render_input(&self, is_focused: bool) -> impl IntoElement {
        let theme_colors = &self.theme.colors;
        let text = self.input.text();
        let cursor_visible = self.cursor_visible && is_focused;
        let input_text_color = rgb(theme_colors.text.primary);
        let placeholder_text_color = rgb(theme_colors.text.muted);
        let mut input_content = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .text_size(px(14.0))
            .text_color(input_text_color);
        input_content = input_content.child(
            crate::components::text_input::render_text_input_cursor_selection(
                crate::components::text_input::TextInputRenderConfig {
                    cursor: self.input.cursor(),
                    selection: Some(self.input.selection()),
                    cursor_visible,
                    cursor_color: theme_colors.accent.selected,
                    text_color: theme_colors.text.primary,
                    selection_color: theme_colors.accent.selected,
                    selection_text_color: theme_colors.text.primary,
                    transform: Some(Self::input_display_text),
                    ..crate::components::text_input::TextInputRenderConfig::default_for_prompt(text)
                },
            ),
        );

        if text.is_empty() {
            let placeholder = self
                .placeholder
                .clone()
                .unwrap_or_else(|| "Ask follow-up...".into());
            input_content =
                input_content.child(div().text_color(placeholder_text_color).child(placeholder));
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
