//! Confirm Positioning — Micro style with keycap shortcuts
//!
//! Footer-replacing confirm with dimmed content. Micro aesthetic:
//! tiny text, tight spacing, keycap-style shortcut hints.

use gpui::*;

use crate::list_item::FONT_MONO;
use crate::storybook::{Story, StorySurface, StoryVariant};
use crate::theme::opacity::*;
use crate::ui_foundation::HexColorExt;

pub struct ConfirmPositionVariationsStory;

impl Story for ConfirmPositionVariationsStory {
    fn id(&self) -> &'static str {
        "confirm-position-variations"
    }

    fn name(&self) -> &'static str {
        "Confirm Micro+Keycap (5)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::MainMenu
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_positioned_confirm(&variant.stable_id())
    }

    fn render(&self) -> AnyElement {
        let variants = self.variants();
        crate::storybook::story_container()
            .child(
                crate::storybook::story_section("Micro + Keycap — Footer Replacement + Dim")
                    .children(variants.into_iter().enumerate().map(|(i, v)| {
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .mb_6()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0xfbbf24))
                                    .child(format!("{}. {}", i + 1, v.name)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgba(0xffffff99))
                                    .child(v.description.clone().unwrap_or_default()),
                            )
                            .child(render_positioned_confirm(&v.stable_id()))
                    })),
            )
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant::default_named("micro-single", "Micro Single Line").description(
                "Single-line message + right-aligned keycap actions. Tightest possible.",
            ),
            StoryVariant::default_named("micro-two-line", "Micro Two Line").description(
                "Title + body stacked, keycap row below. For confirms needing explanation.",
            ),
            StoryVariant::default_named("micro-gold-border", "Micro Gold Border")
                .description("Gold top border to draw attention. Single-line + keycaps."),
            StoryVariant::default_named("micro-danger", "Micro Danger")
                .description("Warning icon + red keycap delete action. Single-line danger."),
            StoryVariant::default_named("micro-danger-two-line", "Micro Danger Two-Line")
                .description(
                "Full danger: icon + title + body, red keycap row. Maximum info in minimum space.",
            ),
        ]
    }
}

// ─── Mock data ──────────────────────────────────────────────────────────

const MOCK_ITEMS: &[(&str, &str, &str)] = &[
    (
        "Clipboard History",
        "Browse and paste from clipboard",
        "\u{1f4cb}",
    ),
    (
        "Open Application",
        "Launch any app on your Mac",
        "\u{1f680}",
    ),
    ("Run Script", "Execute a Script Kit script", "\u{26a1}"),
    ("Search Files", "Find files across your system", "\u{1f50d}"),
    ("System Info", "View system information", "\u{1f4bb}"),
    ("Emoji Picker", "Search and insert emoji", "\u{1f60a}"),
];

// ─── Shared shell helpers ───────────────────────────────────────────────

fn shell(theme: &crate::theme::Theme) -> Div {
    div()
        .w_full()
        .max_w(px(480.))
        .h(px(420.))
        .flex()
        .flex_col()
        .rounded(px(12.))
        .border_1()
        .border_color(theme.colors.ui.border.to_rgb())
        .bg(theme.colors.background.main.to_rgb())
        .overflow_hidden()
}

fn shell_divider(theme: &crate::theme::Theme) -> Div {
    div()
        .w_full()
        .h(px(1.))
        .bg(theme.colors.ui.border.with_opacity(0.3))
}

fn shell_header(theme: &crate::theme::Theme) -> Div {
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
                .child(
                    div()
                        .w(px(1.5))
                        .h(px(18.))
                        .bg(theme.colors.accent.selected.to_rgb())
                        .rounded(px(1.)),
                ),
        )
        .child(
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
                ),
        )
}

fn shell_list(theme: &crate::theme::Theme, dim_opacity: f32) -> Div {
    let selected = 1usize;
    div()
        .flex_1()
        .min_h(px(0.))
        .flex()
        .flex_col()
        .overflow_hidden()
        .opacity(dim_opacity)
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
        .children(
            MOCK_ITEMS
                .iter()
                .enumerate()
                .map(|(i, &(name, _desc, icon))| {
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
                }),
        )
}

/// Keycap badge: small rounded box with mono text
fn keycap(bg: Hsla, color: Hsla, label: &str) -> Div {
    div()
        .px(px(4.))
        .py(px(1.))
        .rounded(px(3.))
        .bg(bg)
        .text_xs()
        .font_family(FONT_MONO)
        .text_color(color)
        .child(label.to_string())
}

// ─── Variant renderer ───────────────────────────────────────────────────

fn render_positioned_confirm(variant_id: &str) -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let accent = theme.colors.accent.selected;
    let error = theme.colors.ui.error;
    let text_primary = theme.colors.text.primary;
    let text_secondary = theme.colors.text.secondary;
    let text_dimmed = theme.colors.text.dimmed;
    let border = theme.colors.ui.border;

    // Shared dim level
    let dim = 0.55_f32;

    match variant_id {
        // ── 1. Micro Single Line ───────────────────────────────
        "micro-single" => shell(&theme)
            .child(div().opacity(dim).child(shell_header(&theme)))
            .child(shell_divider(&theme))
            .child(shell_list(&theme, dim))
            .child(
                div()
                    .w_full()
                    .px(px(10.))
                    .py(px(6.))
                    .border_t_1()
                    .border_color(border.with_opacity(0.3))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    // Message
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(text_primary.to_rgb())
                            .child("Clear conversation? This can\u{2019}t be undone."),
                    )
                    // Keycap action row
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .gap(px(12.))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(3.))
                                    .child(keycap(
                                        border.with_opacity(OPACITY_GHOST),
                                        text_dimmed.to_rgb(),
                                        "Esc",
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(text_dimmed.to_rgb())
                                            .child("Cancel"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(3.))
                                    .child(keycap(
                                        accent.with_opacity(OPACITY_GHOST),
                                        accent.to_rgb(),
                                        "\u{21b5}",
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(text_primary.to_rgb())
                                            .child("Clear"),
                                    ),
                            ),
                    ),
            )
            .into_any_element(),

        // ── 2. Micro Two Line ──────────────────────────────────
        "micro-two-line" => shell(&theme)
            .child(div().opacity(dim).child(shell_header(&theme)))
            .child(shell_divider(&theme))
            .child(shell_list(&theme, dim))
            .child(
                div()
                    .w_full()
                    .px(px(10.))
                    .py(px(6.))
                    .border_t_1()
                    .border_color(border.with_opacity(0.3))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    // Title + body
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(1.))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(text_primary.to_rgb())
                                    .child("Clear Conversation"),
                            )
                            .child(div().text_xs().text_color(text_secondary.to_rgb()).child(
                                "This will remove all messages. You can\u{2019}t undo this.",
                            )),
                    )
                    // Keycap action row
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .gap(px(12.))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(3.))
                                    .child(keycap(
                                        border.with_opacity(OPACITY_GHOST),
                                        text_dimmed.to_rgb(),
                                        "Esc",
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(text_dimmed.to_rgb())
                                            .child("Cancel"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(3.))
                                    .child(keycap(
                                        accent.with_opacity(OPACITY_GHOST),
                                        accent.to_rgb(),
                                        "\u{21b5}",
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(text_primary.to_rgb())
                                            .child("Clear"),
                                    ),
                            ),
                    ),
            )
            .into_any_element(),

        // ── 3. Micro Gold Border ───────────────────────────────
        "micro-gold-border" => shell(&theme)
            .child(div().opacity(dim).child(shell_header(&theme)))
            .child(shell_divider(&theme))
            .child(shell_list(&theme, dim))
            .child(
                div()
                    .w_full()
                    .px(px(10.))
                    .py(px(6.))
                    .border_t_2()
                    .border_color(accent.with_opacity(OPACITY_SELECTED))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(text_primary.to_rgb())
                            .child("Clear conversation? This can\u{2019}t be undone."),
                    )
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .gap(px(12.))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(3.))
                                    .child(keycap(
                                        border.with_opacity(OPACITY_GHOST),
                                        text_dimmed.to_rgb(),
                                        "Esc",
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(text_dimmed.to_rgb())
                                            .child("Cancel"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(3.))
                                    .child(keycap(
                                        accent.with_opacity(OPACITY_GHOST),
                                        accent.to_rgb(),
                                        "\u{21b5}",
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(text_primary.to_rgb())
                                            .child("Clear"),
                                    ),
                            ),
                    ),
            )
            .into_any_element(),

        // ── 4. Micro Danger ────────────────────────────────────
        "micro-danger" => shell(&theme)
            .child(div().opacity(dim).child(shell_header(&theme)))
            .child(shell_divider(&theme))
            .child(shell_list(&theme, dim))
            .child(
                div()
                    .w_full()
                    .px(px(10.))
                    .py(px(6.))
                    .border_t_1()
                    .border_color(error.with_opacity(OPACITY_SUBTLE))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(5.))
                            .child(div().text_xs().text_color(error.to_rgb()).child("\u{26a0}"))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(text_primary.to_rgb())
                                    .child("Delete script? This can\u{2019}t be undone."),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .gap(px(12.))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(3.))
                                    .child(keycap(
                                        border.with_opacity(OPACITY_GHOST),
                                        text_dimmed.to_rgb(),
                                        "Esc",
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(text_dimmed.to_rgb())
                                            .child("Cancel"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(3.))
                                    .child(keycap(
                                        error.with_opacity(OPACITY_GHOST),
                                        error.to_rgb(),
                                        "\u{21b5}",
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(error.to_rgb())
                                            .child("Delete"),
                                    ),
                            ),
                    ),
            )
            .into_any_element(),

        // ── 5. Micro Danger Two-Line ───────────────────────────
        "micro-danger-two-line" => {
            shell(&theme)
                .child(div().opacity(dim).child(shell_header(&theme)))
                .child(shell_divider(&theme))
                .child(shell_list(&theme, dim))
                .child(
                    div()
                        .w_full()
                        .px(px(10.))
                        .py(px(6.))
                        .border_t_1()
                        .border_color(error.with_opacity(OPACITY_SUBTLE))
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(1.))
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(5.))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(error.to_rgb())
                                                .child("\u{26a0}"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(text_primary.to_rgb())
                                                .child("Delete Script"),
                                        ),
                                )
                                .child(div().text_xs().text_color(text_secondary.to_rgb()).child(
                                    "This will permanently delete the script and its data.",
                                )),
                        )
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .flex_row()
                                .justify_end()
                                .gap(px(12.))
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(3.))
                                        .child(keycap(
                                            border.with_opacity(OPACITY_GHOST),
                                            text_dimmed.to_rgb(),
                                            "Esc",
                                        ))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(text_dimmed.to_rgb())
                                                .child("Cancel"),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(3.))
                                        .child(keycap(
                                            error.with_opacity(OPACITY_GHOST),
                                            error.to_rgb(),
                                            "\u{21b5}",
                                        ))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(error.to_rgb())
                                                .child("Delete"),
                                        ),
                                ),
                        ),
                )
                .into_any_element()
        }

        // Fallback
        _ => div()
            .p(px(16.))
            .child(
                div()
                    .text_sm()
                    .text_color(text_secondary.to_rgb())
                    .child(format!("Unknown variant: {}", variant_id)),
            )
            .into_any_element(),
    }
}
