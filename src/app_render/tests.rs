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
        assert_eq!(
            builtin_feature_annotation(&BuiltInFeature::PasteSequentially),
            "Paste Sequentially"
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
        let typography = designs::DesignTypography {
            font_size_xs: 9.25,
            ..designs::DesignTypography::default()
        };

        assert_eq!(
            preview_panel_typography_section_label_size(typography),
            typography.font_size_xs
        );
    }

    #[test]
    fn test_preview_panel_typography_body_line_height_uses_relaxed_multiplier() {
        let typography = designs::DesignTypography {
            font_size_sm: 13.0,
            line_height_relaxed: 1.6,
            ..designs::DesignTypography::default()
        };

        let line_height = preview_panel_typography_body_line_height(typography);

        assert!((line_height - 20.8).abs() < 0.0001);
    }

    #[test]
    fn test_truncate_preview_line_for_display_does_not_split_unicode_scalars() {
        let line = "A\u{1F680}BCDEF";

        let truncated = truncate_preview_line_for_display(line, 3);

        assert_eq!(truncated, "A\u{1F680}B...");
    }

    #[test]
    fn test_truncate_preview_line_for_display_returns_ellipsis_when_max_is_zero() {
        let truncated = truncate_preview_line_for_display("abcdef", 0);
        assert_eq!(truncated, "...");
    }

    #[test]
    fn test_preview_scriptlet_cache_key_changes_with_theme_and_source() {
        let base = scripts::Scriptlet {
            name: "Open Repo".to_string(),
            description: None,
            code: "echo hello".to_string(),
            tool: "bash".to_string(),
            shortcut: None,
            keyword: None,
            group: None,
            file_path: Some("/tmp/tools.md#open-repo".to_string()),
            command: Some("open-repo".to_string()),
            alias: None,
        };

        let dark_key = preview_scriptlet_cache_key(&base, true);
        let light_key = preview_scriptlet_cache_key(&base, false);
        assert_ne!(dark_key, light_key);

        let mut different_source = base.clone();
        different_source.file_path = Some("/tmp/other.md#open-repo".to_string());
        let different_source_key = preview_scriptlet_cache_key(&different_source, true);
        assert_ne!(dark_key, different_source_key);
    }
}
