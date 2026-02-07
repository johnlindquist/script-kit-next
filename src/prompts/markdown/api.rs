use super::*;

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
