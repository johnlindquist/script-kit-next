impl ScriptListApp {
    /// Helper: compute filtered preset indices from a filter string
    fn theme_chooser_filtered_indices(filter: &str) -> Vec<usize> {
        let presets = theme::presets::all_presets();
        if filter.is_empty() {
            (0..presets.len()).collect()
        } else {
            let f = filter.to_lowercase();
            presets
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    p.name.to_lowercase().contains(&f)
                        || p.description.to_lowercase().contains(&f)
                })
                .map(|(i, _)| i)
                .collect()
        }
    }

    /// Accent color palette for theme customization
    const ACCENT_PALETTE: &'static [(u32, &'static str)] = &[
        (0xFBBF24, "Amber"),
        (0x3B82F6, "Blue"),
        (0x8B5CF6, "Violet"),
        (0xEC4899, "Pink"),
        (0xEF4444, "Red"),
        (0xF97316, "Orange"),
        (0x22C55E, "Green"),
        (0x14B8A6, "Teal"),
        (0x06B6D4, "Cyan"),
        (0x6366F1, "Indigo"),
    ];

    /// Opacity presets for quick selection
    const OPACITY_PRESETS: &'static [(f32, &'static str)] = &[
        (0.10, "10%"),
        (0.30, "30%"),
        (0.50, "50%"),
        (0.80, "80%"),
        (1.00, "100%"),
    ];

    /// Compute on-accent text color based on accent luminance
    fn accent_on_text_color(accent: u32, bg_main: u32) -> u32 {
        let r = ((accent >> 16) & 0xFF) as f32;
        let g = ((accent >> 8) & 0xFF) as f32;
        let b = (accent & 0xFF) as f32;
        if (0.299 * r + 0.587 * g + 0.114 * b) > 128.0 {
            bg_main
        } else {
            0xFFFFFF
        }
    }

    /// Find the closest accent palette index for a given accent color
    fn find_accent_palette_index(accent: u32) -> Option<usize> {
        Self::ACCENT_PALETTE.iter().position(|&(c, _)| c == accent)
    }

    /// Find the closest opacity preset index for a given opacity value
    fn find_opacity_preset_index(opacity: f32) -> usize {
        Self::OPACITY_PRESETS
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.0 - opacity)
                    .abs()
                    .partial_cmp(&(b.0 - opacity).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Render the theme chooser with search, live preview, and preview panel
    pub(crate) fn render_theme_chooser(
        &mut self,
        filter: &str,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        include!("theme_chooser_setup_key.rs");
        include!("theme_chooser_list_header.rs");
        include!("theme_chooser_customize_controls.rs");
        include!("theme_chooser_preview_panel.rs");
        include!("theme_chooser_footer_return.rs");
    }
}
