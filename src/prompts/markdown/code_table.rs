use super::*;
use crate::list_item::FONT_MONO;

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
pub(super) fn build_code_block_element(
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
            .font_family(FONT_MONO)
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
pub(super) fn build_table_element(
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
