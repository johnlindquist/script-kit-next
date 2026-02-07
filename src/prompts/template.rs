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
use std::collections::HashMap;
use std::sync::Arc;

use crate::designs::{get_tokens, DesignVariant};
use crate::logging;
use crate::panel::PROMPT_INPUT_FIELD_HEIGHT;
use crate::template_variables;
use crate::theme;
use crate::ui_foundation::get_vibrancy_background;

use super::SubmitCallback;

/// Input definition for a template placeholder
#[derive(Clone, Debug)]
pub struct TemplateInput {
    /// Name of the placeholder (e.g., "name", "email")
    pub name: String,
    /// Human-readable label shown in form UI
    pub label: String,
    /// Placeholder text to show when empty
    pub placeholder: String,
    /// Group header for visual organization
    pub group: String,
    /// Whether this field must be provided
    pub required: bool,
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
    /// Per-field validation errors
    pub validation_errors: Vec<Option<String>>,
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

#[derive(Debug)]
struct TemplatePlaceholderMatch {
    start: usize,
    end: usize,
    name: String,
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
        let validation_errors: Vec<Option<String>> = inputs.iter().map(|_| None).collect();

        TemplatePrompt {
            id,
            template,
            inputs,
            values,
            validation_errors,
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
        template_variables::extract_variable_names(template)
            .into_iter()
            .map(|name| TemplateInput {
                label: Self::label_for_field(&name),
                placeholder: Self::placeholder_for_field(&name),
                group: Self::group_for_field(&name),
                required: Self::is_required_field(&name),
                name,
            })
            .collect()
    }

    fn is_supported_placeholder(raw_placeholder: &str, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        if raw_placeholder.starts_with("{{") {
            return !name.starts_with('#') && !name.starts_with('/') && name != "else";
        }

        !name.chars().any(char::is_whitespace) && !name.contains('(') && !name.contains(')')
    }

    fn parse_placeholder_matches(template: &str) -> Vec<TemplatePlaceholderMatch> {
        let placeholder_re =
            Regex::new(r"\{\{\s*([^{}]+?)\s*\}\}|\$\{([^}]+)\}").expect("Invalid regex");
        let mut matches = Vec::new();

        for captures in placeholder_re.captures_iter(template) {
            let Some(full_match) = captures.get(0) else {
                continue;
            };
            let Some(name_match) = captures.get(1).or_else(|| captures.get(2)) else {
                continue;
            };

            let name = name_match.as_str().trim();
            if !Self::is_supported_placeholder(full_match.as_str(), name) {
                continue;
            }

            matches.push(TemplatePlaceholderMatch {
                start: full_match.start(),
                end: full_match.end(),
                name: name.to_string(),
            });
        }

        matches
    }

    fn render_template_single_pass<F>(template: &str, mut render_placeholder: F) -> String
    where
        F: FnMut(&str, &str) -> String,
    {
        let matches = Self::parse_placeholder_matches(template);
        if matches.is_empty() {
            return template.to_string();
        }

        let mut result = String::with_capacity(template.len());
        let mut cursor = 0;

        for placeholder_match in matches {
            if placeholder_match.start > cursor {
                result.push_str(&template[cursor..placeholder_match.start]);
            }

            let raw_placeholder = &template[placeholder_match.start..placeholder_match.end];
            result.push_str(&render_placeholder(
                &placeholder_match.name,
                raw_placeholder,
            ));
            cursor = placeholder_match.end;
        }

        if cursor < template.len() {
            result.push_str(&template[cursor..]);
        }

        result
    }

    fn label_for_field(name: &str) -> String {
        let normalized = name.to_lowercase();
        match normalized.as_str() {
            "script_name" => "Script Name".to_string(),
            "extension_name" => "Scriptlet Bundle Name".to_string(),
            "name" => "Name".to_string(),
            "author" => "Author".to_string(),
            "description" => "Description".to_string(),
            "icon" => "Icon".to_string(),
            _ => normalized
                .split('_')
                .filter(|part| !part.is_empty())
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        Some(first) => {
                            let mut word = first.to_uppercase().to_string();
                            word.push_str(chars.as_str());
                            word
                        }
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    fn placeholder_for_field(name: &str) -> String {
        let normalized = name.to_lowercase();
        match normalized.as_str() {
            "script_name" => "my-script-name".to_string(),
            "extension_name" => "my-scriptlet-bundle".to_string(),
            "name" => "My Bundle".to_string(),
            "author" => "Your Name".to_string(),
            "description" => "What this template creates".to_string(),
            "icon" => "wrench".to_string(),
            _ if normalized.contains("name") || normalized.contains("slug") => {
                "my-script-name".to_string()
            }
            _ => format!("Enter {}", normalized.replace('_', " ")),
        }
    }

    fn group_for_field(name: &str) -> String {
        let normalized = name.to_lowercase();
        if normalized.contains("name") || normalized.contains("slug") {
            "Naming".to_string()
        } else if normalized.contains("author")
            || normalized.contains("description")
            || normalized.contains("icon")
            || normalized.contains("tag")
        {
            "Metadata".to_string()
        } else if normalized.contains("content")
            || normalized.contains("body")
            || normalized.contains("template")
            || normalized.contains("command")
        {
            "Content".to_string()
        } else {
            "Details".to_string()
        }
    }

    fn is_required_field(name: &str) -> bool {
        let normalized = name.to_lowercase();
        normalized == "script_name"
            || normalized == "extension_name"
            || normalized == "name"
            || normalized.contains("slug")
    }

    fn is_name_field(name: &str) -> bool {
        let normalized = name.to_lowercase();
        normalized == "script_name"
            || normalized == "extension_name"
            || normalized == "name"
            || normalized.contains("slug")
            || normalized.ends_with("_name")
    }

    fn is_slug_like(value: &str) -> bool {
        if value.is_empty() || value.starts_with('-') || value.ends_with('-') {
            return false;
        }

        let mut previous_hyphen = false;
        for ch in value.chars() {
            if ch == '-' {
                if previous_hyphen {
                    return false;
                }
                previous_hyphen = true;
                continue;
            }

            if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() {
                return false;
            }
            previous_hyphen = false;
        }

        true
    }

    pub fn validate_input_value(input: &TemplateInput, raw_value: &str) -> Result<(), String> {
        let value = raw_value.trim();

        if input.required && value.is_empty() {
            return Err(format!("{} is required", input.label));
        }

        if value.is_empty() {
            return Ok(());
        }

        if Self::is_name_field(&input.name) && !Self::is_slug_like(value) {
            return Err(format!(
                "{} must use lowercase letters, numbers, and hyphens",
                input.label
            ));
        }

        Ok(())
    }

    fn validate_all_inputs(&mut self) -> bool {
        let mut is_valid = true;

        for idx in 0..self.inputs.len() {
            let value = self.values.get(idx).map(String::as_str).unwrap_or_default();
            let validation = Self::validate_input_value(&self.inputs[idx], value);
            self.validation_errors[idx] = validation.err();
            if self.validation_errors[idx].is_some() {
                is_valid = false;
            }
        }

        is_valid
    }

    /// Get the filled template string by replacing all placeholders
    pub fn filled_template(&self) -> String {
        let values_by_name: HashMap<&str, &str> = self
            .inputs
            .iter()
            .zip(self.values.iter())
            .map(|(input, value)| (input.name.as_str(), value.as_str()))
            .collect();

        Self::render_template_single_pass(&self.template, |name, raw_placeholder| {
            match values_by_name.get(name).copied() {
                Some(value) if !value.is_empty() => value.to_string(),
                _ => raw_placeholder.to_string(),
            }
        })
    }

    /// Get the preview string - shows filled values or placeholder hints
    fn preview_template(&self) -> String {
        let values_by_name: HashMap<&str, &str> = self
            .inputs
            .iter()
            .zip(self.values.iter())
            .map(|(input, value)| (input.name.as_str(), value.as_str()))
            .collect();
        let labels_by_name: HashMap<&str, &str> = self
            .inputs
            .iter()
            .map(|input| (input.name.as_str(), input.label.as_str()))
            .collect();

        Self::render_template_single_pass(&self.template, |name, raw_placeholder| {
            match values_by_name.get(name).copied() {
                Some(value) if !value.is_empty() => value.to_string(),
                Some(_) => {
                    let label = labels_by_name.get(name).copied().unwrap_or(name);
                    format!("[{}]", label)
                }
                None => raw_placeholder.to_string(),
            }
        })
    }

    /// Set the current input value programmatically
    pub fn set_input(&mut self, text: String, cx: &mut Context<Self>) {
        if let Some(value) = self.values.get_mut(self.current_input) {
            if *value == text {
                return;
            }
            *value = text;
            if let Some(input) = self.inputs.get(self.current_input) {
                self.validation_errors[self.current_input] =
                    Self::validate_input_value(input, value).err();
            }
            cx.notify();
        }
    }

    /// Submit the filled template
    fn submit(&mut self, cx: &mut Context<Self>) {
        if !self.validate_all_inputs() {
            if let Some(first_invalid) = self.validation_errors.iter().position(Option::is_some) {
                self.current_input = first_invalid;
            }
            cx.notify();
            return;
        }

        // Replace placeholders with actual values for final submission in a single pass.
        let values_by_name: HashMap<&str, &str> = self
            .inputs
            .iter()
            .zip(self.values.iter())
            .map(|(input, value)| (input.name.as_str(), value.as_str()))
            .collect();
        let result = Self::render_template_single_pass(&self.template, |name, raw_placeholder| {
            values_by_name
                .get(name)
                .map(|value| value.trim().to_string())
                .unwrap_or_else(|| raw_placeholder.to_string())
        });
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
            if let Some(input) = self.inputs.get(self.current_input) {
                self.validation_errors[self.current_input] =
                    Self::validate_input_value(input, value).err();
            }
            cx.notify();
        }
    }

    /// Handle backspace for current field
    fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if let Some(value) = self.values.get_mut(self.current_input) {
            if !value.is_empty() {
                value.pop();
                if let Some(input) = self.inputs.get(self.current_input) {
                    self.validation_errors[self.current_input] =
                        Self::validate_input_value(input, value).err();
                }
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
                    "enter" | "return" => this.submit(cx),
                    "escape" | "esc" => this.submit_cancel(),
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
        let error_color = rgb(self.theme.colors.accent.selected);

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
                        "Fill {} field(s). Required fields are marked with *.",
                        self.inputs.len()
                    )),
            );

            let mut previous_group: Option<String> = None;
            for (idx, input) in self.inputs.iter().enumerate() {
                if previous_group.as_deref() != Some(input.group.as_str()) {
                    previous_group = Some(input.group.clone());
                    container = container.child(
                        div()
                            .mt(px(spacing.padding_md))
                            .text_xs()
                            .text_color(muted_color)
                            .child(input.group.clone()),
                    );
                }

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

                let label = if input.required {
                    format!("{} *", input.label)
                } else {
                    input.label.clone()
                };

                let mut row = div()
                    .mt(px(spacing.padding_sm))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(140.))
                                    .text_sm()
                                    .text_color(muted_color)
                                    .child(label),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(PROMPT_INPUT_FIELD_HEIGHT))
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

                if let Some(Some(error_message)) = self.validation_errors.get(idx) {
                    row = row.child(
                        div()
                            .pl(px(144.))
                            .text_xs()
                            .text_color(error_color)
                            .child(error_message.clone()),
                    );
                }

                container = container.child(row);
            }
        }

        let has_name_fields = self
            .inputs
            .iter()
            .any(|input| Self::is_name_field(&input.name));
        if has_name_fields {
            container = container.child(
                div()
                    .mt(px(spacing.padding_md))
                    .text_xs()
                    .text_color(muted_color)
                    .child("Naming tip: use lowercase letters, numbers, and hyphens."),
            );
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

    #[test]
    fn test_parse_dollar_brace_placeholders() {
        let inputs =
            TemplatePrompt::parse_template_inputs("Script ${script_name} by ${author_name}");
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].name, "script_name");
        assert_eq!(inputs[1].name, "author_name");
    }

    #[test]
    fn test_parse_brace_placeholders_with_whitespace_and_skip_control_tags() {
        let inputs = TemplatePrompt::parse_template_inputs(
            "Hello {{ first_name }} {{#if cond}}ignored{{/if}} {{ else }} {{last_name}}",
        );
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].name, "first_name");
        assert_eq!(inputs[1].name, "last_name");
    }

    #[test]
    fn test_parse_skips_javascript_expressions_in_dollar_syntax() {
        let inputs =
            TemplatePrompt::parse_template_inputs("${await clipboard.readText()} {{name}}");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].name, "name");
    }

    #[test]
    fn test_template_inputs_use_human_labels_and_groups() {
        let inputs = TemplatePrompt::parse_template_inputs(
            "Name {{script_name}} by {{author}}: {{description}}",
        );
        assert_eq!(inputs.len(), 3);

        assert_eq!(inputs[0].label, "Script Name");
        assert_eq!(inputs[0].placeholder, "my-script-name");
        assert_eq!(inputs[0].group, "Naming");

        assert_eq!(inputs[1].label, "Author");
        assert_eq!(inputs[1].placeholder, "Your Name");
        assert_eq!(inputs[1].group, "Metadata");

        assert_eq!(inputs[2].label, "Description");
        assert_eq!(inputs[2].placeholder, "What this template creates");
        assert_eq!(inputs[2].group, "Metadata");
    }

    #[test]
    fn test_template_prompt_substitute_single_pass_does_not_rewrite_user_literal_placeholders() {
        let rendered = TemplatePrompt::render_template_single_pass(
            "{{first}} and {{second}}",
            |name, raw_placeholder| match name {
                "first" => "{{second}}".to_string(),
                "second" => "done".to_string(),
                _ => raw_placeholder.to_string(),
            },
        );

        assert_eq!(rendered, "{{second}} and done");
    }

    #[test]
    fn test_single_pass_substitution_skips_javascript_style_expressions() {
        let rendered = TemplatePrompt::render_template_single_pass(
            "${await clipboard.readText()} {{name}}",
            |name, raw_placeholder| match name {
                "name" => "Alice".to_string(),
                _ => raw_placeholder.to_string(),
            },
        );

        assert_eq!(rendered, "${await clipboard.readText()} Alice");
    }

    #[test]
    fn test_validate_name_inputs_require_slug_like_values() {
        let input = TemplateInput {
            name: "script_name".to_string(),
            label: "Script Name".to_string(),
            placeholder: "my-script-name".to_string(),
            group: "Naming".to_string(),
            required: true,
        };

        let err = TemplatePrompt::validate_input_value(&input, "My Cool Script")
            .expect_err("spaces should fail slug validation for script naming");
        assert!(err.contains("letters, numbers, and hyphens"));

        assert!(TemplatePrompt::validate_input_value(&input, "my-cool-script").is_ok());
    }
}
