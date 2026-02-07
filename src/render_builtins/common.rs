impl ScriptListApp {
    /// Available vibrancy material presets for the theme customizer
    const VIBRANCY_MATERIALS: &[(theme::VibrancyMaterial, &str)] = &[
        (theme::VibrancyMaterial::Hud, "HUD"),
        (theme::VibrancyMaterial::Popover, "Popover"),
        (theme::VibrancyMaterial::Menu, "Menu"),
        (theme::VibrancyMaterial::Sidebar, "Sidebar"),
        (theme::VibrancyMaterial::Content, "Content"),
    ];

    /// Available font size presets for the theme customizer
    const FONT_SIZE_PRESETS: &[(f32, &str)] = &[
        (12.0, "12"),
        (13.0, "13"),
        (14.0, "14"),
        (15.0, "15"),
        (16.0, "16"),
        (18.0, "18"),
        (20.0, "20"),
    ];

    /// Find the index of a vibrancy material in the presets array
    fn find_vibrancy_material_index(material: theme::VibrancyMaterial) -> usize {
        Self::VIBRANCY_MATERIALS
            .iter()
            .position(|(m, _)| *m == material)
            .unwrap_or(0)
    }

    /// Return a human-readable name for a hex accent color
    fn accent_color_name(color: u32) -> &'static str {
        match color {
            0xfbbf24 => "Yellow Gold",
            0xf59e0b => "Amber",
            0xf97316 => "Orange",
            0xef4444 => "Red",
            0xec4899 => "Pink",
            0xa855f7 => "Purple",
            0x6366f1 => "Indigo",
            0x3b82f6 => "Blue",
            0x0078d4 => "Blue",
            0x0ea5e9 => "Sky",
            0x14b8a6 => "Teal",
            0x22c55e => "Green",
            0x84cc16 => "Lime",
            _ => "Custom",
        }
    }
}
