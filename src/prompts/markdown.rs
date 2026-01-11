//! Markdown rendering for chat messages
//!
//! Simple markdown parser that renders to GPUI elements.
//! Supports: bold, italic, code, code blocks, links, lists, blockquotes.

use gpui::{div, prelude::*, px, rgb, rgba, IntoElement};

use crate::theme::PromptColors;

/// Render markdown text to GPUI elements
pub fn render_markdown(text: &str, colors: &PromptColors) -> impl IntoElement {
    let mut container = div().flex().flex_col().gap(px(6.0));
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Code block
        if line.starts_with("```") {
            let lang = line.trim_start_matches('`').trim();
            let mut code_lines = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].starts_with("```") {
                code_lines.push(lines[i]);
                i += 1;
            }
            let code = code_lines.join("\n");
            container = container.child(render_code_block(&code, lang, colors));
        }
        // Blockquote
        else if line.starts_with("> ") {
            let quote_text = line.trim_start_matches("> ");
            container = container.child(
                div()
                    .w_full()
                    .pl(px(12.0))
                    .border_l_2()
                    .border_color(rgb(colors.quote_border))
                    .text_color(rgb(colors.text_secondary))
                    .italic()
                    .child(quote_text.to_string()),
            );
        }
        // Headings (check from most specific to least specific)
        else if let Some(heading) = line.strip_prefix("### ") {
            container = container.child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(colors.text_primary))
                    .child(heading.to_string()),
            );
        } else if let Some(heading) = line.strip_prefix("## ") {
            container = container.child(
                div()
                    .text_base()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(colors.text_primary))
                    .child(heading.to_string()),
            );
        } else if let Some(heading) = line.strip_prefix("# ") {
            container = container.child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(colors.text_primary))
                    .child(heading.to_string()),
            );
        }
        // Bullet list
        else if line.starts_with("- ") || line.starts_with("* ") {
            container = container.child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(6.0))
                    .child(div().text_color(rgb(colors.text_tertiary)).child("•"))
                    .child(render_inline(line[2..].trim(), colors)),
            );
        }
        // Numbered list (1. item)
        else if let Some(rest) = parse_numbered(line) {
            let num = line
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>();
            container = container.child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_color(rgb(colors.text_tertiary))
                            .child(format!("{}.", num)),
                    )
                    .child(render_inline(rest, colors)),
            );
        }
        // Regular paragraph
        else if !line.trim().is_empty() {
            container = container.child(render_inline(line, colors));
        }

        i += 1;
    }

    container
}

fn parse_numbered(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let num_end = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
    if num_end > 0 && trimmed[num_end..].starts_with(". ") {
        Some(&trimmed[num_end + 2..])
    } else {
        None
    }
}

fn render_code_block(code: &str, lang: &str, colors: &PromptColors) -> impl IntoElement {
    div()
        .w_full()
        .mt(px(4.0))
        .mb(px(4.0))
        .rounded(px(6.0))
        .bg(rgba((colors.code_bg << 8) | 0xE0))
        .border_1()
        .border_color(rgba((colors.quote_border << 8) | 0x40))
        .flex()
        .flex_col()
        .overflow_hidden()
        .when(!lang.is_empty(), |d| {
            d.child(
                div()
                    .w_full()
                    .px(px(10.0))
                    .py(px(4.0))
                    .border_b_1()
                    .border_color(rgba((colors.quote_border << 8) | 0x30))
                    .text_xs()
                    .text_color(rgb(colors.text_tertiary))
                    .child(lang.to_string()),
            )
        })
        .child(
            div()
                .w_full()
                .px(px(10.0))
                .py(px(8.0))
                .text_sm()
                .text_color(rgb(colors.text_primary))
                .child(code.trim().to_string()),
        )
}

/// Render inline text with bold, italic, code, and links
fn render_inline(text: &str, colors: &PromptColors) -> impl IntoElement {
    let mut row = div().flex().flex_row().flex_wrap().text_sm();
    let mut chars = text.chars().peekable();
    let mut current = String::new();

    while let Some(c) = chars.next() {
        match c {
            // Bold: **text**
            '*' if chars.peek() == Some(&'*') => {
                row = flush_text(row, &mut current, colors);
                chars.next();
                let bold = collect_until(&mut chars, |c, next| c == '*' && next == Some(&'*'));
                if chars.peek() == Some(&'*') {
                    chars.next();
                }
                row = row.child(
                    div()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(rgb(colors.text_primary))
                        .child(bold),
                );
            }
            // Italic: *text* or _text_
            '*' | '_' => {
                let delim = c;
                row = flush_text(row, &mut current, colors);
                let italic = collect_until(&mut chars, |c, _| c == delim);
                row = row.child(
                    div()
                        .italic()
                        .text_color(rgb(colors.text_primary))
                        .child(italic),
                );
            }
            // Inline code: `code`
            '`' => {
                row = flush_text(row, &mut current, colors);
                let code = collect_until(&mut chars, |c, _| c == '`');
                row = row.child(
                    div()
                        .px(px(4.0))
                        .py(px(1.0))
                        .bg(rgba((colors.code_bg << 8) | 0x80))
                        .rounded(px(3.0))
                        .text_color(rgb(colors.text_primary))
                        .child(code),
                );
            }
            // Link: [text](url)
            '[' => {
                row = flush_text(row, &mut current, colors);
                let link_text = collect_until(&mut chars, |c, _| c == ']');
                if chars.peek() == Some(&'(') {
                    chars.next();
                    let _url = collect_until(&mut chars, |c, _| c == ')');
                    row = row.child(
                        div()
                            .text_color(rgb(colors.accent_color))
                            .cursor_pointer()
                            .child(link_text),
                    );
                } else {
                    current.push('[');
                    current.push_str(&link_text);
                    current.push(']');
                }
            }
            _ => current.push(c),
        }
    }

    flush_text(row, &mut current, colors)
}

fn flush_text(row: gpui::Div, current: &mut String, colors: &PromptColors) -> gpui::Div {
    if current.is_empty() {
        return row;
    }
    let text = std::mem::take(current);
    row.child(div().text_color(rgb(colors.text_primary)).child(text))
}

fn collect_until<F>(chars: &mut std::iter::Peekable<std::str::Chars>, end: F) -> String
where
    F: Fn(char, Option<&char>) -> bool,
{
    let mut result = String::new();
    while let Some(&c) = chars.peek() {
        if end(c, chars.clone().nth(1).as_ref()) {
            chars.next();
            break;
        }
        result.push(chars.next().unwrap());
    }
    result
}
