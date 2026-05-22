use gpui::*;
use gpui_component::scroll::ScrollableElement;

use super::super::helpers::{char_len, slice_by_char_range};
use super::super::FormFieldMetrics;
use super::FormTextArea;

impl Focusable for FormTextArea {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FormTextArea {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let metrics = FormFieldMetrics::from_colors(colors);
        let is_focused = self.focus_handle.is_focused(window);
        let display_text = self.value.clone();
        let placeholder = self.field.placeholder.clone().unwrap_or_default();
        let label = self.field.label.clone();
        let rows = self.rows;
        let has_value = !self.value.is_empty();
        let cursor_pos = self.cursor_position;

        // Calculate border and background based on focus using shared whisper surface
        let surface = colors.whisper_surface(is_focused);
        let border_color = surface.border;
        let bg_color = surface.background;

        let height_rems = metrics.text_area_height_rems(rows);

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

        let cursor_element = div()
            .w(px(metrics.cursor_width_px))
            .h(rems(metrics.cursor_height_rems))
            .bg(colors.cursor);

        // Build text content with the same visible cursor affordance as text fields.
        let text_content: Div = if has_value {
            let display_len = char_len(&display_text);
            let safe_cursor = cursor_pos.min(display_len);
            let text_before = slice_by_char_range(&display_text, 0, safe_cursor);
            let text_after = slice_by_char_range(&display_text, safe_cursor, display_len);
            let mut content = div().flex().flex_row().items_start().child(
                div()
                    .text_size(px(colors.input_font_size))
                    .text_color(colors.text)
                    .child(text_before.to_string()),
            );
            if is_focused {
                content = content.child(cursor_element);
            }
            content.child(
                div()
                    .text_size(px(colors.input_font_size))
                    .text_color(colors.text)
                    .child(text_after.to_string()),
            )
        } else {
            let mut content = div().flex().flex_row().items_center();
            if is_focused {
                content = content.child(cursor_element);
            }
            content.child(
                div()
                    .text_size(px(colors.input_font_size))
                    .text_color(colors.placeholder)
                    .child(placeholder),
            )
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
            .gap(px(metrics.field_gap_px))
            .w_full();

        // Add label above textarea if present
        if let Some(label_text) = label {
            container = container.child(
                div()
                    .text_size(px(colors.label_font_size))
                    .text_color(colors.label)
                    .font_weight(FontWeight::MEDIUM)
                    .child(label_text),
            );
        }

        container.child(input_surface)
    }
}
