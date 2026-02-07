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
        let cursor_color = theme
            .get_cursor_style(true)
            .map(|cursor| cursor.color)
            .unwrap_or(theme.colors.accent.selected);
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

impl Default for FormFieldColors {
    fn default() -> Self {
        Self::from_theme(&crate::theme::Theme::default())
    }
}
