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
use tracing::{debug, warn};

use super::types::{get_cached_theme, relative_luminance_srgb, Theme};

const SECONDARY_VIBRANCY_ALPHA: f32 = 0.08;
const SECONDARY_VIBRANCY_HOVER_ALPHA: f32 = 0.14;
const SECONDARY_VIBRANCY_ACTIVE_ALPHA: f32 = 0.22;
const MUTED_VIBRANCY_ALPHA: f32 = 0.06;

/// Convert a u32 hex color to Hsla
#[inline]
pub fn hex_to_hsla(hex: u32) -> Hsla {
    rgb(hex).into()
}

#[inline]
fn shift_lightness(hex: u32, delta: f32) -> Hsla {
    let mut color = hex_to_hsla(hex);
    color.l = (color.l + delta).clamp(0.0, 1.0);
    color
}

#[inline]
fn primary_interaction_color(hex: u32, is_dark: bool, amount: f32) -> Hsla {
    let delta = if is_dark { amount } else { -amount };
    shift_lightness(hex, delta)
}

#[inline]
fn subtle_overlay(hex: u32, alpha: f32) -> Hsla {
    let base = hex_to_hsla(hex);
    hsla(base.h, base.s, base.l, alpha.clamp(0.0, 1.0))
}

#[inline]
fn blend_hex_over_background(foreground: u32, background: u32, alpha: f32) -> u32 {
    let alpha = alpha.clamp(0.0, 1.0);

    let blend_channel = |fg: u8, bg: u8| -> u8 {
        let fg = f32::from(fg);
        let bg = f32::from(bg);
        (bg * (1.0 - alpha) + fg * alpha).round() as u8
    };

    let fg_r = ((foreground >> 16) & 0xff) as u8;
    let fg_g = ((foreground >> 8) & 0xff) as u8;
    let fg_b = (foreground & 0xff) as u8;
    let bg_r = ((background >> 16) & 0xff) as u8;
    let bg_g = ((background >> 8) & 0xff) as u8;
    let bg_b = (background & 0xff) as u8;

    let out_r = blend_channel(fg_r, bg_r);
    let out_g = blend_channel(fg_g, bg_g);
    let out_b = blend_channel(fg_b, bg_b);

    (u32::from(out_r) << 16) | (u32::from(out_g) << 8) | u32::from(out_b)
}

#[inline]
fn contrast_ratio(a: u32, b: u32) -> f32 {
    let luminance_a = relative_luminance_srgb(a);
    let luminance_b = relative_luminance_srgb(b);
    let brighter = luminance_a.max(luminance_b);
    let darker = luminance_a.min(luminance_b);
    (brighter + 0.05) / (darker + 0.05)
}

#[inline]
pub(crate) fn best_contrast_of_two(background: u32, option_a: u32, option_b: u32) -> u32 {
    let contrast_a = contrast_ratio(background, option_a);
    let contrast_b = contrast_ratio(background, option_b);
    if contrast_a >= contrast_b {
        option_a
    } else {
        option_b
    }
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
    let colors = sk_theme.get_colors(true);
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

    // Helper to apply opacity: true alpha when vibrancy is enabled,
    // or precomposed opaque color against background when disabled.
    let with_vibrancy = |hex: u32, alpha: f32| -> Hsla {
        let alpha = alpha.clamp(0.0, 1.0);
        if vibrancy_enabled {
            let base = hex_to_hsla(hex);
            hsla(base.h, base.s, base.l, alpha)
        } else {
            hex_to_hsla(blend_hex_over_background(
                hex,
                colors.background.main,
                alpha,
            ))
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
        }
        .clamp(0.0, 1.0);

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
            vibrancy_enabled, is_dark, "Root background alpha resolved"
        );

        let base = hex_to_hsla(colors.background.main);
        hsla(base.h, base.s, base.l, bg_alpha)
    } else {
        hex_to_hsla(colors.background.main) // Fully opaque when vibrancy disabled
    };

    theme_color.background = main_bg;
    theme_color.foreground = hex_to_hsla(colors.text.primary);

    // Accent colors (Script Kit yellow/gold) - keep opaque for visibility
    let accent_foreground_hex = best_contrast_of_two(
        colors.accent.selected,
        colors.text.on_accent,
        colors.background.main,
    );
    theme_color.accent = hex_to_hsla(colors.accent.selected);
    theme_color.accent_foreground = hex_to_hsla(accent_foreground_hex);

    // Border - keep opaque
    theme_color.border = hex_to_hsla(colors.ui.border);
    theme_color.input = with_vibrancy(colors.background.search_box, opacity.search_box);

    // List/sidebar colors - TRANSPARENT when vibrancy enabled to prevent stacking
    theme_color.list = main_bg; // transparent when vibrancy enabled
    theme_color.list_active = subtle_overlay(colors.accent.selected_subtle, opacity.selected);
    theme_color.list_active_border = hex_to_hsla(colors.accent.selected);
    theme_color.list_hover = subtle_overlay(colors.accent.selected_subtle, opacity.hover);
    theme_color.list_even = main_bg; // transparent when vibrancy enabled
    theme_color.list_head = main_bg; // transparent when vibrancy enabled

    // Sidebar - transparent when vibrancy enabled
    theme_color.sidebar = main_bg;
    theme_color.sidebar_foreground = hex_to_hsla(colors.text.primary);
    theme_color.sidebar_border = hex_to_hsla(colors.ui.border);
    theme_color.sidebar_accent = subtle_overlay(colors.accent.selected_subtle, opacity.selected);
    theme_color.sidebar_accent_foreground = hex_to_hsla(colors.text.primary);
    theme_color.sidebar_primary = hex_to_hsla(colors.accent.selected);
    theme_color.sidebar_primary_foreground = hex_to_hsla(accent_foreground_hex);

    // Primary (accent-colored buttons) - keep opaque for visibility
    theme_color.primary = hex_to_hsla(colors.accent.selected);
    theme_color.primary_foreground = hex_to_hsla(accent_foreground_hex);
    theme_color.primary_hover = primary_interaction_color(colors.accent.selected, is_dark, 0.06);
    theme_color.primary_active = primary_interaction_color(colors.accent.selected, is_dark, 0.12);

    // Secondary (muted buttons) - TRANSPARENT when vibrancy enabled
    theme_color.secondary = if vibrancy_enabled {
        subtle_overlay(colors.accent.selected_subtle, SECONDARY_VIBRANCY_ALPHA)
    } else {
        with_vibrancy(colors.background.search_box, 0.15)
    };
    theme_color.secondary_foreground = hex_to_hsla(colors.text.primary);
    theme_color.secondary_hover = if vibrancy_enabled {
        subtle_overlay(
            colors.accent.selected_subtle,
            SECONDARY_VIBRANCY_HOVER_ALPHA,
        )
    } else {
        with_vibrancy(colors.background.title_bar, 0.2)
    };
    theme_color.secondary_active = if vibrancy_enabled {
        subtle_overlay(
            colors.accent.selected_subtle,
            SECONDARY_VIBRANCY_ACTIVE_ALPHA,
        )
    } else {
        with_vibrancy(colors.background.title_bar, 0.25)
    };

    // Muted (disabled states, subtle elements) - transparent when vibrancy
    theme_color.muted = if vibrancy_enabled {
        subtle_overlay(colors.accent.selected_subtle, MUTED_VIBRANCY_ALPHA)
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
    let status_foreground = |status_background: u32| {
        best_contrast_of_two(
            status_background,
            colors.text.primary,
            colors.background.main,
        )
    };
    theme_color.success = hex_to_hsla(colors.ui.success);
    theme_color.success_foreground = hex_to_hsla(status_foreground(colors.ui.success));
    theme_color.danger = hex_to_hsla(colors.ui.error);
    theme_color.danger_foreground = hex_to_hsla(status_foreground(colors.ui.error));
    theme_color.warning = hex_to_hsla(colors.ui.warning);
    theme_color.warning_foreground = hex_to_hsla(status_foreground(colors.ui.warning));
    theme_color.info = hex_to_hsla(colors.ui.info);
    theme_color.info_foreground = hex_to_hsla(status_foreground(colors.ui.info));

    // Scrollbar - track is transparent so it blends with any background
    theme_color.scrollbar = hsla(0.0, 0.0, 0.0, 0.0);
    theme_color.scrollbar_thumb = hex_to_hsla(colors.text.dimmed);
    theme_color.scrollbar_thumb_hover = hex_to_hsla(colors.text.muted);

    // Caret (cursor) - prefer explicit focused cursor override when configured
    let has_focused_cursor_override = sk_theme
        .focus_aware
        .as_ref()
        .and_then(|focus_aware| focus_aware.focused.as_ref())
        .and_then(|focused| focused.cursor.as_ref())
        .is_some();
    let caret_color = if has_focused_cursor_override {
        sk_theme
            .get_cursor_style(true)
            .map(|cursor| cursor.color)
            .unwrap_or(colors.text.primary)
    } else {
        colors.text.primary
    };
    theme_color.caret = hex_to_hsla(caret_color);

    // Selection - match main input selection alpha (0x60)
    let mut selection = hex_to_hsla(colors.accent.selected);
    selection.a = (96.0_f32 / 255.0_f32).clamp(0.0, 1.0);
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

#[inline]
fn resolve_mode_and_colors(sk_theme: &Theme) -> (ThemeMode, ThemeColor) {
    let is_dark = sk_theme.is_dark_mode();
    let mode = if is_dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    let colors = map_scriptkit_to_gpui_theme(sk_theme, is_dark);
    (mode, colors)
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
    let sk_theme = get_cached_theme();

    sync_gpui_component_theme_for_theme(cx, &sk_theme);
}

/// Sync gpui-component theme from a specific Script Kit theme instance.
///
/// Use this when the active in-memory theme differs from the cached on-disk
/// theme (for example, while previewing presets in the theme chooser).
pub(crate) fn sync_gpui_component_theme_for_theme(cx: &mut App, sk_theme: &Theme) {
    let (mode, custom_colors) = resolve_mode_and_colors(sk_theme);
    let is_dark = matches!(mode, ThemeMode::Dark);

    debug!(
        appearance = ?sk_theme.appearance,
        has_dark_colors = sk_theme.has_dark_colors(),
        "sync_gpui_component_theme_for_theme start"
    );

    // Get font configuration
    let fonts = sk_theme.get_fonts();

    // Apply the custom colors and fonts to the global theme
    let theme = GpuiTheme::global_mut(cx);
    theme.colors = custom_colors;
    theme.mode = mode;
    theme.highlight_theme = Arc::new(build_markdown_highlight_theme(sk_theme, is_dark));

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
    fallback: Option<ThemeStyle>,
) -> Option<ThemeStyle> {
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
    match serde_json::from_value(serde_json::Value::Object(map)) {
        Ok(style) => Some(style),
        Err(e) => {
            warn!(error = %e, "ThemeStyle json deserialize failed; falling back");
            fallback
        }
    }
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

    highlight_theme.style.syntax.title = theme_style(
        Some(accent),
        Some(FontWeightContent::Bold),
        None,
        highlight_theme.style.syntax.title,
    );
    highlight_theme.style.syntax.emphasis = theme_style(
        None,
        None,
        Some(HighlightFontStyle::Italic),
        highlight_theme.style.syntax.emphasis,
    );
    highlight_theme.style.syntax.emphasis_strong = theme_style(
        None,
        Some(FontWeightContent::Bold),
        None,
        highlight_theme.style.syntax.emphasis_strong,
    );
    highlight_theme.style.syntax.text_literal = theme_style(
        Some(secondary),
        None,
        None,
        highlight_theme.style.syntax.text_literal,
    );
    highlight_theme.style.syntax.link_text = theme_style(
        Some(accent),
        Some(FontWeightContent::Medium),
        None,
        highlight_theme.style.syntax.link_text,
    );
    highlight_theme.style.syntax.link_uri = theme_style(
        Some(accent),
        None,
        Some(HighlightFontStyle::Italic),
        highlight_theme.style.syntax.link_uri,
    );
    highlight_theme.style.syntax.punctuation_list_marker = theme_style(
        Some(accent),
        Some(FontWeightContent::Bold),
        None,
        highlight_theme.style.syntax.punctuation_list_marker,
    );
    highlight_theme.style.syntax.punctuation_special = theme_style(
        Some(muted),
        None,
        None,
        highlight_theme.style.syntax.punctuation_special,
    );
    highlight_theme
}

#[cfg(test)]
mod tests {
    use super::super::types::{
        BackgroundOpacity, CursorStyle, FocusAwareColorScheme, FocusColorScheme,
    };
    use super::*;

    fn assert_hsla_close(actual: Hsla, expected: Hsla) {
        assert!((actual.h - expected.h).abs() < 1e-6);
        assert!((actual.s - expected.s).abs() < 1e-6);
        assert!((actual.l - expected.l).abs() < 1e-6);
        assert!((actual.a - expected.a).abs() < 1e-6);
    }

    fn focus_color_scheme_from(
        colors: &crate::theme::ColorScheme,
        cursor: Option<CursorStyle>,
    ) -> FocusColorScheme {
        FocusColorScheme {
            background: colors.background.clone(),
            text: colors.text.clone(),
            accent: colors.accent.clone(),
            ui: colors.ui.clone(),
            cursor,
            terminal: colors.terminal.clone(),
        }
    }

    #[test]
    fn test_shift_lightness_clamps_to_unit_interval() {
        let lighter = shift_lightness(0xffffff, 0.12);
        let darker = shift_lightness(0x000000, -0.12);

        assert!((lighter.l - 1.0).abs() < 1e-6);
        assert!((darker.l - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_map_scriptkit_to_gpui_theme_derives_primary_interaction_states_by_mode() {
        let dark_theme = Theme::dark_default();
        let dark_mapped = map_scriptkit_to_gpui_theme(&dark_theme, true);
        assert_hsla_close(
            dark_mapped.primary_hover,
            primary_interaction_color(dark_theme.colors.accent.selected, true, 0.06),
        );
        assert_hsla_close(
            dark_mapped.primary_active,
            primary_interaction_color(dark_theme.colors.accent.selected, true, 0.12),
        );

        let light_theme = Theme::light_default();
        let light_mapped = map_scriptkit_to_gpui_theme(&light_theme, false);
        assert_hsla_close(
            light_mapped.primary_hover,
            primary_interaction_color(light_theme.colors.accent.selected, false, 0.06),
        );
        assert_hsla_close(
            light_mapped.primary_active,
            primary_interaction_color(light_theme.colors.accent.selected, false, 0.12),
        );
    }

    #[test]
    fn test_resolve_mode_and_colors_uses_passed_theme_appearance() {
        let dark_theme = Theme::dark_default();
        let light_theme = Theme::light_default();

        let (dark_mode, dark_colors) = resolve_mode_and_colors(&dark_theme);
        let (light_mode, light_colors) = resolve_mode_and_colors(&light_theme);

        assert_eq!(dark_mode, ThemeMode::Dark);
        assert_eq!(light_mode, ThemeMode::Light);
        assert_hsla_close(
            dark_colors.background,
            map_scriptkit_to_gpui_theme(&dark_theme, true).background,
        );
        assert_hsla_close(
            light_colors.background,
            map_scriptkit_to_gpui_theme(&light_theme, false).background,
        );
    }

    #[test]
    fn test_map_scriptkit_to_gpui_theme_uses_best_contrast_for_accent_derived_foregrounds() {
        let mut theme = Theme::dark_default();
        theme.colors.accent.selected = 0xf5d86a;
        theme.colors.text.on_accent = 0xffffff;
        theme.colors.background.main = 0x111217;

        let mapped = map_scriptkit_to_gpui_theme(&theme, true);
        let expected = best_contrast_of_two(
            theme.colors.accent.selected,
            theme.colors.text.on_accent,
            theme.colors.background.main,
        );
        assert_hsla_close(mapped.accent_foreground, hex_to_hsla(expected));
        assert_hsla_close(mapped.primary_foreground, hex_to_hsla(expected));
        assert_hsla_close(mapped.sidebar_primary_foreground, hex_to_hsla(expected));
    }

    #[test]
    fn test_map_scriptkit_to_gpui_theme_uses_search_box_color_for_input_background() {
        let mut theme = Theme::dark_default();
        theme.colors.background.search_box = 0x1a2b3c;
        theme.colors.ui.border = 0xddeeff;
        theme.opacity = Some(BackgroundOpacity {
            search_box: 0.42,
            ..theme.get_opacity()
        });

        let mapped = map_scriptkit_to_gpui_theme(&theme, true);
        let search_box = hex_to_hsla(theme.colors.background.search_box);
        let expected = hsla(
            search_box.h,
            search_box.s,
            search_box.l,
            theme.get_opacity().search_box,
        );
        assert_hsla_close(mapped.input, expected);
    }

    #[test]
    fn test_map_scriptkit_to_gpui_theme_clamps_out_of_range_vibrancy_alphas() {
        let mut theme = Theme::dark_default();
        let mut vibrancy = theme.get_vibrancy();
        vibrancy.enabled = true;
        theme.vibrancy = Some(vibrancy);
        theme.opacity = Some(BackgroundOpacity {
            search_box: 1.7,
            vibrancy_background: Some(-0.4),
            ..theme.get_opacity()
        });

        let mapped = map_scriptkit_to_gpui_theme(&theme, true);

        assert!((mapped.input.a - 1.0).abs() < 1e-6);
        assert!((mapped.background.a - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_map_scriptkit_to_gpui_theme_applies_selected_and_hover_overlay_opacity() {
        let mut theme = Theme::dark_default();
        theme.colors.accent.selected_subtle = 0x335577;
        theme.opacity = Some(BackgroundOpacity {
            selected: 0.38,
            hover: 0.24,
            ..theme.get_opacity()
        });

        let mapped = map_scriptkit_to_gpui_theme(&theme, true);
        let expected_selected = subtle_overlay(
            theme.colors.accent.selected_subtle,
            theme.get_opacity().selected,
        );
        let expected_hover = subtle_overlay(
            theme.colors.accent.selected_subtle,
            theme.get_opacity().hover,
        );
        assert_hsla_close(mapped.list_active, expected_selected);
        assert_hsla_close(mapped.sidebar_accent, expected_selected);
        assert_hsla_close(mapped.list_hover, expected_hover);
    }

    #[test]
    fn test_map_scriptkit_to_gpui_theme_status_foregrounds_choose_highest_contrast_option() {
        let mut theme = Theme::dark_default();
        theme.colors.text.primary = 0xffffff;
        theme.colors.background.main = 0x000000;
        theme.colors.ui.success = 0xf5f5f5;
        theme.colors.ui.error = 0x101010;
        theme.colors.ui.warning = 0xf8d65a;
        theme.colors.ui.info = 0x1f3f7f;

        let mapped = map_scriptkit_to_gpui_theme(&theme, true);

        let success_expected = best_contrast_of_two(
            theme.colors.ui.success,
            theme.colors.text.primary,
            theme.colors.background.main,
        );
        let danger_expected = best_contrast_of_two(
            theme.colors.ui.error,
            theme.colors.text.primary,
            theme.colors.background.main,
        );
        let warning_expected = best_contrast_of_two(
            theme.colors.ui.warning,
            theme.colors.text.primary,
            theme.colors.background.main,
        );
        let info_expected = best_contrast_of_two(
            theme.colors.ui.info,
            theme.colors.text.primary,
            theme.colors.background.main,
        );

        assert_hsla_close(mapped.success_foreground, hex_to_hsla(success_expected));
        assert_hsla_close(mapped.danger_foreground, hex_to_hsla(danger_expected));
        assert_hsla_close(mapped.warning_foreground, hex_to_hsla(warning_expected));
        assert_hsla_close(mapped.info_foreground, hex_to_hsla(info_expected));
    }

    #[test]
    fn test_map_scriptkit_to_gpui_theme_vibrancy_secondary_and_muted_alpha_are_perceptible() {
        let dark_theme = Theme::dark_default();
        let dark_mapped = map_scriptkit_to_gpui_theme(&dark_theme, true);
        assert!(dark_mapped.secondary.a > 0.0);
        assert!(dark_mapped.secondary_hover.a > dark_mapped.secondary.a);
        assert!(dark_mapped.secondary_active.a > dark_mapped.secondary_hover.a);
        assert!(dark_mapped.muted.a > 0.0);

        let light_theme = Theme::light_default();
        let light_mapped = map_scriptkit_to_gpui_theme(&light_theme, false);
        assert!(light_mapped.secondary.a > 0.0);
        assert!(light_mapped.secondary_hover.a > light_mapped.secondary.a);
        assert!(light_mapped.secondary_active.a > light_mapped.secondary_hover.a);
        assert!(light_mapped.muted.a > 0.0);
    }

    #[test]
    fn test_map_scriptkit_to_gpui_theme_non_vibrancy_secondary_states_are_distinct() {
        let mut theme = Theme::dark_default();
        theme.colors.background.main = 0x000000;
        theme.colors.background.title_bar = 0xffffff;
        let mut vibrancy = theme.get_vibrancy();
        vibrancy.enabled = false;
        theme.vibrancy = Some(vibrancy);

        let mapped = map_scriptkit_to_gpui_theme(&theme, true);

        assert!((mapped.secondary_hover.a - 1.0).abs() < 1e-6);
        assert!((mapped.secondary_active.a - 1.0).abs() < 1e-6);
        assert!((mapped.secondary_hover.l - mapped.secondary_active.l).abs() > 1e-6);
    }

    #[test]
    fn test_map_scriptkit_to_gpui_theme_prefers_focus_aware_focused_colors() {
        let mut theme = Theme::dark_default();
        theme.colors.text.primary = 0x111111;
        theme.colors.accent.selected = 0x222222;

        let mut focused_colors = theme.colors.clone();
        focused_colors.text.primary = 0xaabbcc;
        focused_colors.accent.selected = 0x123456;
        theme.focus_aware = Some(FocusAwareColorScheme {
            focused: Some(focus_color_scheme_from(&focused_colors, None)),
            unfocused: None,
        });

        let mapped = map_scriptkit_to_gpui_theme(&theme, true);
        assert_hsla_close(mapped.foreground, hex_to_hsla(focused_colors.text.primary));
        assert_hsla_close(mapped.accent, hex_to_hsla(focused_colors.accent.selected));
    }

    #[test]
    fn test_map_scriptkit_to_gpui_theme_uses_focus_aware_cursor_override_for_caret() {
        let mut theme = Theme::dark_default();
        theme.colors.text.primary = 0x111111;

        let focused_colors = theme.colors.clone();
        let cursor_override = CursorStyle {
            color: 0x34c9ff,
            blink_interval_ms: 700,
        };
        theme.focus_aware = Some(FocusAwareColorScheme {
            focused: Some(focus_color_scheme_from(
                &focused_colors,
                Some(cursor_override.clone()),
            )),
            unfocused: None,
        });

        let mapped = map_scriptkit_to_gpui_theme(&theme, true);
        assert_hsla_close(mapped.caret, hex_to_hsla(cursor_override.color));
    }
}
