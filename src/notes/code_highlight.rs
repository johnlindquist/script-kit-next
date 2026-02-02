//! Markdown code fence highlighting for Notes/Markdown preview.
//!
//! - Detects fenced code blocks (```language ... ```)
//! - Highlights code using syntect
//! - Returns styled spans for GPUI rendering

use std::sync::OnceLock;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// A highlighted span of code with its color.
#[derive(Debug, Clone, PartialEq)]
pub struct CodeSpan {
    pub text: String,
    pub color: u32,
}

/// A highlighted line of code.
#[derive(Debug, Clone)]
pub struct CodeLine {
    pub spans: Vec<CodeSpan>,
}

/// A raw fenced code block extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub code: String,
}

/// A highlighted code block with syntect spans.
#[derive(Debug, Clone)]
pub struct HighlightedCodeBlock {
    pub language: Option<String>,
    pub lines: Vec<CodeLine>,
}

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME: OnceLock<Theme> = OnceLock::new();

fn syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme() -> &'static Theme {
    THEME.get_or_init(|| {
        let themes = ThemeSet::load_defaults();
        themes
            .themes
            .get("base16-ocean.dark")
            .cloned()
            .unwrap_or_else(|| {
                themes
                    .themes
                    .values()
                    .next()
                    .cloned()
                    .expect("syntect ThemeSet should contain at least one theme")
            })
    })
}

fn style_to_hex_color(style: &Style) -> u32 {
    let fg = style.foreground;
    ((fg.r as u32) << 16) | ((fg.g as u32) << 8) | (fg.b as u32)
}

fn normalize_language(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let token = trimmed
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(|c| c == '{' || c == '}' || c == '.');
    if token.is_empty() {
        return None;
    }

    let token = token.strip_prefix("language-").unwrap_or(token);
    Some(token.to_lowercase())
}

fn map_language_to_syntax(language: &str) -> &str {
    match language {
        "rust" | "rs" => "Rust",
        "javascript" | "js" | "jsx" => "JavaScript",
        "typescript" | "ts" | "tsx" => "JavaScript", // TypeScript not in defaults
        "python" | "py" => "Python",
        "bash" | "sh" | "shell" | "zsh" => "Bourne Again Shell (bash)",
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "html" | "htm" => "HTML",
        "css" => "CSS",
        _ => language,
    }
}

/// Detect fenced code blocks in a markdown string.
pub fn detect_fenced_code_blocks(markdown: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current_lang: Option<String> = None;
    let mut current_code: Vec<String> = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("```") {
            if in_block {
                blocks.push(CodeBlock {
                    language: current_lang.take(),
                    code: current_code.join("\n"),
                });
                current_code.clear();
                in_block = false;
            } else {
                current_lang = normalize_language(rest);
                in_block = true;
            }
            continue;
        }

        if in_block {
            current_code.push(line.to_string());
        }
    }

    if in_block {
        blocks.push(CodeBlock {
            language: current_lang,
            code: current_code.join("\n"),
        });
    }

    blocks
}

/// Highlight a code string into line-based spans.
pub fn highlight_code_lines(code: &str, language: Option<&str>) -> Vec<CodeLine> {
    if code.is_empty() {
        return Vec::new();
    }

    let ps = syntax_set();
    let theme = theme();
    let default_color = 0xCCCCCC_u32;

    let language = language
        .and_then(normalize_language)
        .unwrap_or_else(|| "text".to_string());
    let syntax_name = map_language_to_syntax(&language);

    let syntax = ps
        .find_syntax_by_name(syntax_name)
        .or_else(|| ps.find_syntax_by_extension(&language))
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut lines = Vec::new();

    for line in LinesWithEndings::from(code) {
        let mut spans: Vec<CodeSpan> = Vec::new();

        match highlighter.highlight_line(line, ps) {
            Ok(ranges) => {
                for (style, text) in ranges {
                    if text.is_empty() {
                        continue;
                    }
                    let clean_text = text.trim_end_matches('\n');
                    if clean_text.is_empty() {
                        continue;
                    }
                    let color = style_to_hex_color(&style);
                    if let Some(last) = spans.last_mut() {
                        if last.color == color {
                            last.text.push_str(clean_text);
                            continue;
                        }
                    }
                    spans.push(CodeSpan {
                        text: clean_text.to_string(),
                        color,
                    });
                }
            }
            Err(_) => {
                let clean_line = line.trim_end_matches('\n');
                if !clean_line.is_empty() {
                    spans.push(CodeSpan {
                        text: clean_line.to_string(),
                        color: default_color,
                    });
                }
            }
        }

        lines.push(CodeLine { spans });
    }

    lines
}

/// Detect and highlight fenced code blocks in markdown.
pub fn highlight_fenced_code_blocks(markdown: &str) -> Vec<HighlightedCodeBlock> {
    detect_fenced_code_blocks(markdown)
        .into_iter()
        .map(|block| HighlightedCodeBlock {
            language: block.language.clone(),
            lines: highlight_code_lines(&block.code, block.language.as_deref()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_fenced_code_blocks() {
        let md = r#"
Text
```rust
fn main() {
  println!("hi");
}
```
More
```javascript
console.log("ok");
```
"#;
        let blocks = detect_fenced_code_blocks(md);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].language.as_deref(), Some("rust"));
        assert!(blocks[0].code.contains("fn main"));
        assert_eq!(blocks[1].language.as_deref(), Some("javascript"));
    }

    #[test]
    fn test_detect_fenced_code_blocks_without_language() {
        let md = "```\nplain\n```";
        let blocks = detect_fenced_code_blocks(md);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, None);
        assert_eq!(blocks[0].code, "plain");
    }

    #[test]
    fn test_highlight_code_lines_preserves_text() {
        let code = "const x = 1;\nconst y = 2;";
        let lines = highlight_code_lines(code, Some("javascript"));
        assert_eq!(lines.len(), 2);
        let reconstructed: String = lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|s| s.text.as_str())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(reconstructed, code);
    }

    #[test]
    fn test_highlight_fenced_code_blocks_returns_lines() {
        let md = "```python\nprint('hi')\n```";
        let blocks = highlight_fenced_code_blocks(md);
        assert_eq!(blocks.len(), 1);
        assert!(!blocks[0].lines.is_empty());
    }

    #[test]
    fn test_supports_common_language_aliases() {
        let code = "let x = 1;";
        let langs = [
            "rust",
            "rs",
            "javascript",
            "js",
            "typescript",
            "ts",
            "python",
            "py",
            "bash",
            "sh",
            "json",
            "yaml",
            "yml",
            "html",
            "css",
        ];
        for lang in langs {
            let lines = highlight_code_lines(code, Some(lang));
            assert!(!lines.is_empty(), "Expected lines for {}", lang);
        }
    }

    #[test]
    fn test_normalize_language_variants() {
        assert_eq!(normalize_language("rust"), Some("rust".to_string()));
        assert_eq!(
            normalize_language("{.typescript}"),
            Some("typescript".to_string())
        );
        assert_eq!(
            normalize_language("language-python"),
            Some("python".to_string())
        );
        assert_eq!(normalize_language(""), None);
    }
}
