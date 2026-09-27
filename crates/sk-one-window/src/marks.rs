//! The words under the sentence: which runs of the input are marked, and in which color.
//!
//! A typed symbol (`@selection`, `@clipboard`, `@file:…`, `/command`, `>:dev`, `todo;`) fills
//! its word exactly and is marked as soon as it's typed. Every other mark is a phrase Jev read as
//! one of the words, marked where it still stands in the text: edits move it with the text, and a
//! word Jev hasn't read yet is left unmarked rather than guessed. Marks never overlap; the input's
//! words and caret are never touched.

use std::ops::Range;

use crate::contract::marks;

/// The words every sentence fills (the spec's vocabulary).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    /// The one thing acted on.
    Subject,
    /// The verb.
    Action,
    /// Where it goes, or who gets it.
    Target,
    /// When, what it says, how.
    Details,
    /// Who does the work.
    Owner,
}

impl Word {
    /// The mark's color, 0xRRGGBB.
    pub fn color(self) -> u32 {
        match self {
            Word::Subject => marks::SUBJECT,
            Word::Action => marks::ACTION,
            Word::Target => marks::TARGET,
            Word::Details => marks::DETAILS,
            Word::Owner => marks::OWNER,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Word::Subject => "subject",
            Word::Action => "action",
            Word::Target => "target",
            Word::Details => "details",
            Word::Owner => "owner",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mark {
    /// Byte offsets into the text.
    pub range: Range<usize>,
    pub word: Word,
}

fn symbol_word(token: &str) -> Option<Word> {
    let lower = token.to_lowercase();
    if lower == "@selection" || lower == "@clipboard" || lower.starts_with("@file:") {
        Some(Word::Subject)
    } else if token.starts_with(">:") && token.len() > 2 {
        Some(Word::Target)
    } else if token.ends_with(';')
        && token.len() > 1
        && token[..token.len() - 1].chars().all(char::is_alphanumeric)
    {
        Some(Word::Target)
    } else if token.starts_with('/') && token[1..].starts_with(|c: char| c.is_ascii_alphabetic()) {
        Some(Word::Action)
    } else {
        None
    }
}

/// The marks for `text`: its typed symbols, then each phrase Jev `read` as a word where it stands
/// in the text now, first come first marked.
pub fn marks_for(text: &str, read: &[(Word, &str)]) -> Vec<Mark> {
    let mut out: Vec<Mark> = Vec::new();
    let mut offset = 0;
    for token in text.split_inclusive(char::is_whitespace) {
        let word = token.trim_end();
        if let Some(kind) = symbol_word(word) {
            out.push(Mark {
                range: offset..offset + word.len(),
                word: kind,
            });
        }
        offset += token.len();
    }
    let lower = text.to_lowercase();
    for (word, phrase) in read {
        let phrase = phrase.trim().to_lowercase();
        if phrase.is_empty() || lower.len() != text.len() {
            continue;
        }
        let found = lower.match_indices(&phrase).find(|(at, _)| {
            let range = *at..*at + phrase.len();
            let whole = !lower[..range.start].ends_with(char::is_alphanumeric)
                && !lower[range.end..].starts_with(char::is_alphanumeric);
            whole
                && !out
                    .iter()
                    .any(|m| m.range.start < range.end && range.start < m.range.end)
        });
        if let Some((at, _)) = found {
            out.push(Mark {
                range: at..at + phrase.len(),
                word: *word,
            });
        }
    }
    out.sort_by_key(|m| m.range.start);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marked<'a>(text: &'a str, marks: &[Mark]) -> Vec<(&'a str, Word)> {
        marks
            .iter()
            .map(|m| (&text[m.range.clone()], m.word))
            .collect()
    }

    #[test]
    fn typed_symbols_are_marked_before_jev_reads_anything() {
        let text = "summarize @selection >:dev todo; /fix";
        assert_eq!(
            marked(text, &marks_for(text, &[])),
            vec![
                ("@selection", Word::Subject),
                (">:dev", Word::Target),
                ("todo;", Word::Target),
                ("/fix", Word::Action),
            ]
        );
    }

    #[test]
    fn marks_follow_the_words_jev_read_as_the_text_is_edited() {
        let read = [(Word::Subject, "call mom"), (Word::Details, "at 5")];
        let text = "remind me to call mom at 5";
        assert_eq!(
            marked(text, &marks_for(text, &read)),
            vec![("call mom", Word::Subject), ("at 5", Word::Details)]
        );
        let edited = "please remind me to call mom at 5";
        assert_eq!(
            marked(edited, &marks_for(edited, &read)),
            vec![("call mom", Word::Subject), ("at 5", Word::Details)],
            "the marks move with the words"
        );
        let changed = "remind me to call dad at 5 tomorrow";
        assert_eq!(
            marked(changed, &marks_for(changed, &read)),
            vec![("at 5", Word::Details)],
            "a phrase that's gone loses its mark, and new words aren't guessed"
        );
    }

    #[test]
    fn marks_cover_whole_words_and_never_overlap() {
        let read = [(Word::Target, "sarah"), (Word::Subject, "sarah hi")];
        let text = "tell sarahs sarah hi";
        let marks = marks_for(text, &read);
        assert_eq!(marked(text, &marks), vec![("sarah", Word::Target)]);
        for pair in marks.windows(2) {
            assert!(pair[0].range.end <= pair[1].range.start);
        }
    }
}
