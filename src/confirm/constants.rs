//! Confirm dialog constants
//!
//! Dimensions and styling constants for the confirm modal window.

/// Width of the confirm dialog
pub const CONFIRM_WIDTH: f32 = 340.0;

/// Height of the confirm dialog (message + buttons + padding)
pub const CONFIRM_HEIGHT: f32 = 140.0;

/// Padding around the dialog content
pub const CONFIRM_PADDING: f32 = 20.0;

/// Height of the button row
pub const BUTTON_ROW_HEIGHT: f32 = 44.0;

/// Gap between buttons
pub const BUTTON_GAP: f32 = 12.0;

/// Button corner radius
pub const BUTTON_RADIUS: f32 = 8.0;

/// Button padding (horizontal)
pub const BUTTON_PADDING_X: f32 = 16.0;

/// Button padding (vertical)
pub const BUTTON_PADDING_Y: f32 = 10.0;

/// Dialog corner radius
pub const DIALOG_RADIUS: f32 = 12.0;

/// Margin from main window edges when positioning
pub const CONFIRM_MARGIN: f32 = 16.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_dimensions() {
        // Ensure dialog is reasonably sized
        assert!(CONFIRM_WIDTH > 200.0);
        assert!(CONFIRM_WIDTH < 500.0);
        assert!(CONFIRM_HEIGHT > 100.0);
        assert!(CONFIRM_HEIGHT < 300.0);
    }

    #[test]
    fn test_button_dimensions() {
        // Buttons should have reasonable touch targets
        assert!(BUTTON_ROW_HEIGHT >= 44.0); // iOS minimum
        assert!(BUTTON_PADDING_X > 0.0);
        assert!(BUTTON_PADDING_Y > 0.0);
    }
}
