//! Markdown rendering for chat messages
//!
//! Uses pulldown-cmark for parsing and syntect for fenced code highlighting.
//! Supports: headings, lists, blockquotes, bold/italic, inline code, code blocks, links.
//!
//! Performance: The markdown is parsed once and cached in a global HashMap keyed
//! by content hash + dark-mode flag. On subsequent render frames (e.g. during
//! scrolling at 60fps) we skip pulldown-cmark parsing and syntect highlighting
//! entirely, and only build cheap GPUI elements from the cached representation.

use gpui::{
    div, prelude::*, px, rgb, rgba, AnyElement, ClipboardItem, FontWeight, IntoElement,
    SharedString,
};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::notes::code_highlight::{highlight_code_lines, CodeLine, CodeSpan};
use crate::theme::PromptColors;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct InlineStyle {
    bold: bool,
    italic: bool,
    code: bool,
    link: bool,
    strikethrough: bool,
}

#[derive(Clone, Debug)]
struct InlineSpan {
    text: String,
    style: InlineStyle,
    /// URL for link spans (None for non-link text)
    link_url: Option<String>,
}

#[derive(Clone, Debug)]
struct ListState {
    ordered: bool,
    start: usize,
    items: Vec<ListItem>,
}

/// Table parsing state
#[derive(Debug)]
struct TableState {
    headers: Vec<Vec<InlineSpan>>,
    rows: Vec<Vec<Vec<InlineSpan>>>,
    current_row: Vec<Vec<InlineSpan>>,
    in_head: bool,
}

#[derive(Debug)]
struct CodeBlockState {
    language: Option<String>,
    code: String,
}

#[derive(Debug)]
struct ImageState {
    url: String,
    alt_text: String,
}

// ---------------------------------------------------------------------------
// Cached intermediate representation
// ---------------------------------------------------------------------------

/// A single list item with inline spans and optional task-list checkbox state.
#[derive(Clone, Debug)]
struct ListItem {
    spans: Vec<InlineSpan>,
    /// `Some(true)` = checked `[x]`, `Some(false)` = unchecked `[ ]`, `None` = regular item
    checked: Option<bool>,
    nested_lists: Vec<ListState>,
}

/// Cached intermediate representation of a parsed markdown block.
/// Stored in a global cache to avoid re-parsing on every render frame.
#[derive(Clone, Debug)]
enum ParsedBlock {
    Paragraph {
        spans: Vec<InlineSpan>,
        quote_depth: usize,
    },
    Heading {
        level: u32,
        spans: Vec<InlineSpan>,
        quote_depth: usize,
    },
    ListBlock {
        ordered: bool,
        start: usize,
        items: Vec<ListItem>,
        quote_depth: usize,
    },
    CodeBlock {
        lang_label: String,
        lines: Vec<CodeLine>,
        /// Raw code text for copy-to-clipboard functionality
        raw_code: Arc<str>,
        quote_depth: usize,
    },
    Table {
        headers: Vec<Vec<InlineSpan>>,
        rows: Vec<Vec<Vec<InlineSpan>>>,
        quote_depth: usize,
    },
    HorizontalRule {
        quote_depth: usize,
    },
}

static MARKDOWN_CACHE: OnceLock<Mutex<HashMap<u64, Arc<Vec<ParsedBlock>>>>> = OnceLock::new();
static MARKDOWN_VOLATILE_SCOPE_COUNTER: AtomicU64 = AtomicU64::new(1);
const INFERRED_SCOPE_PREFIX_CHARS: usize = 256;

fn markdown_cache_key(text: &str, is_dark: bool) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    is_dark.hash(&mut hasher);
    hasher.finish()
}

fn stable_markdown_scope_hash(scope: Option<&str>) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    match scope {
        Some(scope) => {
            "scoped".hash(&mut hasher);
            scope.hash(&mut hasher);
        }
        None => {
            // Unscoped renders need unique IDs to avoid collisions when the same
            // markdown appears in multiple places at once. These IDs are stable
            // only within a single render pass.
            let nonce = MARKDOWN_VOLATILE_SCOPE_COUNTER.fetch_add(1, Ordering::Relaxed);
            "volatile".hash(&mut hasher);
            nonce.hash(&mut hasher);
        }
    }
    hasher.finish()
}

fn scoped_markdown_element_id(
    scope_hash: u64,
    kind: &str,
    primary_index: usize,
    secondary_index: usize,
) -> SharedString {
    SharedString::from(format!(
        "md-{scope_hash:016x}-{kind}-{primary_index}-{secondary_index}"
    ))
}

fn scoped_markdown_numeric_key(
    scope_hash: u64,
    kind: &str,
    primary_index: usize,
    secondary_index: usize,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    scope_hash.hash(&mut hasher);
    kind.hash(&mut hasher);
    primary_index.hash(&mut hasher);
    secondary_index.hash(&mut hasher);
    hasher.finish()
}

fn inferred_markdown_scope_hash(text: &str) -> u64 {
    let prefix_end = text
        .char_indices()
        .nth(INFERRED_SCOPE_PREFIX_CHARS)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len());
    stable_markdown_scope_hash(Some(&text[..prefix_end]))
}

// ---------------------------------------------------------------------------
// Phase 1: Parse markdown → Vec<ParsedBlock>  (cached)
// ---------------------------------------------------------------------------

fn parse_markdown(text: &str, is_dark: bool) -> Vec<ParsedBlock> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(text, options);

    let mut blocks: Vec<ParsedBlock> = Vec::new();
    let mut spans: Vec<InlineSpan> = Vec::new();
    let mut style_stack: Vec<InlineStyle> = vec![InlineStyle::default()];
    let mut heading_level: Option<u32> = None;
    let mut list_stack: Vec<ListState> = Vec::new();
    let mut list_item_stack: Vec<ListItem> = Vec::new();
    let mut quote_depth: usize = 0;
    let mut code_block: Option<CodeBlockState> = None;
    let mut image_state: Option<ImageState> = None;
    let mut current_link_url: Option<String> = None;
    let mut table_state: Option<TableState> = None;
    let mut in_table_cell: bool = false;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => spans.clear(),
                Tag::Heading { level, .. } => {
                    heading_level = Some(heading_level_to_u32(level));
                    spans.clear();
                }
                Tag::List(start) => {
                    list_stack.push(ListState {
                        ordered: start.is_some(),
                        start: start.unwrap_or(1) as usize,
                        items: Vec::new(),
                    });
                }
                Tag::Item => {
                    list_item_stack.push(ListItem {
                        spans: Vec::new(),
                        checked: None,
                        nested_lists: Vec::new(),
                    });
                    spans.clear();
                }
                Tag::BlockQuote(_) => {
                    quote_depth += 1;
                }
                Tag::Emphasis => push_style(&mut style_stack, |style| style.italic = true),
                Tag::Strong => push_style(&mut style_stack, |style| style.bold = true),
                Tag::Link { dest_url, .. } => {
                    push_style(&mut style_stack, |style| style.link = true);
                    current_link_url = Some(dest_url.to_string());
                }
                Tag::Image { dest_url, .. } => {
                    image_state = Some(ImageState {
                        url: dest_url.to_string(),
                        alt_text: String::new(),
                    });
                }
                Tag::Strikethrough => {
                    push_style(&mut style_stack, |style| style.strikethrough = true)
                }
                Tag::CodeBlock(kind) => {
                    code_block = Some(CodeBlockState {
                        language: code_block_language(&kind),
                        code: String::new(),
                    });
                }
                Tag::Table(_alignments) => {
                    table_state = Some(TableState {
                        headers: Vec::new(),
                        rows: Vec::new(),
                        current_row: Vec::new(),
                        in_head: false,
                    });
                }
                Tag::TableHead => {
                    if let Some(ts) = table_state.as_mut() {
                        ts.in_head = true;
                        ts.current_row.clear();
                    }
                }
                Tag::TableRow => {
                    if let Some(ts) = table_state.as_mut() {
                        ts.current_row.clear();
                    }
                }
                Tag::TableCell => {
                    in_table_cell = true;
                    spans.clear();
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => {
                    if !spans.is_empty() {
                        if let Some(item) = list_item_stack.last_mut() {
                            item.spans.append(&mut spans);
                        } else {
                            blocks.push(ParsedBlock::Paragraph {
                                spans: spans.clone(),
                                quote_depth,
                            });
                            spans.clear();
                        }
                    }
                }
                TagEnd::Heading(_) => {
                    if !spans.is_empty() {
                        let level = heading_level.take().unwrap_or(3);
                        blocks.push(ParsedBlock::Heading {
                            level,
                            spans: spans.clone(),
                            quote_depth,
                        });
                        spans.clear();
                    }
                }
                TagEnd::Item => {
                    if !spans.is_empty() {
                        if let Some(item) = list_item_stack.last_mut() {
                            item.spans.append(&mut spans);
                        }
                    }
                    if let Some(item) = list_item_stack.pop() {
                        if let Some(list) = list_stack.last_mut() {
                            list.items.push(item);
                        }
                    }
                }
                TagEnd::List(_) => {
                    if let Some(list) = list_stack.pop() {
                        if let Some(parent_item) = list_item_stack.last_mut() {
                            parent_item.nested_lists.push(list);
                        } else {
                            blocks.push(ParsedBlock::ListBlock {
                                ordered: list.ordered,
                                start: list.start,
                                items: list.items,
                                quote_depth,
                            });
                        }
                    }
                }
                TagEnd::BlockQuote(_) => {
                    quote_depth = quote_depth.saturating_sub(1);
                }
                TagEnd::Link => {
                    pop_style(&mut style_stack);
                    current_link_url = None;
                }
                TagEnd::Image => {
                    if let Some(image) = image_state.take() {
                        let alt_text = image.alt_text.trim();
                        let label = if alt_text.is_empty() {
                            "[Image]".to_string()
                        } else {
                            format!("[Image: {}]", alt_text)
                        };
                        let mut style = *style_stack.last().unwrap_or(&InlineStyle::default());
                        style.link = true;
                        push_text_span(&mut spans, &label, style, Some(image.url.as_str()));
                    }
                }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {
                    pop_style(&mut style_stack);
                }
                TagEnd::CodeBlock => {
                    if let Some(block) = code_block.take() {
                        let lang_label = block.language.as_deref().unwrap_or("").trim().to_string();
                        let lines =
                            highlight_code_lines(&block.code, block.language.as_deref(), is_dark);
                        let raw_code: Arc<str> = Arc::from(block.code);
                        blocks.push(ParsedBlock::CodeBlock {
                            lang_label,
                            lines,
                            raw_code,
                            quote_depth,
                        });
                    }
                }
                TagEnd::Table => {
                    if let Some(ts) = table_state.take() {
                        blocks.push(ParsedBlock::Table {
                            headers: ts.headers,
                            rows: ts.rows,
                            quote_depth,
                        });
                    }
                }
                TagEnd::TableHead => {
                    if let Some(ts) = table_state.as_mut() {
                        ts.headers = ts.current_row.clone();
                        ts.current_row.clear();
                        ts.in_head = false;
                    }
                }
                TagEnd::TableRow => {
                    if let Some(ts) = table_state.as_mut() {
                        if !ts.in_head {
                            ts.rows.push(ts.current_row.clone());
                            ts.current_row.clear();
                        }
                    }
                }
                TagEnd::TableCell => {
                    if let Some(ts) = table_state.as_mut() {
                        ts.current_row.push(spans.clone());
                    }
                    spans.clear();
                    in_table_cell = false;
                }
                _ => {}
            },
            Event::Text(text) => {
                if let Some(block) = code_block.as_mut() {
                    block.code.push_str(&text);
                } else if let Some(image) = image_state.as_mut() {
                    image.alt_text.push_str(&text);
                } else {
                    let style = *style_stack.last().unwrap_or(&InlineStyle::default());
                    push_text_span(&mut spans, &text, style, current_link_url.as_deref());
                }
            }
            Event::Code(code) => {
                if let Some(image) = image_state.as_mut() {
                    image.alt_text.push_str(&code);
                } else {
                    let mut style = *style_stack.last().unwrap_or(&InlineStyle::default());
                    style.code = true;
                    push_text_span(&mut spans, &code, style, current_link_url.as_deref());
                }
            }
            Event::SoftBreak => {
                if let Some(image) = image_state.as_mut() {
                    image.alt_text.push(' ');
                } else {
                    let style = *style_stack.last().unwrap_or(&InlineStyle::default());
                    push_text_span(&mut spans, " ", style, current_link_url.as_deref());
                }
            }
            Event::HardBreak => {
                if let Some(image) = image_state.as_mut() {
                    image.alt_text.push('\n');
                } else {
                    let style = *style_stack.last().unwrap_or(&InlineStyle::default());
                    push_text_span(&mut spans, "\n", style, current_link_url.as_deref());
                }
            }
            Event::Rule => {
                blocks.push(ParsedBlock::HorizontalRule { quote_depth });
            }
            Event::Html(html) => {
                if let Some(image) = image_state.as_mut() {
                    image.alt_text.push_str(&html);
                } else {
                    let style = *style_stack.last().unwrap_or(&InlineStyle::default());
                    push_text_span(&mut spans, &html, style, current_link_url.as_deref());
                }
            }
            Event::TaskListMarker(checked) => {
                if let Some(item) = list_item_stack.last_mut() {
                    item.checked = Some(checked);
                }
            }
            _ => {}
        }
    }

    // Suppress unused variable warning — in_table_cell tracks state during parsing
    let _ = in_table_cell;

    blocks
}

// ---------------------------------------------------------------------------
// Phase 2: Vec<ParsedBlock> → GPUI elements  (every frame, cheap)
// ---------------------------------------------------------------------------

fn build_markdown_elements(
    blocks: &[ParsedBlock],
    colors: &PromptColors,
    render_scope_hash: u64,
) -> Vec<AnyElement> {
    let mut elements: Vec<AnyElement> = Vec::new();
    let mut block_index: usize = 0;

    for block in blocks {
        match block {
            ParsedBlock::Paragraph { spans, quote_depth } => {
                if !spans.is_empty() {
                    let current_block = next_markdown_block_index(&mut block_index);
                    let element =
                        render_inline_spans(spans, colors, render_scope_hash, current_block, 0)
                            .w_full();
                    push_scoped_block(
                        &mut elements,
                        element,
                        *quote_depth,
                        colors,
                        render_scope_hash,
                        current_block,
                    );
                }
            }
            ParsedBlock::Heading {
                level,
                spans,
                quote_depth,
            } => {
                if !spans.is_empty() {
                    let current_block = next_markdown_block_index(&mut block_index);
                    let mut heading =
                        render_inline_spans(spans, colors, render_scope_hash, current_block, 0)
                            .w_full()
                            .text_color(rgb(colors.text_primary));
                    heading = match level {
                        1 => heading.text_lg().font_weight(FontWeight::BOLD),
                        2 => heading.text_base().font_weight(FontWeight::SEMIBOLD),
                        3 => heading.text_sm().font_weight(FontWeight::SEMIBOLD),
                        _ => heading.text_sm().font_weight(FontWeight::MEDIUM),
                    };
                    push_scoped_block(
                        &mut elements,
                        heading,
                        *quote_depth,
                        colors,
                        render_scope_hash,
                        current_block,
                    );
                }
            }
            ParsedBlock::ListBlock {
                ordered,
                start,
                items,
                quote_depth,
            } => {
                append_list_items(
                    &mut elements,
                    *ordered,
                    *start,
                    items,
                    *quote_depth,
                    colors,
                    0,
                    render_scope_hash,
                    &mut block_index,
                );
            }
            ParsedBlock::CodeBlock {
                lang_label,
                lines,
                raw_code,
                quote_depth,
            } => {
                let current_block = next_markdown_block_index(&mut block_index);
                let element = build_code_block_element(
                    lang_label,
                    lines,
                    raw_code,
                    colors,
                    scoped_markdown_element_id(render_scope_hash, "code", current_block, 0),
                    scoped_markdown_numeric_key(render_scope_hash, "code-copy", current_block, 0),
                );
                push_scoped_block(
                    &mut elements,
                    element,
                    *quote_depth,
                    colors,
                    render_scope_hash,
                    current_block,
                );
            }
            ParsedBlock::Table {
                headers,
                rows,
                quote_depth,
            } => {
                let current_block = next_markdown_block_index(&mut block_index);
                let element =
                    build_table_element(headers, rows, colors, render_scope_hash, current_block);
                push_scoped_block(
                    &mut elements,
                    element,
                    *quote_depth,
                    colors,
                    render_scope_hash,
                    current_block,
                );
            }
            ParsedBlock::HorizontalRule { quote_depth } => {
                let current_block = next_markdown_block_index(&mut block_index);
                push_scoped_block(
                    &mut elements,
                    render_hr(colors),
                    *quote_depth,
                    colors,
                    render_scope_hash,
                    current_block,
                );
            }
        }
    }

    elements
}

fn append_list_items(
    elements: &mut Vec<AnyElement>,
    ordered: bool,
    start: usize,
    items: &[ListItem],
    quote_depth: usize,
    colors: &PromptColors,
    depth: usize,
    render_scope_hash: u64,
    block_index: &mut usize,
) {
    for (index, item) in items.iter().enumerate() {
        let current_block = next_markdown_block_index(block_index);

        // Task list checkbox or regular marker
        let marker_element: AnyElement = if let Some(checked) = item.checked {
            let (symbol, color) = if checked {
                ("\u{2611}", rgb(colors.accent_color)) // ☑ checked
            } else {
                ("\u{2610}", rgb(colors.text_tertiary)) // ☐ unchecked
            };
            div()
                .flex_shrink_0()
                .text_color(color)
                .child(symbol.to_string())
                .into_any_element()
        } else {
            let marker = list_marker(ordered, start, index);
            div()
                .flex_shrink_0()
                .text_color(rgb(colors.text_tertiary))
                .child(marker)
                .into_any_element()
        };

        // Checked items get muted text color (strikethrough style)
        let content =
            render_inline_spans(&item.spans, colors, render_scope_hash, current_block, depth)
                .min_w_0()
                .flex_1();
        let content = if item.checked == Some(true) {
            content
                .text_color(rgb(colors.text_tertiary))
                .into_any_element()
        } else {
            content.into_any_element()
        };

        let mut row = div()
            .flex()
            .flex_row()
            .w_full()
            .gap(px(6.0))
            .text_sm()
            .child(marker_element)
            .child(content);
        if depth > 0 {
            row = row.pl(px(depth as f32 * 14.0));
        }
        push_scoped_block(
            elements,
            row,
            quote_depth,
            colors,
            render_scope_hash,
            current_block,
        );

        for nested_list in &item.nested_lists {
            append_list_items(
                elements,
                nested_list.ordered,
                nested_list.start,
                &nested_list.items,
                quote_depth,
                colors,
                depth + 1,
                render_scope_hash,
                block_index,
            );
        }
    }
}

/// Tracks the last-copied code block ID and the instant it was copied.
/// Used to show brief "Copied!" feedback on the copy button.
static LAST_COPIED_CODE_BLOCK: OnceLock<Mutex<(u64, std::time::Instant)>> = OnceLock::new();

fn mark_code_block_copied(block_id: u64) {
    let state = LAST_COPIED_CODE_BLOCK.get_or_init(|| Mutex::new((0, std::time::Instant::now())));
    if let Ok(mut guard) = state.lock() {
        *guard = (block_id, std::time::Instant::now());
    }
}

fn is_code_block_recently_copied(block_id: u64) -> bool {
    let state = LAST_COPIED_CODE_BLOCK.get_or_init(|| Mutex::new((0, std::time::Instant::now())));
    if let Ok(guard) = state.lock() {
        guard.0 == block_id && guard.1.elapsed().as_secs() < 2
    } else {
        false
    }
}

/// Build a code block element from pre-highlighted lines (avoids re-running syntect).
/// Includes a language label header and a hover-revealed copy button.
fn build_code_block_element(
    lang_label: &str,
    lines: &[CodeLine],
    raw_code: &Arc<str>,
    colors: &PromptColors,
    code_block_element_id: SharedString,
    copy_state_id: u64,
) -> gpui::Stateful<gpui::Div> {
    let group_name: SharedString = format!("code-block-{}", copy_state_id).into();

    let code_for_copy = raw_code.clone();
    let header_border_color = rgba((colors.quote_border << 8) | 0x30);
    let text_tertiary = colors.text_tertiary;

    let mut code_container = div()
        .id(code_block_element_id)
        .group(group_name.clone())
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

    // Header row: language label + copy button
    let has_label = !lang_label.is_empty();
    if has_label || !raw_code.is_empty() {
        code_container = code_container.child(
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .px(px(10.0))
                .py(px(4.0))
                .when(has_label || !raw_code.is_empty(), |d| {
                    d.border_b_1().border_color(header_border_color)
                })
                .child({
                    // Build combined label: "lang · N lines" or just "N lines" or just "lang"
                    let line_count = lines.len();
                    let combined_label = match (has_label, line_count > 1) {
                        (true, true) => format!("{} · {} lines", lang_label, line_count),
                        (true, false) => lang_label.to_string(),
                        (false, true) => format!("{} lines", line_count),
                        (false, false) => String::new(),
                    };
                    div()
                        .text_xs()
                        .text_color(rgb(text_tertiary))
                        .child(combined_label)
                })
                // Copy button - visible on hover, shows "Copied!" feedback
                .child({
                    let is_just_copied = is_code_block_recently_copied(copy_state_id);
                    let copy_block_id = copy_state_id;
                    let hover_bg = rgba((colors.quote_border << 8) | 0x30);
                    div()
                        .id(SharedString::from(format!("copy-code-{}", copy_state_id)))
                        .flex()
                        .items_center()
                        .gap(px(3.))
                        .px(px(4.))
                        .py(px(1.))
                        .rounded(px(3.))
                        .cursor_pointer()
                        .when(!is_just_copied, |d| {
                            d.opacity(0.).group_hover(group_name, |s| s.opacity(1.0))
                        })
                        .hover(|s| s.bg(hover_bg))
                        .on_click(move |_event, _window, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(
                                code_for_copy.to_string(),
                            ));
                            mark_code_block_copied(copy_block_id);
                        })
                        .child(if is_just_copied {
                            div()
                                .flex()
                                .items_center()
                                .gap(px(3.))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(colors.accent_color))
                                        .child("Copied!"),
                                )
                                .into_any_element()
                        } else {
                            div()
                                .text_xs()
                                .text_color(rgb(text_tertiary))
                                .child("Copy")
                                .into_any_element()
                        })
                }),
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
            for span in &line.spans {
                line_div =
                    line_div.child(div().text_color(rgb(span.color)).child(span.text.clone()));
            }
        }

        body = body.child(line_div);
    }

    code_container.child(body)
}

/// Build a table element from parsed header/row data.
/// Renders as a bordered grid with header row in bold and alternating row backgrounds.
fn build_table_element(
    headers: &[Vec<InlineSpan>],
    rows: &[Vec<Vec<InlineSpan>>],
    colors: &PromptColors,
    render_scope_hash: u64,
    block_index: usize,
) -> gpui::Div {
    let border_color = rgba((colors.quote_border << 8) | 0x40);
    let header_bg = rgba((colors.code_bg << 8) | 0xC0);
    let row_alt_bg = rgba((colors.code_bg << 8) | 0x40);

    let mut table = div()
        .w_full()
        .mt(px(4.0))
        .mb(px(4.0))
        .rounded(px(6.0))
        .border_1()
        .border_color(border_color)
        .overflow_hidden()
        .flex()
        .flex_col();

    // Header row
    if !headers.is_empty() {
        let mut header_row = div()
            .flex()
            .flex_row()
            .w_full()
            .bg(header_bg)
            .border_b_1()
            .border_color(border_color);
        for (cell_index, cell_spans) in headers.iter().enumerate() {
            header_row = header_row.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .px(px(8.0))
                    .py(px(6.0))
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(colors.text_primary))
                    .child(render_inline_spans(
                        cell_spans,
                        colors,
                        render_scope_hash,
                        block_index,
                        cell_index,
                    )),
            );
        }
        table = table.child(header_row);
    }

    // Data rows
    for (row_idx, row) in rows.iter().enumerate() {
        let is_alt = row_idx % 2 == 1;
        let is_last = row_idx == rows.len() - 1;
        let mut row_div = div().flex().flex_row().w_full();
        if is_alt {
            row_div = row_div.bg(row_alt_bg);
        }
        if !is_last {
            row_div = row_div.border_b_1().border_color(border_color);
        }
        for (cell_index, cell_spans) in row.iter().enumerate() {
            row_div = row_div.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .px(px(8.0))
                    .py(px(5.0))
                    .text_xs()
                    .text_color(rgb(colors.text_primary))
                    .child(render_inline_spans(
                        cell_spans,
                        colors,
                        render_scope_hash,
                        block_index,
                        headers.len() + (row_idx * 16) + cell_index,
                    )),
            );
        }
        table = table.child(row_div);
    }

    table
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Render markdown text to GPUI elements.
///
/// Uses a global cache to avoid re-parsing markdown and re-highlighting code
/// on every render frame. The cache is keyed on (content hash, dark-mode flag).
pub fn render_markdown(text: &str, colors: &PromptColors) -> gpui::Div {
    render_markdown_with_scope(text, colors, None)
}

/// Render markdown with a stable scope identifier.
///
/// When `scope` is stable across updates (for example: assistant message ID while
/// streaming), interactive element IDs remain stable too, allowing GPUI to reuse
/// unchanged subtrees instead of replacing the entire markdown tree every tick.
pub fn render_markdown_with_scope(
    text: &str,
    colors: &PromptColors,
    scope: Option<&str>,
) -> gpui::Div {
    // Check cache for parsed blocks
    let key = markdown_cache_key(text, colors.is_dark);
    let cache = MARKDOWN_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let render_scope_hash = scope
        .map(|s| stable_markdown_scope_hash(Some(s)))
        .unwrap_or_else(|| inferred_markdown_scope_hash(text));

    let parsed_blocks = if let Ok(guard) = cache.lock() {
        guard.get(&key).cloned()
    } else {
        None
    };

    let parsed_blocks = parsed_blocks.unwrap_or_else(|| {
        let blocks = Arc::new(parse_markdown(text, colors.is_dark));
        if let Ok(mut guard) = cache.lock() {
            // Cap cache size to prevent unbounded growth.
            // Use a high limit to avoid full-cache clears during streaming,
            // which would force every message to be re-parsed.
            if guard.len() > 1024 {
                guard.clear();
            }
            guard.insert(key, blocks.clone());
        }
        blocks
    });

    let elements = build_markdown_elements(parsed_blocks.as_slice(), colors, render_scope_hash);

    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .w_full()
        .min_w_0()
        .children(elements)
}

// ---------------------------------------------------------------------------
// Helpers (shared by both phases)
// ---------------------------------------------------------------------------

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

fn list_marker(ordered: bool, start: usize, index: usize) -> String {
    if ordered {
        format!("{}.", start + index)
    } else {
        "\u{2022}".to_string()
    }
}

fn is_allowed_markdown_url(url: &str) -> bool {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return false;
    }

    let lowercase = trimmed.to_ascii_lowercase();
    if lowercase.starts_with("http://")
        || lowercase.starts_with("https://")
        || lowercase.starts_with("mailto:")
    {
        return true;
    }

    // Relative links and bare hosts (without scheme) are allowed.
    !trimmed.contains(':')
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

fn push_text_span(
    spans: &mut Vec<InlineSpan>,
    text: &str,
    style: InlineStyle,
    link_url: Option<&str>,
) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = spans.last_mut() {
        if last.style == style && last.link_url.as_deref() == link_url {
            last.text.push_str(text);
            return;
        }
    }
    spans.push(InlineSpan {
        text: text.to_string(),
        style,
        link_url: link_url.map(|s| s.to_string()),
    });
}

fn next_markdown_block_index(block_index: &mut usize) -> usize {
    let current = *block_index;
    *block_index += 1;
    current
}

fn into_quoted_block(
    element: impl IntoElement,
    quote_depth: usize,
    colors: &PromptColors,
) -> AnyElement {
    let mut element = element.into_any_element();
    if quote_depth > 0 {
        element = div()
            .w_full()
            .pl(px(12.0))
            .py(px(2.0))
            .border_l_2()
            .border_color(rgb(colors.quote_border))
            .bg(rgba((colors.quote_border << 8) | 0x10))
            .rounded_r(px(4.0))
            .child(element)
            .into_any_element();
    }
    element
}

fn push_scoped_block(
    blocks: &mut Vec<AnyElement>,
    element: impl IntoElement,
    quote_depth: usize,
    colors: &PromptColors,
    render_scope_hash: u64,
    block_index: usize,
) {
    let quoted = into_quoted_block(element, quote_depth, colors);
    blocks.push(
        div()
            .id(scoped_markdown_element_id(
                render_scope_hash,
                "block",
                block_index,
                0,
            ))
            .w_full()
            .child(quoted)
            .into_any_element(),
    );
}

/// Counter for generating unique link element IDs (needed for on_click handlers).
static NEXT_LINK_ID: AtomicU64 = AtomicU64::new(0);

fn style_span(
    text: &str,
    style: &InlineStyle,
    colors: &PromptColors,
    link_url: Option<&str>,
    link_element_id: Option<SharedString>,
) -> AnyElement {
    if style.code {
        return div()
            .px(px(4.0))
            .py(px(1.0))
            .bg(rgba((colors.code_bg << 8) | 0x80))
            .rounded(px(3.0))
            .font_family("Menlo")
            .text_color(rgb(colors.text_primary))
            .child(text.to_string())
            .into_any_element();
    }

    // Clickable link with URL — needs .id() for interactivity
    if style.link {
        if let Some(url) = link_url {
            let fallback_id = NEXT_LINK_ID.fetch_add(1, Ordering::Relaxed);
            let id = link_element_id
                .unwrap_or_else(|| SharedString::from(format!("md-link-{fallback_id}")));
            let url_owned = url.to_string();
            let accent = rgb(colors.accent_color);
            if is_allowed_markdown_url(url) {
                return div()
                    .id(id.clone())
                    .text_color(accent)
                    .border_b_1()
                    .border_color(rgba((colors.accent_color << 8) | 0x40))
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.7))
                    .on_click(move |_, _window, _cx| {
                        let _ = open::that(&url_owned);
                    })
                    .when(style.bold, |d| d.font_weight(FontWeight::BOLD))
                    .when(style.italic, |d| d.italic())
                    .child(text.to_string())
                    .into_any_element();
            }

            return div()
                .id(id)
                .text_color(accent)
                .border_b_1()
                .border_color(rgba((colors.accent_color << 8) | 0x40))
                .opacity(0.7)
                .when(style.bold, |d| d.font_weight(FontWeight::BOLD))
                .when(style.italic, |d| d.italic())
                .child(text.to_string())
                .into_any_element();
        }
        // Link without URL (fallback — just styled text)
        let mut piece = div()
            .text_color(rgb(colors.accent_color))
            .child(text.to_string());
        if style.bold {
            piece = piece.font_weight(FontWeight::BOLD);
        }
        if style.italic {
            piece = piece.italic();
        }
        return piece.into_any_element();
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
    if style.strikethrough {
        // Simulate strikethrough with muted color + line-through border trick
        piece = piece.text_color(rgb(colors.text_tertiary));
    }
    piece.into_any_element()
}

fn render_inline_spans(
    spans: &[InlineSpan],
    colors: &PromptColors,
    render_scope_hash: u64,
    block_index: usize,
    span_group_index: usize,
) -> gpui::Div {
    // Fast path: single plain-text span — render as simple text child.
    // Avoids flex_wrap entirely so text wraps naturally at word boundaries.
    if spans.len() == 1 && spans[0].style == InlineStyle::default() && spans[0].link_url.is_none() {
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
    let link_kind = format!("link-{}", span_group_index);
    let mut link_index: usize = 0;

    for span in spans {
        let url = span.link_url.as_deref();
        if span.style.code {
            // Code spans stay as single units (they have bg/padding)
            row = row.child(style_span(&span.text, &span.style, colors, url, None));
        } else if span.style.link && url.is_some() {
            // Link spans: keep as a single unit so the underline is continuous
            let current_link = link_index;
            link_index += 1;
            row = row.child(style_span(
                &span.text,
                &span.style,
                colors,
                url,
                Some(scoped_markdown_element_id(
                    render_scope_hash,
                    link_kind.as_str(),
                    block_index,
                    current_link,
                )),
            ));
        } else {
            // Split text at whitespace for natural word wrapping while preserving hard breaks.
            for line_segment in span.text.split_inclusive('\n') {
                let has_break = line_segment.ends_with('\n');
                let segment = line_segment.strip_suffix('\n').unwrap_or(line_segment);
                for word in segment.split_inclusive(char::is_whitespace) {
                    if !word.is_empty() {
                        row = row.child(style_span(word, &span.style, colors, url, None));
                    }
                }
                if has_break {
                    row = row.child(div().w_full().h(px(0.0)));
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

/// Render a code block (kept for backward compatibility; new path uses
/// `build_code_block_element` with pre-highlighted lines).
#[allow(dead_code)]
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
    /// A single list item: (marker, text). Marker is "☑"/"☐" for task list items.
    ListItem(String, String),
    /// A code block: (language, code).
    CodeBlock(Option<String>, String),
    /// A table: (headers, rows) where each cell is concatenated text.
    Table(Vec<String>, Vec<Vec<String>>),
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
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(text, options);

    let mut blocks: Vec<TestBlock> = Vec::new();
    let mut spans: Vec<InlineSpan> = Vec::new();
    let mut style_stack: Vec<InlineStyle> = vec![InlineStyle::default()];
    let mut heading_level: Option<u32> = None;
    let mut list_stack: Vec<ListState> = Vec::new();
    let mut list_item_stack: Vec<ListItem> = Vec::new();
    let mut code_block: Option<CodeBlockState> = None;
    let mut image_state: Option<ImageState> = None;
    let mut table_state: Option<TableState> = None;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => spans.clear(),
                Tag::Heading { level, .. } => {
                    heading_level = Some(heading_level_to_u32(level));
                    spans.clear();
                }
                Tag::List(start) => {
                    list_stack.push(ListState {
                        ordered: start.is_some(),
                        start: start.unwrap_or(1) as usize,
                        items: Vec::new(),
                    });
                }
                Tag::Item => {
                    list_item_stack.push(ListItem {
                        spans: Vec::new(),
                        checked: None,
                        nested_lists: Vec::new(),
                    });
                    spans.clear();
                }
                Tag::Emphasis => push_style(&mut style_stack, |s| s.italic = true),
                Tag::Strong => push_style(&mut style_stack, |s| s.bold = true),
                Tag::Link { .. } => push_style(&mut style_stack, |s| s.link = true),
                Tag::Image { dest_url, .. } => {
                    image_state = Some(ImageState {
                        url: dest_url.to_string(),
                        alt_text: String::new(),
                    });
                }
                Tag::CodeBlock(kind) => {
                    code_block = Some(CodeBlockState {
                        language: code_block_language(&kind),
                        code: String::new(),
                    });
                }
                Tag::Table(_) => {
                    table_state = Some(TableState {
                        headers: Vec::new(),
                        rows: Vec::new(),
                        current_row: Vec::new(),
                        in_head: false,
                    });
                }
                Tag::TableHead => {
                    if let Some(ts) = table_state.as_mut() {
                        ts.in_head = true;
                        ts.current_row.clear();
                    }
                }
                Tag::TableRow => {
                    if let Some(ts) = table_state.as_mut() {
                        ts.current_row.clear();
                    }
                }
                Tag::TableCell => {
                    spans.clear();
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => {
                    if !spans.is_empty() {
                        if let Some(item) = list_item_stack.last_mut() {
                            item.spans.append(&mut spans);
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
                    if !spans.is_empty() {
                        if let Some(item) = list_item_stack.last_mut() {
                            item.spans.append(&mut spans);
                        }
                    }
                    if let Some(item) = list_item_stack.pop() {
                        if let Some(list) = list_stack.last_mut() {
                            list.items.push(item);
                        }
                    }
                }
                TagEnd::List(_) => {
                    if let Some(list) = list_stack.pop() {
                        if let Some(parent_item) = list_item_stack.last_mut() {
                            parent_item.nested_lists.push(list);
                        } else {
                            flatten_test_list(&list, 0, &mut blocks);
                        }
                    }
                }
                TagEnd::TableCell => {
                    if let Some(ts) = table_state.as_mut() {
                        let cell_text: String = spans.iter().map(|s| s.text.as_str()).collect();
                        ts.current_row.push(vec![InlineSpan {
                            text: cell_text,
                            style: InlineStyle::default(),
                            link_url: None,
                        }]);
                    }
                    spans.clear();
                }
                TagEnd::TableHead => {
                    if let Some(ts) = table_state.as_mut() {
                        ts.headers = ts.current_row.clone();
                        ts.current_row.clear();
                        ts.in_head = false;
                    }
                }
                TagEnd::TableRow => {
                    if let Some(ts) = table_state.as_mut() {
                        if !ts.in_head {
                            ts.rows.push(ts.current_row.clone());
                            ts.current_row.clear();
                        }
                    }
                }
                TagEnd::Table => {
                    if let Some(ts) = table_state.take() {
                        let header_texts: Vec<String> = ts
                            .headers
                            .iter()
                            .map(|cell| cell.iter().map(|s| s.text.as_str()).collect())
                            .collect();
                        let row_texts: Vec<Vec<String>> = ts
                            .rows
                            .iter()
                            .map(|row| {
                                row.iter()
                                    .map(|cell| cell.iter().map(|s| s.text.as_str()).collect())
                                    .collect()
                            })
                            .collect();
                        blocks.push(TestBlock::Table(header_texts, row_texts));
                    }
                }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Link | TagEnd::Strikethrough => {
                    pop_style(&mut style_stack);
                }
                TagEnd::Image => {
                    if let Some(image) = image_state.take() {
                        let alt_text = image.alt_text.trim();
                        let label = if alt_text.is_empty() {
                            "[Image]".to_string()
                        } else {
                            format!("[Image: {}]", alt_text)
                        };
                        let mut style = *style_stack.last().unwrap_or(&InlineStyle::default());
                        style.link = true;
                        push_text_span(&mut spans, &label, style, Some(image.url.as_str()));
                    }
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
                } else if let Some(image) = image_state.as_mut() {
                    image.alt_text.push_str(&text);
                } else {
                    let style = *style_stack.last().unwrap_or(&InlineStyle::default());
                    push_text_span(&mut spans, &text, style, None);
                }
            }
            Event::Code(code) => {
                if let Some(image) = image_state.as_mut() {
                    image.alt_text.push_str(&code);
                } else {
                    let mut style = *style_stack.last().unwrap_or(&InlineStyle::default());
                    style.code = true;
                    push_text_span(&mut spans, &code, style, None);
                }
            }
            Event::SoftBreak => {
                if let Some(image) = image_state.as_mut() {
                    image.alt_text.push(' ');
                } else {
                    let style = *style_stack.last().unwrap_or(&InlineStyle::default());
                    push_text_span(&mut spans, " ", style, None);
                }
            }
            Event::HardBreak => {
                if let Some(image) = image_state.as_mut() {
                    image.alt_text.push('\n');
                } else {
                    let style = *style_stack.last().unwrap_or(&InlineStyle::default());
                    push_text_span(&mut spans, "\n", style, None);
                }
            }
            Event::Rule => {
                blocks.push(TestBlock::Hr);
            }
            Event::TaskListMarker(checked) => {
                if let Some(item) = list_item_stack.last_mut() {
                    item.checked = Some(checked);
                }
            }
            _ => {}
        }
    }

    blocks
}

#[cfg(test)]
fn flatten_test_list(list: &ListState, depth: usize, blocks: &mut Vec<TestBlock>) {
    for (index, item) in list.items.iter().enumerate() {
        let marker = if let Some(checked) = item.checked {
            if checked {
                "\u{2611}".to_string() // ☑
            } else {
                "\u{2610}".to_string() // ☐
            }
        } else {
            list_marker(list.ordered, list.start, index)
        };
        let depth_prefix = "  ".repeat(depth);
        let item_text: String = item.spans.iter().map(|s| s.text.as_str()).collect();
        blocks.push(TestBlock::ListItem(
            format!("{}{}", depth_prefix, marker),
            item_text,
        ));

        for nested_list in &item.nested_lists {
            flatten_test_list(nested_list, depth + 1, blocks);
        }
    }
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
                TestBlock::ListItem("\u{2022}".into(), "First item".into()),
                TestBlock::ListItem("\u{2022}".into(), "Second item".into()),
                TestBlock::ListItem("\u{2022}".into(), "Third item".into()),
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
    fn nested_lists_preserve_parent_child_structure() {
        let md = "1. Parent\n   - Child A\n   - Child B\n2. Next\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(
            blocks,
            vec![
                TestBlock::ListItem("1.".into(), "Parent".into()),
                TestBlock::ListItem("  \u{2022}".into(), "Child A".into()),
                TestBlock::ListItem("  \u{2022}".into(), "Child B".into()),
                TestBlock::ListItem("2.".into(), "Next".into()),
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
                TestBlock::ListItem("\u{2022}".into(), "Item one".into()),
                TestBlock::ListItem("\u{2022}".into(), "Item two".into()),
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
                TestBlock::ListItem("\u{2022}".into(), "Item A".into()),
                TestBlock::ListItem("\u{2022}".into(), "Item B".into()),
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
                TestBlock::ListItem("\u{2022}".into(), "Bold item".into()),
                TestBlock::ListItem("\u{2022}".into(), "Item with code".into()),
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
                TestBlock::ListItem("\u{2022}".into(), "Item".into()),
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
                TestBlock::ListItem("\u{2022}".into(), "First item".into()),
                TestBlock::ListItem("\u{2022}".into(), "Second item".into()),
                TestBlock::ListItem("\u{2022}".into(), "Third item".into()),
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

    #[test]
    fn task_list_renders_checkboxes() {
        let md = "- [x] Done task\n- [ ] Pending task\n- Regular item\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(
            blocks,
            vec![
                TestBlock::ListItem("\u{2611}".into(), "Done task".into()),
                TestBlock::ListItem("\u{2610}".into(), "Pending task".into()),
                TestBlock::ListItem("\u{2022}".into(), "Regular item".into()),
            ]
        );
    }

    #[test]
    fn simple_table_parses_headers_and_rows() {
        let md = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(
            blocks,
            vec![TestBlock::Table(
                vec!["Name".into(), "Age".into()],
                vec![
                    vec!["Alice".into(), "30".into()],
                    vec!["Bob".into(), "25".into()],
                ]
            )]
        );
    }

    #[test]
    fn hard_break_preserves_line_break() {
        let md = "line one  \nline two\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(
            blocks,
            vec![TestBlock::Paragraph("line one\nline two".into())]
        );
    }

    #[test]
    fn markdown_image_preserves_alt_text_and_url() {
        let md = "![diagram](https://example.com/diagram.png)\n";
        let blocks = parse_markdown(md, true);
        assert_eq!(blocks.len(), 1);

        match &blocks[0] {
            ParsedBlock::Paragraph { spans, .. } => {
                assert_eq!(spans.len(), 1);
                assert_eq!(spans[0].text, "[Image: diagram]");
                assert!(spans[0].style.link);
                assert_eq!(
                    spans[0].link_url.as_deref(),
                    Some("https://example.com/diagram.png")
                );
            }
            other => panic!("expected paragraph block, got: {other:?}"),
        }
    }

    #[test]
    fn markdown_link_url_allowlist_rejects_unsafe_schemes() {
        assert!(is_allowed_markdown_url("https://example.com"));
        assert!(is_allowed_markdown_url("http://example.com"));
        assert!(is_allowed_markdown_url("mailto:test@example.com"));
        assert!(is_allowed_markdown_url("relative/path"));
        assert!(!is_allowed_markdown_url("file:///tmp/secrets.txt"));
        assert!(!is_allowed_markdown_url("javascript:alert(1)"));
        assert!(!is_allowed_markdown_url("data:text/html,hello"));
    }

    #[test]
    fn markdown_scope_hash_is_deterministic_for_same_scope() {
        let first = stable_markdown_scope_hash(Some("assistant-msg-123"));
        let second = stable_markdown_scope_hash(Some("assistant-msg-123"));
        let other = stable_markdown_scope_hash(Some("assistant-msg-456"));

        assert_eq!(first, second);
        assert_ne!(first, other);
    }

    #[test]
    fn scoped_markdown_element_id_is_stable_and_indexed() {
        let scope_hash = stable_markdown_scope_hash(Some("assistant-msg-123"));
        let block_a = scoped_markdown_element_id(scope_hash, "block", 7, 0);
        let block_a_again = scoped_markdown_element_id(scope_hash, "block", 7, 0);
        let block_b = scoped_markdown_element_id(scope_hash, "block", 8, 0);

        assert_eq!(block_a, block_a_again);
        assert_ne!(block_a, block_b);
    }

    #[test]
    fn inferred_scope_hash_stays_stable_for_appended_content_after_prefix_window() {
        let stable_prefix = "a".repeat(INFERRED_SCOPE_PREFIX_CHARS + 32);
        let baseline = format!("{stable_prefix}\n\n- item 1");
        let appended = format!("{baseline}\n- item 2\n- item 3");

        assert_eq!(
            inferred_markdown_scope_hash(&baseline),
            inferred_markdown_scope_hash(&appended),
            "Appended tail content should not change inferred scope hash once prefix window is filled",
        );
    }
}
