use super::*;

impl AiApp {
    fn model_display_name_for_provider(
        available_models: &[ModelInfo],
        model_id: &str,
        provider: &str,
    ) -> String {
        available_models
            .iter()
            .find(|m| m.id == model_id && m.provider == provider)
            .map(|m| m.display_name.clone())
            .unwrap_or_else(|| model_id.to_string())
    }

    /// Compute box shadows from theme configuration (called once at construction)
    pub(super) fn compute_box_shadows() -> Vec<BoxShadow> {
        let theme = crate::theme::get_cached_theme();
        let shadow_config = theme.get_drop_shadow();

        if !shadow_config.enabled {
            return vec![];
        }

        let color = crate::ui_foundation::hex_to_hsla_with_alpha(
            shadow_config.color,
            shadow_config.opacity,
        );

        vec![BoxShadow {
            color,
            offset: point(px(shadow_config.offset_x), px(shadow_config.offset_y)),
            blur_radius: px(shadow_config.blur_radius),
            spread_radius: px(shadow_config.spread_radius),
        }]
    }

    /// Compute the list of last used model+provider settings from recent chats
    /// Returns up to 3 unique model+provider combinations, most recent first
    pub(super) fn compute_last_used_settings(
        chats: &[Chat],
        available_models: &[ModelInfo],
    ) -> Vec<LastUsedSetting> {
        use std::collections::HashSet;

        let mut seen = HashSet::new();
        let mut result = Vec::new();

        // Iterate through chats (already sorted by updated_at DESC)
        for chat in chats.iter().take(20) {
            let key = format!("{}:{}", chat.model_id, chat.provider);
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key);

            // Look up display names from available models
            let display_name = Self::model_display_name_for_provider(
                available_models,
                &chat.model_id,
                &chat.provider,
            );

            let provider_display_name = match chat.provider.as_str() {
                "anthropic" => "Anthropic".to_string(),
                "openai" => "OpenAI".to_string(),
                "google" => "Google".to_string(),
                "groq" => "Groq".to_string(),
                "openrouter" => "OpenRouter".to_string(),
                "vercel" => "Vercel".to_string(),
                other => other.to_string(),
            };

            result.push(LastUsedSetting {
                model_id: chat.model_id.clone(),
                provider: chat.provider.clone(),
                display_name,
                provider_display_name,
            });

            // Stop after 3 unique settings
            if result.len() >= 3 {
                break;
            }
        }

        result
    }

    /// Update the last used settings when a new chat is created
    pub(super) fn update_last_used_settings(&mut self, model_id: &str, provider: &str) {
        // Find display names
        let display_name =
            Self::model_display_name_for_provider(&self.available_models, model_id, provider);

        let provider_display_name = match provider {
            "anthropic" => "Anthropic".to_string(),
            "openai" => "OpenAI".to_string(),
            "google" => "Google".to_string(),
            "groq" => "Groq".to_string(),
            "openrouter" => "OpenRouter".to_string(),
            "vercel" => "Vercel".to_string(),
            other => other.to_string(),
        };

        let new_setting = LastUsedSetting {
            model_id: model_id.to_string(),
            provider: provider.to_string(),
            display_name,
            provider_display_name,
        };

        // Remove any existing entry with same model+provider
        self.last_used_settings
            .retain(|s| !(s.model_id == model_id && s.provider == provider));

        // Insert at front
        self.last_used_settings.insert(0, new_setting);

        // Keep only 3
        self.last_used_settings.truncate(3);
    }

    // =====================================================
    // Vibrancy Helper Functions
    // =====================================================
    // These use the same approach as the main window (render_script_list.rs)
    // to ensure vibrancy works correctly by using rgba() with hex colors
    // directly from the Script Kit theme.
    // NOTE: hex_to_rgba_with_opacity moved to crate::ui_foundation (centralized)

    /// Get main background color with vibrancy opacity
    /// Uses Script Kit theme hex colors directly (like main window)
    pub(super) fn get_vibrancy_background() -> gpui::Rgba {
        let sk_theme = crate::theme::get_cached_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.main;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.main,
        ))
    }

    /// Get modal overlay background (theme-aware)
    ///
    /// Uses theme background colors for overlay instead of hardcoded black/white.
    /// 50% opacity (0x80) for good contrast without being too heavy.
    pub(super) fn get_modal_overlay_background() -> gpui::Rgba {
        let sk_theme = crate::theme::get_cached_theme();
        crate::theme::modal_overlay_bg(&sk_theme, 0x80)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_display_name_for_provider_uses_provider_when_model_ids_overlap() {
        let available_models = vec![
            ModelInfo::new("shared-model", "OpenAI Shared", "openai", true, 128_000),
            ModelInfo::new(
                "shared-model",
                "Anthropic Shared",
                "anthropic",
                true,
                200_000,
            ),
        ];

        let display_name =
            AiApp::model_display_name_for_provider(&available_models, "shared-model", "anthropic");

        assert_eq!(
            display_name, "Anthropic Shared",
            "Display name lookup must match on provider+model_id, not model_id alone"
        );
    }

    #[test]
    fn test_compute_last_used_settings_uses_provider_scoped_model_display_names() {
        let chats = vec![Chat::new("shared-model", "anthropic")];
        let available_models = vec![
            ModelInfo::new("shared-model", "OpenAI Shared", "openai", true, 128_000),
            ModelInfo::new(
                "shared-model",
                "Anthropic Shared",
                "anthropic",
                true,
                200_000,
            ),
        ];

        let settings = AiApp::compute_last_used_settings(&chats, &available_models);

        assert_eq!(settings.len(), 1);
        assert_eq!(
            settings[0].display_name, "Anthropic Shared",
            "Last-used settings should resolve model names using provider+model_id identity"
        );
    }
}
