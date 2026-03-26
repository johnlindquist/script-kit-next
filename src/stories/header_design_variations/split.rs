//! Header Design Variations - Layout Explorations
//!
//! 20 different header layout variations exploring different arrangements
//! of: input, Ask AI hint, buttons, logo, separators, and spacing.
//!
//! All variations use the same theme colors and fonts - only layout differs.

use gpui::*;

use crate::components::PromptHeaderColors;
use crate::storybook::{story_container, story_divider, story_section, Story, StorySurface, StoryVariant};
use crate::ui_foundation::HexColorExt;

// Story showcasing 20 header layout variations

// --- merged from part_01.rs ---
pub struct HeaderDesignVariationsStory;

impl Story for HeaderDesignVariationsStory {
    fn id(&self) -> &'static str {
        "header-design-variations"
    }

    fn name(&self) -> &'static str {
        "Header Design Variations"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Header
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let theme = crate::theme::get_cached_theme();
        let colors = PromptHeaderColors::from_theme(&theme);

        match variant.stable_id().as_str() {
            "current-production" => render_variation_1(colors).into_any_element(),
            "compact" => render_variation_2(colors).into_any_element(),
            "minimal" => render_variation_8(colors).into_any_element(),
            "command-palette" => render_variation_13(colors).into_any_element(),
            "raycast-style" => render_variation_20(colors).into_any_element(),
            _ => render_variation_1(colors).into_any_element(),
        }
    }

    fn render(&self) -> AnyElement {
        let theme = crate::theme::get_cached_theme();
        let colors = PromptHeaderColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Layout Variations (1-5)")
                    .child(header_variation_item(
                        "1. Current Production",
                        render_variation_1(colors),
                    ))
                    .child(header_variation_item(
                        "2. Compact - No Separators",
                        render_variation_2(colors),
                    ))
                    .child(header_variation_item(
                        "3. Buttons Left",
                        render_variation_3(colors),
                    ))
                    .child(header_variation_item(
                        "4. Centered Input",
                        render_variation_4(colors),
                    ))
                    .child(header_variation_item(
                        "5. Logo Left",
                        render_variation_5(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Layout Variations (6-10)")
                    .child(header_variation_item(
                        "6. Two Rows",
                        render_variation_6(colors),
                    ))
                    .child(header_variation_item(
                        "7. Pill Buttons",
                        render_variation_7(colors),
                    ))
                    .child(header_variation_item(
                        "8. Minimal - Input + Enter Only",
                        render_variation_8(colors),
                    ))
                    .child(header_variation_item(
                        "9. Search Box Style",
                        render_variation_9(colors),
                    ))
                    .child(header_variation_item(
                        "10. Tab Bar Style",
                        render_variation_10(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Layout Variations (11-15)")
                    .child(header_variation_item(
                        "11. Floating Actions",
                        render_variation_11(colors),
                    ))
                    .child(header_variation_item(
                        "12. Breadcrumb Style",
                        render_variation_12(colors),
                    ))
                    .child(header_variation_item(
                        "13. Command Palette",
                        render_variation_13(colors),
                    ))
                    .child(header_variation_item(
                        "14. Toolbar Style",
                        render_variation_14(colors),
                    ))
                    .child(header_variation_item(
                        "15. Split Header",
                        render_variation_15(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Layout Variations (16-20)")
                    .child(header_variation_item(
                        "16. Icon Buttons",
                        render_variation_16(colors),
                    ))
                    .child(header_variation_item(
                        "17. Grouped Actions",
                        render_variation_17(colors),
                    ))
                    .child(header_variation_item(
                        "18. Spotlight Style",
                        render_variation_18(colors),
                    ))
                    .child(header_variation_item(
                        "19. Alfred Style",
                        render_variation_19(colors),
                    ))
                    .child(header_variation_item(
                        "20. Raycast Style",
                        render_variation_20(colors),
                    )),
            )
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant::default_named("current-production", "Current Production")
                .description("Existing production header layout")
                .with_prop("layoutFamily", "production")
                .with_prop("density", "comfortable")
                .with_prop("actionPlacement", "trailing"),
            StoryVariant::default_named("compact", "Compact")
                .description("No separators, tighter button grouping")
                .with_prop("layoutFamily", "compact")
                .with_prop("density", "dense")
                .with_prop("separatorMode", "none"),
            StoryVariant::default_named("minimal", "Minimal")
                .description("Input plus primary action only")
                .with_prop("layoutFamily", "minimal")
                .with_prop("density", "airy")
                .with_prop("chromeLevel", "low"),
            StoryVariant::default_named("command-palette", "Command Palette")
                .description("Palette-style header with stronger grouping")
                .with_prop("layoutFamily", "palette")
                .with_prop("density", "comfortable")
                .with_prop("grouping", "strong"),
            StoryVariant::default_named("raycast-style", "Raycast Style")
                .description("Header aligned to Raycast treatment")
                .with_prop("layoutFamily", "raycast")
                .with_prop("density", "comfortable")
                .with_prop("reference", "raycast"),
        ]
    }
}

/// Wrapper for each header variation
fn header_variation_item(label: &str, content: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .w_full()
        .mb_4()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
        .child(
            div()
                .w_full()
                .bg(rgb(0x252526))
                .rounded_md()
                .overflow_hidden()
                .child(content),
        )
}

// ============================================================================
// VARIATION 1: Current Production Layout
// [Input] ................ [Ask AI Tab] | [Run ↵] | [Actions ⌘K] | [Logo]
// ============================================================================
fn render_variation_1(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        // Input area
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        // Ask AI hint
        .child(render_ask_ai_hint(colors))
        // Separator
        .child(render_separator(colors))
        // Run button
        .child(render_button("Run", "↵", colors))
        // Separator
        .child(render_separator(colors))
        // Actions button
        .child(render_button("Actions", "⌘K", colors))
        // Separator
        .child(render_separator(colors))
        // Logo
        .child(render_logo(colors))
}

// ============================================================================
// VARIATION 2: Compact - No Separators
// [Input] ................ [Ask AI Tab] [Run ↵] [Actions ⌘K] [Logo]
// ============================================================================
fn render_variation_2(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_2()
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_ask_ai_hint(colors))
        .child(render_button("Run", "↵", colors))
        .child(render_button("Actions", "⌘K", colors))
        .child(render_logo(colors))
}

// ============================================================================
// VARIATION 3: Buttons Left
// [Logo] [Run ↵] [Actions ⌘K] | [Input] ................ [Ask AI Tab]
// ============================================================================
fn render_variation_3(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(render_logo(colors))
        .child(render_button("Run", "↵", colors))
        .child(render_button("Actions", "⌘K", colors))
        .child(render_separator(colors))
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_ask_ai_hint(colors))
}

// ============================================================================
// VARIATION 4: Centered Input
// [Logo] | [Actions ⌘K] ...... [Input] ...... [Ask AI Tab] | [Run ↵]
// ============================================================================
fn render_variation_4(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(render_logo(colors))
        .child(render_separator(colors))
        .child(render_button("Actions", "⌘K", colors))
        .child(
            div()
                .flex_1()
                .flex()
                .justify_center()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_ask_ai_hint(colors))
        .child(render_separator(colors))
        .child(render_button("Run", "↵", colors))
}

// ============================================================================
// VARIATION 5: Logo Left with Title
// [Logo] Script Kit | [Input] .......... [Ask AI Tab] [Run ↵] [Actions]
// ============================================================================
fn render_variation_5(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(render_logo(colors))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors.text_primary.to_rgb())
                .child("Script Kit"),
        )
        .child(render_separator(colors))
        .child(
            div()
                .flex_1()
                .px_3()
                .py_1()
                .bg(colors.search_box_bg.to_rgb())
                .rounded_md()
                .text_color(colors.text_muted.to_rgb())
                .child("Type to search..."),
        )
        .child(render_ask_ai_hint(colors))
        .child(render_button("Run", "↵", colors))
        .child(render_button("Actions", "⌘K", colors))
}

// ============================================================================
// VARIATION 6: Two Rows
// Row 1: [Logo] Script Kit .......................... [Ask AI Tab]
// Row 2: [Input] .................. [Run ↵] | [Actions ⌘K]
// ============================================================================
fn render_variation_6(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .px_4()
        .py_2()
        .gap_2()
        // Row 1: Title bar
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(render_logo(colors))
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("SCRIPT KIT"),
                )
                .child(render_ask_ai_hint(colors)),
        )
        // Row 2: Input + buttons
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex_1()
                        .text_lg()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Type to search..."),
                )
                .child(render_button("Run", "↵", colors))
                .child(render_separator(colors))
                .child(render_button("Actions", "⌘K", colors)),
        )
}

// --- merged from part_02.rs ---

// ============================================================================
// VARIATION 7: Pill Buttons
// [Input] ............ [Ask AI Tab] [(Run ↵)] [(Actions ⌘K)] [Logo]
// ============================================================================
fn render_variation_7(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_ask_ai_hint(colors))
        .child(render_pill_button("Run ↵", colors, false))
        .child(render_pill_button("Actions ⌘K", colors, true))
        .child(render_logo(colors))
}

// ============================================================================
// VARIATION 8: Minimal - Input + Enter Only
// [Input] .................................................. [↵]
// ============================================================================
fn render_variation_8(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_3()
        .child(
            div()
                .flex_1()
                .text_xl()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(
            div()
                .px_3()
                .py_1()
                .rounded_md()
                .bg(colors.accent.rgba8(0x20))
                .text_color(colors.accent.to_rgb())
                .text_sm()
                .child("↵"),
        )
}

// ============================================================================
// VARIATION 9: Search Box Style (outlined input)
// [🔍 Input ...........................] [Ask AI] [Run] [⋮]
// ============================================================================
fn render_variation_9(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px_3()
                .py_2()
                .bg(colors.search_box_bg.to_rgb())
                .border_1()
                .border_color(colors.border.to_rgb())
                .rounded_lg()
                .child(div().text_color(colors.text_dimmed.to_rgb()).child("🔍"))
                .child(
                    div()
                        .flex_1()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Script Kit"),
                ),
        )
        .child(render_text_button("Ask AI", colors))
        .child(render_text_button("Run", colors))
        .child(div().text_color(colors.text_dimmed.to_rgb()).child("⋮"))
}

// ============================================================================
// VARIATION 10: Tab Bar Style
// [Script Kit ▾] | [Input ......................] | [⌘K] [↵]
// ============================================================================
fn render_variation_10(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .px_2()
                .py_1()
                .bg(colors.search_box_bg.to_rgb())
                .rounded_md()
                .child(render_logo(colors))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_primary.to_rgb())
                        .child("Script Kit"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("▾"),
                ),
        )
        .child(render_separator(colors))
        .child(
            div()
                .flex_1()
                .text_color(colors.text_muted.to_rgb())
                .child("Type to search..."),
        )
        .child(render_separator(colors))
        .child(render_kbd("⌘K", colors))
        .child(render_kbd("↵", colors))
}

// ============================================================================
// VARIATION 11: Floating Actions (actions in a separate container)
// [Input] ........................ [Ask AI Tab] | [  Run ↵  Actions ⌘K  ]
// ============================================================================
fn render_variation_11(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_ask_ai_hint(colors))
        .child(render_separator(colors))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px_3()
                .py_1()
                .bg(colors.search_box_bg.to_rgb())
                .rounded_lg()
                .child(render_button("Run", "↵", colors))
                .child(render_button("Actions", "⌘K", colors)),
        )
        .child(render_logo(colors))
}

// ============================================================================
// VARIATION 12: Breadcrumb Style
// [Logo] > [Scripts] > [Input ...................] [Ask AI] [Run ↵]
// ============================================================================
fn render_variation_12(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_2()
        .child(render_logo(colors))
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .text_sm()
                .child(">"),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child("Scripts"),
        )
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .text_sm()
                .child(">"),
        )
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Search..."),
        )
        .child(render_ask_ai_hint(colors))
        .child(render_button("Run", "↵", colors))
}

// ============================================================================
// VARIATION 13: Command Palette Style (VS Code inspired)
// [>] [Input ........................................] [Esc to close]
// ============================================================================
fn render_variation_13(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_3()
        .gap_2()
        .child(
            div()
                .text_lg()
                .text_color(colors.accent.to_rgb())
                .child(">"),
        )
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Type a command..."),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("Esc to close"),
        )
}

// ============================================================================
// VARIATION 14: Toolbar Style (with icon buttons)
// [Logo] | [🏠] [📁] [⚙️] | [Input ..........] | [Ask AI] [Run ↵]
// ============================================================================
fn render_variation_14(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_2()
        .child(render_logo(colors))
        .child(render_separator(colors))
        .child(render_icon_button("🏠", colors))
        .child(render_icon_button("📁", colors))
        .child(render_icon_button("⚙️", colors))
        .child(render_separator(colors))
        .child(
            div()
                .flex_1()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_separator(colors))
        .child(render_ask_ai_hint(colors))
        .child(render_button("Run", "↵", colors))
}

// ============================================================================
// VARIATION 15: Split Header (left/right sections)
// [Logo] [Input ............] || [Ask AI Tab] [Run ↵] [Actions ⌘K]
// ============================================================================
fn render_variation_15(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_4()
        // Left section
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(render_logo(colors))
                .child(
                    div()
                        .flex_1()
                        .text_lg()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Script Kit"),
                ),
        )
        // Thick separator
        .child(div().w_px().h_6().bg(colors.border.to_rgb()))
        // Right section
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(render_ask_ai_hint(colors))
                .child(render_button("Run", "↵", colors))
                .child(render_button("Actions", "⌘K", colors)),
        )
}

// ============================================================================
// VARIATION 16: Icon-Only Buttons
// [Input] .......................... [Ask AI Tab] [▶] [⚡] [⋯] [Logo]
// ============================================================================
fn render_variation_16(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_ask_ai_hint(colors))
        .child(render_icon_button("▶", colors))
        .child(render_icon_button("⚡", colors))
        .child(render_icon_button("⋯", colors))
        .child(render_logo(colors))
}

// --- merged from part_03.rs ---

// ============================================================================
// VARIATION 17: Grouped Actions (with background)
// [Input] .......... | [Ask AI] | [ Run  |  Actions ] | [Logo]
// ============================================================================
fn render_variation_17(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_separator(colors))
        .child(render_text_button("Ask AI", colors))
        .child(render_separator(colors))
        // Grouped buttons
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .bg(colors.search_box_bg.to_rgb())
                .rounded_md()
                .overflow_hidden()
                .child(
                    div()
                        .px_3()
                        .py_1()
                        .text_sm()
                        .text_color(colors.accent.to_rgb())
                        .hover(|s| s.bg(colors.accent.rgba8(0x20)))
                        .child("Run ↵"),
                )
                .child(div().w_px().h_4().bg(colors.border.to_rgb()))
                .child(
                    div()
                        .px_3()
                        .py_1()
                        .text_sm()
                        .text_color(colors.accent.to_rgb())
                        .hover(|s| s.bg(colors.accent.rgba8(0x20)))
                        .child("Actions ⌘K"),
                ),
        )
        .child(render_separator(colors))
        .child(render_logo(colors))
}

// ============================================================================
// VARIATION 18: Spotlight Style (Apple Spotlight inspired)
// [ 🔍  Input .................................................. ]
// ============================================================================
fn render_variation_18(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_6()
        .py_4()
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .text_xl()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("🔍"),
                )
                .child(
                    div()
                        .flex_1()
                        .text_2xl()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Script Kit"),
                ),
        )
}

// ============================================================================
// VARIATION 19: Alfred Style
// [Input ...................] [↵] .......... [⌘1] [⌘2] [⌘3]
// ============================================================================
fn render_variation_19(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_kbd("↵", colors))
        .child(div().flex_1()) // Spacer
        .child(render_kbd("⌘1", colors))
        .child(render_kbd("⌘2", colors))
        .child(render_kbd("⌘3", colors))
}

// ============================================================================
// VARIATION 20: Raycast Style (current production look)
// [Input] ............ [Ask AI Tab] [Run ↵] | [Actions ⌘K] | [▶]
// ============================================================================
fn render_variation_20(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_ask_ai_hint(colors))
        .child(render_button("Run", "↵", colors))
        .child(render_separator(colors))
        .child(render_button("Actions", "⌘K", colors))
        .child(render_separator(colors))
        .child(
            div()
                .w_6()
                .h_6()
                .flex()
                .items_center()
                .justify_center()
                .rounded_md()
                .bg(colors.accent.to_rgb())
                .text_color(rgb(0x000000))
                .text_sm()
                .child("▶"),
        )
}

// ============================================================================
// HELPER COMPONENTS
// ============================================================================

/// Render the "Ask AI [Tab]" hint
fn render_ask_ai_hint(colors: PromptHeaderColors) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .flex_shrink_0()
        .child(
            div()
                .text_sm()
                .text_color(colors.text_muted.to_rgb())
                .child("Ask AI"),
        )
        .child(
            div()
                .px_1()
                .py_px()
                .rounded(px(3.))
                .border_1()
                .border_color(colors.border.to_rgb())
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("Tab"),
        )
}

/// Render a separator
fn render_separator(colors: PromptHeaderColors) -> Div {
    div()
        .text_sm()
        .text_color(colors.text_dimmed.rgba8(0x60))
        .child("|")
}

/// Render a text button (label + shortcut)
fn render_button(label: &str, shortcut: &str, colors: PromptHeaderColors) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .text_sm()
        .text_color(colors.accent.to_rgb())
        .child(label.to_string())
        .child(
            div()
                .text_color(colors.text_dimmed.to_rgb())
                .child(shortcut.to_string()),
        )
}

/// Render a pill-style button
fn render_pill_button(label: &str, colors: PromptHeaderColors, outlined: bool) -> Div {
    let base = div()
        .px_3()
        .py_1()
        .rounded_full()
        .text_sm()
        .text_color(colors.accent.to_rgb());

    if outlined {
        base.border_1().border_color(colors.accent.rgba8(0x40))
    } else {
        base.bg(colors.accent.rgba8(0x20))
    }
    .child(label.to_string())
}

/// Render a text-only button
fn render_text_button(label: &str, colors: PromptHeaderColors) -> Div {
    div()
        .text_sm()
        .text_color(colors.accent.to_rgb())
        .child(label.to_string())
}

/// Render an icon button
fn render_icon_button(icon: &str, colors: PromptHeaderColors) -> Div {
    div()
        .w_7()
        .h_7()
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .hover(|s| s.bg(colors.search_box_bg.to_rgb()))
        .text_color(colors.text_muted.to_rgb())
        .child(icon.to_string())
}

/// Render a keyboard shortcut badge
fn render_kbd(key: &str, colors: PromptHeaderColors) -> Div {
    div()
        .px_2()
        .py_1()
        .rounded(px(4.))
        .bg(colors.search_box_bg.to_rgb())
        .border_1()
        .border_color(colors.border.to_rgb())
        .text_xs()
        .text_color(colors.text_muted.to_rgb())
        .child(key.to_string())
}

/// Render the logo
fn render_logo(colors: PromptHeaderColors) -> Div {
    div()
        .w_4()
        .h_4()
        .flex()
        .items_center()
        .justify_center()
        .text_color(colors.accent.to_rgb())
        .child("▶") // Placeholder for actual SVG logo
}

#[cfg(test)]
fn supports_header_variant(variant_id: &str) -> bool {
    matches!(
        variant_id,
        "current-production" | "compact" | "minimal" | "command-palette" | "raycast-style"
    )
}

#[cfg(test)]
mod tests {
    use super::{supports_header_variant, HeaderDesignVariationsStory};
    use crate::storybook::Story;

    #[test]
    fn compare_variants_have_preview_dispatch() {
        let story = HeaderDesignVariationsStory;
        for variant in story.variants() {
            let variant_id = variant.stable_id();
            assert!(
                supports_header_variant(&variant_id),
                "missing header preview dispatch for {}",
                variant_id
            );
        }
    }
}
