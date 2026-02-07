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

// ==================== Directory Path Detection Tests ====================

#[test]
fn test_is_directory_path_home_with_trailing_slash() {
    assert!(is_directory_path("~/dev/"));
}

#[test]
fn test_is_directory_path_home_without_trailing_slash() {
    assert!(is_directory_path("~/dev"));
}

#[test]
fn test_is_directory_path_absolute() {
    assert!(is_directory_path("/usr/local/bin"));
}

#[test]
fn test_is_directory_path_relative_current_dir() {
    assert!(is_directory_path("./src/"));
}

#[test]
fn test_is_directory_path_relative_parent_dir() {
    assert!(is_directory_path("../foo"));
}

#[test]
fn test_is_directory_path_negative_search_term() {
    assert!(!is_directory_path("search term"));
}

#[test]
fn test_is_directory_path_negative_simple_word() {
    assert!(!is_directory_path("clipboard"));
}

#[test]
fn test_is_directory_path_just_tilde() {
    assert!(is_directory_path("~"));
}

#[test]
fn test_is_directory_path_empty() {
    assert!(!is_directory_path(""));
}

#[test]
fn test_is_directory_path_whitespace() {
    assert!(!is_directory_path("   "));
}

#[test]
fn test_is_directory_path_relative_dot_only() {
    assert!(is_directory_path("."));
}

#[test]
fn test_is_directory_path_relative_dotdot_only() {
    assert!(is_directory_path(".."));
}
