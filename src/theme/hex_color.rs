//! Hex color parsing and serialization
//!
//! Provides custom serialization/deserialization for HexColor values,
//! supporting multiple input formats: hex strings, RGB/RGBA strings, and numbers.

use serde::{Deserializer, Serializer};

/// Transparent color constant (fully transparent black)
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
                Ok(value as HexColor)
            }

            fn visit_i64<E>(self, value: i64) -> Result<HexColor, E>
            where
                E: de::Error,
            {
                Ok(value as HexColor)
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

        // Try parsing as bare hex (6 characters)
        if s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit()) {
            return parse_hex(s);
        }

        Err(format!(
            "invalid color format '{}' - use #RRGGBB, rgba(r,g,b,a), or a number",
            s
        ))
    }

    fn parse_hex(hex: &str) -> Result<HexColor, String> {
        if hex.len() != 6 {
            return Err(format!("hex color must be 6 characters, got {}", hex.len()));
        }
        u32::from_str_radix(hex, 16).map_err(|_| format!("invalid hex color: {}", hex))
    }
}

/// Wrapper module for Option<HexColor> serialization
pub mod hex_color_option_serde {
    use super::*;
    use serde::de::{self, Visitor};
    use std::fmt;

    #[allow(dead_code)]
    pub fn serialize<S>(color: &Option<HexColor>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match color {
            Some(c) => serializer.serialize_str(&format!("#{:06X}", c)),
            None => serializer.serialize_none(),
        }
    }

    #[allow(dead_code)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<HexColor>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OptionHexColorVisitor;

        impl<'de> Visitor<'de> for OptionHexColorVisitor {
            type Value = Option<HexColor>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("null, a number, hex string, or rgba()")
            }

            fn visit_none<E>(self) -> Result<Option<HexColor>, E>
            where
                E: de::Error,
            {
                Ok(None)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Option<HexColor>, D::Error>
            where
                D: Deserializer<'de>,
            {
                hex_color_serde::deserialize(deserializer).map(Some)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Option<HexColor>, E>
            where
                E: de::Error,
            {
                Ok(Some(value as HexColor))
            }

            fn visit_str<E>(self, value: &str) -> Result<Option<HexColor>, E>
            where
                E: de::Error,
            {
                super::hex_color_serde::parse_color_string(value)
                    .map(Some)
                    .map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(OptionHexColorVisitor)
    }
}
