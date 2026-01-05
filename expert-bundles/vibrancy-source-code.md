# Vibrancy Source Code Reference

## platform.rs - NSVisualEffectView Configuration

```rust
/// NSVisualEffectMaterial values
pub mod ns_visual_effect_material {
    pub const TITLEBAR: isize = 3;
    pub const SELECTION: isize = 4;        // GPUI default (colorless)
    pub const MENU: isize = 5;
    pub const POPOVER: isize = 6;
    pub const SIDEBAR: isize = 7;
    pub const HEADER_VIEW: isize = 10;
    pub const SHEET: isize = 11;
    pub const WINDOW_BACKGROUND: isize = 12;
    pub const HUD_WINDOW: isize = 13;      // Dark, high contrast
    pub const FULL_SCREEN_UI: isize = 15;
    pub const TOOL_TIP: isize = 17;
    pub const CONTENT_BACKGROUND: isize = 18;
    pub const UNDER_WINDOW_BACKGROUND: isize = 21;
    pub const UNDER_PAGE_BACKGROUND: isize = 22;
    pub const DARK: isize = 2;             // Deprecated
    pub const MEDIUM_DARK: isize = 8;      // Undocumented
    pub const ULTRA_DARK: isize = 9;       // Undocumented
}

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    static NSAppearanceNameDarkAqua: id;
    static NSAppearanceNameAqua: id;
    static NSAppearanceNameVibrantDark: id;
    static NSAppearanceNameVibrantLight: id;
}

pub fn configure_window_vibrancy_material() {
    unsafe {
        let window = window_manager::get_main_window()?;

        // Set window appearance to DarkAqua
        let dark_appearance: id = msg_send![
            class!(NSAppearance),
            appearanceNamed: NSAppearanceNameDarkAqua
        ];
        let _: () = msg_send![window, setAppearance: dark_appearance];

        // Find NSVisualEffectView
        let content_view: id = msg_send![window, contentView];
        let subviews: id = msg_send![content_view, subviews];
        let count: usize = msg_send![subviews, count];

        for i in 0..count {
            let subview: id = msg_send![subviews, objectAtIndex: i];
            let is_visual_effect_view: bool =
                msg_send![subview, isKindOfClass: class!(NSVisualEffectView)];

            if is_visual_effect_view {
                let _: () = msg_send![subview, setMaterial: SIDEBAR];
                let _: () = msg_send![subview, setState: 1isize]; // Active
                let _: () = msg_send![subview, setBlendingMode: 0isize]; // BehindWindow
                let _: () = msg_send![subview, setEmphasized: true];
                return;
            }
        }
    }
}
```

## theme/types.rs - Vibrancy Settings

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VibrancyMaterial {
    Hud,
    #[default]
    Popover,
    Menu,
    Sidebar,
    Content,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrancySettings {
    pub enabled: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundOpacity {
    pub main: f32,             // 0.30
    pub title_bar: f32,        // 0.30
    pub search_box: f32,       // 0.40
    pub log_panel: f32,        // 0.40
    pub selected: f32,         // 0.15
    pub hover: f32,            // 0.08
    pub preview: f32,          // 0.0
    pub dialog: f32,           // 0.35
    pub input: f32,            // 0.30
    pub panel: f32,            // 0.20
    pub input_inactive: f32,   // 0.25
    pub input_active: f32,     // 0.50
    pub border_inactive: f32,  // 0.125
    pub border_active: f32,    // 0.25
}
```

## theme/gpui_integration.rs - Theme Mapping

```rust
pub fn map_scriptkit_to_gpui_theme(sk_theme: &Theme) -> ThemeColor {
    let colors = &sk_theme.colors;
    let opacity = sk_theme.get_opacity();
    let vibrancy_enabled = sk_theme.is_vibrancy_enabled();

    let mut theme_color = *ThemeColor::dark();

    let with_vibrancy = |hex: u32, alpha: f32| -> Hsla {
        if vibrancy_enabled {
            let base = hex_to_hsla(hex);
            hsla(base.h, base.s, base.l, alpha)
        } else {
            hex_to_hsla(hex)
        }
    };

    // Main background FULLY TRANSPARENT for vibrancy
    let main_bg = if vibrancy_enabled {
        hsla(0.0, 0.0, 0.0, 0.0)
    } else {
        hex_to_hsla(colors.background.main)
    };

    theme_color.background = main_bg.clone();
    theme_color.list = main_bg.clone();
    theme_color.sidebar = main_bg.clone();
    theme_color.title_bar = main_bg.clone();
    theme_color.popover = main_bg.clone();

    // Text stays opaque
    theme_color.foreground = hex_to_hsla(colors.text.primary);
    theme_color.accent = hex_to_hsla(colors.accent.selected);

    // Secondary elements partial opacity
    theme_color.secondary = with_vibrancy(colors.background.search_box, 0.15);
    theme_color.muted = with_vibrancy(colors.background.search_box, 0.1);

    theme_color
}
```

## main.rs - Window Creation

```rust
// Load theme to determine window background appearance
let initial_theme = theme::load_theme();
let window_background = if initial_theme.is_vibrancy_enabled() {
    WindowBackgroundAppearance::Blurred
} else {
    WindowBackgroundAppearance::Opaque
};

// After window creation
platform::configure_as_floating_panel();
platform::configure_window_vibrancy_material();
```

## actions/window.rs - Popup Vibrancy

```rust
pub fn open_actions_window(...) -> anyhow::Result<WindowHandle<Root>> {
    let theme = theme::load_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    let window_options = WindowOptions {
        window_background,
        focus: false,
        kind: WindowKind::PopUp,
        ..Default::default()
    };
}

pub unsafe fn configure_actions_popup_window(window: id) {
    let _: () = msg_send![window, setMovable: false];
    let _: () = msg_send![window, setHidesOnDeactivate: true];
    let _: () = msg_send![window, setHasShadow: false];
    let _: () = msg_send![window, setAnimationBehavior: 2i64];
}
```

## notes/window.rs - Vibrancy Helpers

```rust
fn get_vibrancy_background(_cx: &Context<Self>) -> gpui::Rgba {
    let sk_theme = crate::theme::load_theme();
    let opacity = sk_theme.get_opacity();
    let bg_hex = sk_theme.colors.background.main;
    rgba(Self::hex_to_rgba_with_opacity(bg_hex, opacity.main))
}

fn hex_to_rgba_with_opacity(hex: u32, opacity: f32) -> u32 {
    let alpha = (opacity * 255.0) as u32;
    (hex << 8) | alpha
}
```
