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

