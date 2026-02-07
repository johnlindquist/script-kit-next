// ============================================================================
// Key Normalization - Allocation-free key matching
// ============================================================================
//
// IMPORTANT: These helpers use eq_ignore_ascii_case() instead of to_lowercase()
// to avoid allocations on every keystroke. This is a hot path optimization.

/// Check if key is an up arrow (handles both "up" and "arrowup" formats).
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_up(key: &str) -> bool {
    key.eq_ignore_ascii_case("up") || key.eq_ignore_ascii_case("arrowup")
}
/// Check if key is a down arrow (handles both "down" and "arrowdown" formats).
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_down(key: &str) -> bool {
    key.eq_ignore_ascii_case("down") || key.eq_ignore_ascii_case("arrowdown")
}
/// Check if key is a left arrow (handles both "left" and "arrowleft" formats).
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_left(key: &str) -> bool {
    key.eq_ignore_ascii_case("left") || key.eq_ignore_ascii_case("arrowleft")
}
/// Check if key is a right arrow (handles both "right" and "arrowright" formats).
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_right(key: &str) -> bool {
    key.eq_ignore_ascii_case("right") || key.eq_ignore_ascii_case("arrowright")
}
/// Check if key is Enter/Return.
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_enter(key: &str) -> bool {
    key.eq_ignore_ascii_case("enter") || key.eq_ignore_ascii_case("return")
}
/// Check if key is Escape.
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_escape(key: &str) -> bool {
    key.eq_ignore_ascii_case("escape") || key.eq_ignore_ascii_case("esc")
}
/// Check if key is Backspace.
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_backspace(key: &str) -> bool {
    key.eq_ignore_ascii_case("backspace")
}
/// Check if key is the "k" key (for Cmd+K shortcut).
///
/// Uses allocation-free ASCII case-insensitive comparison.
#[inline]
pub fn is_key_k(key: &str) -> bool {
    key.eq_ignore_ascii_case("k")
}
/// Extract printable character from a KeyDownEvent's key_char field.
///
/// Returns Some(char) if the key_char contains a non-control character,
/// None otherwise (for special keys like arrows, escape, etc.).
#[inline]
pub fn printable_char(key_char: Option<&str>) -> Option<char> {
    key_char
        .and_then(|s| s.chars().next())
        .filter(|ch| !ch.is_control())
}
#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // HexColorExt Tests
    // ========================================================================

    #[test]
    fn test_hex_color_to_rgb() {
        // White should convert correctly
        let white = 0xFFFFFFu32.to_rgb();
        assert!(
            (white.l - 1.0).abs() < 0.01,
            "White should have lightness ~1.0"
        );

        // Black should convert correctly
        let black = 0x000000u32.to_rgb();
        assert!(black.l < 0.01, "Black should have lightness ~0.0");

        // Alpha should be 1.0 (fully opaque)
        assert!(
            (white.a - 1.0).abs() < 0.001,
            "to_rgb should be fully opaque"
        );
        assert!(
            (black.a - 1.0).abs() < 0.001,
            "to_rgb should be fully opaque"
        );
    }

    #[test]
    fn test_hex_color_rgba8() {
        // Test with 50% alpha (0x80 = 128)
        let semi = 0xFFFFFFu32.rgba8(0x80);
        // Alpha should be approximately 128/255 = 0.502
        assert!(
            (semi.a - 0.502).abs() < 0.01,
            "rgba8(0x80) should have ~50% alpha, got {}",
            semi.a
        );

        // Test with 0 alpha
        let transparent = 0xFFFFFFu32.rgba8(0x00);
        assert!(
            transparent.a < 0.01,
            "rgba8(0x00) should be fully transparent"
        );

        // Test with full alpha
        let opaque = 0xFFFFFFu32.rgba8(0xFF);
        assert!(
            (opaque.a - 1.0).abs() < 0.01,
            "rgba8(0xFF) should be fully opaque"
        );
    }

    #[test]
    fn test_hex_color_with_opacity() {
        // 50% opacity
        let half = 0xFFFFFFu32.with_opacity(0.5);
        assert!(
            (half.a - 0.5).abs() < 0.02,
            "with_opacity(0.5) should have ~50% alpha, got {}",
            half.a
        );

        // 0% opacity
        let transparent = 0xFFFFFFu32.with_opacity(0.0);
        assert!(
            transparent.a < 0.01,
            "with_opacity(0.0) should be fully transparent"
        );

        // 100% opacity
        let opaque = 0xFFFFFFu32.with_opacity(1.0);
        assert!(
            (opaque.a - 1.0).abs() < 0.01,
            "with_opacity(1.0) should be fully opaque"
        );
    }

    #[test]
    fn test_hex_color_opacity_clamping() {
        // Opacity > 1.0 should clamp to 1.0
        let over = 0xFFFFFFu32.with_opacity(1.5);
        assert!(
            (over.a - 1.0).abs() < 0.01,
            "with_opacity(1.5) should clamp to 1.0"
        );

        // Opacity < 0.0 should clamp to 0.0
        let under = 0xFFFFFFu32.with_opacity(-0.5);
        assert!(under.a < 0.01, "with_opacity(-0.5) should clamp to 0.0");
    }

    // ========================================================================
    // Key Normalization Tests (Allocation-free helpers)
    // ========================================================================

    #[test]
    fn test_is_key_up() {
        // All valid forms
        assert!(is_key_up("up"));
        assert!(is_key_up("Up"));
        assert!(is_key_up("UP"));
        assert!(is_key_up("arrowup"));
        assert!(is_key_up("ArrowUp"));
        assert!(is_key_up("ARROWUP"));
        // Invalid
        assert!(!is_key_up("down"));
        assert!(!is_key_up("left"));
        assert!(!is_key_up("enter"));
    }

    #[test]
    fn test_is_key_down() {
        assert!(is_key_down("down"));
        assert!(is_key_down("Down"));
        assert!(is_key_down("DOWN"));
        assert!(is_key_down("arrowdown"));
        assert!(is_key_down("ArrowDown"));
        assert!(is_key_down("ARROWDOWN"));
        assert!(!is_key_down("up"));
        assert!(!is_key_down("right"));
    }

    #[test]
    fn test_is_key_left() {
        assert!(is_key_left("left"));
        assert!(is_key_left("Left"));
        assert!(is_key_left("arrowleft"));
        assert!(is_key_left("ArrowLeft"));
        assert!(!is_key_left("right"));
        assert!(!is_key_left("up"));
    }

    #[test]
    fn test_is_key_right() {
        assert!(is_key_right("right"));
        assert!(is_key_right("Right"));
        assert!(is_key_right("arrowright"));
        assert!(is_key_right("ArrowRight"));
        assert!(!is_key_right("left"));
        assert!(!is_key_right("down"));
    }

    #[test]
    fn test_is_key_enter() {
        assert!(is_key_enter("enter"));
        assert!(is_key_enter("Enter"));
        assert!(is_key_enter("ENTER"));
        assert!(is_key_enter("return"));
        assert!(is_key_enter("Return"));
        assert!(!is_key_enter("escape"));
        assert!(!is_key_enter("space"));
    }

    #[test]
    fn test_is_key_escape() {
        assert!(is_key_escape("escape"));
        assert!(is_key_escape("Escape"));
        assert!(is_key_escape("ESCAPE"));
        assert!(is_key_escape("esc"));
        assert!(is_key_escape("Esc"));
        assert!(!is_key_escape("enter"));
    }

    #[test]
    fn test_is_key_backspace() {
        assert!(is_key_backspace("backspace"));
        assert!(is_key_backspace("Backspace"));
        assert!(is_key_backspace("BACKSPACE"));
        assert!(!is_key_backspace("delete"));
        assert!(!is_key_backspace("enter"));
    }

    #[test]
    fn test_is_key_k() {
        assert!(is_key_k("k"));
        assert!(is_key_k("K"));
        assert!(!is_key_k("j"));
        assert!(!is_key_k("enter"));
    }

    #[test]
    fn test_printable_char() {
        // Normal printable chars
        assert_eq!(printable_char(Some("a")), Some('a'));
        assert_eq!(printable_char(Some("A")), Some('A'));
        assert_eq!(printable_char(Some("1")), Some('1'));
        assert_eq!(printable_char(Some("!")), Some('!'));
        assert_eq!(printable_char(Some(" ")), Some(' '));

        // Control characters should return None
        assert_eq!(printable_char(Some("\n")), None);
        assert_eq!(printable_char(Some("\t")), None);
        assert_eq!(printable_char(Some("\x1b")), None); // ESC

        // Empty/None cases
        assert_eq!(printable_char(None), None);
        assert_eq!(printable_char(Some("")), None);
    }

    // ========================================================================
    // Original Tests
    // ========================================================================

    #[test]
    fn test_hex_to_rgba_with_opacity() {
        // Test 30% opacity (0.30 * 255 = 76.5 -> truncates to 76 = 0x4C)
        let result = hex_to_rgba_with_opacity(0x1E1E1E, 0.30);
        assert_eq!(result, 0x1E1E1E4C);

        // Test full opacity
        let result = hex_to_rgba_with_opacity(0xFFFFFF, 1.0);
        assert_eq!(result, 0xFFFFFFFF);

        // Test zero opacity
        let result = hex_to_rgba_with_opacity(0x000000, 0.0);
        assert_eq!(result, 0x00000000);

        // Test 50% opacity (0.5 * 255 = 127.5 -> truncates to 127 = 0x7F)
        let result = hex_to_rgba_with_opacity(0xABCDEF, 0.5);
        assert_eq!(result, 0xABCDEF7F);
    }

    #[test]
    fn test_opacity_clamping() {
        // Test opacity > 1.0 gets clamped
        let result = hex_to_rgba_with_opacity(0x123456, 1.5);
        assert_eq!(result, 0x123456FF);

        // Test opacity < 0.0 gets clamped
        let result = hex_to_rgba_with_opacity(0x123456, -0.5);
        assert_eq!(result, 0x12345600);
    }

    #[test]
    fn test_vibrancy_background_with_default_theme() {
        let theme = Theme::default();
        // Default theme has vibrancy enabled
        let bg = get_vibrancy_background(&theme);
        // Should return None when vibrancy is enabled
        assert!(bg.is_none());
    }
}
