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

