// ============================================================================
// Separator Category
// ============================================================================

/// Categories for grouping separator styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum SeparatorCategory {
    /// Line-based separators using horizontal rules
    LineBased,
    /// Typographic separators focusing on text styling
    Typographic,
    /// Decorative separators with symbols and ornaments
    Decorative,
    /// Spacing-based separators using whitespace
    SpacingBased,
    /// Background-based separators with fills and panels
    Background,
    /// Minimalist separators with minimal visual weight
    Minimalist,
    /// Retro separators with ASCII/terminal aesthetics
    Retro,
    /// Modern separators with effects and animations
    Modern,
}

impl SeparatorCategory {
    /// Get the display name for this category.
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            SeparatorCategory::LineBased => "Line-Based",
            SeparatorCategory::Typographic => "Typographic",
            SeparatorCategory::Decorative => "Decorative",
            SeparatorCategory::SpacingBased => "Spacing-Based",
            SeparatorCategory::Background => "Background",
            SeparatorCategory::Minimalist => "Minimalist",
            SeparatorCategory::Retro => "Retro",
            SeparatorCategory::Modern => "Modern",
        }
    }

    /// Get all separator styles in this category.
    #[allow(dead_code)]
    pub fn styles(&self) -> Vec<SeparatorStyle> {
        SeparatorStyle::all()
            .iter()
            .filter(|s| s.category() == *self)
            .copied()
            .collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separator_count() {
        // Verify we have 25+ separator styles
        assert!(
            SeparatorStyle::count() >= 25,
            "Expected at least 25 separator styles, got {}",
            SeparatorStyle::count()
        );
    }

    #[test]
    fn test_all_styles_have_names() {
        for style in SeparatorStyle::all() {
            let name = style.name();
            assert!(!name.is_empty(), "Style {:?} has empty name", style);
        }
    }

    #[test]
    fn test_all_styles_have_descriptions() {
        for style in SeparatorStyle::all() {
            let desc = style.description();
            assert!(!desc.is_empty(), "Style {:?} has empty description", style);
            assert!(
                desc.len() > 20,
                "Style {:?} description too short: {}",
                style,
                desc
            );
        }
    }

    #[test]
    fn test_all_styles_have_categories() {
        for style in SeparatorStyle::all() {
            // This should not panic
            let _ = style.category();
        }
    }

    #[test]
    fn test_all_styles_have_default_configs() {
        for style in SeparatorStyle::all() {
            let config = style.default_config();
            assert!(config.height > 0.0, "Style {:?} has invalid height", style);
        }
    }

    #[test]
    fn test_category_coverage() {
        // Ensure all categories have at least one style
        let categories = [
            SeparatorCategory::LineBased,
            SeparatorCategory::Typographic,
            SeparatorCategory::Decorative,
            SeparatorCategory::SpacingBased,
            SeparatorCategory::Background,
            SeparatorCategory::Minimalist,
            SeparatorCategory::Retro,
            SeparatorCategory::Modern,
        ];

        for category in categories {
            let styles = category.styles();
            assert!(!styles.is_empty(), "Category {:?} has no styles", category);
        }
    }

    #[test]
    fn test_recommended_styles_exist() {
        for variant in DesignVariant::all() {
            let recommended = SeparatorStyle::recommended_for(*variant);
            assert!(
                !recommended.is_empty(),
                "No recommended styles for {:?}",
                variant
            );
        }
    }

    #[test]
    fn test_config_defaults_are_reasonable() {
        let config = SeparatorConfig::default();

        assert_eq!(
            config.height, 24.0,
            "Default height should match SECTION_HEADER_HEIGHT"
        );
        assert!(config.line_thickness >= 0.5 && config.line_thickness <= 4.0);
        assert!(config.padding_x > 0.0);
        assert!(config.opacity >= 0.0 && config.opacity <= 1.0);
        assert_eq!(config.color_primary, SeparatorColorRole::UiBorder);
        assert_eq!(config.color_secondary, SeparatorColorRole::UiBorderSubtle);
        assert_eq!(config.color_background, SeparatorColorRole::UiSurface);
        assert_eq!(config.color_text, SeparatorColorRole::TextMuted);
    }

    #[test]
    fn test_style_default_is_solid_line() {
        assert_eq!(SeparatorStyle::default(), SeparatorStyle::SolidLine);
    }

    #[test]
    fn test_decorations_exist_for_decorated_styles() {
        // Styles that should have decorations
        let decorated = [
            SeparatorStyle::ChevronArrow,
            SeparatorStyle::DotsCenter,
            SeparatorStyle::BracketWrap,
            SeparatorStyle::AsciiBox,
            SeparatorStyle::BoxDrawing,
        ];

        for style in decorated {
            assert!(
                style.decorations().is_some(),
                "Style {:?} should have decorations",
                style
            );
        }
    }

    #[test]
    fn test_category_names_not_empty() {
        let categories = [
            SeparatorCategory::LineBased,
            SeparatorCategory::Typographic,
            SeparatorCategory::Decorative,
            SeparatorCategory::SpacingBased,
            SeparatorCategory::Background,
            SeparatorCategory::Minimalist,
            SeparatorCategory::Retro,
            SeparatorCategory::Modern,
        ];

        for category in categories {
            assert!(!category.name().is_empty());
        }
    }

    #[test]
    fn test_unreferenced_in_recommendations_reports_catalog_only_styles() {
        let unreferenced = SeparatorStyle::unreferenced_in_recommendations();

        assert!(
            !unreferenced.is_empty(),
            "Expected at least one catalog-only style so design audits can track coverage"
        );
        assert!(unreferenced.contains(&SeparatorStyle::DashedLine));
        assert!(unreferenced.contains(&SeparatorStyle::AnimatedFade));
    }

    #[test]
    fn test_shared_default_config_pairs_reports_known_pairs() {
        let pairs = SeparatorStyle::shared_default_config_pairs();

        assert!(
            pairs.contains(&(SeparatorStyle::DottedLine, SeparatorStyle::DashedLine)),
            "Expected dotted and dashed styles to share baseline config"
        );
        assert!(
            pairs.contains(&(SeparatorStyle::ColonPrefix, SeparatorStyle::SlashPrefix)),
            "Expected colon and slash prefix styles to share baseline config"
        );
    }

    #[test]
    fn test_color_role_fallbacks_remain_stable_for_audit_visibility() {
        assert_eq!(SeparatorColorRole::UiBorder.fallback_hex(), 0x464647);
        assert_eq!(SeparatorColorRole::AccentWarning.fallback_hex(), 0xfbbf24);
        assert_eq!(SeparatorColorRole::AccentTerminal.fallback_hex(), 0x00ff00);
        assert_eq!(SeparatorColorRole::AccentNeon.fallback_hex(), 0x00ffff);
    }
}
