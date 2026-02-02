//! Raycast Light Theme Vibrancy POC
//!
//! Run with: cargo run --bin vibrancy-poc
//!
//! Demonstrates macOS vibrancy (frosted glass blur) effect using GPUI.
//! The key is combining:
//! 1. WindowBackgroundAppearance::Blurred - enables NSVisualEffectView
//! 2. Semi-transparent background colors - allows blur to show through
//! 3. Light color palette matching Raycast's light theme

use gpui::{
    div, point, prelude::*, px, rgba, size, App, Application, Context, FocusHandle, Focusable,
    Render, Window, WindowBackgroundAppearance, WindowBounds, WindowOptions,
};

fn main() {
    Application::new().run(|cx: &mut App| {
        // Window size similar to Raycast
        let window_size = size(px(750.), px(500.));

        // Calculate centered position on screen
        let bounds = gpui::Bounds {
            origin: point(px(400.), px(200.)),
            size: window_size,
        };

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            // No titlebar for clean Raycast-like appearance
            titlebar: None,
            is_movable: true,
            // KEY: This enables macOS vibrancy (NSVisualEffectView blur effect)
            window_background: WindowBackgroundAppearance::Blurred,
            focus: true,
            show: true,
            ..Default::default()
        };

        cx.open_window(window_options, |window, cx| {
            cx.new(|cx| RaycastPOC::new(window, cx))
        })
        .unwrap();
    });
}

struct RaycastPOC {
    focus_handle: FocusHandle,
}

impl RaycastPOC {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);

        Self { focus_handle }
    }
}

impl Focusable for RaycastPOC {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RaycastPOC {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Raycast light theme colors (with transparency for vibrancy)
        // The key insight: use semi-transparent backgrounds to let the blur show through

        // Main container background - very light with slight transparency
        // rgba(250, 250, 250, 0.85) - light gray, 85% opacity lets blur through
        let container_bg = rgba(0xFAFAFAD9);

        // Input area background - slightly lighter/more transparent
        let input_area_bg = rgba(0xFFFFFFE6); // white at 90% opacity

        // List item hover/selected background
        let selected_bg = rgba(0xE8E8E8CC); // light gray at 80% opacity

        // Separator color
        let separator_color = rgba(0xE0E0E0FF);

        // Text colors
        let primary_text = rgba(0x1A1A1AFF); // near black
        let secondary_text = rgba(0x6B6B6BFF); // medium gray
        let hint_text = rgba(0x9B9B9BFF); // light gray

        div()
            .id("raycast-container")
            .track_focus(&self.focus_handle)
            .size_full()
            .flex()
            .flex_col()
            // Main container with rounded corners and subtle transparency
            .bg(container_bg)
            .rounded(px(12.))
            .border_1()
            .border_color(rgba(0xD0D0D040)) // subtle border
            .overflow_hidden()
            .child(
                // Search input area
                div()
                    .id("search-area")
                    .w_full()
                    .px(px(16.))
                    .py(px(12.))
                    .bg(input_area_bg)
                    .border_b_1()
                    .border_color(separator_color)
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        // Fake search input (text display)
                        div()
                            .flex_1()
                            .text_size(px(18.))
                            .text_color(primary_text)
                            .child("testing"),
                    )
                    .child(
                        // "Ask AI" hint on the right
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .text_color(hint_text)
                                    .child("Ask AI"),
                            )
                            .child(
                                div()
                                    .px(px(6.))
                                    .py(px(2.))
                                    .rounded(px(4.))
                                    .bg(rgba(0xE8E8E8FF))
                                    .text_size(px(11.))
                                    .text_color(secondary_text)
                                    .child("Tab"),
                            ),
                    ),
            )
            .child(
                // Results section header
                div()
                    .w_full()
                    .px(px(16.))
                    .py(px(8.))
                    .text_size(px(12.))
                    .text_color(hint_text)
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child("Results"),
            )
            .child(
                // Results list
                div()
                    .id("results-list")
                    .flex_1()
                    .w_full()
                    .flex()
                    .flex_col()
                    // Sample list items
                    .child(render_list_item(
                        "System Settings",
                        "Application",
                        true, // selected
                        primary_text,
                        secondary_text,
                        selected_bg,
                    ))
                    .child(render_list_item(
                        "Rewrite Selected Text",
                        "AI Writing Assistant",
                        false,
                        primary_text,
                        secondary_text,
                        selected_bg,
                    ))
                    .child(render_list_item(
                        "Export Settings & Data",
                        "Raycast",
                        false,
                        primary_text,
                        secondary_text,
                        selected_bg,
                    ))
                    .child(render_list_item(
                        "Top Center Sixth",
                        "Window Management",
                        false,
                        primary_text,
                        secondary_text,
                        selected_bg,
                    ))
                    .child(render_list_item(
                        "General",
                        "Raycast Settings",
                        false,
                        primary_text,
                        secondary_text,
                        selected_bg,
                    )),
            )
            .child(
                // Footer / action bar
                div()
                    .w_full()
                    .px(px(16.))
                    .py(px(10.))
                    .border_t_1()
                    .border_color(separator_color)
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        // Left side - app icon placeholder
                        div().size(px(20.)).rounded(px(4.)).bg(rgba(0xD0D0D0FF)),
                    )
                    .child(
                        // Right side - actions
                        div()
                            .flex()
                            .items_center()
                            .gap(px(16.))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .text_color(secondary_text)
                                            .child("Open Application"),
                                    )
                                    .child(
                                        div()
                                            .px(px(6.))
                                            .py(px(2.))
                                            .rounded(px(4.))
                                            .bg(rgba(0xE8E8E8FF))
                                            .text_size(px(11.))
                                            .text_color(secondary_text)
                                            .child("\u{21B5}"), // return symbol
                                    ),
                            )
                            .child(div().h(px(16.)).w(px(1.)).bg(separator_color))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .text_color(secondary_text)
                                            .child("Actions"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .gap(px(2.))
                                            .child(
                                                div()
                                                    .px(px(6.))
                                                    .py(px(2.))
                                                    .rounded(px(4.))
                                                    .bg(rgba(0xE8E8E8FF))
                                                    .text_size(px(11.))
                                                    .text_color(secondary_text)
                                                    .child("\u{2318}"), // cmd symbol
                                            )
                                            .child(
                                                div()
                                                    .px(px(6.))
                                                    .py(px(2.))
                                                    .rounded(px(4.))
                                                    .bg(rgba(0xE8E8E8FF))
                                                    .text_size(px(11.))
                                                    .text_color(secondary_text)
                                                    .child("K"),
                                            ),
                                    ),
                            ),
                    ),
            )
    }
}

fn render_list_item(
    title: &str,
    subtitle: &str,
    selected: bool,
    primary_text: gpui::Rgba,
    secondary_text: gpui::Rgba,
    selected_bg: gpui::Rgba,
) -> impl IntoElement {
    div()
        .w_full()
        .px(px(12.))
        .py(px(8.))
        .mx(px(4.))
        .rounded(px(8.))
        .when(selected, |d| d.bg(selected_bg))
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(12.))
                .child(
                    // Icon placeholder
                    div()
                        .size(px(28.))
                        .rounded(px(6.))
                        .bg(rgba(0xD0D0D0FF))
                        .flex()
                        .items_center()
                        .justify_center(),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_size(px(14.))
                                .text_color(primary_text)
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(title.to_string()),
                        )
                        .child(
                            div()
                                .text_size(px(13.))
                                .text_color(secondary_text)
                                .child(subtitle.to_string()),
                        ),
                ),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(secondary_text)
                .child(if selected { "Application" } else { "Command" }),
        )
}
