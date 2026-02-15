//! gpui-component Theme Integration
//!
//! These functions sync Script Kit's theme with gpui-component's ThemeColor system.
//! Used by both main.rs and notes/window.rs for consistent theming.

use gpui::{hsla, rgb, App, Hsla};
use gpui_component::highlighter::{
    FontStyle as HighlightFontStyle, FontWeightContent, HighlightTheme, ThemeStyle,
};
use gpui_component::theme::{Theme as GpuiTheme, ThemeColor, ThemeMode};
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

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
/// # Arguments
/// * `sk_theme` - The Script Kit theme to map
/// * `is_dark` - Whether we're rendering in dark mode (affects base theme and tint alpha)
///
/// NOTE: We intentionally do NOT apply opacity.* values to theme colors here.
/// The opacity values are for window-level transparency (vibrancy effect),
/// not for making UI elements semi-transparent. UI elements should remain solid
/// so that text and icons are readable regardless of the vibrancy setting.
pub fn map_scriptkit_to_gpui_theme(sk_theme: &Theme, is_dark: bool) -> ThemeColor {
    let colors = &sk_theme.colors;
    let opacity = sk_theme.get_opacity();
    let vibrancy_enabled = sk_theme.is_vibrancy_enabled();

    debug!(
        is_dark,
        vibrancy_enabled,
        opacity_main = opacity.main,
        "map_scriptkit_to_gpui_theme entry"
    );

    // Get appropriate base theme based on appearance mode
    let mut theme_color = if is_dark {
        *ThemeColor::dark()
    } else {
        *ThemeColor::light()
    };

    // Helper to apply opacity to a color when vibrancy is enabled
    let with_vibrancy = |hex: u32, alpha: f32| -> Hsla {
        if vibrancy_enabled {
            let base = hex_to_hsla(hex);
            hsla(base.h, base.s, base.l, alpha)
        } else {
            hex_to_hsla(hex)
        }
    };

    // ╔════════════════════════════════════════════════════════════════════════════╗
    // ║ VIBRANCY BACKGROUND - CONSISTENT FOR ALL CONTENT IN WINDOW                 ║
    // ╠════════════════════════════════════════════════════════════════════════════╣
    // ║ gpui_component::Root applies .bg(theme.background) on ALL content.         ║
    // ║ This is the SINGLE SOURCE OF TRUTH for window background color.            ║
    // ║                                                                            ║
    // ║ For vibrancy: Use semi-transparent background that works with blur.        ║
    // ║ Opacity is now controlled via theme.opacity.vibrancy_background.           ║
    // ║ - Lower opacity = more blur visible                                        ║
    // ║ - Higher opacity = more solid color                                        ║
    // ╚════════════════════════════════════════════════════════════════════════════╝
    let main_bg = if vibrancy_enabled {
        // Get opacity from theme, with fallbacks for different modes
        // This controls how much blur shows through the window background
        // Fallback value (0.85) matches the vibrancy POC (src/bin/vibrancy-poc.rs):
        // - POC uses rgba(0xFAFAFAD9) = #FAFAFA at 85% opacity (0xD9/255 = 0.851)
        //
        // IMPORTANT: Light mode requires higher opacity for readability.
        // User theme.json may have low dark mode values (e.g., 0.3) that would
        // make light mode backgrounds too transparent. We enforce a minimum
        // of 0.85 for light mode to ensure text remains readable.
        let bg_alpha = if is_dark {
            // Dark mode: use user's value or default
            opacity.vibrancy_background.unwrap_or(0.85)
        } else {
            // Light mode: ensure minimum 0.85 opacity for visibility
            // User's value is clamped to at least 0.85 for light mode
            opacity
                .vibrancy_background
                .map(|v| v.max(0.85))
                .unwrap_or(0.85)
        };

        debug!(
            is_dark,
            vibrancy_enabled,
            opacity_main = opacity.main,
            vibrancy_background_config = ?opacity.vibrancy_background,
            resolved_background_alpha = bg_alpha,
            "map_scriptkit_to_gpui_theme vibrancy alpha resolved"
        );

        debug!(
            root_background_alpha = bg_alpha,
            vibrancy_enabled,
            is_dark,
            "Root background alpha resolved"
        );

        let base = hex_to_hsla(colors.background.main);
        hsla(base.h, base.s, base.l, bg_alpha)
    } else {
        hex_to_hsla(colors.background.main) // Fully opaque when vibrancy disabled
    };

    theme_color.background = main_bg;
    theme_color.foreground = hex_to_hsla(colors.text.primary);

    // Accent colors (Script Kit yellow/gold) - keep opaque for visibility
    theme_color.accent = hex_to_hsla(colors.accent.selected);
    theme_color.accent_foreground = hex_to_hsla(colors.text.primary);

    // Border - keep opaque
    theme_color.border = hex_to_hsla(colors.ui.border);
    theme_color.input = with_vibrancy(colors.ui.border, opacity.search_box);

    // List/sidebar colors - TRANSPARENT when vibrancy enabled to prevent stacking
    theme_color.list = main_bg; // transparent when vibrancy enabled
    theme_color.list_active = hex_to_hsla(colors.accent.selected_subtle); // Keep selection visible
    theme_color.list_active_border = hex_to_hsla(colors.accent.selected);
    theme_color.list_hover = hex_to_hsla(colors.accent.selected_subtle); // Keep hover visible
    theme_color.list_even = main_bg; // transparent when vibrancy enabled
    theme_color.list_head = main_bg; // transparent when vibrancy enabled

    // Sidebar - transparent when vibrancy enabled
    theme_color.sidebar = main_bg;
    theme_color.sidebar_foreground = hex_to_hsla(colors.text.primary);
    theme_color.sidebar_border = hex_to_hsla(colors.ui.border);
    theme_color.sidebar_accent = hex_to_hsla(colors.accent.selected_subtle);
    theme_color.sidebar_accent_foreground = hex_to_hsla(colors.text.primary);
    theme_color.sidebar_primary = hex_to_hsla(colors.accent.selected);
    theme_color.sidebar_primary_foreground = hex_to_hsla(colors.text.primary);

    // Primary (accent-colored buttons) - keep opaque for visibility
    theme_color.primary = hex_to_hsla(colors.accent.selected);
    theme_color.primary_foreground = hex_to_hsla(colors.background.main);
    theme_color.primary_hover = hex_to_hsla(colors.accent.selected);
    theme_color.primary_active = hex_to_hsla(colors.accent.selected);

    // Secondary (muted buttons) - TRANSPARENT when vibrancy enabled
    theme_color.secondary = if vibrancy_enabled {
        hsla(0.0, 0.0, 0.0, 0.0)
    } else {
        with_vibrancy(colors.background.search_box, 0.15)
    };
    theme_color.secondary_foreground = hex_to_hsla(colors.text.primary);
    theme_color.secondary_hover = if vibrancy_enabled {
        // Very subtle hover effect
        hsla(0.0, 0.0, if is_dark { 1.0 } else { 0.0 }, 0.05)
    } else {
        with_vibrancy(colors.background.title_bar, 0.2)
    };
    theme_color.secondary_active = if vibrancy_enabled {
        hsla(0.0, 0.0, if is_dark { 1.0 } else { 0.0 }, 0.1)
    } else {
        with_vibrancy(colors.background.title_bar, 0.25)
    };

    // Muted (disabled states, subtle elements) - transparent when vibrancy
    theme_color.muted = if vibrancy_enabled {
        hsla(0.0, 0.0, 0.0, 0.0)
    } else {
        with_vibrancy(colors.background.search_box, 0.1)
    };
    theme_color.muted_foreground = hex_to_hsla(colors.text.muted);

    // Title bar - transparent when vibrancy enabled
    theme_color.title_bar = main_bg;
    theme_color.title_bar_border = hex_to_hsla(colors.ui.border);

    // Popover - transparent when vibrancy enabled
    theme_color.popover = main_bg;
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

    // Scrollbar - track is transparent so it blends with any background
    theme_color.scrollbar = hsla(0.0, 0.0, 0.0, 0.0);
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
        is_dark,
        vibrancy_enabled,
        opacity_main = opacity.main,
        mapped_background_h = theme_color.background.h,
        mapped_background_s = theme_color.background.s,
        mapped_background_l = theme_color.background.l,
        mapped_background_a = theme_color.background.a,
        mapped_foreground_h = theme_color.foreground.h,
        mapped_foreground_s = theme_color.foreground.s,
        mapped_foreground_l = theme_color.foreground.l,
        mapped_foreground_a = theme_color.foreground.a,
        mapped_list_a = theme_color.list.a,
        mapped_sidebar_a = theme_color.sidebar.a,
        mapped_input_a = theme_color.input.a,
        mapped_secondary_a = theme_color.secondary.a,
        mapped_selection_a = theme_color.selection.a,
        "map_scriptkit_to_gpui_theme exit"
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

    // Determine if we're in dark mode based on SYSTEM appearance (not theme colors)
    // This ensures correct rendering when user switches between light/dark mode in macOS
    let is_dark = sk_theme.is_dark_mode();

    // Map Script Kit colors to gpui-component ThemeColor with appearance awareness
    let custom_colors = map_scriptkit_to_gpui_theme(&sk_theme, is_dark);

    // Get font configuration
    let fonts = sk_theme.get_fonts();

    // Apply the custom colors and fonts to the global theme
    let theme = GpuiTheme::global_mut(cx);
    theme.colors = custom_colors;
    // Set ThemeMode based on system appearance
    theme.mode = if is_dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    theme.highlight_theme = Arc::new(build_markdown_highlight_theme(&sk_theme, is_dark));

    // Debug: Log the background color to verify vibrancy is applied
    debug!(
        background_h = custom_colors.background.h,
        background_s = custom_colors.background.s,
        background_l = custom_colors.background.l,
        background_alpha = custom_colors.background.a,
        vibrancy_enabled = sk_theme.is_vibrancy_enabled(),
        opacity_main = sk_theme.get_opacity().main,
        is_dark = is_dark,
        "Theme background HSLA set"
    );

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

    debug!("gpui-component theme synchronized with Script Kit");
}

fn theme_style(
    color: Option<u32>,
    weight: Option<FontWeightContent>,
    style: Option<HighlightFontStyle>,
) -> ThemeStyle {
    let mut map = serde_json::Map::new();
    if let Some(hex) = color {
        map.insert("color".to_string(), json!(format!("#{:06x}", hex)));
    }
    if let Some(weight) = weight {
        map.insert("font_weight".to_string(), json!(weight));
    }
    if let Some(style) = style {
        map.insert("font_style".to_string(), json!(style));
    }
    serde_json::from_value(serde_json::Value::Object(map))
        .expect("ThemeStyle should deserialize from json map")
}

pub(crate) fn build_markdown_highlight_theme(sk_theme: &Theme, is_dark: bool) -> HighlightTheme {
    let mut highlight_theme = if is_dark {
        (*HighlightTheme::default_dark()).clone()
    } else {
        (*HighlightTheme::default_light()).clone()
    };

    let colors = &sk_theme.colors;
    let accent = colors.accent.selected;
    let secondary = colors.text.secondary;
    let muted = colors.text.muted;

    highlight_theme.appearance = if is_dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };

    highlight_theme.style.syntax.title = Some(theme_style(
        Some(accent),
        Some(FontWeightContent::Bold),
        None,
    ));
    highlight_theme.style.syntax.emphasis =
        Some(theme_style(None, None, Some(HighlightFontStyle::Italic)));
    highlight_theme.style.syntax.emphasis_strong =
        Some(theme_style(None, Some(FontWeightContent::Bold), None));
    highlight_theme.style.syntax.text_literal = Some(theme_style(Some(secondary), None, None));
    highlight_theme.style.syntax.link_text = Some(theme_style(
        Some(accent),
        Some(FontWeightContent::Medium),
        None,
    ));
    highlight_theme.style.syntax.link_uri = Some(theme_style(
        Some(accent),
        None,
        Some(HighlightFontStyle::Italic),
    ));
    highlight_theme.style.syntax.punctuation_list_marker = Some(theme_style(
        Some(accent),
        Some(FontWeightContent::Bold),
        None,
    ));
    highlight_theme.style.syntax.punctuation_special = Some(theme_style(Some(muted), None, None));
    highlight_theme
}
