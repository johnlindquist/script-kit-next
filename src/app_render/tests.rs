#[cfg(test)]
mod preview_panel_metadata_tests {
    use super::*;
    use crate::builtins::BuiltInFeature;

    #[test]
    fn test_preview_keyword_tags_dedupes_trims_and_limits() {
        let keywords = vec![
            "  screenshot ".to_string(),
            "capture".to_string(),
            "Screenshot".to_string(),
            "".to_string(),
            "tools".to_string(),
            "utility".to_string(),
            "shortcuts".to_string(),
            "extra".to_string(),
        ];

        let tags = preview_keyword_tags(&keywords);

        assert_eq!(
            tags,
            vec![
                "screenshot".to_string(),
                "capture".to_string(),
                "tools".to_string(),
                "utility".to_string(),
                "shortcuts".to_string(),
                "extra".to_string(),
            ]
        );
    }

    #[test]
    fn test_builtin_feature_annotation_maps_known_labels() {
        assert_eq!(
            builtin_feature_annotation(&BuiltInFeature::FileSearch),
            "File Browser"
        );
        assert_eq!(
            builtin_feature_annotation(&BuiltInFeature::AiChat),
            "AI Assistant"
        );
        assert_eq!(
            builtin_feature_annotation(&BuiltInFeature::UtilityCommand(
                crate::builtins::UtilityCommandType::ScratchPad,
            )),
            "Quick Utility"
        );
    }

    #[test]
    fn test_builtin_feature_annotation_uses_app_name_for_app_variant() {
        assert_eq!(
            builtin_feature_annotation(&BuiltInFeature::App("Visual Studio Code".into())),
            "Visual Studio Code"
        );
    }

    #[test]
    fn test_preview_panel_typography_section_label_size_uses_xs_token() {
        let mut typography = designs::DesignTypography::default();
        typography.font_size_xs = 9.25;

        assert_eq!(
            preview_panel_typography_section_label_size(typography),
            typography.font_size_xs
        );
    }

    #[test]
    fn test_preview_panel_typography_body_line_height_uses_relaxed_multiplier() {
        let mut typography = designs::DesignTypography::default();
        typography.font_size_sm = 13.0;
        typography.line_height_relaxed = 1.6;

        let line_height = preview_panel_typography_body_line_height(typography);

        assert!((line_height - 20.8).abs() < 0.0001);
    }
}
