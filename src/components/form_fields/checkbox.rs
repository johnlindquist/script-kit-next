use gpui::*;

use crate::protocol::Field;

use super::{FormFieldColors, FormFieldState};

pub struct FormCheckbox {
    /// Field definition from protocol
    field: Field,
    /// Pre-computed colors
    colors: FormFieldColors,
    /// Whether the checkbox is checked
    checked: bool,
    /// Focus handle for keyboard navigation
    focus_handle: FocusHandle,
    /// Shared state for external access (stores "true" or "false")
    pub state: FormFieldState,
}

impl FormCheckbox {
    /// Create a new checkbox from a Field definition
    pub fn new(field: Field, colors: FormFieldColors, cx: &mut App) -> Self {
        // Parse initial checked state from value
        let checked = field.value.as_deref() == Some("true");
        let state = FormFieldState::new(if checked {
            "true".to_string()
        } else {
            "false".to_string()
        });

        Self {
            field,
            colors,
            checked,
            focus_handle: cx.focus_handle(),
            state,
        }
    }

    /// Get whether the checkbox is checked
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Get the field name
    pub fn name(&self) -> &str {
        &self.field.name
    }

    /// Toggle the checkbox state
    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.checked = !self.checked;
        self.state.set_value(if self.checked {
            "true".to_string()
        } else {
            "false".to_string()
        });
        cx.notify();
    }

    /// Set the checked state
    pub fn set_checked(&mut self, checked: bool, cx: &mut Context<Self>) {
        self.checked = checked;
        self.state.set_value(if checked {
            "true".to_string()
        } else {
            "false".to_string()
        });
        cx.notify();
    }
}

impl Focusable for FormCheckbox {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FormCheckbox {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let is_focused = self.focus_handle.is_focused(window);
        let checked = self.checked;
        let label = self
            .field
            .label
            .clone()
            .unwrap_or_else(|| self.field.name.clone());

        // Calculate border based on focus
        let border_color = if is_focused {
            rgb(colors.border_focused)
        } else {
            rgb(colors.border)
        };

        // Checkbox box styling
        let box_bg = if checked {
            rgb(colors.checkbox_checked)
        } else {
            rgba((colors.background << 8) | 0x80)
        };

        let field_name = self.field.name.clone();

        // Keyboard handler for Space key to toggle
        let handle_key = cx.listener(
            |this: &mut Self,
             event: &KeyDownEvent,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                let key = event.keystroke.key.as_str();
                if key == "space" || key == " " {
                    this.toggle(cx);
                }
            },
        );

        // Build checkbox box with optional checkmark
        let mut checkbox_box = div()
            .flex()
            .items_center()
            .justify_center()
            .w(rems(1.125))
            .h(rems(1.125))
            .bg(box_bg)
            .border_1()
            .border_color(border_color)
            .rounded(px(4.));

        // Add checkmark when checked
        if checked {
            checkbox_box = checkbox_box.child(
                div()
                    .text_size(px(colors.label_font_size))
                    .text_color(rgb(colors.checkbox_mark))
                    .font_weight(FontWeight::BOLD)
                    .child("✓"),
            );
        }

        // Main container - horizontal layout consistent with other form fields
        div()
            .id(ElementId::Name(
                format!("form-checkbox-{}", field_name).into(),
            ))
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .flex()
            .flex_row()
            .items_center()
            .gap(rems(0.75))
            .w_full()
            .cursor_pointer()
            .on_click(cx.listener(|this, _event: &ClickEvent, _window, cx| {
                this.toggle(cx);
            }))
            // Empty label area for alignment with other fields
            .child(div().w(rems(7.5)))
            // Checkbox and label group
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(rems(0.5))
                    // Checkbox box
                    .child(checkbox_box)
                    // Label
                    .child(
                        div()
                            .text_size(px(colors.label_font_size))
                            .text_color(rgb(colors.text))
                            .child(label),
                    ),
            )
    }
}

// Note: Full GPUI component tests require the test harness which has macro recursion
// limit issues. The form field components are integration-tested via the main
// application's form prompt rendering. Unit tests for helper functions are in
// src/components/form_fields_tests.rs.
