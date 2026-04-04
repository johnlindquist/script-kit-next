//! Slash Command Menu — Design Variations
//!
//! 14 visual treatments for the slash command dropdown. Variants 6–14 follow
//! .impeccable.md whisper chrome: ghost backgrounds (0.03–0.06), gold accent
//! bar, spacing-only structure, hint-opacity secondary text.
//!
//! Reference: current implementation in `src/ai/window/context_picker/render.rs`

use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::storybook::{
    story_container, story_divider, story_item, story_section, Story, StoryVariant,
};
use crate::theme::get_cached_theme;
use crate::theme::opacity::{OPACITY_BORDER, OPACITY_HOVER, OPACITY_SELECTED, OPACITY_TEXT_MUTED};

// ─── Layout constants ──────────────────────────────────────────────────

const MENU_W: f32 = 320.0;
const MENU_MAX_H: f32 = 280.0;
const ICON_SZ: f32 = 14.0;
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
    icon_char: &'static str,
}

const ITEMS: &[SlashItem] = &[
    SlashItem {
        command: "/context",
        label: "Current Context",
        description: "Attach minimal desktop context",
        icon_char: "◎",
    },
    SlashItem {
        command: "/context-full",
        label: "Full Context",
        description: "Attach complete desktop context",
        icon_char: "◉",
    },
    SlashItem {
        command: "/selection",
        label: "Selection",
        description: "Attach selected text",
        icon_char: "▋",
    },
    SlashItem {
        command: "/browser",
        label: "Browser URL",
        description: "Attach current browser URL",
        icon_char: "◆",
    },
    SlashItem {
        command: "/window",
        label: "Focused Window",
        description: "Attach focused window info",
        icon_char: "▢",
    },
];

// ─── Story ─────────────────────────────────────────────────────────────

pub struct SlashCommandMenuVariationsStory;

impl Story for SlashCommandMenuVariationsStory {
    fn id(&self) -> &'static str {
        "slash-command-menu-variations"
    }

    fn name(&self) -> &'static str {
        "Slash Command Menu (15)"
    }

    fn category(&self) -> &'static str {
        "Components"
    }

    fn render(&self) -> AnyElement {
        let variants = self.variants();

        story_container()
            .child(story_section("All Variants — '/' Typed").children(
                variants.iter().enumerate().map(|(i, v)| {
                    story_item(&format!("{}. {}", i + 1, v.name), self.render_variant(v))
                }),
            ))
            .into_any_element()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let id = variant.stable_id();
        match id.as_str() {
            "current" => render_current(),
            "compact-tags" => render_compact_tags(),
            "gold-accent" => render_gold_accent(),
            "grouped-sections" => render_grouped_sections(),
            "raycast-polish" => render_raycast_polish(),
            "whisper-bare" => render_whisper_bare(),
            "ghost-float" => render_ghost_float(),
            "hairline-flush" => render_hairline_flush(),
            "gold-only" => render_gold_only(),
            "monoline-hint" => render_monoline_hint(),
            "dot-separator" => render_dot_separator(),
            "vibrancy-panel" => render_vibrancy_panel(),
            "ultra-dense" => render_ultra_dense(),
            "list-anatomy" => render_list_anatomy(),
            "vibrancy-monoline" => render_vibrancy_monoline(),
            _ => render_current(),
        }
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            // Original 5
            StoryVariant::default_named("current", "Current (Baseline)")
                .description("Existing picker — accent tint, icon + label + subtitle"),
            StoryVariant::default_named("compact-tags", "Compact Tags")
                .description("Single-line rows with right-aligned command pill"),
            StoryVariant::default_named("gold-accent", "Gold Accent Bar")
                .description("Gold left bar on selected, description on focus only"),
            StoryVariant::default_named("grouped-sections", "Grouped Sections")
                .description("Section headers with spacing-only dividers"),
            StoryVariant::default_named("raycast-polish", "Raycast Polish")
                .description("Icon pills, gold bar, description on focus"),
            // 9 new minimal variants
            StoryVariant::default_named("whisper-bare", "Whisper Bare")
                .description("No icons, no border. Just text + gold bar. Maximum minimalism."),
            StoryVariant::default_named("ghost-float", "Ghost Float")
                .description("Ghost-opacity bg on focus, no container border, bare text only"),
            StoryVariant::default_named("hairline-flush", "Hairline Flush")
                .description("Gold bar flush left, hairline container, tight vertical rhythm"),
            StoryVariant::default_named("gold-only", "Gold Bar Only")
                .description("Gold bar is sole visual affordance. No bg change. No icons."),
            StoryVariant::default_named("monoline-hint", "Monoline Hint")
                .description("One line per row: label left, /command right in hint opacity"),
            StoryVariant::default_named("dot-separator", "Dot Separator")
                .description("Label · /command on one line, gold bar, no icons"),
            StoryVariant::default_named("vibrancy-panel", "Vibrancy Panel")
                .description("Transparent bg — rows float on vibrancy. Gold bar + text."),
            StoryVariant::default_named("ultra-dense", "Ultra Dense")
                .description("Tiny text, 2px padding, maximum items in minimum space"),
            StoryVariant::default_named("list-anatomy", "List Anatomy")
                .description("Exact .impeccable.md list item spec: gold bar, name, desc on focus, hint metadata"),
            StoryVariant::default_named("vibrancy-monoline", "Vibrancy Monoline")
                .description("Transparent bg for vibrancy bleed-through, monoline label + /command, gold bar, ghost focus"),
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Original 5 variants (kept as-is)
// ═══════════════════════════════════════════════════════════════════════

fn render_current() -> AnyElement {
    let theme = get_cached_theme();
    let bg = h(theme.colors.background.main);
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let border = h(theme.colors.ui.border);
    let accent = h(0x3b82f6);

    menu_shell(bg, border)
        .child(section_header("Context", muted))
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .px(px(12.))
                .py(px(4.))
                .rounded(px(6.))
                .bg(if selected {
                    accent.opacity(OPACITY_SELECTED)
                } else {
                    transparent_black()
                })
                .child(
                    div()
                        .size(px(ICON_SZ))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_xs()
                        .text_color(accent)
                        .child(item.icon_char),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        .child(
                            div()
                                .text_sm()
                                .text_color(fg)
                                .text_ellipsis()
                                .child(item.label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted)
                                .text_ellipsis()
                                .child(item.description),
                        ),
                )
        }))
        .into_any_element()
}

fn render_compact_tags() -> AnyElement {
    let theme = get_cached_theme();
    let bg = h(theme.colors.background.main);
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let border = h(theme.colors.ui.border);
    let accent = h(0x3b82f6);

    menu_shell(bg, border)
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(12.))
                .py(px(6.))
                .rounded(px(4.))
                .bg(if selected {
                    accent.opacity(OPACITY_HOVER)
                } else {
                    transparent_black()
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .size(px(ICON_SZ))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_xs()
                                .text_color(if selected { fg } else { muted })
                                .child(item.icon_char),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected { fg } else { muted.opacity(0.85) })
                                .child(item.label),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(muted.opacity(OPACITY_TEXT_MUTED))
                        .px(px(6.))
                        .py(px(2.))
                        .rounded(px(3.))
                        .bg(border.opacity(0.3))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

fn render_gold_accent() -> AnyElement {
    let theme = get_cached_theme();
    let bg = h(theme.colors.background.main);
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let border = h(theme.colors.ui.border);
    let gold = h(GOLD);

    menu_shell(bg, border)
        .child(section_header("Context", muted))
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .px(px(12.))
                .py(px(5.))
                .bg(if selected {
                    gold.opacity(0.06)
                } else {
                    transparent_black()
                })
                .child(div().w(px(2.)).h(px(18.)).rounded(px(1.)).bg(if selected {
                    gold
                } else {
                    transparent_black()
                }))
                .child(
                    div()
                        .flex()
                        .flex_1()
                        .flex_col()
                        .overflow_hidden()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected { fg } else { muted.opacity(0.85) })
                                .font_weight(if selected {
                                    FontWeight::MEDIUM
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_ellipsis()
                                .child(item.label),
                        )
                        .when(selected, |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(muted.opacity(OPACITY_TEXT_MUTED))
                                    .text_ellipsis()
                                    .child(item.description),
                            )
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(if selected { 0.5 } else { 0.35 }))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

fn render_grouped_sections() -> AnyElement {
    let theme = get_cached_theme();
    let bg = h(theme.colors.background.main);
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let border = h(theme.colors.ui.border);
    let accent = h(0x3b82f6);

    menu_shell(bg, border)
        .child(section_header("Context Snapshots", muted))
        .children(
            ITEMS[0..2]
                .iter()
                .enumerate()
                .map(|(i, item)| grouped_row(item, i == 0, fg, muted, accent)),
        )
        .child(div().h(px(8.)))
        .child(section_header("Target Sources", muted))
        .children(
            ITEMS[2..]
                .iter()
                .map(|item| grouped_row(item, false, fg, muted, accent)),
        )
        .into_any_element()
}

fn grouped_row(item: &SlashItem, selected: bool, fg: Hsla, muted: Hsla, accent: Hsla) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .px(px(14.))
        .py(px(5.))
        .mx(px(4.))
        .rounded(px(6.))
        .bg(if selected {
            accent.opacity(OPACITY_HOVER)
        } else {
            transparent_black()
        })
        .child(
            div()
                .size(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(4.))
                .bg(if selected {
                    accent.opacity(0.15)
                } else {
                    muted.opacity(0.08)
                })
                .text_xs()
                .text_color(if selected { accent } else { muted })
                .child(item.icon_char),
        )
        .child(
            div()
                .flex()
                .flex_1()
                .flex_col()
                .child(
                    div()
                        .text_sm()
                        .text_color(if selected { fg } else { muted.opacity(0.85) })
                        .child(item.label),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(0.5))
                        .child(item.description),
                ),
        )
}

fn render_raycast_polish() -> AnyElement {
    let theme = get_cached_theme();
    let bg = h(theme.colors.background.main);
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let border = h(theme.colors.ui.border);
    let gold = h(GOLD);

    div()
        .w(px(MENU_W))
        .max_h(px(MENU_MAX_H))
        .overflow_hidden()
        .rounded(px(10.))
        .border_1()
        .border_color(border.opacity(OPACITY_BORDER * 0.7))
        .bg(bg)
        .py(px(6.))
        .flex()
        .flex_col()
        .child(
            div()
                .px(px(14.))
                .py(px(4.))
                .text_xs()
                .text_color(muted.opacity(0.4))
                .child("Commands"),
        )
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .px(px(10.))
                .py(px(6.))
                .mx(px(6.))
                .rounded(px(8.))
                .bg(if selected {
                    gold.opacity(0.08)
                } else {
                    transparent_black()
                })
                .child(
                    div()
                        .w(px(2.5))
                        .h(px(20.))
                        .rounded(px(1.5))
                        .bg(if selected { gold } else { transparent_black() }),
                )
                .child(
                    div()
                        .size(px(22.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(6.))
                        .bg(if selected {
                            gold.opacity(0.12)
                        } else {
                            muted.opacity(0.06)
                        })
                        .text_xs()
                        .text_color(if selected { gold } else { muted.opacity(0.6) })
                        .child(item.icon_char),
                )
                .child(
                    div()
                        .flex()
                        .flex_1()
                        .flex_col()
                        .overflow_hidden()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(if selected {
                                    FontWeight::MEDIUM
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_color(if selected { fg } else { muted.opacity(0.8) })
                                .text_ellipsis()
                                .child(item.label),
                        )
                        .when(selected, |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(muted.opacity(0.5))
                                    .text_ellipsis()
                                    .child(item.description),
                            )
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(if selected { 0.45 } else { 0.3 }))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// 9 new minimal variants (.impeccable.md whisper chrome)
// ═══════════════════════════════════════════════════════════════════════

// ─── 6. Whisper Bare ──────────────────────────────────────────────────
// No icons, no container border. Just text + gold bar.

fn render_whisper_bare() -> AnyElement {
    let theme = get_cached_theme();
    let bg = h(theme.colors.background.main);
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    div()
        .w(px(MENU_W))
        .bg(bg)
        .py(px(4.))
        .flex()
        .flex_col()
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .px(px(10.))
                .py(px(5.))
                .bg(if selected {
                    gold.opacity(GHOST)
                } else {
                    transparent_black()
                })
                .child(div().w(px(2.)).h(px(16.)).rounded(px(1.)).bg(if selected {
                    gold
                } else {
                    transparent_black()
                }))
                .child(
                    div().flex().flex_1().overflow_hidden().child(
                        div()
                            .text_sm()
                            .text_color(if selected {
                                fg.opacity(PRESENT)
                            } else {
                                muted.opacity(HINT)
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
        }))
        .into_any_element()
}

// ─── 7. Ghost Float ──────────────────────────────────────────────────
// No container border. Ghost bg on focus only. Bare text.

fn render_ghost_float() -> AnyElement {
    let theme = get_cached_theme();
    let bg = h(theme.colors.background.main);
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    div()
        .w(px(MENU_W))
        .bg(bg.opacity(0.85))
        .py(px(2.))
        .flex()
        .flex_col()
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .px(px(12.))
                .py(px(6.))
                .bg(if selected {
                    fg.opacity(GHOST_HI)
                } else {
                    transparent_black()
                })
                .child(div().w(px(2.)).h(px(14.)).rounded(px(1.)).bg(if selected {
                    gold
                } else {
                    transparent_black()
                }))
                .child(
                    div().ml(px(8.)).flex().flex_1().overflow_hidden().child(
                        div()
                            .text_sm()
                            .text_color(if selected {
                                fg
                            } else {
                                muted.opacity(MUTED_OP)
                            })
                            .text_ellipsis()
                            .child(item.label),
                    ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(0.35))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

// ─── 8. Hairline Flush ───────────────────────────────────────────────
// Gold bar flush to left edge. Hairline container border. Tight vertical.

fn render_hairline_flush() -> AnyElement {
    let theme = get_cached_theme();
    let bg = h(theme.colors.background.main);
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let border = h(theme.colors.ui.border);
    let gold = h(GOLD);

    div()
        .w(px(MENU_W))
        .bg(bg)
        .py(px(2.))
        .flex()
        .flex_col()
        .border_1()
        .border_color(border.opacity(GHOST_HI))
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .py(px(4.))
                .pr(px(12.))
                // Gold bar flush to left wall
                .child(div().w(px(2.)).h(px(16.)).ml(px(0.)).bg(if selected {
                    gold
                } else {
                    transparent_black()
                }))
                .child(
                    div()
                        .ml(px(10.))
                        .flex()
                        .flex_1()
                        .flex_col()
                        .overflow_hidden()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected {
                                    fg
                                } else {
                                    muted.opacity(MUTED_OP)
                                })
                                .text_ellipsis()
                                .child(item.label),
                        )
                        .when(selected, |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(muted.opacity(HINT))
                                    .text_ellipsis()
                                    .child(item.description),
                            )
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(0.3))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

// ─── 9. Gold Bar Only ────────────────────────────────────────────────
// Gold bar is sole visual affordance. No bg. No icons. No border.

fn render_gold_only() -> AnyElement {
    let theme = get_cached_theme();
    let bg = h(theme.colors.background.main);
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    div()
        .w(px(MENU_W))
        .bg(bg)
        .py(px(4.))
        .flex()
        .flex_col()
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .px(px(8.))
                .py(px(5.))
                .child(div().w(px(2.)).h(px(16.)).rounded(px(1.)).bg(if selected {
                    gold
                } else {
                    transparent_black()
                }))
                .child(
                    div()
                        .ml(px(10.))
                        .text_sm()
                        .text_color(if selected { fg } else { muted.opacity(HINT) })
                        .child(item.label),
                )
        }))
        .into_any_element()
}

// ─── 10. Monoline Hint ──────────────────────────────────────────────
// One line: label left, /command right in hint opacity. Gold bar.

fn render_monoline_hint() -> AnyElement {
    let theme = get_cached_theme();
    let bg = h(theme.colors.background.main);
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    div()
        .w(px(MENU_W))
        .bg(bg)
        .py(px(4.))
        .flex()
        .flex_col()
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(10.))
                .py(px(5.))
                .bg(if selected {
                    gold.opacity(GHOST)
                } else {
                    transparent_black()
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(div().w(px(2.)).h(px(14.)).rounded(px(1.)).bg(if selected {
                            gold
                        } else {
                            transparent_black()
                        }))
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected {
                                    fg
                                } else {
                                    muted.opacity(MUTED_OP)
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
        }))
        .into_any_element()
}

// ─── 11. Dot Separator ──────────────────────────────────────────────
// Label · /command on one line. Gold bar. No icons.

fn render_dot_separator() -> AnyElement {
    let theme = get_cached_theme();
    let bg = h(theme.colors.background.main);
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    div()
        .w(px(MENU_W))
        .bg(bg)
        .py(px(4.))
        .flex()
        .flex_col()
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            let label_text: SharedString = format!("{}  ·  {}", item.label, item.command).into();
            div()
                .flex()
                .items_center()
                .px(px(10.))
                .py(px(5.))
                .child(div().w(px(2.)).h(px(14.)).rounded(px(1.)).bg(if selected {
                    gold
                } else {
                    transparent_black()
                }))
                .child(
                    div()
                        .ml(px(8.))
                        .text_sm()
                        .text_color(if selected {
                            fg.opacity(PRESENT)
                        } else {
                            muted.opacity(HINT)
                        })
                        .child(label_text),
                )
        }))
        .into_any_element()
}

// ─── 12. Vibrancy Panel ─────────────────────────────────────────────
// Near-transparent bg. Rows float on vibrancy. Gold bar + text.

fn render_vibrancy_panel() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    div()
        .w(px(MENU_W))
        .bg(fg.opacity(0.02))
        .py(px(4.))
        .flex()
        .flex_col()
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .px(px(10.))
                .py(px(6.))
                .bg(if selected {
                    fg.opacity(GHOST)
                } else {
                    transparent_black()
                })
                .child(div().w(px(2.)).h(px(16.)).rounded(px(1.)).bg(if selected {
                    gold
                } else {
                    transparent_black()
                }))
                .child(
                    div()
                        .ml(px(8.))
                        .flex()
                        .flex_1()
                        .flex_col()
                        .overflow_hidden()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected { fg } else { fg.opacity(MUTED_OP) })
                                .text_ellipsis()
                                .child(item.label),
                        )
                        .when(selected, |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(muted.opacity(HINT))
                                    .text_ellipsis()
                                    .child(item.description),
                            )
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(0.3))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

// ─── 13. Ultra Dense ─────────────────────────────────────────────────
// Tiny text, minimal padding, maximum items visible.

fn render_ultra_dense() -> AnyElement {
    let theme = get_cached_theme();
    let bg = h(theme.colors.background.main);
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    div()
        .w(px(MENU_W))
        .bg(bg)
        .py(px(2.))
        .flex()
        .flex_col()
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(8.))
                .py(px(2.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(div().w(px(1.5)).h(px(10.)).bg(if selected {
                            gold
                        } else {
                            transparent_black()
                        }))
                        .child(
                            div()
                                .text_xs()
                                .text_color(if selected {
                                    fg
                                } else {
                                    muted.opacity(MUTED_OP)
                                })
                                .child(item.label),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(0.3))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

// ─── 14. List Anatomy (.impeccable.md spec) ─────────────────────────
// Gold bar, name at present opacity, description on focus at muted,
// right-aligned hint metadata. Ghost bg. No icons. No border.

fn render_list_anatomy() -> AnyElement {
    let theme = get_cached_theme();
    let bg = h(theme.colors.background.main);
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    div()
        .w(px(MENU_W))
        .bg(bg)
        .py(px(4.))
        .flex()
        .flex_col()
        .children(ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .px(px(8.))
                .py(px(5.))
                .bg(if selected {
                    fg.opacity(GHOST)
                } else {
                    transparent_black()
                })
                // Gold left bar
                .child(div().w(px(2.)).h(px(18.)).rounded(px(1.)).bg(if selected {
                    gold
                } else {
                    transparent_black()
                }))
                // Text column
                .child(
                    div()
                        .ml(px(10.))
                        .flex()
                        .flex_1()
                        .flex_col()
                        .overflow_hidden()
                        // Name: present opacity when focused, hint when not
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected {
                                    fg.opacity(PRESENT)
                                } else {
                                    fg.opacity(HINT)
                                })
                                .font_weight(if selected {
                                    FontWeight::MEDIUM
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_ellipsis()
                                .child(item.label),
                        )
                        // Description: muted, only on focus
                        .when(selected, |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(muted.opacity(MUTED_OP))
                                    .text_ellipsis()
                                    .child(item.description),
                            )
                        }),
                )
                // Right metadata: hint opacity
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(HINT))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

// ─── 15. Vibrancy Monoline (10 + 12 combined) ───────────────────────
// Near-transparent bg lets vibrancy bleed through. Monoline layout:
// gold bar | label | /command right-aligned in hint. No icons. No border.

fn render_vibrancy_monoline() -> AnyElement {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let muted = h(theme.colors.text.dimmed);
    let gold = h(GOLD);

    // Near-transparent — vibrancy shows through
    div()
        .w(px(MENU_W))
        .bg(fg.opacity(0.02))
        .py(px(3.))
        .flex()
        .flex_col()
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
                // Gold bar
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(div().w(px(2.)).h(px(14.)).rounded(px(1.)).bg(if selected {
                            gold
                        } else {
                            transparent_black()
                        }))
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected { fg } else { fg.opacity(MUTED_OP) })
                                .child(item.label),
                        ),
                )
                // /command hint right-aligned
                .child(
                    div()
                        .text_xs()
                        .text_color(muted.opacity(if selected { HINT } else { 0.3 }))
                        .child(item.command),
                )
        }))
        .into_any_element()
}

// ─── Helpers ───────────────────────────────────────────────────────────

fn menu_shell(bg: Hsla, border: Hsla) -> Div {
    div()
        .w(px(MENU_W))
        .max_h(px(MENU_MAX_H))
        .overflow_hidden()
        .rounded(px(8.))
        .border_1()
        .border_color(border.opacity(OPACITY_BORDER))
        .bg(bg)
        .py(px(4.))
        .flex()
        .flex_col()
}

fn section_header(title: &str, muted: Hsla) -> Div {
    div()
        .px(px(12.))
        .pt(px(6.))
        .pb(px(2.))
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(muted.opacity(OPACITY_TEXT_MUTED))
        .child(title.to_string())
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::SlashCommandMenuVariationsStory;
    use crate::storybook::Story;

    #[test]
    fn slash_command_story_has_15_variants() {
        let story = SlashCommandMenuVariationsStory;
        assert_eq!(story.variants().len(), 15);
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
