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

