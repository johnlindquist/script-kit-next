//! Local line tools; no clipboard access, network, clock or writes.
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineAction {
    Sort,
    Unique,
}
impl LineAction {
    pub fn request(words: &str) -> Option<Self> {
        match words {
            "sort" | "sort lines" | "sort these lines" | "alphabetize" | "alphabetize lines" => {
                Some(Self::Sort)
            }
            "remove duplicates"
            | "remove duplicate lines"
            | "deduplicate"
            | "deduplicate lines"
            | "unique lines" => Some(Self::Unique),
            _ => None,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::Sort => "sort-lines",
            Self::Unique => "unique-lines",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Sort => "Sort lines alphabetically",
            Self::Unique => "Remove duplicate lines",
        }
    }
    pub fn hint(self) -> &'static str {
        match self {
            Self::Sort => "Sort by lowercase spelling; leave the original untouched",
            Self::Unique => "Keep the first copy of each exact line, in its original order",
        }
    }
    pub fn apply(self, text: &str) -> String {
        let mut lines: Vec<&str> = text.lines().collect();
        match self {
            Self::Sort => lines.sort_by_cached_key(|s| s.to_lowercase()),
            Self::Unique => {
                let mut seen = HashSet::new();
                lines.retain(|line| seen.insert(*line));
            }
        }
        let crlf = text
            .as_bytes()
            .iter()
            .enumerate()
            .filter(|(_, b)| **b == b'\n')
            .all(|(i, _)| i > 0 && text.as_bytes()[i - 1] == b'\r');
        let newline = if crlf { "\r\n" } else { "\n" };
        let mut result = lines.join(newline);
        if text.ends_with('\n') {
            result.push_str(newline);
        }
        result
    }
}

pub fn wants_compare(words: &str) -> bool {
    matches!(
        words.trim().to_ascii_lowercase().as_str(),
        "compare" | "compare text" | "compare passages" | "compare these" | "text diff"
    )
}

/// Identify the first differing line, without constructing a quadratic edit matrix or
/// presenting truncated snippets as a complete diff. Distinguish line endings from text.
pub fn first_difference(selection: &str, copy: &str) -> String {
    if selection == copy {
        return "The texts are identical.".into();
    }
    let mut a = selection.lines();
    let mut b = copy.lines();
    let mut index = 0;
    let excerpt = |s: Option<&str>| match s {
        None => "(end of text)".to_owned(),
        Some("") => "(empty line)".to_owned(),
        Some(v) => {
            let mut chars = v.chars();
            let mut head: String = chars.by_ref().take(150).collect();
            if chars.next().is_some() {
                head.push('…');
            }
            head
        }
    };
    loop {
        let (left, right) = (a.next(), b.next());
        index += 1;
        if left != right {
            return format!(
                "Line {index} differs\nSelected: {}\nCopied: {}",
                excerpt(left),
                excerpt(right)
            );
        }
        if left.is_none() {
            return "The lines contain the same text, but the line endings differ.".into();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sorting_and_dedup_have_different_order_and_blank_line_rules() {
        let v = "Cedar\r\napple\r\nCedar\r\nApple\r\n";
        assert_eq!(
            LineAction::Sort.apply(v),
            "apple\r\nApple\r\nCedar\r\nCedar\r\n"
        );
        assert_eq!(LineAction::Unique.apply(v), "Cedar\r\napple\r\nApple\r\n");
        assert_eq!(LineAction::Unique.apply(" x\n\n x\nx\n"), " x\n\nx\n");
        assert_eq!(LineAction::Sort.apply(""), "");
        assert_eq!(LineAction::Unique.apply("one"), "one");
    }
    #[test]
    fn a_difference_can_be_a_missing_line_or_just_a_terminator() {
        assert_eq!(
            first_difference("A\nB", "A\nC"),
            "Line 2 differs\nSelected: B\nCopied: C"
        );
        assert!(first_difference("A\nB", "A").contains("(end of text)"));
        assert!(first_difference("A", "A\n").contains("line endings differ"));
        assert_eq!(first_difference("A", "A"), "The texts are identical.");
    }
}
