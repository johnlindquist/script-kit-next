// ============================================================================
// Header Layout Constants (Reference: Main Menu)
// ============================================================================
// These constants define the canonical header layout used by the main script list.
// All prompts (ArgPrompt, EnvPrompt, etc.) should use these exact values to ensure
// visual consistency with the main menu search input.

/// Header horizontal padding (px) - matches main menu
pub const HEADER_PADDING_X: f32 = 16.0;

/// Header vertical padding (px) - matches main menu
/// NOTE: This is 8px, NOT 12px (design_spacing.padding_md). The main menu uses
/// a tighter vertical padding for a more compact header appearance.
pub const HEADER_PADDING_Y: f32 = 8.0;

/// Header gap between input and buttons (px) - matches main menu
pub const HEADER_GAP: f32 = 12.0;

/// Button height in header (px)
pub const HEADER_BUTTON_HEIGHT: f32 = 28.0;

/// Divider height below header (px)
pub const HEADER_DIVIDER_HEIGHT: f32 = 1.0;

/// Total header height including padding and divider (45px)
/// Calculated as: HEADER_PADDING_Y * 2 + HEADER_BUTTON_HEIGHT + HEADER_DIVIDER_HEIGHT
/// This is the y-offset where content begins below the header.
pub const HEADER_TOTAL_HEIGHT: f32 =
    HEADER_PADDING_Y * 2.0 + HEADER_BUTTON_HEIGHT + HEADER_DIVIDER_HEIGHT;

/// Canonical single-line prompt input/container height used by prompt UIs.
pub const PROMPT_INPUT_FIELD_HEIGHT: f32 = 44.0;

/// Minimum inset from visible display edges when clamping window geometry.
/// Keeps the window fully visible after dynamic height changes.
pub const WINDOW_VISIBLE_EDGE_MARGIN: f64 = 4.0;

/// Shared status prefix for prompts shown while a script is actively running.
pub const SCRIPT_RUNNING_STATUS_PREFIX: &str = "Script running";

/// Build a concise running status message used in prompt headers/footers.
pub fn running_status_message(context: &str) -> String {
    format!("{SCRIPT_RUNNING_STATUS_PREFIX} · {context}")
}

// ============================================================================
// Input Placeholder Configuration
// ============================================================================

/// Default placeholder text for the main search input
pub const DEFAULT_PLACEHOLDER: &str = "Script Kit";

// ============================================================================
// Cursor Styling Constants
// ============================================================================

/// Standard cursor width in pixels for text input fields
///
/// This matches the standard cursor width used in editor.rs and provides
/// visual consistency across all input fields.
pub const CURSOR_WIDTH: f32 = 2.0;

/// Horizontal gap between the cursor and adjacent text/placeholder, in pixels.
///
/// Keep this identical in empty and non-empty states to avoid horizontal shifting
/// when switching between placeholder and typed text.
pub const CURSOR_GAP_X: f32 = 2.0;

/// Cursor height for large text (.text_lg() / 18px font)
///
/// This value is calculated to align properly with GPUI's .text_lg() text rendering:
/// - GPUI's text_lg() uses ~18px font size
/// - With natural line height (~1.55), this gives ~28px line height
/// - Cursor should be 18px with 5px top/bottom spacing for vertical centering
///
/// NOTE: This value differs from `font_size_lg * line_height_normal` in design tokens
/// because GPUI's .text_lg() has different line-height than our token calculations.
/// Using this constant ensures proper cursor-text alignment.
pub const CURSOR_HEIGHT_LG: f32 = 18.0;

/// Vertical margin for cursor centering within text line
///
/// Apply this as `.my(px(CURSOR_MARGIN_Y))` to vertically center the cursor
/// within its text line. This follows the editor.rs pattern.
pub const CURSOR_MARGIN_Y: f32 = 2.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_placeholder_text() {
        assert_eq!(DEFAULT_PLACEHOLDER, "Script Kit");
    }

    #[test]
    fn test_cursor_width_constant() {
        assert_eq!(CURSOR_WIDTH, 2.0);
    }

    #[test]
    fn test_cursor_height_lg_matches_text_lg() {
        // CURSOR_HEIGHT_LG should be 18px to match GPUI's .text_lg() font size
        // This ensures proper vertical alignment of cursor with text
        assert_eq!(CURSOR_HEIGHT_LG, 18.0);
    }

    #[test]
    fn test_cursor_margin_constant() {
        // Margin should be 2px for proper vertical centering
        assert_eq!(CURSOR_MARGIN_Y, 2.0);
    }

    #[test]
    fn running_status_message_uses_shared_prefix() {
        assert_eq!(
            running_status_message("awaiting input"),
            "Script running · awaiting input"
        );
    }

    #[test]
    fn window_visible_edge_margin_is_positive() {
        let margin = std::hint::black_box(WINDOW_VISIBLE_EDGE_MARGIN);
        assert!(margin > 0.0);
    }
}
