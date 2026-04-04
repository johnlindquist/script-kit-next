use gpui::*;
use gpui_component::scroll::ScrollableElement;

use super::FormTextArea;

impl Focusable for FormTextArea {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FormTextArea {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let is_focused = self.focus_handle.is_focused(window);
        let display_text = self.value.clone();
        let placeholder = self.field.placeholder.clone().unwrap_or_default();
        let label = self.field.label.clone();
        let rows = self.rows;
        let has_value = !self.value.is_empty();

        // Calculate border and background based on focus using shared whisper surface
        let surface = colors.whisper_surface(is_focused);
        let border_color = surface.border;
        let bg_color = surface.background;

        // Calculate height based on rows (1.5rem per row + 1rem padding)
        let height_rems = (rows as f32) * 1.5 + 1.0;

        let field_name = self.field.name.clone();
        let field_name_for_log = field_name.clone();

        // Handle click to focus this field
        let focus_handle_for_click = self.focus_handle.clone();
        let handle_click = cx.listener(
            move |_this: &mut Self,
                  _event: &ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                #[cfg(debug_assertions)]
                crate::logging::log(
                    "FIELD",
                    &format!("TextArea[{}] clicked - focusing", field_name_for_log),
                );
                focus_handle_for_click.focus(window, cx);
            },
        );

        // Keyboard handler for text input - use unified handler that properly
        // handles char indexing, modifiers, selection, and clipboard
        let handle_key = cx.listener(
            |this: &mut Self,
             event: &KeyDownEvent,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                // Use the unified key event handler which:
                // - Uses char indices (not byte indices) for cursor/selection
                // - Handles Cmd/Ctrl modifiers correctly (won't insert "v" on Cmd+V)
                // - Supports selection with Shift+Arrow
                // - Supports clipboard operations
                // - Handles Enter to insert newlines
                this.handle_key_event(event, cx);
            },
        );

        // Build text content
        let text_content: Div = if has_value {
            div()
                .flex()
                .flex_col()
                .text_size(px(colors.input_font_size))
                .text_color(rgb(colors.text))
                .child(display_text)
        } else {
            div()
                .text_size(px(colors.input_font_size))
                .text_color(rgb(colors.placeholder))
                .child(placeholder)
        };

        // Input surface - uses shared prompt_surface for consistent card chrome
        let input_surface = crate::components::prompt_surface(bg_color, border_color)
            .id(ElementId::Name(format!("textarea-{}", field_name).into()))
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .on_click(handle_click)
            .flex()
            .flex_col()
            .h(rems(height_rems))
            .cursor_text()
            .overflow_x_hidden()
            .overflow_y_scrollbar()
            .child(text_content);

        // Build the main container - stacked vertical layout with label above textarea
        let mut container = div()
            .id(ElementId::Name(
                format!("form-textarea-{}", field_name).into(),
            ))
            .flex()
            .flex_col()
            .gap(px(6.))
            .w_full();

        // Add label above textarea if present
        if let Some(label_text) = label {
            container = container.child(
                div()
                    .text_size(px(colors.label_font_size))
                    .text_color(rgb(colors.label))
                    .font_weight(FontWeight::MEDIUM)
                    .child(label_text),
            );
        }

        container.child(input_surface)
    }
}
