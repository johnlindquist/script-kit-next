use super::*;

pub(super) fn heading_level_to_u32(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

pub(super) fn code_block_language(kind: &CodeBlockKind) -> Option<String> {
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

pub(super) fn list_marker(ordered: bool, start: usize, index: usize) -> String {
    if ordered {
        format!("{}.", start + index)
    } else {
        "\u{2022}".to_string()
    }
}

pub(super) fn is_allowed_markdown_url(url: &str) -> bool {
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

pub(super) fn push_style(stack: &mut Vec<InlineStyle>, update: impl FnOnce(&mut InlineStyle)) {
    let mut next = *stack.last().unwrap_or(&InlineStyle::default());
    update(&mut next);
    stack.push(next);
}

pub(super) fn pop_style(stack: &mut Vec<InlineStyle>) {
    if stack.len() > 1 {
        stack.pop();
    }
}

pub(super) fn push_text_span(
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

pub(super) fn next_markdown_block_index(block_index: &mut usize) -> usize {
    let current = *block_index;
    *block_index += 1;
    current
}

pub(super) fn into_quoted_block(
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

pub(super) fn push_scoped_block(
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
pub(super) static NEXT_LINK_ID: AtomicU64 = AtomicU64::new(0);
