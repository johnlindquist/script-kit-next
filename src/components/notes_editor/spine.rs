// Shared by binary-only Today wiring first; Notes will consume the same helpers
// as spine parity moves over. The library target does not currently instantiate
// those hosts, so it would otherwise warn on every helper.
#![allow(dead_code)]

use std::{collections::HashMap, ops::Range};

use crate::ai::message_parts::AiContextPart;

#[derive(Debug)]
pub(crate) struct NotesEditorSpineRuntime<Flat> {
    pub(crate) selected_index: usize,
    pub(crate) hovered_index: Option<usize>,
    pub(crate) cache_key: String,
    pub(crate) cwd_revision: u64,
    pub(crate) cwd_submit_anchor: bool,
    pub(crate) dismissed_cache_key: Option<String>,
    pub(crate) mention_aliases: HashMap<String, AiContextPart>,
    pub(crate) grouped_cache: Vec<crate::list_item::GroupedListItem>,
    pub(crate) flat_cache: Vec<Flat>,
    pub(crate) alias_cache: HashMap<String, (String, AiContextPart)>,
}

impl<Flat> Default for NotesEditorSpineRuntime<Flat> {
    fn default() -> Self {
        Self {
            selected_index: 0,
            hovered_index: None,
            cache_key: String::new(),
            cwd_revision: 0,
            cwd_submit_anchor: false,
            dismissed_cache_key: None,
            mention_aliases: HashMap::new(),
            grouped_cache: Vec::new(),
            flat_cache: Vec::new(),
            alias_cache: HashMap::new(),
        }
    }
}

impl<Flat> NotesEditorSpineRuntime<Flat> {
    pub(crate) fn reset(&mut self, clear_cwd_anchor: bool, clear_mentions: bool) {
        self.selected_index = 0;
        self.hovered_index = None;
        if clear_cwd_anchor {
            self.cwd_submit_anchor = false;
        }
        self.dismissed_cache_key = None;
        if clear_mentions {
            self.mention_aliases.clear();
        }
        self.cache_key.clear();
        self.grouped_cache.clear();
        self.flat_cache.clear();
        self.alias_cache.clear();
    }

    pub(crate) fn dismiss_current_key(&mut self, key: Option<String>) {
        self.dismissed_cache_key = key;
        self.selected_index = 0;
        self.hovered_index = None;
    }

    pub(crate) fn clear_alias_cache(&mut self) {
        self.alias_cache.clear();
    }

    pub(crate) fn register_mention_alias(&mut self, token: String, part: AiContextPart) {
        self.mention_aliases.insert(token, part);
    }

    pub(crate) fn prune_mention_aliases_for_content(&mut self, content: &str) {
        prune_mention_aliases(&mut self.mention_aliases, content);
    }

    pub(crate) fn clear_transient_cache(&mut self) {
        self.dismissed_cache_key = None;
        self.alias_cache.clear();
    }

    pub(crate) fn coerce_selection_for_cached_rows(&mut self) {
        self.selected_index =
            crate::list_item::coerce_selection(&self.grouped_cache, self.selected_index)
                .unwrap_or(0);
    }

    pub(crate) fn replace_cached_rows(
        &mut self,
        key: String,
        grouped: Vec<crate::list_item::GroupedListItem>,
        flat: Vec<Flat>,
        aliases: HashMap<String, (String, AiContextPart)>,
    ) where
        Flat: Clone,
    {
        self.cache_key = key;
        self.grouped_cache = grouped;
        self.flat_cache = flat;
        self.alias_cache = aliases;
        self.coerce_selection_for_cached_rows();
    }
}

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
    fn runtime_reset_clears_transient_state_and_optionally_mentions() {
        let mut runtime = NotesEditorSpineRuntime::<usize>::default();
        runtime.selected_index = 3;
        runtime.hovered_index = Some(2);
        runtime.cache_key = "key".to_string();
        runtime.cwd_submit_anchor = true;
        runtime.dismissed_cache_key = Some("dismissed".to_string());
        runtime.grouped_cache = vec![crate::list_item::GroupedListItem::Item(0)];
        runtime.flat_cache = vec![1];
        runtime.alias_cache.insert(
            "row".to_string(),
            (
                "@clipboard:Latest".to_string(),
                test_text_block_part("Latest"),
            ),
        );
        runtime.register_mention_alias(
            "@clipboard:Latest".to_string(),
            test_text_block_part("Latest"),
        );

        runtime.reset(false, false);

        assert_eq!(runtime.selected_index, 0);
        assert_eq!(runtime.hovered_index, None);
        assert_eq!(runtime.cache_key, "");
        assert!(runtime.cwd_submit_anchor);
        assert!(runtime.dismissed_cache_key.is_none());
        assert!(runtime.grouped_cache.is_empty());
        assert!(runtime.flat_cache.is_empty());
        assert!(runtime.alias_cache.is_empty());
        assert!(runtime.mention_aliases.contains_key("@clipboard:Latest"));

        runtime.reset(true, true);

        assert!(!runtime.cwd_submit_anchor);
        assert!(runtime.mention_aliases.is_empty());
    }

    #[test]
    fn runtime_prunes_mentions_using_inline_token_spans() {
        let mut runtime = NotesEditorSpineRuntime::<usize>::default();
        runtime.register_mention_alias(
            "@clipboard:Latest".to_string(),
            test_text_block_part("Latest"),
        );

        runtime.prune_mention_aliases_for_content("literal @clipboard:Latest-ish");

        assert!(runtime.mention_aliases.is_empty());
    }

    #[test]
    fn runtime_dismisses_only_the_current_cache_key() {
        let mut runtime = NotesEditorSpineRuntime::<usize>::default();

        runtime.dismiss_current_key(Some("alpha".to_string()));

        assert_eq!(runtime.dismissed_cache_key.as_deref(), Some("alpha"));
        assert_ne!(runtime.dismissed_cache_key.as_deref(), Some("beta"));
        assert_eq!(runtime.selected_index, 0);
        assert_eq!(runtime.hovered_index, None);
    }

    #[test]
    fn runtime_cache_replacement_coerces_selected_index_to_item() {
        let mut runtime = NotesEditorSpineRuntime::<usize>::default();
        runtime.selected_index = 0;

        runtime.replace_cached_rows(
            "key".to_string(),
            vec![
                crate::list_item::GroupedListItem::SectionHeader("Suggested".to_string(), None),
                crate::list_item::GroupedListItem::Item(0),
            ],
            vec![42],
            HashMap::new(),
        );

        assert_eq!(runtime.cache_key, "key");
        assert_eq!(runtime.selected_index, 1);
        assert_eq!(runtime.flat_cache, vec![42]);
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
