//! Live, entirely local displays. Recognizing a request never copies text or changes a theme.
use crate::Row;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Cue,
    Color,
    Qr,
    Palette,
    Beat,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Display {
    pub kind: Kind,
    pub text: String,
    qr: Option<std::sync::Arc<crate::display_code::QrGrid>>,
    pub beat: Option<Beat>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Beat {
    pub id: u128,
    pub start: u64,
    pub bpm: u16,
    pub number: u8,
}
impl Display {
    pub fn request(words: &str) -> Option<Self> {
        let words = words.trim();
        let lower = words.to_ascii_lowercase();
        let (kind, seed) = if matches!(lower.as_str(), "metronome" | "visual metronome") {
            (Kind::Beat, "90")
        } else if lower.starts_with("metronome ") {
            let suffix = words[10..].trim();
            let value = if suffix.to_ascii_lowercase().ends_with(" bpm") {
                suffix[..suffix.len() - 4].trim()
            } else {
                suffix
            };
            let bpm = value.parse::<u16>().ok()?;
            if !(30..=240).contains(&bpm) {
                return None;
            }
            (Kind::Beat, value)
        } else if matches!(
            lower.as_str(),
            "qr" | "qr code" | "make qr code" | "make a qr code"
        ) {
            (Kind::Qr, "")
        } else if lower.starts_with("qr code ") {
            (Kind::Qr, words[8..].trim())
        } else if matches!(
            lower.as_str(),
            "palette" | "color palette" | "colour palette" | "palette generator"
        ) {
            (Kind::Palette, "63B7A6")
        } else if lower.starts_with("palette ") {
            let seed = words[8..].trim();
            hex(seed)?;
            (Kind::Palette, seed)
        } else if matches!(
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
            qr: None,
            beat: None,
        })
    }
    pub fn activate(&mut self) {
        if self.kind == Kind::Qr {
            self.qr = crate::display_code::QrGrid::encode(&self.text).map(std::sync::Arc::new);
        }
    }
    pub fn set_text(&mut self, text: String) {
        if self.text != text {
            self.text = text;
            self.beat = None;
            self.activate();
        }
    }
    pub fn bpm(&self) -> Option<u16> {
        let n = self.text.trim().parse::<u16>().ok()?;
        (self.kind == Kind::Beat && (30..=240).contains(&n)).then_some(n)
    }
    pub fn toggle_beat(&mut self, id: u128, now: u64) -> Option<u64> {
        if self.beat.take().is_some() {
            return None;
        }
        let bpm = self.bpm()?;
        self.beat = Some(Beat {
            id,
            start: now,
            bpm,
            number: 1,
        });
        Some(60000u64.div_ceil(u64::from(bpm)))
    }
    pub fn advance_beat(&mut self, id: u128, now: u64) -> Option<u64> {
        let b = self.beat.as_mut().filter(|v| v.id == id)?;
        let elapsed = now.saturating_sub(b.start);
        let rate = u64::from(b.bpm);
        let count = elapsed.saturating_mul(rate) / 60000;
        b.number = (count % 4) as u8 + 1;
        Some(
            ((count + 1).saturating_mul(60000).div_ceil(rate))
                .saturating_sub(elapsed)
                .max(1),
        )
    }
    pub fn qr(&self) -> Option<&std::sync::Arc<crate::display_code::QrGrid>> {
        self.qr.as_ref()
    }
    pub fn palette(&self) -> Option<[u32; 5]> {
        if self.kind != Kind::Palette {
            return None;
        }
        Some(crate::display_code::tones(hex(self.text.trim())?))
    }
    pub fn copy_available(&self) -> bool {
        match self.kind {
            Kind::Beat => false,
            Kind::Qr => self.qr.is_some(),
            Kind::Palette => self.palette().is_some(),
            Kind::Cue => !self.text.trim().is_empty(),
            Kind::Color => hex(self.text.trim()).is_some(),
        }
    }
    pub fn enter_label(&self) -> &'static str {
        match (self.kind, self.copy_available()) {
            (Kind::Beat, _) => {
                if self.beat.is_some() {
                    "Stop beat"
                } else if self.bpm().is_some() {
                    "Start beat"
                } else {
                    "Enter 30–240 BPM"
                }
            }
            (Kind::Qr, true) => "Copy QR as SVG",
            (Kind::Palette, true) => "Copy palette",
            (Kind::Color, true) => "Copy hex",
            (Kind::Cue, true) => "Copy cue",
            (Kind::Qr, false) => "Enter QR text",
            (Kind::Color | Kind::Palette, false) => "Enter a hex color",
            (Kind::Cue, false) => "Type a cue",
        }
    }
    pub fn copied_label(&self) -> &'static str {
        match self.kind {
            Kind::Beat => "Visual metronome",
            Kind::Qr => "Copied QR as SVG",
            Kind::Palette => "Copied palette",
            Kind::Color => "Copied hex color",
            Kind::Cue => "Copied cue",
        }
    }
    pub fn id(&self) -> &'static str {
        match self.kind {
            Kind::Cue => "large-type",
            Kind::Color => "color-preview",
            Kind::Qr => "qr-code",
            Kind::Beat => "visual-metronome",
            Kind::Palette => "color-palette",
        }
    }
    pub fn title(&self) -> &'static str {
        match self.kind {
            Kind::Cue => "Large Type",
            Kind::Color => "Color preview",
            Kind::Qr => "QR code",
            Kind::Beat => "Visual metronome",
            Kind::Palette => "Color palette",
        }
    }
    pub fn value(&self) -> Option<String> {
        match self.kind {
            Kind::Cue => (!self.text.trim().is_empty()).then(|| self.text.clone()),
            Kind::Color => Some(format!("#{:06X}", hex(self.text.trim())?)),
            Kind::Qr => Some(self.qr.as_ref()?.svg()),
            Kind::Beat => None,
            Kind::Palette => Some(self.palette()?.map(|rgb| format!("#{rgb:06X}")).join("\n")),
        }
    }
    pub fn shown(&self) -> String {
        match self.kind {
            Kind::Beat => match (self.bpm(), self.beat) {
                (Some(n), Some(b)) => format!("Beat {} of 4 · {n} BPM", b.number),
                (Some(n), None) => format!("{n} BPM · ready"),
                _ => "Enter 30–240 BPM".into(),
            },
            Kind::Cue => self.value().unwrap_or_else(|| "Type your cue above".into()),
            Kind::Color => self
                .value()
                .unwrap_or_else(|| "Enter a hex color, such as #63B7A6".into()),
            Kind::Palette => {
                if self.copy_available() {
                    "Five tones · change the hex color above".into()
                } else {
                    "Enter a hex color, such as #63B7A6".into()
                }
            }
            Kind::Qr => {
                if self.qr.is_some() {
                    format!("Encoded locally · {} bytes", self.text.len())
                } else if self.text.len() > crate::display_code::MAX_QR_BYTES {
                    format!(
                        "Use at most {} UTF-8 bytes",
                        crate::display_code::MAX_QR_BYTES
                    )
                } else {
                    "Type the text to encode above".into()
                }
            }
        }
    }
    pub fn row(&self) -> Row {
        match self.kind {
            Kind::Beat => Row::new("quick-display:visual-metronome", "Open a visual metronome")
                .with_sub("A silent four-beat pulse; set the tempo and press Start")
                .with_enter_label("Open metronome"),
            Kind::Qr => Row::new("quick-display:qr-code", "Make a QR code")
                .with_sub("Encode text locally; scan the preview or copy an SVG")
                .with_icon("grid")
                .with_enter_label("Open QR code"),
            Kind::Palette => Row::new("quick-display:color-palette", "Generate a color palette")
                .with_sub("See five tones of one seed color without changing your theme")
                .with_icon("palette")
                .with_enter_label("Open palette"),
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
                n = (n << 8) | (ch.to_digit(16)? * 17);
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
    fn beat_timing_is_monotonic_and_stale_generations_do_not_restart_it() {
        let mut view = Display::request("metronome 120 bpm").unwrap();
        assert_eq!(view.toggle_beat(8, 1000), Some(500));
        assert_eq!(view.advance_beat(8, 2500), Some(500));
        assert_eq!(view.beat.unwrap().number, 4);
        assert_eq!(view.advance_beat(7, 3000), None);
        view.set_text("90".into());
        assert!(view.beat.is_none());
        assert_eq!(view.advance_beat(8, 3000), None);
        assert!(Display::request("metronome 999").is_none());
    }
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
