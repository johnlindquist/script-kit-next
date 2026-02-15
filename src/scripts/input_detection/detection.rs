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

static CODE_FUNC_CALL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\w+\s*\([^)]*\)").expect("Invalid code function call regex"));

static CODE_IDENT_CALL_PREFIX_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[a-zA-Z_]\w*\s*\(").expect("Invalid code identifier call prefix regex")
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

/// Check if the input looks like a directory path
///
/// This is a more lenient check than `is_file_path` - it recognizes:
/// - `~` or `~/...` (home directory paths)
/// - `/...` (absolute paths)
/// - `.` or `./...` (current directory relative paths)
/// - `..` or `../...` (parent directory relative paths)
///
/// Unlike `is_file_path`, this also matches:
/// - `~` alone (home directory)
/// - `~/dev` without trailing slash
/// - `.` or `..` alone
/// - `../foo` without trailing slash
pub fn is_directory_path(input: &str) -> bool {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return false;
    }

    // Home directory path (~ or ~/...)
    if trimmed == "~" || trimmed.starts_with("~/") {
        return true;
    }

    // Unix-style absolute path
    if trimmed.starts_with('/') {
        return true;
    }

    // Current directory (. or ./...)
    if trimmed == "." || trimmed.starts_with("./") {
        return true;
    }

    // Parent directory (.. or ../...)
    if trimmed == ".." || trimmed.starts_with("../") {
        return true;
    }

    false
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
    if CODE_FUNC_CALL_REGEX.is_match(trimmed) {
        // But exclude math expressions like (1+2)
        // Only match if there's a word before the paren
        if CODE_IDENT_CALL_PREFIX_REGEX.is_match(trimmed) {
            return true;
        }
    }

    false
}
