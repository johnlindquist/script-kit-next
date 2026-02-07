use super::*;

impl ThemeAdapter {
    /// Creates a theme adapter from a Script Kit theme.
    ///
    /// Maps theme colors to terminal colors:
    /// - `theme.colors.text.primary` → foreground
    /// - `theme.colors.background.main` → background
    /// - `theme.colors.accent.selected` → cursor
    /// - `theme.colors.accent.selected_subtle` → selection background
    /// - `theme.colors.text.secondary` → selection foreground
    /// - `theme.colors.terminal.*` → ANSI color palette (all 16 colors)
    ///
    /// All 16 ANSI colors are now fully themeable via `theme.colors.terminal`.
    pub fn from_theme(theme: &Theme) -> Self {
        let colors = &theme.colors;

        let foreground = hex_to_rgb(colors.text.primary);
        let background = hex_to_rgb(colors.background.main);
        let cursor = hex_to_rgb(colors.accent.selected);
        let selection_background = hex_to_rgb(colors.accent.selected_subtle);
        let selection_foreground = hex_to_rgb(colors.text.secondary);

        let terminal = &colors.terminal;
        let ansi = AnsiColors {
            black: hex_to_rgb(terminal.black),
            red: hex_to_rgb(terminal.red),
            green: hex_to_rgb(terminal.green),
            yellow: hex_to_rgb(terminal.yellow),
            blue: hex_to_rgb(terminal.blue),
            magenta: hex_to_rgb(terminal.magenta),
            cyan: hex_to_rgb(terminal.cyan),
            white: hex_to_rgb(terminal.white),
            bright_black: hex_to_rgb(terminal.bright_black),
            bright_red: hex_to_rgb(terminal.bright_red),
            bright_green: hex_to_rgb(terminal.bright_green),
            bright_yellow: hex_to_rgb(terminal.bright_yellow),
            bright_blue: hex_to_rgb(terminal.bright_blue),
            bright_magenta: hex_to_rgb(terminal.bright_magenta),
            bright_cyan: hex_to_rgb(terminal.bright_cyan),
            bright_white: hex_to_rgb(terminal.bright_white),
        };

        Self {
            foreground,
            background,
            cursor,
            selection_background,
            selection_foreground,
            ansi,
            is_focused: true,
            original_foreground: foreground,
            original_background: background,
            original_cursor: cursor,
            original_selection_background: selection_background,
            original_selection_foreground: selection_foreground,
            original_ansi: ansi,
        }
    }

    /// Creates a theme adapter with sensible dark defaults.
    ///
    /// Uses colors that work well with most dark themes:
    /// - Background: #1e1e1e (VS Code dark)
    /// - Foreground: #d4d4d4 (Light gray)
    /// - Cursor: #ffffff (White)
    pub fn dark_default() -> Self {
        let foreground = hex_to_rgb(0xd4d4d4);
        let background = hex_to_rgb(0x1e1e1e);
        let cursor = hex_to_rgb(0xffffff);
        let selection_background = hex_to_rgb(0x264f78);
        let selection_foreground = hex_to_rgb(0xffffff);
        let ansi = AnsiColors::default();

        Self {
            foreground,
            background,
            cursor,
            selection_background,
            selection_foreground,
            ansi,
            is_focused: true,
            original_foreground: foreground,
            original_background: background,
            original_cursor: cursor,
            original_selection_background: selection_background,
            original_selection_foreground: selection_foreground,
            original_ansi: ansi,
        }
    }

    /// Creates a theme adapter with sensible light defaults.
    ///
    /// Uses colors that work well with light themes:
    /// - Background: #f5f5f5 (Light gray for terminal panel)
    /// - Foreground: #000000 (Black text for maximum contrast)
    /// - Cursor: #000000 (Black cursor visible on light background)
    pub fn light_default() -> Self {
        let foreground = hex_to_rgb(0x000000);
        let background = hex_to_rgb(0xf5f5f5);
        let cursor = hex_to_rgb(0x000000);
        let selection_background = hex_to_rgb(0x0078d4);
        let selection_foreground = hex_to_rgb(0xffffff);
        let ansi = AnsiColors::light_default();

        Self {
            foreground,
            background,
            cursor,
            selection_background,
            selection_foreground,
            ansi,
            is_focused: true,
            original_foreground: foreground,
            original_background: background,
            original_cursor: cursor,
            original_selection_background: selection_background,
            original_selection_foreground: selection_foreground,
            original_ansi: ansi,
        }
    }

    /// Returns the foreground text color.
    #[inline]
    pub fn foreground(&self) -> Rgb {
        self.foreground
    }

    /// Returns the background color.
    #[inline]
    pub fn background(&self) -> Rgb {
        self.background
    }

    /// Returns the cursor color.
    #[inline]
    pub fn cursor(&self) -> Rgb {
        self.cursor
    }

    /// Returns the selection background color.
    #[inline]
    pub fn selection_background(&self) -> Rgb {
        self.selection_background
    }

    /// Returns the selection foreground color.
    #[inline]
    pub fn selection_foreground(&self) -> Rgb {
        self.selection_foreground
    }

    /// Returns an ANSI color by index (0-15).
    #[inline]
    pub fn ansi_color(&self, index: u8) -> Rgb {
        self.ansi.get(index)
    }

    /// Returns whether the adapter is in focused state.
    #[inline]
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Updates colors based on window focus state.
    pub fn update_for_focus(&mut self, is_focused: bool) {
        if self.is_focused == is_focused {
            return;
        }

        self.is_focused = is_focused;

        if is_focused {
            self.foreground = self.original_foreground;
            self.background = self.original_background;
            self.cursor = self.original_cursor;
            self.selection_background = self.original_selection_background;
            self.selection_foreground = self.original_selection_foreground;
            self.ansi = self.original_ansi;
        } else {
            const DIM_FACTOR: f32 = 0.7;

            self.foreground = dim_color(self.original_foreground, DIM_FACTOR);
            self.background = dim_color(self.original_background, DIM_FACTOR);
            self.cursor = dim_color(self.original_cursor, DIM_FACTOR);
            self.selection_background = dim_color(self.original_selection_background, DIM_FACTOR);
            self.selection_foreground = dim_color(self.original_selection_foreground, DIM_FACTOR);
            self.ansi = self.original_ansi.dimmed(DIM_FACTOR);
        }
    }

    /// Updates the theme adapter from a new Theme.
    ///
    /// This allows updating terminal colors when the theme changes at runtime.
    /// Preserves the current focus state (if unfocused, colors will be dimmed).
    pub fn update_from_theme(&mut self, theme: &Theme) {
        let new_adapter = Self::from_theme(theme);
        let was_focused = self.is_focused;

        self.original_foreground = new_adapter.original_foreground;
        self.original_background = new_adapter.original_background;
        self.original_cursor = new_adapter.original_cursor;
        self.original_selection_background = new_adapter.original_selection_background;
        self.original_selection_foreground = new_adapter.original_selection_foreground;
        self.original_ansi = new_adapter.original_ansi;

        if was_focused {
            self.foreground = new_adapter.foreground;
            self.background = new_adapter.background;
            self.cursor = new_adapter.cursor;
            self.selection_background = new_adapter.selection_background;
            self.selection_foreground = new_adapter.selection_foreground;
            self.ansi = new_adapter.ansi;
        } else {
            const DIM_FACTOR: f32 = 0.7;
            self.foreground = dim_color(self.original_foreground, DIM_FACTOR);
            self.background = dim_color(self.original_background, DIM_FACTOR);
            self.cursor = dim_color(self.original_cursor, DIM_FACTOR);
            self.selection_background = dim_color(self.original_selection_background, DIM_FACTOR);
            self.selection_foreground = dim_color(self.original_selection_foreground, DIM_FACTOR);
            self.ansi = self.original_ansi.dimmed(DIM_FACTOR);
        }
    }
}

impl Default for ThemeAdapter {
    fn default() -> Self {
        Self::dark_default()
    }
}
