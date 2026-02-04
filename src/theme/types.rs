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
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};

/// Cache for system appearance detection to avoid spawning subprocesses on every render
static APPEARANCE_CACHE: OnceLock<Mutex<AppearanceCache>> = OnceLock::new();

/// Cache for loaded theme to avoid file I/O on every render
static THEME_CACHE: OnceLock<Mutex<ThemeCache>> = OnceLock::new();

/// How long to cache the system appearance before re-detecting
const APPEARANCE_CACHE_TTL: Duration = Duration::from_secs(5);

#[derive(Debug)]
struct AppearanceCache {
    is_dark: bool,
    last_check: Instant,
}

impl Default for AppearanceCache {
    fn default() -> Self {
        Self {
            is_dark: true,                                     // Default to dark mode
            last_check: Instant::now() - APPEARANCE_CACHE_TTL, // Force immediate check
        }
    }
}

/// Cache for loaded theme to avoid repeated file I/O
#[derive(Debug, Clone)]
struct ThemeCache {
    theme: Theme,
    loaded_at: Instant,
}

impl Default for ThemeCache {
    fn default() -> Self {
        // Create with a dark default theme - will be replaced on first load
        Self {
            theme: Theme::dark_default(),
            loaded_at: Instant::now() - Duration::from_secs(3600), // Force reload
        }
    }
}

use super::hex_color::{hex_color_serde, HexColor};

/// Theme appearance mode for determining light/dark rendering
///
/// This controls how the theme system renders colors and vibrancy effects.
/// - `Auto`: Detect from system preferences (macOS AppleInterfaceStyle)
/// - `Light`: Force light mode appearance
/// - `Dark`: Force dark mode appearance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AppearanceMode {
    /// Automatically detect from system preferences
    #[default]
    Auto,
    /// Force light mode appearance
    Light,
    /// Force dark mode appearance
    Dark,
}

/// Background opacity settings for window transparency
/// Values range from 0.0 (fully transparent) to 1.0 (fully opaque)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundOpacity {
    /// Main background opacity (default: 0.30)
    pub main: f32,
    /// Title bar opacity (default: 0.30)
    pub title_bar: f32,
    /// Search box/input opacity (default: 0.40)
    pub search_box: f32,
    /// Log panel opacity (default: 0.40)
    pub log_panel: f32,
    /// Selected list item background opacity (default: 0.35)
    #[serde(default = "default_selected_opacity")]
    pub selected: f32,
    /// Hovered list item background opacity (default: 0.25)
    #[serde(default = "default_hover_opacity")]
    pub hover: f32,
    /// Preview panel background opacity (default: 0.0)
    #[serde(default = "default_preview_opacity")]
    pub preview: f32,
    /// Dialog/popup background opacity (default: 0.60)
    #[serde(default = "default_dialog_opacity")]
    pub dialog: f32,
    /// Input field background opacity (default: 0.30)
    #[serde(default = "default_input_opacity")]
    pub input: f32,
    /// Panel/container background opacity (default: 0.20)
    #[serde(default = "default_panel_opacity")]
    pub panel: f32,
    /// Input field inactive/empty state background opacity (default: 0.25)
    #[serde(default = "default_input_inactive_opacity")]
    pub input_inactive: f32,
    /// Input field active/filled state background opacity (default: 0.50)
    #[serde(default = "default_input_active_opacity")]
    pub input_active: f32,
    /// Border inactive/empty state opacity (default: 0.125)
    #[serde(default = "default_border_inactive_opacity")]
    pub border_inactive: f32,
    /// Border active/filled state opacity (default: 0.25)
    #[serde(default = "default_border_active_opacity")]
    pub border_active: f32,
    /// Opacity for main window vibrancy background (0.0-1.0)
    /// Lower = more blur visible, Higher = more solid color
    /// Default: 0.85 for dark, 0.92 for light
    #[serde(default)]
    pub vibrancy_background: Option<f32>,
}

fn default_selected_opacity() -> f32 {
    0.12 // Subtle selection with vibrancy like Raycast
}

fn default_hover_opacity() -> f32 {
    0.07 // Very subtle hover feedback
}

fn default_preview_opacity() -> f32 {
    0.0
}

fn default_dialog_opacity() -> f32 {
    0.15 // Very low opacity - let vibrancy blur show through more
}

fn default_input_opacity() -> f32 {
    0.30
}

fn default_panel_opacity() -> f32 {
    0.20
}

fn default_input_inactive_opacity() -> f32 {
    0.25 // 0x40 / 255 ≈ 0.25
}

fn default_input_active_opacity() -> f32 {
    0.50 // 0x80 / 255 ≈ 0.50
}

fn default_border_inactive_opacity() -> f32 {
    0.125 // 0x20 / 255 ≈ 0.125
}

fn default_border_active_opacity() -> f32 {
    0.25 // 0x40 / 255 ≈ 0.25
}

impl BackgroundOpacity {
    /// Clamp all opacity values to the valid 0.0-1.0 range
    pub fn clamped(self) -> Self {
        Self {
            main: self.main.clamp(0.0, 1.0),
            title_bar: self.title_bar.clamp(0.0, 1.0),
            search_box: self.search_box.clamp(0.0, 1.0),
            log_panel: self.log_panel.clamp(0.0, 1.0),
            selected: self.selected.clamp(0.0, 1.0),
            hover: self.hover.clamp(0.0, 1.0),
            preview: self.preview.clamp(0.0, 1.0),
            dialog: self.dialog.clamp(0.0, 1.0),
            input: self.input.clamp(0.0, 1.0),
            panel: self.panel.clamp(0.0, 1.0),
            input_inactive: self.input_inactive.clamp(0.0, 1.0),
            input_active: self.input_active.clamp(0.0, 1.0),
            border_inactive: self.border_inactive.clamp(0.0, 1.0),
            border_active: self.border_active.clamp(0.0, 1.0),
            vibrancy_background: self.vibrancy_background.map(|v| v.clamp(0.0, 1.0)),
        }
    }
}

impl Default for BackgroundOpacity {
    fn default() -> Self {
        Self::dark_default()
    }
}

impl BackgroundOpacity {
    /// Dark mode opacity defaults - lower opacity for dark vibrancy
    pub fn dark_default() -> Self {
        BackgroundOpacity {
            // Lower opacity values to allow vibrancy blur to show through
            main: 0.30,                      // Root wrapper background
            title_bar: 0.30,                 // Title bar areas
            search_box: 0.40,                // Search input backgrounds
            log_panel: 0.40,                 // Log/terminal panels
            selected: 0.12,                  // Selected list item highlight - subtle with vibrancy
            hover: 0.07,                     // Hovered list item highlight - very subtle feedback
            preview: 0.0,                    // Preview panel (0 = fully transparent)
            dialog: 0.15, // Dialogs/popups - very low opacity, let vibrancy blur show through
            input: 0.30,  // Input fields
            panel: 0.20,  // Panels/containers
            input_inactive: 0.25, // Input fields when empty/inactive
            input_active: 0.50, // Input fields when has text/active
            border_inactive: 0.125, // Borders when inactive
            border_active: 0.25, // Borders when active
            vibrancy_background: Some(0.85), // Main window vibrancy background
        }
    }

    /// Light mode opacity defaults - tuned for vibrancy blur visibility
    ///
    /// Lower opacity allows more blur to show through while keeping text readable.
    /// Use Cmd+Shift+[ and Cmd+Shift+] to adjust opacity in real-time.
    ///
    /// These values are aligned with the vibrancy POC (src/bin/vibrancy-poc.rs):
    /// - POC uses rgba(0xFAFAFAD9) = #FAFAFA at 85% opacity (0xD9/255 = 0.851)
    /// - POC uses rgba(0xFFFFFFE6) = white at 90% opacity for input area
    pub fn light_default() -> Self {
        BackgroundOpacity {
            // 85% opacity matches the POC and provides good blur visibility
            // while maintaining text readability. Adjustable via Cmd+-/+
            main: 0.85,                      // 85% - matches POC container_bg
            title_bar: 0.85,                 // Match main for consistency
            search_box: 0.90,                // 90% - matches POC input_area_bg
            log_panel: 0.90,                 // Slightly more opaque for terminal readability
            selected: 0.20, // Light mode selection - stronger overlay for visibility on vibrancy
            hover: 0.12,    // Visible hover on light vibrancy backgrounds
            preview: 0.0,   // Preview panel (0 = fully transparent)
            dialog: 0.85,   // Dialogs match main
            input: 0.90,    // Input fields - 90% like POC input_area_bg
            panel: 0.85,    // Panels match main
            input_inactive: 0.85, // Input fields when empty/inactive
            input_active: 0.90, // Input fields when has text/active
            border_inactive: 0.30, // Borders when inactive
            border_active: 0.45, // Borders when active
            vibrancy_background: Some(0.85), // Match POC: 85% opacity (0xD9/255)
        }
    }
}

/// Vibrancy material type for macOS window backgrounds
///
/// Different materials provide different levels of blur and background interaction.
/// Maps to NSVisualEffectMaterial values on macOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VibrancyMaterial {
    /// Dark, high contrast material (like HUD windows)
    Hud,
    /// Light blur, used in popovers (default)
    #[default]
    Popover,
    /// Similar to system menus
    Menu,
    /// Sidebar-style blur
    Sidebar,
    /// Content background blur
    Content,
}

impl std::fmt::Display for VibrancyMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hud => write!(f, "hud"),
            Self::Popover => write!(f, "popover"),
            Self::Menu => write!(f, "menu"),
            Self::Sidebar => write!(f, "sidebar"),
            Self::Content => write!(f, "content"),
        }
    }
}

/// Vibrancy/blur effect settings for the window background
/// This creates the native macOS translucent effect like Spotlight/Raycast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrancySettings {
    /// Whether vibrancy is enabled (default: true)
    pub enabled: bool,
    /// Vibrancy material type
    /// - `hud`: Dark, high contrast (like HUD windows)
    /// - `popover`: Light blur, used in popovers (default)
    /// - `menu`: Similar to system menus
    /// - `sidebar`: Sidebar-style blur
    /// - `content`: Content background blur
    #[serde(default)]
    pub material: VibrancyMaterial,
}

impl Default for VibrancySettings {
    fn default() -> Self {
        VibrancySettings {
            enabled: true,
            material: VibrancyMaterial::default(),
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

impl DropShadow {
    /// Clamp opacity value to the valid 0.0-1.0 range
    ///
    /// This prevents invalid opacity values from config files from causing
    /// rendering issues.
    #[allow(dead_code)]
    pub fn clamped(self) -> Self {
        Self {
            opacity: self.opacity.clamp(0.0, 1.0),
            ..self
        }
    }
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
    /// Text color for content on accent backgrounds (#FFFFFF - white for dark themes)
    /// Used for text on selected items, warning banners, etc.
    #[serde(with = "hex_color_serde", default = "default_text_on_accent")]
    pub on_accent: HexColor,
}

fn default_text_on_accent() -> HexColor {
    0xffffff // White provides good contrast on most accent colors
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

/// Default subtle selection color
/// Uses white for near-invisible Raycast-like highlighting
fn default_selected_subtle() -> HexColor {
    0xffffff // White - rendered at very low opacity for subtle brightening
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
    0x50fa7b // Dracula green - vibrant for executables
}
fn default_terminal_yellow() -> HexColor {
    0xe5e510
}
fn default_terminal_blue() -> HexColor {
    0x5c9ceb // Brighter blue for directories
}
fn default_terminal_magenta() -> HexColor {
    0xbc3fbc
}
fn default_terminal_cyan() -> HexColor {
    0x56d4e2 // Brighter cyan for symlinks
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
    0x69ff94 // Very bright green
}
fn default_terminal_bright_yellow() -> HexColor {
    0xf5f543
}
fn default_terminal_bright_blue() -> HexColor {
    0x6eb4ff // Vibrant blue for directories
}
fn default_terminal_bright_magenta() -> HexColor {
    0xd670d6
}
fn default_terminal_bright_cyan() -> HexColor {
    0x8be9fd // Dracula cyan - very visible
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
    /// Dark mode terminal colors (Dracula/One Dark inspired for better visibility)
    pub fn dark_default() -> Self {
        TerminalColors {
            black: 0x000000,
            red: 0xcd3131,
            green: 0x50fa7b, // Dracula green - vibrant for executables
            yellow: 0xe5e510,
            blue: 0x5c9ceb, // Brighter blue for directories
            magenta: 0xbc3fbc,
            cyan: 0x56d4e2, // Brighter cyan for symlinks
            white: 0xe5e5e5,
            bright_black: 0x666666,
            bright_red: 0xf14c4c,
            bright_green: 0x69ff94, // Very bright green
            bright_yellow: 0xf5f543,
            bright_blue: 0x6eb4ff, // Vibrant blue for directories
            bright_magenta: 0xd670d6,
            bright_cyan: 0x8be9fd, // Dracula cyan - very visible
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
    /// Appearance mode: Auto (detect from system), Light, or Dark
    ///
    /// Controls how the theme renders colors and vibrancy effects.
    /// When set to Auto (default), the system appearance is detected.
    #[serde(default)]
    pub appearance: AppearanceMode,
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
                on_accent: 0xffffff, // White text on accent backgrounds
            },
            accent: AccentColors {
                selected: 0xfbbf24,        // Script Kit primary: #fbbf24 (yellow/gold)
                selected_subtle: 0xffffff, // White - near-invisible brightening like Raycast
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
    ///
    /// Colors derived from POC testing with Raycast-like light theme.
    /// Key differences from dark mode:
    /// - `selected_subtle` is BLACK (0x000000) instead of white for visible selection
    /// - Higher contrast text colors
    /// - Darker UI colors for visibility on light backgrounds
    pub fn light_default() -> Self {
        ColorScheme {
            background: BackgroundColors {
                main: 0xfafafa,       // Light gray from POC (0xFAFAFA)
                title_bar: 0xffffff,  // Pure white for input area
                search_box: 0xffffff, // Pure white for search
                log_panel: 0xf5f5f5,  // Slightly darker for terminal
            },
            text: TextColors {
                primary: 0x000000,   // Pure black for maximum contrast
                secondary: 0x4a4a4a, // Darker gray for better readability
                tertiary: 0x6b6b6b,  // Medium gray for hints
                muted: 0x808080,     // Mid gray for placeholders
                dimmed: 0x999999,    // Subtle but readable on light backgrounds
                on_accent: 0xffffff, // White text on accent backgrounds
            },
            accent: AccentColors {
                selected: 0x0078d4, // Blue accent for light mode
                // CRITICAL: Black for light mode selection visibility
                // White at low opacity is INVISIBLE on white backgrounds
                // Black at low opacity creates visible darkening effect
                selected_subtle: 0x000000,
            },
            ui: UIColors {
                border: 0xe0e0e0,  // Light border from POC (0xE0E0E0)
                success: 0x22c55e, // green-500
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
                on_accent: darken_hex(self.text.on_accent),
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
            appearance: AppearanceMode::default(),
        }
    }
}

impl Theme {
    /// Determine if the theme should render in dark mode
    ///
    /// This is the canonical method for checking theme appearance throughout the app.
    /// It handles the AppearanceMode enum and system detection.
    ///
    /// Returns:
    /// - `true` for dark mode rendering
    /// - `false` for light mode rendering
    pub fn is_dark_mode(&self) -> bool {
        match self.appearance {
            AppearanceMode::Dark => true,
            AppearanceMode::Light => false,
            AppearanceMode::Auto => detect_system_appearance(),
        }
    }

    /// Check if the theme's actual colors are dark (based on background luminance)
    ///
    /// This is used to determine vibrancy appearance (VibrantDark vs VibrantLight)
    /// regardless of system appearance setting. This ensures the blur effect matches
    /// the actual color scheme being used.
    ///
    /// Returns:
    /// - `true` if background color luminance < 0.5 (dark colors)
    /// - `false` if background color luminance >= 0.5 (light colors)
    pub fn has_dark_colors(&self) -> bool {
        // Extract RGB from main background color
        let bg = self.colors.background.main;
        let r = ((bg >> 16) & 0xFF) as f32 / 255.0;
        let g = ((bg >> 8) & 0xFF) as f32 / 255.0;
        let b = (bg & 0xFF) as f32 / 255.0;

        // Calculate relative luminance (ITU-R BT.709)
        let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;

        // Dark if luminance < 0.5
        luminance < 0.5
    }

    /// Determine if vibrancy should use dark appearance
    ///
    /// This checks the actual theme colors to determine whether to use
    /// VibrantDark or VibrantLight NSAppearance. This is separate from
    /// is_dark_mode() because we want vibrancy to match the colors being
    /// displayed, not the system preference.
    pub fn should_use_dark_vibrancy(&self) -> bool {
        self.has_dark_colors()
    }

    /// Create a light theme with appropriate defaults
    pub fn light_default() -> Self {
        Theme {
            colors: ColorScheme::light_default(),
            focus_aware: None,
            opacity: Some(BackgroundOpacity::light_default()),
            drop_shadow: Some(DropShadow {
                // Lighter shadow for light theme
                opacity: 0.12,
                ..DropShadow::default()
            }),
            vibrancy: Some(VibrancySettings::default()),
            fonts: Some(FontConfig::default()),
            appearance: AppearanceMode::Light,
        }
    }

    /// Create a dark theme with appropriate defaults
    pub fn dark_default() -> Self {
        Theme {
            colors: ColorScheme::dark_default(),
            focus_aware: None,
            opacity: Some(BackgroundOpacity::dark_default()),
            drop_shadow: Some(DropShadow::default()),
            vibrancy: Some(VibrancySettings::default()),
            fonts: Some(FontConfig::default()),
            appearance: AppearanceMode::Dark,
        }
    }
}

/// Background role for selecting the appropriate color and opacity
///
/// Use this enum with `Theme::background_rgba()` to get the correct
/// color with opacity applied for each UI region. This is the preferred
/// way to set background colors for vibrancy support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BackgroundRole {
    /// Main window background
    Main,
    /// Title bar background
    TitleBar,
    /// Search box / input field background
    SearchBox,
    /// Log panel background
    LogPanel,
}

/// Convert a HexColor to RGBA components (0.0-1.0 range)
fn hex_to_rgba_components(hex: HexColor, alpha: f32) -> (f32, f32, f32, f32) {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    (r, g, b, alpha.clamp(0.0, 1.0))
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

    /// Create a new theme with opacity adjusted by an offset
    ///
    /// Use Cmd+Shift+[ to decrease and Cmd+Shift+] to increase opacity.
    /// The offset is added to all opacity values (clamped to 0.0-1.0).
    ///
    /// # Arguments
    /// * `offset` - The amount to add to opacity values (can be negative)
    ///
    /// # Returns
    /// A new Theme with adjusted opacity values
    pub fn with_opacity_offset(&self, offset: f32) -> Theme {
        let mut theme = self.clone();
        let base = theme.get_opacity();
        theme.opacity = Some(BackgroundOpacity {
            main: (base.main + offset).clamp(0.0, 1.0),
            title_bar: (base.title_bar + offset).clamp(0.0, 1.0),
            search_box: (base.search_box + offset).clamp(0.0, 1.0),
            log_panel: (base.log_panel + offset).clamp(0.0, 1.0),
            selected: base.selected, // Keep selection/hover unchanged
            hover: base.hover,
            preview: base.preview,
            dialog: (base.dialog + offset).clamp(0.0, 1.0),
            input: (base.input + offset).clamp(0.0, 1.0),
            panel: (base.panel + offset).clamp(0.0, 1.0),
            input_inactive: (base.input_inactive + offset).clamp(0.0, 1.0),
            input_active: (base.input_active + offset).clamp(0.0, 1.0),
            border_inactive: base.border_inactive,
            border_active: base.border_active,
            vibrancy_background: base.vibrancy_background,
        });
        theme
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
                selected: (base.selected * 0.9).clamp(0.0, 1.0),
                hover: (base.hover * 0.9).clamp(0.0, 1.0),
                preview: (base.preview * 0.9).clamp(0.0, 1.0),
                dialog: (base.dialog * 0.9).clamp(0.0, 1.0),
                input: (base.input * 0.9).clamp(0.0, 1.0),
                panel: (base.panel * 0.9).clamp(0.0, 1.0),
                input_inactive: (base.input_inactive * 0.9).clamp(0.0, 1.0),
                input_active: (base.input_active * 0.9).clamp(0.0, 1.0),
                border_inactive: (base.border_inactive * 0.9).clamp(0.0, 1.0),
                border_active: (base.border_active * 0.9).clamp(0.0, 1.0),
                vibrancy_background: base.vibrancy_background.map(|v| (v * 0.9).clamp(0.0, 1.0)),
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

    /// Get background RGBA color for a specific role
    ///
    /// This is the single correct way to get background colors with opacity applied.
    /// It combines the color from the color scheme with the appropriate opacity value,
    /// ensuring consistent vibrancy support across the UI.
    ///
    /// # Arguments
    /// * `role` - The background role (Main, TitleBar, SearchBox, LogPanel)
    /// * `is_focused` - Whether the window is currently focused
    ///
    /// # Returns
    /// A tuple of (r, g, b, a) with values in the 0.0-1.0 range
    ///
    /// # Example
    /// ```ignore
    /// let (r, g, b, a) = theme.background_rgba(BackgroundRole::Main, true);
    /// div().bg(rgba(r, g, b, a))
    /// ```
    pub fn background_rgba(&self, role: BackgroundRole, is_focused: bool) -> (f32, f32, f32, f32) {
        let colors = self.get_colors(is_focused);
        let opacity = self.get_opacity_for_focus(is_focused).clamped();

        match role {
            BackgroundRole::Main => hex_to_rgba_components(colors.background.main, opacity.main),
            BackgroundRole::TitleBar => {
                hex_to_rgba_components(colors.background.title_bar, opacity.title_bar)
            }
            BackgroundRole::SearchBox => {
                hex_to_rgba_components(colors.background.search_box, opacity.search_box)
            }
            BackgroundRole::LogPanel => {
                hex_to_rgba_components(colors.background.log_panel, opacity.log_panel)
            }
        }
    }
}

/// Detect system appearance preference on macOS (cached)
///
/// Returns true if dark mode is enabled, false if light mode is enabled.
/// On non-macOS systems or if detection fails, defaults to true (dark mode).
///
/// This function caches the result for 5 seconds to avoid spawning subprocesses
/// on every render call. The system appearance doesn't change frequently, so
/// a small TTL is acceptable.
///
/// Uses the `defaults read -g AppleInterfaceStyle` command to detect the system appearance.
/// Note: On macOS in light mode, the command exits with non-zero status because the
/// AppleInterfaceStyle key doesn't exist, so we check exit status explicitly.
pub fn detect_system_appearance() -> bool {
    let cache = APPEARANCE_CACHE.get_or_init(|| Mutex::new(AppearanceCache::default()));

    let mut cache_guard = match cache.lock() {
        Ok(guard) => guard,
        Err(_) => {
            // Mutex poisoned, return default
            return true;
        }
    };

    // Check if cache is still valid
    if cache_guard.last_check.elapsed() < APPEARANCE_CACHE_TTL {
        return cache_guard.is_dark;
    }

    // Cache expired, re-detect
    let is_dark = detect_system_appearance_uncached();
    cache_guard.is_dark = is_dark;
    cache_guard.last_check = Instant::now();
    is_dark
}

/// Invalidate the appearance cache
///
/// Call this when the system appearance changes (e.g., from observe_window_appearance)
/// to force immediate re-detection on the next call to `detect_system_appearance()`.
pub fn invalidate_appearance_cache() {
    if let Some(cache) = APPEARANCE_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            // Set last_check to past the TTL so next call will re-detect
            guard.last_check = Instant::now() - APPEARANCE_CACHE_TTL - Duration::from_secs(1);
            debug!("Appearance cache invalidated");
        }
    }
}

/// Uncached system appearance detection (internal use)
fn detect_system_appearance_uncached() -> bool {
    // Default to dark mode if detection fails or we're not on macOS
    const DEFAULT_DARK: bool = true;

    // Try to detect macOS dark mode using system defaults
    match Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
    {
        Ok(output) => {
            // In light mode, the AppleInterfaceStyle key typically doesn't exist,
            // causing the command to exit with non-zero status
            if !output.status.success() {
                debug!(
                    appearance = "light",
                    "System appearance detected (key not present)"
                );
                return false; // light mode
            }

            // If the command succeeds and returns "Dark", we're in dark mode
            let stdout = String::from_utf8_lossy(&output.stdout);
            let is_dark = stdout.to_lowercase().contains("dark");
            debug!(
                appearance = if is_dark { "dark" } else { "light" },
                "System appearance detected"
            );
            is_dark
        }
        Err(e) => {
            // Command failed to execute (e.g., not on macOS, or `defaults` not found)
            debug!(
                error = %e,
                default = DEFAULT_DARK,
                "System appearance detection failed, using default"
            );
            DEFAULT_DARK
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
        let theme = if is_dark {
            Theme::dark_default()
        } else {
            Theme::light_default()
        };
        log_theme_config(&theme);
        return theme;
    }

    // Read and parse the JSON file
    match std::fs::read_to_string(&theme_path) {
        Err(e) => {
            error!(path = %theme_path.display(), error = %e, "Failed to read theme file, using defaults");
            let is_dark = detect_system_appearance();
            let theme = if is_dark {
                Theme::dark_default()
            } else {
                Theme::light_default()
            };
            log_theme_config(&theme);
            theme
        }
        Ok(contents) => match serde_json::from_str::<Theme>(&contents) {
            Ok(mut theme) => {
                debug!(path = %theme_path.display(), "Successfully loaded theme");

                // Key behavior: When appearance is Auto, use system appearance to
                // determine which color scheme to use (light or dark).
                // This allows the app to follow macOS light/dark mode automatically.
                let is_system_dark = detect_system_appearance();
                let should_use_light = match theme.appearance {
                    AppearanceMode::Light => true,
                    AppearanceMode::Dark => false,
                    AppearanceMode::Auto => !is_system_dark, // Follow system
                };

                if should_use_light {
                    // System is in light mode (or explicitly set to light)
                    // Use light color scheme, but preserve any non-color settings from theme.json
                    let light_colors = ColorScheme::light_default();
                    theme.colors = light_colors;
                    theme.appearance = AppearanceMode::Light; // Mark as light for consistency

                    // Use light opacity defaults
                    if theme.opacity.is_none() {
                        theme.opacity = Some(BackgroundOpacity::light_default());
                    }

                    debug!(
                        system_appearance = if is_system_dark { "dark" } else { "light" },
                        "Using light theme colors (system is in light mode)"
                    );
                } else {
                    // System is in dark mode (or explicitly set to dark)
                    // Use the colors from theme.json (which are dark)
                    if theme.opacity.is_none() {
                        theme.opacity = Some(BackgroundOpacity::dark_default());
                    }
                }

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
                let theme = if is_dark {
                    Theme::dark_default()
                } else {
                    Theme::light_default()
                };
                log_theme_config(&theme);
                theme
            }
        },
    }
}

/// Get a cached version of the theme for use in render functions
///
/// This avoids file I/O on every render call by caching the loaded theme.
/// The cache is automatically invalidated when `invalidate_theme_cache()` is called
/// (typically by the theme file watcher).
///
/// # Performance
///
/// Use this function instead of `load_theme()` in render paths:
/// - Render methods
/// - Background color calculations
/// - Any code that runs frequently
///
/// Use `load_theme()` for:
/// - Initial setup
/// - When you need guaranteed fresh theme data
/// - After explicitly invalidating the cache
pub fn get_cached_theme() -> Theme {
    let cache = THEME_CACHE.get_or_init(|| Mutex::new(ThemeCache::default()));

    let cache_guard = match cache.lock() {
        Ok(guard) => guard,
        Err(_) => {
            // Mutex poisoned, return default
            return Theme::dark_default();
        }
    };

    cache_guard.theme.clone()
}

/// Reload and cache the theme from disk
///
/// Call this when you need to refresh the cached theme (e.g., from the theme watcher).
/// This function loads the theme from disk and updates the cache.
pub fn reload_theme_cache() -> Theme {
    let theme = load_theme();

    let cache = THEME_CACHE.get_or_init(|| Mutex::new(ThemeCache::default()));
    if let Ok(mut guard) = cache.lock() {
        guard.theme = theme.clone();
        guard.loaded_at = Instant::now();
        debug!("Theme cache reloaded");
    }

    theme
}

/// Initialize the theme cache on startup
///
/// Call this during app initialization to ensure the theme is loaded
/// before any render calls. This ensures `get_cached_theme()` returns
/// the correct theme from the first render.
pub fn init_theme_cache() {
    reload_theme_cache();
    debug!("Theme cache initialized");
}

/// Invalidate the theme cache, forcing a reload on next access
///
/// Call this when the theme file changes to ensure the next call to
/// `get_cached_theme()` or `reload_theme_cache()` loads fresh data.
#[allow(dead_code)]
pub fn invalidate_theme_cache() {
    if let Some(cache) = THEME_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            // Force reload on next access by setting old timestamp
            guard.loaded_at = Instant::now() - Duration::from_secs(3600);
            debug!("Theme cache invalidated");
        }
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
