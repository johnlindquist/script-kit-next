#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LayoutConfig, ScriptKitUserPreferences, ThemeSelectionPreferences};

    fn preferences_with_preset(preset_id: Option<&str>) -> ScriptKitUserPreferences {
        ScriptKitUserPreferences {
            layout: LayoutConfig::default(),
            theme: ThemeSelectionPreferences {
                preset_id: preset_id.map(ToString::to_string),
            },
        }
    }

    #[test]
    fn test_theme_from_user_preferences_loads_matching_preset() {
        let preferences = preferences_with_preset(Some("nord"));

        let from_preferences =
            theme_from_user_preferences(&preferences, "test-correlation").expect("theme expected");
        let expected = crate::theme::presets::all_presets()
            .into_iter()
            .find(|preset| preset.id == "nord")
            .expect("preset should exist")
            .create_theme();

        assert_eq!(
            from_preferences.colors.background.main,
            expected.colors.background.main
        );
        assert_eq!(
            from_preferences.colors.accent.selected,
            expected.colors.accent.selected
        );
    }

    #[test]
    fn test_theme_from_user_preferences_returns_none_for_unknown_preset() {
        let preferences = preferences_with_preset(Some("unknown-preset-id"));
        assert!(theme_from_user_preferences(&preferences, "test-correlation").is_none());
    }

    #[test]
    fn test_theme_from_user_preferences_returns_none_when_preset_unset() {
        let preferences = preferences_with_preset(None);
        assert!(theme_from_user_preferences(&preferences, "test-correlation").is_none());
    }
}
