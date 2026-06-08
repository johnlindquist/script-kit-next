use std::collections::HashSet;
use std::ops::Range;

use gpui::SharedString;

use super::list::{SpineListAction, SpineListRow, SpineListRowKind};

const MAX_HISTORY_HITS: usize = 12;
const MAX_PROMPT_ROWS: usize = 5;
const MAX_CONVERSATION_ROWS: usize = 5;

pub(crate) fn build_recent_prompt_rows(
    tail_query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Vec<SpineListRow> {
    let hits = history_hits_for_tail(tail_query);
    let mut rows = Vec::new();
    let mut seen_prompts = HashSet::new();

    for hit in hits {
        let entry = &hit.entry;
        let prompt = entry.first_message.trim();
        if prompt.is_empty() {
            continue;
        }
        let dedupe_key = prompt.to_ascii_lowercase();
        if !seen_prompts.insert(dedupe_key) {
            continue;
        }
        rows.push(SpineListRow {
            id: SharedString::from(format!(
                "spine:tail:prompt:{}",
                stable_tail_row_hash(prompt)
            )),
            kind: SpineListRowKind::RecentPrompt {
                prompt_id: SharedString::from(format!("{}", stable_tail_row_hash(prompt))),
            },
            title: SharedString::from(single_line_truncate(prompt, 96)),
            subtitle: Some(SharedString::from(format!(
                "Use recent prompt from {}",
                entry.title_display()
            ))),
            icon: Some(SharedString::from("history")),
            meta: Some(SharedString::from(message_count_label(entry.message_count))),
            badges: vec![],
            score: hit.score as i32,
            is_selectable: true,
            action_label: Some(SharedString::from("Use")),
            action: SpineListAction::InsertSegmentText {
                segment_index,
                segment_byte_range: segment_byte_range.clone(),
                text: SharedString::from(prompt.to_string()),
                trailing_space: false,
            },
        });
        if rows.len() >= MAX_PROMPT_ROWS {
            break;
        }
    }

    if rows.is_empty() {
        rows.push(empty_row(
            "spine:tail:prompt:empty",
            if tail_query.trim().is_empty() {
                "No recent prompts yet"
            } else {
                "No matching recent prompts"
            },
            "Prompt history will appear here after Agent Chat conversations exist.",
        ));
    }
    rows
}

pub(crate) fn build_conversation_rows(tail_query: &str) -> Vec<SpineListRow> {
    let hits = history_hits_for_tail(tail_query);
    let mut rows = Vec::new();

    for hit in hits.into_iter().take(MAX_CONVERSATION_ROWS) {
        let entry = hit.entry;
        let title = single_line_truncate(entry.title_display(), 96);
        let subtitle = conversation_subtitle(&entry);
        rows.push(SpineListRow {
            id: SharedString::from(format!("spine:tail:conversation:{}", entry.session_id)),
            kind: SpineListRowKind::Conversation {
                conversation_id: SharedString::from(entry.session_id.clone()),
            },
            title: SharedString::from(title),
            subtitle: Some(SharedString::from(subtitle)),
            icon: Some(SharedString::from("message-circle")),
            meta: Some(SharedString::from("Resume")),
            badges: vec![],
            score: hit.score as i32,
            is_selectable: true,
            action_label: Some(SharedString::from("Resume")),
            action: SpineListAction::OpenConversation {
                conversation_id: SharedString::from(entry.session_id),
            },
        });
    }

    if rows.is_empty() {
        rows.push(empty_row(
            "spine:tail:conversation:empty",
            if tail_query.trim().is_empty() {
                "No resumable conversations yet"
            } else {
                "No matching conversations"
            },
            "Past Agent Chat conversations will appear here.",
        ));
    }
    rows
}

fn history_hits_for_tail(
    tail_query: &str,
) -> Vec<crate::ai::agent_chat::ui::history::AgentChatHistorySearchHit> {
    let query = tail_query.trim();
    let cached = crate::ai::agent_chat::ui::history::search_history_cached(query, MAX_HISTORY_HITS);
    if !cached.is_empty() || !query.is_empty() {
        return cached;
    }
    crate::ai::agent_chat::ui::history::search_history_direct(query, MAX_HISTORY_HITS)
}

fn conversation_subtitle(
    entry: &crate::ai::agent_chat::ui::history::AgentChatHistoryEntry,
) -> String {
    let preview = single_line_truncate(entry.preview.trim(), 88);
    let count = message_count_label(entry.message_count);
    if preview.is_empty() {
        count
    } else {
        format!("{preview} · {count}")
    }
}

fn message_count_label(count: usize) -> String {
    match count {
        1 => "1 message".to_string(),
        n => format!("{n} messages"),
    }
}

fn empty_row(id: &'static str, title: &'static str, subtitle: &'static str) -> SpineListRow {
    SpineListRow {
        id: SharedString::from(id),
        kind: SpineListRowKind::Empty,
        title: SharedString::from(title),
        subtitle: Some(SharedString::from(subtitle)),
        icon: Some(SharedString::from("info")),
        meta: None,
        badges: vec![],
        score: 0,
        is_selectable: false,
        action_label: None,
        action: SpineListAction::Noop,
    }
}

fn single_line_truncate(input: &str, max_chars: usize) -> String {
    super::text_preview::single_line_truncate(input, max_chars)
}

fn stable_tail_row_hash(value: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
