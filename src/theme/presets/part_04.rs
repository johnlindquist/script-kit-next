/// Write a theme to the user's theme.json file
#[allow(dead_code)]
pub fn write_theme_to_disk(theme: &Theme) -> Result<(), std::io::Error> {
    let theme_path =
        std::path::PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/theme.json").as_ref());

    // Ensure parent directory exists
    if let Some(parent) = theme_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(theme).map_err(std::io::Error::other)?;

    std::fs::write(&theme_path, json)?;
    tracing::debug!(path = %theme_path.display(), "Theme written to disk");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_presets_are_valid() {
        let presets = all_presets();
        assert!(presets.len() >= 10, "Should have at least 10 theme presets");

        for preset in &presets {
            let theme = preset.create_theme();
            // Verify the theme has valid colors (non-zero background)
            assert!(
                theme.colors.background.main != 0 || preset.id == "github-dark",
                "Theme '{}' has zero background color",
                preset.name
            );
            assert!(
                theme.colors.text.primary != 0,
                "Theme '{}' has zero primary text color",
                preset.name
            );
        }
    }

    #[test]
    fn test_preset_ids_are_unique() {
        let presets = all_presets();
        let ids: Vec<&str> = presets.iter().map(|p| p.id).collect();
        for (i, id) in ids.iter().enumerate() {
            for (j, other) in ids.iter().enumerate() {
                if i != j {
                    assert_ne!(id, other, "Duplicate preset ID: {}", id);
                }
            }
        }
    }

    #[test]
    fn test_dark_presets_have_dark_appearance() {
        for preset in all_presets() {
            if preset.is_dark {
                let theme = preset.create_theme();
                assert_eq!(
                    theme.appearance,
                    AppearanceMode::Dark,
                    "Dark preset '{}' should have Dark appearance mode",
                    preset.name
                );
            }
        }
    }

    #[test]
    fn test_light_presets_have_light_appearance() {
        for preset in all_presets() {
            if !preset.is_dark {
                let theme = preset.create_theme();
                assert_eq!(
                    theme.appearance,
                    AppearanceMode::Light,
                    "Light preset '{}' should have Light appearance mode",
                    preset.name
                );
            }
        }
    }

    #[test]
    fn test_theme_serialization() {
        for preset in all_presets() {
            let theme = preset.create_theme();
            let json = theme_to_json(&theme);
            assert!(
                json.is_ok(),
                "Theme '{}' should serialize to JSON",
                preset.name
            );
        }
    }
}
