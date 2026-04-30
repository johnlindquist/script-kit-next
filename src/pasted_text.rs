use std::ops::Range;

const PASTED_TEXT_LINE_THRESHOLD: usize = 8;
const PASTED_TEXT_CHAR_THRESHOLD: usize = 600;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PastedTextToken {
    pub(crate) token: String,
    pub(crate) label: String,
    pub(crate) text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PastedTextTokenRange {
    pub(crate) range: Range<usize>,
    pub(crate) token: String,
    pub(crate) label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreparedPastedText {
    pub(crate) insertion_text: String,
    pub(crate) token: Option<PastedTextToken>,
}

pub(crate) fn prepare_pasted_text(
    text: &str,
    existing_tokens: &[PastedTextToken],
) -> PreparedPastedText {
    if !should_collapse_pasted_text(text) {
        return PreparedPastedText {
            insertion_text: text.to_string(),
            token: None,
        };
    }

    let label = build_pasted_text_label(text, next_token_index(existing_tokens));
    let token = format!("@text:\"{label}\"");

    PreparedPastedText {
        insertion_text: format!("{token} "),
        token: Some(PastedTextToken {
            token,
            label,
            text: text.to_string(),
        }),
    }
}

pub(crate) fn preview_description_for_token(token: &str) -> Option<String> {
    let (prefix, value) = crate::ai::context_mentions::typed_mention_token_parts(token)?;
    if prefix != "text" {
        return None;
    }
    let value = value.trim();
    if !value.starts_with("Pasted text #") {
        return None;
    }
    Some(format!("Pasted text attachment ({value})"))
}

pub(crate) fn expand_pasted_text_tokens(text: &str, tokens: &[PastedTextToken]) -> String {
    let mut expanded = text.to_string();
    let mut ranges = token_ranges(&expanded, tokens);
    ranges.sort_by_key(|entry| entry.range.start);

    for entry in ranges.into_iter().rev() {
        let start = char_to_byte_offset(&expanded, entry.range.start);
        let end = char_to_byte_offset(&expanded, entry.range.end);
        let Some(token) = tokens.iter().find(|token| token.token == entry.token) else {
            continue;
        };
        expanded.replace_range(start..end, &token.text);
    }

    expanded
}

pub(crate) fn token_ranges(text: &str, tokens: &[PastedTextToken]) -> Vec<PastedTextTokenRange> {
    let mut ranges = Vec::new();

    for token in tokens {
        for (byte_start, _) in text.match_indices(&token.token) {
            let start = text[..byte_start].chars().count();
            let end = start + token.token.chars().count();
            ranges.push(PastedTextTokenRange {
                range: start..end,
                token: token.token.clone(),
                label: token.label.clone(),
            });
        }
    }

    ranges.sort_by_key(|entry| entry.range.start);
    ranges
}

pub(crate) fn sync_pasted_text_tokens(tokens: &mut Vec<PastedTextToken>, text: &str) {
    tokens.retain(|token| text.contains(&token.token));
}

pub(crate) fn remove_pasted_text_token_at_cursor(
    text: &str,
    cursor: usize,
    delete_forward: bool,
    tokens: &mut Vec<PastedTextToken>,
) -> Option<(String, usize)> {
    let range = token_range_for_atomic_delete(text, cursor, delete_forward, tokens)?;
    let start = char_to_byte_offset(text, range.start);
    let mut end = char_to_byte_offset(text, range.end);

    if text[end..].starts_with(' ') {
        end += ' '.len_utf8();
    }

    let mut next = String::with_capacity(text.len().saturating_sub(end.saturating_sub(start)));
    next.push_str(&text[..start]);
    next.push_str(&text[end..]);
    sync_pasted_text_tokens(tokens, &next);
    Some((next, range.start))
}

fn token_range_for_atomic_delete(
    text: &str,
    cursor: usize,
    delete_forward: bool,
    tokens: &[PastedTextToken],
) -> Option<Range<usize>> {
    let ranges = token_ranges(text, tokens);

    if delete_forward {
        ranges
            .iter()
            .find(|entry| cursor >= entry.range.start && cursor < entry.range.end)
            .map(|entry| entry.range.clone())
            .or_else(|| {
                ranges
                    .iter()
                    .find(|entry| cursor == entry.range.start)
                    .map(|entry| entry.range.clone())
            })
    } else {
        ranges
            .iter()
            .find(|entry| cursor > entry.range.start && cursor <= entry.range.end)
            .map(|entry| entry.range.clone())
    }
}

fn should_collapse_pasted_text(text: &str) -> bool {
    let line_count = text.lines().count().max(1);
    let char_count = text.chars().count();
    line_count >= PASTED_TEXT_LINE_THRESHOLD || char_count >= PASTED_TEXT_CHAR_THRESHOLD
}

fn build_pasted_text_label(text: &str, index: usize) -> String {
    let line_count = text.lines().count().max(1);
    let char_count = text.chars().count();

    if line_count >= PASTED_TEXT_LINE_THRESHOLD {
        format!("Pasted text #{index} +{line_count} lines")
    } else {
        format!("Pasted text #{index} +{char_count} chars")
    }
}

fn next_token_index(existing_tokens: &[PastedTextToken]) -> usize {
    existing_tokens.len() + 1
}

fn char_to_byte_offset(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::{
        expand_pasted_text_tokens, prepare_pasted_text, preview_description_for_token,
        remove_pasted_text_token_at_cursor, sync_pasted_text_tokens,
    };

    #[test]
    fn prepare_pasted_text_keeps_small_pastes_inline() {
        let prepared = prepare_pasted_text("short note", &[]);
        assert_eq!(prepared.insertion_text, "short note");
        assert!(prepared.token.is_none());
    }

    #[test]
    fn prepare_pasted_text_collapses_large_multiline_paste() {
        let text = (0..12)
            .map(|ix| format!("line {ix}"))
            .collect::<Vec<_>>()
            .join("\n");
        let prepared = prepare_pasted_text(&text, &[]);
        let token = prepared.token.expect("large paste should collapse");

        assert_eq!(token.label, "Pasted text #1 +12 lines");
        assert_eq!(
            prepared.insertion_text,
            "@text:\"Pasted text #1 +12 lines\" "
        );
        assert_eq!(token.text, text);
    }

    #[test]
    fn expand_pasted_text_tokens_restores_original_text() {
        let text = "first\nsecond\nthird\nfourth\nfifth\nsixth\nseventh\neighth";
        let prepared = prepare_pasted_text(text, &[]);
        let token = prepared.token.expect("collapsed token");
        let expanded = expand_pasted_text_tokens(&prepared.insertion_text, &[token]);
        assert_eq!(expanded.trim_end(), text);
    }

    #[test]
    fn remove_pasted_text_token_at_cursor_deletes_whole_token_and_space() {
        let text = (0..10)
            .map(|ix| format!("row {ix}"))
            .collect::<Vec<_>>()
            .join("\n");
        let prepared = prepare_pasted_text(&text, &[]);
        let token = prepared.token.expect("collapsed token");
        let mut tokens = vec![token];
        let input = format!("before {}after", prepared.insertion_text);
        let cursor = "before @text:\"Pasted text #1 +10 lines\"".chars().count();
        let (next, next_cursor) =
            remove_pasted_text_token_at_cursor(&input, cursor, false, &mut tokens)
                .expect("token should delete atomically");

        assert_eq!(next, "before after");
        assert_eq!(next_cursor, "before ".chars().count());
        assert!(tokens.is_empty());
    }

    #[test]
    fn sync_pasted_text_tokens_drops_deleted_tokens() {
        let text = (0..9)
            .map(|ix| format!("line {ix}"))
            .collect::<Vec<_>>()
            .join("\n");
        let prepared = prepare_pasted_text(&text, &[]);
        let token = prepared.token.expect("collapsed token");
        let mut tokens = vec![token];

        sync_pasted_text_tokens(&mut tokens, "plain text");
        assert!(tokens.is_empty());
    }

    #[test]
    fn preview_description_detects_pasted_text_tokens() {
        assert_eq!(
            preview_description_for_token("@text:\"Pasted text #1 +12 lines\""),
            Some("Pasted text attachment (Pasted text #1 +12 lines)".to_string())
        );
        assert_eq!(preview_description_for_token("@text:\"Inline note\""), None);
    }
}
