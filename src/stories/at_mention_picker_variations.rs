//! @-Mention Context Picker — 21 Design Variations
//!
//! Concept: Cursor-style flush `@mention` inline text (no chip container) with
//! an aligned dropdown that appears directly beneath the `@` trigger position.
//! All 21 variations explore this single direction with different dropdown
//! treatments, density levels, selection indicators, and information hierarchy.
//!
//! Design reference: Cursor @-context, VS Code inline completions, Notion mentions.
//! Follows .impeccable.md whisper chrome: ghost bg, gold accent, spacing-only structure.

use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::list_item::FONT_MONO;
use crate::storybook::{story_container, story_section, Story, StorySurface, StoryVariant};
use crate::theme::get_cached_theme;

// ─── Constants ────────────────────────────────────────────────────────

const COMPOSER_W: f32 = 560.0;
const GOLD: u32 = 0xfbbf24;
const AT_OFFSET: f32 = 52.0; // px offset simulating cursor position after "Fix "

// Impeccable opacity tiers
const GHOST: f32 = 0.04;
const HINT: f32 = 0.45;
const MUTED_OP: f32 = 0.65;

fn h(hex: u32) -> Hsla {
    Hsla::from(rgb(hex))
}

fn ha(hex: u32, alpha_byte: u32) -> Hsla {
    let mut c = Hsla::from(rgb(hex));
    c.a = alpha_byte as f32 / 255.0;
    c
}

fn gold() -> Hsla {
    h(GOLD)
}

fn clear() -> Hsla {
    hsla(0., 0., 0., 0.)
}

// ─── Mock data ────────────────────────────────────────────────────────

struct MentionItem {
    mention: &'static str,
    label: &'static str,
    desc: &'static str,
    icon: &'static str,
    category: &'static str,
}

const ITEMS: &[MentionItem] = &[
    MentionItem {
        mention: "@screenshot",
        label: "Screenshot",
        desc: "Capture desktop",
        icon: "📸",
        category: "Capture",
    },
    MentionItem {
        mention: "@screen-region",
        label: "Screen Region",
        desc: "Select area to capture",
        icon: "⬚",
        category: "Capture",
    },
    MentionItem {
        mention: "@selection",
        label: "Selection",
        desc: "Selected text",
        icon: "▋",
        category: "Context",
    },
    MentionItem {
        mention: "@recent-scripts",
        label: "Recent Scripts",
        desc: "Last 5 scripts run",
        icon: "⏱",
        category: "Scripts",
    },
    MentionItem {
        mention: "@clipboard",
        label: "Clipboard",
        desc: "Clipboard contents",
        icon: "📋",
        category: "Context",
    },
    MentionItem {
        mention: "@git-status",
        label: "Git Status",
        desc: "Repo working tree",
        icon: "⎇",
        category: "System",
    },
    MentionItem {
        mention: "@git-diff",
        label: "Git Diff",
        desc: "Uncommitted changes",
        icon: "±",
        category: "System",
    },
    MentionItem {
        mention: "@browser",
        label: "Browser URL",
        desc: "Active browser tab",
        icon: "◆",
        category: "Context",
    },
];

// ─── Story ────────────────────────────────────────────────────────────

pub struct AtMentionPickerVariationsStory;

impl Story for AtMentionPickerVariationsStory {
    fn id(&self) -> &'static str {
        "at-mention-picker-variations"
    }

    fn name(&self) -> &'static str {
        "@-Mention Picker (21)"
    }

    fn category(&self) -> &'static str {
        "ACP Chat"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::MiniAiChat
    }

    fn render(&self) -> AnyElement {
        let variants = self.variants();
        let mut container = story_container();

        container = container.child(
            story_section("Dropdown Shape & Chrome (1–7)").children(
                variants[0..7]
                    .iter()
                    .enumerate()
                    .map(|(i, v)| variation_row(i + 1, v, self.render_variant(v))),
            ),
        );

        container = container.child(
            story_section("Selection & Focus Indicators (8–14)").children(
                variants[7..14]
                    .iter()
                    .enumerate()
                    .map(|(i, v)| variation_row(i + 8, v, self.render_variant(v))),
            ),
        );

        container = container.child(
            story_section("Information Density & Multi-Chip (15–21)").children(
                variants[14..21]
                    .iter()
                    .enumerate()
                    .map(|(i, v)| variation_row(i + 15, v, self.render_variant(v))),
            ),
        );

        container.into_any_element()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        match variant.stable_id().as_str() {
            // Dropdown shape & chrome
            "bare-popover" => v01_bare_popover(),
            "shadow-float" => v02_shadow_float(),
            "hairline-border" => v03_hairline_border(),
            "vibrancy-flush" => v04_vibrancy_flush(),
            "rounded-card" => v05_rounded_card(),
            "inset-panel" => v06_inset_panel(),
            "wide-dropdown" => v07_wide_dropdown(),
            // Selection & focus
            "gold-bar-ghost" => v08_gold_bar_ghost(),
            "gold-bar-no-bg" => v09_gold_bar_no_bg(),
            "underline-select" => v10_underline_select(),
            "bg-fill-only" => v11_bg_fill_only(),
            "border-left-thick" => v12_border_left_thick(),
            "dot-indicator" => v13_dot_indicator(),
            "highlight-label" => v14_highlight_label(),
            // Density & multi-chip
            "dense-monoline" => v15_dense_monoline(),
            "two-line-desc" => v16_two_line_desc(),
            "icon-grid-dense" => v17_icon_grid_dense(),
            "grouped-headers" => v18_grouped_headers(),
            "fuzzy-gold-chars" => v19_fuzzy_gold_chars(),
            "multi-chip-inline" => v20_multi_chip_inline(),
            "full-composition" => v21_full_composition(),
            _ => v01_bare_popover(),
        }
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            // Shape & chrome
            StoryVariant::default_named("bare-popover", "Bare Popover").description(
                "No border, no shadow. Just rows floating beneath @. Maximum whisper.",
            ),
            StoryVariant::default_named("shadow-float", "Shadow Float")
                .description("Subtle shadow, no border. Dropdown floats with depth but no chrome."),
            StoryVariant::default_named("hairline-border", "Hairline Border").description(
                "Single-pixel border at ghost opacity. Barely visible container edge.",
            ),
            StoryVariant::default_named("vibrancy-flush", "Vibrancy Flush").description(
                "Transparent bg lets vibrancy bleed through. Rows are the only structure.",
            ),
            StoryVariant::default_named("rounded-card", "Rounded Card")
                .description("Rounded corners (8px) with ghost fill. Softer, more contained feel."),
            StoryVariant::default_named("inset-panel", "Inset Panel")
                .description("Recessed look — darker bg than composer. Dropdown feels embedded."),
            StoryVariant::default_named("wide-dropdown", "Wide Dropdown")
                .description("Full composer width instead of narrow popover. More room for info."),
            // Selection & focus
            StoryVariant::default_named("gold-bar-ghost", "Gold Bar + Ghost BG").description(
                "2px gold bar + ghost bg on selected. The Script Kit signature selection.",
            ),
            StoryVariant::default_named("gold-bar-no-bg", "Gold Bar Only")
                .description("Gold bar is the SOLE indicator. No bg change. Pure minimal."),
            StoryVariant::default_named("underline-select", "Gold Underline")
                .description("Gold underline beneath selected row label. Horizontal emphasis."),
            StoryVariant::default_named("bg-fill-only", "Background Fill Only")
                .description("No bar, no underline — just a ghost bg fill on selection. Quietest."),
            StoryVariant::default_named("border-left-thick", "Thick Left Border")
                .description("3px gold left border (not bar). Bolder, more opinionated selection."),
            StoryVariant::default_named("dot-indicator", "Gold Dot")
                .description("Small gold dot before the label. Compact selection indicator."),
            StoryVariant::default_named("highlight-label", "Gold Label Text")
                .description("Selected label text turns gold. No bars, no bg — just color."),
            // Density & multi-chip
            StoryVariant::default_named("dense-monoline", "Dense Monoline").description(
                "Tight 20px rows. Label left, @mention right in mono. Maximum density.",
            ),
            StoryVariant::default_named("two-line-desc", "Two-Line with Description")
                .description("Label + description on selected row. More context for discovery."),
            StoryVariant::default_named("icon-grid-dense", "Icon Prefix Dense")
                .description("Leading emoji icon + compact label. Visual category hints."),
            StoryVariant::default_named("grouped-headers", "Grouped with Headers")
                .description("Category headers (CAPTURE, CONTEXT, SYSTEM). Organized discovery."),
            StoryVariant::default_named("fuzzy-gold-chars", "Fuzzy Match Gold")
                .description("Matched chars highlighted in gold. Live feedback on filter."),
            StoryVariant::default_named("multi-chip-inline", "Multi-Chip Inline")
                .description("Two accepted @mentions inline + active dropdown. Full flow state."),
            StoryVariant::default_named("full-composition", "Full Gold Composition").description(
                "Best-of: gold bar, fuzzy highlight, grouped sections, flush @text chips.",
            ),
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Shared helpers
// ═══════════════════════════════════════════════════════════════════════

fn variation_row(num: usize, variant: &StoryVariant, element: AnyElement) -> Div {
    let t = get_cached_theme();
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .mb(px(16.0))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.colors.text.primary))
                        .child(format!("{}. {}", num, variant.name)),
                )
                .when(variant.description.is_some(), |d| {
                    d.child(
                        div()
                            .text_xs()
                            .text_color(rgb(t.colors.text.dimmed))
                            .child(variant.description.clone().unwrap_or_default()),
                    )
                }),
        )
        .child(element)
}

/// Builds the composer shell with input line + aligned dropdown beneath the @ position.
fn composer_with_dropdown(
    prefix: &str,
    accepted_chips: &[&str],
    query: &str,
    dropdown: impl IntoElement,
    dropdown_width: f32,
) -> AnyElement {
    let t = get_cached_theme();
    let bg = t.colors.background.main;
    let border = t.colors.ui.border;
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;
    let muted = t.colors.text.muted;

    // Build input line: prefix text + accepted chips + @query + cursor
    let mut input_row = div()
        .flex()
        .flex_row()
        .items_center()
        .flex_wrap()
        .gap(px(2.0));

    if !prefix.is_empty() {
        input_row = input_row.child(
            div()
                .text_sm()
                .text_color(rgb(fg))
                .child(prefix.to_string()),
        );
    }

    for chip in accepted_chips {
        input_row = input_row.child(
            div()
                .text_sm()
                .text_color(gold().opacity(MUTED_OP))
                .child(chip.to_string()),
        );
    }

    if !query.is_empty() {
        input_row = input_row.child(
            div()
                .text_sm()
                .text_color(gold().opacity(MUTED_OP))
                .child(query.to_string()),
        );
    }

    // Blinking cursor
    input_row = input_row.child(div().w(px(2.0)).h(px(16.0)).bg(gold()).rounded(px(1.0)));

    // Calculate dropdown offset: approximate char width × prefix length
    let char_w: f32 = 7.6; // approximate text_sm char width
    let chip_chars: usize = accepted_chips.iter().map(|c| c.len() + 1).sum();
    let offset = (prefix.len() + chip_chars) as f32 * char_w;

    div()
        .id("mock-composer")
        .w(px(COMPOSER_W))
        .flex()
        .flex_col()
        .bg(rgb(bg))
        .rounded(px(8.0))
        .border_1()
        .border_color(ha(border, 0x20))
        .overflow_hidden()
        // Input row
        .child(div().px(px(12.0)).py(px(10.0)).child(input_row))
        // Dropdown aligned to @ position
        .child(
            div()
                .pl(px(12.0 + offset))
                .pb(px(4.0))
                .child(div().w(px(dropdown_width)).child(dropdown)),
        )
        // Footer
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .py(px(6.0))
                .border_t_1()
                .border_color(ha(border, 0x10))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(dimmed))
                        .child("Sonnet 4.6 \u{25be}"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(muted))
                        .child("\u{21b5} Send · \u{2318}K Actions · \u{2318}W Close"),
                ),
        )
        .into_any_element()
}

/// Standard 4-item filtered list for most variations.
fn make_rows(
    items: &[MentionItem],
    selected: usize,
    row_builder: impl Fn(&MentionItem, bool, usize) -> AnyElement,
) -> Vec<AnyElement> {
    items
        .iter()
        .take(4)
        .enumerate()
        .map(|(i, item)| row_builder(item, i == selected, i))
        .collect()
}

/// Fuzzy highlight: matched chars in gold, rest in base color.
fn fuzzy_label(label: &str, query: &str, base: Hsla, selected: bool) -> AnyElement {
    let lower_label = label.to_lowercase();
    let lower_query = query.to_lowercase();
    let mut match_indices = Vec::new();
    let mut qi = 0;
    let qchars: Vec<char> = lower_query.chars().collect();
    for (li, lc) in lower_label.chars().enumerate() {
        if qi < qchars.len() && lc == qchars[qi] {
            match_indices.push(li);
            qi += 1;
        }
    }
    let mut row = div().flex().flex_row().text_sm();
    for (i, ch) in label.chars().enumerate() {
        let is_match = match_indices.contains(&i);
        row = row.child(
            div()
                .text_color(if is_match { gold() } else { base })
                .when(is_match, |d| d.font_weight(FontWeight::BOLD))
                .child(ch.to_string()),
        );
    }
    row.into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// 1–7: Dropdown Shape & Chrome
// ═══════════════════════════════════════════════════════════════════════

fn v01_bare_popover() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div()
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(8.0))
                .py(px(3.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(if sel { fg } else { dim })
                        .child(item.label),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(dim.opacity(HINT))
                        .child(item.mention),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 240.0)
}

fn v02_shadow_float() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let bg = h(t.colors.background.main);

    let dd = div()
        .bg(bg)
        .rounded(px(6.0))
        .shadow_md()
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(8.0))
                .py(px(3.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(if sel { fg } else { dim })
                        .child(item.label),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(dim.opacity(HINT))
                        .child(item.mention),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 240.0)
}

fn v03_hairline_border() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let bdr = t.colors.ui.border;

    let dd = div()
        .border_1()
        .border_color(ha(bdr, 0x18))
        .rounded(px(4.0))
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(8.0))
                .py(px(3.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(if sel { fg } else { dim })
                        .child(item.label),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(dim.opacity(HINT))
                        .child(item.mention),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 240.0)
}

fn v04_vibrancy_flush() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div()
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(8.0))
                .py(px(3.0))
                .bg(if sel { fg.opacity(GHOST) } else { clear() })
                .child(
                    div()
                        .text_sm()
                        .text_color(if sel { fg } else { dim })
                        .child(item.label),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(dim.opacity(HINT))
                        .child(item.mention),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 240.0)
}

fn v05_rounded_card() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let bdr = t.colors.ui.border;

    let dd = div()
        .bg(ha(bdr, 0x08))
        .rounded(px(8.0))
        .py(px(4.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(10.0))
                .py(px(3.0))
                .rounded(px(4.0))
                .bg(if sel { fg.opacity(GHOST) } else { clear() })
                .child(
                    div()
                        .text_sm()
                        .text_color(if sel { fg } else { dim })
                        .child(item.label),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(dim.opacity(HINT))
                        .child(item.mention),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 250.0)
}

fn v06_inset_panel() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div()
        .bg(hsla(0., 0., 0., 0.15))
        .rounded(px(4.0))
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(8.0))
                .py(px(3.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(if sel { fg } else { dim })
                        .child(item.label),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(dim.opacity(HINT))
                        .child(item.mention),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 240.0)
}

fn v07_wide_dropdown() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let bdr = t.colors.ui.border;

    // Full-width: built differently — dropdown spans composer width
    let mut input_row = div().flex().flex_row().items_center().gap(px(2.0));
    input_row = input_row
        .child(
            div()
                .text_sm()
                .text_color(rgb(t.colors.text.primary))
                .child("Fix "),
        )
        .child(
            div()
                .text_sm()
                .text_color(gold().opacity(MUTED_OP))
                .child("@scr"),
        )
        .child(div().w(px(2.0)).h(px(16.0)).bg(gold()).rounded(px(1.0)));

    let dd = div()
        .w_full()
        .border_t_1()
        .border_color(ha(bdr, 0x10))
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .py(px(3.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(if sel { fg } else { dim })
                        .child(item.label),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(dim.opacity(HINT))
                        .child(item.mention),
                )
                .into_any_element()
        }));

    let t2 = get_cached_theme();
    div()
        .id("mock-composer")
        .w(px(COMPOSER_W))
        .flex()
        .flex_col()
        .bg(rgb(t2.colors.background.main))
        .rounded(px(8.0))
        .border_1()
        .border_color(ha(t2.colors.ui.border, 0x20))
        .overflow_hidden()
        .child(div().px(px(12.0)).py(px(10.0)).child(input_row))
        .child(dd)
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .py(px(6.0))
                .border_t_1()
                .border_color(ha(t2.colors.ui.border, 0x10))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(t2.colors.text.dimmed))
                        .child("Sonnet 4.6 \u{25be}"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(t2.colors.text.muted))
                        .child("\u{21b5} Send · \u{2318}K Actions · \u{2318}W Close"),
                ),
        )
        .into_any_element()
}

// ═══════════════════════════════════════════════════════════════════════
// 8–14: Selection & Focus Indicators
// ═══════════════════════════════════════════════════════════════════════

fn v08_gold_bar_ghost() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div()
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(4.0))
                .py(px(3.0))
                .bg(if sel { fg.opacity(GHOST) } else { clear() })
                .child(div().w(px(2.0)).h(px(14.0)).rounded(px(1.0)).bg(if sel {
                    gold()
                } else {
                    clear()
                }))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if sel { fg } else { dim })
                                .child(item.label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_family(FONT_MONO)
                                .text_color(dim.opacity(HINT))
                                .child(item.mention),
                        ),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 260.0)
}

fn v09_gold_bar_no_bg() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div()
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(4.0))
                .py(px(3.0))
                .child(div().w(px(2.0)).h(px(14.0)).rounded(px(1.0)).bg(if sel {
                    gold()
                } else {
                    clear()
                }))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if sel { fg } else { dim })
                                .child(item.label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_family(FONT_MONO)
                                .text_color(dim.opacity(HINT))
                                .child(item.mention),
                        ),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 260.0)
}

fn v10_underline_select() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div()
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(8.0))
                .py(px(3.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if sel { fg } else { dim })
                                .child(item.label),
                        )
                        .when(sel, |d| {
                            d.child(
                                div()
                                    .w_full()
                                    .h(px(1.5))
                                    .bg(gold().opacity(HINT))
                                    .rounded(px(1.0))
                                    .mt(px(1.0)),
                            )
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(dim.opacity(HINT))
                        .child(item.mention),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 260.0)
}

fn v11_bg_fill_only() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div()
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(8.0))
                .py(px(3.0))
                .rounded(px(4.0))
                .bg(if sel { fg.opacity(GHOST) } else { clear() })
                .child(
                    div()
                        .text_sm()
                        .text_color(if sel { fg } else { dim })
                        .child(item.label),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(dim.opacity(HINT))
                        .child(item.mention),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 250.0)
}

fn v12_border_left_thick() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div()
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(4.0))
                .py(px(3.0))
                .child(div().w(px(3.0)).h(px(16.0)).rounded(px(1.5)).bg(if sel {
                    gold()
                } else {
                    clear()
                }))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if sel { fg } else { dim })
                                .child(item.label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_family(FONT_MONO)
                                .text_color(dim.opacity(HINT))
                                .child(item.mention),
                        ),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 260.0)
}

fn v13_dot_indicator() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div()
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(8.0))
                .py(px(3.0))
                .child(div().w(px(5.0)).h(px(5.0)).rounded(px(999.0)).bg(if sel {
                    gold()
                } else {
                    clear()
                }))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if sel { fg } else { dim })
                                .child(item.label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_family(FONT_MONO)
                                .text_color(dim.opacity(HINT))
                                .child(item.mention),
                        ),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 260.0)
}

fn v14_highlight_label() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div()
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(8.0))
                .py(px(3.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(if sel { gold() } else { dim })
                        .child(item.label),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(dim.opacity(HINT))
                        .child(item.mention),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 240.0)
}

// ═══════════════════════════════════════════════════════════════════════
// 15–21: Information Density & Multi-Chip
// ═══════════════════════════════════════════════════════════════════════

fn v15_dense_monoline() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div()
        .py(px(1.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(6.0))
                .py(px(1.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .child(div().w(px(2.0)).h(px(10.0)).rounded(px(1.0)).bg(if sel {
                            gold()
                        } else {
                            clear()
                        }))
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
                        .child(item.mention),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 220.0)
}

fn v16_two_line_desc() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div()
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(4.0))
                .py(px(3.0))
                .bg(if sel { fg.opacity(GHOST) } else { clear() })
                .child(div().w(px(2.0)).h(px(14.0)).rounded(px(1.0)).bg(if sel {
                    gold()
                } else {
                    clear()
                }))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if sel { fg } else { dim })
                                        .child(item.label),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_family(FONT_MONO)
                                        .text_color(dim.opacity(HINT))
                                        .child(item.mention),
                                ),
                        )
                        .when(sel, |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(dim.opacity(HINT))
                                    .child(item.desc),
                            )
                        }),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 280.0)
}

fn v17_icon_grid_dense() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);
    let bdr = t.colors.ui.border;

    let dd = div()
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(6.0))
                .py(px(3.0))
                .bg(if sel { fg.opacity(GHOST) } else { clear() })
                .child(
                    div()
                        .w(px(22.0))
                        .h(px(22.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(5.0))
                        .bg(ha(bdr, 0x14))
                        .text_xs()
                        .child(item.icon),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if sel { fg } else { dim })
                                .child(item.label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_family(FONT_MONO)
                                .text_color(dim.opacity(HINT))
                                .child(item.mention),
                        ),
                )
                .into_any_element()
        }));

    composer_with_dropdown("Fix ", &[], "@scr", dd, 280.0)
}

fn v18_grouped_headers() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let categories: &[(&str, &[usize])] = &[
        ("CAPTURE", &[0, 1]),
        ("CONTEXT", &[2, 4]),
        ("SYSTEM", &[5, 6]),
    ];

    let mut dd = div().py(px(2.0));

    for (cat, indices) in categories {
        dd = dd.child(
            div()
                .px(px(8.0))
                .pt(px(5.0))
                .pb(px(1.0))
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(dim.opacity(HINT))
                .child(cat.to_string()),
        );

        for &idx in *indices {
            if let Some(item) = ITEMS.get(idx) {
                let sel = idx == 0;
                dd = dd.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .px(px(4.0))
                        .py(px(2.0))
                        .child(div().w(px(2.0)).h(px(14.0)).rounded(px(1.0)).bg(if sel {
                            gold()
                        } else {
                            clear()
                        }))
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if sel { fg } else { dim })
                                        .child(item.label),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_family(FONT_MONO)
                                        .text_color(dim.opacity(HINT))
                                        .child(item.mention),
                                ),
                        ),
                );
            }
        }
    }

    composer_with_dropdown("Fix ", &[], "@scr", dd, 270.0)
}

fn v19_fuzzy_gold_chars() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div().py(px(2.0)).children(
        ITEMS
            .iter()
            .take(4)
            .enumerate()
            .map(|(i, item)| {
                let sel = i == 0;
                let label_el = fuzzy_label(item.label, "scr", if sel { fg } else { dim }, sel);
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(4.0))
                    .py(px(3.0))
                    .bg(if sel { fg.opacity(GHOST) } else { clear() })
                    .child(div().w(px(2.0)).h(px(14.0)).rounded(px(1.0)).bg(if sel {
                        gold()
                    } else {
                        clear()
                    }))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(label_el)
                            .child(
                                div()
                                    .text_xs()
                                    .font_family(FONT_MONO)
                                    .text_color(dim.opacity(HINT))
                                    .child(item.mention),
                            ),
                    )
                    .into_any_element()
            })
            .collect::<Vec<_>>(),
    );

    composer_with_dropdown("Fix ", &[], "@scr", dd, 270.0)
}

fn v20_multi_chip_inline() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let dd = div()
        .py(px(2.0))
        .children(make_rows(&ITEMS, 0, |item, sel, _| {
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(4.0))
                .py(px(3.0))
                .bg(if sel { fg.opacity(GHOST) } else { clear() })
                .child(div().w(px(2.0)).h(px(14.0)).rounded(px(1.0)).bg(if sel {
                    gold()
                } else {
                    clear()
                }))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if sel { fg } else { dim })
                                .child(item.label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_family(FONT_MONO)
                                .text_color(dim.opacity(HINT))
                                .child(item.mention),
                        ),
                )
                .into_any_element()
        }));

    composer_with_dropdown(
        "Compare ",
        &["@clipboard", "@git-diff"],
        " with @scr",
        dd,
        260.0,
    )
}

fn v21_full_composition() -> AnyElement {
    let t = get_cached_theme();
    let fg = h(t.colors.text.primary);
    let dim = h(t.colors.text.dimmed);

    let categories: &[(&str, &[usize])] = &[("CAPTURE", &[0, 1]), ("CONTEXT", &[2, 4])];

    let mut dd = div().rounded(px(6.0)).shadow_sm().py(px(2.0));

    for (cat, indices) in categories {
        dd = dd.child(
            div()
                .px(px(8.0))
                .pt(px(5.0))
                .pb(px(1.0))
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(dim.opacity(HINT))
                .child(cat.to_string()),
        );

        for &idx in *indices {
            if let Some(item) = ITEMS.get(idx) {
                let sel = idx == 0;
                let label_el = fuzzy_label(item.label, "scr", if sel { fg } else { dim }, sel);
                dd = dd.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .px(px(4.0))
                        .py(px(3.0))
                        .bg(if sel { fg.opacity(GHOST) } else { clear() })
                        .child(div().w(px(2.0)).h(px(14.0)).rounded(px(1.0)).bg(if sel {
                            gold()
                        } else {
                            clear()
                        }))
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(label_el)
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_family(FONT_MONO)
                                                .text_color(dim.opacity(HINT))
                                                .child(item.mention),
                                        ),
                                )
                                .when(sel, |d| {
                                    d.child(
                                        div()
                                            .text_xs()
                                            .text_color(dim.opacity(HINT))
                                            .child(item.desc),
                                    )
                                }),
                        ),
                );
            }
        }
    }

    composer_with_dropdown("Analyze ", &["@selection"], " @scr", dd, 280.0)
}

// ═══════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::AtMentionPickerVariationsStory;
    use crate::storybook::Story;

    #[test]
    fn at_mention_story_has_21_variants() {
        let story = AtMentionPickerVariationsStory;
        assert_eq!(story.variants().len(), 21);
    }

    #[test]
    fn all_variant_ids_are_unique() {
        let story = AtMentionPickerVariationsStory;
        let ids: Vec<_> = story.variants().iter().map(|v| v.stable_id()).collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len(), "duplicate variant IDs found");
    }
}
