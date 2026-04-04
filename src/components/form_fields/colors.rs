/// Pre-computed colors for form field rendering
///
/// This struct holds the color values needed for form field rendering,
/// allowing efficient use in closures without cloning the full theme.
#[derive(Clone, Copy, Debug)]
pub struct FormFieldColors {
    /// Background color of the input
    pub background: u32,
    /// Background color when focused
    pub background_focused: u32,
    /// Text color when typing
    pub text: u32,
    /// Placeholder text color
    pub placeholder: u32,
    /// Label text color
    pub label: u32,
    /// Border color
    pub border: u32,
    /// Border color when focused
    pub border_focused: u32,
    /// Cursor color
    pub cursor: u32,
    /// Checkbox checked background
    pub checkbox_checked: u32,
    /// Checkbox check mark color
    pub checkbox_mark: u32,
    /// Shared input font size token for all editable field text
    pub input_font_size: f32,
    /// Shared label font size token for labels, hints, and inline indicators
    pub label_font_size: f32,
}

impl FormFieldColors {
    /// Create FormFieldColors from a Theme
    pub fn from_theme(theme: &crate::theme::Theme) -> Self {
        let ui_font_size = theme.get_fonts().ui_size;
        let cursor_color = theme.colors.accent.selected;
        Self {
            background: theme.colors.background.search_box,
            background_focused: theme.colors.background.main,
            text: theme.colors.text.primary,
            placeholder: theme.colors.text.muted,
            label: theme.colors.text.secondary,
            border: theme.colors.ui.border,
            border_focused: theme.colors.accent.selected,
            cursor: cursor_color,
            checkbox_checked: theme.colors.accent.selected,
            checkbox_mark: theme.colors.background.main,
            input_font_size: (ui_font_size + 2.0).max(12.0),
            label_font_size: (ui_font_size - 2.0).max(10.0),
        }
    }

    /// Create FormFieldColors from design colors
    pub fn from_design(colors: &crate::designs::DesignColors) -> Self {
        let typography = crate::designs::DesignTypography::default();
        Self {
            background: colors.background_secondary,
            background_focused: colors.background,
            text: colors.text_primary,
            placeholder: colors.text_muted,
            label: colors.text_secondary,
            border: colors.border,
            border_focused: colors.accent,
            cursor: colors.accent,
            checkbox_checked: colors.accent,
            checkbox_mark: colors.background,
            input_font_size: typography.font_size_lg,
            label_font_size: typography.font_size_sm,
        }
    }
}

/// Pre-computed whisper-chrome surface for form fields
#[derive(Clone, Copy, Debug)]
pub struct FormFieldSurface {
    /// Background color with low alpha
    pub background: gpui::Rgba,
    /// Border color — accent on focus, ghost otherwise
    pub border: gpui::Rgba,
}

impl FormFieldColors {
    /// Compute a whisper-chrome surface for a form field.
    /// Focused fields get slightly higher alpha; unfocused fields rest at ghost opacity.
    pub fn whisper_surface(&self, focused: bool) -> FormFieldSurface {
        let background_alpha: u32 = if focused { 0x1A } else { 0x10 };
        let border_alpha: u32 = if focused { 0x66 } else { 0x22 };
        FormFieldSurface {
            background: gpui::rgba((self.background << 8) | background_alpha),
            border: if focused {
                gpui::rgb(self.border_focused)
            } else {
                gpui::rgba((self.border << 8) | border_alpha)
            },
        }
    }
}

impl Default for FormFieldColors {
    fn default() -> Self {
        Self::from_theme(&crate::theme::Theme::default())
    }
}
