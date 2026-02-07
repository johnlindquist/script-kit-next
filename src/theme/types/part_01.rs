use serde::{Deserialize, Serialize};
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
    /// Selected list item background opacity (default: 0.33)
    #[serde(default = "default_selected_opacity")]
    pub selected: f32,
    /// Hovered list item background opacity (default: 0.22)
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
    0.33 // Selection with vibrancy — increased for clearer active-item contrast
}

fn default_hover_opacity() -> f32 {
    0.22 // Hover feedback — increased for clearer pointer/selection tracking
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
            selected: 0.33, // Selected list item highlight — improved visibility on vibrancy
            hover: 0.22,    // Hovered list item highlight — clearer state affordance
            preview: 0.0,   // Preview panel (0 = fully transparent)
            dialog: 0.15,   // Dialogs/popups - very low opacity, let vibrancy blur show through
            input: 0.30,    // Input fields
            panel: 0.20,    // Panels/containers
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
            selected: 0.36, // Light mode selection - tuned for stronger non-text contrast
            hover: 0.26,    // Light mode hover - visible on bright vibrancy backgrounds
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

