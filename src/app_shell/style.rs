//! Style caching for the shell
//!
//! Caches computed styles to avoid recomputation every render.
//! Invalidated when theme changes.

use gpui::{px, Hsla, Pixels};

use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

/// Cached styles for the shell
///
/// Stored in the app root, recomputed only when theme changes.
/// Pass to AppShell::render to avoid per-render style computation.
#[derive(Clone, Debug)]
pub struct ShellStyleCache {
    /// Theme revision for invalidation
    pub theme_revision: u64,

    /// Main frame background with vibrancy opacity
    pub frame_bg: Hsla,
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

        // Frame background with vibrancy opacity (70-85% per CLAUDE.md vibrancy gotcha)
        let bg_alpha = theme
            .opacity
            .as_ref()
            .map(|o| o.main)
            .unwrap_or(0.85)
            .clamp(0.70, 0.85);
        let frame_bg = hex_to_hsla_with_alpha(colors.background.main, bg_alpha);

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
}

impl HeaderColors {
    pub fn from_theme(theme: &Theme) -> Self {
        let colors = &theme.colors;
        Self {
            text_primary: colors.text.primary.to_rgb(),
            text_muted: colors.text.muted.to_rgb(),
            text_dimmed: colors.text.dimmed.to_rgb(),
            accent: colors.accent.selected.to_rgb(),
            accent_hex: colors.accent.selected,
            background: colors.background.main.to_rgb(),
            search_box_bg: colors.background.search_box.to_rgb(),
            border: colors.ui.border.to_rgb(),
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
}

impl FooterColors {
    pub fn from_theme(theme: &Theme) -> Self {
        let colors = &theme.colors;
        Self {
            accent: colors.accent.selected.to_rgb(),
            accent_hex: colors.accent.selected,
            text_muted: colors.text.muted.to_rgb(),
            border: colors.ui.border.to_rgb(),
            border_hex: colors.ui.border,
            background: colors.background.main.to_rgb(),
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

/// Convert hex color to HSLA with specified alpha
fn hex_to_hsla_with_alpha(hex: u32, alpha: f32) -> Hsla {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f32::EPSILON {
        // Achromatic (gray)
        return Hsla {
            h: 0.0,
            s: 0.0,
            l,
            a: alpha,
        };
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < f32::EPSILON {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() < f32::EPSILON {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    Hsla {
        h: h / 6.0,
        s,
        l,
        a: alpha,
    }
}
