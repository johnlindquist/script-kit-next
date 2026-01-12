//! AI Command Bar constants
//!
//! Layout dimensions following the patterns in DESIGNING_POPUP_WINDOWS.md.
//! These match the main actions dialog for consistency.

/// Popup width (matches main actions dialog)
pub const POPUP_WIDTH: f32 = 320.0;

/// Maximum height before scrolling
pub const POPUP_MAX_HEIGHT: f32 = 400.0;

/// Fixed height for action items (required for uniform_list virtualization)
/// 44px matches iOS touch target guidelines
pub const ACTION_ITEM_HEIGHT: f32 = 44.0;

/// Height for the search input container
pub const SEARCH_INPUT_HEIGHT: f32 = 44.0;

/// Inner search input width
pub const SEARCH_INPUT_INNER_WIDTH: f32 = 240.0;

/// Inner search input height
pub const SEARCH_INPUT_INNER_HEIGHT: f32 = 28.0;

/// Height for section headers
pub const SECTION_HEADER_HEIGHT: f32 = 24.0;

/// Horizontal inset for action rows (creates rounded pill appearance)
pub const ACTION_ROW_INSET: f32 = 6.0;

/// Corner radius for selection highlight (pill style)
pub const SELECTION_RADIUS: f32 = 8.0;

/// Popup container corner radius
pub const POPUP_RADIUS: f32 = 12.0;

/// Minimum width for keycap badges
pub const KEYCAP_MIN_WIDTH: f32 = 22.0;

/// Height for keycap badges
pub const KEYCAP_HEIGHT: f32 = 22.0;

/// Horizontal padding inside keycaps
pub const KEYCAP_PADDING_X: f32 = 6.0;

/// Border radius for keycaps
pub const KEYCAP_RADIUS: f32 = 5.0;

/// Gap between keycaps
pub const KEYCAP_GAP: f32 = 3.0;

/// Content padding (horizontal)
pub const CONTENT_PADDING_X: f32 = 16.0;

/// Footer height
pub const FOOTER_HEIGHT: f32 = 32.0;

// === Alpha Values (for vibrancy-compatible colors) ===

/// Empty input background alpha (12.5%)
pub const ALPHA_INPUT_EMPTY: u8 = 0x20;

/// Active input background alpha (25%)
pub const ALPHA_INPUT_ACTIVE: u8 = 0x40;

/// Keycap background alpha (50%)
pub const ALPHA_KEYCAP_BG: u8 = 0x80;

/// Keycap border alpha (62.5%)
pub const ALPHA_KEYCAP_BORDER: u8 = 0xA0;

/// Selection highlight alpha (33%)
pub const ALPHA_SELECTED: u8 = 0x54;

/// Hover highlight alpha (15%)
pub const ALPHA_HOVER: u8 = 0x26;

/// Section separator alpha (25%)
pub const ALPHA_SEPARATOR: u8 = 0x40;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popup_dimensions() {
        assert_eq!(POPUP_WIDTH, 320.0);
        assert_eq!(POPUP_MAX_HEIGHT, 400.0);
    }

    #[test]
    fn test_item_height_fits_in_popup() {
        let max_visible = (POPUP_MAX_HEIGHT / ACTION_ITEM_HEIGHT) as usize;
        assert!(max_visible >= 8, "Should fit at least 8 items");
    }
}
