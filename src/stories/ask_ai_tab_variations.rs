//! "Ask AI" + Tab glyph variations for the prompt header input area.
//!
//! 15 variations exploring different treatments for the right-side
//! "Ask" indicator with Tab key-cap in the main menu search input.

use gpui::*;

use crate::components::PromptHeaderColors;
use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

pub struct AskAiTabVariationsStory;

impl Story for AskAiTabVariationsStory {
    fn id(&self) -> &'static str {
        "ask-ai-tab-variations"
    }

    fn name(&self) -> &'static str {
        "Ask AI Tab Variations"
    }

    fn category(&self) -> &'static str {
        "Components"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = PromptHeaderColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Muted / Ghost (1-5)")
                    .child(variation_item(
                        "1. Muted text + bordered keycap",
                        render_v1(colors),
                    ))
                    .child(variation_item(
                        "2. Muted text + filled keycap",
                        render_v2(colors),
                    ))
                    .child(variation_item(
                        "3. Muted text + Tab symbol ⇥",
                        render_v3(colors),
                    ))
                    .child(variation_item(
                        "4. Single combined ghost badge",
                        render_v4(colors),
                    ))
                    .child(variation_item("5. Dimmed ultra-minimal", render_v5(colors))),
            )
            .child(story_divider())
            .child(
                story_section("Accent-tinted (6-10)")
                    .child(variation_item(
                        "6. Accent 'Ask' + muted keycap",
                        render_v6(colors),
                    ))
                    .child(variation_item("7. Accent glow badge", render_v7(colors)))
                    .child(variation_item(
                        "8. Accent text + outline keycap",
                        render_v8(colors),
                    ))
                    .child(variation_item(
                        "9. Pill with accent border",
                        render_v9(colors),
                    ))
                    .child(variation_item(
                        "10. Accent dot + muted Ask Tab",
                        render_v10(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Compact / Alt layouts (11-15)")
                    .child(variation_item(
                        "11. Just 'Tab' keycap, no text",
                        render_v11(colors),
                    ))
                    .child(variation_item(
                        "12. Stacked: Ask above, Tab below",
                        render_v12(colors),
                    ))
                    .child(variation_item(
                        "13. 'Ask AI' reversed: keycap first",
                        render_v13(colors),
                    ))
                    .child(variation_item(
                        "14. Inline 'Ask AI ⇥' no badge",
                        render_v14(colors),
                    ))
                    .child(variation_item(
                        "15. Raycast-exact clone",
                        render_v15(colors),
                    )),
            )
            .child(story_divider())
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        (1..=15)
            .map(|i| StoryVariant {
                name: format!("v{i}").into(),
                description: Some(format!("Variation {i}").into()),
                ..Default::default()
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

/// Fake search input row that contextualizes each variation.
fn input_row(colors: PromptHeaderColors) -> Div {
    div()
        .w_full()
        .h(px(48.))
        .px_4()
        .flex()
        .flex_row()
        .items_center()
        .bg(colors.background.to_rgb())
        .border_1()
        .border_color(colors.border.rgba8(0x33))
        .rounded(px(10.))
        // placeholder text
        .child(
            div()
                .flex_1()
                .text_sm()
                .text_color(colors.text_dimmed.to_rgb())
                .child("Search for apps and commands..."),
        )
}

// ---------------------------------------------------------------------------
// 1-5: Muted / Ghost
// ---------------------------------------------------------------------------

/// 1. Muted text "Ask" + bordered "Tab" keycap (closest to Raycast screenshot)
fn render_v1(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .flex()
            .items_center()
            .gap(px(6.))
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
                    .border_1()
                    .border_color(colors.border.to_rgb())
                    .rounded(px(4.))
                    .text_xs()
                    .text_color(colors.text_muted.to_rgb())
                    .child("Tab"),
            ),
    )
}

/// 2. Muted "Ask" + filled dark keycap
fn render_v2(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .flex()
            .items_center()
            .gap(px(6.))
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
            ),
    )
}

/// 3. Muted "Ask" + Tab arrow symbol ⇥ in keycap
fn render_v3(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .flex()
            .items_center()
            .gap(px(6.))
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
                    .border_1()
                    .border_color(colors.border.to_rgb())
                    .rounded(px(4.))
                    .text_xs()
                    .text_color(colors.text_muted.to_rgb())
                    .child("⇥"),
            ),
    )
}

/// 4. Single combined ghost badge "Ask AI  Tab"
fn render_v4(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .px(px(8.))
            .py(px(3.))
            .bg(rgba(0xffffff08))
            .rounded(px(6.))
            .child(
                div()
                    .text_sm()
                    .text_color(colors.text_muted.to_rgb())
                    .child("Ask AI"),
            )
            .child(
                div()
                    .px(px(5.))
                    .py(px(1.))
                    .bg(colors.search_box_bg.to_rgb())
                    .rounded(px(3.))
                    .text_xs()
                    .text_color(colors.text_dimmed.to_rgb())
                    .child("Tab"),
            ),
    )
}

/// 5. Ultra-minimal dimmed: "Ask AI ⇥" all one color
fn render_v5(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .flex()
            .items_center()
            .gap(px(4.))
            .text_xs()
            .text_color(colors.text_dimmed.rgba8(0x80))
            .child("Ask AI")
            .child("⇥"),
    )
}

// ---------------------------------------------------------------------------
// 6-10: Accent-tinted
// ---------------------------------------------------------------------------

/// 6. Accent-colored "Ask" + muted keycap "Tab"
fn render_v6(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .flex()
            .items_center()
            .gap(px(6.))
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
            ),
    )
}

/// 7. Accent glow badge — "Ask AI" + Tab inside a glowing container
fn render_v7(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .px(px(8.))
            .py(px(3.))
            .bg(colors.accent.rgba8(0x12))
            .border_1()
            .border_color(colors.accent.rgba8(0x30))
            .rounded(px(6.))
            .child(
                div()
                    .text_sm()
                    .text_color(colors.accent.to_rgb())
                    .child("Ask AI"),
            )
            .child(
                div()
                    .px(px(5.))
                    .py(px(1.))
                    .bg(colors.accent.rgba8(0x20))
                    .rounded(px(3.))
                    .text_xs()
                    .text_color(colors.accent.rgba8(0xAA))
                    .child("Tab"),
            ),
    )
}

/// 8. Accent text "Ask" + outline keycap "Tab" with accent border
fn render_v8(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .flex()
            .items_center()
            .gap(px(6.))
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
                    .border_1()
                    .border_color(colors.accent.rgba8(0x60))
                    .rounded(px(4.))
                    .text_xs()
                    .text_color(colors.accent.rgba8(0x99))
                    .child("Tab"),
            ),
    )
}

/// 9. Pill container with accent border — "Ask AI · Tab"
fn render_v9(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .px(px(10.))
            .py(px(3.))
            .border_1()
            .border_color(colors.accent.rgba8(0x40))
            .rounded_full()
            .child(
                div()
                    .text_xs()
                    .text_color(colors.accent.rgba8(0xCC))
                    .child("Ask AI"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors.accent.rgba8(0x40))
                    .child("·"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors.text_muted.to_rgb())
                    .child("Tab"),
            ),
    )
}

/// 10. Small accent dot indicator + "Ask AI  Tab" muted
fn render_v10(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .child(
                div()
                    .w(px(5.))
                    .h(px(5.))
                    .rounded_full()
                    .bg(colors.accent.to_rgb()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(colors.text_muted.to_rgb())
                    .child("Ask AI"),
            )
            .child(
                div()
                    .px(px(5.))
                    .py(px(1.))
                    .border_1()
                    .border_color(colors.border.to_rgb())
                    .rounded(px(3.))
                    .text_xs()
                    .text_color(colors.text_dimmed.to_rgb())
                    .child("Tab"),
            ),
    )
}

// ---------------------------------------------------------------------------
// 11-15: Compact / Alt layouts
// ---------------------------------------------------------------------------

/// 11. Just the "Tab" keycap — no "Ask AI" text
fn render_v11(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .px(px(8.))
            .py(px(3.))
            .border_1()
            .border_color(colors.border.to_rgb())
            .rounded(px(4.))
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(colors.text_muted.to_rgb())
            .child("Tab"),
    )
}

/// 12. Stacked: "Ask" on top, "Tab" keycap below
fn render_v12(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(1.))
            .child(
                div()
                    .text_xs()
                    .text_color(colors.text_muted.to_rgb())
                    .child("Ask AI"),
            )
            .child(
                div()
                    .px(px(5.))
                    .py(px(1.))
                    .bg(colors.search_box_bg.to_rgb())
                    .rounded(px(3.))
                    .text_xs()
                    .text_color(colors.text_dimmed.to_rgb())
                    .child("Tab"),
            ),
    )
}

/// 13. Reversed: keycap "Tab" first, then "Ask AI"
fn render_v13(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .child(
                div()
                    .px(px(6.))
                    .py(px(2.))
                    .border_1()
                    .border_color(colors.border.to_rgb())
                    .rounded(px(4.))
                    .text_xs()
                    .text_color(colors.text_muted.to_rgb())
                    .child("Tab"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(colors.text_muted.to_rgb())
                    .child("Ask AI"),
            ),
    )
}

/// 14. Inline — "Ask AI ⇥" as flat text, no badge
fn render_v14(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .text_sm()
            .text_color(colors.text_muted.to_rgb())
            .child("Ask AI  ⇥"),
    )
}

/// 15. Raycast-exact clone — accent "Ask AI" + grey "Tab" filled badge
fn render_v15(colors: PromptHeaderColors) -> impl IntoElement {
    input_row(colors).child(
        div()
            .flex()
            .items_center()
            .gap(px(4.))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(colors.text_muted.rgba8(0xAA))
                    .child("Ask AI"),
            )
            .child(
                div()
                    .px(px(6.))
                    .py(px(2.))
                    .bg(rgba(0xffffff0F))
                    .rounded(px(4.))
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(colors.text_dimmed.rgba8(0x99))
                    .child("Tab"),
            ),
    )
}
