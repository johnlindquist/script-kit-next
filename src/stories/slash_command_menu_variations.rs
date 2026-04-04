//! Slash Command Menu — Design Variations (Round 4)
//!
//! 21 variations focused on compact list rows. Each row must be ≤40px
//! (matching COMPOSER_H) with text_base (16px) labels matching the input
//! font. Ghost input + vibrancy + tight dropdown.
//!
//! Reference: `src/ai/window/context_picker/render.rs`

use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::storybook::{story_container, story_item, story_section, Story, StoryVariant};
use crate::theme::get_cached_theme;

// ─── Constants ─────────────────────────────────────────────────────────

const MENU_W: f32 = 320.0;
const ROW_H: f32 = 36.0; // ≤ COMPOSER_H (40px), room for text_base
const GOLD: u32 = 0xfbbf24;

const GHOST: f32 = 0.04;
const GHOST_HI: f32 = 0.06;
const HINT: f32 = 0.45;
const MUTED_OP: f32 = 0.65;

fn h(hex: u32) -> Hsla {
    Hsla::from(rgb(hex))
}

// ─── Mock data ─────────────────────────────────────────────────────────

struct SlashItem {
    command: &'static str,
    label: &'static str,
    description: &'static str,
}

const ITEMS: &[SlashItem] = &[
    SlashItem {
        command: "/context",
        label: "Current Context",
        description: "Attach desktop context",
    },
    SlashItem {
        command: "/context-full",
        label: "Full Context",
        description: "Complete desktop snapshot",
    },
    SlashItem {
        command: "/selection",
        label: "Selection",
        description: "Selected text from frontmost app",
    },
    SlashItem {
        command: "/browser",
        label: "Browser URL",
        description: "Current browser URL",
    },
    SlashItem {
        command: "/window",
        label: "Focused Window",
        description: "Window title and bounds",
    },
];

const AC_TYPED: &str = "/con";
const AC_GHOST: &str = "text";
const AC_QUERY: &str = "con";

fn matches_query(item: &SlashItem, query: &str) -> bool {
    let q = query.to_lowercase();
    item.label.to_lowercase().contains(&q) || item.command.to_lowercase().contains(&q)
}

fn split_highlight(label: &str, query: &str) -> (SharedString, SharedString, SharedString) {
    let lower = label.to_lowercase();
    if let Some(start) = lower.find(&query.to_lowercase()) {
        let end = start + query.len();
        (
            label[..start].to_string().into(),
            label[start..end].to_string().into(),
            label[end..].to_string().into(),
        )
    } else {
        (
            label.to_string().into(),
            SharedString::default(),
            SharedString::default(),
        )
    }
}

// ─── Story ─────────────────────────────────────────────────────────────

pub struct SlashCommandMenuVariationsStory;

impl Story for SlashCommandMenuVariationsStory {
    fn id(&self) -> &'static str {
        "slash-command-menu-variations"
    }

    fn name(&self) -> &'static str {
        "Slash Command Menu (21)"
    }

    fn category(&self) -> &'static str {
        "Components"
    }

    fn render(&self) -> AnyElement {
        let variants = self.variants();
        story_container()
            .child(
                story_section("Compact Rows — 36px max, text_base labels").children(
                    variants.iter().enumerate().map(|(i, v)| {
                        story_item(&format!("{}. {}", i + 1, v.name), self.render_variant(v))
                    }),
                ),
            )
            .into_any_element()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        match variant.stable_id().as_str() {
            "v01" => render_v01(),
            "v02" => render_v02(),
            "v03" => render_v03(),
            "v04" => render_v04(),
            "v05" => render_v05(),
            "v06" => render_v06(),
            "v07" => render_v07(),
            "v08" => render_v08(),
            "v09" => render_v09(),
            "v10" => render_v10(),
            "v11" => render_v11(),
            "v12" => render_v12(),
            "v13" => render_v13(),
            "v14" => render_v14(),
            "v15" => render_v15(),
            "v16" => render_v16(),
            "v17" => render_v17(),
            "v18" => render_v18(),
            "v19" => render_v19(),
            "v20" => render_v20(),
            "v21" => render_v21(),
            _ => render_v01(),
        }
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            // ── Row structure ──
            StoryVariant::default_named("v01", "Baseline Compact")
                .description("36px rows, text_base label, gold bar, /cmd right, ghost input above"),
            StoryVariant::default_named("v02", "No Command Hint")
                .description("Label only — no /command on the right, maximally clean"),
            StoryVariant::default_named("v03", "Command Left of Label")
                .description("/command shown in hint before the label, reading order"),
            StoryVariant::default_named("v04", "Inline Desc After Dash")
                .description("Label — desc on one line, both at text_base, desc dimmed"),
            StoryVariant::default_named("v05", "Desc Below in xs")
                .description("Two-line: text_base label + text_xs desc, still ≤40px"),
            // ── Gold bar variations ──
            StoryVariant::default_named("v06", "Thick Gold Bar")
                .description("3px wide gold bar, taller, more prominent"),
            StoryVariant::default_named("v07", "Gold Left Edge")
                .description("Gold bar flush to container edge, no left padding"),
            StoryVariant::default_named("v08", "Gold Tint Row")
                .description("Selected row has warm gold ghost bg instead of neutral"),
            StoryVariant::default_named("v09", "No Bar, Bold Label")
                .description("No gold bar — selection via ghost bg + medium weight label"),
            StoryVariant::default_named("v10", "Gold Underline")
                .description("No left bar — gold underline beneath selected label instead"),
            // ── Search highlight ──
            StoryVariant::default_named("v11", "Gold Text Match")
                .description("'con' highlighted in gold within labels"),
            StoryVariant::default_named("v12", "Bold Match")
                .description("Matched chars bold, same color as rest"),
            StoryVariant::default_named("v13", "Gold Match + Desc")
                .description("Gold highlight + desc below, still ≤40px"),
            // ── Keyboard hints ──
            StoryVariant::default_named("v14", "Tab Pill Right")
                .description("Tab pill right-aligned on selected row"),
            StoryVariant::default_named("v15", "Tab + Cmd Right")
                .description("/command + Tab pill, compact spacing"),
            StoryVariant::default_named("v16", "Footer Hints")
                .description("↑↓ Tab Esc hint strip below dropdown"),
            // ── Structure ──
            StoryVariant::default_named("v17", "Section Headers")
                .description("CONTEXT / SOURCES section labels between groups"),
            StoryVariant::default_named("v18", "Result Count")
                .description("'3 results' in the input row"),
            // ── Edge states ──
            StoryVariant::default_named("v19", "Empty: Chips")
                .description("No matches — gold hint chips"),
            StoryVariant::default_named("v20", "Single Match")
                .description("One result — shown prominently with Tab pill"),
            StoryVariant::default_named("v21", "Dimmed Non-Matches")
                .description("Non-matching items visible but at ghost opacity"),
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Shared helpers
// ═══════════════════════════════════════════════════════════════════════

fn shell() -> Div {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    div().w(px(MENU_W)).bg(fg.opacity(0.02)).flex().flex_col()
}

fn input_bar(fg: Hsla, muted: Hsla, gold: Hsla) -> Div {
    div().h(px(ROW_H)).px(px(12.)).flex().items_center().child(
        div()
            .flex()
            .items_center()
            .child(div().text_color(fg).child(AC_TYPED))
            .child(div().text_color(muted.opacity(0.3)).child(AC_GHOST))
            .child(div().w(px(1.5)).h(px(16.)).ml(px(1.)).bg(gold.opacity(0.6))),
    )
}

fn input_bar_with_right(fg: Hsla, muted: Hsla, gold: Hsla, right: Div) -> Div {
    div()
        .h(px(ROW_H))
        .px(px(12.))
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .child(div().text_color(fg).child(AC_TYPED))
                .child(div().text_color(muted.opacity(0.3)).child(AC_GHOST))
                .child(div().w(px(1.5)).h(px(16.)).ml(px(1.)).bg(gold.opacity(0.6))),
        )
        .child(right)
}

fn hair(fg: Hsla) -> Div {
    div().h(px(1.)).bg(fg.opacity(GHOST))
}

fn gbar(selected: bool, gold: Hsla, w: f32, ht: f32) -> Div {
    div()
        .w(px(w))
        .h(px(ht))
        .rounded(px(1.))
        .bg(if selected { gold } else { transparent_black() })
}

fn tab_pill(gold: Hsla) -> Div {
    div()
        .px(px(4.))
        .py(px(1.))
        .rounded(px(3.))
        .bg(gold.opacity(0.12))
        .text_xs()
        .text_color(gold.opacity(0.8))
        .font_weight(FontWeight::MEDIUM)
        .child("Tab")
}

fn hint_chip(text: &str, gold: Hsla) -> Div {
    div()
        .px(px(5.))
        .py(px(1.))
        .rounded(px(3.))
        .bg(gold.opacity(0.08))
        .text_xs()
        .text_color(gold.opacity(0.7))
        .child(text.to_string())
}

fn sect(text: &str, muted: Hsla) -> Div {
    div()
        .px(px(12.))
        .h(px(24.))
        .flex()
        .items_center()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(muted.opacity(HINT))
        .child(text.to_string())
}

/// Standard compact row: 36px, text_base label, gold bar, optional right content
fn row(item: &SlashItem, selected: bool, fg: Hsla, gold: Hsla) -> Div {
    div()
        .h(px(ROW_H))
        .flex()
        .items_center()
        .justify_between()
        .px(px(10.))
        .bg(if selected {
            fg.opacity(GHOST)
        } else {
            transparent_black()
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(gbar(selected, gold, 2., 16.))
                .child(
                    div()
                        .text_color(if selected { fg } else { fg.opacity(MUTED_OP) })
                        .text_ellipsis()
                        .child(item.label),
                ),
        )
}

// ═══════════════════════════════════════════════════════════════════════
// V01: Baseline — compact 36px rows, label + /cmd right
// ═══════════════════════════════════════════════════════════════════════

fn render_v01() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    row(item, i == 0, fg, gold).child(
                        div()
                            .text_xs()
                            .text_color(muted.opacity(if i == 0 { HINT } else { 0.3 }))
                            .child(item.command),
                    )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V02: No command hint — label only
// ═══════════════════════════════════════════════════════════════════════

fn render_v02() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div().flex().flex_col().children(
                matched
                    .iter()
                    .enumerate()
                    .map(|(i, item)| row(item, i == 0, fg, gold)),
            ),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V03: Command before label — "/context  Current Context"
// ═══════════════════════════════════════════════════════════════════════

fn render_v03() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .h(px(ROW_H))
                        .flex()
                        .items_center()
                        .px(px(10.))
                        .bg(if selected {
                            fg.opacity(GHOST)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(gbar(selected, gold, 2., 16.))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted.opacity(if selected {
                                            HINT
                                        } else {
                                            0.3
                                        }))
                                        .w(px(80.))
                                        .text_ellipsis()
                                        .child(item.command),
                                )
                                .child(
                                    div()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
                                        .text_ellipsis()
                                        .child(item.label),
                                ),
                        )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V04: Inline desc after dash — "Current Context — Attach desktop context"
// ═══════════════════════════════════════════════════════════════════════

fn render_v04() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .h(px(ROW_H))
                        .flex()
                        .items_center()
                        .px(px(10.))
                        .overflow_hidden()
                        .bg(if selected {
                            fg.opacity(GHOST)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .flex_1()
                                .overflow_hidden()
                                .child(gbar(selected, gold, 2., 16.))
                                .child(
                                    div()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
                                        .flex_shrink_0()
                                        .child(item.label),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted.opacity(if selected {
                                            0.4
                                        } else {
                                            0.25
                                        }))
                                        .child("—"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted.opacity(if selected {
                                            0.4
                                        } else {
                                            0.25
                                        }))
                                        .text_ellipsis()
                                        .child(item.description),
                                ),
                        )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V05: Two-line — text_base label + text_xs desc, ≤40px
// ═══════════════════════════════════════════════════════════════════════

fn render_v05() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .max_h(px(40.))
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .py(px(4.))
                        .bg(if selected {
                            fg.opacity(GHOST)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(gbar(selected, gold, 2., 18.))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .text_color(if selected {
                                                    fg
                                                } else {
                                                    fg.opacity(MUTED_OP)
                                                })
                                                .text_ellipsis()
                                                .line_height(px(18.))
                                                .child(item.label),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(muted.opacity(if selected {
                                                    HINT
                                                } else {
                                                    0.3
                                                }))
                                                .text_ellipsis()
                                                .line_height(px(14.))
                                                .child(item.description),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted.opacity(if selected { HINT } else { 0.25 }))
                                .child(item.command),
                        )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V06: Thick gold bar — 3px × 20px
// ═══════════════════════════════════════════════════════════════════════

fn render_v06() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .h(px(ROW_H))
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .bg(if selected {
                            fg.opacity(GHOST)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(gbar(selected, gold, 3., 20.))
                                .child(
                                    div()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
                                        .text_ellipsis()
                                        .child(item.label),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                                .child(item.command),
                        )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V07: Gold bar flush left — no padding before bar
// ═══════════════════════════════════════════════════════════════════════

fn render_v07() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .h(px(ROW_H))
                        .flex()
                        .items_center()
                        .justify_between()
                        .pr(px(10.))
                        .bg(if selected {
                            fg.opacity(GHOST)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.))
                                .child(gbar(selected, gold, 2., ROW_H))
                                .child(
                                    div()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
                                        .text_ellipsis()
                                        .child(item.label),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                                .child(item.command),
                        )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V08: Gold tint bg — warm ghost on selected
// ═══════════════════════════════════════════════════════════════════════

fn render_v08() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .h(px(ROW_H))
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .bg(if selected {
                            gold.opacity(GHOST_HI)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(gbar(selected, gold, 2., 16.))
                                .child(
                                    div()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
                                        .text_ellipsis()
                                        .child(item.label),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                                .child(item.command),
                        )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V09: No bar — ghost bg + medium weight label
// ═══════════════════════════════════════════════════════════════════════

fn render_v09() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .h(px(ROW_H))
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(12.))
                        .bg(if selected {
                            fg.opacity(GHOST)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .text_color(if selected { fg } else { fg.opacity(MUTED_OP) })
                                .font_weight(if selected {
                                    FontWeight::MEDIUM
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_ellipsis()
                                .child(item.label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                                .child(item.command),
                        )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V10: Gold underline — no left bar, underline on selected label
// ═══════════════════════════════════════════════════════════════════════

fn render_v10() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .h(px(ROW_H))
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(12.))
                        .bg(if selected {
                            fg.opacity(GHOST)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .text_color(if selected { fg } else { fg.opacity(MUTED_OP) })
                                .text_ellipsis()
                                .when(selected, |d| d.border_b_1().border_color(gold))
                                .child(item.label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                                .child(item.command),
                        )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V11: Gold text highlight on query match
// ═══════════════════════════════════════════════════════════════════════

fn render_v11() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    let text_color = if selected { fg } else { fg.opacity(MUTED_OP) };
                    let (before, mid, after) = split_highlight(item.label, AC_QUERY);
                    div()
                        .h(px(ROW_H))
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .bg(if selected {
                            fg.opacity(GHOST)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(gbar(selected, gold, 2., 16.))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .child(div().text_color(text_color).child(before))
                                        .child(
                                            div()
                                                .text_color(gold)
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .child(mid),
                                        )
                                        .child(div().text_color(text_color).child(after)),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                                .child(item.command),
                        )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V12: Bold match — same color, heavier weight
// ═══════════════════════════════════════════════════════════════════════

fn render_v12() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    let text_color = if selected { fg } else { fg.opacity(MUTED_OP) };
                    let (before, mid, after) = split_highlight(item.label, AC_QUERY);
                    div()
                        .h(px(ROW_H))
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .bg(if selected {
                            fg.opacity(GHOST)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(gbar(selected, gold, 2., 16.))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .child(div().text_color(text_color).child(before))
                                        .child(
                                            div()
                                                .text_color(text_color)
                                                .font_weight(FontWeight::BOLD)
                                                .child(mid),
                                        )
                                        .child(div().text_color(text_color).child(after)),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                                .child(item.command),
                        )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V13: Gold match highlight + desc below (≤40px)
// ═══════════════════════════════════════════════════════════════════════

fn render_v13() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    let text_color = if selected { fg } else { fg.opacity(MUTED_OP) };
                    let (before, mid, after) = split_highlight(item.label, AC_QUERY);
                    div()
                        .max_h(px(40.))
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .py(px(4.))
                        .bg(if selected {
                            fg.opacity(GHOST)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(gbar(selected, gold, 2., 18.))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .line_height(px(18.))
                                                .child(div().text_color(text_color).child(before))
                                                .child(
                                                    div()
                                                        .text_color(gold)
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .child(mid),
                                                )
                                                .child(div().text_color(text_color).child(after)),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(muted.opacity(if selected {
                                                    HINT
                                                } else {
                                                    0.3
                                                }))
                                                .text_ellipsis()
                                                .line_height(px(14.))
                                                .child(item.description),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted.opacity(if selected { HINT } else { 0.25 }))
                                .child(item.command),
                        )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V14: Tab pill right-aligned on selected
// ═══════════════════════════════════════════════════════════════════════

fn render_v14() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    row(item, selected, fg, gold).child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                                    .child(item.command),
                            )
                            .when(selected, |d| d.child(tab_pill(gold))),
                    )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V15: Tab + /cmd, compact
// ═══════════════════════════════════════════════════════════════════════

fn render_v15() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    row(item, selected, fg, gold).child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.))
                            .when(selected, |d| {
                                d.child(tab_pill(gold)).child(
                                    div()
                                        .px(px(4.))
                                        .py(px(1.))
                                        .rounded(px(3.))
                                        .bg(fg.opacity(GHOST_HI))
                                        .text_xs()
                                        .text_color(muted.opacity(HINT))
                                        .font_weight(FontWeight::MEDIUM)
                                        .child("↵"),
                                )
                            })
                            .when(!selected, |d| {
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(muted.opacity(0.3))
                                        .child(item.command.to_string()),
                                )
                            }),
                    )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V16: Footer hint strip
// ═══════════════════════════════════════════════════════════════════════

fn render_v16() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    row(item, i == 0, fg, gold).child(
                        div()
                            .text_xs()
                            .text_color(muted.opacity(if i == 0 { HINT } else { 0.3 }))
                            .child(item.command),
                    )
                })),
        )
        .child(hair(fg))
        .child(
            div()
                .h(px(24.))
                .px(px(12.))
                .flex()
                .items_center()
                .gap(px(12.))
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(0.3))
                        .child("↑↓ navigate"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(0.3))
                        .child("Tab complete"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(0.3))
                        .child("Esc close"),
                ),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V17: Section headers
// ═══════════════════════════════════════════════════════════════════════

fn render_v17() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    let snapshots: Vec<&SlashItem> = ITEMS[0..2]
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();
    let sources: Vec<&SlashItem> = ITEMS[2..]
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    let mut list = div().flex().flex_col();
    if !snapshots.is_empty() {
        list = list.child(sect("CONTEXT", muted));
        for (i, item) in snapshots.iter().enumerate() {
            list = list.child(
                row(item, i == 0, fg, gold).child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(if i == 0 { HINT } else { 0.3 }))
                        .child(item.command.to_string()),
                ),
            );
        }
    }
    if !sources.is_empty() {
        list = list.child(sect("SOURCES", muted));
        for item in &sources {
            list = list.child(
                row(item, false, fg, gold).child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(0.3))
                        .child(item.command.to_string()),
                ),
            );
        }
    }

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(list)
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V18: Result count in input row
// ═══════════════════════════════════════════════════════════════════════

fn render_v18() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    shell()
        .child(input_bar_with_right(
            fg,
            muted,
            gold,
            div()
                .text_xs()
                .text_color(muted.opacity(0.35))
                .child(format!("{} results", matched.len())),
        ))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    row(item, i == 0, fg, gold).child(
                        div()
                            .text_xs()
                            .text_color(muted.opacity(if i == 0 { HINT } else { 0.3 }))
                            .child(item.command),
                    )
                })),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V19: Empty state — hint chips
// ═══════════════════════════════════════════════════════════════════════

fn render_v19() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    shell()
        .child(
            div().h(px(ROW_H)).px(px(12.)).flex().items_center().child(
                div()
                    .flex()
                    .items_center()
                    .child(div().text_color(fg).child("/xyz"))
                    .child(div().w(px(1.5)).h(px(16.)).ml(px(1.)).bg(gold.opacity(0.6))),
            ),
        )
        .child(hair(fg))
        .child(
            div()
                .py(px(12.))
                .px(px(12.))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(MUTED_OP))
                        .child("No matching commands"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.))
                        .child(div().text_xs().text_color(muted.opacity(HINT)).child("Try"))
                        .child(hint_chip("/context", gold))
                        .child(hint_chip("/selection", gold))
                        .child(hint_chip("/browser", gold)),
                ),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V20: Single match — prominent with Tab
// ═══════════════════════════════════════════════════════════════════════

fn render_v20() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);
    let single = &ITEMS[2]; // "Selection"

    shell()
        .child(
            div().h(px(ROW_H)).px(px(12.)).flex().items_center().child(
                div()
                    .flex()
                    .items_center()
                    .child(div().text_color(fg).child("/sel"))
                    .child(div().text_color(muted.opacity(0.3)).child("ection"))
                    .child(div().w(px(1.5)).h(px(16.)).ml(px(1.)).bg(gold.opacity(0.6))),
            ),
        )
        .child(hair(fg))
        .child(
            row(single, true, fg, gold).child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted.opacity(HINT))
                            .child(single.command),
                    )
                    .child(tab_pill(gold)),
            ),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V21: Dimmed non-matches below matches
// ═══════════════════════════════════════════════════════════════════════

fn render_v21() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    shell()
        .child(input_bar(fg, muted, gold))
        .child(hair(fg))
        .child(
            div()
                .flex()
                .flex_col()
                .children(ITEMS.iter().enumerate().map(|(i, item)| {
                    let is_match = matches_query(item, AC_QUERY);
                    let selected = i == 0 && is_match;
                    let dim = if is_match { 1.0 } else { 0.25 };

                    div()
                        .h(px(ROW_H))
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .bg(if selected {
                            fg.opacity(GHOST)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(gbar(selected, gold, 2., 16.))
                                .child(
                                    div()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP * dim)
                                        })
                                        .text_ellipsis()
                                        .child(item.label),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted.opacity(0.3 * dim))
                                .child(item.command),
                        )
                })),
        )
        .into_any_element()
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::SlashCommandMenuVariationsStory;
    use crate::storybook::Story;

    #[test]
    fn slash_command_story_has_21_variants() {
        let story = SlashCommandMenuVariationsStory;
        assert_eq!(story.variants().len(), 21);
    }

    #[test]
    fn all_variant_ids_are_unique() {
        let story = SlashCommandMenuVariationsStory;
        let ids: Vec<String> = story.variants().iter().map(|v| v.stable_id()).collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len(), "duplicate variant IDs: {ids:?}");
    }
}
