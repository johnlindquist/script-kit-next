//! The user's words, and every run of them Jev may pick as a slot's value.

use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    pub text: String,
    /// Byte offsets into the request.
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub label: String,
    /// Word indices, `to` exclusive.
    pub from: usize,
    pub to: usize,
}

/// Enough runs for a long dictated sentence while staying under Jev's option cap.
pub const SPAN_LIMIT: usize = 240;

fn leading_punctuation(c: char) -> bool {
    c.is_whitespace() || matches!(c, '"' | '\'' | '“' | '”' | '‘' | '’' | '(' | '[' | '{')
}

fn trailing_punctuation(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '"' | '\''
                | '“'
                | '”'
                | '‘'
                | '’'
                | ')'
                | ']'
                | '}'
                | '.'
                | ','
                | '!'
                | '?'
                | ';'
                | ':'
        )
}

/// Words with byte offsets into the request, trimmed of edge punctuation.
pub fn tokenize(request: &str) -> Vec<Word> {
    let mut out = Vec::new();
    let mut offset = 0;
    for raw in request.split_inclusive(char::is_whitespace) {
        let start_of_raw = offset;
        offset += raw.len();
        let raw = raw.trim_end_matches(char::is_whitespace);
        let lead_trimmed = raw.trim_start_matches(leading_punctuation);
        let text = lead_trimmed.trim_end_matches(trailing_punctuation);
        if text.is_empty() {
            continue;
        }
        let start = start_of_raw + (raw.len() - lead_trimmed.len());
        out.push(Word {
            text: text.to_string(),
            start,
            end: start + text.len(),
        });
    }
    out
}

/// Every contiguous run of words, deduplicated by text. Jev cannot write text, so a
/// slot like "what to remember" is answered by choosing one of these runs. Past
/// `limit`, runs that touch either end of the request are kept before long middles.
pub fn span_candidates(request: &str, words: &[Word], limit: usize) -> Vec<Span> {
    let n = words.len();
    let mut all = Vec::with_capacity(n * (n + 1) / 2);
    for from in 0..n {
        for to in from + 1..=n {
            all.push(Span {
                label: request[words[from].start..words[to - 1].end].to_string(),
                from,
                to,
            });
        }
    }
    if all.len() > limit {
        all.sort_by_key(|span| {
            if span.from == 0 || span.to == n {
                0
            } else {
                span.to - span.from
            }
        });
    }
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for span in all {
        if !seen.insert(span.label.to_lowercase()) {
            continue;
        }
        out.push(span);
        if out.len() >= limit {
            break;
        }
    }
    out.sort_by_key(|span| (span.from, span.to));
    out
}

static CLAUSE_BREAK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\s*,\s*(?:and\s+|then\s+)?|\s+and then\s+|\s+then\s+|\s+and\s+")
        .expect("clause pattern compiles")
});

/// Splits "open gmail, calendar and linear" into steps, lending the first verb to
/// verbless tails.
pub fn split_clauses(text: &str) -> Vec<String> {
    let parts: Vec<&str> = CLAUSE_BREAK
        .split(text)
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if parts.len() < 2 {
        return parts.into_iter().map(str::to_string).collect();
    }
    let verb = parts[0].split_whitespace().next().unwrap_or_default();
    parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            if i > 0 && part.split_whitespace().count() <= 2 {
                format!("{verb} {part}")
            } else {
                part.to_string()
            }
        })
        .collect()
}
