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

/// Source-backed slots, with allocation bounded by `limit`. Long requests keep
/// the whole sentence and edge spans before short interior spans.
pub fn span_candidates(request: &str, words: &[Word], limit: usize) -> Vec<Span> {
    let n = words.len();
    if n == 0 || limit == 0 {
        return Vec::new();
    }
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(limit.min(n.saturating_mul(n.saturating_add(1)) / 2));
    let mut add = |from: usize, to: usize| {
        let label = &request[words[from].start..words[to - 1].end];
        if seen.insert(label.to_lowercase()) {
            out.push(Span {
                label: label.to_string(),
                from,
                to,
            });
        }
        out.len() >= limit
    };
    if n.saturating_mul(n.saturating_add(1)) / 2 <= limit {
        for from in 0..n {
            for to in from + 1..=n {
                add(from, to);
            }
        }
    } else {
        let mut full = add(0, n);
        for length in 1..n {
            if full {
                break;
            }
            full = add(0, length);
            if !full {
                full = add(n - length, n);
            }
        }
        for length in 1..n {
            if full {
                break;
            }
            for from in 1..n - length {
                if add(from, from + length) {
                    full = true;
                    break;
                }
            }
        }
    }
    out.sort_by_key(|span| (span.from, span.to));
    out
}

static CLAUSE_BREAK: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r"(?i)\s*,\s*(?:and\s+|then\s+)?|\s+and then\s+|\s+then\s+|\s+and\s+").ok()
});

/// A fifth clause signals overflow; the engine refuses the whole compound request.
pub const MAX_CLAUSES: usize = 4;

fn starts_action(text: &str) -> bool {
    let word = text.split_whitespace().next().unwrap_or_default();
    [
        "open",
        "launch",
        "close",
        "focus",
        "switch",
        "create",
        "add",
        "delete",
        "remove",
        "complete",
        "rename",
        "show",
        "find",
        "search",
        "send",
        "email",
        "remind",
        "set",
        "move",
        "put",
        "press",
        "resize",
        "copy",
        "paste",
        "run",
        "turn",
        "mute",
        "unmute",
        "play",
        "pause",
        "stop",
        "start",
        "write",
        "save",
        "tell",
        "ask",
        "take",
        "capture",
        "translate",
        "summarize",
        "lock",
        "quit",
    ]
    .iter()
    .any(|action| word.eq_ignore_ascii_case(action))
}

/// Protect quoted content and ordinary conjunctions. List commands may lend their
/// verb to targets ("open Gmail and Calendar"); other conjunctions need a new action.
pub fn split_clauses(text: &str) -> Vec<String> {
    split_clauses_for_apps(text, &[])
}

pub fn split_clauses_for_apps(text: &str, apps: &[&str]) -> Vec<String> {
    let Some(clause_break) = CLAUSE_BREAK.as_ref() else {
        return vec![text.trim().to_string()];
    };
    let mut quoted = Vec::new();
    let mut quote = None;
    let mut quote_start = 0;
    let mut escaped = false;
    let mut previous = None;
    for (at, ch) in text.char_indices() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if quote == Some(ch) {
            quoted.push(quote_start..at + ch.len_utf8());
            quote = None;
        } else if quote.is_none() {
            let closing = match ch {
                '"' | '`' => Some(ch),
                '\'' if !previous.is_some_and(char::is_alphanumeric) => Some('\''),
                '“' => Some('”'),
                '‘' => Some('’'),
                _ => None,
            };
            if let Some(closing) = closing {
                quote_start = at;
                quote = Some(closing);
            }
        }
        previous = Some(ch);
    }
    if quote.is_some() {
        quoted.push(quote_start..text.len());
    }
    let verb = text.split_whitespace().next().unwrap_or_default();
    let list = [
        "open", "opens", "launch", "launches", "close", "closes", "focus", "focuses", "switch",
        "switches",
    ]
    .iter()
    .any(|candidate| verb.eq_ignore_ascii_case(candidate));
    let mut parts = Vec::with_capacity(MAX_CLAUSES + 1);
    let mut start = 0;
    for separator in clause_break.find_iter(text) {
        if quoted
            .iter()
            .any(|range| range.contains(&separator.start()))
        {
            continue;
        }
        let right = text[separator.end()..].trim();
        let explicit = separator
            .as_str()
            .split_whitespace()
            .any(|word| word.eq_ignore_ascii_case("then"));
        // “put Chrome on the left and Ghostty on the right” elides the second verb.
        // Only admit a known app followed by a placement, not arbitrary noun phrases.
        let parallel_window = apps.iter().any(|app| {
            right
                .get(..app.len())
                .is_some_and(|s| s.eq_ignore_ascii_case(app))
                && right
                    .get(app.len()..)
                    .is_some_and(|s| s.starts_with(" on ") || s.starts_with(" to "))
        });
        if right.is_empty() || !(explicit || list || parallel_window || starts_action(right)) {
            continue;
        }
        let part = text[start..separator.start()].trim();
        if !part.is_empty() {
            parts.push(part);
        }
        start = separator.end();
        if parts.len() == MAX_CLAUSES {
            break;
        }
    }
    let rest = text[start..].trim();
    if !rest.is_empty() {
        parts.push(rest);
    }
    parts
        .into_iter()
        .enumerate()
        .map(|(index, part)| {
            if index > 0 && list && !starts_action(part) {
                format!("{verb} {part}")
            } else {
                part.to_string()
            }
        })
        .collect()
}
