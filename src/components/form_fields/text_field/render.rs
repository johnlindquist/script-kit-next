use gpui::*;

use super::super::helpers::{char_len, slice_by_char_range};
use super::FormTextField;

impl Focusable for FormTextField {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FormTextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let is_focused = self.focus_handle.is_focused(window);
        let display_text = self.display_text();
        let placeholder = self.field.placeholder.clone().unwrap_or_default();
        let label = self.field.label.clone();
        let cursor_pos = self.cursor_position;
        let has_value = !self.value.is_empty();

        // Only log in debug builds to avoid performance issues in production
        #[cfg(debug_assertions)]
        if std::env::var("SCRIPT_KIT_FIELD_DEBUG").is_ok() {
            crate::logging::log(
                "FIELD",
                &format!(
                    "TextField[{}] render: is_focused={}, value='{}'",
                    self.field.name, is_focused, self.value
                ),
            );
        }

        // Calculate border and background based on focus
        let border_color = if is_focused {
            rgb(colors.border_focused)
        } else {
            rgb(colors.border)
        };
        let bg_color = if is_focused {
            rgba((colors.background_focused << 8) | 0xff)
        } else {
            rgba((colors.background << 8) | 0x80)
        };

        let field_name = self.field.name.clone();
        let field_name_for_log = field_name.clone();

        // Keyboard handler for text input - use unified handler that properly
        // handles char indexing, modifiers, selection, and clipboard
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                #[cfg(debug_assertions)]
                {
                    let key = event.keystroke.key.as_str();
                    crate::logging::log(
                        "FIELD",
                        &format!(
                            "TextField[{}] key: '{}' (key_char: {:?})",
                            field_name_for_log, key, event.keystroke.key_char
                        ),
                    );
                }

                // Use the unified key event handler which:
                // - Uses char indices (not byte indices) for cursor/selection
                // - Handles Cmd/Ctrl modifiers correctly (won't insert "v" on Cmd+V)
                // - Supports selection with Shift+Arrow
                // - Supports clipboard operations
                this.handle_key_event(event, cx);
            },
        );

        // Build cursor element (2px width is fixed for crisp rendering)
        let cursor_element = div().w(px(2.)).h(rems(1.125)).bg(rgb(colors.cursor));

        // Build text content based on value and focus state
        // IMPORTANT: cursor_pos is a CHAR index, not byte index.
        // For password fields with bullets ("•" = 3 bytes), we must slice by char.
        let display_len = char_len(&display_text);
        let safe_cursor = cursor_pos.min(display_len);
        let text_before = slice_by_char_range(&display_text, 0, safe_cursor);
        let text_after = slice_by_char_range(&display_text, safe_cursor, display_len);

        let text_content: Div = if has_value {
            let mut content = div()
                .flex()
                .flex_row()
                .items_center()
                // Text before cursor
                .child(
                    div()
                        .text_size(px(colors.input_font_size))
                        .text_color(rgb(colors.text))
                        .child(text_before.to_string()),
                );

            // Cursor (only when focused)
            if is_focused {
                content = content.child(cursor_element);
            }

            // Text after cursor
            content.child(
                div()
                    .text_size(px(colors.input_font_size))
                    .text_color(rgb(colors.text))
                    .child(text_after.to_string()),
            )
        } else {
            let mut content = div().flex().flex_row().items_center();

            if is_focused {
                // Cursor when focused and empty
                content = content.child(cursor_element);
            } else {
                // Placeholder when not focused
                content = content.child(
                    div()
                        .text_size(px(colors.input_font_size))
                        .text_color(rgb(colors.placeholder))
                        .child(placeholder),
                );
            }
            content
        };

        // Build the main container - horizontal layout with label beside input
        let mut container = div()
            .id(ElementId::Name(format!("form-field-{}", field_name).into()))
            .flex()
            .flex_row()
            .items_center()
            .gap(rems(0.75))
            .w_full();

        // Add label if present - fixed width for alignment
        if let Some(label_text) = label {
            container = container.child(
                div()
                    .w(rems(7.5))
                    .text_size(px(colors.label_font_size))
                    .text_color(rgb(colors.label))
                    .font_weight(FontWeight::MEDIUM)
                    .child(label_text),
            );
        }

        // Add input container - fills remaining space
        // Handle click to focus this field
        let focus_handle_for_click = self.focus_handle.clone();
        let handle_click = cx.listener(
            move |_this: &mut Self,
                  _event: &ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                crate::logging::log("FIELD", "TextField clicked - focusing");
                focus_handle_for_click.focus(window, cx);
            },
        );

        container.child(
            div()
                .id(ElementId::Name(format!("input-{}", field_name).into()))
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key)
                .on_click(handle_click)
                .flex()
                .flex_row()
                .items_center()
                .flex_1()
                .h(rems(2.25))
                .px(rems(0.75))
                .bg(bg_color)
                .border_1()
                .border_color(border_color)
                .rounded(px(6.))
                .cursor_text()
                // Text content or placeholder
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .flex_1()
                        .overflow_hidden()
                        .child(text_content),
                ),
        )
    }
}
