//! Input detection module for smart fallback commands
//!
//! This module provides functions to detect the type of user input
//! for displaying relevant fallback commands (Raycast-style).

use regex::Regex;
use std::sync::LazyLock;

/// Types of input that can be detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputType {
    /// HTTP, HTTPS, or file:// URLs
    Url,
    /// Absolute paths (/path), home paths (~/path), relative paths (./path)
    FilePath,
    /// Mathematical expressions (2+2, 100*50, etc.)
    MathExpression,
    /// Code snippets (function, const, import, etc.)
    CodeSnippet,
    /// Default fallback for plain text
    PlainText,
}

// Compile regex patterns once using LazyLock
static URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(https?://|file://)[^\s]+$").expect("Invalid URL regex"));

static MATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Matches strings containing only digits, operators, parens, spaces, and decimal points
    // Must contain at least one digit and one operator to be considered math
    Regex::new(r"^[\d\s\+\-\*/%\^\(\)\.]+$").expect("Invalid math regex")
});

// Code keywords to detect
const CODE_KEYWORDS: &[&str] = &[
    "function", "const ", "let ", "var ", "import ", "export ", "=>", "class ", "def ", "fn ",
    "pub fn", "async ", "await ", "return ", "if (", "for (", "while (",
];

/// Detect the type of user input
///
/// Returns the most specific type that matches the input.
/// Priority order: URL > FilePath > MathExpression > CodeSnippet > PlainText
pub fn detect_input_type(input: &str) -> InputType {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return InputType::PlainText;
    }

    // Check in priority order
    if is_url(trimmed) {
        return InputType::Url;
    }

    if is_file_path(trimmed) {
        return InputType::FilePath;
    }

    if is_math_expression(trimmed) {
        return InputType::MathExpression;
    }

    if is_code_snippet(trimmed) {
        return InputType::CodeSnippet;
    }

    InputType::PlainText
}

/// Check if the input is a URL
///
/// Matches:
/// - `http://example.com`
/// - `https://example.com/path?query=1`
/// - `file:///path/to/file`
pub fn is_url(input: &str) -> bool {
    let trimmed = input.trim();

    // Quick prefix check before regex
    if !trimmed.starts_with("http://")
        && !trimmed.starts_with("https://")
        && !trimmed.starts_with("file://")
    {
        return false;
    }

    URL_REGEX.is_match(trimmed)
}

/// Check if the input is a file path
///
/// Matches:
/// - `/absolute/path`
/// - `~/home/path`
/// - `./relative/path`
/// - `../parent/path`
/// - `C:\Windows\path` (Windows drive letter)
/// - `D:/path` (Windows with forward slash)
pub fn is_file_path(input: &str) -> bool {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return false;
    }

    // Unix-style absolute path
    if trimmed.starts_with('/') {
        return true;
    }

    // Home directory path
    if trimmed.starts_with("~/") {
        return true;
    }

    // Relative paths
    if trimmed.starts_with("./") || trimmed.starts_with("../") {
        return true;
    }

    // Windows drive letter (C:\ or C:/)
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() >= 3
        && chars[0].is_ascii_alphabetic()
        && chars[1] == ':'
        && (chars[2] == '\\' || chars[2] == '/')
    {
        return true;
    }

    false
}

/// Check if the input is a mathematical expression
///
/// Matches expressions containing:
/// - Digits (0-9)
/// - Operators (+, -, *, /, %, ^)
/// - Parentheses
/// - Decimal points
/// - Spaces
///
/// Must contain at least one digit and one operator to be considered math.
pub fn is_math_expression(input: &str) -> bool {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return false;
    }

    // Must match the math pattern (only valid characters)
    if !MATH_REGEX.is_match(trimmed) {
        return false;
    }

    // Must contain at least one digit
    let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());
    if !has_digit {
        return false;
    }

    // Must contain at least one operator to be an expression
    let has_operator = trimmed
        .chars()
        .any(|c| matches!(c, '+' | '-' | '*' | '/' | '%' | '^'));

    has_operator
}

/// Check if the input looks like a code snippet
///
/// Matches input containing common programming keywords:
/// - `function`, `const`, `let`, `var` (JavaScript)
/// - `import`, `export` (ES modules)
/// - `=>` (arrow functions)
/// - `class` (OOP)
/// - `def` (Python)
/// - `fn`, `pub fn` (Rust)
/// - `async`, `await`, `return`
/// - Control flow: `if (`, `for (`, `while (`
pub fn is_code_snippet(input: &str) -> bool {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return false;
    }

    // Convert to lowercase for case-insensitive matching of keywords
    let lower = trimmed.to_lowercase();

    // Check for code keywords
    for keyword in CODE_KEYWORDS {
        if lower.contains(keyword) {
            return true;
        }
    }

    // Check for common code patterns
    // Assignment with = (but not == or ===)
    if lower.contains(" = ") && !lower.contains("==") {
        return true;
    }

    // Function call pattern: word followed by parentheses
    // e.g., foo(), bar(1, 2)
    if Regex::new(r"\w+\s*\([^)]*\)")
        .map(|r| r.is_match(trimmed))
        .unwrap_or(false)
    {
        // But exclude math expressions like (1+2)
        // Only match if there's a word before the paren
        if Regex::new(r"[a-zA-Z_]\w*\s*\(")
            .map(|r| r.is_match(trimmed))
            .unwrap_or(false)
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== URL Detection Tests ====================

    #[test]
    fn test_is_url_http() {
        assert!(is_url("http://example.com"));
        assert!(is_url("http://localhost:3000"));
        assert!(is_url("http://example.com/path"));
    }

    #[test]
    fn test_is_url_https() {
        assert!(is_url("https://example.com"));
        assert!(is_url("https://example.com/path?query=1"));
        assert!(is_url("https://sub.example.com/path#anchor"));
    }

    #[test]
    fn test_is_url_file() {
        assert!(is_url("file:///path/to/file"));
        assert!(is_url("file:///Users/john/document.pdf"));
    }

    #[test]
    fn test_is_url_negative() {
        assert!(!is_url("not a url"));
        assert!(!is_url("example.com")); // Missing protocol
        assert!(!is_url("ftp://example.com")); // Not supported
        assert!(!is_url("/path/to/file")); // File path, not URL
    }

    #[test]
    fn test_is_url_with_whitespace() {
        assert!(is_url("  https://example.com  "));
        assert!(!is_url("https://example .com")); // Space in URL
    }

    // ==================== File Path Detection Tests ====================

    #[test]
    fn test_is_file_path_absolute() {
        assert!(is_file_path("/"));
        assert!(is_file_path("/path/to/file"));
        assert!(is_file_path("/Users/john/Documents/file.txt"));
    }

    #[test]
    fn test_is_file_path_home() {
        assert!(is_file_path("~/"));
        assert!(is_file_path("~/Documents"));
        assert!(is_file_path("~/Documents/file.txt"));
    }

    #[test]
    fn test_is_file_path_relative() {
        assert!(is_file_path("./file.txt"));
        assert!(is_file_path("./path/to/file"));
        assert!(is_file_path("../parent/file"));
    }

    #[test]
    fn test_is_file_path_windows() {
        assert!(is_file_path("C:\\Users\\john"));
        assert!(is_file_path("D:/Documents/file.txt"));
        assert!(is_file_path("C:\\"));
    }

    #[test]
    fn test_is_file_path_negative() {
        assert!(!is_file_path("not a path"));
        assert!(!is_file_path("file.txt")); // Just filename
        assert!(!is_file_path("Documents/file.txt")); // Missing prefix
        assert!(!is_file_path(""));
    }

    // ==================== Math Expression Detection Tests ====================

    #[test]
    fn test_is_math_expression_basic() {
        assert!(is_math_expression("2+2"));
        assert!(is_math_expression("10 - 5"));
        assert!(is_math_expression("100*50"));
        assert!(is_math_expression("100 / 4"));
    }

    #[test]
    fn test_is_math_expression_complex() {
        assert!(is_math_expression("(10 + 5) / 3"));
        assert!(is_math_expression("2^8"));
        assert!(is_math_expression("100 % 7"));
        assert!(is_math_expression("3.14 * 2"));
    }

    #[test]
    fn test_is_math_expression_negative() {
        assert!(!is_math_expression("42")); // Just a number, no operator
        assert!(!is_math_expression("hello"));
        assert!(!is_math_expression("2 + x")); // Contains variable
        assert!(!is_math_expression("")); // Empty
        assert!(!is_math_expression("+")); // Just operator
    }

    #[test]
    fn test_is_math_expression_with_spaces() {
        assert!(is_math_expression("  1 + 2  "));
        assert!(is_math_expression("10   *   5"));
    }

    // ==================== Code Snippet Detection Tests ====================

    #[test]
    fn test_is_code_snippet_javascript() {
        assert!(is_code_snippet("function foo() {}"));
        assert!(is_code_snippet("const x = 5"));
        assert!(is_code_snippet("let name = 'John'"));
        assert!(is_code_snippet("var old = true"));
    }

    #[test]
    fn test_is_code_snippet_es_modules() {
        assert!(is_code_snippet("import fs from 'fs'"));
        assert!(is_code_snippet("export default {}"));
        assert!(is_code_snippet("export const API = 'url'"));
    }

    #[test]
    fn test_is_code_snippet_arrow_functions() {
        assert!(is_code_snippet("const fn = () => {}"));
        assert!(is_code_snippet("x => x * 2"));
    }

    #[test]
    fn test_is_code_snippet_other_languages() {
        assert!(is_code_snippet("def python_function():"));
        assert!(is_code_snippet("fn rust_function() {}"));
        assert!(is_code_snippet("pub fn public_rust() {}"));
        assert!(is_code_snippet("class MyClass {}"));
    }

    #[test]
    fn test_is_code_snippet_control_flow() {
        assert!(is_code_snippet("if (x > 5) {}"));
        assert!(is_code_snippet("for (let i = 0; i < 10; i++)"));
        assert!(is_code_snippet("while (running) {}"));
    }

    #[test]
    fn test_is_code_snippet_function_calls() {
        assert!(is_code_snippet("console.log()"));
        assert!(is_code_snippet("myFunction(arg1, arg2)"));
        assert!(is_code_snippet("fetch('url')"));
    }

    #[test]
    fn test_is_code_snippet_negative() {
        assert!(!is_code_snippet("hello world"));
        assert!(!is_code_snippet("just some text"));
        assert!(!is_code_snippet("2 + 2")); // Math, not code
        assert!(!is_code_snippet(""));
    }

    // ==================== Input Type Detection Tests ====================

    #[test]
    fn test_detect_input_type_url() {
        assert_eq!(detect_input_type("https://example.com"), InputType::Url);
        assert_eq!(
            detect_input_type("http://localhost:3000/api"),
            InputType::Url
        );
    }

    #[test]
    fn test_detect_input_type_file_path() {
        assert_eq!(detect_input_type("/path/to/file"), InputType::FilePath);
        assert_eq!(detect_input_type("~/Documents"), InputType::FilePath);
        assert_eq!(detect_input_type("C:\\Users"), InputType::FilePath);
    }

    #[test]
    fn test_detect_input_type_math() {
        assert_eq!(detect_input_type("2+2"), InputType::MathExpression);
        assert_eq!(detect_input_type("(10 + 5) / 3"), InputType::MathExpression);
    }

    #[test]
    fn test_detect_input_type_code() {
        assert_eq!(detect_input_type("const x = 5"), InputType::CodeSnippet);
        assert_eq!(
            detect_input_type("function test() {}"),
            InputType::CodeSnippet
        );
    }

    #[test]
    fn test_detect_input_type_plain() {
        assert_eq!(detect_input_type("hello world"), InputType::PlainText);
        assert_eq!(detect_input_type(""), InputType::PlainText);
        assert_eq!(detect_input_type("   "), InputType::PlainText);
    }

    #[test]
    fn test_detect_input_type_priority() {
        // URL takes priority over file path even if it looks like a path
        assert_eq!(detect_input_type("file:///path/to/file"), InputType::Url);
    }
}
