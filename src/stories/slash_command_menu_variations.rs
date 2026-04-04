//! Slash Command Menu — Design Variations (Round 5)
//!
//! 21 variations aligned with the @-Mention Picker "Dense Monoline" design:
//! ~20px rows, text_xs, gold bar 2×10, label left, /command right in mono,
//! no bg change on selection — just gold bar + text color promotion.
//!
//! Reference: `at_mention_picker_variations.rs` v15_dense_monoline()

use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::list_item::FONT_MONO;
use crate::storybook::{story_container, story_item, story_section, Story, StoryVariant};
use crate::theme::get_cached_theme;

// ─── Constants ─────────────────────────────────────────────────────────

const MENU_W: f32 = 320.0;
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
                story_section("Dense Monoline — ~20px rows, text_xs, mono /cmd").children(
                    variants
                        .iter()
                        .enumerate()
                        .map(|(i, v)| story_item(&format!("{}. {}", i + 1, v.name), self.render_variant(v))),
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
            // ── Core layout ──
            StoryVariant::default_named("v01", "Exact Dense Monoline")
                .description("Pixel-perfect match: 1px pad, text_xs, 2×10 bar, mono /cmd at 0.30"),
            StoryVariant::default_named("v02", "Dense + Ghost Input")
                .description("Same rows with ghost autocomplete input above"),
            StoryVariant::default_named("v03", "Dense + Ghost Bg")
                .description("Add ghost bg (0.04) on selected row"),
            StoryVariant::default_named("v04", "Dense + 2px Pad")
                .description("Slightly more room — 2px vertical padding per row"),
            StoryVariant::default_named("v05", "Dense + 3px Pad")
                .description("3px vertical — still tight but more clickable"),

            // ── Bar variations ──
            StoryVariant::default_named("v06", "Bar 2×14")
                .description("Taller gold bar: 2×14px instead of 2×10"),
            StoryVariant::default_named("v07", "Bar 3×10")
                .description("Wider gold bar: 3×10px"),
            StoryVariant::default_named("v08", "Bar Flush Left")
                .description("Bar touches left edge, full row height, no left pad"),
            StoryVariant::default_named("v09", "No Bar")
                .description("No gold bar — text color promotion only"),
            StoryVariant::default_named("v10", "Gold Tint Row")
                .description("Gold-tinted ghost bg instead of neutral"),

            // ── Command/label ──
            StoryVariant::default_named("v11", "Cmd in text_xs")
                .description("/command in text_xs sans-serif instead of mono"),
            StoryVariant::default_named("v12", "Cmd Before Label")
                .description("/context  Current Context — command first reading order"),
            StoryVariant::default_named("v13", "Label + Desc Inline")
                .description("Label — desc on one line, desc even dimmer"),

            // ── Search ──
            StoryVariant::default_named("v14", "Gold Match")
                .description("Query 'con' highlighted in gold within labels"),
            StoryVariant::default_named("v15", "Bold Match")
                .description("Matched chars bold, same color"),
            StoryVariant::default_named("v16", "Dimmed Non-Matches")
                .description("Non-matching items at 0.15 opacity below matches"),

            // ── Keyboard ──
            StoryVariant::default_named("v17", "Tab Pill")
                .description("Tab pill on selected row, right of /cmd"),
            StoryVariant::default_named("v18", "Footer Hints")
                .description("↑↓ Tab Esc strip below dropdown"),

            // ── Edge ──
            StoryVariant::default_named("v19", "Empty: Chips")
                .description("No matches — hint chips"),
            StoryVariant::default_named("v20", "Single Match")
                .description("One result with ghost completion + Tab"),
            StoryVariant::default_named("v21", "Sections")
                .description("CONTEXT / SOURCES section labels"),
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

fn input_row(fg: Hsla, dim: Hsla, gold: Hsla) -> Div {
    div()
        .px(px(8.))
        .py(px(6.))
        .flex()
        .items_center()
        .child(
            div()
                .flex()
                .items_center()
                .child(div().text_xs().text_color(fg).child(AC_TYPED))
                .child(div().text_xs().text_color(dim.opacity(0.30)).child(AC_GHOST))
                .child(div().w(px(1.)).h(px(12.)).ml(px(1.)).bg(gold.opacity(0.6))),
        )
}

fn hair(fg: Hsla) -> Div {
    div().h(px(1.)).bg(fg.opacity(GHOST))
}

fn gbar(sel: bool, gold: Hsla, w: f32, ht: f32) -> Div {
    div()
        .w(px(w))
        .h(px(ht))
        .rounded(px(1.))
        .bg(if sel { gold } else { transparent_black() })
}

fn tab_pill(gold: Hsla) -> Div {
    div()
        .px(px(3.))
        .py(px(0.))
        .rounded(px(2.))
        .bg(gold.opacity(0.12))
        .text_xs()
        .text_color(gold.opacity(0.8))
        .font_weight(FontWeight::MEDIUM)
        .child("Tab")
}

fn hint_chip(text: &str, gold: Hsla) -> Div {
    div()
        .px(px(4.))
        .py(px(1.))
        .rounded(px(2.))
        .bg(gold.opacity(0.08))
        .text_xs()
        .text_color(gold.opacity(0.7))
        .child(text.to_string())
}

fn sect(text: &str, dim: Hsla) -> Div {
    div()
        .px(px(6.))
        .pt(px(4.))
        .pb(px(1.))
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(dim.opacity(HINT))
        .child(text.to_string())
}

/// The dense monoline row: text_xs label left, /cmd right in mono.
/// Gold bar 2×10 on selected. No bg change.
fn dense_row(item: &SlashItem, sel: bool, fg: Hsla, dim: Hsla, gold: Hsla) -> Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .px(px(6.))
        .py(px(1.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(4.))
                .child(gbar(sel, gold, 2., 10.))
                .child(
                    div()
                        .text_xs()
                        .text_color(if sel { fg } else { dim })
                        .child(item.label),
                ),
        )
        .child(
            div()
                .text_xs()
                .font_family(FONT_MONO)
                .text_color(dim.opacity(0.30))
                .child(item.command),
        )
}

// ═══════════════════════════════════════════════════════════════════════
// V01: Exact Dense Monoline (pixel-match of at-mention v15)
// ═══════════════════════════════════════════════════════════════════════

fn render_v01() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    div()
        .w(px(MENU_W))
        .py(px(1.))
        .flex()
        .flex_col()
        .children(
            matched
                .iter()
                .enumerate()
                .map(|(i, item)| dense_row(item, i == 0, fg, dim, gold)),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V02: Dense + ghost input above
// ═══════════════════════════════════════════════════════════════════════

fn render_v02() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched
                    .iter()
                    .enumerate()
                    .map(|(i, item)| dense_row(item, i == 0, fg, dim, gold)),
            ),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V03: Dense + ghost bg on selected
// ═══════════════════════════════════════════════════════════════════════

fn render_v03() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| {
                    let sel = i == 0;
                    dense_row(item, sel, fg, dim, gold)
                        .bg(if sel { fg.opacity(GHOST) } else { transparent_black() })
                }),
            ),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V04: Dense + 2px pad
// ═══════════════════════════════════════════════════════════════════════

fn render_v04() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| {
                    let sel = i == 0;
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(6.))
                        .py(px(2.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(4.))
                                .child(gbar(sel, gold, 2., 10.))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(if sel { fg } else { dim })
                                        .child(item.label),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_family(FONT_MONO)
                                .text_color(dim.opacity(0.30))
                                .child(item.command),
                        )
                }),
            ),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V05: Dense + 3px pad + ghost bg
// ═══════════════════════════════════════════════════════════════════════

fn render_v05() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| {
                    let sel = i == 0;
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(6.))
                        .py(px(3.))
                        .bg(if sel { fg.opacity(GHOST) } else { transparent_black() })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(4.))
                                .child(gbar(sel, gold, 2., 12.))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(if sel { fg } else { dim })
                                        .child(item.label),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_family(FONT_MONO)
                                .text_color(dim.opacity(0.30))
                                .child(item.command),
                        )
                }),
            ),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V06–V10: Bar variations
// ═══════════════════════════════════════════════════════════════════════

fn render_v06() -> AnyElement { render_bar_variant(2., 14.) }
fn render_v07() -> AnyElement { render_bar_variant(3., 10.) }

fn render_bar_variant(bar_w: f32, bar_h: f32) -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| {
                    let sel = i == 0;
                    div()
                        .flex().items_center().justify_between()
                        .px(px(6.)).py(px(1.))
                        .child(
                            div().flex().items_center().gap(px(4.))
                                .child(gbar(sel, gold, bar_w, bar_h))
                                .child(div().text_xs().text_color(if sel { fg } else { dim }).child(item.label)),
                        )
                        .child(div().text_xs().font_family(FONT_MONO).text_color(dim.opacity(0.30)).child(item.command))
                }),
            ),
        )
        .into_any_element()
}

fn render_v08() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| {
                    let sel = i == 0;
                    div()
                        .flex().items_center().justify_between()
                        .pr(px(6.)).py(px(1.))
                        .child(
                            div().flex().items_center().gap(px(6.))
                                .child(gbar(sel, gold, 2., 18.))
                                .child(div().text_xs().text_color(if sel { fg } else { dim }).child(item.label)),
                        )
                        .child(div().text_xs().font_family(FONT_MONO).text_color(dim.opacity(0.30)).child(item.command))
                }),
            ),
        )
        .into_any_element()
}

fn render_v09() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| {
                    let sel = i == 0;
                    div()
                        .flex().items_center().justify_between()
                        .px(px(8.)).py(px(1.))
                        .child(
                            div().text_xs()
                                .text_color(if sel { fg } else { dim })
                                .font_weight(if sel { FontWeight::MEDIUM } else { FontWeight::NORMAL })
                                .child(item.label),
                        )
                        .child(div().text_xs().font_family(FONT_MONO).text_color(dim.opacity(0.30)).child(item.command))
                }),
            ),
        )
        .into_any_element()
}

fn render_v10() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| {
                    let sel = i == 0;
                    dense_row(item, sel, fg, dim, gold)
                        .bg(if sel { gold.opacity(GHOST_HI) } else { transparent_black() })
                }),
            ),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V11–V13: Command/label variations
// ═══════════════════════════════════════════════════════════════════════

fn render_v11() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| {
                    let sel = i == 0;
                    div()
                        .flex().items_center().justify_between()
                        .px(px(6.)).py(px(1.))
                        .child(
                            div().flex().items_center().gap(px(4.))
                                .child(gbar(sel, gold, 2., 10.))
                                .child(div().text_xs().text_color(if sel { fg } else { dim }).child(item.label)),
                        )
                        .child(div().text_xs().text_color(dim.opacity(0.30)).child(item.command))
                }),
            ),
        )
        .into_any_element()
}

fn render_v12() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| {
                    let sel = i == 0;
                    div()
                        .flex().items_center()
                        .px(px(6.)).py(px(1.))
                        .child(
                            div().flex().items_center().gap(px(4.))
                                .child(gbar(sel, gold, 2., 10.))
                                .child(div().text_xs().font_family(FONT_MONO).text_color(dim.opacity(0.40)).w(px(90.)).child(item.command))
                                .child(div().text_xs().text_color(if sel { fg } else { dim }).child(item.label)),
                        )
                }),
            ),
        )
        .into_any_element()
}

fn render_v13() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| {
                    let sel = i == 0;
                    div()
                        .flex().items_center()
                        .px(px(6.)).py(px(1.)).overflow_hidden()
                        .child(
                            div().flex().items_center().gap(px(4.)).flex_1().overflow_hidden()
                                .child(gbar(sel, gold, 2., 10.))
                                .child(div().text_xs().text_color(if sel { fg } else { dim }).flex_shrink_0().child(item.label))
                                .child(div().text_xs().text_color(dim.opacity(0.20)).text_ellipsis().child(format!("— {}", item.description))),
                        )
                }),
            ),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V14–V16: Search
// ═══════════════════════════════════════════════════════════════════════

fn render_v14() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| {
                    let sel = i == 0;
                    let tc = if sel { fg } else { dim };
                    let (before, mid, after) = split_highlight(item.label, AC_QUERY);
                    div()
                        .flex().items_center().justify_between()
                        .px(px(6.)).py(px(1.))
                        .child(
                            div().flex().items_center().gap(px(4.))
                                .child(gbar(sel, gold, 2., 10.))
                                .child(
                                    div().flex().items_center().text_xs()
                                        .child(div().text_color(tc).child(before))
                                        .child(div().text_color(gold).font_weight(FontWeight::SEMIBOLD).child(mid))
                                        .child(div().text_color(tc).child(after)),
                                ),
                        )
                        .child(div().text_xs().font_family(FONT_MONO).text_color(dim.opacity(0.30)).child(item.command))
                }),
            ),
        )
        .into_any_element()
}

fn render_v15() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| {
                    let sel = i == 0;
                    let tc = if sel { fg } else { dim };
                    let (before, mid, after) = split_highlight(item.label, AC_QUERY);
                    div()
                        .flex().items_center().justify_between()
                        .px(px(6.)).py(px(1.))
                        .child(
                            div().flex().items_center().gap(px(4.))
                                .child(gbar(sel, gold, 2., 10.))
                                .child(
                                    div().flex().items_center().text_xs()
                                        .child(div().text_color(tc).child(before))
                                        .child(div().text_color(tc).font_weight(FontWeight::BOLD).child(mid))
                                        .child(div().text_color(tc).child(after)),
                                ),
                        )
                        .child(div().text_xs().font_family(FONT_MONO).text_color(dim.opacity(0.30)).child(item.command))
                }),
            ),
        )
        .into_any_element()
}

fn render_v16() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                ITEMS.iter().enumerate().map(|(i, item)| {
                    let is_match = matches_query(item, AC_QUERY);
                    let sel = i == 0 && is_match;
                    let op = if is_match { 1.0 } else { 0.15 };
                    div()
                        .flex().items_center().justify_between()
                        .px(px(6.)).py(px(1.))
                        .child(
                            div().flex().items_center().gap(px(4.))
                                .child(gbar(sel, gold, 2., 10.))
                                .child(div().text_xs().text_color(if sel { fg } else { dim.opacity(op) }).child(item.label)),
                        )
                        .child(div().text_xs().font_family(FONT_MONO).text_color(dim.opacity(0.30 * op)).child(item.command))
                }),
            ),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V17–V18: Keyboard hints
// ═══════════════════════════════════════════════════════════════════════

fn render_v17() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| {
                    let sel = i == 0;
                    div()
                        .flex().items_center().justify_between()
                        .px(px(6.)).py(px(1.))
                        .child(
                            div().flex().items_center().gap(px(4.))
                                .child(gbar(sel, gold, 2., 10.))
                                .child(div().text_xs().text_color(if sel { fg } else { dim }).child(item.label)),
                        )
                        .child(
                            div().flex().items_center().gap(px(4.))
                                .child(div().text_xs().font_family(FONT_MONO).text_color(dim.opacity(0.30)).child(item.command))
                                .when(sel, |d| d.child(tab_pill(gold))),
                        )
                }),
            ),
        )
        .into_any_element()
}

fn render_v18() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let matched: Vec<&SlashItem> = ITEMS.iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().children(
                matched.iter().enumerate().map(|(i, item)| dense_row(item, i == 0, fg, dim, gold)),
            ),
        )
        .child(hair(fg))
        .child(
            div().px(px(6.)).py(px(2.)).flex().items_center().gap(px(10.))
                .child(div().text_xs().text_color(dim.opacity(0.25)).child("↑↓"))
                .child(div().text_xs().text_color(dim.opacity(0.25)).child("Tab select"))
                .child(div().text_xs().text_color(dim.opacity(0.25)).child("Esc close")),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V19–V21: Edge states
// ═══════════════════════════════════════════════════════════════════════

fn render_v19() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);

    shell()
        .child(
            div().px(px(8.)).py(px(6.)).flex().items_center()
                .child(
                    div().flex().items_center()
                        .child(div().text_xs().text_color(fg).child("/xyz"))
                        .child(div().w(px(1.)).h(px(12.)).ml(px(1.)).bg(gold.opacity(0.6))),
                ),
        )
        .child(hair(fg))
        .child(
            div().py(px(6.)).px(px(8.)).flex().flex_col().gap(px(4.))
                .child(div().text_xs().text_color(dim.opacity(MUTED_OP)).child("No matches"))
                .child(
                    div().flex().items_center().gap(px(3.))
                        .child(hint_chip("/context", gold))
                        .child(hint_chip("/selection", gold))
                        .child(hint_chip("/browser", gold)),
                ),
        )
        .into_any_element()
}

fn render_v20() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);
    let single = &ITEMS[2];

    shell()
        .child(
            div().px(px(8.)).py(px(6.)).flex().items_center()
                .child(
                    div().flex().items_center()
                        .child(div().text_xs().text_color(fg).child("/sel"))
                        .child(div().text_xs().text_color(dim.opacity(0.30)).child("ection"))
                        .child(div().w(px(1.)).h(px(12.)).ml(px(1.)).bg(gold.opacity(0.6))),
                ),
        )
        .child(hair(fg))
        .child(
            div().py(px(1.)).flex().flex_col().child(
                div().flex().items_center().justify_between()
                    .px(px(6.)).py(px(1.))
                    .child(
                        div().flex().items_center().gap(px(4.))
                            .child(gbar(true, gold, 2., 10.))
                            .child(div().text_xs().text_color(fg).child(single.label)),
                    )
                    .child(
                        div().flex().items_center().gap(px(4.))
                            .child(div().text_xs().font_family(FONT_MONO).text_color(dim.opacity(0.30)).child(single.command))
                            .child(tab_pill(gold)),
                    ),
            ),
        )
        .into_any_element()
}

fn render_v21() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let gold = h(GOLD);

    let snapshots: Vec<&SlashItem> = ITEMS[0..2].iter().filter(|i| matches_query(i, AC_QUERY)).collect();
    let sources: Vec<&SlashItem> = ITEMS[2..].iter().filter(|i| matches_query(i, AC_QUERY)).collect();

    let mut list = div().py(px(1.)).flex().flex_col();
    if !snapshots.is_empty() {
        list = list.child(sect("CONTEXT", dim));
        for (i, item) in snapshots.iter().enumerate() {
            list = list.child(dense_row(item, i == 0, fg, dim, gold));
        }
    }
    if !sources.is_empty() {
        list = list.child(sect("SOURCES", dim));
        for item in &sources {
            list = list.child(dense_row(item, false, fg, dim, gold));
        }
    }

    shell()
        .child(input_row(fg, dim, gold))
        .child(hair(fg))
        .child(list)
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
