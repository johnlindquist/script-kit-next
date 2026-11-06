//! Named local operations on an explicitly visible text subject.
use unicode_segmentation::UnicodeSegmentation;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Slug,
    Reverse,
    ReadingTime,
    PlainMarkdown,
    Base64,
    Whitespace,
}
impl Tool {
    pub fn request(phrase: &str) -> Option<Self> {
        Some(match phrase {
            "slug" | "url slug" | "slugify" | "make a slug" => Self::Slug,
            "reverse" | "reverse text" | "reverse letters" | "mirror text" => Self::Reverse,
            "reading time" | "estimate reading time" | "read time" => Self::ReadingTime,
            "markdown"
            | "plain text"
            | "strip markdown"
            | "remove markdown"
            | "markdown to plain text" => Self::PlainMarkdown,
            "whitespace" | "trim whitespace" | "normalize whitespace" | "remove extra spaces" => {
                Self::Whitespace
            }
            "encode text" | "base64" | "base64 encode" | "encode base64" => Self::Base64,
            _ => return None,
        })
    }
    pub fn all() -> [Self; 6] {
        [
            Self::Slug,
            Self::Reverse,
            Self::ReadingTime,
            Self::PlainMarkdown,
            Self::Base64,
            Self::Whitespace,
        ]
    }
    pub fn from_key(key: &str) -> Option<Self> {
        Self::all().into_iter().find(|v| v.key() == key)
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::Slug => "url-slug",
            Self::Reverse => "reverse-text",
            Self::ReadingTime => "reading-time",
            Self::PlainMarkdown => "plain-markdown",
            Self::Base64 => "encode-base64",
            Self::Whitespace => "normalize-whitespace",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Slug => "Make a short URL slug",
            Self::Reverse => "Reverse text",
            Self::ReadingTime => "Estimate reading time",
            Self::PlainMarkdown => "Markdown to plain text",
            Self::Base64 => "Encode text as Base64",
            Self::Whitespace => "Normalize whitespace",
        }
    }
    pub fn detail(self) -> &'static str {
        match self {
            Self::Slug => "Lowercase letters and numbers with hyphens · up to 48 characters",
            Self::Reverse => "Reverse visible characters without splitting emoji",
            Self::ReadingTime => "Estimated at 200 words per minute",
            Self::PlainMarkdown => "Keep the words, link labels and code; omit markup and raw HTML",
            Self::Base64 => "Encoding, not encryption: anyone can recover the text",
            Self::Whitespace => {
                "Remove extra spaces on each line; keep words, case and line breaks"
            }
        }
    }
}
pub fn whitespace(text: &str) -> String {
    let mut out = String::new();
    for record in text.split_inclusive('\n') {
        let (line, suffix) = if let Some(line) = record.strip_suffix("\r\n") {
            (line, "\r\n")
        } else if let Some(line) = record.strip_suffix('\n') {
            (line, "\n")
        } else {
            (record, "")
        };
        for (i, word) in line.split_whitespace().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            out.push_str(word);
        }
        out.push_str(suffix);
    }
    out
}
pub fn reverse(text: &str) -> String {
    text.graphemes(true).rev().collect()
}
pub fn reading_time(text: &str) -> String {
    let words = text.unicode_words().count();
    let seconds = words.saturating_mul(60).div_ceil(200);
    match seconds {
        0 => "0 seconds".into(),
        1 => "About 1 second".into(),
        2..=59 => format!("About {seconds} seconds"),
        _ => {
            let min = seconds / 60;
            let sec = seconds % 60;
            if sec == 0 {
                format!("About {min} min")
            } else {
                format!("About {min} min {sec} s")
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visible_letters_and_reading_estimates() {
        assert_eq!(reverse("A🇫🇷e\u{301}"), "e\u{301}🇫🇷A");
        assert_eq!(whitespace("  A   B \r\n\tC   D\n"), "A B\r\nC D\n");
        assert_eq!(reading_time(""), "0 seconds");
        assert_eq!(reading_time(&"one ".repeat(200)), "About 1 min");
        assert_eq!(reading_time(&"one ".repeat(201)), "About 1 min 1 s");
    }
}
