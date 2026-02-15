//! Theme presets - Curated color schemes for the theme chooser
//!
//! Each preset returns a complete Theme struct with all colors configured.
//! Presets are designed to work well with Script Kit's vibrancy/blur effects.

// --- merged from part_01.rs ---
use super::types::{
    AccentColors, AppearanceMode, BackgroundColors, BackgroundOpacity, ColorScheme, DropShadow,
    FontConfig, TerminalColors, TextColors, Theme, UIColors, VibrancySettings,
};
use std::{collections::HashMap, sync::OnceLock};

/// A theme preset with metadata for the chooser UI
#[derive(Debug, Clone)]
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

struct PresetsCache {
    presets: Vec<ThemePreset>,
    preset_preview_colors: Vec<PresetPreviewColors>,
    first_light_theme_index: usize,
    preset_index_by_bg_accent: HashMap<u64, usize>,
}

impl PresetsCache {
    fn new() -> Self {
        let presets = all_presets();
        let first_light_theme_index = presets.iter().position(|p| !p.is_dark).unwrap_or(0);
        let mut preset_preview_colors = Vec::with_capacity(presets.len());
        let mut preset_index_by_bg_accent = HashMap::with_capacity(presets.len());

        for (index, preset) in presets.iter().enumerate() {
            let theme = preset.create_theme();
            let bg_main = theme.colors.background.main;
            let accent_selected = theme.colors.accent.selected;

            preset_preview_colors.push(PresetPreviewColors {
                bg: bg_main,
                accent: accent_selected,
                text: theme.colors.text.primary,
                secondary: theme.colors.text.secondary,
                border: theme.colors.ui.border,
            });
            preset_index_by_bg_accent.insert(preset_bg_accent_key(bg_main, accent_selected), index);
        }

        Self {
            presets,
            preset_preview_colors,
            first_light_theme_index,
            preset_index_by_bg_accent,
        }
    }
}

static PRESETS_CACHE: OnceLock<PresetsCache> = OnceLock::new();

fn preset_bg_accent_key(bg_main: u32, accent_selected: u32) -> u64 {
    ((bg_main as u64) << 32) | (accent_selected as u64)
}

fn presets_cache() -> &'static PresetsCache {
    PRESETS_CACHE.get_or_init(PresetsCache::new)
}

pub(crate) fn presets_cached() -> &'static [ThemePreset] {
    &presets_cache().presets
}

pub(crate) fn preset_preview_colors_cached() -> &'static [PresetPreviewColors] {
    &presets_cache().preset_preview_colors
}

/// Find the index of the preset matching the given theme, or 0 if not found.
/// Matches on (background.main, accent.selected) which is unique per preset.
pub fn find_current_preset_index(theme: &Theme) -> usize {
    let key = preset_bg_accent_key(theme.colors.background.main, theme.colors.accent.selected);
    presets_cache()
        .preset_index_by_bg_accent
        .get(&key)
        .copied()
        .unwrap_or(0)
}

/// Index of the first light theme in all_presets() (used for section separator rendering)
pub fn first_light_theme_index() -> usize {
    presets_cache().first_light_theme_index
}

/// Pre-compute preview colors for all presets (avoids creating themes in render closures)
pub fn all_preset_preview_colors() -> Vec<PresetPreviewColors> {
    preset_preview_colors_cached().to_vec()
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

// --- merged from part_02.rs ---
fn theme_dracula() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x282a36,
            title_bar: 0x21222c,
            search_box: 0x44475a,
            log_panel: 0x191a21,
        },
        text: TextColors {
            primary: 0xf8f8f2,
            secondary: 0xbfbfbf,
            tertiary: 0x6272a4,
            muted: 0x6272a4,
            dimmed: 0x44475a,
            on_accent: 0x282a36,
        },
        accent: AccentColors {
            selected: 0xbd93f9,
            selected_subtle: 0xf8f8f2,
        },
        ui: UIColors {
            border: 0x44475a,
            success: 0x50fa7b,
            error: 0xff5555,
            warning: 0xf1fa8c,
            info: 0x8be9fd,
        },
        terminal: TerminalColors {
            black: 0x21222c,
            red: 0xff5555,
            green: 0x50fa7b,
            yellow: 0xf1fa8c,
            blue: 0xbd93f9,
            magenta: 0xff79c6,
            cyan: 0x8be9fd,
            white: 0xf8f8f2,
            bright_black: 0x6272a4,
            bright_red: 0xff6e6e,
            bright_green: 0x69ff94,
            bright_yellow: 0xffffa5,
            bright_blue: 0xd6acff,
            bright_magenta: 0xff92df,
            bright_cyan: 0xa4ffff,
            bright_white: 0xffffff,
        },
    })
}

fn theme_nord() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x2e3440,
            title_bar: 0x3b4252,
            search_box: 0x434c5e,
            log_panel: 0x242933,
        },
        text: TextColors {
            primary: 0xeceff4,
            secondary: 0xd8dee9,
            tertiary: 0x81a1c1,
            muted: 0x7b88a1,
            dimmed: 0x4c566a,
            on_accent: 0x2e3440,
        },
        accent: AccentColors {
            selected: 0x88c0d0,
            selected_subtle: 0xeceff4,
        },
        ui: UIColors {
            border: 0x4c566a,
            success: 0xa3be8c,
            error: 0xbf616a,
            warning: 0xebcb8b,
            info: 0x81a1c1,
        },
        terminal: TerminalColors {
            black: 0x3b4252,
            red: 0xbf616a,
            green: 0xa3be8c,
            yellow: 0xebcb8b,
            blue: 0x81a1c1,
            magenta: 0xb48ead,
            cyan: 0x88c0d0,
            white: 0xe5e9f0,
            bright_black: 0x4c566a,
            bright_red: 0xbf616a,
            bright_green: 0xa3be8c,
            bright_yellow: 0xebcb8b,
            bright_blue: 0x81a1c1,
            bright_magenta: 0xb48ead,
            bright_cyan: 0x8fbcbb,
            bright_white: 0xeceff4,
        },
    })
}

fn theme_catppuccin_mocha() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x1e1e2e,
            title_bar: 0x181825,
            search_box: 0x313244,
            log_panel: 0x11111b,
        },
        text: TextColors {
            primary: 0xcdd6f4,
            secondary: 0xbac2de,
            tertiary: 0xa6adc8,
            muted: 0x7f849c,
            dimmed: 0x585b70,
            on_accent: 0x1e1e2e,
        },
        accent: AccentColors {
            selected: 0xcba6f7,
            selected_subtle: 0xcdd6f4,
        },
        ui: UIColors {
            border: 0x45475a,
            success: 0xa6e3a1,
            error: 0xf38ba8,
            warning: 0xf9e2af,
            info: 0x89b4fa,
        },
        terminal: TerminalColors {
            black: 0x45475a,
            red: 0xf38ba8,
            green: 0xa6e3a1,
            yellow: 0xf9e2af,
            blue: 0x89b4fa,
            magenta: 0xcba6f7,
            cyan: 0x94e2d5,
            white: 0xbac2de,
            bright_black: 0x585b70,
            bright_red: 0xf38ba8,
            bright_green: 0xa6e3a1,
            bright_yellow: 0xf9e2af,
            bright_blue: 0x89b4fa,
            bright_magenta: 0xcba6f7,
            bright_cyan: 0x94e2d5,
            bright_white: 0xa6adc8,
        },
    })
}

fn theme_catppuccin_latte() -> Theme {
    build_light_theme(ColorScheme {
        background: BackgroundColors {
            main: 0xeff1f5,
            title_bar: 0xe6e9ef,
            search_box: 0xdce0e8,
            log_panel: 0xccd0da,
        },
        text: TextColors {
            primary: 0x4c4f69,
            secondary: 0x5c5f77,
            tertiary: 0x6c6f85,
            muted: 0x8c8fa1,
            dimmed: 0x9ca0b0,
            on_accent: 0xeff1f5,
        },
        accent: AccentColors {
            selected: 0x8839ef,
            selected_subtle: 0x4c4f69,
        },
        ui: UIColors {
            border: 0xbcc0cc,
            success: 0x40a02b,
            error: 0xd20f39,
            warning: 0xdf8e1d,
            info: 0x1e66f5,
        },
        terminal: TerminalColors {
            black: 0x5c5f77,
            red: 0xd20f39,
            green: 0x40a02b,
            yellow: 0xdf8e1d,
            blue: 0x1e66f5,
            magenta: 0x8839ef,
            cyan: 0x179299,
            white: 0xacb0be,
            bright_black: 0x6c6f85,
            bright_red: 0xd20f39,
            bright_green: 0x40a02b,
            bright_yellow: 0xdf8e1d,
            bright_blue: 0x1e66f5,
            bright_magenta: 0x8839ef,
            bright_cyan: 0x179299,
            bright_white: 0xbcc0cc,
        },
    })
}

fn theme_one_dark() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x282c34,
            title_bar: 0x21252b,
            search_box: 0x3a3f4b,
            log_panel: 0x1b1d23,
        },
        text: TextColors {
            primary: 0xabb2bf,
            secondary: 0x9da5b4,
            tertiary: 0x7f848e,
            muted: 0x636d83,
            dimmed: 0x4b5263,
            on_accent: 0x282c34,
        },
        accent: AccentColors {
            selected: 0x61afef,
            selected_subtle: 0xabb2bf,
        },
        ui: UIColors {
            border: 0x3e4452,
            success: 0x98c379,
            error: 0xe06c75,
            warning: 0xe5c07b,
            info: 0x61afef,
        },
        terminal: TerminalColors {
            black: 0x3f4451,
            red: 0xe06c75,
            green: 0x98c379,
            yellow: 0xe5c07b,
            blue: 0x61afef,
            magenta: 0xc678dd,
            cyan: 0x56b6c2,
            white: 0xabb2bf,
            bright_black: 0x4f5666,
            bright_red: 0xbe5046,
            bright_green: 0x98c379,
            bright_yellow: 0xd19a66,
            bright_blue: 0x61afef,
            bright_magenta: 0xc678dd,
            bright_cyan: 0x56b6c2,
            bright_white: 0xd7dae0,
        },
    })
}

fn theme_tokyo_night() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x1a1b26,
            title_bar: 0x16161e,
            search_box: 0x292e42,
            log_panel: 0x13131a,
        },
        text: TextColors {
            primary: 0xc0caf5,
            secondary: 0xa9b1d6,
            tertiary: 0x737aa2,
            muted: 0x565f89,
            dimmed: 0x414868,
            on_accent: 0x1a1b26,
        },
        accent: AccentColors {
            selected: 0x7aa2f7,
            selected_subtle: 0xc0caf5,
        },
        ui: UIColors {
            border: 0x3b4261,
            success: 0x9ece6a,
            error: 0xf7768e,
            warning: 0xe0af68,
            info: 0x7dcfff,
        },
        terminal: TerminalColors {
            black: 0x414868,
            red: 0xf7768e,
            green: 0x9ece6a,
            yellow: 0xe0af68,
            blue: 0x7aa2f7,
            magenta: 0xbb9af7,
            cyan: 0x7dcfff,
            white: 0xa9b1d6,
            bright_black: 0x565f89,
            bright_red: 0xf7768e,
            bright_green: 0x9ece6a,
            bright_yellow: 0xe0af68,
            bright_blue: 0x7aa2f7,
            bright_magenta: 0xbb9af7,
            bright_cyan: 0x7dcfff,
            bright_white: 0xc0caf5,
        },
    })
}

fn theme_gruvbox_dark() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x282828,
            title_bar: 0x1d2021,
            search_box: 0x3c3836,
            log_panel: 0x1d2021,
        },
        text: TextColors {
            primary: 0xebdbb2,
            secondary: 0xd5c4a1,
            tertiary: 0xa89984,
            muted: 0x928374,
            dimmed: 0x665c54,
            on_accent: 0x282828,
        },
        accent: AccentColors {
            selected: 0xfe8019,
            selected_subtle: 0xebdbb2,
        },
        ui: UIColors {
            border: 0x504945,
            success: 0xb8bb26,
            error: 0xfb4934,
            warning: 0xfabd2f,
            info: 0x83a598,
        },
        terminal: TerminalColors {
            black: 0x282828,
            red: 0xcc241d,
            green: 0x98971a,
            yellow: 0xd79921,
            blue: 0x458588,
            magenta: 0xb16286,
            cyan: 0x689d6a,
            white: 0xa89984,
            bright_black: 0x928374,
            bright_red: 0xfb4934,
            bright_green: 0xb8bb26,
            bright_yellow: 0xfabd2f,
            bright_blue: 0x83a598,
            bright_magenta: 0xd3869b,
            bright_cyan: 0x8ec07c,
            bright_white: 0xebdbb2,
        },
    })
}

fn theme_rose_pine() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x191724,
            title_bar: 0x1f1d2e,
            search_box: 0x26233a,
            log_panel: 0x13111e,
        },
        text: TextColors {
            primary: 0xe0def4,
            secondary: 0xc4a7e7,
            tertiary: 0x908caa,
            muted: 0x6e6a86,
            dimmed: 0x524f67,
            on_accent: 0x191724,
        },
        accent: AccentColors {
            selected: 0xebbcba,
            selected_subtle: 0xe0def4,
        },
        ui: UIColors {
            border: 0x403d52,
            success: 0x31748f,
            error: 0xeb6f92,
            warning: 0xf6c177,
            info: 0x9ccfd8,
        },
        terminal: TerminalColors {
            black: 0x26233a,
            red: 0xeb6f92,
            green: 0x31748f,
            yellow: 0xf6c177,
            blue: 0x9ccfd8,
            magenta: 0xc4a7e7,
            cyan: 0xebbcba,
            white: 0xe0def4,
            bright_black: 0x6e6a86,
            bright_red: 0xeb6f92,
            bright_green: 0x31748f,
            bright_yellow: 0xf6c177,
            bright_blue: 0x9ccfd8,
            bright_magenta: 0xc4a7e7,
            bright_cyan: 0xebbcba,
            bright_white: 0xe0def4,
        },
    })
}

// --- merged from part_03.rs ---
fn theme_solarized_dark() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x002b36,
            title_bar: 0x073642,
            search_box: 0x073642,
            log_panel: 0x001e26,
        },
        text: TextColors {
            primary: 0xfdf6e3,
            secondary: 0xeee8d5,
            tertiary: 0x93a1a1,
            muted: 0x839496,
            dimmed: 0x657b83,
            on_accent: 0x002b36,
        },
        accent: AccentColors {
            selected: 0x268bd2,
            selected_subtle: 0xfdf6e3,
        },
        ui: UIColors {
            border: 0x586e75,
            success: 0x859900,
            error: 0xdc322f,
            warning: 0xb58900,
            info: 0x268bd2,
        },
        terminal: TerminalColors {
            black: 0x073642,
            red: 0xdc322f,
            green: 0x859900,
            yellow: 0xb58900,
            blue: 0x268bd2,
            magenta: 0xd33682,
            cyan: 0x2aa198,
            white: 0xeee8d5,
            bright_black: 0x586e75,
            bright_red: 0xcb4b16,
            bright_green: 0x859900,
            bright_yellow: 0xb58900,
            bright_blue: 0x268bd2,
            bright_magenta: 0x6c71c4,
            bright_cyan: 0x2aa198,
            bright_white: 0xfdf6e3,
        },
    })
}

fn theme_solarized_light() -> Theme {
    build_light_theme(ColorScheme {
        background: BackgroundColors {
            main: 0xfdf6e3,
            title_bar: 0xeee8d5,
            search_box: 0xeee8d5,
            log_panel: 0xe8e1cd,
        },
        text: TextColors {
            primary: 0x073642,
            secondary: 0x586e75,
            tertiary: 0x657b83,
            muted: 0x839496,
            dimmed: 0x93a1a1,
            on_accent: 0xfdf6e3,
        },
        accent: AccentColors {
            selected: 0x268bd2,
            selected_subtle: 0x073642,
        },
        ui: UIColors {
            border: 0x93a1a1,
            success: 0x859900,
            error: 0xdc322f,
            warning: 0xb58900,
            info: 0x268bd2,
        },
        terminal: TerminalColors {
            black: 0x073642,
            red: 0xdc322f,
            green: 0x859900,
            yellow: 0xb58900,
            blue: 0x268bd2,
            magenta: 0xd33682,
            cyan: 0x2aa198,
            white: 0xeee8d5,
            bright_black: 0x586e75,
            bright_red: 0xcb4b16,
            bright_green: 0x859900,
            bright_yellow: 0xb58900,
            bright_blue: 0x268bd2,
            bright_magenta: 0x6c71c4,
            bright_cyan: 0x2aa198,
            bright_white: 0xfdf6e3,
        },
    })
}

fn theme_github_dark() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x0d1117,
            title_bar: 0x161b22,
            search_box: 0x21262d,
            log_panel: 0x010409,
        },
        text: TextColors {
            primary: 0xf0f6fc,
            secondary: 0xc9d1d9,
            tertiary: 0x8b949e,
            muted: 0x6e7681,
            dimmed: 0x484f58,
            on_accent: 0xffffff,
        },
        accent: AccentColors {
            selected: 0x58a6ff,
            selected_subtle: 0xf0f6fc,
        },
        ui: UIColors {
            border: 0x30363d,
            success: 0x3fb950,
            error: 0xf85149,
            warning: 0xd29922,
            info: 0x58a6ff,
        },
        terminal: TerminalColors {
            black: 0x484f58,
            red: 0xff7b72,
            green: 0x3fb950,
            yellow: 0xd29922,
            blue: 0x58a6ff,
            magenta: 0xbc8cff,
            cyan: 0x39c5cf,
            white: 0xb1bac4,
            bright_black: 0x6e7681,
            bright_red: 0xffa198,
            bright_green: 0x56d364,
            bright_yellow: 0xe3b341,
            bright_blue: 0x79c0ff,
            bright_magenta: 0xd2a8ff,
            bright_cyan: 0x56d4dd,
            bright_white: 0xf0f6fc,
        },
    })
}

fn theme_github_light() -> Theme {
    build_light_theme(ColorScheme {
        background: BackgroundColors {
            main: 0xffffff,
            title_bar: 0xf6f8fa,
            search_box: 0xf6f8fa,
            log_panel: 0xf0f2f4,
        },
        text: TextColors {
            primary: 0x1f2328,
            secondary: 0x424a53,
            tertiary: 0x656d76,
            muted: 0x818b98,
            dimmed: 0xafb8c1,
            on_accent: 0xffffff,
        },
        accent: AccentColors {
            selected: 0x0969da,
            selected_subtle: 0x1f2328,
        },
        ui: UIColors {
            border: 0xd0d7de,
            success: 0x1a7f37,
            error: 0xcf222e,
            warning: 0x9a6700,
            info: 0x0969da,
        },
        terminal: TerminalColors {
            black: 0x24292f,
            red: 0xcf222e,
            green: 0x116329,
            yellow: 0x4d2d00,
            blue: 0x0550ae,
            magenta: 0x8250df,
            cyan: 0x1b7c83,
            white: 0x6e7781,
            bright_black: 0x57606a,
            bright_red: 0xa40e26,
            bright_green: 0x1a7f37,
            bright_yellow: 0x633c01,
            bright_blue: 0x0969da,
            bright_magenta: 0x8250df,
            bright_cyan: 0x1b7c83,
            bright_white: 0x8c959f,
        },
    })
}

fn theme_monokai_pro() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x2d2a2e,
            title_bar: 0x221f22,
            search_box: 0x403e41,
            log_panel: 0x19181a,
        },
        text: TextColors {
            primary: 0xfcfcfa,
            secondary: 0xc1c0c0,
            tertiary: 0x939293,
            muted: 0x727072,
            dimmed: 0x5b595c,
            on_accent: 0x2d2a2e,
        },
        accent: AccentColors {
            selected: 0xffd866,
            selected_subtle: 0xfcfcfa,
        },
        ui: UIColors {
            border: 0x403e41,
            success: 0xa9dc76,
            error: 0xff6188,
            warning: 0xfc9867,
            info: 0x78dce8,
        },
        terminal: TerminalColors {
            black: 0x403e41,
            red: 0xff6188,
            green: 0xa9dc76,
            yellow: 0xffd866,
            blue: 0x78dce8,
            magenta: 0xab9df2,
            cyan: 0x78dce8,
            white: 0xfcfcfa,
            bright_black: 0x727072,
            bright_red: 0xff6188,
            bright_green: 0xa9dc76,
            bright_yellow: 0xffd866,
            bright_blue: 0x78dce8,
            bright_magenta: 0xab9df2,
            bright_cyan: 0x78dce8,
            bright_white: 0xfcfcfa,
        },
    })
}

fn theme_everforest_dark() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x2d353b,
            title_bar: 0x272e33,
            search_box: 0x343f44,
            log_panel: 0x232a2e,
        },
        text: TextColors {
            primary: 0xd3c6aa,
            secondary: 0x9da9a0,
            tertiary: 0x859289,
            muted: 0x7a8478,
            dimmed: 0x56635f,
            on_accent: 0x2d353b,
        },
        accent: AccentColors {
            selected: 0xa7c080,
            selected_subtle: 0xd3c6aa,
        },
        ui: UIColors {
            border: 0x475258,
            success: 0xa7c080,
            error: 0xe67e80,
            warning: 0xdbbc7f,
            info: 0x7fbbb3,
        },
        terminal: TerminalColors {
            black: 0x343f44,
            red: 0xe67e80,
            green: 0xa7c080,
            yellow: 0xdbbc7f,
            blue: 0x7fbbb3,
            magenta: 0xd699b6,
            cyan: 0x83c092,
            white: 0xd3c6aa,
            bright_black: 0x56635f,
            bright_red: 0xe67e80,
            bright_green: 0xa7c080,
            bright_yellow: 0xdbbc7f,
            bright_blue: 0x7fbbb3,
            bright_magenta: 0xd699b6,
            bright_cyan: 0x83c092,
            bright_white: 0xd3c6aa,
        },
    })
}

fn theme_kanagawa() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x1f1f28,
            title_bar: 0x1a1a22,
            search_box: 0x2a2a37,
            log_panel: 0x16161d,
        },
        text: TextColors {
            primary: 0xdcd7ba,
            secondary: 0xc8c093,
            tertiary: 0x727169,
            muted: 0x625e5a,
            dimmed: 0x54546d,
            on_accent: 0x1f1f28,
        },
        accent: AccentColors {
            selected: 0x7e9cd8,
            selected_subtle: 0xdcd7ba,
        },
        ui: UIColors {
            border: 0x54546d,
            success: 0x76946a,
            error: 0xc34043,
            warning: 0xc0a36e,
            info: 0x7fb4ca,
        },
        terminal: TerminalColors {
            black: 0x2a2a37,
            red: 0xc34043,
            green: 0x76946a,
            yellow: 0xc0a36e,
            blue: 0x7e9cd8,
            magenta: 0x957fb8,
            cyan: 0x6a9589,
            white: 0xdcd7ba,
            bright_black: 0x54546d,
            bright_red: 0xe82424,
            bright_green: 0x98bb6c,
            bright_yellow: 0xe6c384,
            bright_blue: 0x7fb4ca,
            bright_magenta: 0x938aa9,
            bright_cyan: 0x7aa89f,
            bright_white: 0xc8c093,
        },
    })
}

fn theme_ayu_dark() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x0a0e14,
            title_bar: 0x070a0f,
            search_box: 0x1d2631,
            log_panel: 0x050709,
        },
        text: TextColors {
            primary: 0xb3b1ad,
            secondary: 0x9a9892,
            tertiary: 0x626a73,
            muted: 0x4d5566,
            dimmed: 0x3d4455,
            on_accent: 0x0a0e14,
        },
        accent: AccentColors {
            selected: 0xe6b450,
            selected_subtle: 0xb3b1ad,
        },
        ui: UIColors {
            border: 0x1d2631,
            success: 0xc2d94c,
            error: 0xff3333,
            warning: 0xff8f40,
            info: 0x59c2ff,
        },
        terminal: TerminalColors {
            black: 0x1d2631,
            red: 0xff3333,
            green: 0xc2d94c,
            yellow: 0xe6b450,
            blue: 0x59c2ff,
            magenta: 0xd2a6ff,
            cyan: 0x95e6cb,
            white: 0xb3b1ad,
            bright_black: 0x626a73,
            bright_red: 0xff3333,
            bright_green: 0xc2d94c,
            bright_yellow: 0xe6b450,
            bright_blue: 0x59c2ff,
            bright_magenta: 0xd2a6ff,
            bright_cyan: 0x95e6cb,
            bright_white: 0xb3b1ad,
        },
    })
}

fn theme_material_ocean() -> Theme {
    build_dark_theme(ColorScheme {
        background: BackgroundColors {
            main: 0x0f111a,
            title_bar: 0x090b10,
            search_box: 0x1f2233,
            log_panel: 0x070810,
        },
        text: TextColors {
            primary: 0xeeffff,
            secondary: 0xb0bec5,
            tertiary: 0x8f93a2,
            muted: 0x717cb4,
            dimmed: 0x3b3f51,
            on_accent: 0x0f111a,
        },
        accent: AccentColors {
            selected: 0x84ffff,
            selected_subtle: 0xeeffff,
        },
        ui: UIColors {
            border: 0x1f2233,
            success: 0xc3e88d,
            error: 0xff5370,
            warning: 0xffcb6b,
            info: 0x82aaff,
        },
        terminal: TerminalColors {
            black: 0x1f2233,
            red: 0xff5370,
            green: 0xc3e88d,
            yellow: 0xffcb6b,
            blue: 0x82aaff,
            magenta: 0xc792ea,
            cyan: 0x89ddff,
            white: 0xeeffff,
            bright_black: 0x3b3f51,
            bright_red: 0xff5370,
            bright_green: 0xc3e88d,
            bright_yellow: 0xffcb6b,
            bright_blue: 0x82aaff,
            bright_magenta: 0xc792ea,
            bright_cyan: 0x89ddff,
            bright_white: 0xeeffff,
        },
    })
}

// --- merged from part_04.rs ---
/// Write a theme to the user's theme.json file
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
            let json = serde_json::to_string_pretty(&theme);
            assert!(
                json.is_ok(),
                "Theme '{}' should serialize to JSON",
                preset.name
            );
        }
    }

    #[test]
    fn test_presets_cached_matches_all_presets_order_and_metadata() {
        let all = all_presets();
        let cached = presets_cached();
        assert_eq!(cached.len(), all.len());

        for (cached_preset, all_preset) in cached.iter().zip(all.iter()) {
            assert_eq!(cached_preset.id, all_preset.id);
            assert_eq!(cached_preset.name, all_preset.name);
            assert_eq!(cached_preset.description, all_preset.description);
            assert_eq!(cached_preset.is_dark, all_preset.is_dark);
        }
    }

    #[test]
    fn test_find_current_preset_index_does_lookup_for_each_cached_preset() {
        for (index, preset) in presets_cached().iter().enumerate() {
            let theme = preset.create_theme();
            assert_eq!(find_current_preset_index(&theme), index);
        }
    }

    #[test]
    fn test_find_current_preset_index_returns_zero_when_theme_not_in_cache() {
        let mut theme = presets_cached()[0].create_theme();
        let missing_bg = u32::MAX;
        let missing_accent = 1;
        let missing_key = preset_bg_accent_key(missing_bg, missing_accent);
        assert!(!presets_cache()
            .preset_index_by_bg_accent
            .contains_key(&missing_key));

        theme.colors.background.main = missing_bg;
        theme.colors.accent.selected = missing_accent;

        assert_eq!(find_current_preset_index(&theme), 0);
    }

    #[test]
    fn test_first_light_theme_index_uses_cached_value() {
        let expected = presets_cached()
            .iter()
            .position(|p| !p.is_dark)
            .unwrap_or(0);
        assert_eq!(first_light_theme_index(), expected);
    }

    #[test]
    fn test_all_preset_preview_colors_matches_cached_preview_colors() {
        let all_preview_colors = all_preset_preview_colors();
        let cached_preview_colors = preset_preview_colors_cached();
        assert_eq!(all_preview_colors.len(), cached_preview_colors.len());

        for (all_colors, cached_colors) in
            all_preview_colors.iter().zip(cached_preview_colors.iter())
        {
            assert_eq!(all_colors.bg, cached_colors.bg);
            assert_eq!(all_colors.accent, cached_colors.accent);
            assert_eq!(all_colors.text, cached_colors.text);
            assert_eq!(all_colors.secondary, cached_colors.secondary);
            assert_eq!(all_colors.border, cached_colors.border);
        }
    }
}
