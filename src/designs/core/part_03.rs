/// Detect why a script matched the search query when the name didn't match directly.
/// Returns a concise reason string (e.g., "tag: productivity", "shortcut") for
/// display in the search source hint area, helping users understand search results.
#[cfg(test)]
pub(crate) fn detect_match_reason_for_script(
    script: &crate::scripts::Script,
    query: &str,
) -> Option<String> {
    if query.len() < 2 {
        return None;
    }
    let q = query.to_lowercase();

    // If name already matches, no need for a "via" indicator
    if crate::scripts::search::contains_ignore_ascii_case(&script.name, &q) {
        return None;
    }

    // Check metadata fields in priority order
    if let Some(ref meta) = script.typed_metadata {
        // Tags
        for tag in &meta.tags {
            if crate::scripts::search::contains_ignore_ascii_case(tag, &q) {
                return Some(format!("tag: {}", tag));
            }
        }
        // Author
        if let Some(ref author) = meta.author {
            if crate::scripts::search::contains_ignore_ascii_case(author, &q) {
                return Some(format!("by {}", author));
            }
        }
    }

    // Shortcut
    if let Some(ref shortcut) = script.shortcut {
        if crate::scripts::search::contains_ignore_ascii_case(shortcut, &q) {
            return Some("shortcut".to_string());
        }
    }

    // Kit name
    if let Some(ref kit) = script.kit_name {
        if kit != "main" && crate::scripts::search::contains_ignore_ascii_case(kit, &q) {
            return Some(format!("kit: {}", kit));
        }
    }

    // Alias (when not shown as badge - if shortcut exists, alias isn't the badge)
    if let Some(ref alias) = script.alias {
        if crate::scripts::search::contains_ignore_ascii_case(alias, &q) {
            return Some(format!("alias: /{}", alias));
        }
    }

    // Description excerpt - show brief matching context when description matched
    if let Some(ref desc) = script.description {
        if crate::scripts::search::contains_ignore_ascii_case(desc, &q) {
            let excerpt = excerpt_around_match(desc, &q, 40);
            return Some(format!("desc: {}", excerpt));
        }
    }

    // Path match - when path matched but nothing else above did
    if crate::scripts::search::contains_ignore_ascii_case(&script.path.to_string_lossy(), &q) {
        return Some("path match".to_string());
    }

    None
}

/// Extract a brief excerpt from text around the first match of a query.
/// Returns a truncated substring centered on the match, with ellipsis as needed.
/// `max_len` is the maximum character length of the returned excerpt.
#[cfg(test)]
pub(crate) fn excerpt_around_match(text: &str, query_lower: &str, max_len: usize) -> String {
    let text_chars: Vec<char> = text.chars().collect();
    let text_len = text_chars.len();

    if text_len <= max_len {
        return text.to_string();
    }

    // Find the match position (char-level search via lowercased text)
    let text_lower: String = text_chars.iter().map(|c| c.to_ascii_lowercase()).collect();
    let match_byte_pos = text_lower.find(query_lower).unwrap_or(0);
    // Convert byte position to char position
    let char_pos = text_lower[..match_byte_pos.min(text_lower.len())]
        .chars()
        .count();

    // Center the excerpt around the match
    let half = max_len / 2;
    let start = char_pos.saturating_sub(half);
    let end = (start + max_len).min(text_len);
    let start = if end == text_len && text_len > max_len {
        text_len - max_len
    } else {
        start
    };

    let excerpt: String = text_chars[start..end].iter().collect();
    if start > 0 && end < text_len {
        format!("...{}...", excerpt.trim())
    } else if start > 0 {
        format!("...{}", excerpt.trim())
    } else if end < text_len {
        format!("{}...", excerpt.trim())
    } else {
        excerpt
    }
}

/// Detect why a scriptlet matched the search query when the name didn't match directly.
/// Returns a concise reason string for display in search source hints.
#[cfg(test)]
pub(crate) fn detect_match_reason_for_scriptlet(
    scriptlet: &crate::scripts::Scriptlet,
    query: &str,
) -> Option<String> {
    if query.len() < 2 {
        return None;
    }
    let q = query.to_lowercase();

    // If name already matches, no need for indicator
    if crate::scripts::search::contains_ignore_ascii_case(&scriptlet.name, &q) {
        return None;
    }

    // Keyword
    if let Some(ref keyword) = scriptlet.keyword {
        if crate::scripts::search::contains_ignore_ascii_case(keyword, &q) {
            return Some(format!("keyword: {}", keyword));
        }
    }

    // Shortcut
    if let Some(ref shortcut) = scriptlet.shortcut {
        if crate::scripts::search::contains_ignore_ascii_case(shortcut, &q) {
            return Some("shortcut".to_string());
        }
    }

    // Group
    if let Some(ref group) = scriptlet.group {
        if group != "main" && crate::scripts::search::contains_ignore_ascii_case(group, &q) {
            return Some(format!("group: {}", group));
        }
    }

    // Alias
    if let Some(ref alias) = scriptlet.alias {
        if crate::scripts::search::contains_ignore_ascii_case(alias, &q) {
            return Some(format!("alias: /{}", alias));
        }
    }

    // Tool type (e.g., searching "bash" finds bash scriptlets)
    if crate::scripts::search::contains_ignore_ascii_case(&scriptlet.tool, &q) {
        return Some(format!("tool: {}", scriptlet.tool_display_name()));
    }

    // Description excerpt
    if let Some(ref desc) = scriptlet.description {
        if crate::scripts::search::contains_ignore_ascii_case(desc, &q) {
            let excerpt = excerpt_around_match(desc, &q, 35);
            return Some(format!("desc: {}", excerpt));
        }
    }

    // Code content (only for longer queries to avoid noise)
    if q.len() >= 4 && crate::scripts::search::contains_ignore_ascii_case(&scriptlet.code, &q) {
        return Some("code match".to_string());
    }

    None
}

