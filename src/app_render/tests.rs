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
}
