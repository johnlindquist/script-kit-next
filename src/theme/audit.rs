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

pub fn audit_theme_contrast(theme: &Theme) -> Vec<ThemeContrastSample> {
    let colors = &theme.colors;
    vec![
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
            "input.primary",
            colors.text.primary,
            colors.background.search_box,
            4.5,
        ),
        sample(
            "accent.on_accent",
            colors.text.on_accent,
            colors.accent.selected,
            4.5,
        ),
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
    ]
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
        assert_eq!(samples.len(), 8);
    }

    #[test]
    fn default_light_theme_passes_all_contrast_checks() {
        let theme = Theme::light_default();
        let (passing, total) = theme_contrast_score(&theme);
        assert_eq!(passing, total, "light default should pass all contrast checks");
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
