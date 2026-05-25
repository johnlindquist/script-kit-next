use crate::theme::{best_readable_text_hex, contrast_ratio, Theme};

pub const INLINE_AGENT_PRIMARY_TEXT_MIN_CONTRAST: f32 = 4.5;
pub const INLINE_AGENT_SECONDARY_TEXT_MIN_CONTRAST: f32 = 4.5;
pub const INLINE_AGENT_DISABLED_TEXT_MIN_CONTRAST: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InlineAgentColors {
    pub surface: u32,
    pub border: u32,
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_disabled: u32,
    pub accent: u32,
    pub accent_text: u32,
    pub thinking: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InlineAgentContrastSummary {
    pub primary_text: f32,
    pub secondary_text: f32,
    pub disabled_text: f32,
    pub thinking: f32,
    pub accent_text: f32,
}

impl InlineAgentColors {
    pub fn from_theme(theme: &Theme) -> Self {
        let surface = theme.colors.background.search_box;
        let accent = theme.colors.accent.selected;
        let text_primary = readable_text_or_fallback(
            theme.colors.text.primary,
            surface,
            INLINE_AGENT_PRIMARY_TEXT_MIN_CONTRAST,
        );
        let text_secondary = readable_text_or_fallback(
            theme.colors.text.secondary,
            surface,
            INLINE_AGENT_SECONDARY_TEXT_MIN_CONTRAST,
        );
        let text_disabled = readable_text_or_fallback(
            theme.colors.text.muted,
            surface,
            INLINE_AGENT_DISABLED_TEXT_MIN_CONTRAST,
        );
        let accent_text = readable_text_or_fallback(
            theme.colors.text.on_accent,
            accent,
            INLINE_AGENT_PRIMARY_TEXT_MIN_CONTRAST,
        );
        let thinking = if contrast_ratio(accent, surface) >= INLINE_AGENT_PRIMARY_TEXT_MIN_CONTRAST
        {
            accent
        } else {
            text_primary
        };

        Self {
            surface,
            border: theme.colors.ui.border,
            text_primary,
            text_secondary,
            text_disabled,
            accent,
            accent_text,
            thinking,
        }
    }

    pub fn contrast_summary(&self) -> InlineAgentContrastSummary {
        InlineAgentContrastSummary {
            primary_text: contrast_ratio(self.text_primary, self.surface),
            secondary_text: contrast_ratio(self.text_secondary, self.surface),
            disabled_text: contrast_ratio(self.text_disabled, self.surface),
            thinking: contrast_ratio(self.thinking, self.surface),
            accent_text: contrast_ratio(self.accent_text, self.accent),
        }
    }
}

fn readable_text_or_fallback(candidate: u32, background: u32, min_ratio: f32) -> u32 {
    if contrast_ratio(candidate, background) >= min_ratio {
        candidate
    } else {
        best_readable_text_hex(background)
    }
}
