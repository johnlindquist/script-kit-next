//! Markdown preview rendering for Notes.
//!
//! Uses pulldown-cmark to parse markdown and renders to GPUI StyledText with highlights.

use gpui::{
    div, px, AnyElement, FontStyle, FontWeight, HighlightStyle, Hsla, IntoElement, ParentElement,
    Styled, StyledText, UnderlineStyle,
};
use gpui_component::theme::Theme;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::ops::Range;

use crate::notes::code_highlight::highlight_code_lines;

/// Convert hex color to HSLA
fn hex_to_hsla(hex: u32) -> Hsla {
    let color = gpui::rgb(hex);
    color.into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownText {
    pub text: String,
    pub spans: Vec<MarkdownSpan>,
}

impl MarkdownText {
    fn new() -> Self {
        Self {
            text: String::new(),
            spans: Vec::new(),
        }
    }

    fn push_text(&mut self, chunk: &str, style: SpanStyle) {
        if chunk.is_empty() {
            return;
        }
        let start = self.text.len();
        self.text.push_str(chunk);
        let end = self.text.len();
        if style.is_default() {
            return;
        }
        if let Some(last) = self.spans.last_mut() {
            if last.style == style && last.range.end == start {
                last.range.end = end;
                return;
            }
        }
        self.spans.push(MarkdownSpan {
            range: start..end,
            style,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownSpan {
    pub range: Range<usize>,
    pub style: SpanStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpanStyle {
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub link: Option<String>,
}

impl SpanStyle {
    fn is_default(&self) -> bool {
        !self.bold && !self.italic && !self.code && self.link.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownBlock {
    Paragraph(MarkdownText),
    Heading {
        level: u8,
        text: MarkdownText,
    },
    ListItem {
        ordered: bool,
        number: usize,
        indent: usize,
        text: MarkdownText,
    },
    CodeBlock {
        language: Option<String>,
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BlockKind {
    Paragraph,
    Heading(u8),
    ListItem {
        ordered: bool,
        number: usize,
        indent: usize,
    },
    CodeBlock {
        language: Option<String>,
    },
}

struct BlockBuilder {
    kind: BlockKind,
    text: MarkdownText,
    code_text: String,
}

impl BlockBuilder {
    fn new(kind: BlockKind) -> Self {
        Self {
            kind,
            text: MarkdownText::new(),
            code_text: String::new(),
        }
    }

    fn finish(self) -> Option<MarkdownBlock> {
        match self.kind {
            BlockKind::Paragraph => {
                if self.text.text.trim().is_empty() {
                    None
                } else {
                    Some(MarkdownBlock::Paragraph(self.text))
                }
            }
            BlockKind::Heading(level) => Some(MarkdownBlock::Heading {
                level,
                text: self.text,
            }),
            BlockKind::ListItem {
                ordered,
                number,
                indent,
            } => Some(MarkdownBlock::ListItem {
                ordered,
                number,
                indent,
                text: self.text,
            }),
            BlockKind::CodeBlock { language } => Some(MarkdownBlock::CodeBlock {
                language,
                text: self.code_text,
            }),
        }
    }
}

#[derive(Default)]
struct InlineState {
    bold_depth: usize,
    italic_depth: usize,
    link_stack: Vec<String>,
}

impl InlineState {
    fn current_style(&self) -> SpanStyle {
        SpanStyle {
            bold: self.bold_depth > 0,
            italic: self.italic_depth > 0,
            code: false,
            link: self.link_stack.last().cloned(),
        }
    }

    fn with_code(&self) -> SpanStyle {
        let mut style = self.current_style();
        style.code = true;
        style
    }
}

struct ListState {
    ordered: bool,
    next_number: usize,
}

fn flush_current(current: &mut Option<BlockBuilder>, blocks: &mut Vec<MarkdownBlock>) {
    if let Some(builder) = current.take() {
        if let Some(block) = builder.finish() {
            blocks.push(block);
        }
    }
}

/// Parse markdown into blocks with inline style spans using pulldown-cmark.
pub fn parse_markdown(markdown: &str) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let mut current: Option<BlockBuilder> = None;
    let mut inline = InlineState::default();
    let mut list_stack: Vec<ListState> = Vec::new();

    let mut options = Options::all();
    options.remove(Options::ENABLE_DEFINITION_LIST);

    let parser = Parser::new_ext(markdown, options);
    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    let in_list_item = current
                        .as_ref()
                        .is_some_and(|b| matches!(&b.kind, BlockKind::ListItem { .. }));
                    if !in_list_item {
                        flush_current(&mut current, &mut blocks);
                        current = Some(BlockBuilder::new(BlockKind::Paragraph));
                    }
                }
                Tag::Heading { level, .. } => {
                    flush_current(&mut current, &mut blocks);
                    current = Some(BlockBuilder::new(BlockKind::Heading(heading_level_to_u8(
                        level,
                    ))));
                }
                Tag::List(start) => {
                    let ordered = start.is_some();
                    let next_number = start.unwrap_or(1) as usize;
                    list_stack.push(ListState {
                        ordered,
                        next_number,
                    });
                }
                Tag::Item => {
                    let (ordered, number, indent) = list_stack
                        .last()
                        .map(|state| {
                            (
                                state.ordered,
                                state.next_number,
                                list_stack.len().saturating_sub(1),
                            )
                        })
                        .unwrap_or((false, 1, 0));
                    flush_current(&mut current, &mut blocks);
                    current = Some(BlockBuilder::new(BlockKind::ListItem {
                        ordered,
                        number,
                        indent,
                    }));
                }
                Tag::CodeBlock(kind) => {
                    flush_current(&mut current, &mut blocks);
                    let language = match kind {
                        CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.to_string()),
                        _ => None,
                    };
                    current = Some(BlockBuilder::new(BlockKind::CodeBlock { language }));
                }
                Tag::Strong => {
                    inline.bold_depth += 1;
                }
                Tag::Emphasis => {
                    inline.italic_depth += 1;
                }
                Tag::Link { dest_url, .. } => {
                    inline.link_stack.push(dest_url.to_string());
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Paragraph => {
                    let in_list_item = current
                        .as_ref()
                        .is_some_and(|b| matches!(&b.kind, BlockKind::ListItem { .. }));
                    if !in_list_item {
                        flush_current(&mut current, &mut blocks);
                    }
                }
                TagEnd::Heading(_) => {
                    flush_current(&mut current, &mut blocks);
                }
                TagEnd::Item => {
                    flush_current(&mut current, &mut blocks);
                    if let Some(state) = list_stack.last_mut() {
                        state.next_number += 1;
                    }
                }
                TagEnd::List(_) => {
                    list_stack.pop();
                }
                TagEnd::CodeBlock => {
                    flush_current(&mut current, &mut blocks);
                }
                TagEnd::Strong => {
                    inline.bold_depth = inline.bold_depth.saturating_sub(1);
                }
                TagEnd::Emphasis => {
                    inline.italic_depth = inline.italic_depth.saturating_sub(1);
                }
                TagEnd::Link => {
                    inline.link_stack.pop();
                }
                _ => {}
            },
            Event::Text(text) => {
                if current.is_none() {
                    current = Some(BlockBuilder::new(BlockKind::Paragraph));
                }
                if let Some(ref mut builder) = current {
                    match builder.kind {
                        BlockKind::CodeBlock { .. } => builder.code_text.push_str(&text),
                        _ => builder.text.push_text(&text, inline.current_style()),
                    }
                }
            }
            Event::Code(text) => {
                if current.is_none() {
                    current = Some(BlockBuilder::new(BlockKind::Paragraph));
                }
                if let Some(ref mut builder) = current {
                    builder.text.push_text(&text, inline.with_code());
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if current.is_none() {
                    current = Some(BlockBuilder::new(BlockKind::Paragraph));
                }
                if let Some(ref mut builder) = current {
                    match builder.kind {
                        BlockKind::CodeBlock { .. } => builder.code_text.push('\n'),
                        _ => builder.text.push_text("\n", inline.current_style()),
                    }
                }
            }
            _ => {}
        }
    }

    flush_current(&mut current, &mut blocks);
    blocks
}

fn heading_level_to_u8(level: pulldown_cmark::HeadingLevel) -> u8 {
    match level {
        pulldown_cmark::HeadingLevel::H1 => 1,
        pulldown_cmark::HeadingLevel::H2 => 2,
        pulldown_cmark::HeadingLevel::H3 => 3,
        pulldown_cmark::HeadingLevel::H4 => 4,
        pulldown_cmark::HeadingLevel::H5 => 5,
        pulldown_cmark::HeadingLevel::H6 => 6,
    }
}

struct RenderStyles {
    text: Hsla,
    muted: Hsla,
    link: Hsla,
    code_bg: Hsla,
    code_block_bg: Hsla,
    border: Hsla,
    mono_font: String,
}

impl RenderStyles {
    fn from_theme(theme: &Theme) -> Self {
        let code_bg = with_alpha(theme.muted, 0.28);
        let code_block_bg = with_alpha(theme.muted, 0.2);
        Self {
            text: theme.foreground,
            muted: theme.muted_foreground,
            link: theme.link,
            code_bg,
            code_block_bg,
            border: with_alpha(theme.border, 0.4),
            mono_font: theme.mono_font_family.to_string(),
        }
    }
}

fn with_alpha(mut color: Hsla, alpha: f32) -> Hsla {
    color.a = alpha;
    color
}

/// Render markdown to GPUI elements using StyledText + HighlightStyle.
pub fn render_markdown_preview(markdown: &str, theme: &Theme) -> impl IntoElement {
    let styles = RenderStyles::from_theme(theme);
    let blocks = parse_markdown(markdown);

    let mut container = div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .w_full()
        .min_w_0()
        .text_sm();

    for block in blocks {
        container = container.child(render_block(&block, &styles));
    }

    container
}

fn render_block(block: &MarkdownBlock, styles: &RenderStyles) -> AnyElement {
    match block {
        MarkdownBlock::Paragraph(text) => div()
            .text_color(styles.text)
            .child(styled_text(text, styles))
            .into_any_element(),
        MarkdownBlock::Heading { level, text } => {
            let base = div()
                .text_color(styles.text)
                .child(styled_text(text, styles));
            let sized = match level {
                1 => base.text_lg().font_weight(FontWeight::BOLD),
                2 => base.text_base().font_weight(FontWeight::BOLD),
                3 => base.text_sm().font_weight(FontWeight::SEMIBOLD),
                4 => base.text_sm().font_weight(FontWeight::SEMIBOLD),
                5 => base.text_sm().text_color(styles.muted),
                _ => base.text_xs().text_color(styles.muted),
            };
            sized.into_any_element()
        }
        MarkdownBlock::ListItem {
            ordered,
            number,
            indent,
            text,
        } => {
            let bullet = if *ordered {
                format!("{}.", number)
            } else {
                "-".to_string()
            };
            div()
                .flex()
                .flex_row()
                .items_start()
                .gap(px(8.0))
                .pl(px(12.0 * (*indent as f32)))
                .text_color(styles.text)
                .child(div().text_color(styles.muted).child(bullet))
                .child(div().flex_1().min_w_0().child(styled_text(text, styles)))
                .into_any_element()
        }
        MarkdownBlock::CodeBlock { language, text } => {
            let styled_code = styled_code_block(text, language.as_deref());
            div()
                .rounded(px(6.0))
                .bg(styles.code_block_bg)
                .border_1()
                .border_color(styles.border)
                .p(px(10.0))
                .font_family(styles.mono_font.clone())
                .text_color(styles.text)
                .child(styled_code)
                .into_any_element()
        }
    }
}

fn styled_code_block(code: &str, language: Option<&str>) -> StyledText {
    let trimmed = code.trim_end_matches('\n');
    if trimmed.is_empty() {
        return StyledText::new(String::new());
    }

    let lines = highlight_code_lines(trimmed, language);
    if lines.is_empty() {
        return StyledText::new(trimmed.to_string());
    }

    let mut text = String::new();
    let mut highlights: Vec<(Range<usize>, HighlightStyle)> = Vec::new();

    for (line_index, line) in lines.iter().enumerate() {
        for span in &line.spans {
            let start = text.len();
            text.push_str(&span.text);
            let end = text.len();
            if start < end {
                highlights.push((
                    start..end,
                    HighlightStyle {
                        color: Some(hex_to_hsla(span.color)),
                        ..Default::default()
                    },
                ));
            }
        }

        if line_index + 1 < lines.len() {
            text.push('\n');
        }
    }

    if highlights.is_empty() {
        StyledText::new(text)
    } else {
        StyledText::new(text).with_highlights(highlights)
    }
}

fn styled_text(text: &MarkdownText, styles: &RenderStyles) -> StyledText {
    if text.spans.is_empty() {
        return StyledText::new(text.text.clone());
    }

    let highlights = text
        .spans
        .iter()
        .map(|span| (span.range.clone(), highlight_for_span(&span.style, styles)));
    StyledText::new(text.text.clone()).with_highlights(highlights)
}

fn highlight_for_span(style: &SpanStyle, styles: &RenderStyles) -> HighlightStyle {
    let mut highlight = HighlightStyle::default();
    if style.bold {
        highlight.font_weight = Some(FontWeight::BOLD);
    }
    if style.italic {
        highlight.font_style = Some(FontStyle::Italic);
    }
    if style.code {
        highlight.background_color = Some(styles.code_bg);
        highlight.font_weight = Some(FontWeight::MEDIUM);
    }
    if style.link.is_some() {
        highlight.color = Some(styles.link);
        highlight.underline = Some(UnderlineStyle {
            thickness: px(1.0),
            ..Default::default()
        });
    }
    highlight
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_heading_block() {
        let blocks = parse_markdown("# Hello");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            MarkdownBlock::Heading { level, text } => {
                assert_eq!(*level, 1);
                assert_eq!(text.text, "Hello");
            }
            other => panic!("Expected heading block, got {other:?}"),
        }
    }

    #[test]
    fn parses_inline_styles() {
        let blocks = parse_markdown("Hello **bold** _italic_ `code` [link](https://example.com)");
        assert_eq!(blocks.len(), 1);
        let paragraph = match &blocks[0] {
            MarkdownBlock::Paragraph(text) => text,
            other => panic!("Expected paragraph, got {other:?}"),
        };

        assert_eq!(paragraph.text, "Hello bold italic code link");

        let mut found_bold = false;
        let mut found_italic = false;
        let mut found_code = false;
        let mut found_link = false;

        for span in &paragraph.spans {
            let slice = &paragraph.text[span.range.clone()];
            if span.style.bold {
                found_bold = slice == "bold";
            }
            if span.style.italic {
                found_italic = slice == "italic";
            }
            if span.style.code {
                found_code = slice == "code";
            }
            if span.style.link.is_some() {
                found_link = slice == "link";
            }
        }

        assert!(found_bold, "bold span missing");
        assert!(found_italic, "italic span missing");
        assert!(found_code, "code span missing");
        assert!(found_link, "link span missing");
    }

    #[test]
    fn parses_lists_and_code_blocks() {
        let markdown = "- one\n- two\n\n1. first\n2. second\n\n```\nlet x = 1;\n```";
        let blocks = parse_markdown(markdown);
        assert_eq!(blocks.len(), 5);

        match &blocks[0] {
            MarkdownBlock::ListItem { ordered, text, .. } => {
                assert!(!ordered);
                assert_eq!(text.text, "one");
            }
            other => panic!("Expected list item, got {other:?}"),
        }

        match &blocks[1] {
            MarkdownBlock::ListItem { ordered, text, .. } => {
                assert!(!ordered);
                assert_eq!(text.text, "two");
            }
            other => panic!("Expected list item, got {other:?}"),
        }

        match &blocks[2] {
            MarkdownBlock::ListItem {
                ordered,
                number,
                text,
                ..
            } => {
                assert!(*ordered);
                assert_eq!(*number, 1);
                assert_eq!(text.text, "first");
            }
            other => panic!("Expected ordered list item, got {other:?}"),
        }

        match &blocks[3] {
            MarkdownBlock::ListItem {
                ordered,
                number,
                text,
                ..
            } => {
                assert!(*ordered);
                assert_eq!(*number, 2);
                assert_eq!(text.text, "second");
            }
            other => panic!("Expected ordered list item, got {other:?}"),
        }

        match &blocks[4] {
            MarkdownBlock::CodeBlock { text, .. } => {
                assert_eq!(text.trim(), "let x = 1;");
            }
            other => panic!("Expected code block, got {other:?}"),
        }
    }
}
