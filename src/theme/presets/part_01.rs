use super::types::{
    AccentColors, AppearanceMode, BackgroundColors, BackgroundOpacity, ColorScheme, DropShadow,
    FontConfig, TerminalColors, TextColors, Theme, UIColors, VibrancySettings,
};

/// A theme preset with metadata for the chooser UI
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ThemePreset {
    /// Unique identifier for the preset (used in tests and for persistence)
    pub id: &'static str,
    /// Display name shown in the chooser
    pub name: &'static str,
    /// Short description of the theme style
    pub description: &'static str,
    /// Whether this is a dark or light theme
    pub is_dark: bool,
    /// Function to create the theme
    theme: fn() -> Theme,
}

impl ThemePreset {
    /// Create the theme for this preset
    pub fn create_theme(&self) -> Theme {
        (self.theme)()
    }
}

/// Preview colors for rendering palette swatches in the theme chooser
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct PresetPreviewColors {
    pub bg: u32,
    pub accent: u32,
    pub text: u32,
    pub secondary: u32,
    pub border: u32,
}

/// Get all available theme presets (dark themes first, then light)
pub fn all_presets() -> Vec<ThemePreset> {
    vec![
        // ── Dark Themes ──────────────────────────────────────────
        ThemePreset {
            id: "script-kit-dark",
            name: "Script Kit Dark",
            description: "Default dark theme with yellow accent",
            is_dark: true,
            theme: theme_script_kit_dark,
        },
        ThemePreset {
            id: "dracula",
            name: "Dracula",
            description: "Dark theme with vibrant purple and pink",
            is_dark: true,
            theme: theme_dracula,
        },
        ThemePreset {
            id: "nord",
            name: "Nord",
            description: "Arctic, north-bluish color palette",
            is_dark: true,
            theme: theme_nord,
        },
        ThemePreset {
            id: "catppuccin-mocha",
            name: "Catppuccin Mocha",
            description: "Soothing pastel dark theme",
            is_dark: true,
            theme: theme_catppuccin_mocha,
        },
        ThemePreset {
            id: "one-dark",
            name: "One Dark",
            description: "Atom-inspired balanced dark theme",
            is_dark: true,
            theme: theme_one_dark,
        },
        ThemePreset {
            id: "tokyo-night",
            name: "Tokyo Night",
            description: "Clean dark theme with muted tones",
            is_dark: true,
            theme: theme_tokyo_night,
        },
        ThemePreset {
            id: "gruvbox-dark",
            name: "Gruvbox Dark",
            description: "Retro groove with warm earthy tones",
            is_dark: true,
            theme: theme_gruvbox_dark,
        },
        ThemePreset {
            id: "rose-pine",
            name: "Rosé Pine",
            description: "Elegant dark theme with muted rose",
            is_dark: true,
            theme: theme_rose_pine,
        },
        ThemePreset {
            id: "solarized-dark",
            name: "Solarized Dark",
            description: "Precision colors for machines and people",
            is_dark: true,
            theme: theme_solarized_dark,
        },
        ThemePreset {
            id: "github-dark",
            name: "GitHub Dark",
            description: "GitHub's dark default color scheme",
            is_dark: true,
            theme: theme_github_dark,
        },
        ThemePreset {
            id: "monokai-pro",
            name: "Monokai Pro",
            description: "Classic vibrant syntax theme",
            is_dark: true,
            theme: theme_monokai_pro,
        },
        ThemePreset {
            id: "everforest-dark",
            name: "Everforest Dark",
            description: "Nature-inspired warm green palette",
            is_dark: true,
            theme: theme_everforest_dark,
        },
        ThemePreset {
            id: "kanagawa",
            name: "Kanagawa",
            description: "Muted wave-inspired Japanese palette",
            is_dark: true,
            theme: theme_kanagawa,
        },
        ThemePreset {
            id: "ayu-dark",
            name: "Ayu Dark",
            description: "Minimal and modern dark theme",
            is_dark: true,
            theme: theme_ayu_dark,
        },
        ThemePreset {
            id: "material-ocean",
            name: "Material Ocean",
            description: "Material Design oceanic dark theme",
            is_dark: true,
            theme: theme_material_ocean,
        },
        // ── Light Themes ─────────────────────────────────────────
        ThemePreset {
            id: "script-kit-light",
            name: "Script Kit Light",
            description: "Default light theme with blue accent",
            is_dark: false,
            theme: theme_script_kit_light,
        },
        ThemePreset {
            id: "catppuccin-latte",
            name: "Catppuccin Latte",
            description: "Soothing pastel light theme",
            is_dark: false,
            theme: theme_catppuccin_latte,
        },
        ThemePreset {
            id: "solarized-light",
            name: "Solarized Light",
            description: "Warm light theme with balanced contrast",
            is_dark: false,
            theme: theme_solarized_light,
        },
        ThemePreset {
            id: "github-light",
            name: "GitHub Light",
            description: "GitHub's clean light color scheme",
            is_dark: false,
            theme: theme_github_light,
        },
    ]
}

/// Find the index of the preset matching the given theme, or 0 if not found.
/// Matches on (background.main, accent.selected) which is unique per preset.
pub fn find_current_preset_index(theme: &Theme) -> usize {
    let current_bg = theme.colors.background.main;
    let current_accent = theme.colors.accent.selected;
    let presets = all_presets();
    presets
        .iter()
        .position(|p| {
            let t = p.create_theme();
            t.colors.background.main == current_bg && t.colors.accent.selected == current_accent
        })
        .unwrap_or(0)
}

/// Index of the first light theme in all_presets() (used for section separator rendering)
#[allow(dead_code)]
pub fn first_light_theme_index() -> usize {
    all_presets().iter().position(|p| !p.is_dark).unwrap_or(0)
}

/// Pre-compute preview colors for all presets (avoids creating themes in render closures)
#[allow(dead_code)]
pub fn all_preset_preview_colors() -> Vec<PresetPreviewColors> {
    all_presets()
        .iter()
        .map(|p| {
            let t = p.create_theme();
            PresetPreviewColors {
                bg: t.colors.background.main,
                accent: t.colors.accent.selected,
                text: t.colors.text.primary,
                secondary: t.colors.text.secondary,
                border: t.colors.ui.border,
            }
        })
        .collect()
}

// ============================================================================
// Helper to build a theme from a color scheme
// ============================================================================

fn build_dark_theme(colors: ColorScheme) -> Theme {
    Theme {
        colors,
        focus_aware: None,
        opacity: Some(BackgroundOpacity::dark_default()),
        drop_shadow: Some(DropShadow::default()),
        vibrancy: Some(VibrancySettings::default()),
        fonts: Some(FontConfig::default()),
        appearance: AppearanceMode::Dark,
    }
}

fn build_light_theme(colors: ColorScheme) -> Theme {
    Theme {
        colors,
        focus_aware: None,
        opacity: Some(BackgroundOpacity::light_default()),
        drop_shadow: Some(DropShadow {
            opacity: 0.12,
            ..DropShadow::default()
        }),
        vibrancy: Some(VibrancySettings::default()),
        fonts: Some(FontConfig::default()),
        appearance: AppearanceMode::Light,
    }
}

// ============================================================================
// Theme Definitions
// ============================================================================

fn theme_script_kit_dark() -> Theme {
    Theme::dark_default()
}

fn theme_script_kit_light() -> Theme {
    build_light_theme(ColorScheme {
        background: BackgroundColors {
            main: 0xfafafa,
            title_bar: 0xffffff,
            search_box: 0xffffff,
            log_panel: 0xf5f5f5,
        },
        text: TextColors {
            primary: 0x1a1a1a,
            secondary: 0x4a4a4a,
            tertiary: 0x6b6b6b,
            muted: 0x808080,
            dimmed: 0xaaaaaa,
            on_accent: 0xffffff,
        },
        accent: AccentColors {
            selected: 0x0078d4,
            selected_subtle: 0x1a1a1a,
        },
        ui: UIColors {
            border: 0xe0e0e0,
            success: 0x22c55e,
            error: 0xdc2626,
            warning: 0xd97706,
            info: 0x2563eb,
        },
        terminal: TerminalColors::light_default(),
    })
}

