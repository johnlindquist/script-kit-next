//! Canonical semantic opacity tokens used across the app.
//!
//! These constants intentionally define opacity in the standard direction:
//! `0.0` is fully transparent and `1.0` is fully opaque.
//! Keep all new UI opacity values sourced from this module.

/// Assistant bubble tint background.
pub const OPACITY_MESSAGE_ASSISTANT_BACKGROUND: f32 = 0.10;
/// User bubble tint background.
pub const OPACITY_MESSAGE_USER_BACKGROUND: f32 = 0.12;
/// Very subtle tint/background accents.
pub const OPACITY_SUBTLE: f32 = 0.15;
/// Generic hover surfaces.
pub const OPACITY_HOVER: f32 = 0.30;
/// Muted UI text/surfaces.
pub const OPACITY_MUTED: f32 = OPACITY_HOVER;
/// Disabled state content.
pub const OPACITY_DISABLED: f32 = 0.40;
/// Medium border emphasis.
pub const OPACITY_BORDER: f32 = 0.45;
/// Selected/highlighted state.
pub const OPACITY_SELECTED: f32 = 0.50;
/// Secondary icon emphasis.
pub const OPACITY_ICON_MUTED: f32 = 0.55;
/// Medium accent emphasis.
pub const OPACITY_ACCENT_MEDIUM: f32 = 0.60;
/// Secondary label/body text emphasis.
pub const OPACITY_TEXT_MUTED: f32 = 0.65;
/// Strong emphasis.
pub const OPACITY_STRONG: f32 = 0.70;
/// Positive feedback emphasis.
pub const OPACITY_SUCCESS: f32 = 0.80;
/// Near-full emphasis.
pub const OPACITY_NEAR_FULL: f32 = 0.85;

/// Message bubble border emphasis.
pub const OPACITY_MESSAGE_BORDER: f32 = OPACITY_BORDER;

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_opacity_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.000_001,
            "expected {}, got {}",
            expected,
            actual
        );
    }

    #[test]
    fn test_ai_legacy_opacity_values_remain_available_via_canonical_tokens() {
        assert_opacity_eq(OPACITY_MESSAGE_ASSISTANT_BACKGROUND, 0.10);
        assert_opacity_eq(OPACITY_MESSAGE_USER_BACKGROUND, 0.12);
        assert_opacity_eq(OPACITY_MUTED, 0.30);
        assert_opacity_eq(OPACITY_MESSAGE_BORDER, 0.45);
        assert_opacity_eq(OPACITY_STRONG, 0.70);
        assert_opacity_eq(OPACITY_NEAR_FULL, 0.85);
    }
}
