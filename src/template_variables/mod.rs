//! Template variable substitution and discovery for Script Kit content.
//!
//! This module centralizes both runtime substitution (replace placeholders
//! with values) and variable discovery (extract placeholders so callers can
//! promote them to prompt inputs).
//!
//! # Supported Placeholder Syntax
//!
//! Both placeholder syntaxes are interchangeable:
//! - `${var}` (JavaScript/shell style)
//! - `{{var}}` (Handlebars/Mustache style)
//!
//! A variable can be written in either form and resolves through the same
//! lookup path.
//!
//! # Discovery and Prompt Promotion Rules
//!
//! [`extract_variable_names`] returns unique variable names in first-seen
//! order and intentionally skips patterns that are not prompt-friendly:
//! - `${...}` entries containing spaces
//! - `${...}` entries containing `(`
//! - Handlebars control tokens such as `{{#if ...}}`, `{{/if}}`, and `{{else}}`
//!
//! This prevents expression placeholders (for example
//! `${await clipboard.readText()}`) and flow-control markers from being
//! promoted as user-editable prompt fields.
//!
//! # Built-in Variables
//!
//! Built-ins are filled from local runtime state unless overridden in
//! [`VariableContext`].
//!
//! | Name | Format | Example |
//! | --- | --- | --- |
//! | `clipboard` | Raw clipboard text | `copied text` |
//! | `date` | `%Y-%m-%d` | `2026-02-12` |
//! | `time` | `%H:%M:%S` | `07:14:41` |
//! | `datetime` | `%Y-%m-%d %H:%M:%S` | `2026-02-12 07:14:41` |
//! | `timestamp` | Unix seconds | `1765235681` |
//! | `date_short` | `%m/%d/%Y` | `02/12/2026` |
//! | `date_long` | `%B %d, %Y` | `February 12, 2026` |
//! | `date_iso` | `%Y-%m-%dT%H:%M:%S%z` | `2026-02-12T07:14:41-0800` |
//! | `time_12h` | `%-I:%M %p` | `7:14 AM` |
//! | `time_short` | `%H:%M` | `07:14` |
//! | `year` | Numeric year | `2026` |
//! | `month` | `%B` | `February` |
//! | `month_num` | Numeric month (`1-12`) | `2` |
//! | `day` | `%A` | `Thursday` |
//! | `day_num` | Numeric day of month (`1-31`) | `12` |
//! | `hour` | Numeric hour (`0-23`) | `7` |
//! | `minute` | Numeric minute (`0-59`) | `14` |
//! | `second` | Numeric second (`0-59`) | `41` |
//! | `weekday` | `chrono::Weekday::to_string()` | `Thu` |
//!
//! # `VariableContext` Precedence and Built-in Control
//!
//! `VariableContext` controls override and disable behavior:
//! - Custom values always win when a name matches a built-in.
//! - [`VariableContext::new`] enables built-in evaluation.
//! - [`VariableContext::custom_only`] disables built-ins from the start.
//! - [`VariableContext::with_builtins(false)`] disables built-ins while
//!   preserving custom values already set in the context.
//! - When built-ins are disabled and no custom value exists, placeholders
//!   remain unchanged in output.
//!
//! # Legacy Compatibility
//!
//! In addition to standard placeholders, the exact string
//! `${await clipboard.readText()}` is replaced with the resolved `clipboard`
//! value so older Script Kit templates continue to work.
//!

// --- merged from part_000.rs ---
use arboard::Clipboard;
use chrono::{Datelike, Local, Timelike};
use std::collections::HashMap;
use tracing::{debug, warn};
// ============================================================================
// Variable Context
// ============================================================================

/// Variable-resolution context used during template substitution.
///
/// Use this when you need to:
/// - Provide custom variable values (e.g., user inputs)
/// - Override built-in variables for testing
/// - Add application-specific variables
/// - Disable built-in variable evaluation entirely
///
/// Resolution order is:
/// 1. `custom_vars` (explicit overrides)
/// 2. Built-ins (only when enabled)
#[derive(Debug, Clone, Default)]
pub struct VariableContext {
    /// Custom variable values (name -> value)
    custom_vars: HashMap<String, String>,
    /// Whether to evaluate built-in variables (clipboard, date, etc.)
    /// Defaults to true
    evaluate_builtins: bool,
}
impl VariableContext {
    /// Create a new empty context with built-ins enabled.
    pub fn new() -> Self {
        Self {
            custom_vars: HashMap::new(),
            evaluate_builtins: true,
        }
    }

    /// Create a context with only custom variables (no built-ins).
    ///
    /// Equivalent behavior to `VariableContext::new().with_builtins(false)`.
    #[allow(dead_code)]
    pub fn custom_only() -> Self {
        Self {
            custom_vars: HashMap::new(),
            evaluate_builtins: false,
        }
    }

    /// Set one custom variable value.
    ///
    /// This value overrides any built-in with the same name.
    #[allow(dead_code)]
    pub fn set(&mut self, name: &str, value: &str) -> &mut Self {
        self.custom_vars.insert(name.to_string(), value.to_string());
        self
    }

    /// Set multiple custom variable values.
    ///
    /// Entries in `vars` override existing custom values with the same name.
    #[allow(dead_code)]
    pub fn set_all(&mut self, vars: HashMap<String, String>) -> &mut Self {
        self.custom_vars.extend(vars);
        self
    }

    /// Get a custom variable value by name.
    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<&String> {
        self.custom_vars.get(name)
    }

    /// Return whether built-in variable evaluation is enabled.
    pub fn should_evaluate_builtins(&self) -> bool {
        self.evaluate_builtins
    }

    /// Enable or disable built-in variable evaluation.
    ///
    /// Disabling built-ins leaves placeholders unchanged unless a matching
    /// custom variable is present.
    #[allow(dead_code)]
    pub fn with_builtins(mut self, enabled: bool) -> Self {
        self.evaluate_builtins = enabled;
        self
    }
}
// ============================================================================
// Main Substitution Functions
// ============================================================================

/// Substitute template variables in content using the default context.
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
/// Substitute template variables with a custom context.
///
/// Use this when you need to provide custom variable values or
/// control which built-ins are evaluated.
///
/// Also supports legacy compatibility replacement of the exact expression
/// `${await clipboard.readText()}` using the resolved `clipboard` value.
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
/// Extract variable names from template content.
///
/// Returns a list of unique variable names found in the template.
///
/// This helper is used by prompt-building flows to determine which variables
/// should be surfaced as user inputs. It intentionally excludes:
/// - `${...}` names containing spaces or `(` (expression-like placeholders)
/// - `{{...}}` control markers (`#...`, `/...`, and `else`)
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

/// Build the complete map of variable values.
///
/// Custom variables are inserted first so they take precedence over built-ins.
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
/// Add all built-in variables to the values map when absent.
///
/// Each key is only inserted if it was not already supplied by custom values.
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

// --- merged from part_001.rs ---
// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Basic Substitution Tests
    // ========================================

    #[test]
    fn test_no_variables_returns_unchanged() {
        let input = "Hello world!";
        let result = substitute_variables(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_dollar_brace_syntax() {
        let mut ctx = VariableContext::custom_only();
        ctx.set("name", "Alice");

        let result = substitute_variables_with_context("Hello ${name}!", &ctx);
        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_double_brace_syntax() {
        let mut ctx = VariableContext::custom_only();
        ctx.set("name", "Bob");

        let result = substitute_variables_with_context("Hello {{name}}!", &ctx);
        assert_eq!(result, "Hello Bob!");
    }

    #[test]
    fn test_mixed_syntax() {
        let mut ctx = VariableContext::custom_only();
        ctx.set("first", "John");
        ctx.set("last", "Doe");

        let result = substitute_variables_with_context("${first} {{last}}", &ctx);
        assert_eq!(result, "John Doe");
    }

    #[test]
    fn test_multiple_same_variable() {
        let mut ctx = VariableContext::custom_only();
        ctx.set("x", "test");

        let result = substitute_variables_with_context("${x} and {{x}} and ${x}", &ctx);
        assert_eq!(result, "test and test and test");
    }

    #[test]
    fn test_unknown_variable_unchanged() {
        let ctx = VariableContext::custom_only();

        let result = substitute_variables_with_context("Hello ${unknown}!", &ctx);
        assert_eq!(result, "Hello ${unknown}!");
    }

    #[test]
    fn test_empty_variable_name() {
        let ctx = VariableContext::custom_only();

        let result = substitute_variables_with_context("Hello ${}!", &ctx);
        assert_eq!(result, "Hello ${}!");
    }

    // ========================================
    // Built-in Variable Tests
    // ========================================

    #[test]
    fn test_date_variable_format() {
        let result = substitute_variables("${date}");
        // Should be YYYY-MM-DD format
        assert!(result.len() == 10, "Date should be 10 chars: {}", result);
        assert!(
            result.contains('-'),
            "Date should contain dashes: {}",
            result
        );
    }

    #[test]
    fn test_time_variable_format() {
        let result = substitute_variables("${time}");
        // Should be HH:MM:SS format
        assert!(result.len() == 8, "Time should be 8 chars: {}", result);
        assert!(
            result.contains(':'),
            "Time should contain colons: {}",
            result
        );
    }

    #[test]
    fn test_datetime_variable_format() {
        let result = substitute_variables("${datetime}");
        // Should be YYYY-MM-DD HH:MM:SS format (19 chars)
        assert!(
            result.len() == 19,
            "Datetime should be 19 chars: {}",
            result
        );
        assert!(
            result.contains(' '),
            "Datetime should contain space: {}",
            result
        );
    }

    #[test]
    fn test_timestamp_is_numeric() {
        let result = substitute_variables("${timestamp}");
        assert!(
            result.parse::<i64>().is_ok(),
            "Timestamp should be numeric: {}",
            result
        );
    }

    #[test]
    fn test_year_is_four_digits() {
        let result = substitute_variables("${year}");
        assert!(result.len() == 4, "Year should be 4 digits: {}", result);
        assert!(
            result.parse::<u32>().is_ok(),
            "Year should be numeric: {}",
            result
        );
    }

    #[test]
    fn test_month_is_word() {
        let result = substitute_variables("${month}");
        let months = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];
        assert!(
            months.contains(&result.as_str()),
            "Month should be full name: {}",
            result
        );
    }

    #[test]
    fn test_day_is_weekday() {
        let result = substitute_variables("${day}");
        let days = [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ];
        assert!(
            days.contains(&result.as_str()),
            "Day should be weekday name: {}",
            result
        );
    }

    #[test]
    fn test_date_short_format() {
        let result = substitute_variables("${date_short}");
        // Should be MM/DD/YYYY format (10 chars)
        assert!(
            result.len() == 10,
            "date_short should be 10 chars: {}",
            result
        );
        assert!(
            result.contains('/'),
            "date_short should contain slashes: {}",
            result
        );
    }

    #[test]
    fn test_time_12h_format() {
        let result = substitute_variables("${time_12h}");
        // Should contain AM or PM
        assert!(
            result.contains("AM") || result.contains("PM"),
            "time_12h should contain AM/PM: {}",
            result
        );
    }

    // ========================================
    // Context Tests
    // ========================================

    #[test]
    fn test_custom_overrides_builtin() {
        let mut ctx = VariableContext::new();
        ctx.set("date", "CUSTOM_DATE");

        let result = substitute_variables_with_context("${date}", &ctx);
        assert_eq!(result, "CUSTOM_DATE");
    }

    #[test]
    fn test_custom_only_no_builtins() {
        let ctx = VariableContext::custom_only();

        let result = substitute_variables_with_context("${date}", &ctx);
        // Should remain unchanged since builtins are disabled
        assert_eq!(result, "${date}");
    }

    #[test]
    fn test_context_set_returns_self() {
        let mut ctx = VariableContext::new();
        ctx.set("a", "1").set("b", "2");

        assert_eq!(ctx.get("a"), Some(&"1".to_string()));
        assert_eq!(ctx.get("b"), Some(&"2".to_string()));
    }

    // ========================================
    // Helper Function Tests
    // ========================================

    #[test]
    fn test_has_variables_dollar() {
        assert!(has_variables("Hello ${name}!"));
        assert!(has_variables("${a} ${b}"));
    }

    #[test]
    fn test_has_variables_braces() {
        assert!(has_variables("Hello {{name}}!"));
        assert!(has_variables("{{a}} {{b}}"));
    }

    #[test]
    fn test_has_variables_none() {
        assert!(!has_variables("Hello world!"));
        assert!(!has_variables(""));
        assert!(!has_variables("Just some text"));
    }

    #[test]
    fn test_extract_variable_names_dollar() {
        let names = extract_variable_names("${first} ${second}");
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"first".to_string()));
        assert!(names.contains(&"second".to_string()));
    }

    #[test]
    fn test_extract_variable_names_braces() {
        let names = extract_variable_names("{{name}} {{email}}");
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"name".to_string()));
        assert!(names.contains(&"email".to_string()));
    }

    #[test]
    fn test_extract_variable_names_mixed() {
        let names = extract_variable_names("${a} and {{b}}");
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"a".to_string()));
        assert!(names.contains(&"b".to_string()));
    }

    #[test]
    fn test_extract_variable_names_duplicates() {
        let names = extract_variable_names("${x} {{x}} ${x}");
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "x");
    }

    #[test]
    fn test_extract_ignores_conditionals() {
        let names = extract_variable_names("{{#if flag}}{{name}}{{/if}}");
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "name");
        assert!(!names.contains(&"#if flag".to_string()));
        assert!(!names.contains(&"/if".to_string()));
    }

    #[test]
    fn test_extract_ignores_js_expressions() {
        let names = extract_variable_names("${await clipboard.readText()}");
        // Should not extract "await clipboard.readText()" as a variable name
        assert!(names.is_empty());
    }

    // ========================================
    // Edge Cases
    // ========================================

    #[test]
    fn test_nested_braces() {
        let mut ctx = VariableContext::custom_only();
        ctx.set("x", "value");

        // This is an edge case - nested braces
        let result = substitute_variables_with_context("{{{x}}}", &ctx);
        // Should handle gracefully
        assert!(result.contains("value") || result.contains("{{"));
    }

    #[test]
    fn test_unclosed_variable() {
        let ctx = VariableContext::custom_only();

        // Unclosed variables should remain unchanged
        let result = substitute_variables_with_context("Hello ${name", &ctx);
        assert_eq!(result, "Hello ${name");
    }

    #[test]
    fn test_empty_content() {
        let result = substitute_variables("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_special_characters_in_value() {
        let mut ctx = VariableContext::custom_only();
        ctx.set("special", "Hello ${{world}}!");

        let result = substitute_variables_with_context("Value: ${special}", &ctx);
        assert_eq!(result, "Value: Hello ${{world}}!");
    }

    #[test]
    fn test_unicode_in_value() {
        let mut ctx = VariableContext::custom_only();
        ctx.set("greeting", "こんにちは 🎉");

        let result = substitute_variables_with_context("${greeting}", &ctx);
        assert_eq!(result, "こんにちは 🎉");
    }

    #[test]
    fn test_multiline_content() {
        let mut ctx = VariableContext::custom_only();
        ctx.set("name", "World");

        let input = "Hello\n${name}\nGoodbye";
        let result = substitute_variables_with_context(input, &ctx);
        assert_eq!(result, "Hello\nWorld\nGoodbye");
    }

    #[test]
    fn test_js_clipboard_syntax() {
        let mut ctx = VariableContext::new();
        ctx.set("clipboard", "clipboard content");

        let result = substitute_variables_with_context("Text: ${await clipboard.readText()}", &ctx);
        assert_eq!(result, "Text: clipboard content");
    }

    // ========================================
    // Integration-style Tests
    // ========================================

    #[test]
    fn test_realistic_email_template() {
        let mut ctx = VariableContext::custom_only();
        ctx.set("name", "John Doe");
        ctx.set("date", "2024-01-15");

        let template = r#"Dear {{name}},

As of ${date}, your account has been updated.

Best regards,
The Team"#;

        let expected = r#"Dear John Doe,

As of 2024-01-15, your account has been updated.

Best regards,
The Team"#;

        let result = substitute_variables_with_context(template, &ctx);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_realistic_signature() {
        let mut ctx = VariableContext::custom_only();
        ctx.set("clipboard", "Important message content here");

        let template = r#"Please review the following:
<clipboard>${clipboard}</clipboard>

Thank you!"#;

        let expected = r#"Please review the following:
<clipboard>Important message content here</clipboard>

Thank you!"#;

        let result = substitute_variables_with_context(template, &ctx);
        assert_eq!(result, expected);
    }
}
