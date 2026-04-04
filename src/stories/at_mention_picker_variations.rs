//! @-Mention Context Picker — 21 Design Variations
//!
//! Explores inline `@` autocomplete for the ACP chat input. Each variation shows:
//! 1. A mock composer input with typed `@scr` query text
//! 2. The picker overlay with mock context items
//! 3. Inline chips for already-accepted mentions
//!
//! Design reference: Claude Code @-file picker, Cursor @-context, Raycast AI mentions.
//! Follows .impeccable.md whisper chrome: ghost bg, gold accent, spacing-only structure.

use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::list_item::FONT_MONO;
use crate::storybook::{story_container, story_section, Story, StorySurface, StoryVariant};
use crate::theme::get_cached_theme;

// ─── Constants ────────────────────────────────────────────────────────

const COMPOSER_W: f32 = 560.0;
const PICKER_MAX_H: f32 = 260.0;
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

// ─── Mock data ────────────────────────────────────────────────────────

struct MentionItem {
    mention: &'static str,
    label: &'static str,
    description: &'static str,
    icon_char: &'static str,
    category: &'static str,
}

const FILTERED_ITEMS: &[MentionItem] = &[
    MentionItem {
        mention: "@screenshot",
        label: "Screenshot",
        description: "Capture desktop screenshot",
        icon_char: "📸",
        category: "Capture",
    },
    MentionItem {
        mention: "@screen-region",
        label: "Screen Region",
        description: "Select a region to capture",
        icon_char: "⬚",
        category: "Capture",
    },
    MentionItem {
        mention: "@selection",
        label: "Selection",
        description: "Currently selected text",
        icon_char: "▋",
        category: "Context",
    },
    MentionItem {
        mention: "@recent-scripts",
        label: "Recent Scripts",
        description: "Last 5 executed scripts",
        icon_char: "⏱",
        category: "Scripts",
    },
];

const ALL_ITEMS: &[MentionItem] = &[
    MentionItem {
        mention: "@screenshot",
        label: "Screenshot",
        description: "Capture desktop screenshot",
        icon_char: "📸",
        category: "Capture",
    },
    MentionItem {
        mention: "@clipboard",
        label: "Clipboard",
        description: "Current clipboard contents",
        icon_char: "📋",
        category: "Context",
    },
    MentionItem {
        mention: "@selection",
        label: "Selection",
        description: "Currently selected text",
        icon_char: "▋",
        category: "Context",
    },
    MentionItem {
        mention: "@dictation",
        label: "Last Dictation",
        description: "Most recent voice transcript",
        icon_char: "🎙",
        category: "Context",
    },
    MentionItem {
        mention: "@git-status",
        label: "Git Status",
        description: "Current repo status",
        icon_char: "⎇",
        category: "System",
    },
    MentionItem {
        mention: "@git-diff",
        label: "Git Diff",
        description: "Uncommitted changes",
        icon_char: "±",
        category: "System",
    },
    MentionItem {
        mention: "@browser",
        label: "Browser URL",
        description: "Current browser tab URL",
        icon_char: "◆",
        category: "Context",
    },
    MentionItem {
        mention: "@window",
        label: "Focused Window",
        description: "Frontmost window info",
        icon_char: "▢",
        category: "Context",
    },
];

// Chips already accepted in the input
struct AcceptedChip {
    label: &'static str,
    icon_char: &'static str,
}

const CHIPS_SINGLE: &[AcceptedChip] = &[AcceptedChip {
    label: "@clipboard",
    icon_char: "📋",
}];

const CHIPS_MULTI: &[AcceptedChip] = &[
    AcceptedChip {
        label: "@clipboard",
        icon_char: "📋",
    },
    AcceptedChip {
        label: "@git-diff",
        icon_char: "±",
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
        "AI Chat"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::MiniAiChat
    }

    fn render(&self) -> AnyElement {
        let variants = self.variants();

        let mut container = story_container();

        // Group: Picker Overlay Style (1-7)
        container = container.child(
            story_section("Picker Overlay — How the dropdown looks when @ is typed")
                .children(variants[0..7].iter().enumerate().map(|(i, v)| {
                    variation_row(i + 1, v, self.render_variant(v))
                })),
        );

        // Group: Inline Chip Style (8-14)
        container = container.child(
            story_section(
                "Inline Chips — How accepted @mentions render in the input",
            )
            .children(variants[7..14].iter().enumerate().map(|(i, v)| {
                variation_row(i + 8, v, self.render_variant(v))
            })),
        );

        // Group: Full Composition (15-21)
        container = container.child(
            story_section("Full Composition — Picker + chips + input together")
                .children(variants[14..21].iter().enumerate().map(|(i, v)| {
                    variation_row(i + 15, v, self.render_variant(v))
                })),
        );

        container.into_any_element()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let id = variant.stable_id();
        match id.as_str() {
            // Picker overlay styles (1-7)
            "picker-vibrancy-monoline" => render_picker_vibrancy_monoline(),
            "picker-gold-bar" => render_picker_gold_bar(),
            "picker-grouped-sections" => render_picker_grouped_sections(),
            "picker-compact-two-col" => render_picker_compact_two_col(),
            "picker-icon-grid" => render_picker_icon_grid(),
            "picker-fuzzy-highlight" => render_picker_fuzzy_highlight(),
            "picker-cursor-anchored" => render_picker_cursor_anchored(),
            // Inline chip styles (8-14)
            "chip-ghost-pill" => render_chip_ghost_pill(),
            "chip-gold-tint" => render_chip_gold_tint(),
            "chip-outlined-tag" => render_chip_outlined_tag(),
            "chip-flush-at" => render_chip_flush_at(),
            "chip-mono-capsule" => render_chip_mono_capsule(),
            "chip-icon-badge" => render_chip_icon_badge(),
            "chip-accent-underline" => render_chip_accent_underline(),
            // Full compositions (15-21)
            "full-whisper-minimal" => render_full_whisper_minimal(),
            "full-raycast-polish" => render_full_raycast_polish(),
            "full-cursor-style" => render_full_cursor_style(),
            "full-claude-code" => render_full_claude_code(),
            "full-multi-chip-flow" => render_full_multi_chip_flow(),
            "full-dense-power" => render_full_dense_power(),
            "full-gold-signature" => render_full_gold_signature(),
            _ => render_picker_vibrancy_monoline(),
        }
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            // ── Picker Overlay Styles ──
            StoryVariant::default_named("picker-vibrancy-monoline", "Vibrancy Monoline")
                .description("Transparent bg, monoline label + @mention right, gold bar, ghost focus. Maximum whisper."),
            StoryVariant::default_named("picker-gold-bar", "Gold Bar Only")
                .description("Gold bar is sole selection indicator. No bg change. Label + description on focus."),
            StoryVariant::default_named("picker-grouped-sections", "Grouped Sections")
                .description("Items grouped by category (Capture, Context, System). Uppercase section headers in hint opacity."),
            StoryVariant::default_named("picker-compact-two-col", "Compact Two-Column")
                .description("Label left, @mention right. Dense single-line rows. No icons, no descriptions."),
            StoryVariant::default_named("picker-icon-grid", "Icon Prefix")
                .description("Leading emoji/glyph icon before label. Adds visual type hint without noise."),
            StoryVariant::default_named("picker-fuzzy-highlight", "Fuzzy Match Highlight")
                .description("Matched characters in the label rendered in gold. Shows filtering is live."),
            StoryVariant::default_named("picker-cursor-anchored", "Cursor-Anchored Popover")
                .description("Picker appears as a tight popover anchored to the @ position, not full-width."),

            // ── Inline Chip Styles ──
            StoryVariant::default_named("chip-ghost-pill", "Ghost Pill")
                .description("Full rounded pill, ghost bg (0.06). Chip blends into the text flow."),
            StoryVariant::default_named("chip-gold-tint", "Gold Tint")
                .description("Gold-tinted background (rgba fbbf24, 0.12). The signature Script Kit chip."),
            StoryVariant::default_named("chip-outlined-tag", "Outlined Tag")
                .description("Hairline border, no fill. Square corners. Terminal-native feel."),
            StoryVariant::default_named("chip-flush-at", "Flush @ Prefix")
                .description("No container — just @mention text in accent color. Absolute minimum chrome."),
            StoryVariant::default_named("chip-mono-capsule", "Mono Capsule")
                .description("Monospace font in a bordered capsule. Code-native, like a variable reference."),
            StoryVariant::default_named("chip-icon-badge", "Icon Badge")
                .description("Leading emoji icon + label in a ghost pill. Visual category hint at a glance."),
            StoryVariant::default_named("chip-accent-underline", "Accent Underline")
                .description("No container — gold underline beneath the @mention text. Minimal but clear."),

            // ── Full Compositions ──
            StoryVariant::default_named("full-whisper-minimal", "Whisper Minimal")
                .description("Ghost pill chips + vibrancy monoline picker. Maximum whisper chrome composition."),
            StoryVariant::default_named("full-raycast-polish", "Raycast Polish")
                .description("Gold tint chips + grouped section picker + icon prefixes. Polished and discoverable."),
            StoryVariant::default_named("full-cursor-style", "Cursor-Style")
                .description("Flush @text chips + compact two-column picker. Dense, developer-focused."),
            StoryVariant::default_named("full-claude-code", "Claude Code Style")
                .description("Mono capsule chips + fuzzy highlight picker. Code-native, precision feel."),
            StoryVariant::default_named("full-multi-chip-flow", "Multi-Chip Flow")
                .description("Multiple chips inline with text. Shows 'Compare @clipboard with @git-diff' pattern."),
            StoryVariant::default_named("full-dense-power", "Dense Power User")
                .description("Minimal chips + ultra-dense picker. Maximum information density for power users."),
            StoryVariant::default_named("full-gold-signature", "Gold Signature")
                .description("Gold tint chips + gold bar picker + accent underline. Full Script Kit brand expression."),
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Shared helpers
// ═══════════════════════════════════════════════════════════════════════

fn variation_row(num: usize, variant: &StoryVariant, element: AnyElement) -> Div {
    let t = get_cached_theme();
    let dimmed = t.colors.text.dimmed;

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
                            .text_color(rgb(dimmed))
                            .child(variant.description.clone().unwrap_or_default()),
                    )
                }),
        )
        .child(element)
}

/// Full composer shell: input area + optional picker + footer.
fn composer_shell(
    input_content: impl IntoElement,
    picker: Option<AnyElement>,
    chips_below: Option<AnyElement>,
) -> AnyElement {
    let t = get_cached_theme();
    let bg = t.colors.background.main;
    let border = t.colors.ui.border;
    let dimmed = t.colors.text.dimmed;
    let muted = t.colors.text.muted;

    div()
        .id("mock-composer")
        .w(px(COMPOSER_W))
        .flex()
        .flex_col()
        .bg(rgb(bg))
        .rounded(px(8.0))
        .border_1()
        .border_color(rgba((border << 8) | 0x20))
        .overflow_hidden()
        // Input row
        .child(
            div()
                .px(px(12.0))
                .py(px(10.0))
                .child(input_content),
        )
        // Chips row (if present)
        .when_some(chips_below, |d, chips| d.child(chips))
        // Picker overlay (if present)
        .when_some(picker, |d, p| d.child(p))
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
                .border_color(rgba((border << 8) | 0x10))
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
                        .child("\u{21b5} Send \u{00b7} \u{2318}K Actions \u{00b7} \u{2318}W Close"),
                ),
        )
        .into_any_element()
}

/// Input line showing typed text with an inline chip.
fn input_with_chips_and_query(
    prefix_text: &str,
    chips: &[(&str, Option<u32>)], // (label, optional tint color)
    query_text: &str,
    chip_style: ChipStyle,
) -> Div {
    let t = get_cached_theme();
    let primary = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;

    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .flex_wrap()
        .gap(px(4.0));

    if !prefix_text.is_empty() {
        row = row.child(
            div()
                .text_sm()
                .text_color(rgb(primary))
                .child(prefix_text.to_string()),
        );
    }

    for (label, tint) in chips {
        row = row.child(render_chip(label, *tint, chip_style));
    }

    if !query_text.is_empty() {
        row = row.child(
            div()
                .text_sm()
                .text_color(rgb(primary))
                .child(query_text.to_string()),
        );
    }

    // Blinking cursor
    row = row.child(
        div()
            .w(px(2.0))
            .h(px(16.0))
            .bg(rgb(GOLD))
            .rounded(px(1.0)),
    );

    if prefix_text.is_empty() && chips.is_empty() && query_text.is_empty() {
        row = row.child(
            div()
                .text_sm()
                .text_color(rgb(dimmed))
                .child("Ask Claude Code..."),
        );
    }

    row
}

#[derive(Clone, Copy)]
enum ChipStyle {
    GhostPill,
    GoldTint,
    OutlinedTag,
    FlushAt,
    MonoCapsule,
    IconBadge,
    AccentUnderline,
}

fn render_chip(label: &str, icon_tint: Option<u32>, style: ChipStyle) -> AnyElement {
    let t = get_cached_theme();
    let border = t.colors.ui.border;
    let dimmed = t.colors.text.dimmed;
    let primary = t.colors.text.primary;
    let accent = t.colors.accent.selected;

    match style {
        ChipStyle::GhostPill => div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(3.0))
            .px(px(8.0))
            .py(px(1.0))
            .rounded(px(999.0))
            .bg(rgba((border << 8) | 0x10))
            .child(div().text_xs().text_color(rgb(dimmed)).child(label.to_string()))
            .into_any_element(),

        ChipStyle::GoldTint => div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(3.0))
            .px(px(8.0))
            .py(px(1.0))
            .rounded(px(4.0))
            .bg(rgba((GOLD << 8) | 0x1E))
            .child(
                div()
                    .text_xs()
                    .text_color(h(GOLD).opacity(PRESENT))
                    .child(label.to_string()),
            )
            .into_any_element(),

        ChipStyle::OutlinedTag => div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(3.0))
            .px(px(6.0))
            .py(px(1.0))
            .rounded(px(2.0))
            .border_1()
            .border_color(rgba((border << 8) | 0x28))
            .child(div().text_xs().text_color(rgb(dimmed)).child(label.to_string()))
            .into_any_element(),

        ChipStyle::FlushAt => div()
            .child(
                div()
                    .text_xs()
                    .text_color(h(GOLD).opacity(MUTED_OP))
                    .child(label.to_string()),
            )
            .into_any_element(),

        ChipStyle::MonoCapsule => div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(3.0))
            .px(px(6.0))
            .py(px(1.0))
            .rounded(px(4.0))
            .bg(rgba((border << 8) | 0x0C))
            .border_1()
            .border_color(rgba((border << 8) | 0x18))
            .child(
                div()
                    .font_family(FONT_MONO)
                    .text_xs()
                    .text_color(rgb(dimmed))
                    .child(label.to_string()),
            )
            .into_any_element(),

        ChipStyle::IconBadge => {
            let icon = icon_tint.map(|_| "📋").unwrap_or("◎");
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.0))
                .px(px(8.0))
                .py(px(1.0))
                .rounded(px(999.0))
                .bg(rgba((border << 8) | 0x10))
                .child(div().text_xs().child(icon.to_string()))
                .child(div().text_xs().text_color(rgb(dimmed)).child(label.to_string()))
                .into_any_element()
        }

        ChipStyle::AccentUnderline => div()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(primary))
                    .child(label.to_string()),
            )
            .child(
                div()
                    .w_full()
                    .h(px(1.5))
                    .bg(h(GOLD).opacity(HINT))
                    .rounded(px(1.0)),
            )
            .into_any_element(),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 1-7: Picker Overlay Variations
// ═══════════════════════════════════════════════════════════════════════

fn render_picker_vibrancy_monoline() -> AnyElement {
    let t = get_cached_theme();
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;
    let border = t.colors.ui.border;

    let input = input_with_chips_and_query("", &[], "@scr", ChipStyle::GhostPill);

    let picker = div()
        .w_full()
        .max_h(px(PICKER_MAX_H))
        .py(px(2.0))
        .children(FILTERED_ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .py(px(4.0))
                .bg(if selected {
                    rgba((fg << 8) | 0x0A)
                } else {
                    transparent_black()
                })
                // Gold bar
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .w(px(2.0))
                                .h(px(14.0))
                                .rounded(px(1.0))
                                .bg(if selected { rgb(GOLD) } else { transparent_black() }),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected {
                                    rgb(fg)
                                } else {
                                    rgba((fg << 8) | ((MUTED_OP * 255.0) as u32))
                                })
                                .child(item.label),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgba((dimmed << 8) | ((HINT * 255.0) as u32)))
                        .child(item.mention),
                )
        }))
        .into_any_element();

    composer_shell(input, Some(picker), None)
}

fn render_picker_gold_bar() -> AnyElement {
    let t = get_cached_theme();
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;

    let input = input_with_chips_and_query("", &[], "@scr", ChipStyle::GhostPill);

    let picker = div()
        .w_full()
        .max_h(px(PICKER_MAX_H))
        .py(px(2.0))
        .children(FILTERED_ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .flex_col()
                .px(px(12.0))
                .py(px(4.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .w(px(2.0))
                                .h(px(14.0))
                                .rounded(px(1.0))
                                .bg(if selected { rgb(GOLD) } else { transparent_black() }),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected { rgb(fg) } else { rgb(dimmed) })
                                .child(item.label),
                        ),
                )
                .when(selected, |d| {
                    d.child(
                        div()
                            .pl(px(8.0))
                            .text_xs()
                            .text_color(rgba((dimmed << 8) | ((HINT * 255.0) as u32)))
                            .child(item.description),
                    )
                })
        }))
        .into_any_element();

    composer_shell(input, Some(picker), None)
}

fn render_picker_grouped_sections() -> AnyElement {
    let t = get_cached_theme();
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;
    let border = t.colors.ui.border;

    let input = input_with_chips_and_query("", &[], "@scr", ChipStyle::GhostPill);

    let mut picker_col = div().w_full().max_h(px(PICKER_MAX_H)).py(px(2.0));

    // Group by category
    let categories: &[(&str, &[usize])] = &[
        ("CAPTURE", &[0, 1]),
        ("CONTEXT", &[2]),
        ("SCRIPTS", &[3]),
    ];

    for (cat_name, indices) in categories {
        picker_col = picker_col.child(
            div()
                .px(px(12.0))
                .pt(px(6.0))
                .pb(px(2.0))
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgba((dimmed << 8) | ((HINT * 255.0) as u32)))
                .child(cat_name.to_string()),
        );

        for &idx in *indices {
            if let Some(item) = FILTERED_ITEMS.get(idx) {
                let selected = idx == 0;
                picker_col = picker_col.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(12.0))
                        .py(px(3.0))
                        .bg(if selected {
                            rgba((fg << 8) | 0x0A)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .w(px(2.0))
                                        .h(px(14.0))
                                        .rounded(px(1.0))
                                        .bg(if selected {
                                            rgb(GOLD)
                                        } else {
                                            transparent_black()
                                        }),
                                )
                                .child(div().text_xs().child(item.icon_char))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if selected { rgb(fg) } else { rgb(dimmed) })
                                        .child(item.label),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgba((dimmed << 8) | ((HINT * 255.0) as u32)))
                                .child(item.mention),
                        ),
                );
            }
        }
    }

    composer_shell(input, Some(picker_col.into_any_element()), None)
}

fn render_picker_compact_two_col() -> AnyElement {
    let t = get_cached_theme();
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;

    let input = input_with_chips_and_query("", &[], "@scr", ChipStyle::GhostPill);

    let picker = div()
        .w_full()
        .max_h(px(PICKER_MAX_H))
        .py(px(2.0))
        .children(FILTERED_ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .py(px(3.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .w(px(2.0))
                                .h(px(12.0))
                                .rounded(px(1.0))
                                .bg(if selected { rgb(GOLD) } else { transparent_black() }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(if selected { rgb(fg) } else { rgb(dimmed) })
                                .child(item.label),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(rgba((dimmed << 8) | 0x60))
                        .child(item.mention),
                )
        }))
        .into_any_element();

    composer_shell(input, Some(picker), None)
}

fn render_picker_icon_grid() -> AnyElement {
    let t = get_cached_theme();
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;
    let border = t.colors.ui.border;

    let input = input_with_chips_and_query("", &[], "@scr", ChipStyle::GhostPill);

    let picker = div()
        .w_full()
        .max_h(px(PICKER_MAX_H))
        .py(px(2.0))
        .children(FILTERED_ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(12.0))
                .py(px(4.0))
                .bg(if selected {
                    rgba((fg << 8) | 0x08)
                } else {
                    transparent_black()
                })
                // Icon circle
                .child(
                    div()
                        .w(px(24.0))
                        .h(px(24.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(6.0))
                        .bg(rgba((border << 8) | 0x14))
                        .text_xs()
                        .child(item.icon_char),
                )
                // Label + description column
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected { rgb(fg) } else { rgb(dimmed) })
                                .child(item.label),
                        )
                        .when(selected, |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(rgba((dimmed << 8) | ((HINT * 255.0) as u32)))
                                    .child(item.description),
                            )
                        }),
                )
        }))
        .into_any_element();

    composer_shell(input, Some(picker), None)
}

fn render_picker_fuzzy_highlight() -> AnyElement {
    let t = get_cached_theme();
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;

    let input = input_with_chips_and_query("", &[], "@scr", ChipStyle::GhostPill);

    // Simulate fuzzy matching "scr" in labels
    let picker = div()
        .w_full()
        .max_h(px(PICKER_MAX_H))
        .py(px(2.0))
        .children(FILTERED_ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;

            // Build label with highlighted matching chars
            let label_chars = fuzzy_highlight_label(item.label, "scr", fg, dimmed, selected);

            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .py(px(4.0))
                .bg(if selected {
                    rgba((fg << 8) | 0x08)
                } else {
                    transparent_black()
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .w(px(2.0))
                                .h(px(14.0))
                                .rounded(px(1.0))
                                .bg(if selected { rgb(GOLD) } else { transparent_black() }),
                        )
                        .child(label_chars),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgba((dimmed << 8) | ((HINT * 255.0) as u32)))
                        .child(item.mention),
                )
        }))
        .into_any_element();

    composer_shell(input, Some(picker), None)
}

/// Render a label with fuzzy-matched characters highlighted in gold.
fn fuzzy_highlight_label(
    label: &str,
    query: &str,
    fg: u32,
    dimmed: u32,
    selected: bool,
) -> AnyElement {
    let base_color = if selected { rgb(fg) } else { rgb(dimmed) };

    let lower_label = label.to_lowercase();
    let lower_query = query.to_lowercase();

    // Simple sequential char match
    let mut match_indices = Vec::new();
    let mut qi = 0;
    let query_chars: Vec<char> = lower_query.chars().collect();
    for (li, lc) in lower_label.chars().enumerate() {
        if qi < query_chars.len() && lc == query_chars[qi] {
            match_indices.push(li);
            qi += 1;
        }
    }

    let mut row = div().flex().flex_row().text_sm();
    for (i, ch) in label.chars().enumerate() {
        let is_match = match_indices.contains(&i);
        row = row.child(
            div()
                .text_color(if is_match { rgb(GOLD) } else { base_color })
                .when(is_match, |d| d.font_weight(FontWeight::BOLD))
                .child(ch.to_string()),
        );
    }

    row.into_any_element()
}

fn render_picker_cursor_anchored() -> AnyElement {
    let t = get_cached_theme();
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;
    let border = t.colors.ui.border;
    let bg = t.colors.background.main;

    let input = input_with_chips_and_query("Compare ", &[], "@scr", ChipStyle::GhostPill);

    // Narrow popover anchored offset from left (simulating cursor position)
    let picker = div()
        .pl(px(72.0)) // Offset to simulate cursor position after "Compare "
        .child(
            div()
                .w(px(240.0))
                .max_h(px(180.0))
                .bg(rgb(bg))
                .rounded(px(6.0))
                .border_1()
                .border_color(rgba((border << 8) | 0x20))
                .py(px(2.0))
                .shadow_md()
                .children(FILTERED_ITEMS.iter().enumerate().map(|(i, item)| {
                    let selected = i == 0;
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .px(px(8.0))
                        .py(px(3.0))
                        .bg(if selected {
                            rgba((fg << 8) | 0x08)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .w(px(2.0))
                                .h(px(12.0))
                                .rounded(px(1.0))
                                .bg(if selected { rgb(GOLD) } else { transparent_black() }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(if selected { rgb(fg) } else { rgb(dimmed) })
                                .child(item.label),
                        )
                })),
        )
        .into_any_element();

    composer_shell(input, Some(picker), None)
}

// ═══════════════════════════════════════════════════════════════════════
// 8-14: Inline Chip Style Variations
// ═══════════════════════════════════════════════════════════════════════

fn render_chip_ghost_pill() -> AnyElement {
    let input = input_with_chips_and_query(
        "Explain ",
        &[("@clipboard", None)],
        " to me",
        ChipStyle::GhostPill,
    );
    composer_shell(input, None, None)
}

fn render_chip_gold_tint() -> AnyElement {
    let input = input_with_chips_and_query(
        "Explain ",
        &[("@clipboard", None)],
        " to me",
        ChipStyle::GoldTint,
    );
    composer_shell(input, None, None)
}

fn render_chip_outlined_tag() -> AnyElement {
    let input = input_with_chips_and_query(
        "Explain ",
        &[("@clipboard", None)],
        " to me",
        ChipStyle::OutlinedTag,
    );
    composer_shell(input, None, None)
}

fn render_chip_flush_at() -> AnyElement {
    let input = input_with_chips_and_query(
        "Explain ",
        &[("@clipboard", None)],
        " to me",
        ChipStyle::FlushAt,
    );
    composer_shell(input, None, None)
}

fn render_chip_mono_capsule() -> AnyElement {
    let input = input_with_chips_and_query(
        "Explain ",
        &[("@clipboard", None)],
        " to me",
        ChipStyle::MonoCapsule,
    );
    composer_shell(input, None, None)
}

fn render_chip_icon_badge() -> AnyElement {
    let input = input_with_chips_and_query(
        "Explain ",
        &[("@clipboard", Some(GOLD))],
        " to me",
        ChipStyle::IconBadge,
    );
    composer_shell(input, None, None)
}

fn render_chip_accent_underline() -> AnyElement {
    let input = input_with_chips_and_query(
        "Explain ",
        &[("@clipboard", None)],
        " to me",
        ChipStyle::AccentUnderline,
    );
    composer_shell(input, None, None)
}

// ═══════════════════════════════════════════════════════════════════════
// 15-21: Full Composition Variations
// ═══════════════════════════════════════════════════════════════════════

fn render_full_whisper_minimal() -> AnyElement {
    let t = get_cached_theme();
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;

    let input = input_with_chips_and_query(
        "What is ",
        &[("@clipboard", None)],
        " @scr",
        ChipStyle::GhostPill,
    );

    let picker = div()
        .w_full()
        .py(px(2.0))
        .children(FILTERED_ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .py(px(4.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .w(px(2.0))
                                .h(px(14.0))
                                .rounded(px(1.0))
                                .bg(if selected { rgb(GOLD) } else { transparent_black() }),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected { rgb(fg) } else { rgb(dimmed) })
                                .child(item.label),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgba((dimmed << 8) | ((HINT * 255.0) as u32)))
                        .child(item.mention),
                )
        }))
        .into_any_element();

    composer_shell(input, Some(picker), None)
}

fn render_full_raycast_polish() -> AnyElement {
    let t = get_cached_theme();
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;
    let border = t.colors.ui.border;

    let input = input_with_chips_and_query(
        "Analyze ",
        &[("@selection", None)],
        " @scr",
        ChipStyle::GoldTint,
    );

    let mut picker_col = div().w_full().py(px(2.0));

    let categories: &[(&str, &[usize])] = &[("CAPTURE", &[0, 1]), ("CONTEXT", &[2]), ("SCRIPTS", &[3])];

    for (cat_name, indices) in categories {
        picker_col = picker_col.child(
            div()
                .px(px(12.0))
                .pt(px(6.0))
                .pb(px(2.0))
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgba((dimmed << 8) | ((HINT * 255.0) as u32)))
                .child(cat_name.to_string()),
        );

        for &idx in *indices {
            if let Some(item) = FILTERED_ITEMS.get(idx) {
                let selected = idx == 0;
                picker_col = picker_col.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(12.0))
                        .py(px(3.0))
                        .bg(if selected {
                            rgba((fg << 8) | 0x08)
                        } else {
                            transparent_black()
                        })
                        .child(
                            div()
                                .w(px(22.0))
                                .h(px(22.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded(px(5.0))
                                .bg(rgba((border << 8) | 0x14))
                                .text_xs()
                                .child(item.icon_char),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(if selected { rgb(fg) } else { rgb(dimmed) })
                                .child(item.label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgba((dimmed << 8) | 0x60))
                                .child(item.mention),
                        ),
                );
            }
        }
    }

    composer_shell(input, Some(picker_col.into_any_element()), None)
}

fn render_full_cursor_style() -> AnyElement {
    let t = get_cached_theme();
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;

    let input = input_with_chips_and_query(
        "Fix ",
        &[("@git-diff", None)],
        " @scr",
        ChipStyle::FlushAt,
    );

    let picker = div()
        .w_full()
        .py(px(2.0))
        .children(FILTERED_ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .py(px(2.0))
                .child(
                    div()
                        .text_xs()
                        .text_color(if selected { rgb(fg) } else { rgb(dimmed) })
                        .child(item.label),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(rgba((dimmed << 8) | 0x50))
                        .child(item.mention),
                )
        }))
        .into_any_element();

    composer_shell(input, Some(picker), None)
}

fn render_full_claude_code() -> AnyElement {
    let t = get_cached_theme();
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;

    let input = input_with_chips_and_query(
        "Refactor ",
        &[("@selection", None)],
        " @scr",
        ChipStyle::MonoCapsule,
    );

    let picker = div()
        .w_full()
        .py(px(2.0))
        .children(FILTERED_ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            let label_el = fuzzy_highlight_label(item.label, "scr", fg, dimmed, selected);
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .py(px(4.0))
                .bg(if selected {
                    rgba((fg << 8) | 0x06)
                } else {
                    transparent_black()
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .w(px(2.0))
                                .h(px(14.0))
                                .rounded(px(1.0))
                                .bg(if selected { rgb(GOLD) } else { transparent_black() }),
                        )
                        .child(label_el),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(rgba((dimmed << 8) | 0x50))
                        .child(item.mention),
                )
        }))
        .into_any_element();

    composer_shell(input, Some(picker), None)
}

fn render_full_multi_chip_flow() -> AnyElement {
    let input = input_with_chips_and_query(
        "Compare ",
        &[("@clipboard", None), ("@git-diff", None)],
        " and explain the differences",
        ChipStyle::GoldTint,
    );
    composer_shell(input, None, None)
}

fn render_full_dense_power() -> AnyElement {
    let t = get_cached_theme();
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;

    let input = input_with_chips_and_query(
        "",
        &[("@clipboard", None)],
        " @scr",
        ChipStyle::OutlinedTag,
    );

    let picker = div()
        .w_full()
        .py(px(1.0))
        .children(ALL_ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(8.0))
                .py(px(1.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .w(px(2.0))
                                .h(px(10.0))
                                .rounded(px(1.0))
                                .bg(if selected { rgb(GOLD) } else { transparent_black() }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(if selected { rgb(fg) } else { rgb(dimmed) })
                                .child(item.label),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(rgba((dimmed << 8) | 0x40))
                        .child(item.mention),
                )
        }))
        .into_any_element();

    composer_shell(input, Some(picker), None)
}

fn render_full_gold_signature() -> AnyElement {
    let t = get_cached_theme();
    let fg = t.colors.text.primary;
    let dimmed = t.colors.text.dimmed;

    let input = input_with_chips_and_query(
        "Summarize ",
        &[("@selection", None)],
        " @scr",
        ChipStyle::GoldTint,
    );

    let picker = div()
        .w_full()
        .py(px(2.0))
        .border_t_1()
        .border_color(rgba((GOLD << 8) | 0x10))
        .children(FILTERED_ITEMS.iter().enumerate().map(|(i, item)| {
            let selected = i == 0;
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .py(px(4.0))
                .bg(if selected {
                    rgba((GOLD << 8) | 0x08)
                } else {
                    transparent_black()
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .w(px(2.0))
                                .h(px(14.0))
                                .rounded(px(1.0))
                                .bg(if selected { rgb(GOLD) } else { transparent_black() }),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected {
                                    rgb(fg)
                                } else {
                                    rgb(dimmed)
                                })
                                .child(item.label),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(h(GOLD).opacity(if selected { MUTED_OP } else { HINT }))
                        .child(item.mention),
                )
        }))
        .into_any_element();

    composer_shell(input, Some(picker), None)
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
