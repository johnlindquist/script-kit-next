#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextMetrics {
    pub bytes: usize,
    pub chars: usize,
    pub utf16_units: usize,
    pub words: usize,
    pub lines: usize,
    pub estimated_tokens: usize,
}

impl TextMetrics {
    pub fn from_text(text: &str) -> Self {
        let chars = text.chars().count();
        Self {
            bytes: text.len(),
            chars,
            utf16_units: text.encode_utf16().count(),
            words: text.split_whitespace().count(),
            lines: if text.is_empty() {
                0
            } else {
                text.lines().count().max(1)
            },
            estimated_tokens: chars.div_ceil(4),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TextMetrics;

    #[test]
    fn text_metrics_counts_words_without_content_leakage() {
        let metrics = TextMetrics::from_text("alpha beta\ngamma");
        assert_eq!(metrics.words, 3);
        assert_eq!(TextMetrics::from_text("").words, 0);
    }
}
