//! Actions dialog constants
//!
//! Overlay popup dimensions and styling constants used by the ActionsDialog.

/// Popup width for the actions dialog
pub const POPUP_WIDTH: f32 = 320.0;

/// Maximum height for the actions dialog popup
pub const POPUP_MAX_HEIGHT: f32 = 400.0;

/// Fixed height for action items (required for uniform_list virtualization)
/// Standardized to 44px for consistent touch targets (matches iOS guidelines, Notes panel)
pub const ACTION_ITEM_HEIGHT: f32 = 44.0;

/// Fixed height for the search input row (matches Notes panel PANEL_SEARCH_HEIGHT)
pub const SEARCH_INPUT_HEIGHT: f32 = 44.0;

/// Width of the left accent bar for selected items (legacy, kept for reference)
pub const ACCENT_BAR_WIDTH: f32 = 3.0;

/// Height for the header row showing context title
pub const HEADER_HEIGHT: f32 = 44.0;

/// Horizontal inset for action rows (creates rounded pill appearance)
pub const ACTION_ROW_INSET: f32 = 6.0;

/// Corner radius for selected row background (pill style)
pub const SELECTION_RADIUS: f32 = 8.0;

/// Minimum width for keycap badges
pub const KEYCAP_MIN_WIDTH: f32 = 22.0;

/// Height for keycap badges
pub const KEYCAP_HEIGHT: f32 = 22.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popup_constants() {
        assert_eq!(POPUP_WIDTH, 320.0);
        assert_eq!(POPUP_MAX_HEIGHT, 400.0);
    }

    #[test]
    fn test_action_item_height_constant() {
        // Fixed height is required for uniform_list virtualization
        // Standardized to 44px for consistent touch targets (matches iOS guidelines)
        assert_eq!(ACTION_ITEM_HEIGHT, 44.0);
        // Ensure item height is positive and reasonable
        const _: () = assert!(ACTION_ITEM_HEIGHT > 0.0);
        const _: () = assert!(ACTION_ITEM_HEIGHT < POPUP_MAX_HEIGHT);
    }

    #[test]
    fn test_max_visible_items() {
        // Calculate max visible items that can fit in the popup
        // This helps verify scroll virtualization is worthwhile
        let max_visible = (POPUP_MAX_HEIGHT / ACTION_ITEM_HEIGHT) as usize;
        // With 400px max height and 44px items, ~9 items fit
        assert!(max_visible >= 8, "Should fit at least 8 items");
        assert!(max_visible <= 15, "Sanity check on max visible");
    }
}
