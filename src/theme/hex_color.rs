//! Hex color parsing and serialization
//!
//! Provides custom serialization/deserialization for HexColor values,
//! supporting multiple input formats: hex strings, RGB/RGBA strings, and numbers.

use serde::{Deserializer, Serializer};

/// Transparent color constant (fully transparent black)
#[cfg(test)]
pub const TRANSPARENT: u32 = 0x00000000;

/// Hex color representation (u32)
/// Supports deserialization from:
/// - Numbers: `1973790`
/// - Hex strings: `"#1E1E1E"` or `"1E1E1E"` or `"0x1E1E1E"`
/// - RGB/RGBA strings: `"rgb(30, 30, 30)"` or `"rgba(30, 30, 30, 1.0)"`
pub type HexColor = u32;

/// Custom serialization/deserialization for HexColor
/// Serializes as hex string "#RRGGBB" for readability
/// Deserializes from number, hex string, or rgba() format
pub mod hex_color_serde {
    use super::*;
    use serde::de::{self, Visitor};
    use std::fmt;

    pub fn serialize<S>(color: &HexColor, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as "#RRGGBB" hex string for readability
        serializer.serialize_str(&format!("#{:06X}", color))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HexColor, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HexColorVisitor;

        impl<'de> Visitor<'de> for HexColorVisitor {
            type Value = HexColor;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a number, hex string (#RRGGBB), or rgba(r, g, b, a)")
            }

            fn visit_u64<E>(self, value: u64) -> Result<HexColor, E>
            where
                E: de::Error,
            {
                if value > u64::from(u32::MAX) {
                    return Err(de::Error::custom("color value exceeds u32::MAX"));
                }
                Ok(value as HexColor)
            }

            fn visit_i64<E>(self, value: i64) -> Result<HexColor, E>
            where
                E: de::Error,
            {
                if value < 0 {
                    return Err(de::Error::custom("color value cannot be negative"));
                }
                self.visit_u64(value as u64)
            }

            fn visit_str<E>(self, value: &str) -> Result<HexColor, E>
            where
                E: de::Error,
            {
                parse_color_string(value).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(HexColorVisitor)
    }

    /// Parse a color string into a HexColor
    /// Supports: "#RRGGBB", "RRGGBB", "0xRRGGBB", "rgb(r,g,b)", "rgba(r,g,b,a)"
    pub fn parse_color_string(s: &str) -> Result<HexColor, String> {
        let s = s.trim();

        // Handle hex formats: #RRGGBB, RRGGBB, 0xRRGGBB
        if let Some(hex) = s.strip_prefix('#') {
            return parse_hex(hex);
        }
        if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            return parse_hex(hex);
        }

        // Handle rgba(r, g, b, a) format
        if let Some(inner) = s.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 4 {
                let r: u8 = parts[0].parse().map_err(|_| "invalid red value")?;
                let g: u8 = parts[1].parse().map_err(|_| "invalid green value")?;
                let b: u8 = parts[2].parse().map_err(|_| "invalid blue value")?;
                // Alpha is ignored for HexColor (RGB only)
                return Ok(((r as u32) << 16) | ((g as u32) << 8) | (b as u32));
            }
            return Err("rgba() requires 4 values: r, g, b, a".to_string());
        }

        // Handle rgb(r, g, b) format
        if let Some(inner) = s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 3 {
                let r: u8 = parts[0].parse().map_err(|_| "invalid red value")?;
                let g: u8 = parts[1].parse().map_err(|_| "invalid green value")?;
                let b: u8 = parts[2].parse().map_err(|_| "invalid blue value")?;
                return Ok(((r as u32) << 16) | ((g as u32) << 8) | (b as u32));
            }
            return Err("rgb() requires 3 values: r, g, b".to_string());
        }

        // Try parsing as bare hex
        if s.chars().all(|c| c.is_ascii_hexdigit()) {
            return parse_hex(s);
        }

        Err(format!(
            "invalid color format '{}' - use #RRGGBB, rgba(r,g,b,a), or a number",
            s
        ))
    }

    fn parse_hex(hex: &str) -> Result<HexColor, String> {
        let hex = hex.trim();

        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("invalid hex color: {}", hex));
        }

        let normalized = match hex.len() {
            3 => {
                let mut expanded = String::with_capacity(6);
                for ch in hex.chars() {
                    expanded.push(ch);
                    expanded.push(ch);
                }
                expanded
            }
            4 => {
                let mut expanded = String::with_capacity(6);
                for ch in hex.chars().take(3) {
                    expanded.push(ch);
                    expanded.push(ch);
                }
                expanded
            }
            6 => hex.to_string(),
            8 => hex
                .get(..6)
                .ok_or_else(|| format!("invalid hex color: {}", hex))?
                .to_string(),
            _ => {
                return Err(format!(
                    "hex color must be 3, 4, 6, or 8 characters, got {}",
                    hex.len()
                ));
            }
        };

        u32::from_str_radix(&normalized, 16).map_err(|_| format!("invalid hex color: {}", hex))
    }

    #[cfg(test)]
    mod tests {
        use super::{parse_color_string, parse_hex, HexColor};
        use serde::Deserialize;

        #[derive(Debug, Deserialize)]
        struct HexColorWrapper {
            #[serde(deserialize_with = "super::deserialize")]
            color: HexColor,
        }

        #[test]
        fn test_deserialize_rejects_negative_i64_value_when_numeric_json() {
            let error = serde_json::from_str::<HexColorWrapper>(r#"{"color":-1}"#)
                .expect_err("negative color value should not wrap to 0xFFFFFFFF");

            assert!(
                error.to_string().contains("color value cannot be negative"),
                "unexpected error: {error}"
            );
        }

        #[test]
        fn test_deserialize_accepts_u64_value_within_u32_range() {
            let parsed = serde_json::from_str::<HexColorWrapper>(r#"{"color":16777215}"#)
                .expect("value within u32 range should parse");

            assert_eq!(parsed.color, 0xFFFFFF);
        }

        #[test]
        fn test_deserialize_rejects_u64_value_when_above_u32_max() {
            let error = serde_json::from_str::<HexColorWrapper>(r#"{"color":4294967296}"#)
                .expect_err("value above u32::MAX should fail");

            assert!(
                error.to_string().contains("color value exceeds u32::MAX"),
                "unexpected error: {error}"
            );
        }

        #[test]
        fn test_parse_color_string_expands_rgb_when_hex_len_is_3() {
            assert_eq!(
                parse_color_string("fff").expect("3-digit hex should parse"),
                0xFFFFFF
            );
        }

        #[test]
        fn test_parse_color_string_ignores_alpha_when_hex_len_is_4() {
            assert_eq!(
                parse_color_string("FFFA").expect("4-digit hex should parse as RGB"),
                0xFFFFFF
            );
        }

        #[test]
        fn test_parse_color_string_parses_rgb_when_hex_len_is_6() {
            assert_eq!(
                parse_color_string("1E1E1E").expect("6-digit hex should parse"),
                0x1E1E1E
            );
        }

        #[test]
        fn test_parse_color_string_ignores_alpha_when_hex_len_is_8() {
            assert_eq!(
                parse_color_string("1E1E1EFF").expect("8-digit hex should parse as RGB"),
                0x1E1E1E
            );
        }

        #[test]
        fn test_parse_color_string_rejects_invalid_hex_length_when_bare_hex() {
            assert!(parse_color_string("ABCDE").is_err());
        }

        #[test]
        fn test_parse_color_string_rejects_non_hex_chars_for_prefixed_hex() {
            assert!(parse_color_string("#112233GG").is_err());
        }

        #[test]
        fn test_parse_hex_trims_whitespace_before_parsing() {
            assert_eq!(
                parse_hex(" 1E1E1E ").expect("hex parser should trim leading/trailing whitespace"),
                0x1E1E1E
            );
        }
    }
}
