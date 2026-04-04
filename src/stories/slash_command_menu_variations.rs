//! Slash Command Menu — Design Variations (Round 2)
//!
//! 21 variations exploring four themes the user selected:
//! 1. Inline Autocomplete — ghost completion text in input
//! 2. Search Highlight — gold highlight on query matches
//! 3. Vibrancy — transparent bg, vibrancy bleed-through
//! 4. Description Always Visible — two-line rows
//!
//! Each variation mixes these themes at different intensities.
//! Reference: `src/ai/window/context_picker/render.rs`

use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::storybook::{story_container, story_item, story_section, Story, StoryVariant};
use crate::theme::get_cached_theme;

// ─── Layout constants ──────────────────────────────────────────────────

const MENU_W: f32 = 320.0;
const GOLD: u32 = 0xfbbf24;

// Impeccable opacity tiers
const GHOST: f32 = 0.04;
const GHOST_HI: f32 = 0.06;
const HINT: f32 = 0.45;
const MUTED_OP: f32 = 0.65;
const PRESENT: f32 = 0.90;

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
        description: "Attach minimal desktop context",
    },
    SlashItem {
        command: "/context-full",
        label: "Full Context",
        description: "Attach complete desktop context",
    },
    SlashItem {
        command: "/selection",
        label: "Selection",
        description: "Attach selected text from frontmost app",
    },
    SlashItem {
        command: "/browser",
        label: "Browser URL",
        description: "Attach current browser URL",
    },
    SlashItem {
        command: "/window",
        label: "Focused Window",
        description: "Attach focused window title and bounds",
    },
];

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
                story_section("All Variants — Inline / Search / Vibrancy / Descriptions").children(
                    variants.iter().enumerate().map(|(i, v)| {
                        story_item(&format!("{}. {}", i + 1, v.name), self.render_variant(v))
                    }),
                ),
            )
            .into_any_element()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let id = variant.stable_id();
        match id.as_str() {
            "v01-vibrancy-desc" => render_v01(),
            "v02-vibrancy-desc-gold-bar" => render_v02(),
            "v03-vibrancy-desc-sections" => render_v03(),
            "v04-vibrancy-desc-tab-badge" => render_v04(),
            "v05-vibrancy-desc-compact" => render_v05(),
            "v06-search-gold-highlight" => render_v06(),
            "v07-search-underline" => render_v07(),
            "v08-search-bold-match" => render_v08(),
            "v09-search-desc-highlight" => render_v09(),
            "v10-search-grouped" => render_v10(),
            "v11-autocomplete-ghost" => render_v11(),
            "v12-autocomplete-tab-pill" => render_v12(),
            "v13-autocomplete-desc" => render_v13(),
            "v14-autocomplete-dimmed-rest" => render_v14(),
            "v15-autocomplete-inline-only" => render_v15(),
            "v16-full-vibrancy-search-desc" => render_v16(),
            "v17-full-autocomplete-search" => render_v17(),
            "v18-full-all-four" => render_v18(),
            "v19-empty-state-hints" => render_v19(),
            "v20-empty-state-recent" => render_v20(),
            "v21-full-all-four-dense" => render_v21(),
            _ => render_v01(),
        }
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            // ── Theme 1: Vibrancy + Description Always ──
            StoryVariant::default_named("v01-vibrancy-desc", "Vibrancy + Descriptions")
                .description("Transparent bg, two-line rows always visible, gold bar on focus"),
            StoryVariant::default_named(
                "v02-vibrancy-desc-gold-bar",
                "Vibrancy + Desc + Gold Fill",
            )
            .description("Same as V1 but selected row gets a subtle gold-tinted bg"),
            StoryVariant::default_named("v03-vibrancy-desc-sections", "Vibrancy + Desc + Sections")
                .description("Grouped sections (Snapshots/Sources), descriptions always, vibrancy"),
            StoryVariant::default_named(
                "v04-vibrancy-desc-tab-badge",
                "Vibrancy + Desc + Tab Badge",
            )
            .description("Tab completion badge on selected row, descriptions visible"),
            StoryVariant::default_named("v05-vibrancy-desc-compact", "Vibrancy + Desc Compact")
                .description(
                    "Tighter vertical rhythm, descriptions in hint opacity, no wasted space",
                ),
            // ── Theme 2: Search Highlight ──
            StoryVariant::default_named("v06-search-gold-highlight", "Search: Gold Text Highlight")
                .description("Query 'con' highlighted in gold within matching labels"),
            StoryVariant::default_named("v07-search-underline", "Search: Gold Underline")
                .description("Matching chars get gold underline instead of color change"),
            StoryVariant::default_named("v08-search-bold-match", "Search: Bold Match")
                .description("Matched portion rendered in semibold, rest stays normal weight"),
            StoryVariant::default_named(
                "v09-search-desc-highlight",
                "Search: Highlight + Descriptions",
            )
            .description("Gold highlight in labels, descriptions always visible below"),
            StoryVariant::default_named("v10-search-grouped", "Search: Highlight + Sections")
                .description("Gold highlight with section headers, vibrancy bg"),
            // ── Theme 3: Inline Autocomplete ──
            StoryVariant::default_named("v11-autocomplete-ghost", "Autocomplete: Ghost Text")
                .description("Input shows /con with ghost 'text' completion, menu below"),
            StoryVariant::default_named(
                "v12-autocomplete-tab-pill",
                "Autocomplete: Ghost + Tab Pill",
            )
            .description("Ghost completion + Tab pill on selected row"),
            StoryVariant::default_named(
                "v13-autocomplete-desc",
                "Autocomplete: Ghost + Descriptions",
            )
            .description("Ghost completion with descriptions always visible in menu"),
            StoryVariant::default_named(
                "v14-autocomplete-dimmed-rest",
                "Autocomplete: Dimmed Non-Matches",
            )
            .description("Top match highlighted, non-matching items at ghost opacity"),
            StoryVariant::default_named(
                "v15-autocomplete-inline-only",
                "Autocomplete: Inline Only",
            )
            .description("No dropdown at all — ghost text in input, cycle with Up/Down"),
            // ── Theme 4: All Four Combined ──
            StoryVariant::default_named(
                "v16-full-vibrancy-search-desc",
                "Full: Vibrancy + Search + Desc",
            )
            .description("Vibrancy bg, gold search highlights, descriptions always visible"),
            StoryVariant::default_named(
                "v17-full-autocomplete-search",
                "Full: Autocomplete + Search",
            )
            .description("Ghost input completion + gold highlights in dropdown"),
            StoryVariant::default_named("v18-full-all-four", "Full: All Four Themes")
                .description("Ghost input + gold highlights + vibrancy + descriptions + Tab badge"),
            StoryVariant::default_named("v19-empty-state-hints", "Empty: No Matches + Hints")
                .description("Empty state with command hint chips when query has no results"),
            StoryVariant::default_named("v20-empty-state-recent", "Empty: No Matches + Recents")
                .description("Empty state showing recently used commands as suggestions"),
            StoryVariant::default_named("v21-full-all-four-dense", "Full: All Four Dense")
                .description(
                    "All four themes in a dense, compact layout — maximum info, minimum space",
                ),
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════

fn gold_bar(selected: bool, gold: Hsla) -> Div {
    div()
        .w(px(2.))
        .h(px(14.))
        .rounded(px(1.))
        .bg(if selected { gold } else { transparent_black() })
}

fn gold_bar_tall(selected: bool, gold: Hsla) -> Div {
    div()
        .w(px(2.))
        .h(px(18.))
        .rounded(px(1.))
        .bg(if selected { gold } else { transparent_black() })
}

/// Splits `label` around the first occurrence of `query` (case-insensitive)
/// and returns (before, matched, after) as SharedStrings.
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

fn mock_input(typed: &str, ghost: &str, fg: Hsla, muted: Hsla, gold: Hsla) -> Div {
    div().px(px(12.)).py(px(8.)).flex().items_center().child(
        div()
            .flex()
            .items_center()
            .child(div().text_sm().text_color(fg).child(typed.to_string()))
            .child(
                div()
                    .text_sm()
                    .text_color(muted.opacity(0.3))
                    .child(ghost.to_string()),
            )
            .child(div().w(px(1.5)).h(px(16.)).ml(px(1.)).bg(gold.opacity(0.6))),
    )
}

fn hairline(fg: Hsla) -> Div {
    div().h(px(1.)).bg(fg.opacity(GHOST))
}

fn section_label(text: &str, muted: Hsla) -> Div {
    div()
        .px(px(12.))
        .pt(px(6.))
        .pb(px(2.))
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(muted.opacity(HINT))
        .child(text.to_string())
}

fn tab_badge(gold: Hsla) -> Div {
    div()
        .px(px(5.))
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

fn vibrancy_shell() -> Div {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    div()
        .w(px(MENU_W))
        .bg(fg.opacity(0.02))
        .py(px(3.))
        .flex()
        .flex_col()
}

fn vibrancy_shell_no_pad() -> Div {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    div().w(px(MENU_W)).bg(fg.opacity(0.02)).flex().flex_col()
}

// ═══════════════════════════════════════════════════════════════════════
// V01–V05: Vibrancy + Description Always
// ═══════════════════════════════════════════════════════════════════════

// V01: Pure vibrancy + descriptions always visible
fn render_v01() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    vibrancy_shell()
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .px(px(10.))
                .py(px(5.))
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
                        .child(gold_bar_tall(selected, gold))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
                                        .text_ellipsis()
                                        .child(item.label),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(if selected {
                                            muted.opacity(MUTED_OP)
                                        } else {
                                            muted.opacity(0.35)
                                        })
                                        .text_ellipsis()
                                        .child(item.description),
                                ),
                        ),
                )
        }))
        .into_any_element()
}

// V02: Vibrancy + descriptions + gold-tinted fill on selection
fn render_v02() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    vibrancy_shell()
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(10.))
                .py(px(5.))
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
                        .child(gold_bar_tall(selected, gold))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
                                        .text_ellipsis()
                                        .child(item.label),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(if selected {
                                            muted.opacity(MUTED_OP)
                                        } else {
                                            muted.opacity(0.35)
                                        })
                                        .text_ellipsis()
                                        .child(item.description),
                                ),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

// V03: Vibrancy + descriptions + section headers
fn render_v03() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    vibrancy_shell()
        .child(section_label("CONTEXT SNAPSHOTS", muted))
        .children(
            ITEMS[0..2]
                .iter()
                .enumerate()
                .map(|(i, item)| desc_row(item, i == 0, fg, muted, gold, true)),
        )
        .child(div().h(px(4.)))
        .child(section_label("TARGET SOURCES", muted))
        .children(
            ITEMS[2..]
                .iter()
                .map(|item| desc_row(item, false, fg, muted, gold, true)),
        )
        .into_any_element()
}

// V04: Vibrancy + descriptions + Tab badge
fn render_v04() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    vibrancy_shell()
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(10.))
                .py(px(5.))
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
                        .child(gold_bar_tall(selected, gold))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
                                        .text_ellipsis()
                                        .child(item.label),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(if selected {
                                            muted.opacity(MUTED_OP)
                                        } else {
                                            muted.opacity(0.35)
                                        })
                                        .text_ellipsis()
                                        .child(item.description),
                                ),
                        ),
                )
                .child(
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
                        .when(selected, |d| d.child(tab_badge(gold))),
                )
        }))
        .into_any_element()
}

// V05: Vibrancy + descriptions compact — tighter rhythm
fn render_v05() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    vibrancy_shell()
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(8.))
                .py(px(3.))
                .bg(if selected {
                    fg.opacity(GHOST)
                } else {
                    transparent_black()
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(gold_bar(selected, gold))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .overflow_hidden()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
                                        .text_ellipsis()
                                        .child(item.label),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted.opacity(if selected {
                                            HINT
                                        } else {
                                            0.25
                                        }))
                                        .child("—"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted.opacity(if selected {
                                            HINT
                                        } else {
                                            0.25
                                        }))
                                        .text_ellipsis()
                                        .child(item.description),
                                ),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(if selected { HINT } else { 0.2 }))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

fn desc_row(
    item: &SlashItem,
    selected: bool,
    fg: Hsla,
    muted: Hsla,
    gold: Hsla,
    show_cmd: bool,
) -> Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .px(px(10.))
        .py(px(5.))
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
                .child(gold_bar_tall(selected, gold))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected { fg } else { fg.opacity(MUTED_OP) })
                                .text_ellipsis()
                                .child(item.label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(if selected {
                                    muted.opacity(MUTED_OP)
                                } else {
                                    muted.opacity(0.35)
                                })
                                .text_ellipsis()
                                .child(item.description),
                        ),
                ),
        )
        .when(show_cmd, |d| {
            d.child(
                div()
                    .text_xs()
                    .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                    .child(item.command.to_string()),
            )
        })
}

// ═══════════════════════════════════════════════════════════════════════
// V06–V10: Search Highlight
// ═══════════════════════════════════════════════════════════════════════

const SEARCH_QUERY: &str = "con";

fn matches_query(item: &SlashItem, query: &str) -> bool {
    let q = query.to_lowercase();
    item.label.to_lowercase().contains(&q) || item.command.to_lowercase().contains(&q)
}

/// Render label with gold-highlighted match spans
fn highlighted_label(label: &str, query: &str, fg: Hsla, gold: Hsla, selected: bool) -> Div {
    let (before, matched, after) = split_highlight(label, query);
    div()
        .flex()
        .items_center()
        .text_sm()
        .child(
            div()
                .text_color(if selected { fg } else { fg.opacity(MUTED_OP) })
                .child(before),
        )
        .child(
            div()
                .text_color(gold)
                .font_weight(FontWeight::SEMIBOLD)
                .child(matched),
        )
        .child(
            div()
                .text_color(if selected { fg } else { fg.opacity(MUTED_OP) })
                .child(after),
        )
}

// V06: Gold text highlight on query match
fn render_v06() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, SEARCH_QUERY))
        .collect();

    vibrancy_shell()
        .child(
            div()
                .px(px(12.))
                .py(px(4.))
                .flex()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(HINT))
                        .child("Filter:"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(gold)
                        .font_weight(FontWeight::MEDIUM)
                        .child(format!("/{SEARCH_QUERY}")),
                ),
        )
        .children(matched.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(10.))
                .py(px(5.))
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
                        .child(gold_bar(selected, gold))
                        .child(highlighted_label(
                            item.label,
                            SEARCH_QUERY,
                            fg,
                            gold,
                            selected,
                        )),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

// V07: Gold underline on matched chars instead of color change
fn render_v07() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, SEARCH_QUERY))
        .collect();

    vibrancy_shell()
        .children(matched.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            let (before, match_text, after) = split_highlight(item.label, SEARCH_QUERY);
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(10.))
                .py(px(5.))
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
                        .child(gold_bar(selected, gold))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .text_sm()
                                .child(
                                    div()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
                                        .child(before),
                                )
                                .child(
                                    div()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
                                        .border_b_1()
                                        .border_color(gold)
                                        .child(match_text),
                                )
                                .child(
                                    div()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
                                        .child(after),
                                ),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

// V08: Bold match text, same color
fn render_v08() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, SEARCH_QUERY))
        .collect();

    vibrancy_shell()
        .children(matched.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            let (before, match_text, after) = split_highlight(item.label, SEARCH_QUERY);
            let text_color = if selected { fg } else { fg.opacity(MUTED_OP) };
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(10.))
                .py(px(5.))
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
                        .child(gold_bar(selected, gold))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .text_sm()
                                .child(
                                    div()
                                        .text_color(text_color)
                                        .font_weight(FontWeight::NORMAL)
                                        .child(before),
                                )
                                .child(
                                    div()
                                        .text_color(text_color)
                                        .font_weight(FontWeight::BOLD)
                                        .child(match_text),
                                )
                                .child(
                                    div()
                                        .text_color(text_color)
                                        .font_weight(FontWeight::NORMAL)
                                        .child(after),
                                ),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

// V09: Gold highlight + descriptions always visible
fn render_v09() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, SEARCH_QUERY))
        .collect();

    vibrancy_shell()
        .children(matched.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(10.))
                .py(px(5.))
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
                        .child(gold_bar_tall(selected, gold))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .child(highlighted_label(
                                    item.label,
                                    SEARCH_QUERY,
                                    fg,
                                    gold,
                                    selected,
                                ))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(if selected {
                                            muted.opacity(MUTED_OP)
                                        } else {
                                            muted.opacity(0.35)
                                        })
                                        .text_ellipsis()
                                        .child(item.description),
                                ),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

// V10: Gold highlight + sections
fn render_v10() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    let snapshots: Vec<&SlashItem> = ITEMS[0..2]
        .iter()
        .filter(|i| matches_query(i, SEARCH_QUERY))
        .collect();
    let sources: Vec<&SlashItem> = ITEMS[2..]
        .iter()
        .filter(|i| matches_query(i, SEARCH_QUERY))
        .collect();

    let mut shell = vibrancy_shell();

    if !snapshots.is_empty() {
        shell = shell.child(section_label("CONTEXT SNAPSHOTS", muted));
        for (i, item) in snapshots.iter().enumerate() {
            let selected = i == 0;
            shell = shell.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(10.))
                    .py(px(5.))
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
                            .child(gold_bar(selected, gold))
                            .child(highlighted_label(
                                item.label,
                                SEARCH_QUERY,
                                fg,
                                gold,
                                selected,
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                            .child(item.command),
                    ),
            );
        }
    }

    if !sources.is_empty() {
        shell = shell.child(div().h(px(4.)));
        shell = shell.child(section_label("TARGET SOURCES", muted));
        for item in &sources {
            shell = shell.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(10.))
                    .py(px(5.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(gold_bar(false, gold))
                            .child(highlighted_label(item.label, SEARCH_QUERY, fg, gold, false)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted.opacity(0.3))
                            .child(item.command),
                    ),
            );
        }
    }

    shell.into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V11–V15: Inline Autocomplete
// ═══════════════════════════════════════════════════════════════════════

const AC_TYPED: &str = "/con";
const AC_GHOST: &str = "text";
const AC_QUERY: &str = "con";

// V11: Ghost completion text in input + dropdown
fn render_v11() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    vibrancy_shell_no_pad()
        .child(mock_input(AC_TYPED, AC_GHOST, fg, muted, gold))
        .child(hairline(fg))
        .child(
            div()
                .py(px(3.))
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .py(px(5.))
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
                                .child(gold_bar(selected, gold))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
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

// V12: Ghost completion + Tab pill on selected
fn render_v12() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    vibrancy_shell_no_pad()
        .child(mock_input(AC_TYPED, AC_GHOST, fg, muted, gold))
        .child(hairline(fg))
        .child(
            div()
                .py(px(3.))
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .py(px(5.))
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
                                .child(gold_bar(selected, gold))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if selected {
                                            fg
                                        } else {
                                            fg.opacity(MUTED_OP)
                                        })
                                        .child(item.label),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted.opacity(if selected {
                                            HINT
                                        } else {
                                            0.3
                                        }))
                                        .child(item.command),
                                )
                                .when(selected, |d| d.child(tab_badge(gold))),
                        )
                })),
        )
        .into_any_element()
}

// V13: Ghost completion + descriptions always visible
fn render_v13() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    vibrancy_shell_no_pad()
        .child(mock_input(AC_TYPED, AC_GHOST, fg, muted, gold))
        .child(hairline(fg))
        .child(
            div()
                .py(px(3.))
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .py(px(5.))
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
                                .child(gold_bar_tall(selected, gold))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(if selected {
                                                    fg
                                                } else {
                                                    fg.opacity(MUTED_OP)
                                                })
                                                .text_ellipsis()
                                                .child(item.label),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(if selected {
                                                    muted.opacity(MUTED_OP)
                                                } else {
                                                    muted.opacity(0.35)
                                                })
                                                .text_ellipsis()
                                                .child(item.description),
                                        ),
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

// V14: Ghost completion + non-matching items dimmed to ghost
fn render_v14() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    vibrancy_shell_no_pad()
        .child(mock_input(AC_TYPED, AC_GHOST, fg, muted, gold))
        .child(hairline(fg))
        .child(
            div()
                .py(px(3.))
                .flex()
                .flex_col()
                .children(ITEMS.iter().enumerate().map(|(i, item)| {
                    let is_match = matches_query(item, AC_QUERY);
                    let selected = i == 0 && is_match;
                    let row_opacity = if is_match { 1.0 } else { 0.25 };
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .py(px(if is_match { 5. } else { 3. }))
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
                                .child(gold_bar(selected, gold))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(fg.opacity(if selected {
                                            PRESENT
                                        } else {
                                            row_opacity * MUTED_OP
                                        }))
                                        .child(item.label),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted.opacity(row_opacity * 0.3))
                                .child(item.command),
                        )
                })),
        )
        .into_any_element()
}

// V15: Inline only — no dropdown, ghost text in input, Up/Down hint
fn render_v15() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    div()
        .w(px(MENU_W))
        .bg(fg.opacity(0.02))
        .flex()
        .flex_col()
        // Input with ghost completion
        .child(
            div()
                .px(px(12.))
                .py(px(10.))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .child(div().text_sm().text_color(fg).child("/con"))
                        .child(div().text_sm().text_color(muted.opacity(0.3)).child("text"))
                        .child(div().w(px(1.5)).h(px(16.)).ml(px(1.)).bg(gold.opacity(0.6))),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted.opacity(0.3))
                                .child("↑↓ cycle"),
                        )
                        .child(tab_badge(gold)),
                ),
        )
        // Description of current selection
        .child(hairline(fg))
        .child(
            div().px(px(12.)).py(px(6.)).child(
                div()
                    .text_xs()
                    .text_color(muted.opacity(HINT))
                    .child("Attach minimal desktop context"),
            ),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// V16–V21: Combined Themes
// ═══════════════════════════════════════════════════════════════════════

// V16: Vibrancy + search highlight + descriptions
fn render_v16() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, SEARCH_QUERY))
        .collect();

    vibrancy_shell()
        .child(
            div()
                .px(px(12.))
                .py(px(3.))
                .flex()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(0.35))
                        .child("Showing results for"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(gold)
                        .child(format!("/{SEARCH_QUERY}")),
                ),
        )
        .children(matched.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(10.))
                .py(px(5.))
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
                        .child(gold_bar_tall(selected, gold))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .child(highlighted_label(
                                    item.label,
                                    SEARCH_QUERY,
                                    fg,
                                    gold,
                                    selected,
                                ))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(if selected {
                                            muted.opacity(MUTED_OP)
                                        } else {
                                            muted.opacity(0.35)
                                        })
                                        .text_ellipsis()
                                        .child(item.description),
                                ),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

// V17: Autocomplete + search highlight
fn render_v17() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    vibrancy_shell_no_pad()
        .child(mock_input(AC_TYPED, AC_GHOST, fg, muted, gold))
        .child(hairline(fg))
        .child(
            div()
                .py(px(3.))
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .py(px(5.))
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
                                .child(gold_bar(selected, gold))
                                .child(highlighted_label(item.label, AC_QUERY, fg, gold, selected)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted.opacity(if selected {
                                            HINT
                                        } else {
                                            0.3
                                        }))
                                        .child(item.command),
                                )
                                .when(selected, |d| d.child(tab_badge(gold))),
                        )
                })),
        )
        .into_any_element()
}

// V18: All four — autocomplete + highlight + vibrancy + descriptions + Tab
fn render_v18() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    vibrancy_shell_no_pad()
        .child(mock_input(AC_TYPED, AC_GHOST, fg, muted, gold))
        .child(hairline(fg))
        .child(
            div()
                .py(px(3.))
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .py(px(5.))
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
                                .child(gold_bar_tall(selected, gold))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .overflow_hidden()
                                        .child(highlighted_label(
                                            item.label, AC_QUERY, fg, gold, selected,
                                        ))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(if selected {
                                                    muted.opacity(MUTED_OP)
                                                } else {
                                                    muted.opacity(0.35)
                                                })
                                                .text_ellipsis()
                                                .child(item.description),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted.opacity(if selected {
                                            HINT
                                        } else {
                                            0.3
                                        }))
                                        .child(item.command),
                                )
                                .when(selected, |d| d.child(tab_badge(gold))),
                        )
                })),
        )
        .into_any_element()
}

// V19: Empty state with command hint chips
fn render_v19() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    vibrancy_shell_no_pad()
        .child(mock_input("/xyz", "", fg, muted, gold))
        .child(hairline(fg))
        .child(
            div()
                .py(px(16.))
                .px(px(16.))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .text_sm()
                        .text_color(muted.opacity(MUTED_OP))
                        .child("No matching commands"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(div().text_xs().text_color(muted.opacity(HINT)).child("Try"))
                        .child(hint_chip("/context", gold))
                        .child(hint_chip("/selection", gold))
                        .child(hint_chip("/browser", gold)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(0.35))
                        .child("or type @ to attach files"),
                ),
        )
        .into_any_element()
}

// V20: Empty state with recently used commands
fn render_v20() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    vibrancy_shell_no_pad()
        .child(mock_input("/xyz", "", fg, muted, gold))
        .child(hairline(fg))
        .child(
            div()
                .py(px(8.))
                .flex()
                .flex_col()
                .child(
                    div()
                        .px(px(12.))
                        .py(px(4.))
                        .text_xs()
                        .text_color(muted.opacity(HINT))
                        .child("No matches — recently used:"),
                )
                // Recent items shown at muted opacity
                .children(ITEMS[0..3].iter().map(|item| {
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.))
                        .py(px(4.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(gold_bar(false, gold))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(fg.opacity(HINT))
                                                .text_ellipsis()
                                                .child(item.label),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(muted.opacity(0.3))
                                                .text_ellipsis()
                                                .child(item.description),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted.opacity(0.25))
                                .child(item.command),
                        )
                })),
        )
        .into_any_element()
}

// V21: All four themes, dense compact layout
fn render_v21() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    let matched: Vec<&SlashItem> = ITEMS
        .iter()
        .filter(|i| matches_query(i, AC_QUERY))
        .collect();

    vibrancy_shell_no_pad()
        // Compact input
        .child(
            div()
                .px(px(10.))
                .py(px(6.))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .child(div().text_xs().text_color(fg).child("/con"))
                        .child(div().text_xs().text_color(muted.opacity(0.3)).child("text"))
                        .child(div().w(px(1.)).h(px(12.)).ml(px(1.)).bg(gold.opacity(0.6))),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(0.25))
                        .child(format!("{} results", matched.len())),
                ),
        )
        .child(hairline(fg))
        // Dense rows
        .child(
            div()
                .py(px(2.))
                .flex()
                .flex_col()
                .children(matched.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(8.))
                        .py(px(3.))
                        .bg(if selected {
                            fg.opacity(GHOST)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .child(
                                    div().w(px(1.5)).h(px(12.)).rounded(px(1.)).bg(if selected {
                                        gold
                                    } else {
                                        transparent_black()
                                    }),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .overflow_hidden()
                                        .child(highlighted_label(
                                            item.label, AC_QUERY, fg, gold, selected,
                                        ))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(if selected {
                                                    muted.opacity(HINT)
                                                } else {
                                                    muted.opacity(0.25)
                                                })
                                                .text_ellipsis()
                                                .child(item.description),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted.opacity(if selected {
                                            0.35
                                        } else {
                                            0.2
                                        }))
                                        .child(item.command),
                                )
                                .when(selected, |d| {
                                    d.child(
                                        div()
                                            .px(px(4.))
                                            .py(px(0.))
                                            .rounded(px(2.))
                                            .bg(gold.opacity(0.10))
                                            .text_xs()
                                            .text_color(gold.opacity(0.7))
                                            .font_weight(FontWeight::MEDIUM)
                                            .child("⇥"),
                                    )
                                }),
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
