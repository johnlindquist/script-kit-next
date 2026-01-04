//! Actions dialog constants
//!
//! Overlay popup dimensions and styling constants used by the ActionsDialog.

/// Popup width for the actions dialog
pub const POPUP_WIDTH: f32 = 320.0;

/// Maximum height for the actions dialog popup
pub const POPUP_MAX_HEIGHT: f32 = 400.0;

/// Fixed height for action items (required for uniform_list virtualization)
/// Increased from 36px to 42px for better touch targets and visual breathing room
pub const ACTION_ITEM_HEIGHT: f32 = 42.0;

/// Width of the left accent bar for selected items
pub const ACCENT_BAR_WIDTH: f32 = 3.0;

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
        // Increased to 42px for better touch targets and visual breathing room
        assert_eq!(ACTION_ITEM_HEIGHT, 42.0);
        // Ensure item height is positive and reasonable
        const _: () = assert!(ACTION_ITEM_HEIGHT > 0.0);
        const _: () = assert!(ACTION_ITEM_HEIGHT < POPUP_MAX_HEIGHT);
    }

    #[test]
    fn test_max_visible_items() {
        // Calculate max visible items that can fit in the popup
        // This helps verify scroll virtualization is worthwhile
        let max_visible = (POPUP_MAX_HEIGHT / ACTION_ITEM_HEIGHT) as usize;
        // With 400px max height and 42px items, ~9 items fit
        assert!(max_visible >= 8, "Should fit at least 8 items");
        assert!(max_visible <= 15, "Sanity check on max visible");
    }
}
