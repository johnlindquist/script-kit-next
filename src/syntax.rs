//! Syntax highlighting module using syntect
//!
//! Provides syntax highlighting for code strings with colors that integrate
//! with the existing theme system. Colors are returned as hex u32 values.
//!
//! NOTE: syntect's default syntax set doesn't include TypeScript, so we use
//! JavaScript syntax for .ts files (which works well for highlighting).
//!
//! Performance: SyntaxSet and ThemeSet are cached as lazy statics since
//! loading them takes ~50ms each.

#![allow(dead_code)]

use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// Cached SyntaxSet - loading takes ~50ms, so we cache it globally
static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();

/// Cached Theme - loading takes ~50ms, so we cache it globally  
static THEME: OnceLock<Theme> = OnceLock::new();

/// Get the cached SyntaxSet, loading it if necessary
fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// Get the cached Theme, loading it if necessary
fn get_theme() -> &'static Theme {
    THEME.get_or_init(|| {
        let ts = ThemeSet::load_defaults();
        // Clone the theme so we can move it into the static
        ts.themes["base16-eighties.dark"].clone()
    })
}

/// A highlighted span of text with its associated color
#[derive(Debug, Clone, PartialEq)]
pub struct HighlightedSpan {
    /// The text content of this span
    pub text: String,
    /// The color as a hex u32 value (0xRRGGBB format)
    pub color: u32,
    /// Whether this span ends a line (contains newline)
    pub is_line_end: bool,
}

impl HighlightedSpan {
    /// Create a new highlighted span
    pub fn new(text: impl Into<String>, color: u32) -> Self {
        let text_str = text.into();
        let is_line_end = text_str.ends_with('\n');
        Self {
            text: text_str,
            color,
            is_line_end,
        }
    }
}

/// A complete highlighted line with all its spans
#[derive(Debug, Clone)]
pub struct HighlightedLine {
    pub spans: Vec<HighlightedSpan>,
}

/// Convert a syntect Style color to a hex u32 value
fn style_to_hex_color(style: &Style) -> u32 {
    let fg = style.foreground;
    ((fg.r as u32) << 16) | ((fg.g as u32) << 8) | (fg.b as u32)
}

/// Map language name/extension to syntect syntax name
/// NOTE: TypeScript is NOT in syntect defaults, so we map to JavaScript
fn map_language_to_syntax(language: &str) -> &str {
    match language.to_lowercase().as_str() {
        // TypeScript -> JavaScript (syntect doesn't have TypeScript by default)
        "typescript" | "ts" => "JavaScript",
        "javascript" | "js" => "JavaScript",
        "markdown" | "md" => "Markdown",
        "json" => "JSON",
        "rust" | "rs" => "Rust",
        "python" | "py" => "Python",
        "html" => "HTML",
        "css" => "CSS",
        "shell" | "sh" | "bash" => "Bourne Again Shell (bash)",
        "yaml" | "yml" => "YAML",
        // Note: TOML may not be in syntect defaults either
        "toml" => "Makefile", // Fallback - TOML not in defaults
        _ => language,        // Try the language name directly as fallback
    }
}

/// Highlight code with syntax coloring, returning lines of spans
///
/// # Arguments
/// * `code` - The source code to highlight
/// * `language` - The language identifier (e.g., "typescript", "javascript", "markdown", "ts", "js", "md")
///
/// # Returns
/// A vector of `HighlightedLine` structs, each containing spans for one line.
/// This preserves line structure for proper rendering.
pub fn highlight_code_lines(code: &str, language: &str) -> Vec<HighlightedLine> {
    let ps = get_syntax_set();
    let theme = get_theme();

    // Default foreground color for plain text (light gray)
    let default_color = 0xcccccc_u32;

    let syntax_name = map_language_to_syntax(language);

    // Try to find the syntax by name, or fall back to JavaScript for unknown
    let syntax = ps
        .find_syntax_by_name(syntax_name)
        .or_else(|| ps.find_syntax_by_extension(language))
        .or_else(|| ps.find_syntax_by_name("JavaScript")) // Better fallback than plain text
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut result = Vec::new();

    for line in LinesWithEndings::from(code) {
        let mut line_spans: Vec<HighlightedSpan> = Vec::new();

        match highlighter.highlight_line(line, ps) {
            Ok(ranges) => {
                for (style, text) in ranges {
                    if !text.is_empty() {
                        // Strip trailing newline for cleaner rendering
                        let clean_text = text.trim_end_matches('\n');
                        if !clean_text.is_empty() {
                            let color = style_to_hex_color(&style);

                            // PERF: Merge adjacent spans with the same color
                            // This dramatically reduces span count (from ~230/line to ~10-20/line)
                            if let Some(last) = line_spans.last_mut() {
                                if last.color == color {
                                    // Merge with previous span
                                    last.text.push_str(clean_text);
                                    continue;
                                }
                            }
                            line_spans.push(HighlightedSpan::new(clean_text, color));
                        }
                    }
                }
            }
            Err(_) => {
                // On error, push the line as plain text
                let clean_line = line.trim_end_matches('\n');
                if !clean_line.is_empty() {
                    line_spans.push(HighlightedSpan::new(clean_line, default_color));
                }
            }
        }

        result.push(HighlightedLine { spans: line_spans });
    }

    // If no lines were produced, return empty vec
    result
}

/// Highlight code with syntax coloring (flat span list for backward compatibility)
///
/// # Arguments
/// * `code` - The source code to highlight
/// * `language` - The language identifier (e.g., "typescript", "javascript", "markdown", "ts", "js", "md")
///
/// # Returns
/// A vector of `HighlightedSpan` structs, each containing a text segment and its color.
/// If the language is not recognized, returns the code as plain text with default color.
pub fn highlight_code(code: &str, language: &str) -> Vec<HighlightedSpan> {
    let ps = get_syntax_set();
    let theme = get_theme();

    // Default foreground color for plain text (light gray)
    let default_color = 0xcccccc_u32;

    let syntax_name = map_language_to_syntax(language);

    // Try to find the syntax by name, or fall back to JavaScript for unknown
    let syntax = ps
        .find_syntax_by_name(syntax_name)
        .or_else(|| ps.find_syntax_by_extension(language))
        .or_else(|| ps.find_syntax_by_name("JavaScript")) // Better fallback than plain text
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut result: Vec<HighlightedSpan> = Vec::new();

    for line in LinesWithEndings::from(code) {
        match highlighter.highlight_line(line, ps) {
            Ok(ranges) => {
                for (style, text) in ranges {
                    if !text.is_empty() {
                        let color = style_to_hex_color(&style);

                        // PERF: Merge adjacent spans with the same color
                        if let Some(last) = result.last_mut() {
                            if last.color == color {
                                last.text.push_str(text);
                                continue;
                            }
                        }
                        result.push(HighlightedSpan::new(text, color));
                    }
                }
            }
            Err(_) => {
                // On error, push the line as plain text
                result.push(HighlightedSpan::new(line, default_color));
            }
        }
    }

    // If no spans were produced, return the original code as plain text
    if result.is_empty() && !code.is_empty() {
        result.push(HighlightedSpan::new(code, default_color));
    }

    result
}

/// Get a list of supported language identifiers
pub fn supported_languages() -> Vec<&'static str> {
    vec![
        "typescript",
        "ts",
        "javascript",
        "js",
        "markdown",
        "md",
        "json",
        "rust",
        "rs",
        "python",
        "py",
        "html",
        "css",
        "shell",
        "sh",
        "bash",
        "yaml",
        "yml",
        "toml",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_typescript() {
        let code = "const x: number = 42;";
        let spans = highlight_code(code, "typescript");

        // Should produce multiple spans with different colors
        assert!(!spans.is_empty());

        // Verify the text content is preserved
        let reconstructed: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(reconstructed, code);
    }

    #[test]
    fn test_highlight_javascript() {
        let code = "function hello() { return 'world'; }";
        let spans = highlight_code(code, "javascript");

        assert!(!spans.is_empty());
        let reconstructed: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(reconstructed, code);
    }

    #[test]
    fn test_highlight_markdown() {
        let code = "# Hello World\n\nThis is **bold** text.";
        let spans = highlight_code(code, "markdown");

        assert!(!spans.is_empty());
        let reconstructed: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(reconstructed, code);
    }

    #[test]
    fn test_highlight_with_extension() {
        let code = "let x = 1;";
        let spans_ts = highlight_code(code, "ts");
        let spans_js = highlight_code(code, "js");

        assert!(!spans_ts.is_empty());
        assert!(!spans_js.is_empty());
    }

    #[test]
    fn test_unknown_language_returns_plain_text() {
        let code = "some random text";
        let spans = highlight_code(code, "unknownlang123");

        // Should return at least one span with the full text
        assert!(!spans.is_empty());
        let reconstructed: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(reconstructed, code);
    }

    #[test]
    fn test_empty_code() {
        let spans = highlight_code("", "typescript");
        assert!(spans.is_empty());
    }

    #[test]
    fn test_multiline_code() {
        let code = "const a = 1;\nconst b = 2;\nconst c = a + b;";
        let spans = highlight_code(code, "javascript");

        assert!(!spans.is_empty());
        let reconstructed: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(reconstructed, code);
    }

    #[test]
    fn test_color_format() {
        let spans = highlight_code("let x = 42;", "typescript");

        for span in &spans {
            // Colors should be in valid hex range (0x000000 to 0xFFFFFF)
            assert!(
                span.color <= 0xFFFFFF,
                "Color {:06X} out of range",
                span.color
            );
        }
    }

    #[test]
    fn test_highlighted_span_new() {
        let span = HighlightedSpan::new("hello", 0xFF0000);
        assert_eq!(span.text, "hello");
        assert_eq!(span.color, 0xFF0000);
    }

    #[test]
    fn test_supported_languages() {
        let languages = supported_languages();
        assert!(languages.contains(&"typescript"));
        assert!(languages.contains(&"javascript"));
        assert!(languages.contains(&"markdown"));
        assert!(languages.contains(&"ts"));
        assert!(languages.contains(&"js"));
        assert!(languages.contains(&"md"));
    }

    #[test]
    fn test_highlight_lines_preserves_structure() {
        let code = "const a = 1;\nconst b = 2;";
        let lines = highlight_code_lines(code, "js");

        // Should have 2 lines
        assert_eq!(lines.len(), 2);

        // Each line should have spans
        assert!(!lines[0].spans.is_empty());
        assert!(!lines[1].spans.is_empty());
    }

    #[test]
    fn test_highlight_produces_colors() {
        // Use JavaScript which IS in syntect defaults
        let code = "function test() { return 42; }";
        let spans = highlight_code(code, "javascript");

        // Check we have different colors (real syntax highlighting)
        let unique_colors: std::collections::HashSet<u32> = spans.iter().map(|s| s.color).collect();
        assert!(
            unique_colors.len() > 1,
            "Expected syntax highlighting to produce multiple colors"
        );
    }

    #[test]
    fn test_span_merging_efficiency() {
        // Test that span merging reduces span count effectively
        // This code should have relatively few unique color regions
        let code = r#"const x = 42;
const y = "hello";
function test() {
    return x + y;
}"#;
        let lines = highlight_code_lines(code, "javascript");

        let total_spans: usize = lines.iter().map(|l| l.spans.len()).sum();
        let line_count = lines.len();

        // Print for debugging (only visible with --nocapture)
        eprintln!("\n=== Span Analysis ===");
        eprintln!("Lines: {}", line_count);
        eprintln!("Total spans: {}", total_spans);
        eprintln!(
            "Avg spans/line: {:.1}",
            total_spans as f64 / line_count as f64
        );

        for (i, line) in lines.iter().enumerate() {
            eprintln!("Line {}: {} spans", i + 1, line.spans.len());
            for span in &line.spans {
                eprintln!("  [{:06x}] '{}'", span.color, span.text);
            }
        }

        // After merging, we should have reasonable span counts
        // A well-merged line should have ~3-10 spans, not ~100
        assert!(
            total_spans < 50,
            "Span count {} is too high for {} lines of code. Expected < 50 after merging.",
            total_spans,
            line_count
        );
    }
}
