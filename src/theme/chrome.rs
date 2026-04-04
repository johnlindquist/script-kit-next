use crate::ui_foundation::hex_to_rgba_with_opacity;

use super::Theme;

/// Shared chrome contract for app surfaces, badges, selection, and hover.
///
/// All color/opacity decisions route through `Theme` — view code consumes
/// resolved RGBA values instead of computing them locally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AppChromeColors {
    pub text_primary_hex: u32,
    pub text_secondary_hex: u32,
    pub text_muted_hex: u32,
    pub text_dimmed_hex: u32,
    pub accent_hex: u32,

    pub window_surface_rgba: u32,
    pub surface_rgba: u32,
    pub input_surface_rgba: u32,
    pub divider_rgba: u32,
    pub border_rgba: u32,

    pub selection_rgba: u32,
    pub hover_rgba: u32,

    pub badge_bg_rgba: u32,
    pub badge_border_rgba: u32,
    pub badge_text_hex: u32,

    pub accent_badge_bg_rgba: u32,
    pub accent_badge_border_rgba: u32,
    pub accent_badge_text_hex: u32,
}

/// Contrast-safe colors for semantic status chips (OK, Err, Warn, Info).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SemanticChipColors {
    pub bg_rgba: u32,
    pub border_rgba: u32,
    pub text_hex: u32,
}

impl AppChromeColors {
    /// Resolve contrast-safe chip colors for a given semantic base color.
    #[allow(dead_code)] // used by binary target (theme_chooser.rs)
    pub(crate) fn semantic_chip_colors(
        &self,
        theme: &Theme,
        base_hex: u32,
    ) -> SemanticChipColors {
        let opacity = theme.get_opacity();
        let bg_alpha = opacity.hover.max(0.18);
        let border_alpha = opacity.selected.max(0.28);
        let text_hex = super::best_readable_text_hex(base_hex);
        tracing::debug!(
            base_hex,
            text_hex,
            bg_alpha,
            border_alpha,
            "theme_semantic_chip_resolved"
        );
        SemanticChipColors {
            bg_rgba: hex_to_rgba_with_opacity(base_hex, bg_alpha),
            border_rgba: hex_to_rgba_with_opacity(base_hex, border_alpha),
            text_hex,
        }
    }

    pub(crate) fn from_theme(theme: &Theme) -> Self {
        let opacity = theme.get_opacity();
        let colors = &theme.colors;

        Self {
            text_primary_hex: colors.text.primary,
            text_secondary_hex: colors.text.secondary,
            text_muted_hex: colors.text.muted,
            text_dimmed_hex: colors.text.dimmed,
            accent_hex: colors.accent.selected,

            window_surface_rgba: hex_to_rgba_with_opacity(colors.background.main, opacity.main),
            surface_rgba: hex_to_rgba_with_opacity(colors.background.title_bar, opacity.title_bar),
            input_surface_rgba: hex_to_rgba_with_opacity(
                colors.background.search_box,
                opacity.search_box,
            ),
            divider_rgba: hex_to_rgba_with_opacity(colors.ui.border, opacity.border_inactive),
            border_rgba: hex_to_rgba_with_opacity(
                colors.ui.border,
                opacity.border_active.max(opacity.border_inactive),
            ),

            selection_rgba: hex_to_rgba_with_opacity(
                colors.accent.selected_subtle,
                opacity.selected,
            ),
            hover_rgba: hex_to_rgba_with_opacity(colors.accent.selected_subtle, opacity.hover),

            badge_bg_rgba: hex_to_rgba_with_opacity(
                colors.background.search_box,
                opacity.input_inactive,
            ),
            badge_border_rgba: hex_to_rgba_with_opacity(colors.ui.border, opacity.border_inactive),
            badge_text_hex: colors.text.secondary,

            accent_badge_bg_rgba: hex_to_rgba_with_opacity(colors.accent.selected, opacity.hover),
            accent_badge_border_rgba: hex_to_rgba_with_opacity(
                colors.accent.selected,
                opacity.selected,
            ),
            accent_badge_text_hex: colors.accent.selected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppChromeColors;
    use crate::theme::Theme;
    use crate::ui_foundation::hex_to_rgba_with_opacity;

    #[test]
    fn light_theme_selection_follows_selected_subtle_and_theme_selected_opacity() {
        let theme = Theme::light_default();
        let chrome = AppChromeColors::from_theme(&theme);
        let opacity = theme.get_opacity();

        assert_eq!(
            chrome.selection_rgba,
            hex_to_rgba_with_opacity(theme.colors.accent.selected_subtle, opacity.selected,)
        );
        assert_eq!(
            chrome.hover_rgba,
            hex_to_rgba_with_opacity(theme.colors.accent.selected_subtle, opacity.hover,)
        );
    }

    #[test]
    fn text_dimmed_and_window_surface_resolve_from_theme() {
        let theme = Theme::light_default();
        let chrome = AppChromeColors::from_theme(&theme);
        assert_eq!(chrome.text_dimmed_hex, theme.colors.text.dimmed);
        assert_eq!(
            chrome.window_surface_rgba,
            hex_to_rgba_with_opacity(theme.colors.background.main, theme.get_opacity().main,)
        );
    }

    #[test]
    fn dark_theme_accent_badges_follow_accent_and_hover_selected_opacity() {
        let theme = Theme::dark_default();
        let chrome = AppChromeColors::from_theme(&theme);
        let opacity = theme.get_opacity();

        assert_eq!(
            chrome.accent_badge_bg_rgba,
            hex_to_rgba_with_opacity(theme.colors.accent.selected, opacity.hover)
        );
        assert_eq!(
            chrome.accent_badge_border_rgba,
            hex_to_rgba_with_opacity(theme.colors.accent.selected, opacity.selected)
        );
    }
}
