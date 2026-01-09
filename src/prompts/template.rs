//! TemplatePrompt - String template with {{placeholder}} syntax
//!
//! Features:
//! - Parse template strings with {{name}} placeholders
//! - Tab through placeholders to fill them in
//! - Live preview of filled template
//! - Submit returns the filled template string

use gpui::{
    div, prelude::*, px, rgb, rgba, Context, FocusHandle, Focusable, Render, SharedString, Window,
};
use regex::Regex;
use std::collections::HashSet;
use std::sync::Arc;

use crate::designs::{get_tokens, DesignVariant};
use crate::logging;
use crate::theme;
use crate::ui_foundation::get_vibrancy_background;

use super::SubmitCallback;

/// Input definition for a template placeholder
#[derive(Clone, Debug)]
pub struct TemplateInput {
    /// Name of the placeholder (e.g., "name", "email")
    pub name: String,
    /// Placeholder text to show when empty
    pub placeholder: String,
}

/// TemplatePrompt - Tab-through template editor
///
/// Allows editing template strings with {{placeholder}} syntax.
/// Tab moves between placeholders, Enter submits the filled template.
pub struct TemplatePrompt {
    /// Unique ID for this prompt instance
    pub id: String,
    /// Original template string with placeholders
    pub template: String,
    /// Parsed input placeholders (unique, in order of appearance)
    pub inputs: Vec<TemplateInput>,
    /// Current values for each input
    pub values: Vec<String>,
    /// Currently focused input index
    pub current_input: usize,
    /// Focus handle for keyboard input
    pub focus_handle: FocusHandle,
    /// Callback when user submits
    pub on_submit: SubmitCallback,
    /// Theme for styling
    pub theme: Arc<theme::Theme>,
    /// Design variant for styling
    pub design_variant: DesignVariant,
}

impl TemplatePrompt {
    pub fn new(
        id: String,
        template: String,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        logging::log(
            "PROMPTS",
            &format!("TemplatePrompt::new template: {}", template),
        );

        // Parse inputs from template
        let inputs = Self::parse_template_inputs(&template);
        let values: Vec<String> = inputs.iter().map(|_| String::new()).collect();

        TemplatePrompt {
            id,
            template,
            inputs,
            values,
            current_input: 0,
            focus_handle,
            on_submit,
            theme,
            design_variant: DesignVariant::Default,
        }
    }

    /// Parse template string to extract {{name}} placeholders
    /// Returns unique placeholders in order of first appearance
    fn parse_template_inputs(template: &str) -> Vec<TemplateInput> {
        let mut inputs = Vec::new();
        let mut seen = HashSet::new();

        // Match {{name}} pattern - name can be alphanumeric with underscores
        let re = Regex::new(r"\{\{(\w+)\}\}").expect("Invalid regex");

        for cap in re.captures_iter(template) {
            if let Some(name_match) = cap.get(1) {
                let name = name_match.as_str().to_string();
                if !seen.contains(&name) {
                    seen.insert(name.clone());
                    inputs.push(TemplateInput {
                        placeholder: format!("Enter {}", name),
                        name,
                    });
                }
            }
        }

        inputs
    }

    /// Get the filled template string by replacing all placeholders
    pub fn filled_template(&self) -> String {
        let mut result = self.template.clone();

        for (input, value) in self.inputs.iter().zip(self.values.iter()) {
            let placeholder = format!("{{{{{}}}}}", input.name);
            let replacement = if value.is_empty() {
                // Show placeholder name if empty
                format!("{{{{{}}}}}", input.name)
            } else {
                value.clone()
            };
            result = result.replace(&placeholder, &replacement);
        }

        result
    }

    /// Get the preview string - shows filled values or placeholder hints
    fn preview_template(&self) -> String {
        let mut result = self.template.clone();

        for (input, value) in self.inputs.iter().zip(self.values.iter()) {
            let placeholder = format!("{{{{{}}}}}", input.name);
            let replacement = if value.is_empty() {
                format!("[{}]", input.name) // Show as [name] when empty
            } else {
                value.clone()
            };
            result = result.replace(&placeholder, &replacement);
        }

        result
    }

    /// Set the current input value programmatically
    pub fn set_input(&mut self, text: String, cx: &mut Context<Self>) {
        if let Some(value) = self.values.get_mut(self.current_input) {
            if *value == text {
                return;
            }
            *value = text;
            cx.notify();
        }
    }

    /// Submit the filled template
    fn submit(&mut self) {
        // Replace placeholders with actual values for final submission
        let mut result = self.template.clone();
        for (input, value) in self.inputs.iter().zip(self.values.iter()) {
            let placeholder = format!("{{{{{}}}}}", input.name);
            result = result.replace(&placeholder, value);
        }
        (self.on_submit)(self.id.clone(), Some(result));
    }

    /// Cancel - submit None
    fn submit_cancel(&mut self) {
        (self.on_submit)(self.id.clone(), None);
    }

    /// Move to next input (Tab)
    fn next_input(&mut self, cx: &mut Context<Self>) {
        if !self.inputs.is_empty() {
            self.current_input = (self.current_input + 1) % self.inputs.len();
            cx.notify();
        }
    }

    /// Move to previous input (Shift+Tab)
    fn prev_input(&mut self, cx: &mut Context<Self>) {
        if !self.inputs.is_empty() {
            if self.current_input == 0 {
                self.current_input = self.inputs.len() - 1;
            } else {
                self.current_input -= 1;
            }
            cx.notify();
        }
    }

    /// Handle character input for current field
    fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        if let Some(value) = self.values.get_mut(self.current_input) {
            value.push(ch);
            cx.notify();
        }
    }

    /// Handle backspace for current field
    fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if let Some(value) = self.values.get_mut(self.current_input) {
            if !value.is_empty() {
                value.pop();
                cx.notify();
            }
        }
    }
}

impl Focusable for TemplatePrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TemplatePrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let colors = tokens.colors();
        let spacing = tokens.spacing();

        let handle_key = cx.listener(
            |this: &mut Self,
             event: &gpui::KeyDownEvent,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();

                match key_str.as_str() {
                    "tab" => {
                        if event.keystroke.modifiers.shift {
                            this.prev_input(cx);
                        } else {
                            this.next_input(cx);
                        }
                    }
                    "enter" => this.submit(),
                    "escape" => this.submit_cancel(),
                    "backspace" => this.handle_backspace(cx),
                    _ => {
                        if let Some(ref key_char) = event.keystroke.key_char {
                            if let Some(ch) = key_char.chars().next() {
                                if !ch.is_control() {
                                    this.handle_char(ch, cx);
                                }
                            }
                        }
                    }
                }
            },
        );

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(&self.theme);

        let (_main_bg, text_color, muted_color, border_color) =
            if self.design_variant == DesignVariant::Default {
                (
                    rgb(self.theme.colors.background.main),
                    rgb(self.theme.colors.text.secondary),
                    rgb(self.theme.colors.text.muted),
                    rgb(self.theme.colors.ui.border),
                )
            } else {
                (
                    rgb(colors.background),
                    rgb(colors.text_secondary),
                    rgb(colors.text_muted),
                    rgb(colors.border),
                )
            };

        let preview = self.preview_template();

        let mut container = div()
            .id(gpui::ElementId::Name("window:template".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg)) // Only apply bg when vibrancy disabled
            .text_color(text_color)
            .p(px(spacing.padding_lg))
            .key_context("template_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key);

        // Preview section with live template
        container = container
            .child(div().text_sm().text_color(muted_color).child("Preview:"))
            .child(
                div()
                    .mt(px(spacing.padding_sm))
                    .px(px(spacing.item_padding_x))
                    .py(px(spacing.padding_md))
                    .bg(rgb(self.theme.colors.background.search_box))
                    .border_1()
                    .border_color(border_color)
                    .rounded(px(4.))
                    .text_base()
                    .child(preview),
            );

        // Input fields section
        if self.inputs.is_empty() {
            container = container.child(
                div()
                    .mt(px(spacing.padding_lg))
                    .text_color(muted_color)
                    .child("No {{placeholders}} found in template"),
            );
        } else {
            container = container.child(
                div()
                    .mt(px(spacing.padding_lg))
                    .text_sm()
                    .text_color(muted_color)
                    .child(format!(
                        "Tab through {} field(s), Enter to submit",
                        self.inputs.len()
                    )),
            );

            for (idx, input) in self.inputs.iter().enumerate() {
                let is_current = idx == self.current_input;
                let value = self.values.get(idx).cloned().unwrap_or_default();

                let display = if value.is_empty() {
                    SharedString::from(input.placeholder.clone())
                } else {
                    SharedString::from(value.clone())
                };

                // Use low-opacity for vibrancy support (see VIBRANCY.md)
                let field_bg = if is_current {
                    rgba((self.theme.colors.accent.selected_subtle << 8) | 0x0f)
                // ~6% opacity
                } else {
                    rgb(self.theme.colors.background.search_box)
                };

                let field_border = if is_current {
                    rgb(self.theme.colors.accent.selected)
                } else {
                    border_color
                };

                let text_display_color = if value.is_empty() {
                    muted_color
                } else {
                    text_color
                };

                container = container.child(
                    div()
                        .mt(px(spacing.padding_sm))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .w(px(100.))
                                .text_sm()
                                .text_color(muted_color)
                                .child(format!("{{{{{}}}}}:", input.name)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .px(px(spacing.item_padding_x))
                                .py(px(spacing.padding_sm))
                                .bg(field_bg)
                                .border_1()
                                .border_color(field_border)
                                .rounded(px(4.))
                                .text_color(text_display_color)
                                .child(display),
                        ),
                );
            }
        }

        // Help text at bottom
        container = container.child(
            div()
                .mt(px(spacing.padding_lg))
                .text_xs()
                .text_color(muted_color)
                .child("Tab: next field | Shift+Tab: previous | Enter: submit | Escape: cancel"),
        );

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_placeholder() {
        let inputs = TemplatePrompt::parse_template_inputs("Hello {{name}}!");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].name, "name");
    }

    #[test]
    fn test_parse_multiple_placeholders() {
        let inputs =
            TemplatePrompt::parse_template_inputs("Hello {{name}}, your email is {{email}}");
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].name, "name");
        assert_eq!(inputs[1].name, "email");
    }

    #[test]
    fn test_parse_duplicate_placeholders() {
        let inputs =
            TemplatePrompt::parse_template_inputs("{{name}} is {{name}}'s name, email: {{email}}");
        assert_eq!(inputs.len(), 2); // Duplicates should be removed
        assert_eq!(inputs[0].name, "name");
        assert_eq!(inputs[1].name, "email");
    }

    #[test]
    fn test_parse_no_placeholders() {
        let inputs = TemplatePrompt::parse_template_inputs("Hello world!");
        assert_eq!(inputs.len(), 0);
    }

    #[test]
    fn test_parse_placeholder_with_underscore() {
        let inputs = TemplatePrompt::parse_template_inputs("Hello {{first_name}} {{last_name}}!");
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].name, "first_name");
        assert_eq!(inputs[1].name, "last_name");
    }

    #[test]
    fn test_parse_placeholder_with_numbers() {
        let inputs = TemplatePrompt::parse_template_inputs("Field {{field1}} and {{field2}}");
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].name, "field1");
        assert_eq!(inputs[1].name, "field2");
    }
}
