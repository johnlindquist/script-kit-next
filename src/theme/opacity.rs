//! Canonical semantic opacity tokens used across the app.
//!
//! These constants intentionally define opacity in the standard direction:
//! `0.0` is fully transparent and `1.0` is fully opaque.
//! Keep all new UI opacity values sourced from this module.

/// Fully transparent — used for hover-reveal patterns where an element
/// starts invisible and fades in on group hover.
pub const OPACITY_HIDDEN: f32 = 0.0;
/// Assistant bubble tint background.
pub const OPACITY_MESSAGE_ASSISTANT_BACKGROUND: f32 = 0.10;
/// User bubble tint background (kept for reference, now using inline 0.18 with muted tone).
pub const OPACITY_MESSAGE_USER_BACKGROUND: f32 = 0.18;
/// Very subtle tint/background accents.
pub const OPACITY_SUBTLE: f32 = 0.15;
/// Background opacity for suggestion/prompt cards (subtle neutral tint).
pub const OPACITY_CARD_BG: f32 = 0.12;
/// Danger-tinted background surfaces (error banners, delete confirmation).
pub const OPACITY_DANGER_BG: f32 = 0.20;
/// Danger hover state (slightly stronger than OPACITY_DANGER_BG).
pub const OPACITY_DANGER_HOVER: f32 = 0.25;
/// Generic hover surfaces.
pub const OPACITY_HOVER: f32 = 0.30;
/// Suggestion card hover background.
pub const OPACITY_SUGGESTION_HOVER: f32 = 0.35;
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
/// Preview/secondary text in sidebars.
pub const OPACITY_PREVIEW_TEXT: f32 = 0.75;
/// Prominent emphasis (button borders, key UI chrome).
pub const OPACITY_PROMINENT: f32 = 0.80;
/// Near-full emphasis.
pub const OPACITY_NEAR_FULL: f32 = 0.85;
/// Active/pressed state (button hover, focused chrome).
pub const OPACITY_ACTIVE: f32 = 0.90;

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
        assert_opacity_eq(OPACITY_HIDDEN, 0.0);
        assert_opacity_eq(OPACITY_MESSAGE_ASSISTANT_BACKGROUND, 0.10);
        assert_opacity_eq(OPACITY_MESSAGE_USER_BACKGROUND, 0.18);
        assert_opacity_eq(OPACITY_MUTED, 0.30);
        assert_opacity_eq(OPACITY_STRONG, 0.70);
        assert_opacity_eq(OPACITY_NEAR_FULL, 0.85);
        assert_opacity_eq(OPACITY_CARD_BG, 0.12);
        assert_opacity_eq(OPACITY_DANGER_BG, 0.20);
        assert_opacity_eq(OPACITY_DANGER_HOVER, 0.25);
        assert_opacity_eq(OPACITY_SUGGESTION_HOVER, 0.35);
        assert_opacity_eq(OPACITY_PREVIEW_TEXT, 0.75);
        assert_opacity_eq(OPACITY_PROMINENT, 0.80);
        assert_opacity_eq(OPACITY_ACTIVE, 0.90);
    }
}
