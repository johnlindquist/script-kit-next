//! → at the end of a sentence opens how Jev reads it (S04): the same seven words for every
//! sentence, each saying where it came from. A typed symbol fills its word exactly and Jev isn't
//! asked about it; Jev reads the rest from its answers; Context and Focused are on screen. It's a
//! list in place of the list, and Enter still runs the row the list had highlighted.

use sk_one_window::enter::EnterTarget;
use sk_one_window::marks::{typed_symbols, Word};

use crate::intent::{WordKind, WordRead, WordSource};

pub(crate) const READING: &str = "reading";

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WordRow {
    pub(crate) kind: WordKind,
    pub(crate) value: String,
    pub(crate) sub: String,
    /// "typed", "Jev 93%", "on screen", or empty for words the sentence says outright.
    pub(crate) meta: String,
    pub(crate) typed: bool,
    /// Nothing in the sentence fills it.
    pub(crate) dim: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ReadingView {
    pub(crate) text: String,
    pub(crate) rows: Vec<WordRow>,
    /// What Enter did in the list when → opened this; it still does.
    pub(crate) enter: EnterTarget,
    /// ⇥ made the agent the Owner.
    pub(crate) owner_tab: bool,
}

impl ReadingView {
    pub(crate) fn new(
        text: String,
        words: Option<&[WordRead]>,
        enter: EnterTarget,
        owner_tab: bool,
    ) -> Self {
        let rows = reading_rows(&text, words, owner_tab);
        Self {
            text,
            rows,
            enter,
            owner_tab,
        }
    }

    pub(crate) fn enter(&self) -> EnterTarget {
        EnterTarget::new(
            format!("why:run:{}", self.enter.id),
            self.enter.label.clone(),
        )
    }

    /// The word each run of the sentence is marked as while this is open.
    pub(crate) fn marks(&self, words: Option<&[WordRead]>) -> Vec<(Word, String)> {
        let typed: Vec<WordKind> = self
            .rows
            .iter()
            .filter(|r| r.typed)
            .map(|r| r.kind)
            .collect();
        words
            .unwrap_or_default()
            .iter()
            .filter(|w| !typed.contains(&w.kind))
            .filter_map(|w| Some((word_of(w.kind)?, w.said.clone()?)))
            .collect()
    }
}

pub(crate) fn word_of(kind: WordKind) -> Option<Word> {
    match kind {
        WordKind::Subject => Some(Word::Subject),
        WordKind::Action => Some(Word::Action),
        WordKind::Target => Some(Word::Target),
        WordKind::Details => Some(Word::Details),
        WordKind::Owner => Some(Word::Owner),
        WordKind::Context | WordKind::Focused => None,
    }
}

fn kind_of(word: Word) -> WordKind {
    match word {
        Word::Subject => WordKind::Subject,
        Word::Action => WordKind::Action,
        Word::Target => WordKind::Target,
        Word::Details => WordKind::Details,
        Word::Owner => WordKind::Owner,
    }
}

/// The chip color of each word, 0xRRGGBB.
pub(crate) fn word_color(kind: WordKind) -> u32 {
    match word_of(kind) {
        Some(word) => word.color(),
        None if kind == WordKind::Focused => sk_one_window::contract::marks::FOCUSED,
        None => sk_one_window::contract::marks::CONTEXT,
    }
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// What a typed symbol fills its word with.
fn typed_value(token: &str) -> String {
    let lower = token.to_lowercase();
    if lower == "@selection" {
        "The selected text".into()
    } else if lower == "@clipboard" {
        "The clipboard".into()
    } else if let Some(file) = lower.strip_prefix("@file:") {
        if file.is_empty() {
            "A file".into()
        } else {
            token["@file:".len()..].to_string()
        }
    } else if let Some(project) = token.strip_prefix(">:") {
        format!("The {project} project")
    } else if let Some(command) = token.strip_prefix('/') {
        capitalized(command)
    } else if let Some(due) = lower.strip_prefix("due:") {
        format!("Due {due}")
    } else if let Some(style) = token.strip_prefix('.') {
        format!("{} (style)", capitalized(style))
    } else if let Some(profile) = token.strip_prefix('|') {
        format!("The {profile} profile")
    } else if let Some(target) = token.strip_prefix(';') {
        capitalized(target)
    } else {
        capitalized(token.trim_end_matches(';'))
    }
}

fn from_jev(word: &WordRead) -> WordRow {
    WordRow {
        kind: word.kind,
        value: word.value.clone(),
        sub: word.sub.clone(),
        meta: match (word.source, word.probability) {
            (WordSource::Screen, _) => "on screen".into(),
            (WordSource::Jev, Some(p)) => format!("Jev {}%", (p * 100.0).round() as i64),
            (WordSource::Jev, None) => "Jev".into(),
            (WordSource::Words, _) => String::new(),
        },
        typed: false,
        dim: word.dim,
    }
}

/// The seven rows for `text`: typed symbols at once, the rest as Jev read them, or waiting on
/// Jev while it reads.
pub(crate) fn reading_rows(
    text: &str,
    words: Option<&[WordRead]>,
    owner_tab: bool,
) -> Vec<WordRow> {
    let typed = typed_symbols(text);
    let typed_token = |kind: WordKind| {
        typed
            .iter()
            .find(|mark| kind_of(mark.word) == kind)
            .map(|mark| &text[mark.range.clone()])
    };
    let jev = |kind: WordKind| words.and_then(|w| w.iter().find(|w| w.kind == kind));
    WordKind::ALL
        .iter()
        .map(|&kind| {
            if kind == WordKind::Owner && owner_tab {
                return WordRow {
                    kind,
                    value: capitalized(sk_one_window::contract::words::OWNER_AGENT),
                    sub: "You pressed ⇥, so Jev isn't asked".into(),
                    meta: "⇥ Tab".into(),
                    typed: true,
                    dim: false,
                };
            }
            if let Some(token) = typed_token(kind) {
                return WordRow {
                    kind,
                    value: typed_value(token),
                    sub: format!("You typed {token}, so Jev isn't asked"),
                    meta: "typed".into(),
                    typed: true,
                    dim: false,
                };
            }
            match jev(kind) {
                Some(word) => {
                    let mut row = from_jev(word);
                    if kind == WordKind::Focused {
                        let this = jev(WordKind::Subject)
                            .and_then(|s| s.said.as_deref())
                            .filter(|said| {
                                matches!(said.to_lowercase().as_str(), "this" | "it" | "here")
                            });
                        if let (Some(this), Some(token)) = (this, typed_token(WordKind::Subject)) {
                            row.sub =
                                format!("“{this}” would mean this, but {token} names the Subject");
                            row.dim = true;
                        }
                    }
                    row
                }
                None => WordRow {
                    kind,
                    value: "Jev is reading…".into(),
                    sub: String::new(),
                    meta: String::new(),
                    typed: false,
                    dim: true,
                },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(kind: WordKind, value: &str, said: Option<&str>, p: Option<f64>) -> WordRead {
        WordRead {
            kind,
            value: value.into(),
            sub: String::new(),
            said: said.map(str::to_string),
            source: if matches!(kind, WordKind::Context | WordKind::Focused) {
                WordSource::Screen
            } else {
                WordSource::Jev
            },
            probability: p,
            dim: false,
        }
    }

    fn read(words: &[(WordKind, &str, Option<&str>)]) -> Vec<WordRead> {
        WordKind::ALL
            .iter()
            .map(|&kind| match words.iter().find(|(k, ..)| *k == kind) {
                Some((_, value, said)) => word(kind, value, *said, Some(0.93)),
                None => word(kind, "—", None, None),
            })
            .collect()
    }

    #[test]
    fn typed_symbols_fill_their_word_and_jev_fills_the_rest() {
        let words = read(&[
            (WordKind::Subject, "“hey john…”", Some("this")),
            (WordKind::Action, "Summarize", Some("summarize")),
        ]);
        let rows = reading_rows("summarize this @clipboard .concise", Some(&words), false);
        assert_eq!(rows.len(), 7);
        let row = |kind| rows.iter().find(|r| r.kind == kind).unwrap();
        assert_eq!(row(WordKind::Subject).value, "The clipboard");
        assert_eq!(row(WordKind::Subject).meta, "typed");
        assert_eq!(row(WordKind::Details).value, "Concise (style)");
        assert_eq!(row(WordKind::Action).meta, "Jev 93%");
        assert_eq!(row(WordKind::Context).meta, "on screen");
        assert_eq!(
            row(WordKind::Focused).sub,
            "“this” would mean this, but @clipboard names the Subject"
        );
        let view = ReadingView::new(
            "summarize this @clipboard .concise".into(),
            Some(&words),
            EnterTarget::new("row", "Summarize"),
            false,
        );
        assert_eq!(
            view.marks(Some(&words)),
            vec![(Word::Action, "summarize".to_string())],
            "Jev's “this” isn't marked once @clipboard names the Subject"
        );
    }

    #[test]
    fn words_wait_on_jev_until_it_answers() {
        let rows = reading_rows("have the agent fix this", None, false);
        assert!(rows
            .iter()
            .all(|r| r.value == "Jev is reading…" && !r.typed));
        let rows = reading_rows("fix this |claude", None, false);
        assert_eq!(rows.iter().filter(|r| r.typed).count(), 1);
        assert_eq!(rows[6].value, "The claude profile");
    }

    #[test]
    fn tab_fills_the_owner_and_jev_isnt_asked_about_it() {
        let words = read(&[
            (WordKind::Action, "Fix", Some("fix")),
            (WordKind::Owner, "You", None),
        ]);
        let view = ReadingView::new(
            "fix this".into(),
            Some(&words),
            EnterTarget::new("owner:agent", "Start the agent"),
            true,
        );
        let owner = view
            .rows
            .iter()
            .find(|r| r.kind == WordKind::Owner)
            .unwrap();
        assert_eq!(
            (owner.value.as_str(), owner.meta.as_str(), owner.typed),
            ("Agent", "⇥ Tab", true)
        );
        assert_eq!(view.enter().label, "Start the agent");
    }
}
