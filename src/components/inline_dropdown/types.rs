use gpui::{AnyElement, Hsla, SharedString};

#[derive(Clone, Copy, Debug)]
pub(crate) struct InlineDropdownColors {
    pub(crate) surface_rgba: u32,
    pub(crate) border_rgba: u32,
    pub(crate) divider_rgba: u32,
    pub(crate) foreground: Hsla,
    pub(crate) muted_foreground: Hsla,
}

impl InlineDropdownColors {
    pub(crate) fn from_theme(theme: &crate::theme::Theme) -> Self {
        let chrome = crate::theme::AppChromeColors::from_theme(theme);
        Self {
            surface_rgba: chrome.inline_dropdown_surface_rgba,
            border_rgba: chrome.border_rgba,
            divider_rgba: chrome.divider_rgba,
            foreground: gpui::rgb(theme.colors.text.primary).into(),
            muted_foreground: gpui::rgb(theme.colors.text.muted).into(),
        }
    }
}

pub(crate) struct InlineDropdownEmptyState {
    pub(crate) message: SharedString,
    pub(crate) hints: Vec<AnyElement>,
}

pub(crate) struct InlineDropdownSynopsis {
    pub(crate) label: SharedString,
    pub(crate) meta: SharedString,
    pub(crate) description: SharedString,
}
