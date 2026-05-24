#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextMetrics {
    pub bytes: usize,
    pub chars: usize,
    pub utf16_units: usize,
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
            lines: if text.is_empty() {
                0
            } else {
                text.lines().count().max(1)
            },
            estimated_tokens: chars.div_ceil(4),
        }
    }
}
