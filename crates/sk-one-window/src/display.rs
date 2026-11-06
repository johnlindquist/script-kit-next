//! Live, entirely local displays. Recognizing a request never copies text or changes a theme.
use crate::Row;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Cue,
    Color,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Display {
    pub kind: Kind,
    pub text: String,
}
impl Display {
    pub fn request(words: &str) -> Option<Self> {
        let words = words.trim();
        let lower = words.to_ascii_lowercase();
        let (kind, seed) = if matches!(
            lower.as_str(),
            "large type" | "large text" | "show large type"
        ) {
            (Kind::Cue, "")
        } else if lower.starts_with("large type ") {
            (Kind::Cue, words[11..].trim())
        } else if matches!(
            lower.as_str(),
            "color" | "colour" | "color preview" | "colour preview" | "hex color"
        ) {
            (Kind::Color, "")
        } else if let Some((word, rest)) = words
            .split_once(char::is_whitespace)
            .filter(|(v, _)| v.eq_ignore_ascii_case("color") || v.eq_ignore_ascii_case("colour"))
        {
            let _ = word;
            hex(rest.trim())?;
            (Kind::Color, rest.trim())
        } else if words.starts_with('#') && hex(words).is_some() {
            (Kind::Color, words)
        } else {
            return None;
        };
        Some(Self {
            kind,
            text: seed.into(),
        })
    }
    pub fn id(&self) -> &'static str {
        match self.kind {
            Kind::Cue => "large-type",
            Kind::Color => "color-preview",
        }
    }
    pub fn title(&self) -> &'static str {
        match self.kind {
            Kind::Cue => "Large Type",
            Kind::Color => "Color preview",
        }
    }
    pub fn value(&self) -> Option<String> {
        match self.kind {
            Kind::Cue => (!self.text.trim().is_empty()).then(|| self.text.clone()),
            Kind::Color => Some(format!("#{:06X}", hex(self.text.trim())?)),
        }
    }
    pub fn shown(&self) -> String {
        self.value().unwrap_or_else(|| match self.kind {
            Kind::Cue => "Type your cue above".into(),
            Kind::Color => "Enter a hex color, such as #63B7A6".into(),
        })
    }
    pub fn row(&self) -> Row {
        match self.kind {
            Kind::Cue => Row::new("quick-display:large-type", "Show large type")
                .with_sub("Display a cue in large text in this window")
                .with_icon("type")
                .with_enter_label("Open large type"),
            Kind::Color => Row::new(
                "quick-display:color-preview",
                self.value()
                    .map_or_else(|| "Preview a color".to_string(), |s| format!("Preview {s}")),
            )
            .with_sub("See the swatch without changing your theme")
            .with_icon("palette")
            .with_enter_label("Preview color"),
        }
    }
}
pub fn hex(s: &str) -> Option<u32> {
    let digits = s.strip_prefix('#').unwrap_or(s);
    if !digits.bytes().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    match digits.len() {
        6 => u32::from_str_radix(digits, 16).ok(),
        3 => {
            let mut n = 0u32;
            for ch in digits.chars() {
                n = (n << 8) | ch.to_digit(16)? * 17;
            }
            Some(n)
        }
        _ => None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn safe_color_and_cue_requests_keep_their_values() {
        assert_eq!(
            Display::request("color #abc").unwrap().value().as_deref(),
            Some("#AABBCC")
        );
        assert_eq!(
            Display::request("large type Paper Moon").unwrap().text,
            "Paper Moon"
        );
        for s in ["# new note", "#", "color coffee", "large typescript"] {
            assert!(Display::request(s).is_none());
        }
    }
}
