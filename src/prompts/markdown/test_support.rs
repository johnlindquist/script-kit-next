use super::*;

#[cfg(test)]
fn flush_spans_to_active_list_item(spans: &mut Vec<InlineSpan>, list_item_stack: &mut [ListItem]) {
    if spans.is_empty() {
        return;
    }
    if let Some(item) = list_item_stack.last_mut() {
        item.spans.append(spans);
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TestBlock {
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
pub(super) fn parse_markdown_blocks(text: &str) -> Vec<TestBlock> {
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
                    flush_spans_to_active_list_item(&mut spans, &mut list_item_stack);
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
                        if !list_item_stack.is_empty() {
                            flush_spans_to_active_list_item(&mut spans, &mut list_item_stack);
                        } else {
                            let text: String = std::mem::take(&mut spans)
                                .iter()
                                .map(|s| s.text.as_str())
                                .collect();
                            blocks.push(TestBlock::Paragraph(text));
                        }
                    }
                }
                TagEnd::Heading(_) => {
                    if !spans.is_empty() {
                        let level = heading_level.take().unwrap_or(3);
                        let text: String = std::mem::take(&mut spans)
                            .iter()
                            .map(|s| s.text.as_str())
                            .collect();
                        blocks.push(TestBlock::Heading(level, text));
                    }
                }
                TagEnd::Item => {
                    flush_spans_to_active_list_item(&mut spans, &mut list_item_stack);
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
                        let cell_text: String = std::mem::take(&mut spans)
                            .iter()
                            .map(|s| s.text.as_str())
                            .collect();
                        ts.current_row.push(vec![InlineSpan {
                            text: cell_text,
                            style: InlineStyle::default(),
                            link_url: None,
                        }]);
                    }
                }
                TagEnd::TableHead => {
                    if let Some(ts) = table_state.as_mut() {
                        ts.headers = std::mem::take(&mut ts.current_row);
                        ts.in_head = false;
                    }
                }
                TagEnd::TableRow => {
                    if let Some(ts) = table_state.as_mut() {
                        if !ts.in_head {
                            ts.rows.push(std::mem::take(&mut ts.current_row));
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
