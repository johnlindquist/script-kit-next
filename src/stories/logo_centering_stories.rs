//! Logo Centering Variations
//!
//! 20 variations exploring different approaches to perfectly center
//! the Script Kit logo inside a yellow rounded rectangle.
//!
//! Variations explore:
//! - Container sizes (smaller rectangles)
//! - SVG sizing ratios
//! - Flexbox alignment options
//! - Padding approaches
//! - Offset corrections

use gpui::*;

use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};

/// Story showcasing 20 logo centering variations
pub struct LogoCenteringStory;

impl Story for LogoCenteringStory {
    fn id(&self) -> &'static str {
        "logo-centering"
    }

    fn name(&self) -> &'static str {
        "Logo Centering"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        story_container()
            .child(
                story_section("Container Size Variations (1-5)").child(variation_row(vec![
                    ("1. 24px container", render_v1().into_any_element()),
                    ("2. 22px container", render_v2().into_any_element()),
                    ("3. 20px container", render_v3().into_any_element()),
                    ("4. 18px container", render_v4().into_any_element()),
                    ("5. 16px container", render_v5().into_any_element()),
                ])),
            )
            .child(story_divider())
            .child(
                story_section("SVG Size Ratios (6-10)").child(variation_row(vec![
                    ("6. 60% ratio", render_v6().into_any_element()),
                    ("7. 65% ratio", render_v7().into_any_element()),
                    ("8. 70% ratio", render_v8().into_any_element()),
                    ("9. 75% ratio", render_v9().into_any_element()),
                    ("10. 80% ratio", render_v10().into_any_element()),
                ])),
            )
            .child(story_divider())
            .child(
                story_section("Fixed SVG Sizes (11-15)").child(variation_row(vec![
                    ("11. 12px SVG", render_v11().into_any_element()),
                    ("12. 13px SVG", render_v12().into_any_element()),
                    ("13. 14px SVG", render_v13().into_any_element()),
                    ("14. 15px SVG", render_v14().into_any_element()),
                    ("15. 16px SVG", render_v15().into_any_element()),
                ])),
            )
            .child(story_divider())
            .child(
                story_section("Alignment Tweaks (16-20)").child(variation_row(vec![
                    ("16. Negative margin", render_v16().into_any_element()),
                    ("17. Padding offset", render_v17().into_any_element()),
                    ("18. Line height trick", render_v18().into_any_element()),
                    ("19. Absolute position", render_v19().into_any_element()),
                    ("20. Golden ratio", render_v20().into_any_element()),
                ])),
            )
            .child(story_divider())
            .child(story_section("Side-by-Side Comparison").child(comparison_row()))
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "container-24".into(),
                description: Some("24px container".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "container-22".into(),
                description: Some("22px container".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "container-20".into(),
                description: Some("20px container".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "container-18".into(),
                description: Some("18px container".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "container-16".into(),
                description: Some("16px container".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "ratio-60".into(),
                description: Some("60% SVG ratio".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "ratio-65".into(),
                description: Some("65% SVG ratio".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "ratio-70".into(),
                description: Some("70% SVG ratio".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "ratio-75".into(),
                description: Some("75% SVG ratio".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "ratio-80".into(),
                description: Some("80% SVG ratio".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "fixed-12".into(),
                description: Some("12px fixed SVG".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "fixed-13".into(),
                description: Some("13px fixed SVG".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "fixed-14".into(),
                description: Some("14px fixed SVG".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "fixed-15".into(),
                description: Some("15px fixed SVG".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "fixed-16".into(),
                description: Some("16px fixed SVG".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "negative-margin".into(),
                description: Some("Negative margin offset".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "padding-offset".into(),
                description: Some("Padding offset".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "line-height".into(),
                description: Some("Line height trick".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "absolute-pos".into(),
                description: Some("Absolute positioning".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "golden-ratio".into(),
                description: Some("Golden ratio sizing".into()),
                ..Default::default()
            },
        ]
    }
}

/// Row of variations with labels
fn variation_row(items: Vec<(&str, AnyElement)>) -> impl IntoElement {
    let mut row = div().flex().flex_row().items_center().gap_6().py_4();

    for (label, content) in items {
        row = row.child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(content)
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x888888))
                        .child(SharedString::from(label.to_string())),
                ),
        );
    }
    row
}

/// Comparison row showing multiple sizes together
fn comparison_row() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_4()
        .py_4()
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_1()
                .child(render_v3()) // 20px - likely best
                .child(div().text_xs().text_color(rgb(0x4ade80)).child("★ Best")),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_1()
                .child(render_v8()) // 70% ratio
                .child(div().text_xs().text_color(rgb(0x888888)).child("70%")),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_1()
                .child(render_v20()) // Golden ratio
                .child(div().text_xs().text_color(rgb(0x888888)).child("Golden")),
        )
}

// =============================================================================
// CONTAINER SIZE VARIATIONS (1-5)
// Using 20px container as baseline, varying the container size
// =============================================================================

/// 1. 24px container (original size)
fn render_v1() -> impl IntoElement {
    logo_box(24., 5., 15.)
}

/// 2. 22px container
fn render_v2() -> impl IntoElement {
    logo_box(22., 5., 14.)
}

/// 3. 20px container (smaller, tighter)
fn render_v3() -> impl IntoElement {
    logo_box(20., 4., 13.)
}

/// 4. 18px container
fn render_v4() -> impl IntoElement {
    logo_box(18., 4., 12.)
}

/// 5. 16px container (very compact)
fn render_v5() -> impl IntoElement {
    logo_box(16., 3., 10.)
}

// =============================================================================
// SVG SIZE RATIOS (6-10)
// Fixed 20px container, varying the SVG size ratio
// =============================================================================

/// 6. 60% ratio (SVG = 12px in 20px container)
fn render_v6() -> impl IntoElement {
    logo_box(20., 4., 12.)
}

/// 7. 65% ratio (SVG = 13px in 20px container)
fn render_v7() -> impl IntoElement {
    logo_box(20., 4., 13.)
}

/// 8. 70% ratio (SVG = 14px in 20px container)
fn render_v8() -> impl IntoElement {
    logo_box(20., 4., 14.)
}

/// 9. 75% ratio (SVG = 15px in 20px container)
fn render_v9() -> impl IntoElement {
    logo_box(20., 4., 15.)
}

/// 10. 80% ratio (SVG = 16px in 20px container)
fn render_v10() -> impl IntoElement {
    logo_box(20., 4., 16.)
}

// =============================================================================
// FIXED SVG SIZES (11-15)
// 22px container with various fixed SVG sizes
// =============================================================================

/// 11. 12px SVG in 22px container
fn render_v11() -> impl IntoElement {
    logo_box(22., 4., 12.)
}

/// 12. 13px SVG in 22px container
fn render_v12() -> impl IntoElement {
    logo_box(22., 4., 13.)
}

/// 13. 14px SVG in 22px container
fn render_v13() -> impl IntoElement {
    logo_box(22., 4., 14.)
}

/// 14. 15px SVG in 22px container
fn render_v14() -> impl IntoElement {
    logo_box(22., 4., 15.)
}

/// 15. 16px SVG in 22px container
fn render_v15() -> impl IntoElement {
    logo_box(22., 4., 16.)
}

// =============================================================================
// ALIGNMENT TWEAKS (16-20)
// Different centering approaches
// =============================================================================

/// 16. Negative margin to nudge left (SVG has slight right bias)
fn render_v16() -> impl IntoElement {
    div()
        .w(px(20.))
        .h(px(20.))
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0xffc821))
        .rounded(px(4.))
        .child(
            svg()
                .external_path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
                .size(px(13.))
                .text_color(rgb(0x000000))
                .ml(px(-0.5)), // Nudge left
        )
}

/// 17. Padding offset approach
fn render_v17() -> impl IntoElement {
    div()
        .w(px(20.))
        .h(px(20.))
        .flex()
        .items_center()
        .justify_center()
        .pl(px(0.5)) // Slight left padding to push SVG right
        .bg(rgb(0xffc821))
        .rounded(px(4.))
        .child(
            svg()
                .external_path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
                .size(px(13.))
                .text_color(rgb(0x000000)),
        )
}

/// 18. Using line_height for vertical centering
fn render_v18() -> impl IntoElement {
    div()
        .w(px(20.))
        .h(px(20.))
        .flex()
        .items_center()
        .justify_center()
        .line_height(px(20.))
        .bg(rgb(0xffc821))
        .rounded(px(4.))
        .child(
            svg()
                .external_path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
                .size(px(13.))
                .text_color(rgb(0x000000)),
        )
}

/// 19. Absolute positioning for pixel-perfect control
fn render_v19() -> impl IntoElement {
    div()
        .w(px(20.))
        .h(px(20.))
        .relative()
        .bg(rgb(0xffc821))
        .rounded(px(4.))
        .child(
            div().absolute().top(px(3.5)).left(px(3.5)).child(
                svg()
                    .external_path(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.svg"))
                    .size(px(13.))
                    .text_color(rgb(0x000000)),
            ),
        )
}

/// 20. Golden ratio sizing (container:svg ≈ 1.618)
fn render_v20() -> impl IntoElement {
    // If SVG is 13px, container should be ~21px for golden ratio
    // 13 * 1.618 ≈ 21
    logo_box(21., 4., 13.)
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Standard logo box with configurable dimensions
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
