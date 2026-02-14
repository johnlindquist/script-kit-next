use super::*;

impl AiApp {
    pub(super) fn create_box_shadows(&self) -> Vec<BoxShadow> {
        self.cached_box_shadows.clone()
    }

    /// Compute box shadows from theme configuration (called once at construction)
    pub(super) fn compute_box_shadows() -> Vec<BoxShadow> {
        let theme = crate::theme::get_cached_theme();
        let shadow_config = theme.get_drop_shadow();

        if !shadow_config.enabled {
            return vec![];
        }

        // Convert hex color to HSLA
        let r = ((shadow_config.color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((shadow_config.color >> 8) & 0xFF) as f32 / 255.0;
        let b = (shadow_config.color & 0xFF) as f32 / 255.0;

        // Simple RGB to HSL conversion
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        let (h, s) = if max == min {
            (0.0, 0.0)
        } else {
            let d = max - min;
            let s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };
            let h = if max == r {
                (g - b) / d + if g < b { 6.0 } else { 0.0 }
            } else if max == g {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            };
            (h / 6.0, s)
        };

        vec![BoxShadow {
            color: hsla(h, s, l, shadow_config.opacity),
            offset: point(px(shadow_config.offset_x), px(shadow_config.offset_y)),
            blur_radius: px(shadow_config.blur_radius),
            spread_radius: px(shadow_config.spread_radius),
        }]
    }

    /// Update cached box shadows when theme changes
    pub fn update_theme(&mut self, _cx: &mut Context<Self>) {
        self.cached_box_shadows = Self::compute_box_shadows();
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
            let display_name = available_models
                .iter()
                .find(|m| m.id == chat.model_id)
                .map(|m| m.display_name.clone())
                .unwrap_or_else(|| chat.model_id.clone());

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
        let display_name = self
            .available_models
            .iter()
            .find(|m| m.id == model_id)
            .map(|m| m.display_name.clone())
            .unwrap_or_else(|| model_id.to_string());

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
