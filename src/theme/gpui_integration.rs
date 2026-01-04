//! gpui-component Theme Integration
//!
//! These functions sync Script Kit's theme with gpui-component's ThemeColor system.
//! Used by both main.rs and notes/window.rs for consistent theming.

use gpui::{rgb, App, Hsla};
use gpui_component::theme::{Theme as GpuiTheme, ThemeColor, ThemeMode};
use tracing::{debug, info as tracing_info};

use super::types::{load_theme, Theme};

/// Convert a u32 hex color to Hsla
#[inline]
pub fn hex_to_hsla(hex: u32) -> Hsla {
    rgb(hex).into()
}

/// Map Script Kit's ColorScheme to gpui-component's ThemeColor
///
/// This function takes our Script Kit theme and maps all colors to the
/// gpui-component ThemeColor system, enabling consistent styling across
/// all gpui-component widgets (buttons, inputs, lists, etc.)
///
/// NOTE: We intentionally do NOT apply opacity.* values to theme colors here.
/// The opacity values are for window-level transparency (vibrancy effect),
/// not for making UI elements semi-transparent. UI elements should remain solid
/// so that text and icons are readable regardless of the vibrancy setting.
pub fn map_scriptkit_to_gpui_theme(sk_theme: &Theme) -> ThemeColor {
    let colors = &sk_theme.colors;
    // NOTE: opacity is NOT used here - it's only for window background vibrancy

    // Get default dark theme as base and override with Script Kit colors
    let mut theme_color = *ThemeColor::dark();

    // Main background and foreground
    theme_color.background = hex_to_hsla(colors.background.main);
    theme_color.foreground = hex_to_hsla(colors.text.primary);

    // Accent colors (Script Kit yellow/gold)
    theme_color.accent = hex_to_hsla(colors.accent.selected);
    theme_color.accent_foreground = hex_to_hsla(colors.text.primary);

    // Border
    theme_color.border = hex_to_hsla(colors.ui.border);
    theme_color.input = hex_to_hsla(colors.ui.border);

    // List/sidebar colors
    theme_color.list = hex_to_hsla(colors.background.main);
    theme_color.list_active = hex_to_hsla(colors.accent.selected_subtle);
    theme_color.list_active_border = hex_to_hsla(colors.accent.selected);
    theme_color.list_hover = hex_to_hsla(colors.accent.selected_subtle);
    theme_color.list_even = hex_to_hsla(colors.background.main);
    theme_color.list_head = hex_to_hsla(colors.background.title_bar);

    // Sidebar (use slightly lighter background)
    theme_color.sidebar = hex_to_hsla(colors.background.title_bar);
    theme_color.sidebar_foreground = hex_to_hsla(colors.text.primary);
    theme_color.sidebar_border = hex_to_hsla(colors.ui.border);
    theme_color.sidebar_accent = hex_to_hsla(colors.accent.selected_subtle);
    theme_color.sidebar_accent_foreground = hex_to_hsla(colors.text.primary);
    theme_color.sidebar_primary = hex_to_hsla(colors.accent.selected);
    theme_color.sidebar_primary_foreground = hex_to_hsla(colors.text.primary);

    // Primary (accent-colored buttons)
    theme_color.primary = hex_to_hsla(colors.accent.selected);
    theme_color.primary_foreground = hex_to_hsla(colors.background.main);
    theme_color.primary_hover = hex_to_hsla(colors.accent.selected);
    theme_color.primary_active = hex_to_hsla(colors.accent.selected);

    // Secondary (muted buttons)
    theme_color.secondary = hex_to_hsla(colors.background.search_box);
    theme_color.secondary_foreground = hex_to_hsla(colors.text.primary);
    theme_color.secondary_hover = hex_to_hsla(colors.background.title_bar);
    theme_color.secondary_active = hex_to_hsla(colors.background.title_bar);

    // Muted (disabled states, subtle elements)
    theme_color.muted = hex_to_hsla(colors.background.search_box);
    theme_color.muted_foreground = hex_to_hsla(colors.text.muted);

    // Title bar
    theme_color.title_bar = hex_to_hsla(colors.background.title_bar);
    theme_color.title_bar_border = hex_to_hsla(colors.ui.border);

    // Popover
    theme_color.popover = hex_to_hsla(colors.background.main);
    theme_color.popover_foreground = hex_to_hsla(colors.text.primary);

    // Status colors
    theme_color.success = hex_to_hsla(colors.ui.success);
    theme_color.success_foreground = hex_to_hsla(colors.text.primary);
    theme_color.danger = hex_to_hsla(colors.ui.error);
    theme_color.danger_foreground = hex_to_hsla(colors.text.primary);
    theme_color.warning = hex_to_hsla(colors.ui.warning);
    theme_color.warning_foreground = hex_to_hsla(colors.text.primary);
    theme_color.info = hex_to_hsla(colors.ui.info);
    theme_color.info_foreground = hex_to_hsla(colors.text.primary);

    // Scrollbar
    theme_color.scrollbar = hex_to_hsla(colors.background.main);
    theme_color.scrollbar_thumb = hex_to_hsla(colors.text.dimmed);
    theme_color.scrollbar_thumb_hover = hex_to_hsla(colors.text.muted);

    // Caret (cursor) - match main input text color
    theme_color.caret = hex_to_hsla(colors.text.primary);

    // Selection - match main input selection alpha (0x60)
    let mut selection = hex_to_hsla(colors.accent.selected);
    selection.a = 96.0 / 255.0;
    theme_color.selection = selection;

    // Ring (focus ring)
    theme_color.ring = hex_to_hsla(colors.accent.selected);

    // Tab colors
    theme_color.tab = hex_to_hsla(colors.background.main);
    theme_color.tab_active = hex_to_hsla(colors.background.search_box);
    theme_color.tab_active_foreground = hex_to_hsla(colors.text.primary);
    theme_color.tab_foreground = hex_to_hsla(colors.text.secondary);
    theme_color.tab_bar = hex_to_hsla(colors.background.title_bar);

    debug!(
        background = format!("#{:06x}", colors.background.main),
        accent = format!("#{:06x}", colors.accent.selected),
        "Script Kit theme mapped to gpui-component"
    );

    theme_color
}

/// Sync Script Kit theme with gpui-component's global Theme
///
/// This function loads the Script Kit theme and applies it to gpui-component's
/// global Theme, ensuring all gpui-component widgets use our colors.
///
/// Call this:
/// 1. After `gpui_component::init(cx)` in main.rs
/// 2. When system appearance changes (light/dark mode)
/// 3. When theme.json is reloaded
pub fn sync_gpui_component_theme(cx: &mut App) {
    // Load Script Kit's theme
    let sk_theme = load_theme();

    // Map Script Kit colors to gpui-component ThemeColor
    let custom_colors = map_scriptkit_to_gpui_theme(&sk_theme);

    // Get font configuration
    let fonts = sk_theme.get_fonts();

    // Apply the custom colors and fonts to the global theme
    let theme = GpuiTheme::global_mut(cx);
    theme.colors = custom_colors;
    theme.mode = ThemeMode::Dark; // Script Kit uses dark mode by default

    // Set monospace font for code editor (used by InputState in code_editor mode)
    theme.mono_font_family = fonts.mono_family.clone().into();
    theme.mono_font_size = gpui::px(fonts.mono_size);

    // Set UI font
    theme.font_family = fonts.ui_family.clone().into();
    theme.font_size = gpui::px(fonts.ui_size);

    debug!(
        mono_font = fonts.mono_family,
        mono_size = fonts.mono_size,
        ui_font = fonts.ui_family,
        ui_size = fonts.ui_size,
        "Font configuration applied to gpui-component"
    );

    tracing_info!("gpui-component theme synchronized with Script Kit");
}
