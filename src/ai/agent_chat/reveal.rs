//! Word-paced reveal chunking for streamed agent text.
//!
//! Engines deliver assistant and reasoning text in provider-sized deltas. The
//! Agent Chat transcript reveals text a word (or a whole markdown structural
//! line) at a time so a large delta does not land as a single jump. The
//! splitter never alters bytes: joining the chunks reproduces the delta.

pub(crate) const REVEAL_MAX_UNBROKEN_CHARS: usize = 32;

/// Pause between consecutive reveal chunks of one delta.
pub(crate) const REVEAL_CHUNK_DELAY_MS: u64 = 6;

pub(crate) fn split_text_delta_for_reveal(delta: &str) -> Vec<String> {
    if delta.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut line_start = 0;

    for (idx, ch) in delta.char_indices() {
        if ch == '\n' {
            push_line_reveal_chunks(&delta[line_start..idx + ch.len_utf8()], &mut chunks);
            line_start = idx + ch.len_utf8();
        }
    }

    if line_start < delta.len() {
        push_line_reveal_chunks(&delta[line_start..], &mut chunks);
    }

    if cfg!(debug_assertions) {
        let joined = chunks.concat();
        if joined != delta {
            tracing::warn!(
                target: "script_kit::agent_chat",
                event = "reveal_chunk_mismatch",
                delta_len = delta.len(),
                chunks_len = chunks.len(),
                joined_len = joined.len(),
            );
            chunks.clear();
            if !delta.is_empty() {
                chunks.push(delta.to_string());
            }
        }
        chunks.retain(|c| !c.is_empty());
    }
    chunks
}

fn push_line_reveal_chunks(line: &str, chunks: &mut Vec<String>) {
    if line.is_empty() {
        return;
    }
    if line.chars().all(char::is_whitespace) {
        if let Some(previous) = chunks.last_mut() {
            previous.push_str(line);
        } else {
            chunks.push(line.to_string());
        }
        return;
    }

    if is_markdown_fence_line(line) || is_markdown_table_line(line) {
        chunks.push(line.to_string());
        return;
    }

    let protected_prefix_len = markdown_structural_prefix_len(line).unwrap_or(0);
    push_word_reveal_chunks(line, chunks, protected_prefix_len);
}

fn is_markdown_fence_line(line: &str) -> bool {
    let leading_spaces = line.chars().take_while(|ch| *ch == ' ').count();
    if leading_spaces > 3 {
        return false;
    }
    let trimmed = &line[leading_spaces..];
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

fn is_markdown_table_line(line: &str) -> bool {
    line.trim_start().starts_with('|')
}

fn markdown_structural_prefix_len(line: &str) -> Option<usize> {
    let leading_len = line
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(idx, _)| idx)
        .unwrap_or(line.len());
    let rest = &line[leading_len..];
    if rest.is_empty() {
        return None;
    }

    if let Some(len) = markdown_heading_prefix_len(rest)
        .or_else(|| markdown_blockquote_prefix_len(rest))
        .or_else(|| markdown_task_prefix_len(rest))
        .or_else(|| markdown_list_prefix_len(rest))
    {
        return Some(leading_len + len);
    }

    (leading_len > 0).then_some(leading_len)
}

fn markdown_heading_prefix_len(rest: &str) -> Option<usize> {
    let hash_len = rest.chars().take_while(|ch| *ch == '#').count();
    if (1..=6).contains(&hash_len) && rest.as_bytes().get(hash_len) == Some(&b' ') {
        Some(hash_len + 1)
    } else {
        None
    }
}

fn markdown_blockquote_prefix_len(rest: &str) -> Option<usize> {
    let mut len = 0;
    let bytes = rest.as_bytes();
    while bytes.get(len) == Some(&b'>') {
        len += 1;
    }
    if len == 0 {
        return None;
    }
    if bytes.get(len) == Some(&b' ') {
        len += 1;
    }
    Some(len)
}

fn markdown_task_prefix_len(rest: &str) -> Option<usize> {
    for marker in ["- [ ] ", "- [x] ", "- [X] ", "* [ ] ", "* [x] ", "* [X] "] {
        if rest.starts_with(marker) {
            return Some(marker.len());
        }
    }
    None
}

fn markdown_list_prefix_len(rest: &str) -> Option<usize> {
    for marker in ["- ", "* ", "+ "] {
        if rest.starts_with(marker) {
            return Some(marker.len());
        }
    }

    let mut digit_end = 0;
    for (idx, ch) in rest.char_indices() {
        if ch.is_ascii_digit() {
            digit_end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    if digit_end > 0
        && matches!(rest.as_bytes().get(digit_end), Some(b'.' | b')'))
        && rest.as_bytes().get(digit_end + 1) == Some(&b' ')
    {
        return Some(digit_end + 2);
    }

    None
}

fn push_word_reveal_chunks(line: &str, chunks: &mut Vec<String>, protected_prefix_len: usize) {
    let mut pos = protected_prefix_len.min(line.len());
    let mut prefix_pending = protected_prefix_len > 0;

    while pos < line.len() {
        if let Some((offset, _)) = line[pos..]
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace())
        {
            let word_start = pos + offset;
            if word_start > pos && !prefix_pending {
                chunks.push(line[pos..word_start].to_string());
            }

            let word_end = line[word_start..]
                .char_indices()
                .find(|(_, ch)| ch.is_whitespace())
                .map(|(idx, _)| word_start + idx)
                .unwrap_or(line.len());
            let whitespace_end = line[word_end..]
                .char_indices()
                .find(|(_, ch)| !ch.is_whitespace())
                .map(|(idx, _)| word_end + idx)
                .unwrap_or(line.len());
            let prefix = if prefix_pending {
                &line[..protected_prefix_len]
            } else {
                ""
            };
            push_word_with_suffix(
                prefix,
                &line[word_start..word_end],
                &line[word_end..whitespace_end],
                chunks,
            );
            prefix_pending = false;
            pos = whitespace_end;
        } else {
            if pos < line.len() && !prefix_pending {
                chunks.push(line[pos..].to_string());
            }
            break;
        }
    }
}

fn push_word_with_suffix(prefix: &str, word: &str, suffix: &str, chunks: &mut Vec<String>) {
    if word.chars().count() <= REVEAL_MAX_UNBROKEN_CHARS {
        chunks.push(format!("{prefix}{word}{suffix}"));
        return;
    }

    let mut current = String::new();
    current.push_str(prefix);
    let mut current_chars = 0usize;
    for ch in word.chars() {
        current.push(ch);
        current_chars += 1;
        if current_chars >= REVEAL_MAX_UNBROKEN_CHARS {
            chunks.push(std::mem::take(&mut current));
            current_chars = 0;
        }
    }
    current.push_str(suffix);
    if !current.is_empty() {
        chunks.push(current);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_delta_splits_for_reveal_without_losing_whitespace() {
        let delta = "Hello world\n\nNext step";
        let chunks = split_text_delta_for_reveal(delta);

        assert_eq!(chunks, vec!["Hello ", "world\n\n", "Next ", "step"]);
        assert_eq!(chunks.concat(), delta);
    }

    #[test]
    fn text_delta_splits_preserve_indentation_trailing_spaces_and_blank_lines() {
        let delta = "  - first item  \n\n    indented next  ";
        let chunks = split_text_delta_for_reveal(delta);

        assert_eq!(chunks.concat(), delta);
        assert!(chunks.iter().all(|chunk| !chunk.is_empty()));
    }

    #[test]
    fn text_delta_splits_code_fence_lines_atomically() {
        let delta = "```ts\nconst value = 1;\n```\n";
        let chunks = split_text_delta_for_reveal(delta);

        assert_eq!(chunks.concat(), delta);
        assert_eq!(chunks.first().map(String::as_str), Some("```ts\n"));
        assert!(chunks.iter().any(|chunk| chunk == "```\n"));
        assert!(chunks.iter().all(|chunk| !chunk.is_empty()));
    }

    #[test]
    fn text_delta_attaches_markdown_prefixes_to_first_content() {
        let delta = "# Heading one\n> quoted text\n- [ ] task item\n indented code\n";
        let chunks = split_text_delta_for_reveal(delta);

        assert_eq!(chunks.concat(), delta);
        assert!(chunks.iter().all(|chunk| !chunk.is_empty()));
        for bad in ["# ", "> ", "- [ ] ", " "] {
            assert!(
                !chunks.iter().any(|chunk| chunk == bad),
                "must not emit standalone markdown prefix chunk {bad:?}: {chunks:?}"
            );
        }
        assert!(chunks.iter().any(|chunk| chunk.starts_with("# Heading")));
        assert!(chunks.iter().any(|chunk| chunk.starts_with("> quoted")));
        assert!(chunks.iter().any(|chunk| chunk.starts_with("- [ ] task")));
        assert!(chunks.iter().any(|chunk| chunk.starts_with(" indented")));
    }

    #[test]
    fn text_delta_splits_markdown_tables_by_line() {
        let delta = "| Name | Value |\n| --- | --- |\n| Foo | Bar |\n";
        let chunks = split_text_delta_for_reveal(delta);

        assert_eq!(
            chunks,
            vec!["| Name | Value |\n", "| --- | --- |\n", "| Foo | Bar |\n"]
        );
        assert_eq!(chunks.concat(), delta);
    }

    #[test]
    fn text_delta_splits_long_unbroken_words() {
        let long_word = "a".repeat(REVEAL_MAX_UNBROKEN_CHARS * 2 + 7);
        let chunks = split_text_delta_for_reveal(&long_word);

        assert!(chunks.len() > 1);
        assert_eq!(chunks.concat(), long_word);
        assert!(chunks
            .iter()
            .all(|chunk| chunk.chars().count() <= REVEAL_MAX_UNBROKEN_CHARS));
    }

    #[test]
    fn text_delta_complex_markdown_cases_preserve_exact_bytes() {
        let cases = [
            "\n\nNext paragraph",
            " - first item \n\n indented next ",
            "```ts\nconst message = \"hello world\";\n```\n",
            "> quote with **bold** text\n\n1. ordered item\n",
            "| A | B |\n|---|---|\n| 1 | 2 |\n",
        ];

        for delta in cases {
            let chunks = split_text_delta_for_reveal(delta);
            assert_eq!(chunks.concat(), delta, "failed case: {delta:?}");
            assert!(chunks.iter().all(|chunk| !chunk.is_empty()));
        }
    }
}
