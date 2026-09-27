//! The words under the sentence: which runs of the input are marked, and in which color.
//!
//! A typed symbol fills its word exactly and is marked as soon as it's typed: `@selection`,
//! `@clipboard` and `@file:…` the Subject, `/command` the Action, `>:dev`, `todo;` and `;todo` the
//! Target (a capture target by name, or while it's being typed, the start of one), `due:…` and
//! `.style` the Details, `|profile` the Owner. Every other mark is a phrase Jev
//! read as one of the words, marked where it still stands in the text: edits move it with the
//! text, and a word Jev hasn't read yet is left unmarked rather than guessed. Marks never
//! overlap; the input's words and caret are never touched.

use std::ops::Range;

use crate::contract::{CAPTURE_TARGETS, marks};

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

/// A name after a sigil: letters first, then letters, digits, `-` or `_`.
fn is_name(rest: &str) -> bool {
    rest.starts_with(|c: char| c.is_ascii_alphabetic())
        && rest
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn symbol_word(token: &str) -> Option<Word> {
    let lower = token.to_lowercase();
    if lower == "@selection" || lower == "@clipboard" || lower.starts_with("@file:") {
        Some(Word::Subject)
    } else if token.starts_with(">:") && token.len() > 2 {
        Some(Word::Target)
    } else if token
        .strip_suffix(';')
        .is_some_and(|name| CAPTURE_TARGETS.iter().any(|t| t.eq_ignore_ascii_case(name)))
    {
        Some(Word::Target)
    } else if token.strip_prefix(';').is_some_and(|name| {
        let name = name.to_ascii_lowercase();
        is_name(&name) && CAPTURE_TARGETS.iter().any(|t| t.starts_with(&name))
    }) {
        Some(Word::Target)
    } else if token.starts_with('/') && token[1..].starts_with(|c: char| c.is_ascii_alphabetic()) {
        Some(Word::Action)
    } else if lower
        .strip_prefix("due:")
        .is_some_and(|rest| !rest.is_empty())
    {
        Some(Word::Details)
    } else if token
        .strip_prefix('.')
        .is_some_and(|rest| rest.chars().all(|c| c.is_ascii_alphabetic()) && !rest.is_empty())
    {
        Some(Word::Details)
    } else if token.strip_prefix('|').is_some_and(is_name) {
        Some(Word::Owner)
    } else {
        None
    }
}

/// The symbols typed in `text`, each marking its word.
pub fn typed_symbols(text: &str) -> Vec<Mark> {
    let mut out = Vec::new();
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
    out
}

/// Whether a phrase may start right after `before`: at the start, after a space, or after an
/// opening quote or bracket, never inside a token like `3.5` or `a|b`.
fn token_starts(before: &str) -> bool {
    before
        .chars()
        .next_back()
        .is_none_or(|c| c.is_whitespace() || matches!(c, '"' | '\'' | '(' | '[' | '“' | '‘'))
}

/// Whether a phrase may end right before `after`: at the end, before a space, or before closing
/// punctuation that ends its token.
fn token_ends(after: &str) -> bool {
    let token = after.split(char::is_whitespace).next().unwrap_or_default();
    token.chars().all(|c| {
        matches!(
            c,
            '.' | ',' | '!' | '?' | ';' | ':' | ')' | ']' | '"' | '\'' | '”' | '’'
        )
    })
}

/// The marks for `text`: its typed symbols, then each phrase Jev `read` as a word where it stands
/// in the text now, first come first marked.
pub fn marks_for(text: &str, read: &[(Word, &str)]) -> Vec<Mark> {
    compose(text, Vec::new(), read)
}

/// Like [`marks_for`], with `exact` marks first: runs the launcher's own grammar already knows
/// the word of, such as a command it resolved.
pub fn compose(text: &str, exact: Vec<Mark>, read: &[(Word, &str)]) -> Vec<Mark> {
    let mut out: Vec<Mark> = Vec::new();
    let free = |out: &[Mark], range: &Range<usize>| {
        !out.iter()
            .any(|m| m.range.start < range.end && range.start < m.range.end)
    };
    let names_something = |range: &Range<usize>| {
        range.start < range.end
            && range.end <= text.len()
            && text.is_char_boundary(range.start)
            && text.is_char_boundary(range.end)
            && text[range.clone()].chars().any(char::is_alphanumeric)
    };
    for mark in exact {
        if names_something(&mark.range) && free(&out, &mark.range) {
            out.push(mark);
        }
    }
    for mark in typed_symbols(text) {
        let covered: Vec<usize> = (0..out.len())
            .filter(|&i| out[i].range.start < mark.range.end && mark.range.start < out[i].range.end)
            .collect();
        let grows = covered.len() == 1 && {
            let inner = &out[covered[0]];
            inner.word == mark.word
                && mark.range.start <= inner.range.start
                && inner.range.end <= mark.range.end
        };
        if grows {
            out[covered[0]] = mark;
        } else if covered.is_empty() {
            out.push(mark);
        }
    }
    let lower = text.to_lowercase();
    for (word, phrase) in read {
        let phrase = phrase.trim().to_lowercase();
        if phrase.is_empty() || lower.len() != text.len() {
            continue;
        }
        let found = lower.match_indices(&phrase).find(|(at, _)| {
            let range = *at..*at + phrase.len();
            let whole = token_starts(&lower[..range.start]) && token_ends(&lower[range.end..]);
            whole && names_something(&range) && free(&out, &range)
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
    fn every_symbol_the_launcher_reads_fills_its_word() {
        let text = "@clipboard |creative .concise due:friday ;todo @file:readme.md /rewrite";
        assert_eq!(
            marked(text, &marks_for(text, &[])),
            vec![
                ("@clipboard", Word::Subject),
                ("|creative", Word::Owner),
                (".concise", Word::Details),
                ("due:friday", Word::Details),
                (";todo", Word::Target),
                ("@file:readme.md", Word::Subject),
                ("/rewrite", Word::Action),
            ]
        );
        for plain in ["e.g.", "3.5", "a|b", "due:", "|", ".", "file.txt", ";"] {
            assert!(
                marks_for(plain, &[]).is_empty(),
                "{plain:?} is not a symbol"
            );
        }
    }

    #[test]
    fn a_capture_target_is_marked_while_it_is_typed_and_only_when_it_names_one() {
        for text in [";t", ";to", ";tod", ";todo", ";Note", "note;", "Todo;"] {
            assert_eq!(
                marked(text, &marks_for(text, &[])),
                vec![(text, Word::Target)],
                "{text:?} names a capture target"
            );
        }
        for text in [";xyz", ";todos", "hello;", "to;", "3;"] {
            assert!(
                marks_for(text, &[]).is_empty(),
                "{text:?} names no capture target"
            );
        }
    }

    #[test]
    fn marks_the_launcher_resolved_come_first_and_nothing_overlaps_them() {
        let text = "summarize this @clipboard";
        let exact = vec![Mark {
            range: 0..9,
            word: Word::Action,
        }];
        let read = [(Word::Action, "summarize this"), (Word::Subject, "this")];
        assert_eq!(
            marked(text, &compose(text, exact, &read)),
            vec![
                ("summarize", Word::Action),
                ("this", Word::Subject),
                ("@clipboard", Word::Subject),
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
    fn a_bare_symbol_marks_nothing_until_a_word_follows_it() {
        for text in [">:", ";", ".", "|", "@", "/"] {
            let exact = vec![Mark {
                range: 0..text.len(),
                word: Word::Target,
            }];
            assert!(
                compose(text, exact, &[]).is_empty(),
                "{text:?} names nothing yet"
            );
        }
    }

    #[test]
    fn a_typed_symbol_grows_the_launchers_shorter_mark_of_the_same_word() {
        let text = "@file:readme.md";
        let exact = vec![Mark {
            range: 0..6,
            word: Word::Subject,
        }];
        assert_eq!(
            marked(text, &compose(text, exact, &[])),
            vec![("@file:readme.md", Word::Subject)]
        );
    }

    #[test]
    fn jevs_phrases_never_cut_a_token_in_half() {
        let text = "e.g. 3.5 a|b file.txt";
        let read = [(Word::Subject, "e.g. 3"), (Word::Target, "b file")];
        assert!(marks_for(text, &read).is_empty());
        let text = "open slack, then \"call mom\"";
        let read = [(Word::Subject, "slack"), (Word::Details, "call mom")];
        assert_eq!(
            marked(text, &marks_for(text, &read)),
            vec![("slack", Word::Subject), ("call mom", Word::Details)]
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
