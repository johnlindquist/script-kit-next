use super::*;

pub(super) fn build_markdown_elements(
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

#[allow(clippy::too_many_arguments)]
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
