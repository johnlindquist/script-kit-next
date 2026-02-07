use gpui::*;
use std::rc::Rc;

use crate::components::{Button, ButtonColors, ButtonVariant};
use crate::panel::{CURSOR_GAP_X, CURSOR_HEIGHT_LG, CURSOR_MARGIN_Y, CURSOR_WIDTH};
use crate::ui_foundation::{hstack, HexColorExt};

use super::{PromptHeaderColors, PromptHeaderConfig, HEADER_PATH_PREFIX_MAX_WIDTH_PX};

/// Callback type for button click events
pub type HeaderClickCallback = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A reusable header component for prompts
///
/// Displays:
/// - Search input with blinking cursor
/// - Path prefix (optional)
/// - Primary action button (Run/Select)
/// - Actions button (optional)
/// - Script Kit logo
///
#[derive(IntoElement)]
pub struct PromptHeader {
    config: PromptHeaderConfig,
    colors: PromptHeaderColors,
    on_primary_click: Option<Rc<HeaderClickCallback>>,
    on_actions_click: Option<Rc<HeaderClickCallback>>,
}

impl PromptHeader {
    /// Create a new PromptHeader with the given configuration and colors
    pub fn new(config: PromptHeaderConfig, colors: PromptHeaderColors) -> Self {
        Self {
            config,
            colors,
            on_primary_click: None,
            on_actions_click: None,
        }
    }

    /// Set the primary button click callback
    pub fn on_primary_click(mut self, callback: HeaderClickCallback) -> Self {
        self.on_primary_click = Some(Rc::new(callback));
        self
    }

    /// Set the actions button click callback
    pub fn on_actions_click(mut self, callback: HeaderClickCallback) -> Self {
        self.on_actions_click = Some(Rc::new(callback));
        self
    }

    /// Render the search input area with cursor
    fn render_input_area(&self) -> impl IntoElement {
        let colors = self.colors;
        let filter_is_empty = self.config.filter_text.is_empty();
        let cursor_visible = self.config.cursor_visible && self.config.is_focused;

        // Display text: filter text or placeholder
        let display_text: SharedString = if filter_is_empty {
            self.config.placeholder.clone().into()
        } else {
            self.config.filter_text.clone().into()
        };

        // Text color: muted for placeholder, primary for input
        let text_color = if filter_is_empty {
            colors.text_muted.to_rgb()
        } else {
            colors.text_primary.to_rgb()
        };

        // Build input container using hstack() helper
        let mut input = hstack()
            .flex_1()
            .text_size(px(colors.input_font_size))
            .text_color(text_color);

        // Path prefix (if present)
        if let Some(ref prefix) = self.config.path_prefix {
            input = input.child(
                div()
                    .max_w(px(HEADER_PATH_PREFIX_MAX_WIDTH_PX))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .text_color(colors.text_muted.to_rgb())
                    .child(prefix.clone()),
            );
        }

        // Cursor position:
        // - When empty: cursor LEFT (before placeholder)
        // - When typing: cursor RIGHT (after text)
        //
        // ALIGNMENT FIX: The left cursor (when empty) takes up space (CURSOR_WIDTH + CURSOR_GAP_X).
        // We apply a negative margin to the placeholder text to pull it back by that amount,
        // so placeholder and typed text share the same starting x-position. This eliminates
        // the "jump" when typing begins.

        // Left cursor (when empty)
        // Use conditional background instead of .when() to avoid type inference issues
        if filter_is_empty {
            let cursor_bg = if cursor_visible {
                colors.text_primary.to_rgb()
            } else {
                0x000000u32.with_opacity(0.0)
            };
            input = input.child(
                div()
                    .w(px(CURSOR_WIDTH))
                    .h(px(CURSOR_HEIGHT_LG))
                    .my(px(CURSOR_MARGIN_Y))
                    .mr(px(CURSOR_GAP_X))
                    .bg(cursor_bg),
            );
        }

        // Display text - with negative margin for placeholder alignment
        if filter_is_empty {
            // Placeholder: pull back by cursor space to align with typed text position
            input = input.child(
                div()
                    .ml(px(-(CURSOR_WIDTH + CURSOR_GAP_X)))
                    .child(display_text),
            );
        } else {
            input = input.child(display_text);
        }

        // Right cursor (when not empty)
        if !filter_is_empty {
            let cursor_bg = if cursor_visible {
                colors.text_primary.to_rgb()
            } else {
                0x000000u32.with_opacity(0.0)
            };
            input = input.child(
                div()
                    .w(px(CURSOR_WIDTH))
                    .h(px(CURSOR_HEIGHT_LG))
                    .my(px(CURSOR_MARGIN_Y))
                    .ml(px(CURSOR_GAP_X))
                    .bg(cursor_bg),
            );
        }

        input
    }

    /// Render the action buttons area (Run + Actions) - no separators style
    fn render_buttons_area(&self) -> impl IntoElement {
        let colors = self.colors;
        let button_colors = ButtonColors {
            text_color: colors.accent,
            text_hover: colors.text_primary,
            background: colors.background,
            background_hover: colors.background,
            accent: colors.accent,
            border: colors.border,
            focus_ring: colors.accent,
            focus_tint: colors.background,
            hover_overlay: colors.hover_overlay,
        };

        let on_primary = self.on_primary_click.clone();
        let on_actions = self.on_actions_click.clone();

        // Use hstack() helper with gap for clean spacing (no pipe separators)
        let mut container = hstack().justify_end().gap(px(16.));

        // Primary button
        let mut primary_btn = Button::new(self.config.primary_button_label.clone(), button_colors)
            .variant(ButtonVariant::Ghost)
            .shortcut(self.config.primary_button_shortcut.clone());

        if let Some(callback) = on_primary {
            primary_btn = primary_btn.on_click(Box::new(move |event, window, cx| {
                tracing::debug!("Primary button callback invoked");
                callback(event, window, cx);
            }));
        }
        container = container.child(primary_btn);

        // Actions button (if enabled)
        if self.config.show_actions_button {
            let mut actions_btn = Button::new("Actions", button_colors)
                .variant(ButtonVariant::Ghost)
                .shortcut("⌘ K");

            if let Some(callback) = on_actions {
                actions_btn = actions_btn.on_click(Box::new(move |event, window, cx| {
                    tracing::debug!("Actions button callback invoked");
                    callback(event, window, cx);
                }));
            }

            container = container.child(actions_btn);
        }

        container
    }

    /// Render the actions search input (when in actions_mode)
    fn render_actions_search(&self) -> impl IntoElement {
        let colors = self.colors;
        let search_is_empty = self.config.actions_search_text.is_empty();
        let cursor_visible = self.config.cursor_visible && self.config.is_focused;

        let search_display: SharedString = if search_is_empty {
            "Search actions...".into()
        } else {
            self.config.actions_search_text.clone().into()
        };

        // Compute cursor background color using HexColorExt
        let cursor_bg = if cursor_visible {
            colors.accent.to_rgb()
        } else {
            0x000000u32.with_opacity(0.0)
        };

        // Build the search input element using hstack() helper
        let search_width = self.config.actions_density.actions_search_width_px();
        let mut search_input = hstack()
            .flex_shrink_0()
            .w(px(search_width))
            .min_w(px(search_width))
            .max_w(px(search_width))
            .h(rems(1.5))
            .min_h(rems(1.5))
            .max_h(rems(1.5))
            .overflow_hidden()
            .px(rems(0.5))
            .rounded(px(4.))
            // Use rgba8() instead of manual << 8 | alpha
            .bg(colors
                .search_box_bg
                .rgba8(if search_is_empty { 0x40 } else { 0x80 }))
            .border_1()
            .border_color(
                colors
                    .accent
                    .rgba8(if search_is_empty { 0x20 } else { 0x40 }),
            )
            .text_size(px(colors.supporting_font_size))
            .text_color(if search_is_empty {
                colors.text_muted.to_rgb()
            } else {
                colors.text_primary.to_rgb()
            });

        // Cursor before placeholder when empty
        if search_is_empty {
            search_input = search_input.child(
                div()
                    .w(px(2.))
                    .h(rems(0.875))
                    .mr(px(2.))
                    .rounded(px(1.))
                    .bg(cursor_bg),
            );
        }

        search_input = search_input.child(search_display);

        // Cursor after text when not empty
        if !search_is_empty {
            search_input = search_input.child(
                div()
                    .w(px(2.))
                    .h(rems(0.875))
                    .ml(px(2.))
                    .rounded(px(1.))
                    .bg(cursor_bg),
            );
        }

        // Use hstack() helper for container
        hstack()
            .justify_end()
            .gap(rems(0.5))
            // ⌘K indicator
            .child(
                div()
                    .text_color(colors.text_dimmed.to_rgb())
                    .text_size(px(colors.caption_font_size))
                    .child("⌘K"),
            )
            // Search input display
            .child(search_input)
            .child(
                div()
                    .mx(rems(0.25))
                    .text_color(colors.text_dimmed.rgba8(0x60))
                    .text_size(px(colors.supporting_font_size))
                    .child("|"),
            )
    }

    /// Render the "Ask AI" hint with Tab badge (Raycast-style)
    ///
    /// Displays: "Ask AI [Tab]" where Tab is in a subtle bordered badge
    fn render_ask_ai_hint(&self) -> impl IntoElement {
        let colors = self.colors;

        hstack()
            .flex_shrink_0()
            .gap(rems(0.375))
            .items_center()
            // "Ask AI" text in muted color
            .child(
                div()
                    .text_size(px(colors.supporting_font_size))
                    .text_color(colors.text_muted.to_rgb())
                    .child("Ask AI"),
            )
            // "Tab" badge with border
            .child(
                div()
                    .flex_shrink_0()
                    .px(rems(0.375))
                    .py(rems(0.125))
                    .rounded(px(4.))
                    .border_1()
                    .border_color(colors.border.to_rgb())
                    .text_size(px(colors.caption_font_size))
                    .text_color(colors.text_muted.to_rgb())
                    .child("Tab"),
            )
    }

    /// Render the Script Kit logo (golden ratio: 21px container, 13px SVG, 4px radius)
    fn render_logo(&self) -> impl IntoElement {
        div()
            .w(px(21.))
            .h(px(21.))
            .flex()
            .items_center()
            .justify_center()
            .bg(self.colors.accent.rgba8(0xD9)) // 85% opacity (0xD9 = 217 = 85% of 255)
            .rounded(px(4.))
            .child(
                svg()
                    .external_path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
                    .size(px(13.))
                    // Use logo_icon color from theme/design for contrast on accent background
                    .text_color(rgb(self.colors.logo_icon)),
            )
    }
}

impl RenderOnce for PromptHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let actions_mode = self.config.actions_mode;
        let show_ask_ai_hint = self.config.show_ask_ai_hint;

        // Pre-compute visibility styles for buttons and search layers
        // Use opacity and visibility for CLS-free toggling
        let (buttons_opacity, buttons_visible) = if actions_mode {
            (0., false)
        } else {
            (1., true)
        };
        let (search_opacity, search_visible) = if actions_mode {
            (1., true)
        } else {
            (0., false)
        };

        // Build buttons layer using hstack() helper
        let mut buttons_layer = hstack()
            .absolute()
            .inset_0()
            .justify_end()
            .opacity(buttons_opacity);

        if !buttons_visible {
            buttons_layer = buttons_layer.invisible();
        }
        buttons_layer = buttons_layer.child(self.render_buttons_area());

        // Build search layer using hstack() helper
        let mut search_layer = hstack()
            .absolute()
            .inset_0()
            .justify_end()
            .opacity(search_opacity);

        if !search_visible {
            search_layer = search_layer.invisible();
        }
        search_layer = search_layer.child(self.render_actions_search());

        // Main header using hstack() helper
        let mut header = hstack()
            .w_full()
            .px(rems(1.0))
            .py(rems(0.5))
            .gap(rems(0.75))
            // Search input area
            .child(self.render_input_area());

        // "Ask AI [Tab]" hint (conditionally rendered before buttons)
        if show_ask_ai_hint {
            header = header.child(self.render_ask_ai_hint());
        }

        // CLS-free actions area with stacked layers
        // Note: This container needs min-width for absolute children to be visible
        header = header.child(
            div()
                .relative()
                .flex_shrink_0()
                .min_w(px(self.config.actions_density.reserved_min_width_px()))
                .h(rems(1.75))
                .flex()
                .items_center()
                .child(buttons_layer)
                .child(search_layer),
        );

        // Script Kit logo
        header.child(self.render_logo())
    }
}
