//! AppShell renderer
//!
//! The shell is a presentational component that renders the frame, header,
//! footer, and content based on a ShellSpec. It does not own state.

use gpui::{div, prelude::*, px, rgba, AnyElement, App, Window};

use super::chrome::DividerSpec;
use super::focus::ShellFocus;
use super::spec::{FooterSpec, HeaderSpec, ShellSpec};
use super::style::ShellStyleCache;
use crate::ui_foundation::{hstack, HexColorExt};
use crate::utils;

/// Runtime context for the shell
///
/// Contains stable references needed for rendering.
/// Owned by the app root, passed to AppShell::render.
pub struct ShellRuntime<'a> {
    /// Stable focus handles
    pub focus: &'a ShellFocus,
    /// Cached styles
    pub style: &'a ShellStyleCache,
    /// Whether cursor is visible (for blinking)
    pub cursor_visible: bool,
}

/// The App Shell - presentational renderer for the unified frame
///
/// This is not a View - it's a function that takes a spec and returns elements.
/// State lives in the app root; the shell just renders what it's told.
pub struct AppShell;

impl AppShell {
    /// Render the shell with the given spec and runtime context
    ///
    /// This is the main entry point. Each view returns a ShellSpec,
    /// and this function renders the appropriate frame and chrome.
    pub fn render(
        spec: ShellSpec,
        runtime: &ShellRuntime,
        _window: &mut Window,
        _cx: &mut App,
    ) -> AnyElement {
        let style = runtime.style;
        let chrome = &spec.chrome;

        // Build the frame based on chrome mode
        let mut frame = div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .track_focus(runtime.focus.root_handle());

        // Apply background and shadow based on chrome mode
        if chrome.mode.has_background() {
            frame = frame.bg(style.frame_bg).rounded(style.radius);
        }

        if chrome.mode.has_shadow() {
            frame = frame.shadow(style.shadows.clone());
        }

        // Render header if present
        if let Some(ref header_spec) = spec.header {
            frame = frame.child(Self::render_header(header_spec, runtime));

            // Render divider if configured
            if chrome.should_show_divider() {
                frame = frame.child(Self::render_divider(chrome.divider, style));
            }
        }

        // Render content
        if let Some(content) = spec.content {
            frame = frame.child(div().flex_1().overflow_hidden().child(content));
        }

        // Render footer if present
        if let Some(ref footer_spec) = spec.footer {
            frame = frame.child(Self::render_footer(footer_spec, runtime));
        }

        frame.into_any_element()
    }

    /// Render the header based on HeaderSpec
    fn render_header(spec: &HeaderSpec, runtime: &ShellRuntime) -> AnyElement {
        let colors = &runtime.style.header;
        let cursor_visible = runtime.cursor_visible;

        // Main header container
        let mut header = hstack()
            .w_full()
            .px(px(16.0))
            .py(px(8.0))
            .gap(px(12.0))
            .items_center();

        // Render input area if present
        if let Some(ref input) = spec.input {
            let show_cursor = input.cursor_visible && input.is_focused && cursor_visible;
            let text_is_empty = input.text.is_empty();

            let mut input_area = hstack().flex_1().text_lg();

            // Path prefix
            if let Some(ref prefix) = spec.path_prefix {
                input_area =
                    input_area.child(div().text_color(colors.text_muted).child(prefix.clone()));
            }

            // Cursor on left when empty
            if text_is_empty && show_cursor {
                input_area = input_area.child(
                    div()
                        .w(px(2.0))
                        .h(px(18.0))
                        .my(px(2.0))
                        .mr(px(4.0))
                        .bg(colors.text_primary),
                );
            }

            // Text or placeholder
            let display_text = if text_is_empty {
                input_area = input_area.text_color(colors.text_muted);
                input.placeholder.clone()
            } else {
                input_area = input_area.text_color(colors.text_primary);
                input.text.clone()
            };
            input_area = input_area.child(display_text);

            // Cursor on right when has text
            if !text_is_empty && show_cursor {
                input_area = input_area.child(
                    div()
                        .w(px(2.0))
                        .h(px(18.0))
                        .my(px(2.0))
                        .ml(px(4.0))
                        .bg(colors.text_primary),
                );
            }

            header = header.child(input_area);
        }

        // Ask AI hint
        if spec.show_ask_ai_hint {
            header = header.child(Self::render_ask_ai_hint(colors));
        }

        // Buttons
        if !spec.buttons.is_empty() {
            let mut buttons_area = hstack().gap(px(16.0)).justify_end();

            for btn in &spec.buttons {
                buttons_area = buttons_area.child(
                    hstack()
                        .gap(px(4.0))
                        .items_center()
                        .cursor_pointer()
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent)
                                .child(btn.label.clone()),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted)
                                .child(btn.shortcut.clone()),
                        ),
                );
            }

            header = header.child(buttons_area);
        }

        // Logo
        if spec.show_logo {
            header = header.child(Self::render_logo(colors.accent_hex, colors.logo_icon_hex));
        }

        header.into_any_element()
    }

    /// Render the "Ask AI [Tab]" hint
    fn render_ask_ai_hint(colors: &super::style::HeaderColors) -> AnyElement {
        hstack()
            .flex_shrink_0()
            .gap(px(6.0))
            .items_center()
            .child(
                div()
                    .text_sm()
                    .text_color(colors.text_muted)
                    .child("Ask AI"),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(colors.border)
                    .text_xs()
                    .text_color(colors.text_muted)
                    .child("Tab"),
            )
            .into_any_element()
    }

    /// Render the Script Kit logo
    fn render_logo(accent: u32, logo_icon: u32) -> AnyElement {
        div()
            .w(px(21.0))
            .h(px(21.0))
            .flex()
            .items_center()
            .justify_center()
            .bg(accent.rgba8(0xD9)) // 85% opacity
            .rounded(px(4.0))
            .child(
                gpui::svg()
                    .external_path(utils::get_logo_path())
                    .size(px(13.0))
                    .text_color(gpui::rgb(logo_icon)),
            )
            .into_any_element()
    }

    /// Render the divider between header and content
    fn render_divider(divider: DividerSpec, style: &ShellStyleCache) -> AnyElement {
        match divider {
            DividerSpec::None => div().into_any_element(),
            DividerSpec::Hairline => div()
                .mx(px(16.0))
                .h(px(1.0))
                .bg(style.divider.line)
                .into_any_element(),
        }
    }

    /// Render the footer based on FooterSpec
    fn render_footer(spec: &FooterSpec, runtime: &ShellRuntime) -> AnyElement {
        let colors = &runtime.style.footer;

        // Main footer container (35px height)
        let mut footer = div()
            .w_full()
            .h(px(35.0))
            .px(px(12.0))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .border_t_1()
            .border_color(colors.border_hex.rgba8(0x30))
            .bg(rgba(0x00000080)); // 50% opacity for vibrancy balance

        // Left side: Logo + helper text
        let mut left_side = hstack().gap(px(8.0)).items_center();

        if spec.show_logo {
            left_side = left_side.child(Self::render_footer_logo(
                colors.accent_hex,
                colors.logo_icon_hex,
            ));
        }

        if let Some(ref helper) = spec.helper_text {
            left_side = left_side.child(
                div()
                    .text_xs()
                    .text_color(colors.accent)
                    .child(helper.clone()),
            );
        }

        footer = footer.child(left_side);

        // Right side: Info + buttons
        let mut right_side = hstack().gap(px(8.0)).items_center();

        if let Some(ref info) = spec.info_label {
            right_side = right_side.child(
                div()
                    .text_xs()
                    .text_color(colors.text_muted)
                    .child(info.clone()),
            );
        }

        // Buttons container
        let mut buttons = hstack().gap(px(4.0));

        // Primary button
        if !spec.primary_label.is_empty() {
            buttons = buttons.child(Self::render_footer_button(
                spec.primary_label.clone(),
                spec.primary_shortcut.clone(),
                colors,
            ));
        }

        // Divider + Secondary button
        if let Some(ref label) = spec.secondary_label {
            // Divider
            buttons = buttons.child(
                div()
                    .w(px(1.0))
                    .h(px(16.0))
                    .mx(px(4.0))
                    .bg(colors.border_hex.rgba8(0x40)),
            );

            buttons = buttons.child(Self::render_footer_button(
                label.clone(),
                spec.secondary_shortcut.clone().unwrap_or_default(),
                colors,
            ));
        }

        right_side = right_side.child(buttons);
        footer = footer.child(right_side);

        footer.into_any_element()
    }

    /// Render a footer button
    fn render_footer_button(
        label: gpui::SharedString,
        shortcut: gpui::SharedString,
        colors: &super::style::FooterColors,
    ) -> AnyElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .child(div().text_sm().text_color(colors.accent).child(label))
            .child(
                div()
                    .text_sm()
                    .text_color(colors.text_muted)
                    .child(shortcut),
            )
            .into_any_element()
    }

    /// Render the footer logo (slightly smaller than header logo)
    fn render_footer_logo(accent: u32, logo_icon: u32) -> AnyElement {
        div()
            .w(px(20.0))
            .h(px(20.0))
            .flex()
            .items_center()
            .justify_center()
            .bg(accent.rgba8(0xD9))
            .rounded(px(4.0))
            .child(
                gpui::svg()
                    .external_path(utils::get_logo_path())
                    .size(px(13.0))
                    .text_color(gpui::rgb(logo_icon)),
            )
            .into_any_element()
    }
}
