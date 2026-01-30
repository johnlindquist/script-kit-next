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
    /// Background color for footer (matches selected item background)
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
            background: theme.colors.accent.selected_subtle, // Match selected item bg
            is_light_mode: !theme.is_dark_mode(),
        }
    }

    /// Create PromptFooterColors from design colors for design system support
    pub fn from_design(colors: &DesignColors) -> Self {
        Self {
            accent: colors.accent,
            text_muted: colors.text_muted,
            border: colors.border,
            background: colors.background_selected, // Match selected item bg
            is_light_mode: false,                   // Default to dark mode for design colors
        }
    }
}

impl Default for PromptFooterColors {
    fn default() -> Self {
        Self {
            accent: 0xfbbf24, // Script Kit yellow/gold
            text_muted: 0x808080,
            border: 0x464647,
            background: 0xffffff, // White - subtle brightening like Raycast
            is_light_mode: false,
        }
    }
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
            .size(px(16.))
            .ml(px(2.)) // Nudge right
            .text_color(rgb(self.colors.accent)) // Accent color (yellow/gold)
    }

    /// Render a footer button with label and shortcut
    fn render_button(
        &self,
        id: &'static str,
        label: String,
        shortcut: String,
        hover_bg: u32,
        on_click: Option<Rc<FooterClickCallback>>,
    ) -> impl IntoElement {
        let colors = self.colors;
        let mut btn = div()
            .id(id)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .px(px(8.))
            .py(px(2.))
            .rounded(px(4.))
            .cursor_pointer()
            .hover(move |s| s.bg(rgba(hover_bg)));

        if let Some(callback) = on_click {
            btn = btn.on_click(move |event, window, cx| {
                callback(event, window, cx);
            });
        }

        btn.child(
            div()
                .text_sm()
                .text_color(colors.accent.to_rgb())
                .child(label),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child(shortcut),
        )
    }

    /// Render the vertical divider between buttons
    fn render_divider(&self) -> impl IntoElement {
        div()
            .w(px(1.))
            .h(px(16.))
            .mx(px(4.))
            .bg(self.colors.border.rgba8(0x40)) // 25% opacity
    }
}

impl RenderOnce for PromptFooter {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colors = self.colors;
        let hover_bg = (colors.accent << 8) | 0x26; // 15% opacity for hover

        // Build the right-side container (info label + buttons)
        let mut right_side = hstack().gap(px(8.)).items_center();

        // Info label (e.g., "typescript", "5 items") - shown before buttons
        if let Some(ref info) = self.config.info_label {
            right_side = right_side.child(
                div()
                    .text_xs()
                    .text_color(colors.text_muted.to_rgb())
                    .child(info.clone()),
            );
        }

        // Build the buttons container
        let mut buttons = hstack().gap(px(4.)).items_center();

        // Primary button
        buttons = buttons.child(self.render_button(
            "footer-primary-button",
            self.config.primary_label.clone(),
            self.config.primary_shortcut.clone(),
            hover_bg,
            self.on_primary_click.clone(),
        ));

        // Divider + Secondary button (if enabled)
        if self.config.show_secondary {
            buttons = buttons.child(self.render_divider());
            buttons = buttons.child(self.render_button(
                "footer-secondary-button",
                self.config.secondary_label.clone(),
                self.config.secondary_shortcut.clone(),
                hover_bg,
                self.on_secondary_click.clone(),
            ));
        }

        right_side = right_side.child(buttons);

        // Main footer container (uses FOOTER_HEIGHT constant for single source of truth)
        // Light mode: Raycast-style off-white (#ECEAEC) for clean look
        // Dark mode: semi-transparent for vibrancy support
        let footer_bg = if colors.is_light_mode {
            0xeceaecu32.to_rgb() // Raycast-style off-white in light mode
        } else {
            colors.background.rgba8(0x1f) // ~12% opacity in dark mode
        };
        let border_opacity = if colors.is_light_mode { 0x60 } else { 0x30 };

        let mut footer = div()
            .w_full()
            .h(px(FOOTER_HEIGHT))
            .min_h(px(FOOTER_HEIGHT))
            .max_h(px(FOOTER_HEIGHT))
            .flex_shrink_0()
            .overflow_hidden()
            .px(px(12.))
            .pt(px(0.))
            .pb(px(2.)) // Extra bottom padding shifts content up
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .border_t_1()
            .border_color(colors.border.rgba8(border_opacity))
            .bg(footer_bg);

        // Left side: Logo + helper text
        let mut left_side = hstack().gap(px(8.)).items_center();

        // Logo (if enabled)
        if self.config.show_logo {
            left_side = left_side.child(self.render_logo());
        }

        // Helper text (e.g., "Tab 1 of 2 · Tab to continue, Esc to exit")
        if let Some(ref helper) = self.config.helper_text {
            left_side = left_side.child(
                div()
                    .text_xs()
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
