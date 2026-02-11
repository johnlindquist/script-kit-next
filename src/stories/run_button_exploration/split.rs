//! Run Button Exploration - 50+ Variations
//!
//! The challenge: The "Run" button changes text based on context:
//! - "Run" for scripts
//! - "Submit" for forms
//! - "Select" for choices
//! - "Open Chrome" for app launchers
//! - etc.
//!
//! This creates layout instability and visual clutter.
//! We want the header to feel simple, not busy.
//!
//! This story explores every possible approach to solve this.

use gpui::*;

use crate::components::PromptHeaderColors;
use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;
use crate::utils;

// --- merged from part_01.rs ---
pub struct RunButtonExplorationStory;

// --- merged from part_02.rs ---

impl Story for RunButtonExplorationStory {
    fn id(&self) -> &'static str {
        "run-button-exploration"
    }

    fn name(&self) -> &'static str {
        "Run Button Exploration (50+)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = PromptHeaderColors::from_theme(&theme);

        story_container()
            // =================================================================
            // SECTION 1: NO RUN BUTTON AT ALL (1-6)
            // =================================================================
            .child(
                story_section("1. NO RUN BUTTON - Just Enter key hint")
                    .child(variation_label(
                        "The simplest option: don't show a button at all",
                    ))
                    .child(variation_item(
                        "1. Minimal - just shortcuts",
                        render_no_run_minimal(colors),
                    ))
                    .child(variation_item(
                        "2. Ask AI only",
                        render_no_run_ask_ai_only(colors),
                    ))
                    .child(variation_item(
                        "3. Just Actions",
                        render_no_run_actions_only(colors),
                    ))
                    .child(variation_item(
                        "4. Keyboard hint in input",
                        render_no_run_hint_in_input(colors),
                    ))
                    .child(variation_item(
                        "5. Enter hint at far right",
                        render_no_run_enter_far_right(colors),
                    ))
                    .child(variation_item(
                        "6. Floating hint below",
                        render_no_run_floating_hint(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 2: ICON-ONLY APPROACHES (7-14)
            // =================================================================
            .child(
                story_section("2. ICON-ONLY - No text, just icon")
                    .child(variation_label("Icons save space and don't change width"))
                    .child(variation_item(
                        "7. Play icon ▶",
                        render_icon_only_play(colors),
                    ))
                    .child(variation_item(
                        "8. Arrow icon →",
                        render_icon_only_arrow(colors),
                    ))
                    .child(variation_item(
                        "9. Check icon ✓",
                        render_icon_only_check(colors),
                    ))
                    .child(variation_item(
                        "10. Return icon ↵",
                        render_icon_only_return(colors),
                    ))
                    .child(variation_item(
                        "11. Filled circle ●",
                        render_icon_only_circle(colors),
                    ))
                    .child(variation_item(
                        "12. Double arrow »",
                        render_icon_only_double_arrow(colors),
                    ))
                    .child(variation_item(
                        "13. Icon in circle",
                        render_icon_in_circle(colors),
                    ))
                    .child(variation_item(
                        "14. Icon with ring",
                        render_icon_with_ring(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 3: FIXED-WIDTH BUTTON (15-22)
            // =================================================================
            .child(
                story_section("3. FIXED-WIDTH - Prevent layout shift")
                    .child(variation_label(
                        "Fixed width prevents jumping when text changes",
                    ))
                    .child(variation_item(
                        "15. 60px fixed 'Run'",
                        render_fixed_width_60(colors, "Run"),
                    ))
                    .child(variation_item(
                        "16. 60px fixed 'Submit'",
                        render_fixed_width_60(colors, "Submit"),
                    ))
                    .child(variation_item(
                        "17. 80px fixed 'Open Chrome'",
                        render_fixed_width_80(colors, "Open Chrome"),
                    ))
                    .child(variation_item(
                        "18. 80px fixed 'Select'",
                        render_fixed_width_80(colors, "Select"),
                    ))
                    .child(variation_item(
                        "19. Truncate long text",
                        render_fixed_truncate(colors),
                    ))
                    .child(variation_item(
                        "20. Fixed with tooltip",
                        render_fixed_with_tooltip(colors),
                    ))
                    .child(variation_item(
                        "21. Fixed pill style",
                        render_fixed_pill(colors),
                    ))
                    .child(variation_item(
                        "22. Fixed ghost style",
                        render_fixed_ghost(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 4: POSITIONED AT EDGES (23-30)
            // =================================================================
            .child(
                story_section("4. EDGE POSITIONING - Always in same spot")
                    .child(variation_label("Pin to edge so other elements don't shift"))
                    .child(variation_item(
                        "23. Far right (after logo)",
                        render_pos_far_right(colors),
                    ))
                    .child(variation_item(
                        "24. Before logo, fixed position",
                        render_pos_before_logo(colors),
                    ))
                    .child(variation_item(
                        "25. In input field right side",
                        render_pos_in_input(colors),
                    ))
                    .child(variation_item(
                        "26. Overlapping input corner",
                        render_pos_overlap_input(colors),
                    ))
                    .child(variation_item(
                        "27. Below header strip",
                        render_pos_below_header(colors),
                    ))
                    .child(variation_item(
                        "28. Floating bottom right",
                        render_pos_floating_br(colors),
                    ))
                    .child(variation_item(
                        "29. As part of list first item",
                        render_pos_in_list(colors),
                    ))
                    .child(variation_item(
                        "30. Sticky footer action",
                        render_pos_sticky_footer(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 5: COMBINE WITH ACTIONS (31-38)
            // =================================================================
            .child(
                story_section("5. MERGE WITH ACTIONS - One unified button")
                    .child(variation_label("What if Run was inside the Actions menu?"))
                    .child(variation_item(
                        "31. Actions dropdown with Run first",
                        render_actions_merged(colors),
                    ))
                    .child(variation_item(
                        "32. Split button: Run | ▼",
                        render_split_button(colors),
                    ))
                    .child(variation_item(
                        "33. Primary action pill + more",
                        render_pill_plus_more(colors),
                    ))
                    .child(variation_item(
                        "34. Contextual - 'Run' + more actions",
                        render_contextual_primary(colors),
                    ))
                    .child(variation_item(
                        "35. Two-part: icon + dropdown",
                        render_two_part(colors),
                    ))
                    .child(variation_item(
                        "36. Expandable on hover",
                        render_expandable_hover(colors),
                    ))
                    .child(variation_item(
                        "37. Cycle through actions",
                        render_cycle_actions(colors),
                    ))
                    .child(variation_item(
                        "38. Quick action + menu",
                        render_quick_plus_menu(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 6: SEMANTIC/CONTEXTUAL ICONS (39-46)
            // =================================================================
            .child(
                story_section("6. CONTEXTUAL ICONS - Icon changes, not text")
                    .child(variation_label(
                        "Icon conveys meaning without text width changes",
                    ))
                    .child(variation_item(
                        "39. Script: terminal icon",
                        render_context_terminal(colors),
                    ))
                    .child(variation_item(
                        "40. Form: send icon",
                        render_context_send(colors),
                    ))
                    .child(variation_item(
                        "41. Choice: check icon",
                        render_context_check(colors),
                    ))
                    .child(variation_item(
                        "42. App: launch icon",
                        render_context_launch(colors),
                    ))
                    .child(variation_item(
                        "43. File: folder icon",
                        render_context_folder(colors),
                    ))
                    .child(variation_item(
                        "44. URL: globe icon",
                        render_context_globe(colors),
                    ))
                    .child(variation_item(
                        "45. Command: gear icon",
                        render_context_gear(colors),
                    ))
                    .child(variation_item(
                        "46. Copy: clipboard icon",
                        render_context_clipboard(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 7: TIGHTER BUTTONS (47-54)
            // =================================================================
            .child(
                story_section("7. TIGHTER BUTTONS - Minimal padding")
                    .child(variation_label(
                        "How small can buttons get while staying clickable?",
                    ))
                    .child(variation_item(
                        "47. Micro: 2px padding",
                        render_tight_micro(colors),
                    ))
                    .child(variation_item(
                        "48. Small: 4px padding",
                        render_tight_small(colors),
                    ))
                    .child(variation_item(
                        "49. Compact: 4px h, 6px w",
                        render_tight_compact(colors),
                    ))
                    .child(variation_item(
                        "50. Text only, no button",
                        render_tight_text_only(colors),
                    ))
                    .child(variation_item(
                        "51. Underline on hover",
                        render_tight_underline(colors),
                    ))
                    .child(variation_item(
                        "52. Badge style",
                        render_tight_badge(colors),
                    ))
                    .child(variation_item(
                        "53. Inline link style",
                        render_tight_link(colors),
                    ))
                    .child(variation_item(
                        "54. Minimal pill",
                        render_tight_pill(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 8: ALTERNATIVE PLACEMENTS (55-62)
            // =================================================================
            .child(
                story_section("8. ALTERNATIVE PLACEMENTS - Outside header")
                    .child(variation_label(
                        "Maybe the action doesn't belong in the header?",
                    ))
                    .child(variation_item(
                        "55. In selected list item",
                        render_alt_in_list_item(colors),
                    ))
                    .child(variation_item(
                        "56. As hover overlay on item",
                        render_alt_hover_overlay(colors),
                    ))
                    .child(variation_item(
                        "57. Keyboard-only (no visual)",
                        render_alt_keyboard_only(colors),
                    ))
                    .child(variation_item(
                        "58. Status bar bottom",
                        render_alt_status_bar(colors),
                    ))
                    .child(variation_item(
                        "59. Context on right-click",
                        render_alt_right_click(colors),
                    ))
                    .child(variation_item(
                        "60. Gesture hint (swipe)",
                        render_alt_gesture(colors),
                    ))
                    .child(variation_item(
                        "61. Double-click to run",
                        render_alt_double_click(colors),
                    ))
                    .child(variation_item(
                        "62. Long-press actions",
                        render_alt_long_press(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 9: VISUAL HIERARCHY (63-70)
            // =================================================================
            .child(
                story_section("9. VISUAL HIERARCHY - De-emphasize or emphasize")
                    .child(variation_label("Control attention via styling"))
                    .child(variation_item(
                        "63. Ghost (barely visible)",
                        render_hier_ghost(colors),
                    ))
                    .child(variation_item(
                        "64. Muted until hover",
                        render_hier_muted(colors),
                    ))
                    .child(variation_item(
                        "65. Primary action (bold)",
                        render_hier_primary(colors),
                    ))
                    .child(variation_item(
                        "66. Accent background",
                        render_hier_accent_bg(colors),
                    ))
                    .child(variation_item(
                        "67. Outline style",
                        render_hier_outline(colors),
                    ))
                    .child(variation_item(
                        "68. Gradient accent",
                        render_hier_gradient(colors),
                    ))
                    .child(variation_item(
                        "69. Glow effect (hover)",
                        render_hier_glow(colors),
                    ))
                    .child(variation_item(
                        "70. Pulsing attention",
                        render_hier_pulse(colors),
                    )),
            )
            .child(story_divider())
            // =================================================================
            // SECTION 10: RECOMMENDED APPROACHES (71-76)
            // =================================================================
            .child(
                story_section("10. RECOMMENDED - Best combinations")
                    .child(variation_label("Synthesized from above explorations"))
                    .child(variation_item(
                        "71. ★ Icon-only + tooltip",
                        render_rec_icon_tooltip(colors),
                    ))
                    .child(variation_item(
                        "72. ★ Fixed-width ghost",
                        render_rec_fixed_ghost(colors),
                    ))
                    .child(variation_item(
                        "73. ★ No button, Enter hint",
                        render_rec_no_button(colors),
                    ))
                    .child(variation_item(
                        "74. ★ Contextual icon, no text",
                        render_rec_contextual_icon(colors),
                    ))
                    .child(variation_item(
                        "75. ★ Merge into Actions",
                        render_rec_merged(colors),
                    ))
                    .child(variation_item(
                        "76. ★ Split button compact",
                        render_rec_split_compact(colors),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        // Not implementing all 76 as individual variants - the story shows them all
        vec![
            StoryVariant {
                name: "no-run".into(),
                description: Some("No run button".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "icon-only".into(),
                description: Some("Icon without text".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "fixed-width".into(),
                description: Some("Fixed width button".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "recommended".into(),
                description: Some("Recommended approaches".into()),
                ..Default::default()
            },
        ]
    }
}

// --- merged from part_03.rs ---

// =============================================================================
// HELPER COMPONENTS
// =============================================================================

fn variation_label(text: &str) -> impl IntoElement {
    div()
        .text_xs()
        .text_color(rgb(0x666666))
        .italic()
        .mb_2()
        .child(text.to_string())
}

fn variation_item(label: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .mb_3()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
        .child(content)
}

fn header_container(colors: PromptHeaderColors) -> Div {
    div()
        .w_full()
        .px(px(12.))
        .py(px(8.))
        .flex()
        .flex_row()
        .items_center()
        .bg(colors.background.to_rgb())
        .rounded(px(8.))
}

fn script_kit_label(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(colors.text_primary.to_rgb())
        .child("Script Kit")
}

fn logo_box() -> impl IntoElement {
    div()
        .w(px(19.))
        .h(px(19.))
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0xFFD60AD9))
        .rounded(px(4.))
        .child(
            svg()
                .external_path(utils::get_logo_path())
                .size(px(12.))
                .text_color(rgb(0x000000)),
        )
}

fn ask_ai_button(colors: PromptHeaderColors) -> Stateful<Div> {
    let hover_bg = (colors.accent << 8) | 0x26;
    let tab_bg = (colors.search_box_bg << 8) | 0x4D;

    div()
        .id("ask-ai")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .px(px(6.))
        .py(px(3.))
        .rounded(px(4.))
        .cursor_pointer()
        .hover(move |s| s.bg(rgba(hover_bg)))
        .child(
            div()
                .text_xs()
                .text_color(colors.accent.to_rgb())
                .child("Ask AI"),
        )
        .child(
            div()
                .px(px(4.))
                .py(px(1.))
                .bg(rgba(tab_bg))
                .rounded(px(3.))
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("Tab"),
        )
}

fn actions_button(colors: PromptHeaderColors) -> Stateful<Div> {
    let hover_bg = (colors.accent << 8) | 0x26;

    div()
        .id("actions")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .px(px(6.))
        .py(px(3.))
        .rounded(px(4.))
        .cursor_pointer()
        .hover(move |s| s.bg(rgba(hover_bg)))
        .child(
            div()
                .text_xs()
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
// SECTION 1: NO RUN BUTTON
// =============================================================================

fn render_no_run_minimal(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("↵ Enter"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_no_run_ask_ai_only(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_no_run_actions_only(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_no_run_hint_in_input(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            // Simulated input with hint
            div()
                .mx(px(12.))
                .px(px(12.))
                .py(px(8.))
                .bg(rgba((colors.search_box_bg << 8) | 0x80))
                .rounded(px(6.))
                .flex()
                .flex_row()
                .items_center()
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Type to search..."),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("↵"),
                ),
        )
}

fn render_no_run_enter_far_right(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(12.)))
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("↵"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_no_run_floating_hint(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            div().w_full().flex().justify_end().pr(px(20.)).child(
                div()
                    .text_xs()
                    .text_color(colors.text_dimmed.to_rgb())
                    .child("Press Enter to run"),
            ),
        )
}

// =============================================================================
// SECTION 2: ICON-ONLY
// =============================================================================

fn icon_button(colors: PromptHeaderColors, icon: &'static str, id: &'static str) -> Stateful<Div> {
    let hover_bg = (colors.accent << 8) | 0x26;

    div()
        .id(id)
        .w(px(24.))
        .h(px(24.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .cursor_pointer()
        .hover(move |s| s.bg(rgba(hover_bg)))
        .child(
            div()
                .text_sm()
                .text_color(colors.accent.to_rgb())
                .child(icon),
        )
}

fn render_icon_only_play(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(icon_button(colors, "▶", "play-btn"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_icon_only_arrow(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(icon_button(colors, "→", "arrow-btn"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_icon_only_check(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(icon_button(colors, "✓", "check-btn"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_icon_only_return(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(icon_button(colors, "↵", "return-btn"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_icon_only_circle(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(icon_button(colors, "●", "circle-btn"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_icon_only_double_arrow(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(icon_button(colors, "»", "double-arrow-btn"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

// --- merged from part_04.rs ---

fn render_icon_in_circle(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;
    let circle_bg = (colors.accent << 8) | 0x33;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("icon-circle-btn")
                .w(px(24.))
                .h(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .bg(rgba(circle_bg))
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("▶"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_icon_with_ring(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("icon-ring-btn")
                .w(px(24.))
                .h(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .border_1()
                .border_color(rgba((colors.accent << 8) | 0x60))
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("▶"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// SECTION 3: FIXED-WIDTH BUTTON
// =============================================================================

fn render_fixed_width_60(colors: PromptHeaderColors, text: &str) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("fixed-60")
                .w(px(60.))
                .h(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child(text.to_string()),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_fixed_width_80(colors: PromptHeaderColors, text: &str) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("fixed-80")
                .w(px(80.))
                .h(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(text.to_string()),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_fixed_truncate(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("fixed-truncate")
                .w(px(70.))
                .h(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .overflow_hidden()
                        .text_ellipsis()
                        .child("Open Google Ch..."),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_fixed_with_tooltip(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            // Tooltip simulation (just showing concept)
            div().relative().child(
                div()
                    .id("fixed-tooltip")
                    .w(px(60.))
                    .h(px(24.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(4.))
                    .cursor_pointer()
                    .hover(move |s| s.bg(rgba(hover_bg)))
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors.accent.to_rgb())
                            .child("Open..."),
                    ),
            ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_fixed_pill(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;
    let pill_bg = (colors.accent << 8) | 0x1A;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("fixed-pill")
                .w(px(60.))
                .h(px(22.))
                .flex()
                .items_center()
                .justify_center()
                .bg(rgba(pill_bg))
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_fixed_ghost(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("fixed-ghost")
                .w(px(60.))
                .h(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .border_1()
                .border_color(rgba((colors.accent << 8) | 0x40))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// SECTION 4: POSITIONED AT EDGES
// =============================================================================

fn render_pos_far_right(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
        .child(div().w(px(8.)))
        .child(
            div()
                .id("pos-far-right")
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run ↵"),
                ),
        )
}

fn render_pos_before_logo(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(
            div()
                .id("pos-before-logo")
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("↵"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

// --- merged from part_05.rs ---

fn render_pos_in_input(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            div()
                .mx(px(12.))
                .px(px(12.))
                .py(px(8.))
                .bg(rgba((colors.search_box_bg << 8) | 0x80))
                .rounded(px(6.))
                .flex()
                .flex_row()
                .items_center()
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Search..."),
                )
                .child(
                    div()
                        .id("pos-in-input")
                        .px(px(8.))
                        .py(px(4.))
                        .bg(rgba((colors.accent << 8) | 0x20))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("Run"),
                        ),
                ),
        )
}

fn render_pos_overlap_input(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            div()
                .relative()
                .mx(px(12.))
                .child(
                    div()
                        .px(px(12.))
                        .py(px(8.))
                        .pr(px(60.))
                        .bg(rgba((colors.search_box_bg << 8) | 0x80))
                        .rounded(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("Search..."),
                        ),
                )
                .child(
                    div()
                        .absolute()
                        .right(px(4.))
                        .top(px(4.))
                        .id("pos-overlap")
                        .px(px(8.))
                        .py(px(4.))
                        .bg(colors.accent.to_rgb())
                        .rounded(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x000000))
                                .child("Run"),
                        ),
                ),
        )
}

fn render_pos_below_header(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    div()
        .w_full()
        .flex()
        .flex_col()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            div()
                .h(px(28.))
                .px(px(12.))
                .bg(rgba((colors.background << 8) | 0x80))
                .flex()
                .flex_row()
                .items_center()
                .justify_end()
                .child(
                    div()
                        .id("pos-below")
                        .px(px(8.))
                        .py(px(4.))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("Run ↵"),
                        ),
                ),
        )
}

fn render_pos_floating_br(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(100.))
        .relative()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            div()
                .absolute()
                .bottom(px(8.))
                .right(px(12.))
                .id("pos-floating")
                .px(px(12.))
                .py(px(6.))
                .bg(colors.accent.to_rgb())
                .rounded(px(6.))
                .cursor_pointer()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x000000))
                        .child("Run Script"),
                ),
        )
}

fn render_pos_in_list(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            // Simulated list item with run button
            div()
                .mx(px(8.))
                .px(px(12.))
                .py(px(10.))
                .bg(rgba((colors.accent << 8) | 0x15))
                .rounded(px(6.))
                .flex()
                .flex_row()
                .items_center()
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child("Clipboard History"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("View and manage clipboard"),
                        ),
                )
                .child(
                    div()
                        .id("pos-list")
                        .px(px(8.))
                        .py(px(4.))
                        .bg(rgba((colors.accent << 8) | 0x30))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("Run"),
                        ),
                ),
        )
}

fn render_pos_sticky_footer(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(120.))
        .flex()
        .flex_col()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(div().flex_1())
        .child(
            div()
                .h(px(36.))
                .px(px(12.))
                .bg(rgba((colors.background << 8) | 0xE0))
                .border_t_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Selected: Clipboard History"),
                )
                .child(
                    div()
                        .id("pos-footer")
                        .px(px(12.))
                        .py(px(4.))
                        .bg(colors.accent.to_rgb())
                        .rounded(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x000000))
                                .child("Run"),
                        ),
                ),
        )
}

// =============================================================================
// SECTION 5: COMBINE WITH ACTIONS
// =============================================================================

fn render_actions_merged(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(8.)))
        .child(
            // Combined Actions/Run button
            div()
                .id("merged-actions")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Actions"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("↵/⌘K"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

// --- merged from part_06.rs ---

fn render_split_button(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;
    let divider_color = (colors.accent << 8) | 0x40;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            // Split button
            div()
                .flex()
                .flex_row()
                .items_center()
                .border_1()
                .border_color(rgba(divider_color))
                .rounded(px(4.))
                .overflow_hidden()
                .child(
                    div()
                        .id("split-run")
                        .px(px(8.))
                        .py(px(3.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("Run"),
                        ),
                )
                .child(div().w(px(1.)).h(px(16.)).bg(rgba(divider_color)))
                .child(
                    div()
                        .id("split-more")
                        .px(px(4.))
                        .py(px(3.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("▼"),
                        ),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_pill_plus_more(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;
    let pill_bg = (colors.accent << 8) | 0x1A;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("pill-run")
                .px(px(10.))
                .py(px(3.))
                .bg(rgba(pill_bg))
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run ↵"),
                ),
        )
        .child(div().w(px(4.)))
        .child(
            div()
                .id("pill-more")
                .w(px(22.))
                .h(px(22.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("⋯"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_contextual_primary(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("ctx-primary")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .px(px(8.))
                .py(px(3.))
                .bg(colors.accent.to_rgb())
                .rounded(px(4.))
                .cursor_pointer()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x000000))
                        .child("Run"),
                ),
        )
        .child(div().w(px(4.)))
        .child(
            div()
                .id("ctx-more")
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("More ⌘K"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_two_part(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(2.))
                .child(
                    div()
                        .id("two-icon")
                        .w(px(24.))
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(colors.accent.to_rgb())
                        .rounded(px(4.))
                        .cursor_pointer()
                        .child(div().text_sm().text_color(rgb(0x000000)).child("▶")),
                )
                .child(
                    div()
                        .id("two-dropdown")
                        .w(px(20.))
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("▼"),
                        ),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_expandable_hover(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            // Shows icon normally, expands to "Run ↵" on hover
            div()
                .id("expandable")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.accent.to_rgb())
                        .child("▶"),
                )
                .child(
                    // This would be hidden by default, shown on hover
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_cycle_actions(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            // Click cycles: Run → Edit → Copy → Delete → Run...
            div()
                .id("cycle")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .px(px(8.))
                .py(px(3.))
                .border_1()
                .border_color(rgba((colors.accent << 8) | 0x40))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("⇧ cycle"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_quick_plus_menu(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(8.)))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(2.))
                .child(
                    div()
                        .id("quick-run")
                        .px(px(6.))
                        .py(px(3.))
                        .rounded_l(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
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
                        .id("quick-menu")
                        .px(px(6.))
                        .py(px(3.))
                        .rounded_r(px(4.))
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
                                        .text_xs()
                                        .text_color(colors.accent.to_rgb())
                                        .child("Actions"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_dimmed.to_rgb())
                                        .child("⌘K"),
                                ),
                        ),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// SECTION 6: CONTEXTUAL ICONS
// =============================================================================

fn context_icon_button(
    colors: PromptHeaderColors,
    icon: &'static str,
    id: &'static str,
) -> Stateful<Div> {
    let hover_bg = (colors.accent << 8) | 0x26;

    div()
        .id(id)
        .w(px(28.))
        .h(px(24.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .cursor_pointer()
        .hover(move |s| s.bg(rgba(hover_bg)))
        .child(
            div()
                .text_base()
                .text_color(colors.accent.to_rgb())
                .child(icon),
        )
}

// --- merged from part_07.rs ---

fn render_context_terminal(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "⌘", "ctx-terminal"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_send(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "➤", "ctx-send"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_check(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "✓", "ctx-check"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_launch(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "↗", "ctx-launch"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_folder(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "📁", "ctx-folder"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_globe(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "🌐", "ctx-globe"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_gear(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "⚙", "ctx-gear"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_context_clipboard(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(context_icon_button(colors, "📋", "ctx-clipboard"))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// SECTION 7: TIGHTER BUTTONS
// =============================================================================

fn render_tight_micro(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .id("tight-ai")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(2.))
                .px(px(2.))
                .py(px(2.))
                .rounded(px(2.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("AI"),
                ),
        )
        .child(div().w(px(2.)))
        .child(
            div()
                .id("tight-run")
                .px(px(2.))
                .py(px(2.))
                .rounded(px(2.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("↵"),
                ),
        )
        .child(div().w(px(2.)))
        .child(
            div()
                .id("tight-actions")
                .px(px(2.))
                .py(px(2.))
                .rounded(px(2.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("⌘K"),
                ),
        )
        .child(div().w(px(6.)))
        .child(logo_box())
}

fn render_tight_small(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .id("small-ai")
                .px(px(4.))
                .py(px(2.))
                .rounded(px(3.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("AI Tab"),
                ),
        )
        .child(div().w(px(2.)))
        .child(
            div()
                .id("small-run")
                .px(px(4.))
                .py(px(2.))
                .rounded(px(3.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run"),
                ),
        )
        .child(div().w(px(2.)))
        .child(
            div()
                .id("small-actions")
                .px(px(4.))
                .py(px(2.))
                .rounded(px(3.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("⌘K"),
                ),
        )
        .child(div().w(px(6.)))
        .child(logo_box())
}

fn render_tight_compact(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .id("compact-ai")
                .px(px(6.))
                .py(px(2.))
                .rounded(px(3.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Ask AI"),
                ),
        )
        .child(div().w(px(3.)))
        .child(
            div()
                .id("compact-run")
                .px(px(6.))
                .py(px(2.))
                .rounded(px(3.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("↵"),
                ),
        )
        .child(div().w(px(3.)))
        .child(
            div()
                .id("compact-actions")
                .px(px(6.))
                .py(px(2.))
                .rounded(px(3.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Actions"),
                ),
        )
        .child(div().w(px(6.)))
        .child(logo_box())
}

fn render_tight_text_only(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.accent.to_rgb())
                .child("AI"),
        )
        .child(
            div().w(px(8.)).child(
                div()
                    .text_xs()
                    .text_color(colors.text_dimmed.to_rgb())
                    .child("·"),
            ),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.accent.to_rgb())
                .child("Run"),
        )
        .child(
            div().w(px(8.)).child(
                div()
                    .text_xs()
                    .text_color(colors.text_dimmed.to_rgb())
                    .child("·"),
            ),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors.accent.to_rgb())
                .child("Actions"),
        )
        .child(div().w(px(12.)))
        .child(logo_box())
}

fn render_tight_underline(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div().id("ul-ai").cursor_pointer().child(
                div()
                    .text_xs()
                    .text_color(colors.accent.to_rgb())
                    .hover(|s| s.border_b_1().border_color(rgb(0xFBBF24)))
                    .child("Ask AI"),
            ),
        )
        .child(div().w(px(8.)))
        .child(
            div().id("ul-run").cursor_pointer().child(
                div()
                    .text_xs()
                    .text_color(colors.accent.to_rgb())
                    .child("Run"),
            ),
        )
        .child(div().w(px(8.)))
        .child(
            div().id("ul-actions").cursor_pointer().child(
                div()
                    .text_xs()
                    .text_color(colors.accent.to_rgb())
                    .child("Actions"),
            ),
        )
        .child(div().w(px(12.)))
        .child(logo_box())
}

// --- merged from part_08.rs ---

fn render_tight_badge(colors: PromptHeaderColors) -> impl IntoElement {
    let badge_bg = (colors.accent << 8) | 0x20;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .id("badge-ai")
                .px(px(4.))
                .py(px(1.))
                .bg(rgba(badge_bg))
                .rounded(px(2.))
                .cursor_pointer()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("AI"),
                ),
        )
        .child(div().w(px(4.)))
        .child(
            div()
                .id("badge-run")
                .px(px(4.))
                .py(px(1.))
                .bg(rgba(badge_bg))
                .rounded(px(2.))
                .cursor_pointer()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("↵"),
                ),
        )
        .child(div().w(px(4.)))
        .child(
            div()
                .id("badge-actions")
                .px(px(4.))
                .py(px(1.))
                .bg(rgba(badge_bg))
                .rounded(px(2.))
                .cursor_pointer()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("⌘K"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_tight_link(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div().id("link-ai").cursor_pointer().child(
                div()
                    .text_xs()
                    .text_color(colors.accent.to_rgb())
                    .child("Ask AI"),
            ),
        )
        .child(div().w(px(6.)))
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("|"),
        )
        .child(div().w(px(6.)))
        .child(
            div().id("link-run").cursor_pointer().child(
                div()
                    .text_xs()
                    .text_color(colors.accent.to_rgb())
                    .child("Run"),
            ),
        )
        .child(div().w(px(6.)))
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("|"),
        )
        .child(div().w(px(6.)))
        .child(
            div().id("link-actions").cursor_pointer().child(
                div()
                    .text_xs()
                    .text_color(colors.accent.to_rgb())
                    .child("Actions"),
            ),
        )
        .child(div().w(px(12.)))
        .child(logo_box())
}

fn render_tight_pill(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .id("pill-ai")
                .px(px(6.))
                .h(px(18.))
                .flex()
                .items_center()
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("AI"),
                ),
        )
        .child(div().w(px(2.)))
        .child(
            div()
                .id("pill-run")
                .px(px(6.))
                .h(px(18.))
                .flex()
                .items_center()
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("↵"),
                ),
        )
        .child(div().w(px(2.)))
        .child(
            div()
                .id("pill-actions")
                .px(px(6.))
                .h(px(18.))
                .flex()
                .items_center()
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("⌘K"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// SECTION 8: ALTERNATIVE PLACEMENTS
// =============================================================================

fn render_alt_in_list_item(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            div()
                .mx(px(8.))
                .px(px(12.))
                .py(px(10.))
                .bg(rgba((colors.accent << 8) | 0x15))
                .rounded(px(6.))
                .flex()
                .flex_row()
                .items_center()
                .child(
                    div()
                        .w(px(20.))
                        .h(px(20.))
                        .mr(px(12.))
                        .rounded(px(4.))
                        .bg(rgba((colors.accent << 8) | 0x30)),
                )
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(colors.text_primary.to_rgb())
                        .child("Clipboard History"),
                )
                .child(
                    div()
                        .id("list-run")
                        .px(px(8.))
                        .py(px(4.))
                        .bg(colors.accent.to_rgb())
                        .rounded(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x000000))
                                .child("Run"),
                        ),
                ),
        )
}

fn render_alt_hover_overlay(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(
            div()
                .relative()
                .mx(px(8.))
                .child(
                    div()
                        .px(px(12.))
                        .py(px(10.))
                        .bg(rgba((colors.accent << 8) | 0x15))
                        .rounded(px(6.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(12.))
                        .child(
                            div()
                                .w(px(20.))
                                .h(px(20.))
                                .rounded(px(4.))
                                .bg(rgba((colors.accent << 8) | 0x30)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child("Clipboard History (hover shows Run overlay)"),
                        ),
                )
                .child(
                    // Overlay that would appear on hover
                    div()
                        .absolute()
                        .top_0()
                        .right_0()
                        .bottom_0()
                        .w(px(80.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(rgba((colors.background << 8) | 0xE0))
                        .rounded_r(px(6.))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.accent.to_rgb())
                                .child("Run ↵"),
                        ),
                ),
        )
}

fn render_alt_keyboard_only(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
    // Note: No visual Run button - just Enter key
}

fn render_alt_status_bar(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(100.))
        .flex()
        .flex_col()
        .child(
            header_container(colors)
                .child(script_kit_label(colors))
                .child(div().flex_1())
                .child(ask_ai_button(colors))
                .child(div().w(px(4.)))
                .child(actions_button(colors))
                .child(div().w(px(8.)))
                .child(logo_box()),
        )
        .child(div().flex_1())
        .child(
            div()
                .h(px(24.))
                .px(px(12.))
                .bg(rgba((colors.background << 8) | 0x80))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("↵ Run • Tab AI • ⌘K Actions"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("3 items"),
                ),
        )
}

fn render_alt_right_click(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("Right-click for actions"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_alt_gesture(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("→ swipe to run"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

// --- merged from part_09.rs ---

fn render_alt_double_click(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("Double-click or ↵"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_alt_long_press(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("Hold for actions"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// SECTION 9: VISUAL HIERARCHY
// =============================================================================

fn render_hier_ghost(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x15;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("ghost-run")
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgba((colors.accent << 8) | 0x60))
                        .child("Run ↵"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_hier_muted(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            // Muted by default, brighter on hover (concept - actual would need state)
            div()
                .id("muted-run")
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("↵"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_hier_primary(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("primary-run")
                .px(px(10.))
                .py(px(4.))
                .bg(colors.accent.to_rgb())
                .rounded(px(4.))
                .cursor_pointer()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x000000))
                        .child("Run"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_hier_accent_bg(colors: PromptHeaderColors) -> impl IntoElement {
    let accent_bg = (colors.accent << 8) | 0x30;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("accent-run")
                .px(px(8.))
                .py(px(3.))
                .bg(rgba(accent_bg))
                .rounded(px(4.))
                .cursor_pointer()
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run ↵"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_hier_outline(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x15;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("outline-run")
                .px(px(8.))
                .py(px(3.))
                .border_1()
                .border_color(colors.accent.to_rgb())
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Run"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_hier_gradient(colors: PromptHeaderColors) -> impl IntoElement {
    // Simulated gradient with solid color (GPUI doesn't do gradients easily)
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("gradient-run")
                .px(px(10.))
                .py(px(4.))
                .bg(rgb(0xF59E0B)) // Amber gradient approximation
                .rounded(px(4.))
                .cursor_pointer()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x000000))
                        .child("Run"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_hier_glow(colors: PromptHeaderColors) -> impl IntoElement {
    // Simulated glow with background
    let glow_bg = (colors.accent << 8) | 0x40;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("glow-run")
                .px(px(10.))
                .py(px(5.))
                .bg(rgba(glow_bg))
                .rounded(px(6.))
                .cursor_pointer()
                .child(
                    div()
                        .px(px(8.))
                        .py(px(3.))
                        .bg(colors.accent.to_rgb())
                        .rounded(px(4.))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x000000))
                                .child("Run"),
                        ),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_hier_pulse(colors: PromptHeaderColors) -> impl IntoElement {
    // Simulated pulse (static in storybook, would animate)
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .relative()
                .child(
                    // Pulse ring (would animate opacity)
                    div()
                        .absolute()
                        .inset_0()
                        .bg(rgba((colors.accent << 8) | 0x30))
                        .rounded(px(4.)),
                )
                .child(
                    div()
                        .id("pulse-run")
                        .relative()
                        .px(px(8.))
                        .py(px(3.))
                        .bg(colors.accent.to_rgb())
                        .rounded(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x000000))
                                .child("Run"),
                        ),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

// =============================================================================
// SECTION 10: RECOMMENDED
// =============================================================================

fn render_rec_icon_tooltip(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            // Icon-only, tooltip on hover would show full action
            div()
                .id("rec-icon")
                .w(px(24.))
                .h(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.accent.to_rgb())
                        .child("▶"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_rec_fixed_ghost(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .id("rec-ghost")
                .w(px(50.))
                .h(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .border_1()
                .border_color(rgba((colors.accent << 8) | 0x30))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .overflow_hidden()
                        .text_ellipsis()
                        .child("Run"),
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_rec_no_button(colors: PromptHeaderColors) -> impl IntoElement {
    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(
            div()
                .text_xs()
                .text_color(colors.text_dimmed.to_rgb())
                .child("↵"),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

// --- merged from part_10.rs ---

fn render_rec_contextual_icon(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            // Changes icon based on context: ▶ for scripts, ✓ for select, ➤ for send
            div()
                .id("rec-ctx")
                .w(px(24.))
                .h(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.accent.to_rgb())
                        .child("▶"), // Would change dynamically
                ),
        )
        .child(div().w(px(4.)))
        .child(actions_button(colors))
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_rec_merged(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(8.)))
        .child(
            // Run is first item in Actions menu, button just shows ⌘K/↵
            div()
                .id("rec-merged")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("Actions"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("↵ ⌘K"),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}

fn render_rec_split_compact(colors: PromptHeaderColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x26;
    let divider_color = (colors.accent << 8) | 0x30;

    header_container(colors)
        .child(script_kit_label(colors))
        .child(div().flex_1())
        .child(ask_ai_button(colors))
        .child(div().w(px(4.)))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .border_1()
                .border_color(rgba(divider_color))
                .rounded(px(4.))
                .overflow_hidden()
                .child(
                    div()
                        .id("split-action")
                        .w(px(24.))
                        .h(px(22.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb())
                                .child("▶"),
                        ),
                )
                .child(div().w(px(1.)).h(px(14.)).bg(rgba(divider_color)))
                .child(
                    div()
                        .id("split-menu")
                        .w(px(20.))
                        .h(px(22.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("▼"),
                        ),
                ),
        )
        .child(div().w(px(8.)))
        .child(logo_box())
}
