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

        // Frame background with vibrancy opacity - use direct RGBA like POC
        // This avoids HSLA conversion issues and matches the proven POC approach
        // Dark mode: lower opacity (0.37) for better blur effect
        // Light mode: higher opacity (0.85) for visibility like POC's rgba(0xFAFAFAD9)
        let bg_alpha = if theme.has_dark_colors() {
            // Dark mode: use theme opacity or dark default (0.30-0.37)
            theme
                .opacity
                .as_ref()
                .map(|o| o.main)
                .unwrap_or(0.37)
                .clamp(0.30, 0.50)
        } else {
            // Light mode: higher opacity like POC (0.85)
            theme
                .opacity
                .as_ref()
                .map(|o| o.main)
                .unwrap_or(0.85)
                .clamp(0.70, 0.90)
        };
        let frame_bg = gpui::rgba(hex_to_rgba_with_opacity(colors.background.main, bg_alpha));

        // Standard drop shadow
        let shadow = gpui::BoxShadow {
            color: Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.4,
            },
            offset: gpui::point(px(0.0), px(4.0)),
            blur_radius: px(20.0),
            spread_radius: px(0.0),
        };

        let shadows = vec![shadow];

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
        // Logo icon color: For Script Kit, we use black (0x000000) on gold/yellow
        // accent background for brand consistency and maximum contrast.
        // This could be made configurable via theme in the future.
        let logo_icon_hex = 0x000000u32; // Black for contrast on yellow/gold
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
    /// Semi-transparent overlay background (theme-aware: black for dark, white for light)
    /// Used for footer background with vibrancy
    pub overlay_bg: Hsla,
}

impl FooterColors {
    pub fn from_theme(theme: &Theme) -> Self {
        let colors = &theme.colors;
        // Logo icon color: For Script Kit, we use black (0x000000) on gold/yellow
        // accent background for brand consistency and maximum contrast.
        let logo_icon_hex = 0x000000u32; // Black for contrast on yellow/gold

        // Theme-aware overlay: black for dark mode (darkens), white for light mode (lightens)
        // 50% opacity (0x80) for vibrancy balance
        let overlay_bg = if theme.has_dark_colors() {
            0x000000u32.rgba8(0x80) // black at 50% for dark mode
        } else {
            0xffffffu32.rgba8(0x80) // white at 50% for light mode
        };

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
