use super::*;

fn flush_spans_to_active_list_item(spans: &mut Vec<InlineSpan>, list_item_stack: &mut [ListItem]) {
    if spans.is_empty() {
        return;
    }
    if let Some(item) = list_item_stack.last_mut() {
        item.spans.append(spans);
    }
}

pub(super) fn parse_markdown(text: &str, is_dark: bool) -> Vec<ParsedBlock> {
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
                        if !list_item_stack.is_empty() {
                            flush_spans_to_active_list_item(&mut spans, &mut list_item_stack);
                        } else {
                            blocks.push(ParsedBlock::Paragraph {
                                spans: std::mem::take(&mut spans),
                                quote_depth,
                            });
                        }
                    }
                }
                TagEnd::Heading(_) => {
                    if !spans.is_empty() {
                        let level = heading_level.take().unwrap_or(3);
                        blocks.push(ParsedBlock::Heading {
                            level,
                            spans: std::mem::take(&mut spans),
                            quote_depth,
                        });
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
                        let target_url = current_link_url.as_deref().unwrap_or(image.url.as_str());
                        push_text_span(&mut spans, &label, style, Some(target_url));
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
                TagEnd::TableCell => {
                    if let Some(ts) = table_state.as_mut() {
                        ts.current_row.push(std::mem::take(&mut spans));
                    }
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
