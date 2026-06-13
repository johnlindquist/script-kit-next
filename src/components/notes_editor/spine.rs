// Shared by binary-only Today wiring first; Notes will consume the same helpers
// as spine parity moves over. The library target does not currently instantiate
// those hosts, so it would otherwise warn on every helper.
#![allow(dead_code)]

use std::{collections::HashMap, ops::Range};

use crate::ai::message_parts::AiContextPart;

pub(crate) fn current_line_range(content: &str, cursor: usize) -> Range<usize> {
    let cursor = clamp_to_char_boundary(content, cursor.min(content.len()));
    let start = content[..cursor].rfind('\n').map_or(0, |idx| idx + 1);
    let end = content[cursor..]
        .find('\n')
        .map_or(content.len(), |idx| cursor + idx);
    start..end
}

pub(crate) fn clamp_to_char_boundary(text: &str, mut pos: usize) -> usize {
    pos = pos.min(text.len());
    while pos > 0 && !text.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

pub(crate) fn replace_segment_content(
    content: &str,
    line_range: Range<usize>,
    segment_byte_range: Range<usize>,
    replacement: &str,
    trailing_space: bool,
) -> Option<(String, usize)> {
    let start = line_range.start.checked_add(segment_byte_range.start)?;
    let end = line_range.start.checked_add(segment_byte_range.end)?;
    if start > end
        || end > content.len()
        || !content.is_char_boundary(start)
        || !content.is_char_boundary(end)
    {
        return None;
    }

    let suffix = &content[end..];
    let add_space = trailing_space
        && !replacement.ends_with(char::is_whitespace)
        && !suffix.starts_with(char::is_whitespace);
    let space = if add_space { " " } else { "" };
    let new_content = format!("{}{}{}{}", &content[..start], replacement, space, suffix);
    let cursor = start + replacement.len() + space.len();
    Some((new_content, cursor))
}

pub(crate) fn spine_prompt_plan_can_submit(
    parse: &crate::spine::SpineParse,
    cwd_anchor: bool,
    mention_aliases: &HashMap<String, AiContextPart>,
) -> bool {
    let plan =
        crate::spine::prompt_plan::build_spine_prompt_plan_with_aliases(parse, mention_aliases);
    plan.should_submit_to_chat()
        || (cwd_anchor
            && matches!(
                plan.blocked_reason,
                Some(
                    crate::spine::prompt_plan::SpinePromptPlanBlockReason::NoPromptBuilderSegments
                )
            )
            && plan.unknown_warnings.is_empty()
            && !plan.normalized_prompt.trim().is_empty())
}

fn single_char_deletion_index(previous: &str, next: &str) -> Option<usize> {
    let previous_chars: Vec<char> = previous.chars().collect();
    let next_chars: Vec<char> = next.chars().collect();
    if previous_chars.len() != next_chars.len() + 1 {
        return None;
    }
    let mut index = 0;
    while index < next_chars.len() && previous_chars[index] == next_chars[index] {
        index += 1;
    }
    (previous_chars[index + 1..] == next_chars[index..]).then_some(index)
}

fn byte_index_for_char_index(text: &str, char_index: usize) -> usize {
    if char_index == text.chars().count() {
        return text.len();
    }
    text.char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len())
}

pub(crate) fn mention_atomic_delete_fixup(
    previous: &str,
    next: &str,
    mention_aliases: &HashMap<String, AiContextPart>,
) -> Option<(String, usize)> {
    if mention_aliases.is_empty() {
        return None;
    }
    let deleted_char_index = single_char_deletion_index(previous, next)?;
    let deleted_registered_token = crate::ai::context_mentions::inline_token_spans(previous)
        .into_iter()
        .any(|span| {
            deleted_char_index >= span.range.start
                && deleted_char_index < span.range.end
                && mention_aliases.contains_key(&span.token)
        });
    if !deleted_registered_token {
        return None;
    }
    let (fixed, cursor_char) =
        crate::ai::context_mentions::remove_inline_mention_at_cursor_with_aliases(
            previous,
            deleted_char_index + 1,
            false,
            mention_aliases,
        )?;
    let cursor = byte_index_for_char_index(&fixed, cursor_char);
    Some((fixed, cursor))
}

pub(crate) fn prune_mention_aliases(
    mention_aliases: &mut HashMap<String, AiContextPart>,
    content: &str,
) {
    if mention_aliases.is_empty() {
        return;
    }
    let visible_tokens = crate::ai::context_mentions::inline_token_spans(content)
        .into_iter()
        .map(|span| span.token)
        .collect::<std::collections::HashSet<_>>();
    mention_aliases.retain(|token, _| visible_tokens.contains(token));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_line_parser_ignores_prior_captured_mentions() {
        let content = "captured text\nnew /rewrite";
        let cursor = content.len();
        let range = current_line_range(content, cursor);
        assert_eq!(&content[range], "new /rewrite");
    }

    #[test]
    fn current_line_parser_targets_non_final_active_line() {
        let content = "first /rewrite\nmiddle .professional\nlast ;todo";
        let cursor = content.find("professional").expect("active query exists") + "pro".len();
        let range = current_line_range(content, cursor);
        assert_eq!(&content[range], "middle .professional");
    }

    #[test]
    fn current_line_parser_clamps_unicode_cursor_to_char_boundary() {
        let content = "emoji é /rewrite\nnext";
        let cursor_inside_e_acute = "emoji ".len() + 1;
        let range = current_line_range(content, cursor_inside_e_acute);
        assert_eq!(&content[range], "emoji é /rewrite");
    }

    #[test]
    fn current_line_parser_handles_blank_active_line() {
        let content = "above\n\nbelow /rewrite";
        let cursor = "above\n".len();
        let range = current_line_range(content, cursor);
        assert_eq!(&content[range], "");
    }

    #[test]
    fn replace_segment_content_preserves_surrounding_lines() {
        let content = "captured old\nnew /rew\nnext line";
        let line_start = content.find("new ").expect("line exists");
        let line_range = line_start.."captured old\nnew /rew".len();
        let segment_start = "new ".len();
        let segment_end = segment_start + "/rew".len();
        let (new_content, cursor) = replace_segment_content(
            content,
            line_range,
            segment_start..segment_end,
            "/rewrite",
            false,
        )
        .expect("replacement should fit current line");

        assert_eq!(new_content, "captured old\nnew /rewrite\nnext line");
        assert_eq!(cursor, "captured old\nnew /rewrite".len());
    }

    #[test]
    fn replace_segment_content_adds_trailing_space_when_needed() {
        let content = "ask /rew";
        let line_range = 0..content.len();
        let segment_start = "ask ".len();
        let segment_end = segment_start + "/rew".len();
        let (new_content, cursor) = replace_segment_content(
            content,
            line_range,
            segment_start..segment_end,
            "/rewrite",
            true,
        )
        .expect("replacement should fit");

        assert_eq!(new_content, "ask /rewrite ");
        assert_eq!(cursor, "ask /rewrite ".len());
    }

    fn test_text_block_part(label: &str) -> AiContextPart {
        AiContextPart::TextBlock {
            label: label.to_string(),
            source: format!("test:{label}"),
            text: format!("{label} body"),
            mime_type: None,
        }
    }

    #[test]
    fn alias_backed_token_deletes_atomically_and_consumes_space() {
        let mut aliases = HashMap::new();
        aliases.insert(
            "@clipboard:Latest".to_string(),
            test_text_block_part("Latest"),
        );

        let fixed = mention_atomic_delete_fixup(
            "ask @clipboard:Latest now",
            "ask @clipboard:Lates now",
            &aliases,
        )
        .expect("registered token should delete atomically");

        assert_eq!(fixed, ("ask now".to_string(), "ask ".len()));
    }

    #[test]
    fn unresolved_subsearch_token_keeps_normal_character_delete() {
        let mut aliases = HashMap::new();
        aliases.insert(
            "@clipboard:Latest".to_string(),
            test_text_block_part("Latest"),
        );

        assert_eq!(
            mention_atomic_delete_fixup("ask @file:readme now", "ask @file:readm now", &aliases),
            None
        );
    }

    #[test]
    fn prune_aliases_drops_tokens_no_longer_visible() {
        let mut aliases = HashMap::new();
        aliases.insert(
            "@clipboard:Latest".to_string(),
            test_text_block_part("Latest"),
        );
        aliases.insert("@file:demo.rs".to_string(), test_text_block_part("demo.rs"));

        prune_mention_aliases(&mut aliases, "ask @file:demo.rs");

        assert!(!aliases.contains_key("@clipboard:Latest"));
        assert!(aliases.contains_key("@file:demo.rs"));
    }

    #[test]
    fn set_input_prune_boundary_uses_inline_token_spans_not_substrings() {
        let mut aliases = HashMap::new();
        aliases.insert(
            "@clipboard:Latest".to_string(),
            test_text_block_part("Latest"),
        );

        prune_mention_aliases(&mut aliases, "literal @clipboard:Latest-ish");

        assert!(aliases.is_empty());
    }

    #[test]
    fn cmd_enter_preflight_rejects_plain_text_without_cwd_anchor() {
        let parse = crate::spine::parse_spine("summarize this folder");
        let aliases = HashMap::new();

        assert!(!spine_prompt_plan_can_submit(&parse, false, &aliases));
    }

    #[test]
    fn cmd_enter_preflight_allows_plain_text_with_cwd_anchor() {
        let parse = crate::spine::parse_spine("summarize this folder");
        let aliases = HashMap::new();

        assert!(spine_prompt_plan_can_submit(&parse, true, &aliases));
    }
}
