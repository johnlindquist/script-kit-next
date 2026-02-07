use super::*;

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
    pub(super) fn parse_template_inputs(template: &str) -> Vec<TemplateInput> {
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

    pub(super) fn is_supported_placeholder(raw_placeholder: &str, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        if raw_placeholder.starts_with("{{") {
            return !name.starts_with('#') && !name.starts_with('/') && name != "else";
        }

        !name.chars().any(char::is_whitespace) && !name.contains('(') && !name.contains(')')
    }

    pub(super) fn parse_placeholder_matches(template: &str) -> Vec<TemplatePlaceholderMatch> {
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

    pub(super) fn render_template_single_pass<F>(
        template: &str,
        mut render_placeholder: F,
    ) -> String
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

    pub(super) fn label_for_field(name: &str) -> String {
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

    pub(super) fn placeholder_for_field(name: &str) -> String {
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

    pub(super) fn group_for_field(name: &str) -> String {
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

    pub(super) fn is_required_field(name: &str) -> bool {
        let normalized = name.to_lowercase();
        normalized == "script_name"
            || normalized == "extension_name"
            || normalized == "name"
            || normalized.contains("slug")
    }

    pub(super) fn is_name_field(name: &str) -> bool {
        let normalized = name.to_lowercase();
        normalized == "script_name"
            || normalized == "extension_name"
            || normalized == "name"
            || normalized.contains("slug")
            || normalized.ends_with("_name")
    }

    pub(super) fn is_slug_like(value: &str) -> bool {
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

    pub(super) fn validate_all_inputs(&mut self) -> bool {
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
    pub(super) fn preview_template(&self) -> String {
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
    pub(super) fn submit(&mut self, cx: &mut Context<Self>) {
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
    pub(super) fn submit_cancel(&mut self) {
        (self.on_submit)(self.id.clone(), None);
    }

    /// Move to next input (Tab)
    pub(super) fn next_input(&mut self, cx: &mut Context<Self>) {
        if !self.inputs.is_empty() {
            self.current_input = (self.current_input + 1) % self.inputs.len();
            cx.notify();
        }
    }

    /// Move to previous input (Shift+Tab)
    pub(super) fn prev_input(&mut self, cx: &mut Context<Self>) {
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
    pub(super) fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
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
    pub(super) fn handle_backspace(&mut self, cx: &mut Context<Self>) {
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
