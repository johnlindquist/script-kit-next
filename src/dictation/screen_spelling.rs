//! One set of local spelling clues per recording. No text from the window is a
//! prompt, a persisted vocabulary entry, or an automation payload.
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock, Weak};
use std::time::Instant;

const MAX_INPUT: usize = 4096;
const MAX_TERMS: usize = 48;
const MAX_TRANSCRIPT: usize = 8192;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Owner {
    pub generation: u64,
    pub pid: i32,
    pub window: u32,
}

pub enum Evidence {
    Ready {
        source: &'static str,
        terms: super::polish::PreparedVocabulary,
    },
    Skipped(&'static str),
}

impl Evidence {
    pub fn ready(source: &'static str, words: &[String]) -> Self {
        Self::Ready {
            source,
            terms: super::polish::PreparedVocabulary::new(words),
        }
    }
}

pub struct Session {
    owner: Owner,
    evidence: Arc<OnceLock<Evidence>>,
}

pub struct Publisher(Weak<OnceLock<Evidence>>);
impl Publisher {
    pub fn live(&self) -> bool {
        self.0.strong_count() != 0
    }
    pub fn publish(self, result: Evidence) {
        if let Some(slot) = self.0.upgrade() {
            let _ = slot.set(result);
        }
    }
}

#[derive(Clone, Copy, serde::Serialize)]
pub struct Receipt {
    pub status: &'static str,
    pub source: Option<&'static str>,
    pub changes: usize,
    pub micros: u64,
}

impl Session {
    pub fn new(owner: Owner) -> (Self, Publisher) {
        let evidence = Arc::new(OnceLock::new());
        let publish = Publisher(Arc::downgrade(&evidence));
        (Self { owner, evidence }, publish)
    }

    pub fn unavailable(&self, reason: &'static str) {
        let _ = self.evidence.set(Evidence::Skipped(reason));
    }

    pub fn status(&self) -> &'static str {
        match self.evidence.get() {
            Some(Evidence::Ready { .. }) => "ready",
            Some(Evidence::Skipped(reason)) => reason,
            None => "pending",
        }
    }

    /// Never joins the collector, performs I/O, or takes a mutex. Consuming the
    /// session invalidates its publisher even when no result was ready yet.
    pub fn finish(self, owner: Owner, text: &str) -> (Option<String>, Receipt) {
        let start = Instant::now();
        let mut receipt = Receipt {
            status: "notReady",
            changes: 0,
            source: None,
            micros: 0,
        };
        let mut output = None;
        if self.owner != owner {
            receipt.status = "targetChanged";
        } else if text.len() > MAX_TRANSCRIPT {
            receipt.status = "transcriptTooLong";
        } else if let Some(evidence) = self.evidence.get() {
            match evidence {
                Evidence::Skipped(reason) => receipt.status = reason,
                Evidence::Ready { source, terms } => {
                    let (next, changes) = terms.rewrite(text);
                    receipt.source = Some(source);
                    receipt.changes = changes;
                    receipt.status = if changes == 0 { "noMatch" } else { "applied" };
                    output = next;
                }
            }
        }
        receipt.micros = start.elapsed().as_micros() as u64;
        (output, receipt)
    }
}

/// Do not split an oversize token at the excerpt boundary and invent a name.
fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_alphanumeric())
        .take_while(move |word| {
            word.as_ptr() as usize - text.as_ptr() as usize + word.len() <= MAX_INPUT
        })
        .filter(|word| !word.is_empty())
}

/// Mixed-case identifiers and acronyms can stand on their own. Ordinary
/// title-case words need separate evidence (an app name or a named entity),
/// so a sentence starting with “Please” doesn't capitalize every “please”.
pub fn terms(excerpt: &str, names: &[String], personal: &[String]) -> Vec<String> {
    let reserved: Vec<_> = personal
        .iter()
        .flat_map(|s| std::iter::once(s.as_str()).chain(words(s)))
        .map(super::polish::squash)
        .filter(|s| !s.is_empty())
        .collect();
    let mut candidates: BTreeMap<String, Option<&str>> = BTreeMap::new();
    let offer = |word: &str, named: bool| -> bool {
        let n = word.chars().count();
        (3..=40).contains(&n)
            && word
                .chars()
                .all(|c| c.is_alphabetic() || (named && c == ' '))
            && word.split_whitespace().count() <= 3
            && (named && word.chars().any(char::is_uppercase)
                || word.chars().skip(1).any(char::is_uppercase)
                    && (word.chars().any(char::is_lowercase) || n <= 8))
    };
    // Keep a spelling only if the bounded evidence agrees about its case.
    for (word, named) in names
        .iter()
        .take(32)
        .map(|s| (s.trim(), true))
        .chain(words(excerpt).map(|w| (w, false)))
    {
        if !offer(word, named) {
            continue;
        }
        let key = super::polish::squash(word);
        if reserved.iter().any(|k| key.contains(k) || k.contains(&key)) {
            continue;
        }
        candidates
            .entry(key)
            .and_modify(|v| {
                if *v != Some(word) {
                    *v = None;
                }
            })
            .or_insert(Some(word));
    }
    for word in words(excerpt) {
        if let Some(v) = candidates.get_mut(&super::polish::squash(word)) {
            if *v != Some(word) {
                *v = None;
            }
        }
    }
    candidates
        .into_values()
        .flatten()
        .take(MAX_TERMS)
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    const OWNER: Owner = Owner {
        generation: 4,
        pid: 7,
        window: 12,
    };
    #[test]
    fn only_unambiguous_supported_spellings_from_this_recording() {
        let list = terms(
            "Please open QuickLook. Review TypeScript TYPESCRIPT and OpenAI with Maeve.",
            &["Maeve".into()],
            &["Open AI".into()],
        );
        let (s, publish) = Session::new(OWNER);
        publish.publish(Evidence::ready("accessibility", &list));
        let (text, r) = s.finish(
            OWNER,
            "please ask maeve about quick look, type script and open ai. Leave Shawn alone.",
        );
        assert_eq!(r.changes, 2);
        assert_eq!(
            text.unwrap(),
            "please ask Maeve about QuickLook, type script and open ai. Leave Shawn alone."
        );
    }
    #[test]
    fn missing_late_cancelled_and_changed_targets_do_not_rewrite() {
        let (s, p) = Session::new(OWNER);
        assert_eq!(s.finish(OWNER, "type script").1.status, "notReady");
        assert!(!p.live());
        p.publish(Evidence::Skipped("late"));
        let (s, p) = Session::new(OWNER);
        drop(s);
        assert!(!p.live());
        for owner in [
            Owner {
                generation: 5,
                ..OWNER
            },
            Owner {
                window: 13,
                ..OWNER
            },
        ] {
            let (s, p) = Session::new(OWNER);
            p.publish(Evidence::ready("ocr", &["TypeScript".into()]));
            assert!(s.finish(owner, "type script").0.is_none());
        }
    }
}
