//! Header Logo Variations
//!
//! 20 variations exploring logo size and placement in the header.
//! Uses the "No Separators" style with golden ratio logo as baseline.
//!
//! Variations explore:
//! - Logo sizes (container and SVG dimensions)
//! - Logo placement (left, right, after title)
//! - Spacing between logo and other elements
//! - Corner radius options

use gpui::*;

use crate::components::PromptHeaderColors;
use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

/// Story showcasing 20 header logo variations
pub struct HeaderLogoVariationsStory;

impl Story for HeaderLogoVariationsStory {
    fn id(&self) -> &'static str {
        "header-logo-variations"
    }

    fn name(&self) -> &'static str {
        "Header Logo Variations"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = PromptHeaderColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Logo Size Variations (1-5)")
                    .child(variation_item(
                        "1. 18px container / 11px SVG",
                        render_v1(colors),
                    ))
                    .child(variation_item(
                        "2. 19px container / 12px SVG",
                        render_v2(colors),
                    ))
                    .child(variation_item(
                        "3. 20px container / 12px SVG",
                        render_v3(colors),
                    ))
                    .child(variation_item(
                        "4. 21px container / 13px SVG (Golden)",
                        render_v4(colors),
                    ))
                    .child(variation_item(
                        "5. 22px container / 14px SVG",
                        render_v5(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Logo Placement - Right Side (6-10)")
                    .child(variation_item("6. Logo far right", render_v6(colors)))
                    .child(variation_item("7. Logo before actions", render_v7(colors)))
                    .child(variation_item("8. Logo after Run", render_v8(colors)))
                    .child(variation_item("9. Logo with 8px gap", render_v9(colors)))
                    .child(variation_item("10. Logo with 16px gap", render_v10(colors))),
            )
            .child(story_divider())
            .child(
                story_section("Logo Placement - Left Side (11-15)")
                    .child(variation_item("11. Logo before title", render_v11(colors)))
                    .child(variation_item(
                        "12. Logo 8px from title",
                        render_v12(colors),
                    ))
                    .child(variation_item(
                        "13. Logo 12px from title",
                        render_v13(colors),
                    ))
                    .child(variation_item("14. Logo flush left", render_v14(colors)))
                    .child(variation_item(
                        "15. Logo with border left",
                        render_v15(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Corner Radius & Style (16-20)")
                    .child(variation_item(
                        "16. Sharp corners (2px)",
                        render_v16(colors),
                    ))
                    .child(variation_item(
                        "17. Medium corners (4px)",
                        render_v17(colors),
                    ))
                    .child(variation_item(
                        "18. Round corners (6px)",
                        render_v18(colors),
                    ))
                    .child(variation_item("19. Circular logo", render_v19(colors)))
                    .child(variation_item(
                        "20. Squircle style (8px)",
                        render_v20(colors),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "size-18".into(),
                description: Some("18px container".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "size-19".into(),
                description: Some("19px container".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "size-20".into(),
                description: Some("20px container".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "size-21-golden".into(),
                description: Some("21px golden ratio".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "size-22".into(),
                description: Some("22px container".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "right-far".into(),
                description: Some("Logo far right".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "right-before-actions".into(),
                description: Some("Before actions".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "right-after-run".into(),
                description: Some("After Run".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "right-8px-gap".into(),
                description: Some("8px gap".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "right-16px-gap".into(),
                description: Some("16px gap".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "left-before-title".into(),
                description: Some("Before title".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "left-8px".into(),
                description: Some("8px from title".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "left-12px".into(),
                description: Some("12px from title".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "left-flush".into(),
                description: Some("Flush left".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "left-border".into(),
                description: Some("With border".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "corner-2px".into(),
                description: Some("Sharp 2px".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "corner-4px".into(),
                description: Some("Medium 4px".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "corner-6px".into(),
                description: Some("Round 6px".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "circular".into(),
                description: Some("Circular".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "squircle".into(),
                description: Some("Squircle 8px".into()),
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

/// Reusable header container with consistent styling (no separators)
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

/// Golden ratio logo (21px container, 13px SVG, 4px radius)
fn golden_logo() -> impl IntoElement {
    logo_box(21., 4., 13.)
}

/// Logo with custom dimensions
fn logo_box(container_size: f32, corner_radius: f32, svg_size: f32) -> impl IntoElement {
    div()
        .w(px(container_size))
        .h(px(container_size))
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0xffc821))
        .rounded(px(corner_radius))
        .child(
            svg()
                .external_path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
                .size(px(svg_size))
                .text_color(rgb(0x000000)),
        )
}

/// Circular logo
fn circular_logo(size: f32, svg_size: f32) -> impl IntoElement {
    div()
        .w(px(size))
        .h(px(size))
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0xffc821))
        .rounded_full()
        .child(
            svg()
                .external_path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
                .size(px(svg_size))
                .text_color(rgb(0x000000)),
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

/// Ask AI hint with Tab badge (yellow text, grey shortcut)
fn ask_ai_hint(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
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
                .bg(colors.search_box_bg.to_rgb())
                .rounded(px(4.))
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("Tab"),
        )
}

/// Run button (yellow text, grey shortcut)
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

/// Actions button (yellow text, grey shortcut)
fn actions_button(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_sm()
                .text_color(colors.accent.to_rgb())
                .child("Actions"),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("⌘K"),
        )
}

// =============================================================================
// LOGO SIZE VARIATIONS (1-5)
// =============================================================================

/// 1. 18px container / 11px SVG (compact)
fn render_v1(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(18., 4., 11.))
}

/// 2. 19px container / 12px SVG
fn render_v2(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(19., 4., 12.))
}

/// 3. 20px container / 12px SVG
fn render_v3(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(20., 4., 12.))
}

/// 4. 21px container / 13px SVG (Golden ratio - baseline)
fn render_v4(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(golden_logo())
}

/// 5. 22px container / 14px SVG
fn render_v5(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(22., 4., 14.))
}

// =============================================================================
// LOGO PLACEMENT - RIGHT SIDE (6-10)
// =============================================================================

/// 6. Logo far right (after all elements)
fn render_v6(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(24.)))
        .child(golden_logo())
}

/// 7. Logo before actions
fn render_v7(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(golden_logo())
        .child(div().w(px(16.)))
        .child(actions_button(colors))
}

/// 8. Logo after Run (between Run and Actions)
fn render_v8(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(12.)))
        .child(golden_logo())
        .child(div().w(px(12.)))
        .child(actions_button(colors))
}

/// 9. Logo with 8px gap from actions
fn render_v9(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(golden_logo())
}

/// 10. Logo with 16px gap from actions
fn render_v10(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(golden_logo())
}

// =============================================================================
// LOGO PLACEMENT - LEFT SIDE (11-15)
// =============================================================================

/// 11. Logo before title (logo first)
fn render_v11(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(golden_logo())
        .child(div().w(px(10.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
}

/// 12. Logo 8px from title
fn render_v12(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(golden_logo())
        .child(div().w(px(8.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
}

/// 13. Logo 12px from title
fn render_v13(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(golden_logo())
        .child(div().w(px(12.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
}

/// 14. Logo flush left (minimal gap)
fn render_v14(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(golden_logo())
        .child(div().w(px(6.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
}

/// 15. Logo with subtle left border/separator
fn render_v15(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(golden_logo())
        .child(div().w(px(12.)))
        .child(div().w(px(1.)).h(px(20.)).bg(colors.border.to_rgb()))
        .child(div().w(px(12.)))
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
}

// =============================================================================
// CORNER RADIUS & STYLE (16-20)
// =============================================================================

/// 16. Sharp corners (2px radius)
fn render_v16(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(21., 2., 13.))
}

/// 17. Medium corners (4px radius) - same as golden
fn render_v17(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(21., 4., 13.))
}

/// 18. Round corners (6px radius)
fn render_v18(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(21., 6., 13.))
}

/// 19. Circular logo
fn render_v19(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(circular_logo(21., 13.))
}

/// 20. Squircle style (8px radius for smoother corners)
fn render_v20(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_hint(colors))
        .child(div().w(px(16.)))
        .child(run_button(colors))
        .child(div().w(px(16.)))
        .child(actions_button(colors))
        .child(div().w(px(16.)))
        .child(logo_box(21., 8., 13.))
}
