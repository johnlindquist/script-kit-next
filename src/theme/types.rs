//! Theme type definitions
//!
//! Contains all the struct definitions for theme configuration:
//! - BackgroundOpacity, VibrancySettings, DropShadow
//! - BackgroundColors, TextColors, AccentColors, UIColors
//! - TerminalColors (ANSI 16-color palette)
//! - ColorScheme, FocusColorScheme, FocusAwareColorScheme
//! - FontConfig, Theme

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, error, info, warn};

use super::hex_color::{hex_color_serde, HexColor};

/// Background opacity settings for window transparency
/// Values range from 0.0 (fully transparent) to 1.0 (fully opaque)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundOpacity {
    /// Main background opacity (default: 0.85)
    pub main: f32,
    /// Title bar opacity (default: 0.9)
    pub title_bar: f32,
    /// Search box opacity (default: 0.92)
    pub search_box: f32,
    /// Log panel opacity (default: 0.8)
    pub log_panel: f32,
}

impl Default for BackgroundOpacity {
    fn default() -> Self {
        BackgroundOpacity {
            // Lower opacity values to allow vibrancy blur to show through
            // Values below ~0.7 will show more blur effect
            main: 0.60,       // Was 0.85 - lower for more vibrancy
            title_bar: 0.65,  // Was 0.9
            search_box: 0.70, // Was 0.92
            log_panel: 0.55,  // Was 0.8
        }
    }
}

/// Vibrancy/blur effect settings for the window background
/// This creates the native macOS translucent effect like Spotlight/Raycast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrancySettings {
    /// Whether vibrancy is enabled (default: true)
    pub enabled: bool,
    /// Vibrancy material type: "hud", "popover", "menu", "sidebar", "content"
    /// - "hud": Dark, high contrast (like HUD windows)
    /// - "popover": Light blur, used in popovers
    /// - "menu": Similar to system menus
    /// - "sidebar": Sidebar-style blur
    /// - "content": Content background blur
    ///
    /// Default: "popover" for a subtle, native feel
    pub material: String,
}

impl Default for VibrancySettings {
    fn default() -> Self {
        VibrancySettings {
            enabled: true,
            material: "popover".to_string(),
        }
    }
}

/// Drop shadow configuration for the window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropShadow {
    /// Whether drop shadow is enabled (default: true)
    pub enabled: bool,
    /// Blur radius for the shadow (default: 20.0)
    pub blur_radius: f32,
    /// Spread radius for the shadow (default: 0.0)
    pub spread_radius: f32,
    /// Horizontal offset (default: 0.0)
    pub offset_x: f32,
    /// Vertical offset (default: 8.0)
    pub offset_y: f32,
    /// Shadow color as hex (default: #000000 - black)
    #[serde(with = "hex_color_serde")]
    pub color: HexColor,
    /// Shadow opacity (default: 0.25)
    pub opacity: f32,
}

impl Default for DropShadow {
    fn default() -> Self {
        DropShadow {
            enabled: true,
            blur_radius: 20.0,
            spread_radius: 0.0,
            offset_x: 0.0,
            offset_y: 8.0,
            color: 0x000000,
            opacity: 0.25,
        }
    }
}

/// Background color definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundColors {
    /// Main background (#1E1E1E)
    #[serde(with = "hex_color_serde")]
    pub main: HexColor,
    /// Title bar background (#2D2D30)
    #[serde(with = "hex_color_serde")]
    pub title_bar: HexColor,
    /// Search box background (#3C3C3C)
    #[serde(with = "hex_color_serde")]
    pub search_box: HexColor,
    /// Log panel background (#0D0D0D)
    #[serde(with = "hex_color_serde")]
    pub log_panel: HexColor,
}

/// Text color definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextColors {
    /// Primary text color (#FFFFFF - white)
    #[serde(with = "hex_color_serde")]
    pub primary: HexColor,
    /// Secondary text color (#CCCCCC - light gray)
    #[serde(with = "hex_color_serde")]
    pub secondary: HexColor,
    /// Tertiary text color (#999999)
    #[serde(with = "hex_color_serde")]
    pub tertiary: HexColor,
    /// Muted text color (#808080)
    #[serde(with = "hex_color_serde")]
    pub muted: HexColor,
    /// Dimmed text color (#666666)
    #[serde(with = "hex_color_serde")]
    pub dimmed: HexColor,
}

/// Accent and highlight colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccentColors {
    /// Primary accent color (#FBBF24 - yellow/gold for Script Kit)
    /// Used for: selected items, button text, logo, highlights
    #[serde(with = "hex_color_serde")]
    pub selected: HexColor,
    /// Subtle selection for list items - barely visible highlight (#2A2A2A - dark gray)
    /// Used for polished, Raycast-like selection backgrounds
    #[serde(default = "default_selected_subtle", with = "hex_color_serde")]
    pub selected_subtle: HexColor,
}

/// Default subtle selection color (dark gray, barely visible)
fn default_selected_subtle() -> HexColor {
    0x2a2a2a
}

/// Border and UI element colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIColors {
    /// Border color (#464647)
    #[serde(with = "hex_color_serde")]
    pub border: HexColor,
    /// Success color for logs (#00FF00 - green)
    #[serde(with = "hex_color_serde")]
    pub success: HexColor,
    /// Error color for error messages (#EF4444 - red-500)
    #[serde(default = "default_error_color", with = "hex_color_serde")]
    pub error: HexColor,
    /// Warning color for warning messages (#F59E0B - amber-500)
    #[serde(default = "default_warning_color", with = "hex_color_serde")]
    pub warning: HexColor,
    /// Info color for informational messages (#3B82F6 - blue-500)
    #[serde(default = "default_info_color", with = "hex_color_serde")]
    pub info: HexColor,
}

/// Terminal ANSI color palette (16 colors)
///
/// These colors are used by the embedded terminal emulator for ANSI escape sequences.
/// Colors 0-7 are the normal palette, colors 8-15 are the bright variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalColors {
    /// ANSI 0: Black
    #[serde(default = "default_terminal_black", with = "hex_color_serde")]
    pub black: HexColor,
    /// ANSI 1: Red
    #[serde(default = "default_terminal_red", with = "hex_color_serde")]
    pub red: HexColor,
    /// ANSI 2: Green
    #[serde(default = "default_terminal_green", with = "hex_color_serde")]
    pub green: HexColor,
    /// ANSI 3: Yellow
    #[serde(default = "default_terminal_yellow", with = "hex_color_serde")]
    pub yellow: HexColor,
    /// ANSI 4: Blue
    #[serde(default = "default_terminal_blue", with = "hex_color_serde")]
    pub blue: HexColor,
    /// ANSI 5: Magenta
    #[serde(default = "default_terminal_magenta", with = "hex_color_serde")]
    pub magenta: HexColor,
    /// ANSI 6: Cyan
    #[serde(default = "default_terminal_cyan", with = "hex_color_serde")]
    pub cyan: HexColor,
    /// ANSI 7: White
    #[serde(default = "default_terminal_white", with = "hex_color_serde")]
    pub white: HexColor,
    /// ANSI 8: Bright Black (Gray)
    #[serde(default = "default_terminal_bright_black", with = "hex_color_serde")]
    pub bright_black: HexColor,
    /// ANSI 9: Bright Red
    #[serde(default = "default_terminal_bright_red", with = "hex_color_serde")]
    pub bright_red: HexColor,
    /// ANSI 10: Bright Green
    #[serde(default = "default_terminal_bright_green", with = "hex_color_serde")]
    pub bright_green: HexColor,
    /// ANSI 11: Bright Yellow
    #[serde(default = "default_terminal_bright_yellow", with = "hex_color_serde")]
    pub bright_yellow: HexColor,
    /// ANSI 12: Bright Blue
    #[serde(default = "default_terminal_bright_blue", with = "hex_color_serde")]
    pub bright_blue: HexColor,
    /// ANSI 13: Bright Magenta
    #[serde(default = "default_terminal_bright_magenta", with = "hex_color_serde")]
    pub bright_magenta: HexColor,
    /// ANSI 14: Bright Cyan
    #[serde(default = "default_terminal_bright_cyan", with = "hex_color_serde")]
    pub bright_cyan: HexColor,
    /// ANSI 15: Bright White
    #[serde(default = "default_terminal_bright_white", with = "hex_color_serde")]
    pub bright_white: HexColor,
}

// Terminal color defaults (VS Code dark theme inspired)
fn default_terminal_black() -> HexColor {
    0x000000
}
fn default_terminal_red() -> HexColor {
    0xcd3131
}
fn default_terminal_green() -> HexColor {
    0x0dbc79
}
fn default_terminal_yellow() -> HexColor {
    0xe5e510
}
fn default_terminal_blue() -> HexColor {
    0x2472c8
}
fn default_terminal_magenta() -> HexColor {
    0xbc3fbc
}
fn default_terminal_cyan() -> HexColor {
    0x11a8cd
}
fn default_terminal_white() -> HexColor {
    0xe5e5e5
}
fn default_terminal_bright_black() -> HexColor {
    0x666666
}
fn default_terminal_bright_red() -> HexColor {
    0xf14c4c
}
fn default_terminal_bright_green() -> HexColor {
    0x23d18b
}
fn default_terminal_bright_yellow() -> HexColor {
    0xf5f543
}
fn default_terminal_bright_blue() -> HexColor {
    0x3b8eea
}
fn default_terminal_bright_magenta() -> HexColor {
    0xd670d6
}
fn default_terminal_bright_cyan() -> HexColor {
    0x29b8db
}
fn default_terminal_bright_white() -> HexColor {
    0xffffff
}

impl Default for TerminalColors {
    fn default() -> Self {
        Self::dark_default()
    }
}

impl TerminalColors {
    /// Dark mode terminal colors (VS Code dark inspired)
    pub fn dark_default() -> Self {
        TerminalColors {
            black: 0x000000,
            red: 0xcd3131,
            green: 0x0dbc79,
            yellow: 0xe5e510,
            blue: 0x2472c8,
            magenta: 0xbc3fbc,
            cyan: 0x11a8cd,
            white: 0xe5e5e5,
            bright_black: 0x666666,
            bright_red: 0xf14c4c,
            bright_green: 0x23d18b,
            bright_yellow: 0xf5f543,
            bright_blue: 0x3b8eea,
            bright_magenta: 0xd670d6,
            bright_cyan: 0x29b8db,
            bright_white: 0xffffff,
        }
    }

    /// Light mode terminal colors
    pub fn light_default() -> Self {
        TerminalColors {
            black: 0x000000,
            red: 0xcd3131,
            green: 0x00bc00,
            yellow: 0x949800,
            blue: 0x0451a5,
            magenta: 0xbc05bc,
            cyan: 0x0598bc,
            white: 0x555555,
            bright_black: 0x666666,
            bright_red: 0xcd3131,
            bright_green: 0x14ce14,
            bright_yellow: 0xb5ba00,
            bright_blue: 0x0451a5,
            bright_magenta: 0xbc05bc,
            bright_cyan: 0x0598bc,
            bright_white: 0xa5a5a5,
        }
    }

    /// Get color by ANSI index (0-15)
    #[allow(dead_code)]
    pub fn get(&self, index: u8) -> HexColor {
        match index {
            0 => self.black,
            1 => self.red,
            2 => self.green,
            3 => self.yellow,
            4 => self.blue,
            5 => self.magenta,
            6 => self.cyan,
            7 => self.white,
            8 => self.bright_black,
            9 => self.bright_red,
            10 => self.bright_green,
            11 => self.bright_yellow,
            12 => self.bright_blue,
            13 => self.bright_magenta,
            14 => self.bright_cyan,
            15 => self.bright_white,
            _ => self.black, // Fallback
        }
    }
}

/// Default error color (red-500)
fn default_error_color() -> HexColor {
    0xef4444
}

/// Default warning color (amber-500)
fn default_warning_color() -> HexColor {
    0xf59e0b
}

/// Default info color (blue-500)
fn default_info_color() -> HexColor {
    0x3b82f6
}

/// Cursor styling for text input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorStyle {
    /// Cursor color when focused (#00FFFF - cyan)
    #[serde(with = "hex_color_serde")]
    pub color: HexColor,
    /// Cursor blink interval in milliseconds
    pub blink_interval_ms: u64,
}

/// Color scheme for a specific window focus state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusColorScheme {
    pub background: BackgroundColors,
    pub text: TextColors,
    pub accent: AccentColors,
    pub ui: UIColors,
    /// Optional cursor styling
    #[serde(default)]
    pub cursor: Option<CursorStyle>,
    /// Terminal ANSI colors (optional, defaults provided)
    #[serde(default)]
    pub terminal: TerminalColors,
}

/// Complete color scheme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub background: BackgroundColors,
    pub text: TextColors,
    pub accent: AccentColors,
    pub ui: UIColors,
    /// Terminal ANSI colors (optional, defaults provided)
    #[serde(default)]
    pub terminal: TerminalColors,
}

/// Window focus-aware theme with separate styles for focused and unfocused states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusAwareColorScheme {
    /// Colors when window is focused (default to standard colors if not specified)
    #[serde(default)]
    pub focused: Option<FocusColorScheme>,
    /// Colors when window is unfocused (dimmed/desaturated)
    #[serde(default)]
    pub unfocused: Option<FocusColorScheme>,
}

/// Font configuration for the editor and terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// Monospace font family for editor/terminal (default: "Menlo" on macOS)
    #[serde(default = "default_mono_font_family")]
    pub mono_family: String,
    /// Monospace font size in pixels (default: 14.0)
    #[serde(default = "default_mono_font_size")]
    pub mono_size: f32,
    /// UI font family (default: system font)
    #[serde(default = "default_ui_font_family")]
    pub ui_family: String,
    /// UI font size in pixels (default: 14.0)
    #[serde(default = "default_ui_font_size")]
    pub ui_size: f32,
}

fn default_mono_font_family() -> String {
    // JetBrains Mono is bundled with the app and registered at startup
    // It provides excellent code readability with ligatures support
    "JetBrains Mono".to_string()
}

fn default_mono_font_size() -> f32 {
    // 16px provides better readability, especially on high-DPI displays
    16.0
}

fn default_ui_font_family() -> String {
    ".SystemUIFont".to_string()
}

fn default_ui_font_size() -> f32 {
    // 16px provides better readability and matches rem_size for gpui-component
    16.0
}

impl Default for FontConfig {
    fn default() -> Self {
        FontConfig {
            mono_family: default_mono_font_family(),
            mono_size: default_mono_font_size(),
            ui_family: default_ui_font_family(),
            ui_size: default_ui_font_size(),
        }
    }
}

/// Complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub colors: ColorScheme,
    /// Optional focus-aware colors (new feature)
    #[serde(default)]
    pub focus_aware: Option<FocusAwareColorScheme>,
    /// Background opacity settings for window transparency
    #[serde(default)]
    pub opacity: Option<BackgroundOpacity>,
    /// Drop shadow configuration
    #[serde(default)]
    pub drop_shadow: Option<DropShadow>,
    /// Vibrancy/blur effect settings
    #[serde(default)]
    pub vibrancy: Option<VibrancySettings>,
    /// Font configuration for editor and terminal
    #[serde(default)]
    pub fonts: Option<FontConfig>,
}

#[allow(dead_code)]
impl CursorStyle {
    /// Create a default blinking cursor style
    pub fn default_focused() -> Self {
        CursorStyle {
            color: 0x00ffff, // Cyan cursor when focused
            blink_interval_ms: 500,
        }
    }
}

#[allow(dead_code)]
impl FocusColorScheme {
    /// Convert to a standard ColorScheme
    pub fn to_color_scheme(&self) -> ColorScheme {
        ColorScheme {
            background: self.background.clone(),
            text: self.text.clone(),
            accent: self.accent.clone(),
            ui: self.ui.clone(),
            terminal: self.terminal.clone(),
        }
    }
}

impl ColorScheme {
    /// Create a dark mode color scheme (default dark colors)
    pub fn dark_default() -> Self {
        ColorScheme {
            background: BackgroundColors {
                main: 0x1e1e1e,
                title_bar: 0x2d2d30,
                search_box: 0x3c3c3c,
                log_panel: 0x0d0d0d,
            },
            text: TextColors {
                primary: 0xffffff,
                secondary: 0xcccccc,
                tertiary: 0x999999,
                muted: 0x808080,
                dimmed: 0x666666,
            },
            accent: AccentColors {
                selected: 0xfbbf24,        // Script Kit primary: #fbbf24 (yellow/gold)
                selected_subtle: 0x2a2a2a, // Subtle dark gray for list selection backgrounds
            },
            ui: UIColors {
                border: 0x464647,
                success: 0x00ff00,
                error: 0xef4444,   // red-500
                warning: 0xf59e0b, // amber-500
                info: 0x3b82f6,    // blue-500
            },
            terminal: TerminalColors::dark_default(),
        }
    }

    /// Create a light mode color scheme
    pub fn light_default() -> Self {
        ColorScheme {
            background: BackgroundColors {
                main: 0xffffff,
                title_bar: 0xf3f3f3,
                search_box: 0xececec,
                log_panel: 0xfafafa,
            },
            text: TextColors {
                primary: 0x000000,
                secondary: 0x333333,
                tertiary: 0x666666,
                muted: 0x999999,
                dimmed: 0xcccccc,
            },
            accent: AccentColors {
                selected: 0x0078d4,
                selected_subtle: 0xe8e8e8, // Subtle light gray for list selections
            },
            ui: UIColors {
                border: 0xd0d0d0,
                success: 0x00a000,
                error: 0xdc2626,   // red-600 (darker for light mode)
                warning: 0xd97706, // amber-600 (darker for light mode)
                info: 0x2563eb,    // blue-600 (darker for light mode)
            },
            terminal: TerminalColors::light_default(),
        }
    }

    /// Create an unfocused (dimmed) version of this color scheme
    #[allow(dead_code)]
    pub fn to_unfocused(&self) -> Self {
        fn darken_hex(color: HexColor) -> HexColor {
            // Reduce brightness by blending towards mid-gray
            let r = (color >> 16) & 0xFF;
            let g = (color >> 8) & 0xFF;
            let b = color & 0xFF;

            // Reduce saturation and brightness: blend 30% toward gray
            let gray = 0x80u32;
            let new_r = ((r * 70 + gray * 30) / 100) as u8;
            let new_g = ((g * 70 + gray * 30) / 100) as u8;
            let new_b = ((b * 70 + gray * 30) / 100) as u8;

            ((new_r as u32) << 16) | ((new_g as u32) << 8) | (new_b as u32)
        }

        ColorScheme {
            background: BackgroundColors {
                main: darken_hex(self.background.main),
                title_bar: darken_hex(self.background.title_bar),
                search_box: darken_hex(self.background.search_box),
                log_panel: darken_hex(self.background.log_panel),
            },
            text: TextColors {
                primary: darken_hex(self.text.primary),
                secondary: darken_hex(self.text.secondary),
                tertiary: darken_hex(self.text.tertiary),
                muted: darken_hex(self.text.muted),
                dimmed: darken_hex(self.text.dimmed),
            },
            accent: AccentColors {
                selected: darken_hex(self.accent.selected),
                selected_subtle: darken_hex(self.accent.selected_subtle),
            },
            ui: UIColors {
                border: darken_hex(self.ui.border),
                success: darken_hex(self.ui.success),
                error: darken_hex(self.ui.error),
                warning: darken_hex(self.ui.warning),
                info: darken_hex(self.ui.info),
            },
            terminal: TerminalColors {
                black: darken_hex(self.terminal.black),
                red: darken_hex(self.terminal.red),
                green: darken_hex(self.terminal.green),
                yellow: darken_hex(self.terminal.yellow),
                blue: darken_hex(self.terminal.blue),
                magenta: darken_hex(self.terminal.magenta),
                cyan: darken_hex(self.terminal.cyan),
                white: darken_hex(self.terminal.white),
                bright_black: darken_hex(self.terminal.bright_black),
                bright_red: darken_hex(self.terminal.bright_red),
                bright_green: darken_hex(self.terminal.bright_green),
                bright_yellow: darken_hex(self.terminal.bright_yellow),
                bright_blue: darken_hex(self.terminal.bright_blue),
                bright_magenta: darken_hex(self.terminal.bright_magenta),
                bright_cyan: darken_hex(self.terminal.bright_cyan),
                bright_white: darken_hex(self.terminal.bright_white),
            },
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme::dark_default()
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            colors: ColorScheme::default(),
            focus_aware: None,
            opacity: Some(BackgroundOpacity::default()),
            drop_shadow: Some(DropShadow::default()),
            vibrancy: Some(VibrancySettings::default()),
            fonts: Some(FontConfig::default()),
        }
    }
}

#[allow(dead_code)]
impl Theme {
    /// Get the appropriate color scheme based on window focus state
    ///
    /// If focus-aware colors are configured:
    /// - Returns focused colors when focused=true
    /// - Returns unfocused colors when focused=false
    ///
    /// If focus-aware colors are not configured:
    /// - Always returns the standard colors (automatic dimmed version for unfocused)
    pub fn get_colors(&self, is_focused: bool) -> ColorScheme {
        if let Some(ref focus_aware) = self.focus_aware {
            if is_focused {
                if let Some(ref focused) = focus_aware.focused {
                    return focused.to_color_scheme();
                }
            } else if let Some(ref unfocused) = focus_aware.unfocused {
                return unfocused.to_color_scheme();
            }
        }

        // Fallback: use standard colors, with automatic dimming for unfocused
        if is_focused {
            self.colors.clone()
        } else {
            self.colors.to_unfocused()
        }
    }

    /// Get cursor style if window is focused
    pub fn get_cursor_style(&self, is_focused: bool) -> Option<CursorStyle> {
        if !is_focused {
            return None;
        }

        if let Some(ref focus_aware) = self.focus_aware {
            if let Some(ref focused) = focus_aware.focused {
                return focused.cursor.clone();
            }
        }

        // Return default blinking cursor if focused
        Some(CursorStyle::default_focused())
    }

    /// Get background opacity settings
    /// Returns the configured opacity or sensible defaults
    pub fn get_opacity(&self) -> BackgroundOpacity {
        self.opacity.clone().unwrap_or_default()
    }

    /// Get opacity adjusted for focus state
    /// Unfocused windows are slightly more transparent
    pub fn get_opacity_for_focus(&self, is_focused: bool) -> BackgroundOpacity {
        let base = self.get_opacity();
        if is_focused {
            base
        } else {
            // Reduce opacity by 10% when unfocused
            BackgroundOpacity {
                main: (base.main * 0.9).clamp(0.0, 1.0),
                title_bar: (base.title_bar * 0.9).clamp(0.0, 1.0),
                search_box: (base.search_box * 0.9).clamp(0.0, 1.0),
                log_panel: (base.log_panel * 0.9).clamp(0.0, 1.0),
            }
        }
    }

    /// Get drop shadow configuration
    /// Returns the configured shadow or sensible defaults
    pub fn get_drop_shadow(&self) -> DropShadow {
        self.drop_shadow.clone().unwrap_or_default()
    }

    /// Get vibrancy/blur effect settings
    /// Returns the configured vibrancy or sensible defaults
    pub fn get_vibrancy(&self) -> VibrancySettings {
        self.vibrancy.clone().unwrap_or_default()
    }

    /// Check if vibrancy effect should be enabled
    pub fn is_vibrancy_enabled(&self) -> bool {
        self.get_vibrancy().enabled
    }

    /// Get font configuration
    /// Returns the configured fonts or sensible defaults
    pub fn get_fonts(&self) -> FontConfig {
        self.fonts.clone().unwrap_or_default()
    }
}

/// Detect system appearance preference on macOS
///
/// Returns true if dark mode is enabled, false if light mode is enabled.
/// On non-macOS systems or if detection fails, defaults to true (dark mode).
///
/// Uses the `defaults read -g AppleInterfaceStyle` command to detect the system appearance.
pub fn detect_system_appearance() -> bool {
    // Try to detect macOS dark mode using system defaults
    match Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
    {
        Ok(output) => {
            // If the command succeeds and returns "Dark", we're in dark mode
            let stdout = String::from_utf8_lossy(&output.stdout);
            let is_dark = stdout.to_lowercase().contains("dark");
            info!(
                appearance = if is_dark { "dark" } else { "light" },
                "System appearance detected"
            );
            is_dark
        }
        Err(_) => {
            // Command failed or not available (e.g., light mode on macOS returns error)
            debug!(
                "System appearance detection failed or light mode detected, defaulting to light"
            );
            false
        }
    }
}

/// Load theme from ~/.scriptkit/kit/theme.json
///
/// Colors should be specified as decimal integers in the JSON file.
/// For example, 0x1e1e1e (hex) = 1980410 (decimal).
///
/// Example theme.json structure:
/// ```json
/// {
///   "colors": {
///     "background": {
///       "main": 1980410,
///       "title_bar": 2961712,
///       "search_box": 3947580,
///       "log_panel": 851213
///     },
///     "text": {
///       "primary": 16777215,
///       "secondary": 14737920,
///       "tertiary": 10066329,
///       "muted": 8421504,
///       "dimmed": 6710886
///     },
///     "accent": {
///       "selected": 31948
///     },
///     "ui": {
///       "border": 4609607,
///       "success": 65280
///     }
///   }
/// }
/// ```
///
/// If the file doesn't exist or fails to parse, returns a theme based on system appearance detection.
/// If system appearance detection is not available, defaults to dark mode.
/// Logs errors to stderr but doesn't fail the application.
pub fn load_theme() -> Theme {
    let theme_path = PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/theme.json").as_ref());

    // Check if theme file exists
    if !theme_path.exists() {
        warn!(path = %theme_path.display(), "Theme file not found, using defaults based on system appearance");
        // Auto-select based on system appearance
        let is_dark = detect_system_appearance();
        let color_scheme = if is_dark {
            ColorScheme::dark_default()
        } else {
            ColorScheme::light_default()
        };
        let theme = Theme {
            focus_aware: None,
            colors: color_scheme,
            opacity: Some(BackgroundOpacity::default()),
            drop_shadow: Some(DropShadow::default()),
            vibrancy: Some(VibrancySettings::default()),
            fonts: Some(FontConfig::default()),
        };
        log_theme_config(&theme);
        return theme;
    }

    // Read and parse the JSON file
    match std::fs::read_to_string(&theme_path) {
        Err(e) => {
            error!(path = %theme_path.display(), error = %e, "Failed to read theme file, using defaults");
            let is_dark = detect_system_appearance();
            let color_scheme = if is_dark {
                ColorScheme::dark_default()
            } else {
                ColorScheme::light_default()
            };
            let theme = Theme {
                colors: color_scheme,
                focus_aware: None,
                opacity: Some(BackgroundOpacity::default()),
                drop_shadow: Some(DropShadow::default()),
                vibrancy: Some(VibrancySettings::default()),
                fonts: Some(FontConfig::default()),
            };
            log_theme_config(&theme);
            theme
        }
        Ok(contents) => match serde_json::from_str::<Theme>(&contents) {
            Ok(theme) => {
                debug!(path = %theme_path.display(), "Successfully loaded theme");
                log_theme_config(&theme);
                theme
            }
            Err(e) => {
                error!(
                    path = %theme_path.display(),
                    error = %e,
                    "Failed to parse theme JSON, using defaults"
                );
                debug!(content = %contents, "Malformed theme file content");
                let is_dark = detect_system_appearance();
                let color_scheme = if is_dark {
                    ColorScheme::dark_default()
                } else {
                    ColorScheme::light_default()
                };
                let theme = Theme {
                    colors: color_scheme,
                    focus_aware: None,
                    opacity: Some(BackgroundOpacity::default()),
                    drop_shadow: Some(DropShadow::default()),
                    vibrancy: Some(VibrancySettings::default()),
                    fonts: Some(FontConfig::default()),
                };
                log_theme_config(&theme);
                theme
            }
        },
    }
}
// ============================================================================
// End Lightweight Theme Extraction Helpers
// ============================================================================

/// Log theme configuration for debugging
fn log_theme_config(theme: &Theme) {
    let opacity = theme.get_opacity();
    let shadow = theme.get_drop_shadow();
    let vibrancy = theme.get_vibrancy();
    debug!(
        opacity_main = opacity.main,
        opacity_title_bar = opacity.title_bar,
        opacity_search_box = opacity.search_box,
        opacity_log_panel = opacity.log_panel,
        "Theme opacity configured"
    );
    debug!(
        shadow_enabled = shadow.enabled,
        blur_radius = shadow.blur_radius,
        spread_radius = shadow.spread_radius,
        offset_x = shadow.offset_x,
        offset_y = shadow.offset_y,
        shadow_opacity = shadow.opacity,
        "Theme shadow configured"
    );
    debug!(
        vibrancy_enabled = vibrancy.enabled,
        material = %vibrancy.material,
        "Theme vibrancy configured"
    );
    debug!(
        selected = format!("#{:06x}", theme.colors.accent.selected),
        selected_subtle = format!("#{:06x}", theme.colors.accent.selected_subtle),
        "Theme accent colors"
    );
    debug!(
        error = format!("#{:06x}", theme.colors.ui.error),
        warning = format!("#{:06x}", theme.colors.ui.warning),
        info = format!("#{:06x}", theme.colors.ui.info),
        "Theme status colors"
    );
}
