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

/// Serialize a theme to JSON string for writing to disk
#[allow(dead_code)]
pub fn theme_to_json(theme: &Theme) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(theme)
}

