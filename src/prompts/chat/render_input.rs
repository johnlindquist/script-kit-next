use super::*;

impl ChatPrompt {
    pub(super) fn render_input(&self, is_focused: bool) -> impl IntoElement {
        let text = self.input.text();
        let cursor_visible = self.cursor_visible && is_focused;
        let input_text_color = rgb(self.theme.colors.text.primary);
        let placeholder_text_color = rgb(self.theme.colors.text.muted);
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
                    cursor_color: self.theme.colors.accent.selected,
                    text_color: self.theme.colors.text.primary,
                    selection_color: self.theme.colors.accent.selected,
                    selection_text_color: self.theme.colors.text.primary,
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

        crate::components::InlinePromptInput::new(
            div()
                .id("chat-input-field")
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .child(input_content),
        )
    }
}
