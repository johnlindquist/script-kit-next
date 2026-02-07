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
