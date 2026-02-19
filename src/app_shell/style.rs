//! Style caching for the shell
//!
//! Caches computed styles to avoid recomputation every render.
//! Invalidated when theme changes.

use gpui::{px, Hsla, Pixels, Rgba};

use crate::theme::Theme;
use crate::ui_foundation::{hex_to_rgba_with_opacity, HexColorExt};

/// Cached styles for the shell
///
/// Stored in the app root, recomputed only when theme changes.
/// Pass to AppShell::render to avoid per-render style computation.
#[derive(Clone, Debug)]
pub struct ShellStyleCache {
    /// Theme revision for invalidation
    pub theme_revision: u64,

    /// Main frame background with vibrancy opacity (direct RGBA like POC)
    pub frame_bg: Rgba,
    /// Frame shadow (Vec for GPUI compatibility)
    pub shadows: Vec<gpui::BoxShadow>,
    /// Border radius
    pub radius: Pixels,

    /// Header colors
    pub header: HeaderColors,
    /// Footer colors
    pub footer: FooterColors,
    /// Divider colors
    pub divider: DividerColors,
}

impl ShellStyleCache {
    /// Create a new style cache from theme
    pub fn from_theme(theme: &Theme, revision: u64) -> Self {
        let colors = &theme.colors;
        let opacity = theme.get_opacity();

        // Frame background with vibrancy opacity - use direct RGBA like POC
        // Use theme opacity settings as the single source of truth.
        let bg_alpha = opacity
            .vibrancy_background
            .unwrap_or(opacity.main)
            .clamp(0.0, 1.0);
        let frame_bg = gpui::rgba(hex_to_rgba_with_opacity(colors.background.main, bg_alpha));

        // Shadows: DISABLED when vibrancy is enabled
        // Shadows on transparent elements block the vibrancy blur effect,
        // causing a gray fill appearance. The POC doesn't use any shadows.
        let shadows = if theme.is_vibrancy_enabled() {
            vec![] // No shadows for vibrancy - matches POC behavior
        } else {
            let drop_shadow = theme.get_drop_shadow();
            if drop_shadow.enabled {
                vec![gpui::BoxShadow {
                    color: drop_shadow.color.with_opacity(drop_shadow.opacity),
                    offset: gpui::point(px(drop_shadow.offset_x), px(drop_shadow.offset_y)),
                    blur_radius: px(drop_shadow.blur_radius),
                    spread_radius: px(drop_shadow.spread_radius),
                }]
            } else {
                vec![]
            }
        };

        Self {
            theme_revision: revision,
            frame_bg,
            shadows,
            radius: px(12.0),
            header: HeaderColors::from_theme(theme),
            footer: FooterColors::from_theme(theme),
            divider: DividerColors::from_theme(theme),
        }
    }

    /// Check if cache is valid for the given theme revision
    pub fn is_valid(&self, revision: u64) -> bool {
        self.theme_revision == revision
    }

    /// Update the cache if theme has changed
    pub fn update_if_needed(&mut self, theme: &Theme, revision: u64) {
        if !self.is_valid(revision) {
            *self = Self::from_theme(theme, revision);
        }
    }
}

/// Pre-computed header colors (Copy for efficient closure use)
#[derive(Clone, Copy, Debug)]
pub struct HeaderColors {
    pub text_primary: Hsla,
    pub text_muted: Hsla,
    pub text_dimmed: Hsla,
    pub accent: Hsla,
    pub accent_hex: u32,
    pub background: Hsla,
    pub search_box_bg: Hsla,
    pub border: Hsla,
    /// Color for icons/text displayed on accent background (logo icon)
    pub logo_icon: Hsla,
    pub logo_icon_hex: u32,
}

impl HeaderColors {
    pub fn from_theme(theme: &Theme) -> Self {
        let colors = &theme.colors;
        // Use the theme token intended for text/icons on accent backgrounds.
        let logo_icon_hex = colors.text.on_accent;
        Self {
            text_primary: colors.text.primary.to_rgb(),
            text_muted: colors.text.muted.to_rgb(),
            text_dimmed: colors.text.dimmed.to_rgb(),
            accent: colors.accent.selected.to_rgb(),
            accent_hex: colors.accent.selected,
            background: colors.background.main.to_rgb(),
            search_box_bg: colors.background.search_box.to_rgb(),
            border: colors.ui.border.to_rgb(),
            logo_icon: logo_icon_hex.to_rgb(),
            logo_icon_hex,
        }
    }
}

/// Pre-computed footer colors (Copy for efficient closure use)
#[derive(Clone, Copy, Debug)]
pub struct FooterColors {
    pub accent: Hsla,
    pub accent_hex: u32,
    pub text_muted: Hsla,
    pub border: Hsla,
    pub border_hex: u32,
    pub background: Hsla,
    /// Color for icons/text displayed on accent background (logo icon)
    pub logo_icon: Hsla,
    pub logo_icon_hex: u32,
    /// Semi-transparent overlay background derived from theme background.
    /// Used for footer background with vibrancy.
    pub overlay_bg: Hsla,
}

impl FooterColors {
    pub fn from_theme(theme: &Theme) -> Self {
        let colors = &theme.colors;
        let logo_icon_hex = colors.text.on_accent;
        let overlay_bg = colors.background.main.with_opacity(0.5);

        Self {
            accent: colors.accent.selected.to_rgb(),
            accent_hex: colors.accent.selected,
            text_muted: colors.text.muted.to_rgb(),
            border: colors.ui.border.to_rgb(),
            border_hex: colors.ui.border,
            background: colors.background.main.to_rgb(),
            logo_icon: logo_icon_hex.to_rgb(),
            logo_icon_hex,
            overlay_bg,
        }
    }
}

/// Pre-computed divider colors
#[derive(Clone, Copy, Debug)]
pub struct DividerColors {
    pub line: Hsla,
}

impl DividerColors {
    pub fn from_theme(theme: &Theme) -> Self {
        let colors = &theme.colors;
        // 60% opacity for subtle divider
        Self {
            line: colors.ui.border.rgba8(0x99),
        }
    }
}

// Note: hex_to_hsla_with_alpha removed - now using direct RGBA via hex_to_rgba_with_opacity
// This matches the POC approach and avoids potential HSLA conversion issues

#[cfg(test)]
mod tests {
    use super::{FooterColors, HeaderColors, ShellStyleCache};
    use crate::theme::Theme;
    use crate::ui_foundation::HexColorExt;
    use gpui::{point, px};

    #[test]
    fn test_header_colors_logo_icon_uses_text_on_accent_theme_token() {
        let theme = Theme::default();
        let header = HeaderColors::from_theme(&theme);

        assert_eq!(header.logo_icon_hex, theme.colors.text.on_accent);
    }

    #[test]
    fn test_footer_colors_logo_icon_uses_text_on_accent_theme_token() {
        let theme = Theme::default();
        let footer = FooterColors::from_theme(&theme);

        assert_eq!(footer.logo_icon_hex, theme.colors.text.on_accent);
    }

    #[test]
    fn test_footer_colors_overlay_background_uses_theme_background_opacity() {
        let theme = Theme::default();
        let footer = FooterColors::from_theme(&theme);
        let expected = theme.colors.background.main.with_opacity(0.5);

        assert_eq!(footer.overlay_bg.h, expected.h);
        assert_eq!(footer.overlay_bg.s, expected.s);
        assert_eq!(footer.overlay_bg.l, expected.l);
        assert_eq!(footer.overlay_bg.a, expected.a);
    }

    #[test]
    fn test_shell_style_cache_uses_theme_drop_shadow_when_vibrancy_is_disabled() {
        let mut theme = Theme::default();
        let mut vibrancy = theme.get_vibrancy();
        vibrancy.enabled = false;
        theme.vibrancy = Some(vibrancy);

        let mut shadow = theme.get_drop_shadow();
        shadow.enabled = true;
        shadow.color = theme.colors.accent.selected;
        shadow.opacity = 0.62;
        shadow.blur_radius = 11.0;
        shadow.spread_radius = 2.0;
        shadow.offset_x = -3.0;
        shadow.offset_y = 5.0;
        theme.drop_shadow = Some(shadow.clone());

        let styles = ShellStyleCache::from_theme(&theme, 7);

        assert_eq!(styles.shadows.len(), 1);
        let computed = styles
            .shadows
            .first()
            .expect("shell style cache should include one shadow");
        let expected_color = shadow.color.with_opacity(shadow.opacity);

        assert_eq!(computed.color.h, expected_color.h);
        assert_eq!(computed.color.s, expected_color.s);
        assert_eq!(computed.color.l, expected_color.l);
        assert_eq!(computed.color.a, expected_color.a);
        assert_eq!(
            computed.offset,
            point(px(shadow.offset_x), px(shadow.offset_y))
        );
        assert_eq!(computed.blur_radius, px(shadow.blur_radius));
        assert_eq!(computed.spread_radius, px(shadow.spread_radius));
    }

    #[test]
    fn test_shell_style_cache_has_no_shadows_when_drop_shadow_is_disabled() {
        let mut theme = Theme::default();
        let mut vibrancy = theme.get_vibrancy();
        vibrancy.enabled = false;
        theme.vibrancy = Some(vibrancy);

        let mut shadow = theme.get_drop_shadow();
        shadow.enabled = false;
        theme.drop_shadow = Some(shadow);

        let styles = ShellStyleCache::from_theme(&theme, 8);
        assert!(
            styles.shadows.is_empty(),
            "drop shadow should not be rendered when disabled in theme"
        );
    }
}
