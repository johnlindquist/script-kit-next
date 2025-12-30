#![allow(dead_code)]
//! Material Design 3 (Material You) Renderer
//!
//! Implements Google's latest Material Design language with:
//! - Rounded corners (12px for cards, 28px for search pill)
//! - Elevated surfaces with tonal colors
//! - Roboto-style typography (using system font)
//! - Teal/purple accent palette
//! - Card-based layout with elevation
//! - Pill-shaped chips for metadata
//! - M3 search bar style (rounded pill, filled)

use gpui::prelude::FluentBuilder;
use gpui::*;

use super::{DesignRenderer, DesignVariant};
use crate::list_item::LIST_ITEM_HEIGHT;

/// Material Design 3 color tokens
mod colors {
    /// Surface - light lavender background
    pub const SURFACE: u32 = 0xfef7ff;
    /// Surface container - slightly darker lavender
    pub const SURFACE_CONTAINER: u32 = 0xece6f0;
    /// Surface container high - elevated surface
    pub const SURFACE_CONTAINER_HIGH: u32 = 0xe6e0e9;
    /// Primary - purple accent
    pub const PRIMARY: u32 = 0x6750a4;
    /// On primary - text on primary color
    pub const ON_PRIMARY: u32 = 0xffffff;
    /// Primary container - tonal container
    pub const PRIMARY_CONTAINER: u32 = 0xeaddff;
    /// On primary container - text on container
    pub const ON_PRIMARY_CONTAINER: u32 = 0x21005d;
    /// Secondary - teal accent
    pub const SECONDARY: u32 = 0x625b71;
    /// Secondary container - teal tonal
    pub const SECONDARY_CONTAINER: u32 = 0xe8def8;
    /// On surface - primary text color
    pub const ON_SURFACE: u32 = 0x1d1b20;
    /// On surface variant - secondary text
    pub const ON_SURFACE_VARIANT: u32 = 0x49454f;
    /// Outline - borders
    pub const OUTLINE: u32 = 0x79747e;
    /// Outline variant - subtle borders
    pub const OUTLINE_VARIANT: u32 = 0xcac4d0;
    /// Tertiary - accent for chips
    pub const TERTIARY: u32 = 0x7d5260;
    /// Tertiary container - chip background
    pub const TERTIARY_CONTAINER: u32 = 0xffd8e4;
}

/// Corner radius tokens (M3 shape system)
mod corners {
    /// Extra small - 4px
    pub const XS: f32 = 4.0;
    /// Small - 8px
    pub const SM: f32 = 8.0;
    /// Medium - 12px (cards)
    pub const MD: f32 = 12.0;
    /// Large - 16px
    pub const LG: f32 = 16.0;
    /// Extra large - 28px (search pill, FABs)
    pub const XL: f32 = 28.0;
    /// Full - completely round
    pub const FULL: f32 = 999.0;
}

/// Elevation tokens (shadow levels)
mod elevation {
    /// Level 0 - no elevation
    pub const LEVEL0: f32 = 0.0;
    /// Level 1 - slight elevation (cards)
    pub const LEVEL1: f32 = 1.0;
    /// Level 2 - medium elevation (raised buttons)
    pub const LEVEL2: f32 = 3.0;
    /// Level 3 - high elevation (dialogs)
    pub const LEVEL3: f32 = 6.0;
}

/// Material Design 3 renderer
pub struct Material3Renderer;

impl Material3Renderer {
    /// Create a new Material3 renderer
    pub fn new() -> Self {
        Self
    }

    /// Render a M3 search bar (pill-shaped, filled)
    fn render_search_bar(&self, filter: &str) -> impl IntoElement {
        div().w_full().h(px(56.0)).px(px(16.0)).py(px(8.0)).child(
            div()
                .w_full()
                .h_full()
                .bg(rgb(colors::SURFACE_CONTAINER_HIGH))
                .rounded(px(corners::XL))
                .px(px(16.0))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(12.0))
                // Search icon placeholder
                .child(
                    div()
                        .w(px(24.0))
                        .h(px(24.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(colors::ON_SURFACE_VARIANT))
                        .child("🔍"),
                )
                // Search text or placeholder
                .child(
                    div()
                        .flex_1()
                        .text_color(if filter.is_empty() {
                            rgb(colors::ON_SURFACE_VARIANT)
                        } else {
                            rgb(colors::ON_SURFACE)
                        })
                        .text_size(px(16.0))
                        .child(if filter.is_empty() {
                            "Search scripts...".to_string()
                        } else {
                            filter.to_string()
                        }),
                ),
        )
    }

    /// Render a M3 list item card
    fn render_list_item(
        &self,
        name: &str,
        description: Option<&str>,
        shortcut: Option<&str>,
        is_selected: bool,
        index: usize,
    ) -> impl IntoElement {
        let bg_color = if is_selected {
            rgb(colors::PRIMARY_CONTAINER)
        } else {
            rgb(colors::SURFACE)
        };

        let text_color = if is_selected {
            rgb(colors::ON_PRIMARY_CONTAINER)
        } else {
            rgb(colors::ON_SURFACE)
        };

        let secondary_color = if is_selected {
            rgb(colors::ON_PRIMARY_CONTAINER)
        } else {
            rgb(colors::ON_SURFACE_VARIANT)
        };

        div()
            .w_full()
            .h(px(LIST_ITEM_HEIGHT))
            .px(px(16.0))
            .py(px(4.0))
            .id(ElementId::NamedInteger("m3-item".into(), index as u64))
            .child(
                div()
                    .w_full()
                    .h_full()
                    .bg(bg_color)
                    .rounded(px(corners::MD))
                    .px(px(16.0))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(colors::SURFACE_CONTAINER)))
                    // Left side: text content
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .overflow_hidden()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            // Title
                            .child(
                                div()
                                    .text_color(text_color)
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .overflow_hidden()
                                    .child(name.to_string()),
                            )
                            // Description (if present)
                            .when_some(description, |d: Div, desc: &str| {
                                d.child(
                                    div()
                                        .text_color(secondary_color)
                                        .text_size(px(14.0))
                                        .overflow_hidden()
                                        .max_h(px(20.0))
                                        .child(desc.to_string()),
                                )
                            }),
                    )
                    // Right side: shortcut chip
                    .when_some(shortcut, |d: Div, sc: &str| {
                        d.child(self.render_chip(sc, is_selected))
                    }),
            )
    }

    /// Render a M3 chip (pill-shaped metadata badge)
    fn render_chip(&self, text: &str, is_selected: bool) -> impl IntoElement {
        let (bg, fg) = if is_selected {
            (rgb(colors::TERTIARY_CONTAINER), rgb(colors::TERTIARY))
        } else {
            (
                rgb(colors::SURFACE_CONTAINER),
                rgb(colors::ON_SURFACE_VARIANT),
            )
        };

        div()
            .h(px(32.0))
            .px(px(12.0))
            .bg(bg)
            .rounded(px(corners::SM))
            .flex()
            .items_center()
            .justify_center()
            .text_color(fg)
            .text_size(px(12.0))
            .font_weight(FontWeight::MEDIUM)
            .child(text.to_string())
    }

    /// Render the design label badge
    fn render_design_label(&self) -> impl IntoElement {
        div()
            .h(px(32.0))
            .px(px(16.0))
            .bg(rgb(colors::PRIMARY))
            .rounded(px(corners::FULL))
            .flex()
            .items_center()
            .justify_center()
            .text_color(rgb(colors::ON_PRIMARY))
            .text_size(px(12.0))
            .font_weight(FontWeight::MEDIUM)
            .child("Material 3")
    }
}

impl Default for Material3Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl<App> DesignRenderer<App> for Material3Renderer {
    fn render_script_list(&self, _app: &App, _cx: &mut Context<App>) -> AnyElement {
        // Demo content for preview
        let demo_items = vec![
            ("Open Project", Some("Launch your favorite IDE"), Some("⌘O")),
            ("Search Files", Some("Find files in workspace"), Some("⌘P")),
            ("Run Build", Some("Execute build command"), Some("⌘B")),
            ("Git Status", Some("Check repository status"), None),
            ("Terminal", Some("Open new terminal window"), Some("⌘T")),
        ];

        div()
            .w_full()
            .h_full()
            .bg(rgb(colors::SURFACE))
            .flex()
            .flex_col()
            // Header with design label
            .child(
                div()
                    .w_full()
                    .h(px(48.0))
                    .px(px(16.0))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_color(rgb(colors::ON_SURFACE))
                            .text_size(px(22.0))
                            .font_weight(FontWeight::MEDIUM)
                            .child("Scripts"),
                    )
                    .child(self.render_design_label()),
            )
            // Search bar
            .child(self.render_search_bar(""))
            // List items
            .child(
                div()
                    .w_full()
                    .flex_1()
                    .overflow_hidden()
                    .py(px(8.0))
                    .children(demo_items.into_iter().enumerate().map(
                        |(i, (name, desc, shortcut))| {
                            self.render_list_item(name, desc, shortcut, i == 0, i)
                        },
                    )),
            )
            .into_any_element()
    }

    fn variant(&self) -> DesignVariant {
        DesignVariant::Material3
    }
}

// ============================================================================
// Standalone render functions for window components
// ============================================================================

/// Render Material 3-styled header
///
/// M3 top app bar with elevation and tonal colors.
pub fn render_material3_header(title: &str) -> impl IntoElement {
    div()
        .w_full()
        .h(px(64.))
        .px(px(16.))
        .bg(rgb(colors::SURFACE))
        .shadow(vec![BoxShadow {
            color: hsla(0., 0., 0., 0.1),
            offset: point(px(0.), px(2.)),
            blur_radius: px(4.),
            spread_radius: px(0.),
        }])
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_size(px(22.))
                .font_weight(FontWeight::NORMAL)
                .text_color(rgb(colors::ON_SURFACE))
                .child(title.to_string()),
        )
        .child(
            // M3 action icon placeholder
            div()
                .w(px(40.))
                .h(px(40.))
                .rounded(px(corners::FULL))
                .bg(rgb(colors::PRIMARY))
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(colors::ON_PRIMARY))
                .text_size(px(14.))
                .child("M3"),
        )
}

/// Render Material 3-styled preview panel
///
/// Elevated surface with M3 shape and tonal container.
pub fn render_material3_preview_panel(content: Option<&str>) -> impl IntoElement {
    let display_content = content.unwrap_or("Select an item to preview");
    let text_color = if content.is_some() {
        rgb(colors::ON_SURFACE)
    } else {
        rgb(colors::ON_SURFACE_VARIANT)
    };

    div().w_full().h_full().p(px(16.)).child(
        div()
            .w_full()
            .h_full()
            .p(px(20.))
            .bg(rgb(colors::SURFACE_CONTAINER))
            .rounded(px(corners::MD))
            .shadow(vec![BoxShadow {
                color: hsla(0., 0., 0., 0.1),
                offset: point(px(0.), px(elevation::LEVEL1)),
                blur_radius: px(3.),
                spread_radius: px(0.),
            }])
            .flex()
            .flex_col()
            .child(
                div()
                    .text_size(px(12.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(colors::ON_SURFACE_VARIANT))
                    .mb(px(16.))
                    .child("Preview"),
            )
            .child(
                div()
                    .flex_1()
                    .text_size(px(14.))
                    .text_color(text_color)
                    .overflow_hidden()
                    .child(display_content.to_string()),
            ),
    )
}

/// Render Material 3-styled log panel
///
/// Tonal surface container with M3 typography.
pub fn render_material3_log_panel(logs: &[String]) -> impl IntoElement {
    div().w_full().h(px(160.)).px(px(16.)).pb(px(16.)).child(
        div()
            .w_full()
            .h_full()
            .p(px(16.))
            .bg(rgb(colors::SURFACE_CONTAINER_HIGH))
            .rounded(px(corners::MD))
            .flex()
            .flex_col()
            .child(
                div()
                    .text_size(px(12.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(colors::ON_SURFACE_VARIANT))
                    .mb(px(12.))
                    .child("Console Output"),
            )
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .font_family("Roboto Mono")
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .children(logs.iter().map(|log| {
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(colors::ON_SURFACE_VARIANT))
                            .child(log.clone())
                    })),
            ),
    )
}

/// Render Material 3-styled window container
///
/// M3 surface with proper elevation and shape.
pub fn render_material3_window_container(children: impl IntoElement) -> impl IntoElement {
    div()
        .w_full()
        .h_full()
        .bg(rgb(colors::SURFACE))
        .rounded(px(corners::LG))
        .overflow_hidden()
        .shadow(vec![
            BoxShadow {
                color: hsla(0., 0., 0., 0.15),
                offset: point(px(0.), px(elevation::LEVEL2)),
                blur_radius: px(8.),
                spread_radius: px(0.),
            },
            BoxShadow {
                color: hsla(0., 0., 0., 0.08),
                offset: point(px(0.), px(1.)),
                blur_radius: px(3.),
                spread_radius: px(0.),
            },
        ])
        .child(children)
}

// Note: Tests omitted due to GPUI macro recursion limit issues.
// Material 3 colors (M3 palette):
// - SURFACE: 0xfef7ff
// - SURFACE_CONTAINER: 0xece6f0
// - PRIMARY: 0x6750a4
// - ON_SURFACE: 0x1d1b20
// Corner radius tokens: MD = 12.0, XL = 28.0
