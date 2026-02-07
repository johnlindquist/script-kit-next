use gpui::{App, ClickEvent, SharedString, Window};
use std::rc::Rc;

use crate::error::ErrorSeverity;

/// Toast variant determines the visual style and icon
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastVariant {
    /// Success toast (green) - checkmark icon
    Success,
    /// Warning toast (yellow/amber) - warning icon
    Warning,
    /// Error toast (red) - X icon
    Error,
    /// Info toast (blue) - info icon
    #[default]
    Info,
}

impl ToastVariant {
    /// Get the icon character for this variant
    pub fn icon(&self) -> &'static str {
        match self {
            ToastVariant::Success => "✓",
            ToastVariant::Warning => "⚠",
            ToastVariant::Error => "✕",
            ToastVariant::Info => "ℹ",
        }
    }

    /// Convert from ErrorSeverity to ToastVariant
    pub fn from_severity(severity: ErrorSeverity) -> Self {
        match severity {
            ErrorSeverity::Info => ToastVariant::Info,
            ErrorSeverity::Warning => ToastVariant::Warning,
            ErrorSeverity::Error => ToastVariant::Error,
            ErrorSeverity::Critical => ToastVariant::Error,
        }
    }
}

/// Pre-computed colors for Toast rendering
///
/// This struct holds the primitive color values needed for toast rendering,
/// allowing efficient use in closures without cloning the full theme.
#[derive(Clone, Copy, Debug)]
pub struct ToastColors {
    /// Background color of the toast
    pub background: u32,
    /// Text color for the message
    pub text: u32,
    /// Icon color (matches variant)
    pub icon: u32,
    /// Border color
    pub border: u32,
    /// Action button text color
    pub action_text: u32,
    /// Action button background color
    pub action_background: u32,
    /// Dismiss button color
    pub dismiss: u32,
    /// Details section background (theme-aware: black for dark, white for light at low opacity)
    /// Format: 0xRRGGBBAA
    pub details_bg: u32,
}

impl ToastColors {
    pub(super) fn overlay_with_alpha(base_color: u32, alpha: u8) -> u32 {
        ((base_color & 0x00ff_ffff) << 8) | (alpha as u32)
    }

    /// Create ToastColors from theme reference for a specific variant
    pub fn from_theme(theme: &crate::theme::Theme, variant: ToastVariant) -> Self {
        let colors = &theme.colors;

        let (icon_color, border_color) = match variant {
            ToastVariant::Success => (colors.ui.success, colors.ui.success),
            ToastVariant::Warning => (colors.ui.warning, colors.ui.warning),
            ToastVariant::Error => (colors.ui.error, colors.ui.error),
            ToastVariant::Info => (colors.ui.info, colors.ui.info),
        };

        Self {
            background: colors.background.main,
            text: colors.text.primary,
            icon: icon_color,
            border: border_color,
            action_text: colors.accent.selected,
            action_background: colors.accent.selected_subtle,
            dismiss: colors.text.muted,
            details_bg: Self::overlay_with_alpha(colors.accent.selected_subtle, 0x20),
        }
    }

    /// Create ToastColors from design colors for design system support
    pub fn from_design(
        design_colors: &crate::designs::DesignColors,
        variant: ToastVariant,
    ) -> Self {
        let (icon_color, border_color) = match variant {
            ToastVariant::Success => (design_colors.success, design_colors.success),
            ToastVariant::Warning => (design_colors.warning, design_colors.warning),
            ToastVariant::Error => (design_colors.error, design_colors.error),
            ToastVariant::Info => (design_colors.accent, design_colors.accent),
        };

        Self {
            background: design_colors.background,
            text: design_colors.text_primary,
            icon: icon_color,
            border: border_color,
            action_text: design_colors.accent,
            action_background: design_colors.background_selected,
            dismiss: design_colors.text_muted,
            details_bg: Self::overlay_with_alpha(design_colors.background_selected, 0x20),
        }
    }

    /// Create variant-specific colors with custom background opacity
    pub fn with_opacity(mut self, opacity: u8) -> Self {
        // Shift background to include alpha channel
        self.background = (self.background << 8) | (opacity as u32);
        self
    }
}

impl Default for ToastColors {
    fn default() -> Self {
        Self::from_theme(&crate::theme::Theme::default(), ToastVariant::Info)
    }
}

/// Callback type for toast action button clicks
pub type ToastActionCallback = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// An action button that can be displayed on a toast
pub struct ToastAction {
    /// Label text for the action button
    pub label: SharedString,
    /// Callback when the action is clicked
    pub callback: Rc<ToastActionCallback>,
}

impl ToastAction {
    /// Create a new toast action
    pub fn new(label: impl Into<SharedString>, callback: ToastActionCallback) -> Self {
        Self {
            label: label.into(),
            callback: Rc::new(callback),
        }
    }
}

/// Callback type for toast dismiss events
pub type ToastDismissCallback = Box<dyn Fn(&mut Window, &mut App) + 'static>;
