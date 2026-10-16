/// Content types for clipboard entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum ContentType {
    Text,
    Image,
    Link,
    File,
    Color,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Image => "image",
            ContentType::Link => "link",
            ContentType::File => "file",
            ContentType::Color => "color",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        <Self as std::str::FromStr>::from_str(s).unwrap_or(Self::Text)
    }
}

pub fn classify_content(text: &str, has_image: bool) -> ContentType {
    if has_image {
        return ContentType::Image;
    }
    classify_text(text)
}

/// What kind of text was copied: a lone link, a color, a file path, or text.
///
/// Only a whole, single-line copy counts as a link, color, or path; prose
/// that mentions a URL is still text.
pub fn classify_text(text: &str) -> ContentType {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.contains('\n') {
        return ContentType::Text;
    }
    if parse_color(trimmed).is_some() {
        return ContentType::Color;
    }
    if trimmed.contains(char::is_whitespace) {
        return ContentType::Text;
    }
    if is_link(trimmed) {
        return ContentType::Link;
    }
    if (trimmed.starts_with('/') || trimmed.starts_with("~/")) && trimmed.len() > 1 {
        return ContentType::File;
    }
    ContentType::Text
}

fn is_link(text: &str) -> bool {
    if let Some(rest) = text.strip_prefix("mailto:") {
        return rest.contains('@');
    }
    let Some((scheme, rest)) = text.split_once("://") else {
        return false;
    };
    !scheme.is_empty()
        && scheme
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
        && scheme
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic())
        && !rest.is_empty()
}

/// A color as 8-bit red, green, blue, and alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    /// `#RRGGBB`, or `#RRGGBBAA` when not opaque.
    pub fn hex(self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    /// `rgb(r, g, b)`, or `rgba(r, g, b, a)` when not opaque.
    pub fn rgb(self) -> String {
        if self.a == 255 {
            format!("rgb({}, {}, {})", self.r, self.g, self.b)
        } else {
            format!(
                "rgba({}, {}, {}, {})",
                self.r,
                self.g,
                self.b,
                alpha_text(self.a)
            )
        }
    }

    /// `hsl(h, s%, l%)`, or `hsla(...)` when not opaque.
    pub fn hsl(self) -> String {
        let (r, g, b) = (
            f64::from(self.r) / 255.0,
            f64::from(self.g) / 255.0,
            f64::from(self.b) / 255.0,
        );
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let lightness = (max + min) / 2.0;
        let delta = max - min;
        let (hue, saturation) = if delta == 0.0 {
            (0.0, 0.0)
        } else {
            let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
            let hue = if max == r {
                60.0 * (((g - b) / delta).rem_euclid(6.0))
            } else if max == g {
                60.0 * ((b - r) / delta + 2.0)
            } else {
                60.0 * ((r - g) / delta + 4.0)
            };
            (hue, saturation)
        };
        let (h, s, l) = (
            hue.round() as i64 % 360,
            (saturation * 100.0).round() as i64,
            (lightness * 100.0).round() as i64,
        );
        if self.a == 255 {
            format!("hsl({h}, {s}%, {l}%)")
        } else {
            format!("hsla({h}, {s}%, {l}%, {})", alpha_text(self.a))
        }
    }
}

fn alpha_text(alpha: u8) -> String {
    let value = (f64::from(alpha) / 255.0 * 100.0).round() / 100.0;
    let text = format!("{value:.2}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Parse a CSS-style color: `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`,
/// `rgb(...)`/`rgba(...)` with numbers, or `hsl(...)`/`hsla(...)`.
pub fn parse_color(text: &str) -> Option<Rgba> {
    let text = text.trim();
    if let Some(hex) = text.strip_prefix('#') {
        return parse_hex(hex);
    }
    let lower = text.to_ascii_lowercase();
    let (name, args) = lower.strip_suffix(')')?.split_once('(')?;
    let parts: Vec<&str> = args
        .split([',', ' ', '/'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if !(3..=4).contains(&parts.len()) {
        return None;
    }
    let alpha = match parts.get(3) {
        Some(part) => parse_alpha(part)?,
        None => 255,
    };
    match name {
        "rgb" | "rgba" => {
            let channel = |part: &str| -> Option<u8> {
                let value = match part.strip_suffix('%') {
                    Some(percent) => percent.parse::<f64>().ok()? * 2.55,
                    None => part.parse::<f64>().ok()?,
                };
                (0.0..=255.0).contains(&value).then(|| value.round() as u8)
            };
            Some(Rgba {
                r: channel(parts[0])?,
                g: channel(parts[1])?,
                b: channel(parts[2])?,
                a: alpha,
            })
        }
        "hsl" | "hsla" => {
            let hue = parts[0].trim_end_matches("deg").parse::<f64>().ok()?;
            let percent = |part: &str| -> Option<f64> {
                let value = part.strip_suffix('%')?.parse::<f64>().ok()?;
                (0.0..=100.0).contains(&value).then_some(value / 100.0)
            };
            let (s, l) = (percent(parts[1])?, percent(parts[2])?);
            let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
            let h = hue.rem_euclid(360.0) / 60.0;
            let x = c * (1.0 - (h.rem_euclid(2.0) - 1.0).abs());
            let (r, g, b) = match h as u32 {
                0 => (c, x, 0.0),
                1 => (x, c, 0.0),
                2 => (0.0, c, x),
                3 => (0.0, x, c),
                4 => (x, 0.0, c),
                _ => (c, 0.0, x),
            };
            let m = l - c / 2.0;
            let to_byte = |value: f64| ((value + m) * 255.0).round().clamp(0.0, 255.0) as u8;
            Some(Rgba {
                r: to_byte(r),
                g: to_byte(g),
                b: to_byte(b),
                a: alpha,
            })
        }
        _ => None,
    }
}

fn parse_alpha(part: &str) -> Option<u8> {
    let value = match part.strip_suffix('%') {
        Some(percent) => percent.parse::<f64>().ok()? / 100.0,
        None => part.parse::<f64>().ok()?,
    };
    (0.0..=1.0)
        .contains(&value)
        .then(|| (value * 255.0).round() as u8)
}

fn parse_hex(hex: &str) -> Option<Rgba> {
    if !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }
    let digit = |index: usize| {
        u8::from_str_radix(&hex[index..=index], 16)
            .ok()
            .map(|v| v * 17)
    };
    let pair = |index: usize| u8::from_str_radix(&hex[index..index + 2], 16).ok();
    match hex.len() {
        3 | 4 => Some(Rgba {
            r: digit(0)?,
            g: digit(1)?,
            b: digit(2)?,
            a: if hex.len() == 4 { digit(3)? } else { 255 },
        }),
        6 | 8 => Some(Rgba {
            r: pair(0)?,
            g: pair(2)?,
            b: pair(4)?,
            a: if hex.len() == 8 { pair(6)? } else { 255 },
        }),
        _ => None,
    }
}

/// A single clipboard history entry (full, includes content).
#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub id: String,
    pub content: String,
    pub content_type: ContentType,
    pub timestamp: i64,
    pub pinned: bool,
    /// OCR text extracted from images (None for text entries or pending OCR).
    #[allow(dead_code)]
    pub ocr_text: Option<String>,
    /// Human-readable source application name (for example, "Safari").
    #[allow(dead_code)]
    pub source_app_name: Option<String>,
    /// Source application bundle identifier (for example, "com.apple.Safari").
    #[allow(dead_code)]
    pub source_app_bundle_id: Option<String>,
}

/// Lightweight clipboard entry metadata for list views (no payload).
///
/// This struct contains everything needed for displaying entries in a list
/// without loading the full content (which can be megabytes for images).
/// Use the app-owned content service to fetch the full content when needed.
#[derive(Debug, Clone)]
pub struct ClipboardEntryMeta {
    pub id: String,
    pub content_type: ContentType,
    pub timestamp: i64,
    pub pinned: bool,
    /// First 100 chars of text content (for list preview), or "[Image]" for images.
    pub text_preview: String,
    /// Image width in pixels (None for text).
    pub image_width: Option<u32>,
    /// Image height in pixels (None for text).
    pub image_height: Option<u32>,
    /// Content size in bytes (useful for displaying file sizes).
    #[allow(dead_code)]
    pub byte_size: usize,
    /// OCR text extracted from images (None for text entries or pending OCR).
    #[allow(dead_code)]
    pub ocr_text: Option<String>,
}

impl ClipboardEntryMeta {
    /// Get a display-friendly preview string for list items.
    pub fn display_preview(&self) -> String {
        match self.content_type {
            ContentType::Image => {
                if let (Some(w), Some(h)) = (self.image_width, self.image_height) {
                    format!("{}×{} image", w, h)
                } else {
                    "[Image]".to_string()
                }
            }
            ContentType::Text | ContentType::Link | ContentType::File | ContentType::Color => {
                let sanitized = self.text_preview.replace(['\n', '\r'], " ");
                // Gate the ellipsis on the *character* count, not the byte length:
                // `chars().take(50)` truncates by characters, so using `len()` (bytes)
                // here appended a "…" to short multibyte text (emoji/CJK) that was
                // never actually truncated.
                if sanitized.chars().count() > 50 {
                    format!("{}…", sanitized.chars().take(50).collect::<String>())
                } else {
                    sanitized
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_type_conversion_is_stable() {
        assert_eq!(ContentType::Text.as_str(), "text");
        assert_eq!(ContentType::Image.as_str(), "image");
        assert_eq!(ContentType::Link.as_str(), "link");
        assert_eq!(ContentType::File.as_str(), "file");
        assert_eq!(ContentType::Color.as_str(), "color");
        assert_eq!(ContentType::from_str("text"), ContentType::Text);
        assert_eq!(ContentType::from_str("image"), ContentType::Image);
        assert_eq!(ContentType::from_str("link"), ContentType::Link);
        assert_eq!(ContentType::from_str("file"), ContentType::File);
        assert_eq!(ContentType::from_str("color"), ContentType::Color);
        assert_eq!(ContentType::from_str("unknown"), ContentType::Text);
    }

    #[test]
    fn whole_single_line_copies_are_links_colors_or_paths() {
        assert_eq!(
            classify_text("https://example.com/a?b=1"),
            ContentType::Link
        );
        assert_eq!(classify_text("  mailto:me@example.com "), ContentType::Link);
        assert_eq!(
            classify_text("see https://example.com for more"),
            ContentType::Text
        );
        assert_eq!(classify_text("#ff8800"), ContentType::Color);
        assert_eq!(classify_text("rgb(255, 136, 0)"), ContentType::Color);
        assert_eq!(classify_text("hsl(32 100% 50%)"), ContentType::Color);
        assert_eq!(classify_text("#ff8800\n#00ff00"), ContentType::Text);
        assert_eq!(classify_text("/Users/me/file.txt"), ContentType::File);
        assert_eq!(classify_text("~/notes.md"), ContentType::File);
        assert_eq!(classify_text("#hashtag"), ContentType::Text);
        assert_eq!(classify_text("/"), ContentType::Text);
        assert_eq!(classify_text("hello"), ContentType::Text);
    }

    #[test]
    fn colors_parse_and_convert_between_formats() {
        let orange = parse_color("#F80").unwrap();
        assert_eq!(
            orange,
            Rgba {
                r: 255,
                g: 136,
                b: 0,
                a: 255
            }
        );
        assert_eq!(orange.hex(), "#FF8800");
        assert_eq!(orange.rgb(), "rgb(255, 136, 0)");
        assert_eq!(orange.hsl(), "hsl(32, 100%, 50%)");
        assert_eq!(parse_color("hsl(32, 100%, 50%)").unwrap().hex(), "#FF8800");
        assert_eq!(
            parse_color("rgba(0, 0, 0, 0.5)").unwrap().hex(),
            "#00000080"
        );
        assert_eq!(
            parse_color("#00000080").unwrap().rgb(),
            "rgba(0, 0, 0, 0.5)"
        );
        assert_eq!(parse_color("rgb(0 128 255 / 50%)").unwrap().a, 128);
        assert_eq!(parse_color("#fff").unwrap().hsl(), "hsl(0, 0%, 100%)");
        assert!(parse_color("#12345").is_none());
        assert!(parse_color("rgb(300, 0, 0)").is_none());
        assert!(parse_color("blue").is_none());
    }

    #[test]
    fn classify_content_prefers_image_payloads() {
        assert_eq!(
            classify_content("https://example.com", true),
            ContentType::Image
        );
    }

    #[test]
    fn classify_content_recognizes_links() {
        assert_eq!(
            classify_content("http://example.com", false),
            ContentType::Link
        );
        assert_eq!(
            classify_content("https://example.com", false),
            ContentType::Link
        );
        assert_eq!(
            classify_content("custom-scheme://resource", false),
            ContentType::Link
        );
    }

    #[test]
    fn classify_content_recognizes_paths() {
        assert_eq!(classify_content("/tmp/file.txt", false), ContentType::File);
        assert_eq!(classify_content("~/notes.md", false), ContentType::File);
    }

    #[test]
    fn classify_content_defaults_to_text() {
        assert_eq!(
            classify_content("just some plain text", false),
            ContentType::Text
        );
    }

    fn meta(content_type: ContentType, preview: &str) -> ClipboardEntryMeta {
        ClipboardEntryMeta {
            id: "clip-1".to_string(),
            content_type,
            timestamp: 1_778_000_000_000,
            pinned: false,
            text_preview: preview.to_string(),
            image_width: None,
            image_height: None,
            byte_size: preview.len(),
            ocr_text: None,
        }
    }

    #[test]
    fn display_preview_preserves_image_dimensions_and_fallback() {
        let mut entry = meta(ContentType::Image, "[Image]");
        assert_eq!(entry.display_preview(), "[Image]");
        entry.image_width = Some(1440);
        entry.image_height = Some(900);
        assert_eq!(entry.display_preview(), "1440×900 image");
    }

    #[test]
    fn display_preview_does_not_add_ellipsis_for_short_multibyte_text() {
        // Regression: the truncation threshold used `sanitized.len()` (bytes) while
        // the truncation itself used `chars().take(50)`. A short-but-multibyte
        // preview (e.g. 30 two-byte chars = 60 bytes) tripped the byte threshold and
        // got a "…" appended even though nothing was actually cut.
        let text = "é".repeat(30); // 30 chars, 60 bytes
        let entry = meta(ContentType::Text, &text);
        let preview = entry.display_preview();
        assert!(
            !preview.ends_with('…'),
            "no ellipsis when nothing is truncated, got: {preview:?}"
        );
        assert_eq!(preview.chars().count(), 30);
    }

    #[test]
    fn display_preview_truncates_long_multibyte_text_to_50_chars() {
        // 60 multibyte chars > 50-char limit → truncate to exactly 50 chars + "…".
        let text = "🌊".repeat(60);
        let entry = meta(ContentType::Text, &text);
        let preview = entry.display_preview();
        assert!(preview.ends_with('…'));
        assert_eq!(preview.chars().count(), 51); // 50 content chars + ellipsis
    }

    #[test]
    fn display_preview_flattens_and_truncates_text() {
        let entry = meta(
            ContentType::Text,
            "first line\r\nsecond line with enough trailing text to exceed fifty characters",
        );
        let preview = entry.display_preview();
        assert!(!preview.contains(['\r', '\n']));
        assert!(preview.ends_with('…'));
        assert_eq!(preview.chars().count(), 51);
    }
}
