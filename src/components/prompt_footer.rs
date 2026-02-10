//! PromptFooter - Reusable footer component for prompts
//!
//! This module provides a theme-aware footer component used across all prompt types.
//! It includes a Script Kit logo, optional helper text, primary action button, divider,
//! and secondary action button.
//!
//! # Structure
//! ```text
//! | [Logo] [Helper Text] |              | [Info] | [Primary ↵] | [Secondary ⌘K] |
//! ```
//!
//! # Example
//! ```rust,ignore
//! let footer = PromptFooter::new(
//!     PromptFooterConfig::new()
//!         .primary_label("Run Script")
//!         .primary_shortcut("↵")
//!         .secondary_label("Actions")
//!         .secondary_shortcut("⌘K"),
//!     PromptFooterColors::from_theme(&theme),
//! )
//! .on_primary_click(Box::new(|_, _, _| { /* handle run */ }))
//! .on_secondary_click(Box::new(|_, _, _| { /* handle actions */ }));
//! ```

#![allow(dead_code)]

use gpui::*;
use std::rc::Rc;

use crate::designs::DesignColors;
use crate::theme::Theme;
use crate::ui_foundation::{hstack, HexColorExt};
use crate::utils;
use crate::window_resize::layout::FOOTER_HEIGHT;

/// Helper text width cap to preserve room for footer actions.
pub const PROMPT_FOOTER_HELPER_TEXT_MAX_WIDTH_PX: f32 = 420.0;
/// Info label width cap so long labels do not crowd footer actions.
pub const PROMPT_FOOTER_INFO_TEXT_MAX_WIDTH_PX: f32 = 220.0;
/// Shared horizontal spacing between footer sections.
const PROMPT_FOOTER_SECTION_GAP_PX: f32 = 8.0;
/// Shared horizontal spacing between footer buttons/divider.
const PROMPT_FOOTER_BUTTON_GAP_PX: f32 = 4.0;
/// Hover opacity for footer action buttons.
const PROMPT_FOOTER_BUTTON_HOVER_OPACITY: u8 = 0x26;
/// Active/pressed opacity for footer action buttons.
const PROMPT_FOOTER_BUTTON_ACTIVE_OPACITY: u8 = 0x3a;
/// Footer button font size delta from base UI font size.
const PROMPT_FOOTER_BUTTON_FONT_DELTA_PX: f32 = 2.0;
/// Minimum footer button font size.
const PROMPT_FOOTER_BUTTON_FONT_MIN_PX: f32 = 10.0;
/// Footer horizontal padding.
const PROMPT_FOOTER_PADDING_X_PX: f32 = 14.0;
/// Optical bottom padding to align footer content vertically.
const PROMPT_FOOTER_PADDING_BOTTOM_PX: f32 = 2.0;
/// Footer logo icon size.
const PROMPT_FOOTER_LOGO_SIZE_PX: f32 = 16.0;
/// Small optical nudge so the logo appears centered with adjacent text.
const PROMPT_FOOTER_LOGO_NUDGE_X_PX: f32 = 2.0;
/// Divider width between footer buttons.
const PROMPT_FOOTER_DIVIDER_WIDTH_PX: f32 = 1.0;
/// Divider height between footer buttons.
const PROMPT_FOOTER_DIVIDER_HEIGHT_PX: f32 = 16.0;
/// Divider horizontal margin between buttons.
const PROMPT_FOOTER_DIVIDER_MARGIN_X_PX: f32 = 4.0;
/// Footer top-border opacity for contrast on light/dark surfaces.
const PROMPT_FOOTER_BORDER_OPACITY: u8 = 0x50;
/// Footer shadow Y-offset.
const PROMPT_FOOTER_SHADOW_OFFSET_Y_PX: f32 = -1.0;
/// Footer shadow blur radius.
const PROMPT_FOOTER_SHADOW_BLUR_PX: f32 = 8.0;
/// Info label font size delta from base UI font size.
const PROMPT_FOOTER_INFO_FONT_DELTA_PX: f32 = 4.0;
/// Minimum info label font size.
const PROMPT_FOOTER_INFO_FONT_MIN_PX: f32 = 9.0;
/// Helper label font size delta from base UI font size.
const PROMPT_FOOTER_HELPER_FONT_DELTA_PX: f32 = 2.0;
/// Minimum helper label font size.
const PROMPT_FOOTER_HELPER_FONT_MIN_PX: f32 = 10.0;

/// Pre-computed colors for PromptFooter rendering
///
/// This struct holds the primitive color values needed for footer rendering,
/// allowing efficient use in closures without cloning the full theme.
#[derive(Clone, Copy, Debug)]
pub struct PromptFooterColors {
    /// Accent color for logo background and primary text
    pub accent: u32,
    /// Muted text color for shortcut hints
    pub text_muted: u32,
    /// Border color for top border and divider
    pub border: u32,
    /// Background color for footer surface
    pub background: u32,
    /// Whether we're in light mode (affects opacity)
    pub is_light_mode: bool,
}

impl PromptFooterColors {
    /// Create PromptFooterColors from theme reference
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            accent: theme.colors.accent.selected,
            text_muted: theme.colors.text.muted,
            border: theme.colors.ui.border,
            // Match selected item surface token for footer consistency.
            background: theme.colors.accent.selected_subtle,
            is_light_mode: !theme.is_dark_mode(),
        }
    }

    /// Create PromptFooterColors from design colors.
    ///
    /// Prompt footers should stay visually aligned with the app shell, so
    /// this path intentionally resolves to the cached theme tokens.
    pub fn from_design(_colors: &DesignColors) -> Self {
        Self::from_theme(&crate::theme::get_cached_theme())
    }
}

impl Default for PromptFooterColors {
    fn default() -> Self {
        Self::from_theme(&crate::theme::get_cached_theme())
    }
}

/// Resolve footer surface color with mode-specific opacity.
pub fn footer_surface_rgba(colors: PromptFooterColors) -> u32 {
    if colors.is_light_mode {
        // Neutral warm gray for light mode — blocks vibrancy so footer text stays legible.
        0xf0eeefff
    } else {
        // Dark mode: use selected_subtle as a subtle overlay (~12% opacity).
        (colors.background << 8) | 0x1f
    }
}

fn footer_shadow_alpha(colors: PromptFooterColors) -> u8 {
    if colors.is_light_mode {
        0x28
    } else {
        0x50
    }
}

fn is_footer_button_clickable(has_click_handler: bool, disabled: bool) -> bool {
    has_click_handler && !disabled
}

fn is_footer_button_activation_key(key: &str) -> bool {
    matches!(
        key,
        "enter" | "return" | "Enter" | "Return" | " " | "space" | "Space"
    )
}

pub fn footer_button_hover_rgba(colors: PromptFooterColors) -> u32 {
    (colors.background << 8) | (PROMPT_FOOTER_BUTTON_HOVER_OPACITY as u32)
}

pub fn footer_button_active_rgba(colors: PromptFooterColors) -> u32 {
    (colors.background << 8) | (PROMPT_FOOTER_BUTTON_ACTIVE_OPACITY as u32)
}

pub fn footer_button_font_size_px(theme: &Theme) -> f32 {
    (theme.get_fonts().ui_size - PROMPT_FOOTER_BUTTON_FONT_DELTA_PX)
        .max(PROMPT_FOOTER_BUTTON_FONT_MIN_PX)
}

/// Configuration for PromptFooter display
#[derive(Clone, Debug)]
pub struct PromptFooterConfig {
    /// Label for the primary button (e.g., "Run Script", "Submit", "Paste")
    pub primary_label: String,
    /// Shortcut hint for primary button (e.g., "↵", "⌘+Enter")
    pub primary_shortcut: String,
    /// Label for the secondary button (e.g., "Actions")
    pub secondary_label: String,
    /// Shortcut hint for secondary button (e.g., "⌘K")
    pub secondary_shortcut: String,
    /// Whether to show the Script Kit logo
    pub show_logo: bool,
    /// Whether to show the secondary button
    pub show_secondary: bool,
    /// Disable interactions on the primary button
    pub primary_disabled: bool,
    /// Disable interactions on the secondary button
    pub secondary_disabled: bool,
    /// Optional helper text shown next to logo (e.g., "Tab 1 of 2 · Tab to continue")
    pub helper_text: Option<String>,
    /// Optional info label shown before buttons (e.g., "typescript", "5 items")
    pub info_label: Option<String>,
}

impl Default for PromptFooterConfig {
    fn default() -> Self {
        Self {
            primary_label: "Run Script".to_string(),
            primary_shortcut: "↵".to_string(),
            secondary_label: "Actions".to_string(),
            secondary_shortcut: "⌘K".to_string(),
            show_logo: true,
            show_secondary: true,
            primary_disabled: false,
            secondary_disabled: false,
            helper_text: None,
            info_label: None,
        }
    }
}

impl PromptFooterConfig {
    /// Create a new default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the primary button label
    pub fn primary_label(mut self, label: impl Into<String>) -> Self {
        self.primary_label = label.into();
        self
    }

    /// Set the primary button shortcut hint
    pub fn primary_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.primary_shortcut = shortcut.into();
        self
    }

    /// Set the secondary button label
    pub fn secondary_label(mut self, label: impl Into<String>) -> Self {
        self.secondary_label = label.into();
        self
    }

    /// Set the secondary button shortcut hint
    pub fn secondary_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.secondary_shortcut = shortcut.into();
        self
    }

    /// Set whether to show the logo
    pub fn show_logo(mut self, show: bool) -> Self {
        self.show_logo = show;
        self
    }

    /// Set whether to show the secondary button
    pub fn show_secondary(mut self, show: bool) -> Self {
        self.show_secondary = show;
        self
    }

    /// Set whether the primary button is disabled
    pub fn primary_disabled(mut self, disabled: bool) -> Self {
        self.primary_disabled = disabled;
        self
    }

    /// Set whether the secondary button is disabled
    pub fn secondary_disabled(mut self, disabled: bool) -> Self {
        self.secondary_disabled = disabled;
        self
    }

    /// Set optional helper text shown next to the logo
    pub fn helper_text(mut self, text: impl Into<String>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// Set optional info label shown before buttons (e.g., language indicator)
    pub fn info_label(mut self, label: impl Into<String>) -> Self {
        self.info_label = Some(label.into());
        self
    }
}

/// Callback type for button click events
pub type FooterClickCallback = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A reusable footer component for prompts
///
/// Displays:
/// - Script Kit logo (left, optional)
/// - Primary action button with shortcut
/// - Divider (optional, when secondary visible)
/// - Secondary action button with shortcut (optional)
///
/// Height: 40px fixed
#[derive(IntoElement)]
pub struct PromptFooter {
    config: PromptFooterConfig,
    colors: PromptFooterColors,
    on_primary_click: Option<Rc<FooterClickCallback>>,
    on_secondary_click: Option<Rc<FooterClickCallback>>,
}

impl PromptFooter {
    /// Create a new PromptFooter with the given configuration and colors
    pub fn new(config: PromptFooterConfig, colors: PromptFooterColors) -> Self {
        Self {
            config,
            colors,
            on_primary_click: None,
            on_secondary_click: None,
        }
    }

    /// Set the primary button click callback
    pub fn on_primary_click(mut self, callback: FooterClickCallback) -> Self {
        self.on_primary_click = Some(Rc::new(callback));
        self
    }

    /// Set the secondary button click callback
    pub fn on_secondary_click(mut self, callback: FooterClickCallback) -> Self {
        self.on_secondary_click = Some(Rc::new(callback));
        self
    }

    /// Render the Script Kit logo (accent-colored icon, no background)
    fn render_logo(&self) -> impl IntoElement {
        svg()
            .external_path(utils::get_logo_path())
            .size(px(PROMPT_FOOTER_LOGO_SIZE_PX))
            .ml(px(PROMPT_FOOTER_LOGO_NUDGE_X_PX))
            .text_color(rgb(self.colors.accent)) // Accent color (yellow/gold)
    }

    /// Render a footer button with label and shortcut
    fn render_button(
        &self,
        id: &'static str,
        label: String,
        shortcut: String,
        disabled: bool,
        on_click: Option<Rc<FooterClickCallback>>,
    ) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let button_font_size = footer_button_font_size_px(&theme);
        let has_click_handler = on_click.is_some();
        let is_clickable = is_footer_button_clickable(has_click_handler, disabled);
        let on_click_for_key = on_click.clone();
        let hover_bg = rgba(footer_button_hover_rgba(self.colors));
        let active_bg = rgba(footer_button_active_rgba(self.colors));
        let shortcut_bg = self.colors.border.rgba8(0x20);
        let shortcut_border = self.colors.border.rgba8(0x40);

        let mut button = div()
            .id(ElementId::Name(id.into()))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .px(px(8.))
            .py(px(2.))
            .rounded(px(4.))
            .cursor_default()
            .child(
                div()
                    .text_size(px(button_font_size))
                    .text_color(self.colors.accent.to_rgb())
                    .child(label),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .px(px(6.0))
                    .py(px(1.0))
                    .rounded(px(4.0))
                    .bg(shortcut_bg)
                    .border_1()
                    .border_color(shortcut_border)
                    .text_size(px(button_font_size))
                    .font_family(crate::list_item::FONT_MONO)
                    .text_color(self.colors.text_muted.to_rgb())
                    .child(shortcut),
            );

        if is_clickable {
            button = button
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .active(move |s| s.bg(active_bg));
        } else if disabled {
            button = button.opacity(0.5).cursor_default();
        }

        if is_clickable {
            if let Some(callback) = on_click {
                button = button.on_click(move |event, window, cx| {
                    callback(event, window, cx);
                });
            }

            if let Some(callback) = on_click_for_key {
                button = button.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if is_footer_button_activation_key(key) {
                        let click_event = ClickEvent::default();
                        callback(&click_event, window, cx);
                    }
                });
            }
        }

        button
    }

    /// Render the vertical divider between buttons
    fn render_divider(&self) -> impl IntoElement {
        div()
            .w(px(PROMPT_FOOTER_DIVIDER_WIDTH_PX))
            .h(px(PROMPT_FOOTER_DIVIDER_HEIGHT_PX))
            .mx(px(PROMPT_FOOTER_DIVIDER_MARGIN_X_PX))
            .bg(self.colors.border.rgba8(0x40)) // 25% opacity
    }
}

impl RenderOnce for PromptFooter {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colors = self.colors;
        let theme = crate::theme::get_cached_theme();
        let ui_font_size = theme.get_fonts().ui_size;
        let info_font_size =
            (ui_font_size - PROMPT_FOOTER_INFO_FONT_DELTA_PX).max(PROMPT_FOOTER_INFO_FONT_MIN_PX);
        let helper_font_size = (ui_font_size - PROMPT_FOOTER_HELPER_FONT_DELTA_PX)
            .max(PROMPT_FOOTER_HELPER_FONT_MIN_PX);

        // Build the right-side container (info label + buttons)
        let mut right_side = hstack()
            .gap(px(PROMPT_FOOTER_SECTION_GAP_PX))
            .items_center()
            .min_w(px(0.));

        // Info label (e.g., "typescript", "5 items") - shown before buttons
        if let Some(ref info) = self.config.info_label {
            right_side = right_side.child(
                div()
                    .max_w(px(PROMPT_FOOTER_INFO_TEXT_MAX_WIDTH_PX))
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .text_size(px(info_font_size))
                    .text_color(colors.text_muted.to_rgb())
                    .child(info.clone()),
            );
        }

        // Build the buttons container
        let mut buttons = hstack().gap(px(PROMPT_FOOTER_BUTTON_GAP_PX)).items_center();

        // Primary button
        buttons = buttons.child(self.render_button(
            "footer-primary-button",
            self.config.primary_label.clone(),
            self.config.primary_shortcut.clone(),
            self.config.primary_disabled,
            self.on_primary_click.clone(),
        ));

        // Divider + Secondary button (if enabled)
        if self.config.show_secondary {
            buttons = buttons.child(self.render_divider());
            buttons = buttons.child(self.render_button(
                "footer-secondary-button",
                self.config.secondary_label.clone(),
                self.config.secondary_shortcut.clone(),
                self.config.secondary_disabled,
                self.on_secondary_click.clone(),
            ));
        }

        right_side = right_side.child(buttons);

        // Main footer container (uses FOOTER_HEIGHT constant for single source of truth)
        // Resolve from PromptFooterColors.background so color ownership stays within footer tokens.
        let footer_bg = rgba(footer_surface_rgba(colors));

        let mut footer = div()
            .w_full()
            .h(px(FOOTER_HEIGHT))
            .min_h(px(FOOTER_HEIGHT))
            .max_h(px(FOOTER_HEIGHT))
            .flex_shrink_0()
            .overflow_hidden()
            .px(px(PROMPT_FOOTER_PADDING_X_PX))
            .pt(px(0.))
            .pb(px(PROMPT_FOOTER_PADDING_BOTTOM_PX))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .border_t_1()
            .border_color(colors.border.rgba8(PROMPT_FOOTER_BORDER_OPACITY))
            .bg(footer_bg)
            // Inner shadow above the footer for visual separation from content
            // Footers are the ONE scenario where blocking vibrancy is OK
            .shadow(vec![BoxShadow {
                color: colors.border.rgba8(footer_shadow_alpha(colors)),
                offset: point(px(0.), px(PROMPT_FOOTER_SHADOW_OFFSET_Y_PX)),
                blur_radius: px(PROMPT_FOOTER_SHADOW_BLUR_PX),
                spread_radius: px(0.),
            }]);

        // Left side: Logo + helper text
        let mut left_side = hstack()
            .flex_1()
            .min_w(px(0.))
            .overflow_hidden()
            .gap(px(PROMPT_FOOTER_SECTION_GAP_PX))
            .items_center();

        // Logo (if enabled)
        if self.config.show_logo {
            left_side = left_side.child(self.render_logo());
        }

        // Helper text (e.g., "Tab 1 of 2 · Tab to continue, Esc to exit")
        if let Some(ref helper) = self.config.helper_text {
            left_side = left_side.child(
                div()
                    .max_w(px(PROMPT_FOOTER_HELPER_TEXT_MAX_WIDTH_PX))
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .text_size(px(helper_font_size))
                    .text_color(colors.accent.to_rgb())
                    .child(helper.clone()),
            );
        }

        footer = footer.child(left_side);

        // Right: Info label + Buttons
        footer.child(right_side)
    }
}

// Note: Tests omitted for this module due to GPUI macro recursion limit issues.
// The PromptFooter component is integration-tested via the main application's
// prompt rendering in main.rs.
//
// Verified traits:
// - PromptFooterColors: Copy, Clone, Debug, Default
// - PromptFooterConfig: Clone, Debug, Default + builder pattern
// - PromptFooter: builder pattern with .on_primary_click(), .on_secondary_click()

#[cfg(test)]
mod tests {
    use super::{
        footer_surface_rgba, PromptFooterColors, PROMPT_FOOTER_BORDER_OPACITY,
        PROMPT_FOOTER_BUTTON_ACTIVE_OPACITY, PROMPT_FOOTER_BUTTON_FONT_DELTA_PX,
        PROMPT_FOOTER_BUTTON_FONT_MIN_PX, PROMPT_FOOTER_BUTTON_GAP_PX,
        PROMPT_FOOTER_BUTTON_HOVER_OPACITY, PROMPT_FOOTER_DIVIDER_HEIGHT_PX,
        PROMPT_FOOTER_DIVIDER_MARGIN_X_PX, PROMPT_FOOTER_DIVIDER_WIDTH_PX,
        PROMPT_FOOTER_HELPER_FONT_DELTA_PX, PROMPT_FOOTER_HELPER_FONT_MIN_PX,
        PROMPT_FOOTER_INFO_FONT_DELTA_PX, PROMPT_FOOTER_INFO_FONT_MIN_PX,
        PROMPT_FOOTER_LOGO_NUDGE_X_PX, PROMPT_FOOTER_LOGO_SIZE_PX, PROMPT_FOOTER_PADDING_BOTTOM_PX,
        PROMPT_FOOTER_PADDING_X_PX, PROMPT_FOOTER_SECTION_GAP_PX, PROMPT_FOOTER_SHADOW_BLUR_PX,
        PROMPT_FOOTER_SHADOW_OFFSET_Y_PX,
    };

    #[test]
    fn test_footer_surface_rgba_returns_neutral_gray_in_light_mode() {
        let colors = PromptFooterColors {
            accent: 0,
            text_muted: 0,
            border: 0,
            background: 0x2255aa,
            is_light_mode: true,
        };

        // Light mode always returns the neutral warm gray, regardless of background token.
        assert_eq!(footer_surface_rgba(colors), 0xf0eeefff);
    }

    #[test]
    fn test_footer_surface_rgba_uses_background_overlay_in_dark_mode() {
        let colors = PromptFooterColors {
            accent: 0,
            text_muted: 0,
            border: 0,
            background: 0x2255aa,
            is_light_mode: false,
        };

        // Dark mode uses background token at ~12% opacity.
        assert_eq!(footer_surface_rgba(colors), 0x2255aa1f);
    }

    #[test]
    fn test_prompt_footer_colors_from_design_uses_cached_theme_tokens() {
        let design = crate::designs::DesignColors {
            accent: 0x010203,
            text_muted: 0x040506,
            border: 0x070809,
            background_selected: 0x0a0b0c,
            ..crate::designs::DesignColors::default()
        };

        let resolved = PromptFooterColors::from_design(&design);
        let expected = PromptFooterColors::from_theme(&crate::theme::get_cached_theme());

        assert_eq!(resolved.accent, expected.accent);
        assert_eq!(resolved.text_muted, expected.text_muted);
        assert_eq!(resolved.border, expected.border);
        assert_eq!(resolved.background, expected.background);
        assert_eq!(resolved.is_light_mode, expected.is_light_mode);
    }

    #[test]
    fn test_prompt_footer_colors_default_uses_cached_theme_tokens() {
        let resolved = PromptFooterColors::default();
        let expected = PromptFooterColors::from_theme(&crate::theme::get_cached_theme());

        assert_eq!(resolved.accent, expected.accent);
        assert_eq!(resolved.text_muted, expected.text_muted);
        assert_eq!(resolved.border, expected.border);
        assert_eq!(resolved.background, expected.background);
        assert_eq!(resolved.is_light_mode, expected.is_light_mode);
    }

    #[test]
    fn test_footer_shadow_alpha_uses_higher_alpha_in_dark_mode() {
        let light = PromptFooterColors {
            accent: 0,
            text_muted: 0,
            border: 0,
            background: 0,
            is_light_mode: true,
        };
        let dark = PromptFooterColors {
            is_light_mode: false,
            ..light
        };

        assert_eq!(super::footer_shadow_alpha(light), 0x28);
        assert_eq!(super::footer_shadow_alpha(dark), 0x50);
    }

    #[test]
    fn test_footer_button_hover_rgba_uses_background_token_with_standard_opacity() {
        let colors = PromptFooterColors {
            accent: 0,
            text_muted: 0,
            border: 0,
            background: 0x2255aa,
            is_light_mode: false,
        };

        assert_eq!(super::footer_button_hover_rgba(colors), 0x2255aa26);
    }

    #[test]
    fn test_footer_button_active_rgba_uses_background_token_with_pressed_opacity() {
        let colors = PromptFooterColors {
            accent: 0,
            text_muted: 0,
            border: 0,
            background: 0x2255aa,
            is_light_mode: false,
        };

        assert_eq!(super::footer_button_active_rgba(colors), 0x2255aa3a);
    }

    #[test]
    fn test_is_footer_button_clickable_requires_handler_and_enabled_state() {
        assert!(super::is_footer_button_clickable(true, false));
        assert!(!super::is_footer_button_clickable(false, false));
        assert!(!super::is_footer_button_clickable(true, true));
    }

    #[test]
    fn test_is_footer_button_activation_key_accepts_enter_and_space_variants() {
        assert!(super::is_footer_button_activation_key("enter"));
        assert!(super::is_footer_button_activation_key("Enter"));
        assert!(super::is_footer_button_activation_key("return"));
        assert!(super::is_footer_button_activation_key("Space"));
        assert!(super::is_footer_button_activation_key(" "));
        assert!(!super::is_footer_button_activation_key("tab"));
    }

    #[test]
    fn test_prompt_footer_layout_tokens_stay_consistent_when_spacing_is_adjusted() {
        assert_eq!(PROMPT_FOOTER_SECTION_GAP_PX, 8.0);
        assert_eq!(PROMPT_FOOTER_BUTTON_GAP_PX, 4.0);
        assert_eq!(PROMPT_FOOTER_BUTTON_HOVER_OPACITY, 0x26);
        assert_eq!(PROMPT_FOOTER_BUTTON_ACTIVE_OPACITY, 0x3a);
        assert_eq!(PROMPT_FOOTER_BUTTON_FONT_DELTA_PX, 2.0);
        assert_eq!(PROMPT_FOOTER_BUTTON_FONT_MIN_PX, 10.0);
        assert_eq!(PROMPT_FOOTER_PADDING_X_PX, 14.0);
        assert_eq!(PROMPT_FOOTER_PADDING_BOTTOM_PX, 2.0);
        assert_eq!(PROMPT_FOOTER_LOGO_SIZE_PX, 16.0);
        assert_eq!(PROMPT_FOOTER_LOGO_NUDGE_X_PX, 2.0);
        assert_eq!(PROMPT_FOOTER_DIVIDER_WIDTH_PX, 1.0);
        assert_eq!(PROMPT_FOOTER_DIVIDER_HEIGHT_PX, 16.0);
        assert_eq!(PROMPT_FOOTER_DIVIDER_MARGIN_X_PX, 4.0);
        assert_eq!(PROMPT_FOOTER_BORDER_OPACITY, 0x50);
        assert_eq!(PROMPT_FOOTER_SHADOW_OFFSET_Y_PX, -1.0);
        assert_eq!(PROMPT_FOOTER_SHADOW_BLUR_PX, 8.0);
        assert_eq!(PROMPT_FOOTER_INFO_FONT_DELTA_PX, 4.0);
        assert_eq!(PROMPT_FOOTER_INFO_FONT_MIN_PX, 9.0);
        assert_eq!(PROMPT_FOOTER_HELPER_FONT_DELTA_PX, 2.0);
        assert_eq!(PROMPT_FOOTER_HELPER_FONT_MIN_PX, 10.0);
    }
}
