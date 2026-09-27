use gpui::Hsla;

#[derive(Clone, Copy, Debug)]
pub(crate) struct InlineDropdownColors {
    pub(crate) foreground: Hsla,
    pub(crate) muted_foreground: Hsla,
    pub(crate) accent: Hsla,
}

impl InlineDropdownColors {
    #[cfg(test)]
    pub(crate) fn from_theme(theme: &crate::theme::Theme) -> Self {
        Self {
            foreground: gpui::rgb(theme.colors.text.primary).into(),
            muted_foreground: gpui::rgb(theme.colors.text.muted).into(),
            accent: gpui::rgb(theme.colors.accent.selected).into(),
        }
    }
}
