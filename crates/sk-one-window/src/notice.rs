//! Word from the app that isn't a run, and runs that can't be taken back.
//!
//! A notice (a script's `toast()` or `hud()`, a failure, a hint) lands in the ledger: the line
//! above the input says it at once while the window shows, and says it when the window comes back
//! if it was hidden. Nothing pops up, and a notice never covers a message that's waiting to go.
//!
//! Something irreversible (emptying the Trash, clearing ranking data) waits in the line like a
//! message does, and the host runs it only when the wait runs out.
//!
//! Whatever the text, the line says it as one readable line: whitespace and line breaks fold to
//! single spaces, invisible direction controls and simple markup go, an emoji at either end goes
//! (the icon already says what kind of word this is), and an empty notice says nothing. A
//! failure says only its sentence: "Couldn't save the note: Permission denied (os error 13)"
//! shows "Couldn't save the note", and the host logs the rest.

/// The most characters a notice shows before it ends in "…".
pub const MOST_CHARS: usize = 140;

/// A notice for the line above the input.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Notice {
    pub icon: String,
    pub title: String,
}

impl Notice {
    pub fn new(icon: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            title: title.into(),
        }
    }

    /// The words the line says: a failure (`icon` is [`crate::contract::FAILED_ICON`]) only
    /// its sentence ([`failure_words`]), anything else as [`one_line`].
    pub fn shown_title(&self) -> String {
        if self.icon == crate::contract::FAILED_ICON {
            failure_words(&self.title).0
        } else {
            one_line(&self.title)
        }
    }
}

/// `text` as one readable line for the header.
pub fn one_line(text: &str) -> String {
    let text = strip_tags(text);
    let mut folded = String::with_capacity(text.len());
    for c in text.chars() {
        if is_direction_control(c) {
            continue;
        }
        if c.is_whitespace() || c.is_control() {
            if !folded.is_empty() && !folded.ends_with(' ') {
                folded.push(' ');
            }
            continue;
        }
        folded.push(c);
    }
    let folded = folded.replace("**", "").replace('`', "");
    let trimmed = trim_emoji(folded.trim());
    shorten(trimmed)
}

/// A failure's sentence for the screen, and the detail that stays in the logs.
///
/// The app's failures read "<what didn't happen>: <the error>", so the words before the first
/// ": " outside quotes are the sentence. An OS error number never shows, and a bare internal
/// code ("stale_prompt_submission") says [`crate::contract::words::SOMETHING_WENT_WRONG`].
pub fn failure_words(text: &str) -> (String, Option<String>) {
    let line = one_line(text);
    let (sentence, detail) = match split_at_detail(&line) {
        Some(at) => (
            line[..at].trim_end().to_string(),
            Some(line[at + 1..].trim().to_string()),
        ),
        None => (line.clone(), None),
    };
    let (sentence, os_error) = without_os_error(&sentence);
    let detail = match (detail, os_error) {
        (Some(detail), _) => Some(detail),
        (None, true) => Some(line.clone()),
        (None, false) => None,
    };
    if looks_like_a_code(&sentence) {
        return (
            crate::contract::words::SOMETHING_WENT_WRONG.to_string(),
            Some(line),
        );
    }
    (sentence, detail.filter(|detail| !detail.is_empty()))
}

/// Where the first ": " outside “quotes” or "quotes" starts.
fn split_at_detail(line: &str) -> Option<usize> {
    let mut curly = 0usize;
    let mut straight = false;
    let mut chars = line.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        match c {
            '\u{201C}' => curly += 1,
            '\u{201D}' => curly = curly.saturating_sub(1),
            '"' => straight = !straight,
            ':' if curly == 0 && !straight && at > 0 => {
                if matches!(chars.peek(), Some((_, ' '))) {
                    return Some(at);
                }
            }
            _ => {}
        }
    }
    None
}

/// `text` without a trailing "(os error N)", and whether it had one.
fn without_os_error(text: &str) -> (String, bool) {
    match text.find("(os error ") {
        Some(at) if text.trim_end().ends_with(')') => (text[..at].trim_end().to_string(), true),
        _ => (text.to_string(), false),
    }
}

/// One word of lowercase letters, digits and underscores, with at least one underscore.
fn looks_like_a_code(text: &str) -> bool {
    text.contains('_')
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
}

/// Invisible marks that change the direction text runs or join characters unseen.
fn is_direction_control(c: char) -> bool {
    matches!(
        c,
        '\u{200B}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
            | '\u{FEFF}'
    )
}

/// An emoji, or a mark that belongs to one (variation selector, joiner, skin tone, flag half).
fn is_emoji_part(c: char) -> bool {
    matches!(
        c,
        '\u{1F000}'..='\u{1FAFF}'
            | '\u{2600}'..='\u{27BF}'
            | '\u{2B00}'..='\u{2BFF}'
            | '\u{FE0E}'
            | '\u{FE0F}'
            | '\u{200D}'
            | '\u{20E3}'
    )
}

fn trim_emoji(text: &str) -> &str {
    text.trim_start_matches(|c: char| is_emoji_part(c) || c == ' ')
        .trim_end_matches(|c: char| is_emoji_part(c) || c == ' ')
}

/// Simple HTML tags (`<b>`, `</em>`, `<br/>`) out, everything else kept.
fn strip_tags(text: &str) -> String {
    const TAGS: [&str; 15] = [
        "b", "i", "u", "s", "em", "strong", "code", "br", "span", "div", "p", "a", "small", "mark",
        "kbd",
    ];
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('<') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let tag = after
            .find('>')
            .map(|close| (&after[..close], close))
            .filter(|(inner, _)| {
                let name = inner.trim_start_matches('/').trim_end_matches('/');
                let name = name.split(' ').next().unwrap_or("");
                TAGS.contains(&name.to_ascii_lowercase().as_str())
            });
        match tag {
            Some((inner, close)) => {
                if inner.trim_end_matches('/') == "br" {
                    out.push(' ');
                }
                rest = &after[close + 1..];
            }
            None => {
                out.push('<');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

fn shorten(text: &str) -> String {
    if text.chars().count() <= MOST_CHARS {
        return text.to_string();
    }
    let mut short: String = text.chars().take(MOST_CHARS - 1).collect();
    short.truncate(short.trim_end().len());
    short.push('…');
    short
}

/// A run that can't be taken back once it happens.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Irreversible {
    pub icon: String,
    pub title: String,
}

impl Irreversible {
    pub fn new(icon: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            title: title.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failure_says_its_sentence_and_keeps_the_error_for_the_log() {
        assert_eq!(
            failure_words("Couldn't save the note: Permission denied (os error 13)"),
            (
                "Couldn't save the note".to_string(),
                Some("Permission denied (os error 13)".to_string())
            )
        );
        assert_eq!(
            failure_words("Couldn't save \u{201C}a: b\u{201D} as a script."),
            (
                "Couldn't save \u{201C}a: b\u{201D} as a script.".to_string(),
                None
            )
        );
        assert_eq!(
            failure_words("Permission denied (os error 13)").0,
            "Permission denied"
        );
        assert_eq!(
            failure_words("stale_prompt_submission").0,
            crate::contract::words::SOMETHING_WENT_WRONG
        );
        assert_eq!(failure_words("Opened at 10:30").0, "Opened at 10:30");
    }

    #[test]
    fn a_script_says_one_readable_line() {
        assert_eq!(one_line("  \u{2705} **Saved**\n<b>now</b>  "), "Saved now");
        assert_eq!(one_line("a\u{202E}b\tc"), "ab c");
        assert_eq!(one_line("Use <Enter> to go"), "Use <Enter> to go");
        assert_eq!(one_line("__init__.py saved"), "__init__.py saved");
        assert_eq!(one_line(" \n "), "");
        let long = "x".repeat(MOST_CHARS + 10);
        assert_eq!(one_line(&long).chars().count(), MOST_CHARS);
        assert!(one_line(&long).ends_with('…'));
    }
}
