use super::chrome::AppChromeColors;
use super::{best_readable_text_hex, contrast_ratio, Theme};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeContrastSample {
    pub label: &'static str,
    pub foreground_hex: u32,
    pub background_hex: u32,
    pub ratio: f32,
    pub minimum: f32,
}

impl ThemeContrastSample {
    pub fn passes(&self) -> bool {
        self.ratio >= self.minimum
    }
}

fn sample(
    label: &'static str,
    foreground_hex: u32,
    background_hex: u32,
    minimum: f32,
) -> ThemeContrastSample {
    ThemeContrastSample {
        label,
        foreground_hex,
        background_hex,
        ratio: contrast_ratio(foreground_hex, background_hex),
        minimum,
    }
}

/// Composite foreground hex at alpha over background hex (simple alpha blend).
fn composite_over(fg_hex: u32, alpha: f32, bg_hex: u32) -> u32 {
    let blend = |fg_ch: u32, bg_ch: u32| -> u32 {
        ((fg_ch as f32 * alpha + bg_ch as f32 * (1.0 - alpha)).round() as u32).min(255)
    };
    let r = blend((fg_hex >> 16) & 0xFF, (bg_hex >> 16) & 0xFF);
    let g = blend((fg_hex >> 8) & 0xFF, (bg_hex >> 8) & 0xFF);
    let b = blend(fg_hex & 0xFF, bg_hex & 0xFF);
    (r << 16) | (g << 8) | b
}

/// Composite a 0xRRGGBBAA value over an opaque 0xRRGGBB background.
fn composite_rgba_over(rgba_hex: u32, bg_hex: u32) -> u32 {
    let fg_hex = rgba_hex >> 8;
    let alpha = (rgba_hex & 0xFF) as f32 / 255.0;
    composite_over(fg_hex, alpha, bg_hex)
}

pub fn audit_theme_contrast(theme: &Theme) -> Vec<ThemeContrastSample> {
    let colors = &theme.colors;
    let opacity = theme.get_opacity();
    let chrome = AppChromeColors::from_theme(theme);

    // Composited selection background: selected_subtle at opacity.selected over bg.main
    let selection_bg = composite_over(
        colors.accent.selected_subtle,
        opacity.selected,
        colors.background.main,
    );

    // Resolved chrome surfaces composited over the main background
    let surface_window = composite_rgba_over(chrome.window_surface_rgba, colors.background.main);
    let surface_input = composite_rgba_over(chrome.input_surface_rgba, colors.background.main);
    let surface_preview = composite_rgba_over(chrome.preview_surface_rgba, colors.background.main);
    let surface_panel = composite_rgba_over(chrome.panel_surface_rgba, colors.background.main);
    let surface_dialog = composite_rgba_over(chrome.dialog_surface_rgba, colors.background.main);
    let surface_log = composite_rgba_over(chrome.log_panel_surface_rgba, colors.background.main);
    let badge_bg = composite_rgba_over(chrome.badge_bg_rgba, colors.background.main);

    // Prompt/form proxy surfaces composited over main background
    let prompt_field_idle = composite_rgba_over(chrome.badge_bg_rgba, colors.background.main);
    let prompt_field_focused =
        composite_rgba_over(chrome.input_active_rgba, colors.background.main);
    let prompt_checkbox_checked =
        composite_rgba_over(chrome.accent_badge_bg_rgba, colors.background.main);

    let samples = vec![
        // ── Window surface ──────────────────────────────────────
        sample(
            "window.primary",
            colors.text.primary,
            colors.background.main,
            4.5,
        ),
        sample(
            "window.secondary",
            colors.text.secondary,
            colors.background.main,
            4.5,
        ),
        sample(
            "window.muted",
            colors.text.muted,
            colors.background.main,
            3.0,
        ),
        // ── Input / search box surface ──────────────────────────
        sample(
            "input.primary",
            colors.text.primary,
            colors.background.search_box,
            4.5,
        ),
        sample(
            "input.secondary",
            colors.text.secondary,
            colors.background.search_box,
            3.0,
        ),
        // ── Title bar / chrome surface ──────────────────────────
        sample(
            "chrome.primary",
            colors.text.primary,
            colors.background.title_bar,
            4.5,
        ),
        sample(
            "chrome.secondary",
            colors.text.secondary,
            colors.background.title_bar,
            3.0,
        ),
        // ── Selection / accent surfaces ─────────────────────────
        sample(
            "accent.on_accent",
            colors.text.on_accent,
            colors.accent.selected,
            4.5,
        ),
        sample("selection.primary", colors.text.primary, selection_bg, 4.5),
        // ── Border visibility ───────────────────────────────────
        sample(
            "border.on_window",
            colors.ui.border,
            colors.background.main,
            1.2,
        ),
        // ── Resolved chrome surfaces ────────────────────────────
        sample(
            "surface.window.primary",
            colors.text.primary,
            surface_window,
            4.5,
        ),
        sample(
            "surface.input.primary",
            colors.text.primary,
            surface_input,
            4.5,
        ),
        sample(
            "surface.preview.primary",
            colors.text.primary,
            surface_preview,
            4.5,
        ),
        sample(
            "surface.panel.primary",
            colors.text.primary,
            surface_panel,
            4.5,
        ),
        sample(
            "surface.dialog.primary",
            colors.text.primary,
            surface_dialog,
            4.5,
        ),
        sample("surface.log.primary", colors.text.primary, surface_log, 4.5),
        sample("badge.text", chrome.badge_text_hex, badge_bg, 3.0),
        // ── Prompt/form proxy surfaces ─────────────────────────
        sample(
            "prompt.field.label",
            colors.text.primary,
            prompt_field_idle,
            4.5,
        ),
        sample(
            "prompt.field.help",
            colors.text.secondary,
            prompt_field_idle,
            3.0,
        ),
        sample(
            "prompt.field.focused",
            colors.text.primary,
            prompt_field_focused,
            4.5,
        ),
        sample(
            "prompt.checkbox.checked",
            chrome.accent_badge_text_hex,
            prompt_checkbox_checked,
            4.5,
        ),
        // ── Semantic status colors ──────────────────────────────
        sample(
            "success.auto_text",
            best_readable_text_hex(colors.ui.success),
            colors.ui.success,
            4.5,
        ),
        sample(
            "warning.auto_text",
            best_readable_text_hex(colors.ui.warning),
            colors.ui.warning,
            4.5,
        ),
        sample(
            "error.auto_text",
            best_readable_text_hex(colors.ui.error),
            colors.ui.error,
            4.5,
        ),
        sample(
            "info.auto_text",
            best_readable_text_hex(colors.ui.info),
            colors.ui.info,
            4.5,
        ),
    ];

    samples
}

pub fn worst_theme_contrast(theme: &Theme) -> ThemeContrastSample {
    audit_theme_contrast(theme)
        .into_iter()
        .min_by(|a, b| {
            a.ratio
                .partial_cmp(&b.ratio)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| sample("window.primary", 0xFFFFFF, 0x000000, 4.5))
}

pub fn theme_contrast_score(theme: &Theme) -> (usize, usize) {
    let samples = audit_theme_contrast(theme);
    let total = samples.len();
    let passing = samples.iter().filter(|s| s.passes()).count();
    (passing, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_dark_theme_has_expected_sample_count() {
        let theme = Theme::dark_default();
        let samples = audit_theme_contrast(&theme);
        assert_eq!(samples.len(), 25);
    }

    #[test]
    fn default_light_theme_passes_all_contrast_checks() {
        let theme = Theme::light_default();
        let samples = audit_theme_contrast(&theme);
        let failing: Vec<_> = samples.iter().filter(|s| !s.passes()).collect();
        assert!(
            failing.is_empty(),
            "light default should pass all contrast checks, failing: {:?}",
            failing
        );
    }

    #[test]
    fn worst_contrast_returns_a_valid_sample() {
        let theme = Theme::dark_default();
        let worst = worst_theme_contrast(&theme);
        assert!(worst.ratio > 0.0, "worst contrast ratio should be positive");
    }

    #[test]
    fn sample_passes_respects_minimum() {
        let passing = ThemeContrastSample {
            label: "test",
            foreground_hex: 0xFFFFFF,
            background_hex: 0x000000,
            ratio: 21.0,
            minimum: 4.5,
        };
        assert!(passing.passes());

        let failing = ThemeContrastSample {
            label: "test",
            foreground_hex: 0x808080,
            background_hex: 0x909090,
            ratio: 1.2,
            minimum: 4.5,
        };
        assert!(!failing.passes());
    }
}
