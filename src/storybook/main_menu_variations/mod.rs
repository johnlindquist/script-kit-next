//! Main menu composition variations for the design explorer.
//!
//! Each variation is a complete mini-menu mockup that varies item height,
//! density, icon treatment, section headers, accent style, and overall
//! chrome. Some are refined, some are wild explorations.

use gpui::*;

use crate::list_item::FONT_MONO;
use crate::ui_foundation::HexColorExt;

use super::StoryVariant;

/// Stable IDs for main menu composition variations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MainMenuVariationId {
    RaycastClassic,
    CompactDense,
    Spotlight,
    TwoLine,
    MinimalFlat,
    BigCards,
    Terminal,
    Neon,
}

impl MainMenuVariationId {
    pub const ALL: [Self; 8] = [
        Self::RaycastClassic,
        Self::CompactDense,
        Self::Spotlight,
        Self::TwoLine,
        Self::MinimalFlat,
        Self::BigCards,
        Self::Terminal,
        Self::Neon,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::RaycastClassic => "raycast-classic",
            Self::CompactDense => "compact-dense",
            Self::Spotlight => "spotlight",
            Self::TwoLine => "two-line",
            Self::MinimalFlat => "minimal-flat",
            Self::BigCards => "big-cards",
            Self::Terminal => "terminal",
            Self::Neon => "neon",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::RaycastClassic => "Raycast Classic",
            Self::CompactDense => "Compact Dense",
            Self::Spotlight => "Spotlight",
            Self::TwoLine => "Two-Line",
            Self::MinimalFlat => "Minimal Flat",
            Self::BigCards => "Big Cards",
            Self::Terminal => "Terminal",
            Self::Neon => "Neon",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::RaycastClassic => {
                "Current layout: 40px items, icons, accent bar, sections, preview pane"
            }
            Self::CompactDense => {
                "32px items, tiny icons, no sections — max density, IDE command palette"
            }
            Self::Spotlight => {
                "Giant centered input, big type results, no chrome — Apple Spotlight"
            }
            Self::TwoLine => {
                "52px items with name + description stacked, large icons, section counts"
            }
            Self::MinimalFlat => "No icons, no bars, no footer — just text in a list. Brutalist.",
            Self::BigCards => {
                "Wild: each item is a card with icon, title, desc, rounded corners, gaps between"
            }
            Self::Terminal => {
                "Wild: monospace everything, > prefix, green-on-black, no rounded corners"
            }
            Self::Neon => "Wild: thick accent borders, glow effects, oversized input, bold colors",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "raycast-classic" => Some(Self::RaycastClassic),
            "compact-dense" => Some(Self::CompactDense),
            "spotlight" => Some(Self::Spotlight),
            "two-line" => Some(Self::TwoLine),
            "minimal-flat" => Some(Self::MinimalFlat),
            "big-cards" => Some(Self::BigCards),
            "terminal" => Some(Self::Terminal),
            "neon" => Some(Self::Neon),
            _ => None,
        }
    }
}

/// Mock items used in all previews.
const MOCK_ITEMS: &[(&str, &str, &str)] = &[
    ("Clipboard History", "Browse and paste from clipboard", "📋"),
    ("Open Application", "Launch any app on your Mac", "🚀"),
    ("Run Script", "Execute a Script Kit script", "⚡"),
    ("Search Files", "Find files across your system", "🔍"),
    ("System Info", "View system information", "💻"),
    ("Emoji Picker", "Search and insert emoji", "😊"),
];

/// Converts every variation into a `StoryVariant` with semantic props.
pub fn main_menu_story_variants() -> Vec<StoryVariant> {
    MainMenuVariationId::ALL
        .iter()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(id.description())
                .with_prop("surface", "main-menu")
                .with_prop("variantId", id.as_str())
        })
        .collect()
}

/// Renders a preview for the given stable ID inside a mock window shell.
pub fn render_main_menu_story_preview(stable_id: &str) -> AnyElement {
    let id = MainMenuVariationId::from_stable_id(stable_id)
        .unwrap_or(MainMenuVariationId::RaycastClassic);

    match id {
        MainMenuVariationId::RaycastClassic => render_raycast_classic(),
        MainMenuVariationId::CompactDense => render_compact_dense(),
        MainMenuVariationId::Spotlight => render_spotlight(),
        MainMenuVariationId::TwoLine => render_two_line(),
        MainMenuVariationId::MinimalFlat => render_minimal_flat(),
        MainMenuVariationId::BigCards => render_big_cards(),
        MainMenuVariationId::Terminal => render_terminal(),
        MainMenuVariationId::Neon => render_neon(),
    }
}

// ─── Shell helpers ──────────────────────────────────────────────────────

fn shell(theme: &crate::theme::Theme) -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .rounded(px(12.))
        .border_1()
        .border_color(theme.colors.ui.border.to_rgb())
        .bg(theme.colors.background.main.to_rgb())
}

fn divider(theme: &crate::theme::Theme) -> Div {
    div()
        .w_full()
        .h(px(1.))
        .bg(theme.colors.ui.border.with_opacity(0.3))
}

fn cursor(theme: &crate::theme::Theme, height: f32) -> Div {
    div()
        .w(px(1.5))
        .h(px(height))
        .bg(theme.colors.accent.selected.to_rgb())
        .rounded(px(1.))
}

fn ask_ai_badge(theme: &crate::theme::Theme) -> Div {
    div()
        .flex_shrink_0()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .px(px(6.))
                .py(px(3.))
                .rounded(px(4.))
                .bg(theme.colors.accent.selected.with_opacity(0.15))
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.colors.accent.selected.to_rgb())
                .child("Ask AI"),
        )
        .child(
            div()
                .px(px(5.))
                .py(px(2.))
                .rounded(px(3.))
                .bg(theme.colors.text.dimmed.with_opacity(0.12))
                .text_xs()
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child("Tab"),
        )
}

fn footer_hints(theme: &crate::theme::Theme) -> Div {
    div()
        .w_full()
        .h(px(36.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .border_t_1()
        .border_color(theme.colors.ui.border.with_opacity(0.3))
        .child(
            div()
                .text_xs()
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child("↵ Run  ·  ⌘K Actions  ·  Tab AI"),
        )
}

// ─── 1. Raycast Classic ─────────────────────────────────────────────────

fn render_raycast_classic() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 1;

    shell(&theme)
        // Header
        .child(
            div()
                .w_full()
                .px(px(16.))
                .py(px(10.))
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_size(px(16.))
                                .text_color(theme.colors.text.dimmed.to_rgb())
                                .child("Script Kit"),
                        )
                        .child(cursor(&theme, 18.)),
                )
                .child(ask_ai_badge(&theme)),
        )
        .child(divider(&theme))
        // Body — 50/50 split
        .child(
            div()
                .flex_1()
                .min_h(px(0.))
                .flex()
                .flex_row()
                // List half
                .child(
                    div()
                        .w(px(210.))
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        // Section header
                        .child(
                            div()
                                .w_full()
                                .h(px(28.))
                                .px(px(14.))
                                .flex()
                                .flex_row()
                                .items_center()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme.colors.text.dimmed.to_rgb())
                                        .child("Recent"),
                                ),
                        )
                        .children(MOCK_ITEMS.iter().enumerate().map(|(i, &(name, _, icon))| {
                            let is_sel = i == selected;
                            let mut row = div()
                                .w_full()
                                .h(px(40.))
                                .px(px(14.))
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(8.));

                            if is_sel {
                                row = row
                                    .bg(theme.colors.accent.selected.with_opacity(0.1))
                                    .child(
                                        div().w(px(3.)).h(px(20.)).rounded(px(3.)).bg(theme
                                            .colors
                                            .accent
                                            .selected
                                            .to_rgb()),
                                    );
                            }

                            row.child(
                                div()
                                    .w(px(20.))
                                    .h(px(20.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_size(px(16.))
                                    .child(icon.to_string()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .text_size(px(14.))
                                    .font_weight(if is_sel {
                                        FontWeight::MEDIUM
                                    } else {
                                        FontWeight::NORMAL
                                    })
                                    .text_color(if is_sel {
                                        theme.colors.text.primary.to_rgb()
                                    } else {
                                        theme.colors.text.secondary.to_rgb()
                                    })
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .child(name.to_string()),
                            )
                        })),
                )
                // Divider
                .child(
                    div()
                        .w(px(1.))
                        .h_full()
                        .bg(theme.colors.ui.border.with_opacity(0.2)),
                )
                // Preview half
                .child(
                    div()
                        .flex_1()
                        .min_h(px(0.))
                        .p_3()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.colors.text.primary.to_rgb())
                                .child(MOCK_ITEMS[selected].0),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.colors.text.muted.to_rgb())
                                .child(MOCK_ITEMS[selected].1),
                        )
                        .child(
                            div()
                                .mt_2()
                                .rounded(px(6.))
                                .bg(theme.colors.background.title_bar.to_rgb())
                                .p_2()
                                .text_xs()
                                .font_family(FONT_MONO)
                                .text_color(theme.colors.text.dimmed.to_rgb())
                                .child("// Preview placeholder"),
                        ),
                ),
        )
        .child(footer_hints(&theme))
        .into_any_element()
}

// ─── 2. Compact Dense ───────────────────────────────────────────────────

fn render_compact_dense() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 1;

    shell(&theme)
        .child(
            div()
                .w_full()
                .px(px(12.))
                .py(px(8.))
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_size(px(14.))
                                .text_color(theme.colors.text.dimmed.to_rgb())
                                .child("Script Kit"),
                        )
                        .child(cursor(&theme, 16.)),
                )
                .child(ask_ai_badge(&theme)),
        )
        .child(divider(&theme))
        .child(
            div()
                .flex_1()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .overflow_hidden()
                .children(MOCK_ITEMS.iter().enumerate().map(|(i, &(name, _, icon))| {
                    let is_sel = i == selected;
                    let mut row = div()
                        .w_full()
                        .h(px(30.))
                        .px(px(10.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.));

                    if is_sel {
                        row = row
                            .bg(theme.colors.accent.selected.with_opacity(0.1))
                            .child(
                                div().w(px(2.)).h(px(14.)).rounded(px(2.)).bg(theme
                                    .colors
                                    .accent
                                    .selected
                                    .to_rgb()),
                            );
                    }

                    row.child(
                        div()
                            .w(px(14.))
                            .h(px(14.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(11.))
                            .child(icon.to_string()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .text_size(px(12.5))
                            .text_color(if is_sel {
                                theme.colors.text.primary.to_rgb()
                            } else {
                                theme.colors.text.secondary.to_rgb()
                            })
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(name.to_string()),
                    )
                })),
        )
        .child(footer_hints(&theme))
        .into_any_element()
}

// ─── 3. Spotlight ───────────────────────────────────────────────────────

fn render_spotlight() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 1;

    shell(&theme)
        // Big input
        .child(
            div()
                .w_full()
                .px(px(20.))
                .py(px(16.))
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_size(px(24.))
                        .text_color(theme.colors.text.primary.to_rgb())
                        .child("Script Kit"),
                )
                .child(cursor(&theme, 26.)),
        )
        .child(divider(&theme))
        // Results — big, clean, no icons
        .child(
            div()
                .flex_1()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .py(px(4.))
                .overflow_hidden()
                .children(MOCK_ITEMS.iter().enumerate().map(|(i, &(name, _, _))| {
                    let is_sel = i == selected;
                    let mut row = div()
                        .w_full()
                        .h(px(44.))
                        .px(px(20.))
                        .flex()
                        .flex_row()
                        .items_center();

                    if is_sel {
                        row = row.bg(theme.colors.accent.selected.with_opacity(0.08));
                    }

                    row.child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .text_size(px(16.))
                            .font_weight(if is_sel {
                                FontWeight::MEDIUM
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(if is_sel {
                                theme.colors.text.primary.to_rgb()
                            } else {
                                theme.colors.text.muted.to_rgb()
                            })
                            .child(name.to_string()),
                    )
                })),
        )
        .into_any_element()
}

// ─── 4. Two-Line ────────────────────────────────────────────────────────

fn render_two_line() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 1;

    shell(&theme)
        .child(
            div()
                .w_full()
                .px(px(16.))
                .py(px(10.))
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_size(px(16.))
                                .text_color(theme.colors.text.dimmed.to_rgb())
                                .child("Script Kit"),
                        )
                        .child(cursor(&theme, 18.)),
                )
                .child(ask_ai_badge(&theme)),
        )
        .child(divider(&theme))
        .child(
            div()
                .flex_1()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .overflow_hidden()
                // Section header
                .child(
                    div()
                        .w_full()
                        .h(px(24.))
                        .px(px(14.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.colors.text.dimmed.to_rgb())
                                .child("Recent"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.colors.text.dimmed.with_opacity(0.5))
                                .child("6"),
                        ),
                )
                .children(
                    MOCK_ITEMS
                        .iter()
                        .enumerate()
                        .map(|(i, &(name, desc, icon))| {
                            let is_sel = i == selected;
                            let mut row = div()
                                .w_full()
                                .h(px(52.))
                                .px(px(14.))
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(10.));

                            if is_sel {
                                row = row
                                    .bg(theme.colors.accent.selected.with_opacity(0.1))
                                    .child(
                                        div().w(px(3.)).h(px(26.)).rounded(px(3.)).bg(theme
                                            .colors
                                            .accent
                                            .selected
                                            .to_rgb()),
                                    );
                            }

                            row.child(
                                div()
                                    .w(px(28.))
                                    .h(px(28.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_size(px(20.))
                                    .child(icon.to_string()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .flex()
                                    .flex_col()
                                    .gap(px(1.))
                                    .child(
                                        div()
                                            .text_size(px(14.))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(if is_sel {
                                                theme.colors.text.primary.to_rgb()
                                            } else {
                                                theme.colors.text.secondary.to_rgb()
                                            })
                                            .overflow_hidden()
                                            .whitespace_nowrap()
                                            .child(name.to_string()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.))
                                            .text_color(theme.colors.text.dimmed.to_rgb())
                                            .overflow_hidden()
                                            .whitespace_nowrap()
                                            .child(desc.to_string()),
                                    ),
                            )
                        }),
                ),
        )
        .child(footer_hints(&theme))
        .into_any_element()
}

// ─── 5. Minimal Flat ────────────────────────────────────────────────────

fn render_minimal_flat() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 1;

    shell(&theme)
        .child(
            div()
                .w_full()
                .px(px(16.))
                .py(px(10.))
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .text_size(px(16.))
                        .text_color(theme.colors.text.dimmed.to_rgb())
                        .child("Script Kit"),
                )
                .child(cursor(&theme, 18.)),
        )
        .child(divider(&theme))
        .child(
            div()
                .flex_1()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .py(px(4.))
                .overflow_hidden()
                .children(MOCK_ITEMS.iter().enumerate().map(|(i, &(name, _, _))| {
                    let is_sel = i == selected;
                    let mut row = div()
                        .w_full()
                        .h(px(34.))
                        .px(px(16.))
                        .flex()
                        .flex_row()
                        .items_center();

                    if is_sel {
                        row = row.bg(theme.colors.text.primary.with_opacity(0.06));
                    }

                    row.child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .text_size(px(14.))
                            .text_color(if is_sel {
                                theme.colors.text.primary.to_rgb()
                            } else {
                                theme.colors.text.muted.to_rgb()
                            })
                            .child(name.to_string()),
                    )
                })),
        )
        .into_any_element()
}

// ─── 6. Big Cards (wild) ────────────────────────────────────────────────

fn render_big_cards() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 1;

    shell(&theme)
        .child(
            div()
                .w_full()
                .px(px(16.))
                .py(px(12.))
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .px(px(14.))
                        .py(px(10.))
                        .rounded(px(10.))
                        .bg(theme.colors.background.title_bar.to_rgb())
                        .border_1()
                        .border_color(theme.colors.ui.border.to_rgb())
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(18.))
                                .text_color(theme.colors.text.primary.to_rgb())
                                .child("Script Kit"),
                        )
                        .child(cursor(&theme, 20.)),
                ),
        )
        // Cards grid
        .child(
            div()
                .flex_1()
                .min_h(px(0.))
                .px(px(12.))
                .py(px(8.))
                .flex()
                .flex_col()
                .gap(px(6.))
                .overflow_hidden()
                .children(
                    MOCK_ITEMS
                        .iter()
                        .enumerate()
                        .map(|(i, &(name, desc, icon))| {
                            let is_sel = i == selected;
                            let mut card = div()
                                .w_full()
                                .px(px(12.))
                                .py(px(10.))
                                .rounded(px(10.))
                                .border_1()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(10.));

                            if is_sel {
                                card = card
                                    .bg(theme.colors.accent.selected.with_opacity(0.08))
                                    .border_color(theme.colors.accent.selected.with_opacity(0.4));
                            } else {
                                card = card
                                    .bg(theme.colors.background.title_bar.with_opacity(0.5))
                                    .border_color(theme.colors.ui.border.with_opacity(0.3));
                            }

                            card.child(
                                div()
                                    .w(px(36.))
                                    .h(px(36.))
                                    .rounded(px(8.))
                                    .bg(if is_sel {
                                        theme.colors.accent.selected.with_opacity(0.12)
                                    } else {
                                        theme.colors.text.dimmed.with_opacity(0.08)
                                    })
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_size(px(20.))
                                    .child(icon.to_string()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.))
                                    .child(
                                        div()
                                            .text_size(px(14.))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(if is_sel {
                                                theme.colors.text.primary.to_rgb()
                                            } else {
                                                theme.colors.text.secondary.to_rgb()
                                            })
                                            .child(name.to_string()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .text_color(theme.colors.text.dimmed.to_rgb())
                                            .overflow_hidden()
                                            .whitespace_nowrap()
                                            .child(desc.to_string()),
                                    ),
                            )
                        }),
                ),
        )
        .into_any_element()
}

// ─── 7. Terminal (wild) ─────────────────────────────────────────────────

fn render_terminal() -> AnyElement {
    let _theme = crate::theme::get_cached_theme();
    let selected = 1;
    let green: u32 = 0x4ADE80;
    let dim_green: u32 = 0x22C55E;

    div()
        .w_full()
        .flex()
        .flex_col()
        .rounded(px(0.)) // Sharp corners!
        .border_1()
        .border_color(dim_green.with_opacity(0.3))
        .bg(rgb(0x0A0A0A))
        // Title bar
        .child(
            div()
                .w_full()
                .px(px(12.))
                .py(px(8.))
                .bg(rgb(0x111111))
                .border_b_1()
                .border_color(dim_green.with_opacity(0.2))
                .flex()
                .flex_row()
                .items_center()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_xs()
                        .text_color(dim_green.with_opacity(0.6))
                        .child("script-kit v0.1.0"),
                ),
        )
        // Input
        .child(
            div()
                .w_full()
                .px(px(12.))
                .py(px(10.))
                .border_b_1()
                .border_color(dim_green.with_opacity(0.15))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(14.))
                        .text_color(rgb(green))
                        .child(">"),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(14.))
                        .text_color(green.with_opacity(0.5))
                        .child("_"),
                ),
        )
        // List
        .child(
            div()
                .flex_1()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .overflow_hidden()
                .children(MOCK_ITEMS.iter().enumerate().map(|(i, &(name, _, _))| {
                    let is_sel = i == selected;
                    let mut row = div()
                        .w_full()
                        .h(px(32.))
                        .px(px(12.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(8.));

                    if is_sel {
                        row = row.bg(green.with_opacity(0.1));
                    }

                    row.child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(12.))
                            .text_color(if is_sel {
                                green.to_rgb()
                            } else {
                                green.to_rgb().opacity(0.3)
                            })
                            .child(if is_sel { "▸" } else { " " }),
                    )
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(13.))
                            .text_color(if is_sel {
                                green.to_rgb()
                            } else {
                                green.to_rgb().opacity(0.5)
                            })
                            .child(name.to_lowercase().replace(' ', "-")),
                    )
                })),
        )
        // Footer
        .child(
            div()
                .w_full()
                .h(px(28.))
                .px(px(12.))
                .bg(rgb(0x111111))
                .border_t_1()
                .border_color(dim_green.with_opacity(0.2))
                .flex()
                .flex_row()
                .items_center()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_xs()
                        .text_color(dim_green.with_opacity(0.4))
                        .child("RET exec  |  ^K actions  |  TAB ai"),
                ),
        )
        .into_any_element()
}

// ─── 8. Neon (wild) ─────────────────────────────────────────────────────

fn render_neon() -> AnyElement {
    let _theme = crate::theme::get_cached_theme();
    let selected = 1;
    let neon_pink: u32 = 0xFF6EC7;
    let neon_blue: u32 = 0x00D4FF;
    let dark_bg: u32 = 0x0D0D1A;
    let card_bg: u32 = 0x14142B;

    div()
        .w_full()
        .flex()
        .flex_col()
        .rounded(px(16.))
        .border_2()
        .border_color(neon_pink.with_opacity(0.4))
        .bg(rgb(dark_bg))
        // Header — oversized pill input
        .child(
            div().w_full().px(px(16.)).py(px(14.)).child(
                div()
                    .w_full()
                    .px(px(16.))
                    .py(px(12.))
                    .rounded(px(20.))
                    .border_2()
                    .border_color(neon_blue.with_opacity(0.5))
                    .bg(rgb(card_bg))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(20.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(neon_blue))
                            .child("Script Kit"),
                    )
                    .child(
                        div()
                            .w(px(2.))
                            .h(px(22.))
                            .bg(rgb(neon_pink))
                            .rounded(px(1.)),
                    ),
            ),
        )
        // List with glow cards
        .child(
            div()
                .flex_1()
                .min_h(px(0.))
                .px(px(12.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .overflow_hidden()
                .children(
                    MOCK_ITEMS
                        .iter()
                        .enumerate()
                        .map(|(i, &(name, desc, icon))| {
                            let is_sel = i == selected;

                            let mut row = div()
                                .w_full()
                                .h(px(48.))
                                .px(px(14.))
                                .rounded(px(12.))
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(12.));

                            if is_sel {
                                row = row
                                    .bg(neon_pink.with_opacity(0.12))
                                    .border_1()
                                    .border_color(neon_pink.with_opacity(0.5));
                            } else {
                                row = row
                                    .bg(card_bg.with_opacity(0.6))
                                    .border_1()
                                    .border_color(neon_blue.with_opacity(0.1));
                            }

                            row.child(
                                div()
                                    .w(px(32.))
                                    .h(px(32.))
                                    .rounded(px(8.))
                                    .bg(if is_sel {
                                        neon_pink.with_opacity(0.15)
                                    } else {
                                        neon_blue.with_opacity(0.08)
                                    })
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_size(px(18.))
                                    .child(icon.to_string()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .flex()
                                    .flex_col()
                                    .gap(px(1.))
                                    .child(
                                        div()
                                            .text_size(px(14.))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(if is_sel {
                                                rgb(neon_pink)
                                            } else {
                                                rgb(0xE0E0F0)
                                            })
                                            .child(name.to_string()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .text_color(neon_blue.with_opacity(0.5))
                                            .overflow_hidden()
                                            .whitespace_nowrap()
                                            .child(desc.to_string()),
                                    ),
                            )
                        }),
                ),
        )
        // Footer
        .child(
            div()
                .w_full()
                .h(px(36.))
                .px(px(16.))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(neon_pink.with_opacity(0.6))
                        .child("↵ RUN"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(neon_blue.with_opacity(0.4))
                        .child("⌘K Actions  ·  Tab AI"),
                ),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests;
