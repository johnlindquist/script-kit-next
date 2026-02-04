//! Markdown rendering for chat messages
//!
//! Uses pulldown-cmark for parsing and syntect for fenced code highlighting.
//! Supports: headings, lists, blockquotes, bold/italic, inline code, code blocks, links.

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight, IntoElement};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::notes::code_highlight::{highlight_code_lines, CodeLine, CodeSpan};
use crate::theme::PromptColors;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct InlineStyle {
    bold: bool,
    italic: bool,
    code: bool,
    link: bool,
}

#[derive(Clone, Debug)]
struct InlineSpan {
    text: String,
    style: InlineStyle,
}

#[derive(Debug)]
struct ListState {
    ordered: bool,
    start: usize,
    items: Vec<Vec<InlineSpan>>,
}

#[derive(Debug)]
struct CodeBlockState {
    language: Option<String>,
    code: String,
}

/// Render markdown text to GPUI elements.
pub fn render_markdown(text: &str, colors: &PromptColors) -> impl IntoElement {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(text, options);

    let mut blocks: Vec<AnyElement> = Vec::new();
    let mut spans: Vec<InlineSpan> = Vec::new();
    let mut style_stack: Vec<InlineStyle> = vec![InlineStyle::default()];
    let mut heading_level: Option<u32> = None;
    let mut list_state: Option<ListState> = None;
    let mut current_item: Option<Vec<InlineSpan>> = None;
    let mut quote_depth: usize = 0;
    let mut code_block: Option<CodeBlockState> = None;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => spans.clear(),
                Tag::Heading { level, .. } => {
                    heading_level = Some(heading_level_to_u32(level));
                    spans.clear();
                }
                Tag::List(start) => {
                    list_state = Some(ListState {
                        ordered: start.is_some(),
                        start: start.unwrap_or(1) as usize,
                        items: Vec::new(),
                    });
                }
                Tag::Item => {
                    current_item = Some(Vec::new());
                    spans.clear();
                }
                Tag::BlockQuote(_) => {
                    quote_depth += 1;
                }
                Tag::Emphasis => push_style(&mut style_stack, |style| style.italic = true),
                Tag::Strong => push_style(&mut style_stack, |style| style.bold = true),
                Tag::Link { .. } => push_style(&mut style_stack, |style| style.link = true),
                Tag::CodeBlock(kind) => {
                    code_block = Some(CodeBlockState {
                        language: code_block_language(&kind),
                        code: String::new(),
                    });
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => {
                    flush_paragraph(
                        &mut blocks,
                        &mut spans,
                        &mut current_item,
                        quote_depth,
                        colors,
                    );
                }
                TagEnd::Heading(_) => {
                    flush_heading(
                        &mut blocks,
                        &mut spans,
                        heading_level.take(),
                        quote_depth,
                        colors,
                    );
                }
                TagEnd::Item => {
                    if let Some(mut item_spans) = current_item.take() {
                        if !spans.is_empty() {
                            item_spans.append(&mut spans);
                        }
                        if let Some(list) = list_state.as_mut() {
                            list.items.push(item_spans);
                        }
                    }
                }
                TagEnd::List(_) => {
                    if let Some(list) = list_state.take() {
                        for (index, item) in list.items.iter().enumerate() {
                            let marker = if list.ordered {
                                format!("{}.", list.start + index)
                            } else {
                                "•".to_string()
                            };
                            let item_text: String = item.iter().map(|s| s.text.as_str()).collect();
                            let row = div()
                                .flex()
                                .flex_row()
                                .w_full()
                                .gap(px(6.0))
                                .text_sm()
                                .child(
                                    div()
                                        .flex_shrink_0()
                                        .text_color(rgb(colors.text_tertiary))
                                        .child(marker),
                                )
                                .child(
                                    div()
                                        .min_w_0()
                                        .text_color(rgb(colors.text_primary))
                                        .child(item_text),
                                );
                            push_block(&mut blocks, row, quote_depth, colors);
                        }
                    }
                }
                TagEnd::BlockQuote(_) => {
                    quote_depth = quote_depth.saturating_sub(1);
                }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Link => {
                    pop_style(&mut style_stack);
                }
                TagEnd::CodeBlock => {
                    if let Some(block) = code_block.take() {
                        let element =
                            render_code_block(&block.code, block.language.as_deref(), colors);
                        push_block(&mut blocks, element, quote_depth, colors);
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if let Some(block) = code_block.as_mut() {
                    block.code.push_str(&text);
                } else {
                    let style = *style_stack.last().unwrap_or(&InlineStyle::default());
                    push_text_span(&mut spans, &text, style);
                }
            }
            Event::Code(code) => {
                let mut style = *style_stack.last().unwrap_or(&InlineStyle::default());
                style.code = true;
                push_text_span(&mut spans, &code, style);
            }
            Event::SoftBreak | Event::HardBreak => {
                let style = *style_stack.last().unwrap_or(&InlineStyle::default());
                push_text_span(&mut spans, " ", style);
            }
            Event::Rule => {
                push_block(&mut blocks, render_hr(colors), quote_depth, colors);
            }
            Event::Html(html) => {
                let style = *style_stack.last().unwrap_or(&InlineStyle::default());
                push_text_span(&mut spans, &html, style);
            }
            _ => {}
        }
    }

    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .w_full()
        .min_w_0()
        .children(blocks)
}

fn heading_level_to_u32(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn code_block_language(kind: &CodeBlockKind) -> Option<String> {
    match kind {
        CodeBlockKind::Fenced(lang) => {
            let trimmed = lang.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        CodeBlockKind::Indented => None,
    }
}

fn push_style(stack: &mut Vec<InlineStyle>, update: impl FnOnce(&mut InlineStyle)) {
    let mut next = *stack.last().unwrap_or(&InlineStyle::default());
    update(&mut next);
    stack.push(next);
}

fn pop_style(stack: &mut Vec<InlineStyle>) {
    if stack.len() > 1 {
        stack.pop();
    }
}

fn push_text_span(spans: &mut Vec<InlineSpan>, text: &str, style: InlineStyle) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = spans.last_mut() {
        if last.style == style {
            last.text.push_str(text);
            return;
        }
    }
    spans.push(InlineSpan {
        text: text.to_string(),
        style,
    });
}

fn flush_paragraph(
    blocks: &mut Vec<AnyElement>,
    spans: &mut Vec<InlineSpan>,
    current_item: &mut Option<Vec<InlineSpan>>,
    quote_depth: usize,
    colors: &PromptColors,
) {
    if spans.is_empty() {
        return;
    }

    if let Some(item_spans) = current_item.as_mut() {
        item_spans.append(spans);
        return;
    }

    let element = render_inline_spans(spans, colors).w_full();
    spans.clear();
    push_block(blocks, element, quote_depth, colors);
}

fn flush_heading(
    blocks: &mut Vec<AnyElement>,
    spans: &mut Vec<InlineSpan>,
    level: Option<u32>,
    quote_depth: usize,
    colors: &PromptColors,
) {
    if spans.is_empty() {
        return;
    }

    let level = level.unwrap_or(3);
    let mut heading = render_inline_spans(spans, colors)
        .w_full()
        .text_color(rgb(colors.text_primary));
    heading = match level {
        1 => heading.text_lg().font_weight(FontWeight::BOLD),
        2 => heading.text_base().font_weight(FontWeight::SEMIBOLD),
        3 => heading.text_sm().font_weight(FontWeight::SEMIBOLD),
        _ => heading.text_sm().font_weight(FontWeight::MEDIUM),
    };

    spans.clear();
    push_block(blocks, heading, quote_depth, colors);
}

fn push_block(
    blocks: &mut Vec<AnyElement>,
    element: impl IntoElement,
    quote_depth: usize,
    colors: &PromptColors,
) {
    let mut element = element.into_any_element();
    if quote_depth > 0 {
        element = div()
            .w_full()
            .pl(px(12.0))
            .border_l_2()
            .border_color(rgb(colors.quote_border))
            .child(element)
            .into_any_element();
    }
    blocks.push(element);
}

fn style_span(text: &str, style: &InlineStyle, colors: &PromptColors) -> gpui::Div {
    if style.code {
        return div()
            .px(px(4.0))
            .py(px(1.0))
            .bg(rgba((colors.code_bg << 8) | 0x80))
            .rounded(px(3.0))
            .font_family("Menlo")
            .text_color(rgb(colors.text_primary))
            .child(text.to_string());
    }
    let mut piece = div()
        .text_color(rgb(colors.text_primary))
        .child(text.to_string());
    if style.bold {
        piece = piece.font_weight(FontWeight::BOLD);
    }
    if style.italic {
        piece = piece.italic();
    }
    if style.link {
        piece = piece.text_color(rgb(colors.accent_color));
    }
    piece
}

fn render_inline_spans(spans: &[InlineSpan], colors: &PromptColors) -> gpui::Div {
    // Fast path: single plain-text span — render as simple text child.
    // Avoids flex_wrap entirely so text wraps naturally at word boundaries.
    if spans.len() == 1 && spans[0].style == InlineStyle::default() {
        return div()
            .text_sm()
            .text_color(rgb(colors.text_primary))
            .child(spans[0].text.clone());
    }

    // Mixed styles: split non-code text into word-level flex children.
    // Each word is a separate flex item so flex_wrap breaks at word boundaries
    // instead of character boundaries (which happens when a long text span has
    // min_w_0 allowing it to shrink to width 0).
    let mut row = div().flex().flex_row().flex_wrap().min_w_0().text_sm();

    for span in spans {
        if span.style.code {
            // Code spans stay as single units (they have bg/padding)
            row = row.child(style_span(&span.text, &span.style, colors));
        } else {
            // Split text at whitespace for natural word wrapping
            for word in span.text.split_inclusive(char::is_whitespace) {
                if !word.is_empty() {
                    row = row.child(style_span(word, &span.style, colors));
                }
            }
        }
    }

    row
}

fn render_hr(colors: &PromptColors) -> gpui::Div {
    div()
        .w_full()
        .h(px(1.0))
        .my(px(8.0))
        .bg(rgb(colors.hr_color))
}

fn render_code_block(code: &str, lang: Option<&str>, colors: &PromptColors) -> gpui::Div {
    let lang_label = lang.unwrap_or("").trim();
    let lines: Vec<CodeLine> = highlight_code_lines(code, lang, colors.is_dark);

    let mut code_container = div()
        .w_full()
        .mt(px(4.0))
        .mb(px(4.0))
        .rounded(px(6.0))
        .bg(rgba((colors.code_bg << 8) | 0xE0))
        .border_1()
        .border_color(rgba((colors.quote_border << 8) | 0x40))
        .flex()
        .flex_col()
        .overflow_hidden();

    if !lang_label.is_empty() {
        code_container = code_container.child(
            div()
                .w_full()
                .px(px(10.0))
                .py(px(4.0))
                .border_b_1()
                .border_color(rgba((colors.quote_border << 8) | 0x30))
                .text_xs()
                .text_color(rgb(colors.text_tertiary))
                .child(lang_label.to_string()),
        );
    }

    let mut body = div()
        .w_full()
        .px(px(10.0))
        .py(px(8.0))
        .flex()
        .flex_col()
        .gap(px(2.0));

    for line in lines {
        let mut line_div = div()
            .flex()
            .flex_row()
            .w_full()
            .font_family("Menlo")
            .text_sm()
            .min_h(px(16.0));

        if line.spans.is_empty() {
            line_div = line_div.child(" ");
        } else {
            for CodeSpan { text, color } in line.spans {
                line_div = line_div.child(div().text_color(rgb(color)).child(text));
            }
        }

        body = body.child(line_div);
    }

    code_container.child(body)
}

// ---------------------------------------------------------------------------
// Test-only: parse markdown into a list of block descriptions so we can
// verify structure without instantiating GPUI elements.
// ---------------------------------------------------------------------------

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum TestBlock {
    /// A paragraph with concatenated text content.
    Paragraph(String),
    /// A heading with level and concatenated text.
    Heading(u32, String),
    /// A single list item: (marker, text).
    ListItem(String, String),
    /// A code block: (language, code).
    CodeBlock(Option<String>, String),
    /// A horizontal rule.
    Hr,
}

/// Parse markdown into structural blocks for testing. Mirrors the logic in
/// `render_markdown` but produces `TestBlock` values instead of GPUI elements.
#[cfg(test)]
fn parse_markdown_blocks(text: &str) -> Vec<TestBlock> {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(text, options);

    let mut blocks: Vec<TestBlock> = Vec::new();
    let mut spans: Vec<InlineSpan> = Vec::new();
    let mut style_stack: Vec<InlineStyle> = vec![InlineStyle::default()];
    let mut heading_level: Option<u32> = None;
    let mut list_state: Option<ListState> = None;
    let mut current_item: Option<Vec<InlineSpan>> = None;
    let mut code_block: Option<CodeBlockState> = None;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => spans.clear(),
                Tag::Heading { level, .. } => {
                    heading_level = Some(heading_level_to_u32(level));
                    spans.clear();
                }
                Tag::List(start) => {
                    list_state = Some(ListState {
                        ordered: start.is_some(),
                        start: start.unwrap_or(1) as usize,
                        items: Vec::new(),
                    });
                }
                Tag::Item => {
                    current_item = Some(Vec::new());
                    spans.clear();
                }
                Tag::Emphasis => push_style(&mut style_stack, |s| s.italic = true),
                Tag::Strong => push_style(&mut style_stack, |s| s.bold = true),
                Tag::Link { .. } => push_style(&mut style_stack, |s| s.link = true),
                Tag::CodeBlock(kind) => {
                    code_block = Some(CodeBlockState {
                        language: code_block_language(&kind),
                        code: String::new(),
                    });
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => {
                    if !spans.is_empty() {
                        if let Some(item_spans) = current_item.as_mut() {
                            item_spans.append(&mut spans);
                        } else {
                            let text: String = spans.iter().map(|s| s.text.as_str()).collect();
                            blocks.push(TestBlock::Paragraph(text));
                            spans.clear();
                        }
                    }
                }
                TagEnd::Heading(_) => {
                    if !spans.is_empty() {
                        let level = heading_level.take().unwrap_or(3);
                        let text: String = spans.iter().map(|s| s.text.as_str()).collect();
                        blocks.push(TestBlock::Heading(level, text));
                        spans.clear();
                    }
                }
                TagEnd::Item => {
                    if let Some(mut item_spans) = current_item.take() {
                        if !spans.is_empty() {
                            item_spans.append(&mut spans);
                        }
                        if let Some(list) = list_state.as_mut() {
                            list.items.push(item_spans);
                        }
                    }
                }
                TagEnd::List(_) => {
                    if let Some(list) = list_state.take() {
                        for (index, item) in list.items.iter().enumerate() {
                            let marker = if list.ordered {
                                format!("{}.", list.start + index)
                            } else {
                                "•".to_string()
                            };
                            let item_text: String = item.iter().map(|s| s.text.as_str()).collect();
                            blocks.push(TestBlock::ListItem(marker, item_text));
                        }
                    }
                }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Link => {
                    pop_style(&mut style_stack);
                }
                TagEnd::CodeBlock => {
                    if let Some(block) = code_block.take() {
                        blocks.push(TestBlock::CodeBlock(block.language, block.code));
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if let Some(block) = code_block.as_mut() {
                    block.code.push_str(&text);
                } else {
                    let style = *style_stack.last().unwrap_or(&InlineStyle::default());
                    push_text_span(&mut spans, &text, style);
                }
            }
            Event::Code(code) => {
                let mut style = *style_stack.last().unwrap_or(&InlineStyle::default());
                style.code = true;
                push_text_span(&mut spans, &code, style);
            }
            Event::SoftBreak | Event::HardBreak => {
                let style = *style_stack.last().unwrap_or(&InlineStyle::default());
                push_text_span(&mut spans, " ", style);
            }
            Event::Rule => {
                blocks.push(TestBlock::Hr);
            }
            _ => {}
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unordered_list_produces_separate_items() {
        let md = "- First item\n- Second item\n- Third item\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(
            blocks,
            vec![
                TestBlock::ListItem("•".into(), "First item".into()),
                TestBlock::ListItem("•".into(), "Second item".into()),
                TestBlock::ListItem("•".into(), "Third item".into()),
            ]
        );
    }

    #[test]
    fn ordered_list_produces_numbered_items() {
        let md = "1. Alpha\n2. Beta\n3. Gamma\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(
            blocks,
            vec![
                TestBlock::ListItem("1.".into(), "Alpha".into()),
                TestBlock::ListItem("2.".into(), "Beta".into()),
                TestBlock::ListItem("3.".into(), "Gamma".into()),
            ]
        );
    }

    #[test]
    fn paragraph_after_list_is_separate_block() {
        let md = "- Item one\n- Item two\n\nParagraph after the list.\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(
            blocks,
            vec![
                TestBlock::ListItem("•".into(), "Item one".into()),
                TestBlock::ListItem("•".into(), "Item two".into()),
                TestBlock::Paragraph("Paragraph after the list.".into()),
            ]
        );
    }

    #[test]
    fn heading_then_list_then_paragraph() {
        let md = "## My Heading\n\n- Item A\n- Item B\n\nSome text.\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(
            blocks,
            vec![
                TestBlock::Heading(2, "My Heading".into()),
                TestBlock::ListItem("•".into(), "Item A".into()),
                TestBlock::ListItem("•".into(), "Item B".into()),
                TestBlock::Paragraph("Some text.".into()),
            ]
        );
    }

    #[test]
    fn list_with_bold_and_inline_code() {
        let md = "- **Bold** item\n- Item with `code`\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(
            blocks,
            vec![
                TestBlock::ListItem("•".into(), "Bold item".into()),
                TestBlock::ListItem("•".into(), "Item with code".into()),
            ]
        );
    }

    #[test]
    fn code_block_after_list() {
        let md = "- Item\n\n```rust\nfn main() {}\n```\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(
            blocks,
            vec![
                TestBlock::ListItem("•".into(), "Item".into()),
                TestBlock::CodeBlock(Some("rust".into()), "fn main() {}\n".into()),
            ]
        );
    }

    /// Simulates progressive reveal of a markdown string containing a list.
    /// Each intermediate revealed substring should parse without panic and
    /// the final full string should produce the expected block structure.
    #[test]
    fn progressive_reveal_of_list_parses_at_every_boundary() {
        use crate::prompts::chat::chat_tests::next_reveal_boundary_pub;

        let content = "Here's a list:\n\n- First item\n- Second item\n- Third item\n\nDone!\n";
        let mut offset = 0;

        // Reveal word-by-word / line-by-line
        while let Some(new_offset) = next_reveal_boundary_pub(content, offset) {
            if new_offset <= offset {
                break;
            }
            let partial = &content[..new_offset];
            // Should not panic
            let _ = parse_markdown_blocks(partial);
            offset = new_offset;
        }

        // Final flush
        let blocks = parse_markdown_blocks(content);
        assert_eq!(
            blocks,
            vec![
                TestBlock::Paragraph("Here\u{2019}s a list:".into()),
                TestBlock::ListItem("•".into(), "First item".into()),
                TestBlock::ListItem("•".into(), "Second item".into()),
                TestBlock::ListItem("•".into(), "Third item".into()),
                TestBlock::Paragraph("Done!".into()),
            ]
        );
    }

    #[test]
    fn horizontal_rule_between_sections() {
        let md = "Before\n\n---\n\nAfter\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(
            blocks,
            vec![
                TestBlock::Paragraph("Before".into()),
                TestBlock::Hr,
                TestBlock::Paragraph("After".into()),
            ]
        );
    }

    #[test]
    fn empty_string_produces_no_blocks() {
        assert_eq!(parse_markdown_blocks(""), vec![]);
    }

    #[test]
    fn single_paragraph() {
        let blocks = parse_markdown_blocks("Hello world.\n");
        assert_eq!(blocks, vec![TestBlock::Paragraph("Hello world.".into())]);
    }
}
