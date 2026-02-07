//! Utilities for constructing AppleScript string literals safely.

/// Escape characters that terminate or mutate AppleScript string literals.
///
/// AppleScript uses double-quoted strings; backslashes and double quotes must
/// be escaped before interpolating untrusted values.
pub fn escape_applescript_string(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::escape_applescript_string;

    #[test]
    fn test_escape_applescript_string_escapes_double_quote_and_backslash() {
        let input = r#"folder "with"\slashes"#;
        let escaped = escape_applescript_string(input);
        assert_eq!(escaped, r#"folder \"with\"\\slashes"#);
    }

    #[test]
    fn test_escape_applescript_string_preserves_single_quotes() {
        let input = "folder/with'single-quote";
        let escaped = escape_applescript_string(input);
        assert_eq!(escaped, input);
    }
}
