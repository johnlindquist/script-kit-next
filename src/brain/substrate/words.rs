//! Word counting and excerpt generation for fragments.

/// Count whitespace-delimited words in `text`.
pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// First `max_words` words of `text`, never cutting mid-word. Appends `...` when
/// truncated.
pub fn excerpt_words(text: &str, max_words: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= max_words {
        words.join(" ")
    } else {
        format!("{}...", words[..max_words].join(" "))
    }
}
