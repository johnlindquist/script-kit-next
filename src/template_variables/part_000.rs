use arboard::Clipboard;
use chrono::{Datelike, Local, Timelike};
use std::collections::HashMap;
use tracing::{debug, warn};
// ============================================================================
// Variable Context
// ============================================================================

/// Context for variable substitution, allowing custom variable values
///
/// Use this when you need to:
/// - Provide custom variable values (e.g., user inputs)
/// - Override built-in variables for testing
/// - Add application-specific variables
#[derive(Debug, Clone, Default)]
pub struct VariableContext {
    /// Custom variable values (name -> value)
    custom_vars: HashMap<String, String>,
    /// Whether to evaluate built-in variables (clipboard, date, etc.)
    /// Defaults to true
    evaluate_builtins: bool,
}
impl VariableContext {
    /// Create a new empty context with built-in evaluation enabled
    pub fn new() -> Self {
        Self {
            custom_vars: HashMap::new(),
            evaluate_builtins: true,
        }
    }

    /// Create a context with only custom variables (no built-ins)
    #[allow(dead_code)]
    pub fn custom_only() -> Self {
        Self {
            custom_vars: HashMap::new(),
            evaluate_builtins: false,
        }
    }

    /// Set a custom variable value
    #[allow(dead_code)]
    pub fn set(&mut self, name: &str, value: &str) -> &mut Self {
        self.custom_vars.insert(name.to_string(), value.to_string());
        self
    }

    /// Set multiple custom variables from a HashMap
    #[allow(dead_code)]
    pub fn set_all(&mut self, vars: HashMap<String, String>) -> &mut Self {
        self.custom_vars.extend(vars);
        self
    }

    /// Get a custom variable value
    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<&String> {
        self.custom_vars.get(name)
    }

    /// Check if built-ins should be evaluated
    pub fn should_evaluate_builtins(&self) -> bool {
        self.evaluate_builtins
    }

    /// Enable or disable built-in variable evaluation
    #[allow(dead_code)]
    pub fn with_builtins(mut self, enabled: bool) -> Self {
        self.evaluate_builtins = enabled;
        self
    }
}
// ============================================================================
// Main Substitution Functions
// ============================================================================

/// Substitute template variables in content using default context
///
/// This is the primary function for variable substitution. It handles:
/// - `${variable}` syntax
/// - `{{variable}}` syntax
/// - All built-in variables (clipboard, date, time, etc.)
///
/// # Arguments
/// * `content` - The template string with variable placeholders
///
/// # Returns
/// The content with all recognized variables substituted
///
pub fn substitute_variables(content: &str) -> String {
    let ctx = VariableContext::new();
    substitute_variables_with_context(content, &ctx)
}
/// Substitute template variables with a custom context
///
/// Use this when you need to provide custom variable values or
/// control which built-ins are evaluated.
///
/// # Arguments
/// * `content` - The template string with variable placeholders
/// * `ctx` - The variable context with custom values and settings
///
/// # Returns
/// The content with all recognized variables substituted
///
pub fn substitute_variables_with_context(content: &str, ctx: &VariableContext) -> String {
    let mut result = content.to_string();

    // Early exit if no variable markers present
    if !result.contains('$') && !result.contains('{') {
        return result;
    }

    // Build the set of values to substitute
    let values = build_variable_values(ctx);

    // Substitute all variables in both syntaxes
    for (name, value) in &values {
        // ${variable} syntax
        let dollar_pattern = format!("${{{}}}", name);
        result = result.replace(&dollar_pattern, value);

        // {{variable}} syntax
        let brace_pattern = format!("{{{{{}}}}}", name);
        result = result.replace(&brace_pattern, value);
    }

    // Handle special JavaScript-style patterns that may appear in Script Kit templates
    // e.g., ${await clipboard.readText()}
    if result.contains("${await clipboard.readText()}") {
        if let Some(clipboard_value) = values.get("clipboard") {
            result = result.replace("${await clipboard.readText()}", clipboard_value);
        }
    }

    result
}
/// Check if content contains any variable placeholders
///
/// Useful for optimization - skip substitution if no variables present
#[allow(dead_code)]
pub fn has_variables(content: &str) -> bool {
    // Check for ${...} pattern
    if content.contains("${") {
        return true;
    }

    // Check for {{...}} pattern (but not {{{ which could be escaped)
    let bytes = content.as_bytes();
    for i in 0..bytes.len().saturating_sub(1) {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            // Make sure it's not a triple brace (escaped)
            if i + 2 < bytes.len() && bytes[i + 2] != b'{' {
                return true;
            }
        }
    }

    false
}
/// Extract variable names from template content
///
/// Returns a list of unique variable names found in the template
#[allow(dead_code)]
pub fn extract_variable_names(content: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut chars = content.chars().peekable();
    while let Some(ch) = chars.next() {
        // Extract ${variable} patterns
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut name = String::new();

            for next_char in chars.by_ref() {
                if next_char == '}' {
                    break;
                }
                name.push(next_char);
            }

            if !name.is_empty() && !seen.contains(&name) {
                // Skip JS-style expressions like "await clipboard.readText()"
                if !name.contains(' ') && !name.contains('(') {
                    seen.insert(name.clone());
                    names.push(name);
                }
            }
            continue;
        }

        // Extract {{variable}} patterns
        if ch == '{' && chars.peek() == Some(&'{') {
            chars.next(); // consume second '{'
            let mut name = String::new();

            while let Some(next_char) = chars.next() {
                if next_char == '}' && chars.peek() == Some(&'}') {
                    chars.next(); // consume second '}'
                    break;
                }
                name.push(next_char);
            }

            let name = name.trim().to_string();
            if !name.is_empty() && !seen.contains(&name) {
                // Skip conditional syntax like #if, /if
                if !name.starts_with('#') && !name.starts_with('/') && name != "else" {
                    seen.insert(name.clone());
                    names.push(name);
                }
            }
        }
    }

    names
}
// ============================================================================
// Built-in Variable Providers
// ============================================================================

/// Build the complete map of variable values
fn build_variable_values(ctx: &VariableContext) -> HashMap<String, String> {
    let mut values = HashMap::new();

    // Add custom variables first (they take precedence)
    for (name, value) in &ctx.custom_vars {
        values.insert(name.clone(), value.clone());
    }

    // Add built-in variables if enabled
    if ctx.should_evaluate_builtins() {
        add_builtin_variables(&mut values);
    }

    values
}
/// Add all built-in variables to the values map
fn add_builtin_variables(values: &mut HashMap<String, String>) {
    // Clipboard (only fetch if not already provided)
    if !values.contains_key("clipboard") {
        if let Some(text) = get_clipboard_text() {
            values.insert("clipboard".to_string(), text);
        }
    }

    // Date/Time variables (only compute if needed - lazy would be better but simple is fine)
    let now = Local::now();

    // Basic date/time
    if !values.contains_key("date") {
        values.insert("date".to_string(), now.format("%Y-%m-%d").to_string());
    }
    if !values.contains_key("time") {
        values.insert("time".to_string(), now.format("%H:%M:%S").to_string());
    }
    if !values.contains_key("datetime") {
        values.insert(
            "datetime".to_string(),
            now.format("%Y-%m-%d %H:%M:%S").to_string(),
        );
    }
    if !values.contains_key("timestamp") {
        values.insert("timestamp".to_string(), now.timestamp().to_string());
    }

    // Extended date formats
    if !values.contains_key("date_short") {
        values.insert("date_short".to_string(), now.format("%m/%d/%Y").to_string());
    }
    if !values.contains_key("date_long") {
        values.insert("date_long".to_string(), now.format("%B %d, %Y").to_string());
    }
    if !values.contains_key("date_iso") {
        values.insert(
            "date_iso".to_string(),
            now.format("%Y-%m-%dT%H:%M:%S%z").to_string(),
        );
    }

    // Time formats
    if !values.contains_key("time_12h") {
        values.insert("time_12h".to_string(), now.format("%-I:%M %p").to_string());
    }
    if !values.contains_key("time_short") {
        values.insert("time_short".to_string(), now.format("%H:%M").to_string());
    }

    // Individual components
    if !values.contains_key("year") {
        values.insert("year".to_string(), now.year().to_string());
    }
    if !values.contains_key("month") {
        values.insert("month".to_string(), now.format("%B").to_string());
    }
    if !values.contains_key("month_num") {
        values.insert("month_num".to_string(), now.month().to_string());
    }
    if !values.contains_key("day") {
        values.insert("day".to_string(), now.format("%A").to_string());
    }
    if !values.contains_key("day_num") {
        values.insert("day_num".to_string(), now.day().to_string());
    }
    if !values.contains_key("hour") {
        values.insert("hour".to_string(), now.hour().to_string());
    }
    if !values.contains_key("minute") {
        values.insert("minute".to_string(), now.minute().to_string());
    }
    if !values.contains_key("second") {
        values.insert("second".to_string(), now.second().to_string());
    }
    if !values.contains_key("weekday") {
        values.insert("weekday".to_string(), now.weekday().to_string());
    }
}
/// Get clipboard text content safely
fn get_clipboard_text() -> Option<String> {
    match Clipboard::new() {
        Ok(mut clipboard) => match clipboard.get_text() {
            Ok(text) => {
                debug!(
                    text_len = text.len(),
                    "Retrieved clipboard text for variable substitution"
                );
                Some(text)
            }
            Err(e) => {
                debug!(error = %e, "Could not get clipboard text (may be image or empty)");
                None
            }
        },
        Err(e) => {
            warn!(error = %e, "Failed to access clipboard for variable substitution");
            None
        }
    }
}
