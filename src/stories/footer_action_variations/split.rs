//! Footer Action Variations - Script Kit branded footer layouts
//!
//! Based on Raycast's design pattern:
//! - Header: Clean input area + Ask AI (minimal)
//! - Footer: Logo left, contextual action + Actions right
//!
//! This story explores 10 variations of the footer design.

use gpui::*;

use crate::storybook::{story_container, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;
use crate::utils;

// --- merged from part_01.rs ---
pub struct FooterActionVariationsStory;

impl Story for FooterActionVariationsStory {
    fn id(&self) -> &'static str {
        "footer-action-variations"
    }

    fn name(&self) -> &'static str {
        "Footer Actions (10 variations)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = FooterColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Footer Action Variations")
                    .child(variation_item(
                        "1. Base - Logo left, Run + Actions right",
                        full_layout(colors, footer_base(colors)),
                    ))
                    .child(variation_item(
                        "2. No divider between actions",
                        full_layout(colors, footer_no_divider(colors)),
                    ))
                    .child(variation_item(
                        "3. Compact - smaller text",
                        full_layout(colors, footer_compact(colors)),
                    ))
                    .child(variation_item(
                        "4. With item count",
                        full_layout(colors, footer_with_count(colors)),
                    ))
                    .child(variation_item(
                        "5. Selected item preview",
                        full_layout(colors, footer_with_preview(colors)),
                    ))
                    .child(variation_item(
                        "6. Keyboard hints prominent",
                        full_layout(colors, footer_kbd_prominent(colors)),
                    ))
                    .child(variation_item(
                        "7. Icon-style Run button",
                        full_layout(colors, footer_icon_run(colors)),
                    ))
                    .child(variation_item(
                        "8. Ghost button style",
                        full_layout(colors, footer_ghost_buttons(colors)),
                    ))
                    .child(variation_item(
                        "9. Primary Run + ghost Actions",
                        full_layout(colors, footer_primary_run(colors)),
                    ))
                    .child(variation_item(
                        "10. Taller footer with more spacing",
                        full_layout(colors, footer_tall(colors)),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![StoryVariant {
            name: "base".into(),
            description: Some("Base footer layout".into()),
            ..Default::default()
        }]
    }
}

// =============================================================================
// TYPES
// =============================================================================

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct FooterColors {
    background: u32,
    background_elevated: u32,
    text_primary: u32,
    text_secondary: u32,
    text_muted: u32,
    accent: u32,
    border: u32,
}

impl FooterColors {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.colors.background.main,
            background_elevated: theme.colors.background.title_bar,
            text_primary: theme.colors.text.primary,
            text_secondary: theme.colors.text.muted,
            text_muted: theme.colors.text.dimmed,
            accent: theme.colors.accent.selected,
            border: theme.colors.ui.border,
        }
    }
}

// =============================================================================
// HELPERS
// =============================================================================

fn variation_item(label: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .mb_4()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
        .child(content)
}

/// Window container with header, list preview, and footer
fn full_layout(colors: FooterColors, footer: impl IntoElement) -> impl IntoElement {
    div()
        .w_full()
        .h(px(280.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        // Header
        .child(header_clean(colors))
        // Divider
        .child(
            div()
                .mx(px(16.))
                .h(px(1.))
                .bg(rgba((colors.border << 8) | 0x40)),
        )
        // List preview (simplified)
        .child(list_preview(colors))
        // Footer
        .child(footer)
}

/// Clean header - just input and Ask AI
fn header_clean(colors: FooterColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x20;
    let tab_bg = (colors.border << 8) | 0x40;

    div()
        .w_full()
        .px(px(16.))
        .py(px(10.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        // Input
        .child(
            div()
                .flex_1()
                .text_base()
                .text_color(colors.text_primary.to_rgb())
                .child("clipboard"),
        )
        // Ask AI button
        .child(
            div()
                .id("ask-ai")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .px(px(8.))
                .py(px(4.))
                .rounded(px(6.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.accent.to_rgb())
                        .child("Ask AI"),
                )
                .child(
                    div()
                        .px(px(6.))
                        .py(px(2.))
                        .bg(rgba(tab_bg))
                        .rounded(px(4.))
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Tab"),
                ),
        )
}

/// Simplified list preview
fn list_preview(colors: FooterColors) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .px(px(8.))
        .py(px(4.))
        // Selected item
        .child(
            div()
                .w_full()
                .px(px(8.))
                .py(px(8.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(10.))
                .bg(rgba((colors.accent << 8) | 0x12))
                .rounded(px(6.))
                .child(
                    div()
                        .w(px(24.))
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(colors.accent.to_rgb())
                        .rounded(px(5.))
                        .child(div().text_sm().text_color(rgb(0x000000)).child("📋")),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(colors.text_primary.to_rgb())
                                .child("Clipboard History"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_secondary.to_rgb())
                                .child("View and manage clipboard"),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Built-in"),
                ),
        )
        // Another item (unselected)
        .child(
            div()
                .w_full()
                .px(px(8.))
                .py(px(8.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(10.))
                .child(
                    div()
                        .w(px(24.))
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(colors.accent.to_rgb())
                        .rounded(px(5.))
                        .child(div().text_sm().text_color(rgb(0x000000)).child("🔍")),
                )
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(colors.text_primary.to_rgb())
                        .child("Search Files"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Script"),
                ),
        )
}

/// Logo component using actual SVG
fn logo_component(_colors: FooterColors, size: f32) -> impl IntoElement {
    div()
        .w(px(size))
        .h(px(size))
        .flex()
        .items_center()
        .justify_center()
        // .bg(rgba((colors.accent << 8) | 0xD9)) // 85% opacity
        // .rounded(px(4.))
        .child(
            svg()
                .external_path(utils::get_logo_path())
                .size(px(size * 0.65))
                .text_color(rgb(0x000000)),
        )
}

// --- merged from part_02.rs ---

// =============================================================================
// FOOTER VARIATIONS
// =============================================================================

/// 1. Base footer - Logo left, Run Script ↵ | Actions ⌘K right
fn footer_base(colors: FooterColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        // Left: Logo
        .child(logo_component(colors, 20.))
        // Right: Actions
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                // Run Script ↵
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("Run Script"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("↵"),
                        ),
                )
                // Divider
                .child(
                    div()
                        .w(px(1.))
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                // Actions ⌘K
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}

/// 2. No divider between actions
fn footer_no_divider(colors: FooterColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .child(logo_component(colors, 20.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(16.)) // Larger gap instead of divider
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("Run Script"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}

/// 3. Compact - smaller text
fn footer_compact(colors: FooterColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(32.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .child(logo_component(colors, 16.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("Run"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(12.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}

/// 4. With item count on left
fn footer_with_count(colors: FooterColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        // Left: Logo + count
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(10.))
                .child(logo_component(colors, 20.))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("42 items"),
                ),
        )
        // Right: Actions
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("Run Script"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}

/// 5. Selected item preview in footer
fn footer_with_preview(colors: FooterColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        // Left: Logo + selected item
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(10.))
                .child(logo_component(colors, 20.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(div().text_sm().text_color(rgb(0x888888)).child("📋"))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_secondary.to_rgb())
                                .child("Clipboard History"),
                        ),
                ),
        )
        // Right: Actions
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("Run"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}

// --- merged from part_03.rs ---

/// 6. Keyboard hints prominent (badges)
fn footer_kbd_prominent(colors: FooterColors) -> impl IntoElement {
    let badge_bg = (colors.border << 8) | 0x50;

    div()
        .w_full()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .child(logo_component(colors, 20.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("Run Script"),
                        )
                        .child(
                            div()
                                .px(px(6.))
                                .py(px(2.))
                                .bg(rgba(badge_bg))
                                .rounded(px(4.))
                                .text_xs()
                                .text_color(colors.text_secondary.to_rgb())
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(2.))
                                .child(
                                    div()
                                        .px(px(4.))
                                        .py(px(2.))
                                        .bg(rgba(badge_bg))
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(colors.text_secondary.to_rgb())
                                        .child("⌘"),
                                )
                                .child(
                                    div()
                                        .px(px(4.))
                                        .py(px(2.))
                                        .bg(rgba(badge_bg))
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(colors.text_secondary.to_rgb())
                                        .child("K"),
                                ),
                        ),
                ),
        )
}

/// 7. Icon-style Run button
fn footer_icon_run(colors: FooterColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x20;

    div()
        .w_full()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .child(logo_component(colors, 20.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                // Play icon button
                .child(
                    div()
                        .id("icon-run")
                        .w(px(28.))
                        .h(px(28.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(6.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_base()
                                .text_color(colors.accent.to_rgb())
                                .child("▶"),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                // Actions text
                .child(
                    div()
                        .id("actions-text")
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .px(px(6.))
                        .py(px(4.))
                        .rounded(px(6.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}

/// 8. Ghost button style
fn footer_ghost_buttons(colors: FooterColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x20;
    let border_color = (colors.accent << 8) | 0x40;

    div()
        .w_full()
        .h(px(44.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .child(logo_component(colors, 20.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                // Run button (ghost)
                .child(
                    div()
                        .id("ghost-run")
                        .px(px(10.))
                        .py(px(5.))
                        .rounded(px(6.))
                        .border_1()
                        .border_color(rgba(border_color))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(6.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.accent.to_rgb())
                                        .child("Run Script"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("↵"),
                                ),
                        ),
                )
                // Actions button (ghost, no border)
                .child(
                    div()
                        .id("ghost-actions")
                        .px(px(10.))
                        .py(px(5.))
                        .rounded(px(6.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.text_secondary.to_rgb())
                                        .child("Actions"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("⌘K"),
                                ),
                        ),
                ),
        )
}

/// 9. Primary Run + ghost Actions
fn footer_primary_run(colors: FooterColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x20;

    div()
        .w_full()
        .h(px(44.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .child(logo_component(colors, 20.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                // Actions button (ghost)
                .child(
                    div()
                        .id("secondary-actions")
                        .px(px(10.))
                        .py(px(5.))
                        .rounded(px(6.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.text_secondary.to_rgb())
                                        .child("Actions"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("⌘K"),
                                ),
                        ),
                )
                // Run button (primary)
                .child(
                    div()
                        .id("primary-run")
                        .px(px(12.))
                        .py(px(5.))
                        .bg(colors.accent.to_rgb())
                        .rounded(px(6.))
                        .cursor_pointer()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(6.))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0x000000))
                                        .child("Run Script"),
                                )
                                .child(div().text_xs().text_color(rgba(0x00000080)).child("↵")),
                        ),
                ),
        )
}

// --- merged from part_04.rs ---

/// 10. Taller footer with more spacing
fn footer_tall(colors: FooterColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(52.))
        .px(px(16.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        // Left: Logo with breathing room
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(12.))
                .child(logo_component(colors, 24.))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Script Kit"),
                ),
        )
        // Right: Spacious actions
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(12.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(colors.accent.to_rgb())
                                .child("Run Script"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(20.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}
