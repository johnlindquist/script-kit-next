//! Header Raycast Style Variations - Separator & Spacing Explorations
//!
//! 20 variations of the Raycast-style header layout exploring:
//! - Separator styles (|, dots, spacing, none)
//! - Element spacing (tight, normal, loose)
//! - Logo placement and styling
//! - Button groupings
//!
//! All variations use the same theme colors and fonts.
//! The logo is rendered as dark inside a yellow rounded rectangle.

use gpui::*;

use crate::components::PromptHeaderColors;
use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

/// Story showcasing 20 Raycast-style header variations
pub struct HeaderRaycastVariationsStory;

impl Story for HeaderRaycastVariationsStory {
    fn id(&self) -> &'static str {
        "header-raycast-variations"
    }

    fn name(&self) -> &'static str {
        "Header Raycast Variations"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = PromptHeaderColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Separator Styles (1-5)")
                    .child(variation_item(
                        "1. Pipe Separators (Current)",
                        render_v1_pipe_separators(colors),
                    ))
                    .child(variation_item(
                        "2. No Separators",
                        render_v2_no_separators(colors),
                    ))
                    .child(variation_item(
                        "3. Dot Separators",
                        render_v3_dot_separators(colors),
                    ))
                    .child(variation_item(
                        "4. Slash Separators",
                        render_v4_slash_separators(colors),
                    ))
                    .child(variation_item(
                        "5. Thin Line Separators",
                        render_v5_line_separators(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Spacing Variations (6-10)")
                    .child(variation_item(
                        "6. Tight Spacing",
                        render_v6_tight_spacing(colors),
                    ))
                    .child(variation_item(
                        "7. Loose Spacing",
                        render_v7_loose_spacing(colors),
                    ))
                    .child(variation_item(
                        "8. Grouped Buttons",
                        render_v8_grouped_buttons(colors),
                    ))
                    .child(variation_item(
                        "9. Spread Layout",
                        render_v9_spread_layout(colors),
                    ))
                    .child(variation_item(
                        "10. Compact All",
                        render_v10_compact_all(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Logo Variations (11-15)")
                    .child(variation_item(
                        "11. Logo Left",
                        render_v11_logo_left(colors),
                    ))
                    .child(variation_item(
                        "12. Logo Right",
                        render_v12_logo_right(colors),
                    ))
                    .child(variation_item(
                        "13. Logo Larger",
                        render_v13_logo_larger(colors),
                    ))
                    .child(variation_item(
                        "14. Logo with Border",
                        render_v14_logo_border(colors),
                    ))
                    .child(variation_item(
                        "15. Logo Circular",
                        render_v15_logo_circular(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Button Styles (16-20)")
                    .child(variation_item(
                        "16. Pill Buttons",
                        render_v16_pill_buttons(colors),
                    ))
                    .child(variation_item(
                        "17. Ghost Buttons",
                        render_v17_ghost_buttons(colors),
                    ))
                    .child(variation_item(
                        "18. Icon Only",
                        render_v18_icon_only(colors),
                    ))
                    .child(variation_item(
                        "19. Text Only",
                        render_v19_text_only(colors),
                    ))
                    .child(variation_item(
                        "20. Badge Style",
                        render_v20_badge_style(colors),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "separator-pipe".into(),
                description: Some("1. Pipe Separators".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "separator-none".into(),
                description: Some("2. No Separators".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "separator-dot".into(),
                description: Some("3. Dot Separators".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "separator-slash".into(),
                description: Some("4. Slash Separators".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "separator-line".into(),
                description: Some("5. Line Separators".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "spacing-tight".into(),
                description: Some("6. Tight Spacing".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "spacing-loose".into(),
                description: Some("7. Loose Spacing".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "spacing-grouped".into(),
                description: Some("8. Grouped Buttons".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "spacing-spread".into(),
                description: Some("9. Spread Layout".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "spacing-compact".into(),
                description: Some("10. Compact All".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-left".into(),
                description: Some("11. Logo Left".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-right".into(),
                description: Some("12. Logo Right".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-larger".into(),
                description: Some("13. Logo Larger".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-border".into(),
                description: Some("14. Logo with Border".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "logo-circular".into(),
                description: Some("15. Logo Circular".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "buttons-pill".into(),
                description: Some("16. Pill Buttons".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "buttons-ghost".into(),
                description: Some("17. Ghost Buttons".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "buttons-icon".into(),
                description: Some("18. Icon Only".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "buttons-text".into(),
                description: Some("19. Text Only".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "buttons-badge".into(),
                description: Some("20. Badge Style".into()),
                ..Default::default()
            },
        ]
    }
}

/// Wrapper for variation items with label
fn variation_item(label: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .mb_4()
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x888888))
                .child(SharedString::from(label.to_string())),
        )
        .child(content)
}

/// Reusable header container with consistent styling
fn header_container(colors: PromptHeaderColors) -> Div {
    div()
        .w_full()
        .h(px(52.))
        .px_4()
        .flex()
        .flex_row()
        .items_center()
        .bg(colors.background.to_rgb())
        .border_b_1()
        .border_color(colors.border.to_rgb())
}

/// Script Kit logo as dark icon inside yellow rounded rectangle
fn logo_component(size: f32, corner_radius: f32) -> impl IntoElement {
    div()
        .w(px(size))
        .h(px(size))
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0xffc821)) // Script Kit yellow
        .rounded(px(corner_radius))
        .child(
            svg()
                .path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
                .size(px(size * 0.6))
                .text_color(rgb(0x1a1a1a)), // Dark logo inside
        )
}

/// "Script Kit" text label
fn script_kit_label(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .text_base()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(colors.text_primary.to_rgb())
        .child("Script Kit")
}

/// Ask AI hint with Tab badge
fn ask_ai_hint(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child("Ask AI"),
        )
        .child(
            div()
                .px(px(6.))
                .py(px(2.))
                .bg(colors.search_box_bg.to_rgb())
                .rounded(px(4.))
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("Tab"),
        )
}

/// Run button with Enter hint
fn run_button(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors.accent.to_rgb())
                .child("Run"),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child("↵"),
        )
}

/// Actions button with shortcut
fn actions_button(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child("Actions"),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("⌘K"),
        )
}

/// Pipe separator
fn pipe_sep(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .text_color(colors.text_dimmed.to_rgb())
        .mx_2()
        .child("|")
}

/// Dot separator
fn dot_sep(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .text_color(colors.text_dimmed.to_rgb())
        .mx_2()
        .child("·")
}

/// Slash separator
fn slash_sep(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .text_color(colors.text_dimmed.to_rgb())
        .mx_2()
        .child("/")
}

/// Thin line separator
fn line_sep(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w(px(1.))
        .h(px(20.))
        .mx_2()
        .bg(colors.text_dimmed.to_rgb())
}

// =============================================================================
// SEPARATOR STYLES (1-5)
// =============================================================================

/// 1. Pipe Separators (Current style)
fn render_v1_pipe_separators(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(pipe_sep(colors))
        .child(run_button(colors))
        .child(pipe_sep(colors))
        .child(actions_button(colors))
        .child(pipe_sep(colors))
        .child(logo_component(28., 6.))
}

/// 2. No Separators
fn render_v2_no_separators(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_component(28., 6.))
}

/// 3. Dot Separators
fn render_v3_dot_separators(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(dot_sep(colors))
        .child(run_button(colors))
        .child(dot_sep(colors))
        .child(actions_button(colors))
        .child(dot_sep(colors))
        .child(logo_component(28., 6.))
}

/// 4. Slash Separators
fn render_v4_slash_separators(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(slash_sep(colors))
        .child(run_button(colors))
        .child(slash_sep(colors))
        .child(actions_button(colors))
        .child(slash_sep(colors))
        .child(logo_component(28., 6.))
}

/// 5. Thin Line Separators
fn render_v5_line_separators(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(line_sep(colors))
        .child(run_button(colors))
        .child(line_sep(colors))
        .child(actions_button(colors))
        .child(line_sep(colors))
        .child(logo_component(28., 6.))
}

// =============================================================================
// SPACING VARIATIONS (6-10)
// =============================================================================

/// 6. Tight Spacing
fn render_v6_tight_spacing(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .gap_1()
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .mx_1()
                .child("|"),
        )
        .child(run_button(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .mx_1()
                .child("|"),
        )
        .child(actions_button(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .mx_1()
                .child("|"),
        )
        .child(logo_component(24., 4.))
}

/// 7. Loose Spacing
fn render_v7_loose_spacing(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .gap_4()
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .mx_4()
                .child("|"),
        )
        .child(run_button(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .mx_4()
                .child("|"),
        )
        .child(actions_button(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .mx_4()
                .child("|"),
        )
        .child(logo_component(32., 8.))
}

/// 8. Grouped Buttons
fn render_v8_grouped_buttons(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(24.)))
        .child(
            // Button group with background
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .px_2()
                .py_1()
                .bg(rgba(0xffffff10))
                .rounded(px(6.))
                .child(run_button(colors))
                .child(div().w(px(1.)).h(px(16.)).bg(colors.text_dimmed.to_rgb()))
                .child(actions_button(colors)),
        )
        .child(div().w(px(12.)))
        .child(logo_component(28., 6.))
}

/// 9. Spread Layout
fn render_v9_spread_layout(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .justify_between()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(logo_component(28., 6.))
                .child(script_kit_label(colors)),
        )
        .child(ask_ai_hint(colors))
        .child(run_button(colors))
        .child(actions_button(colors))
}

/// 10. Compact All
fn render_v10_compact_all(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(40.))
        .px_3()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .bg(colors.background.to_rgb())
        .border_b_1()
        .border_color(colors.border.to_rgb())
        .child(logo_component(22., 4.))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors.text_primary.to_rgb())
                .child("Script Kit"),
        )
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("AI"),
        )
        .child(
            div()
                .px(px(4.))
                .py(px(1.))
                .bg(colors.search_box_bg.to_rgb())
                .rounded(px(3.))
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("⇥"),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.accent.to_rgb())
                .child("↵"),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("⌘K"),
        )
}

// =============================================================================
// LOGO VARIATIONS (11-15)
// =============================================================================

/// 11. Logo Left
fn render_v11_logo_left(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(logo_component(28., 6.))
        .child(div().w(px(12.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(pipe_sep(colors))
        .child(run_button(colors))
        .child(pipe_sep(colors))
        .child(actions_button(colors))
}

/// 12. Logo Right (far right)
fn render_v12_logo_right(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(pipe_sep(colors))
        .child(run_button(colors))
        .child(pipe_sep(colors))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_component(28., 6.))
}

/// 13. Logo Larger
fn render_v13_logo_larger(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(pipe_sep(colors))
        .child(run_button(colors))
        .child(pipe_sep(colors))
        .child(actions_button(colors))
        .child(pipe_sep(colors))
        .child(logo_component(36., 8.))
}

/// 14. Logo with Border
fn render_v14_logo_border(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(pipe_sep(colors))
        .child(run_button(colors))
        .child(pipe_sep(colors))
        .child(actions_button(colors))
        .child(pipe_sep(colors))
        .child(
            div()
                .w(px(32.))
                .h(px(32.))
                .flex()
                .items_center()
                .justify_center()
                .border_2()
                .border_color(rgb(0xffc821))
                .rounded(px(8.))
                .child(
                    div()
                        .w(px(24.))
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(rgb(0xffc821))
                        .rounded(px(4.))
                        .child(
                            svg()
                                .path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
                                .size(px(14.))
                                .text_color(rgb(0x1a1a1a)),
                        ),
                ),
        )
}

/// 15. Logo Circular
fn render_v15_logo_circular(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(pipe_sep(colors))
        .child(run_button(colors))
        .child(pipe_sep(colors))
        .child(actions_button(colors))
        .child(pipe_sep(colors))
        .child(
            div()
                .w(px(28.))
                .h(px(28.))
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(0xffc821))
                .rounded_full()
                .child(
                    svg()
                        .path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
                        .size(px(16.))
                        .text_color(rgb(0x1a1a1a)),
                ),
        )
}

// =============================================================================
// BUTTON STYLES (16-20)
// =============================================================================

/// 16. Pill Buttons
fn render_v16_pill_buttons(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .px_3()
                .py(px(6.))
                .bg(rgba(0xffffff10))
                .rounded_full()
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Ask AI"),
                )
                .child(
                    div()
                        .px(px(6.))
                        .py(px(2.))
                        .bg(colors.search_box_bg.to_rgb())
                        .rounded_full()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Tab"),
                ),
        )
        .child(div().w(px(8.)))
        .child(
            div()
                .px_3()
                .py(px(6.))
                .bg(colors.accent.to_rgb())
                .rounded_full()
                .text_sm()
                .text_color(rgb(0x000000))
                .font_weight(FontWeight::MEDIUM)
                .child("Run ↵"),
        )
        .child(div().w(px(8.)))
        .child(
            div()
                .px_3()
                .py(px(6.))
                .bg(rgba(0xffffff10))
                .rounded_full()
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child("⌘K"),
        )
        .child(div().w(px(8.)))
        .child(logo_component(28., 6.))
}

/// 17. Ghost Buttons (transparent hover)
fn render_v17_ghost_buttons(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .px_2()
                .py_1()
                .rounded(px(4.))
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child("Ask AI ⇥"),
        )
        .child(
            div()
                .px_2()
                .py_1()
                .rounded(px(4.))
                .text_sm()
                .text_color(colors.accent.to_rgb())
                .font_weight(FontWeight::MEDIUM)
                .child("Run ↵"),
        )
        .child(
            div()
                .px_2()
                .py_1()
                .rounded(px(4.))
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child("Actions ⌘K"),
        )
        .child(div().w(px(8.)))
        .child(logo_component(28., 6.))
}

/// 18. Icon Only
fn render_v18_icon_only(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(logo_component(28., 6.))
        .child(div().w(px(12.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .w(px(32.))
                .h(px(32.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(6.))
                .bg(rgba(0xffffff08))
                .text_color(colors.text_muted.to_rgb())
                .child("✨"), // AI icon
        )
        .child(div().w(px(8.)))
        .child(
            div()
                .w(px(32.))
                .h(px(32.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(6.))
                .bg(colors.accent.to_rgb())
                .text_color(rgb(0x000000))
                .child("▶"),
        )
        .child(div().w(px(8.)))
        .child(
            div()
                .w(px(32.))
                .h(px(32.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(6.))
                .bg(rgba(0xffffff08))
                .text_color(colors.text_muted.to_rgb())
                .child("⋯"),
        )
}

/// 19. Text Only (no icons, minimal)
fn render_v19_text_only(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0xffc821))
                .child("⌘"),
        )
        .child(div().w(px(8.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .text_sm()
                .text_color(colors.text_dimmed.to_rgb())
                .child("Tab for AI"),
        )
        .child(div().w(px(16.)))
        .child(
            div()
                .text_sm()
                .text_color(colors.accent.to_rgb())
                .font_weight(FontWeight::MEDIUM)
                .child("Enter to Run"),
        )
        .child(div().w(px(16.)))
        .child(
            div()
                .text_sm()
                .text_color(colors.text_dimmed.to_rgb())
                .child("⌘K Actions"),
        )
}

/// 20. Badge Style
fn render_v20_badge_style(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(logo_component(28., 6.))
        .child(div().w(px(12.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            // Badge-style buttons
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .px_2()
                        .py(px(4.))
                        .bg(rgba(0x6366f120))
                        .border_1()
                        .border_color(rgba(0x6366f140))
                        .rounded(px(4.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(div().text_xs().text_color(rgb(0x818cf8)).child("AI"))
                        .child(
                            div()
                                .px(px(4.))
                                .py(px(1.))
                                .bg(rgba(0x6366f130))
                                .rounded(px(2.))
                                .text_xs()
                                .text_color(rgb(0x818cf8))
                                .child("⇥"),
                        ),
                )
                .child(
                    div()
                        .px_2()
                        .py(px(4.))
                        .bg(rgba(0x22c55e20))
                        .border_1()
                        .border_color(rgba(0x22c55e40))
                        .rounded(px(4.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(div().text_xs().text_color(rgb(0x4ade80)).child("Run"))
                        .child(
                            div()
                                .px(px(4.))
                                .py(px(1.))
                                .bg(rgba(0x22c55e30))
                                .rounded(px(2.))
                                .text_xs()
                                .text_color(rgb(0x4ade80))
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .px_2()
                        .py(px(4.))
                        .bg(rgba(0xffffff10))
                        .border_1()
                        .border_color(rgba(0xffffff20))
                        .rounded(px(4.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("More"),
                        )
                        .child(
                            div()
                                .px(px(4.))
                                .py(px(1.))
                                .bg(rgba(0xffffff10))
                                .rounded(px(2.))
                                .text_xs()
                                .text_color(colors.text_dimmed.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}
