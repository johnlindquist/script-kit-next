use std::time::Duration;

use crate::theme::Theme;
use crate::transitions;

/// Constants for alias input styling
pub(super) const MODAL_WIDTH: f32 = 420.0;
pub(super) const MODAL_PADDING: f32 = 24.0;
pub(super) const INPUT_PADDING: f32 = 12.0;
pub(super) const BUTTON_GAP: f32 = 12.0;
pub(super) const ALIAS_MAX_LENGTH: usize = 32;
pub(super) const ALIAS_INPUT_PLACEHOLDER: &str =
    "Type a short alias, e.g. ch for Clipboard History";
pub(super) const ALIAS_VALID_HELP_TEXT: &str = "Alias runs with <alias> + space in the main menu";
pub(super) const OVERLAY_ANIMATION_DURATION_MS: u64 = 140;
pub(super) const OVERLAY_MODAL_ENTRY_OFFSET_PX: f32 = 12.0;
pub(super) const OVERLAY_MODAL_START_OPACITY: f32 = 0.82;

#[derive(Clone, Copy, Debug)]
pub(super) struct OverlayAppearStyle {
    pub(super) backdrop_opacity: f32,
    pub(super) modal_opacity: f32,
    pub(super) modal_offset_y: f32,
    pub(super) complete: bool,
}

pub(super) fn compute_overlay_appear_style(elapsed: Duration) -> OverlayAppearStyle {
    let progress =
        (elapsed.as_secs_f32() / (OVERLAY_ANIMATION_DURATION_MS as f32 / 1000.0)).clamp(0.0, 1.0);
    let eased = transitions::ease_out_quad(progress);
    let modal_opacity = OVERLAY_MODAL_START_OPACITY + ((1.0 - OVERLAY_MODAL_START_OPACITY) * eased);

    OverlayAppearStyle {
        backdrop_opacity: eased,
        modal_opacity,
        modal_offset_y: OVERLAY_MODAL_ENTRY_OFFSET_PX * (1.0 - eased),
        complete: progress >= 1.0,
    }
}

/// Pre-computed colors for AliasInput rendering
#[derive(Clone, Copy, Debug)]
pub struct AliasInputColors {
    /// Background color for the modal overlay
    pub overlay_bg: u32,
    /// Background color for the modal itself
    pub modal_bg: u32,
    /// Border color for the modal
    pub border: u32,
    /// Primary text color
    pub text_primary: u32,
    /// Secondary text color (for descriptions)
    pub text_secondary: u32,
    /// Muted text color (for placeholders)
    pub text_muted: u32,
    /// Accent color for highlights
    pub accent: u32,
    /// Input field background
    pub input_bg: u32,
    /// Input field border
    pub input_border: u32,
    /// Selection highlight color
    pub selection_bg: u32,
    /// Error text color for validation feedback
    pub text_error: u32,
}

impl AliasInputColors {
    /// Create colors from theme reference
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            overlay_bg: theme.colors.background.main,
            modal_bg: theme.colors.background.main,
            border: theme.colors.ui.border,
            text_primary: theme.colors.text.primary,
            text_secondary: theme.colors.text.secondary,
            text_muted: theme.colors.text.muted,
            accent: theme.colors.accent.selected,
            input_bg: theme.colors.background.search_box,
            input_border: theme.colors.ui.border,
            selection_bg: theme.colors.accent.selected,
            text_error: theme.colors.ui.error,
        }
    }
}

impl Default for AliasInputColors {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

/// Validation error for alias input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliasValidationError {
    Empty,
    ContainsWhitespace,
    InvalidCharacters,
    TooLong { max_length: usize },
}

/// Validate and normalize alias input from the modal.
pub(super) fn validate_alias_input(input: &str) -> Result<String, AliasValidationError> {
    let alias = input.trim();
    if alias.is_empty() {
        return Err(AliasValidationError::Empty);
    }

    if alias.chars().any(char::is_whitespace) {
        return Err(AliasValidationError::ContainsWhitespace);
    }

    if !alias
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AliasValidationError::InvalidCharacters);
    }

    if alias.chars().count() > ALIAS_MAX_LENGTH {
        return Err(AliasValidationError::TooLong {
            max_length: ALIAS_MAX_LENGTH,
        });
    }

    Ok(alias.to_string())
}

pub(super) fn is_command_modifier(platform: bool, control: bool) -> bool {
    platform || control
}

pub(super) fn is_clear_alias_shortcut(
    key: &str,
    command_modifier: bool,
    has_current_alias: bool,
) -> bool {
    has_current_alias
        && command_modifier
        && (key.eq_ignore_ascii_case("backspace") || key.eq_ignore_ascii_case("delete"))
}

/// Actions that can be triggered by the alias input
#[derive(Clone, Debug, PartialEq)]
pub enum AliasInputAction {
    /// User wants to save the alias
    Save(String),
    /// User wants to cancel
    Cancel,
    /// User wants to clear the current alias
    Clear,
}
