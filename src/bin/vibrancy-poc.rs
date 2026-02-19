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
use script_kit_gpui::theme::{
    get_cached_theme,
    service::{ensure_theme_service, theme_revision},
};
use script_kit_gpui::ui_foundation::hex_to_rgba_with_opacity;

fn main() {
    Application::new().run(|cx: &mut App| {
        ensure_theme_service(cx);

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

        let open_result = cx.open_window(window_options, |window, cx| {
            cx.new(|cx| RaycastPOC::new(window, cx))
        });

        if let Err(error) = open_result {
            eprintln!("failed to open vibrancy POC window: {error:?}");
        }
    });
}

struct RaycastPOC {
    focus_handle: FocusHandle,
    theme_revision_seen: u64,
}

impl RaycastPOC {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);

        let mut view = Self {
            focus_handle,
            theme_revision_seen: theme_revision(),
        };
        view.start_theme_refresh(cx);
        view
    }

    fn start_theme_refresh(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| loop {
            gpui::Timer::after(std::time::Duration::from_millis(250)).await;

            let should_stop = cx
                .update(|cx| {
                    this.update(cx, |view, cx| {
                        let revision = theme_revision();
                        if view.theme_revision_seen != revision {
                            view.theme_revision_seen = revision;
                            cx.notify();
                        }
                    })
                    .is_err()
                })
                .unwrap_or(true);

            if should_stop {
                break;
            }
        })
        .detach();
    }
}

impl Focusable for RaycastPOC {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RaycastPOC {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_theme_revision = theme_revision();
        if self.theme_revision_seen != current_theme_revision {
            self.theme_revision_seen = current_theme_revision;
            cx.notify();
        }

        let theme = get_cached_theme();
        let opacity = theme.get_opacity();
        let colors = &theme.colors;

        let container_bg = theme_rgba(colors.background.main, opacity.main);
        let input_area_bg = theme_rgba(colors.background.search_box, opacity.search_box);
        let selected_bg = theme_rgba(colors.accent.selected_subtle, opacity.selected);
        let separator_color = theme_rgba(colors.ui.border, opacity.border_active);
        let subtle_border_color = theme_rgba(colors.ui.border, opacity.border_inactive);

        let primary_text = theme_rgba(colors.text.primary, 1.0);
        let secondary_text = theme_rgba(colors.text.tertiary, 1.0);
        let hint_text = theme_rgba(colors.text.dimmed, 1.0);

        let keycap_bg = theme_rgba(colors.background.log_panel, opacity.input_inactive);
        let icon_bg = theme_rgba(colors.ui.border, opacity.border_active);

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
            .border_color(subtle_border_color)
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
                                    .bg(keycap_bg)
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
                        icon_bg,
                    ))
                    .child(render_list_item(
                        "Rewrite Selected Text",
                        "AI Writing Assistant",
                        false,
                        primary_text,
                        secondary_text,
                        selected_bg,
                        icon_bg,
                    ))
                    .child(render_list_item(
                        "Export Settings & Data",
                        "Raycast",
                        false,
                        primary_text,
                        secondary_text,
                        selected_bg,
                        icon_bg,
                    ))
                    .child(render_list_item(
                        "Top Center Sixth",
                        "Window Management",
                        false,
                        primary_text,
                        secondary_text,
                        selected_bg,
                        icon_bg,
                    ))
                    .child(render_list_item(
                        "General",
                        "Raycast Settings",
                        false,
                        primary_text,
                        secondary_text,
                        selected_bg,
                        icon_bg,
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
                        div().size(px(20.)).rounded(px(4.)).bg(icon_bg),
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
                                            .bg(keycap_bg)
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
                                                    .bg(keycap_bg)
                                                    .text_size(px(11.))
                                                    .text_color(secondary_text)
                                                    .child("\u{2318}"), // cmd symbol
                                            )
                                            .child(
                                                div()
                                                    .px(px(6.))
                                                    .py(px(2.))
                                                    .rounded(px(4.))
                                                    .bg(keycap_bg)
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
    icon_bg: gpui::Rgba,
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
                        .bg(icon_bg)
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

fn theme_rgba(hex: u32, opacity: f32) -> gpui::Rgba {
    rgba(hex_to_rgba_with_opacity(hex, opacity))
}
