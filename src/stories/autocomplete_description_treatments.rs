//! Autocomplete Description Treatments
//!
//! 21 focused mockups for showing the selected item's description in both
//! slash-command and @-mention popups. Each variant renders the same treatment
//! across both surfaces so it is easy to compare cross-surface consistency.

use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::list_item::FONT_MONO;
use crate::storybook::{story_container, story_section, Story, StorySurface, StoryVariant};
use crate::theme::get_cached_theme;

const CARD_W: f32 = 360.0;
const POPUP_W: f32 = 336.0;
const GOLD: u32 = 0xfbbf24;
const GHOST_HI: f32 = 0.07;
const HINT: f32 = 0.48;
const SELECTED_INDEX: usize = 1;

#[derive(Clone, Copy)]
enum SurfaceKind {
    Slash,
    Mention,
}

#[derive(Clone, Copy)]
struct PopupItem {
    label: &'static str,
    trigger: &'static str,
    description: &'static str,
    category: &'static str,
    icon: &'static str,
}

const SLASH_ITEMS: &[PopupItem] = &[
    PopupItem {
        label: "Current Context",
        trigger: "/context",
        description: "Attach the current desktop snapshot.",
        category: "Context",
        icon: "◉",
    },
    PopupItem {
        label: "Selection",
        trigger: "/selection",
        description: "Use the selected text from the frontmost app.",
        category: "Context",
        icon: "▋",
    },
    PopupItem {
        label: "Browser URL",
        trigger: "/browser",
        description: "Attach the active browser tab URL.",
        category: "Source",
        icon: "◆",
    },
    PopupItem {
        label: "Git Diff",
        trigger: "/git-diff",
        description: "Include the repo's uncommitted changes.",
        category: "Repo",
        icon: "±",
    },
];

const MENTION_ITEMS: &[PopupItem] = &[
    PopupItem {
        label: "Clipboard",
        trigger: "@clipboard",
        description: "Paste the current clipboard contents as context.",
        category: "Context",
        icon: "▣",
    },
    PopupItem {
        label: "Selection",
        trigger: "@selection",
        description: "Use the selected text from the frontmost app.",
        category: "Context",
        icon: "▋",
    },
    PopupItem {
        label: "Screenshot",
        trigger: "@screenshot",
        description: "Capture the current desktop and attach it.",
        category: "Capture",
        icon: "◌",
    },
    PopupItem {
        label: "Browser URL",
        trigger: "@browser",
        description: "Reference the active browser tab URL.",
        category: "Source",
        icon: "◆",
    },
];

const TREATMENTS: [(&str, &str, &str); 21] = [
    (
        "inline-tail",
        "Inline Tail",
        "Selected row appends a muted description after the label.",
    ),
    (
        "inline-tail-fade-meta",
        "Inline Tail + Faded Meta",
        "Inline description with the command token pushed quieter on focus.",
    ),
    (
        "inline-dot-tail",
        "Inline Dot Tail",
        "Inline description separated with a subtle middle dot.",
    ),
    (
        "two-line-row",
        "Two-Line Row",
        "Selected row grows to a second line for the description.",
    ),
    (
        "two-line-meta-stack",
        "Two-Line Meta Stack",
        "Description on line two with the token tucked beneath it.",
    ),
    (
        "two-line-replace-meta",
        "Two-Line Replace Meta",
        "Right token disappears on focus so the description gets the width.",
    ),
    (
        "below-row-caption",
        "Below-Row Caption",
        "A compact caption appears directly under the focused row only.",
    ),
    (
        "footer-preview",
        "Footer Preview",
        "List stays compact; the description sits in a preview strip below.",
    ),
    (
        "footer-preview-icon",
        "Footer Preview + Icon",
        "Preview strip adds a small icon/category cue.",
    ),
    (
        "footer-preview-hint",
        "Footer Preview + Hint",
        "Preview strip also advertises the primary key action.",
    ),
    (
        "sidecar-summary",
        "Sidecar Summary",
        "A narrow right rail shows the focused description.",
    ),
    (
        "sidecar-rich",
        "Sidecar Rich Summary",
        "Right rail adds category and token context, not just the sentence.",
    ),
    (
        "header-summary",
        "Header Summary",
        "Focused item summary appears above the list before the rows.",
    ),
    (
        "floating-note",
        "Floating Note",
        "A small note block sits immediately below the selected row.",
    ),
    (
        "expanded-focus-card",
        "Expanded Focus Card",
        "Selected row opens into a mini card while other rows stay dense.",
    ),
    (
        "expanded-accent-card",
        "Expanded Accent Card",
        "Expanded card with a stronger accent edge for the focus state.",
    ),
    (
        "bottom-doc-band",
        "Bottom Doc Band",
        "A quieter documentation band spans the bottom of the popup.",
    ),
    (
        "category-chip-summary",
        "Category Chip + Summary",
        "Selected row adds a chip before the description for quicker scanning.",
    ),
    (
        "match-highlight-summary",
        "Match Highlight + Summary",
        "Description appears with a stronger highlight treatment on the match.",
    ),
    (
        "compact-synopsis",
        "Compact Synopsis",
        "One-line synopsis lives under the list with almost no extra chrome.",
    ),
    (
        "hybrid-best-of",
        "Hybrid Best-of",
        "Two-line focus row plus a soft preview strip for maximum clarity.",
    ),
];

fn h(hex: u32) -> Hsla {
    Hsla::from(rgb(hex))
}

fn gold() -> Hsla {
    h(GOLD)
}

pub struct AutocompleteDescriptionTreatmentsStory;

impl Story for AutocompleteDescriptionTreatmentsStory {
    fn id(&self) -> &'static str {
        "autocomplete-description-treatments"
    }

    fn name(&self) -> &'static str {
        "Autocomplete Description Treatments (21)"
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

        container =
            container.child(story_section("Inline Row Treatments (1–7)").children(
                variants[0..7].iter().enumerate().map(|(i, variant)| {
                    variation_row(i + 1, variant, self.render_variant(variant))
                }),
            ));

        container = container.child(
            story_section("Detached Preview Treatments (8–14)").children(
                variants[7..14].iter().enumerate().map(|(i, variant)| {
                    variation_row(i + 8, variant, self.render_variant(variant))
                }),
            ),
        );

        container =
            container.child(story_section("Hybrid Treatments (15–21)").children(
                variants[14..21].iter().enumerate().map(|(i, variant)| {
                    variation_row(i + 15, variant, self.render_variant(variant))
                }),
            ));

        container.into_any_element()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let id = variant.stable_id();
        div()
            .flex()
            .gap_3()
            .children([
                preview_card(
                    "Slash popup",
                    "/sel",
                    render_popup(SurfaceKind::Slash, id.as_str()),
                )
                .into_any_element(),
                preview_card(
                    "Mention popup",
                    "Explain @sel",
                    render_popup(SurfaceKind::Mention, id.as_str()),
                )
                .into_any_element(),
            ])
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        TREATMENTS
            .iter()
            .map(|(id, name, description)| {
                StoryVariant::default_named(*id, *name).description(*description)
            })
            .collect()
    }
}

fn variation_row(index: usize, variant: &StoryVariant, element: AnyElement) -> Div {
    let theme = get_cached_theme();

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
                        .text_color(rgb(theme.colors.text.primary))
                        .child(format!("{}. {}", index, variant.name)),
                )
                .when(variant.description.is_some(), |d| {
                    d.child(
                        div()
                            .text_xs()
                            .text_color(rgb(theme.colors.text.dimmed))
                            .child(variant.description.clone().unwrap_or_default()),
                    )
                }),
        )
        .child(element)
}

fn preview_card(title: &str, input_text: &str, popup: AnyElement) -> Div {
    let theme = get_cached_theme();
    let border = h(theme.colors.ui.border);
    let fg = h(theme.colors.text.primary);

    div()
        .w(px(CARD_W))
        .p_3()
        .rounded(px(14.0))
        .border_1()
        .border_color(border.opacity(0.25))
        .bg(fg.opacity(0.015))
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_xs()
                .text_color(fg.opacity(HINT))
                .child(title.to_string()),
        )
        .child(input_shell(input_text))
        .child(popup)
}

fn input_shell(input_text: &str) -> Div {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let border = h(theme.colors.ui.border);

    div()
        .w(px(POPUP_W))
        .px(px(10.0))
        .py(px(7.0))
        .rounded(px(10.0))
        .border_1()
        .border_color(border.opacity(0.22))
        .bg(fg.opacity(0.018))
        .flex()
        .items_center()
        .child(div().text_sm().text_color(fg).child(input_text.to_string()))
        .child(
            div()
                .ml(px(2.0))
                .w(px(2.0))
                .h(px(15.0))
                .rounded(px(1.0))
                .bg(gold()),
        )
}

fn render_popup(surface: SurfaceKind, treatment: &str) -> AnyElement {
    match treatment {
        "inline-tail" => popup_with_selected_row(
            surface,
            selected_inline_tail_row(surface, false),
            None,
            None,
            None,
            None,
        ),
        "inline-tail-fade-meta" => popup_with_selected_row(
            surface,
            selected_inline_tail_row(surface, true),
            None,
            None,
            None,
            None,
        ),
        "inline-dot-tail" => popup_with_selected_row(
            surface,
            selected_inline_dot_row(surface),
            None,
            None,
            None,
            None,
        ),
        "two-line-row" => popup_with_selected_row(
            surface,
            selected_two_line_row(surface, true),
            None,
            None,
            None,
            None,
        ),
        "two-line-meta-stack" => popup_with_selected_row(
            surface,
            selected_two_line_meta_stack_row(surface),
            None,
            None,
            None,
            None,
        ),
        "two-line-replace-meta" => popup_with_selected_row(
            surface,
            selected_two_line_row(surface, false),
            None,
            None,
            None,
            None,
        ),
        "below-row-caption" => popup_with_selected_row(
            surface,
            selected_standard_row(surface, true),
            None,
            Some(focused_caption(surface, false)),
            None,
            None,
        ),
        "footer-preview" => popup_with_selected_row(
            surface,
            selected_standard_row(surface, true),
            None,
            None,
            Some(summary_strip(surface, false, false, false)),
            None,
        ),
        "footer-preview-icon" => popup_with_selected_row(
            surface,
            selected_standard_row(surface, true),
            None,
            None,
            Some(summary_strip(surface, true, false, false)),
            None,
        ),
        "footer-preview-hint" => popup_with_selected_row(
            surface,
            selected_standard_row(surface, true),
            None,
            None,
            Some(summary_strip(surface, true, true, false)),
            None,
        ),
        "sidecar-summary" => popup_with_selected_row(
            surface,
            selected_standard_row(surface, true),
            None,
            None,
            None,
            Some(sidecar_summary(surface, false)),
        ),
        "sidecar-rich" => popup_with_selected_row(
            surface,
            selected_standard_row(surface, true),
            None,
            None,
            None,
            Some(sidecar_summary(surface, true)),
        ),
        "header-summary" => popup_with_selected_row(
            surface,
            selected_standard_row(surface, true),
            Some(summary_strip(surface, false, false, true)),
            None,
            None,
            None,
        ),
        "floating-note" => popup_with_selected_row(
            surface,
            selected_standard_row(surface, true),
            None,
            Some(floating_note(surface)),
            None,
            None,
        ),
        "expanded-focus-card" => popup_with_selected_row(
            surface,
            selected_expanded_card(surface, false),
            None,
            None,
            None,
            None,
        ),
        "expanded-accent-card" => popup_with_selected_row(
            surface,
            selected_expanded_card(surface, true),
            None,
            None,
            None,
            None,
        ),
        "bottom-doc-band" => popup_with_selected_row(
            surface,
            selected_standard_row(surface, true),
            None,
            None,
            Some(doc_band(surface)),
            None,
        ),
        "category-chip-summary" => popup_with_selected_row(
            surface,
            selected_chip_summary_row(surface),
            None,
            None,
            None,
            None,
        ),
        "match-highlight-summary" => popup_with_selected_row(
            surface,
            selected_match_summary_row(surface),
            None,
            None,
            None,
            None,
        ),
        "compact-synopsis" => popup_with_selected_row(
            surface,
            selected_standard_row(surface, true),
            None,
            None,
            Some(
                summary_strip(surface, false, false, false)
                    .px(px(8.0))
                    .py(px(5.0)),
            ),
            None,
        ),
        "hybrid-best-of" => popup_with_selected_row(
            surface,
            selected_two_line_row(surface, true),
            None,
            None,
            Some(summary_strip(surface, true, false, false)),
            None,
        ),
        _ => popup_with_selected_row(
            surface,
            selected_inline_tail_row(surface, false),
            None,
            None,
            None,
            None,
        ),
    }
}

fn popup_with_selected_row(
    surface: SurfaceKind,
    selected_row: AnyElement,
    header: Option<Div>,
    after_selected: Option<Div>,
    footer: Option<Div>,
    sidecar: Option<Div>,
) -> AnyElement {
    let items = items_for(surface);
    let theme = get_cached_theme();
    let border = h(theme.colors.ui.border);

    let mut list = div().flex().flex_col();
    let mut selected_row = Some(selected_row);
    let mut after_selected = after_selected;

    if let Some(header) = header {
        list = list.child(header).child(divider(false));
    }

    for (index, item) in items.iter().enumerate() {
        if index == SELECTED_INDEX {
            if let Some(row) = selected_row.take() {
                list = list.child(row);
            }
            if let Some(note) = after_selected.take() {
                list = list.child(note);
            }
        } else {
            list = list.child(base_row(*item, false));
        }
    }

    if let Some(footer) = footer {
        list = list.child(divider(false)).child(footer);
    }

    let body = if let Some(sidecar) = sidecar {
        div()
            .flex()
            .items_start()
            .child(div().w(px(194.0)).child(list))
            .child(divider(true).mx(px(6.0)))
            .child(div().w(px(118.0)).child(sidecar))
    } else {
        div().child(list)
    };

    div()
        .w(px(POPUP_W))
        .rounded(px(12.0))
        .border_1()
        .border_color(border.opacity(0.22))
        .bg(h(theme.colors.text.primary).opacity(0.018))
        .overflow_hidden()
        .child(body)
        .into_any_element()
}

fn base_row(item: PopupItem, selected: bool) -> Div {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);

    div()
        .px(px(8.0))
        .py(px(6.0))
        .bg(if selected {
            fg.opacity(GHOST_HI)
        } else {
            transparent_black()
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(selection_bar(selected, 14.0))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .justify_between()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if selected { fg } else { fg.opacity(0.82) })
                                .child(item.label),
                        )
                        .child(meta_text(item.trigger, selected, false)),
                ),
        )
}

fn selected_standard_row(surface: SurfaceKind, show_meta: bool) -> AnyElement {
    let item = selected_item(surface);
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);

    div()
        .px(px(8.0))
        .py(px(6.0))
        .bg(fg.opacity(GHOST_HI))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(selection_bar(true, 14.0))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .justify_between()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(fg)
                                .child(item.label),
                        )
                        .when(show_meta, |d| d.child(meta_text(item.trigger, true, false))),
                ),
        )
        .into_any_element()
}

fn selected_inline_tail_row(surface: SurfaceKind, fade_meta: bool) -> AnyElement {
    let item = selected_item(surface);
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let dim = h(theme.colors.text.dimmed);

    div()
        .px(px(8.0))
        .py(px(6.0))
        .bg(fg.opacity(GHOST_HI))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(selection_bar(true, 14.0))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .justify_between()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(fg)
                                        .child(item.label),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(dim.opacity(HINT))
                                        .child(item.description),
                                ),
                        )
                        .child(meta_text(item.trigger, true, fade_meta)),
                ),
        )
        .into_any_element()
}

fn selected_inline_dot_row(surface: SurfaceKind) -> AnyElement {
    let item = selected_item(surface);
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let dim = h(theme.colors.text.dimmed);

    div()
        .px(px(8.0))
        .py(px(6.0))
        .bg(fg.opacity(GHOST_HI))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(selection_bar(true, 14.0))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .justify_between()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(fg)
                                        .child(item.label),
                                )
                                .child(div().text_xs().text_color(dim.opacity(0.3)).child("•"))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(dim.opacity(HINT))
                                        .child(item.description),
                                ),
                        )
                        .child(meta_text(item.trigger, true, true)),
                ),
        )
        .into_any_element()
}

fn selected_two_line_row(surface: SurfaceKind, show_meta: bool) -> AnyElement {
    let item = selected_item(surface);
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let dim = h(theme.colors.text.dimmed);

    div()
        .px(px(8.0))
        .py(px(6.0))
        .bg(fg.opacity(GHOST_HI))
        .child(
            div()
                .flex()
                .items_start()
                .gap_2()
                .child(selection_bar(true, 26.0))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(fg)
                                        .child(item.label),
                                )
                                .when(show_meta, |d| d.child(meta_text(item.trigger, true, true))),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(dim.opacity(HINT))
                                .child(item.description),
                        ),
                ),
        )
        .into_any_element()
}

fn selected_two_line_meta_stack_row(surface: SurfaceKind) -> AnyElement {
    let item = selected_item(surface);
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let dim = h(theme.colors.text.dimmed);

    div()
        .px(px(8.0))
        .py(px(6.0))
        .bg(fg.opacity(GHOST_HI))
        .child(
            div()
                .flex()
                .items_start()
                .gap_2()
                .child(selection_bar(true, 30.0))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(fg)
                                .child(item.label),
                        )
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(dim.opacity(HINT))
                                        .child(item.description),
                                )
                                .child(meta_text(item.trigger, true, true)),
                        ),
                ),
        )
        .into_any_element()
}

fn focused_caption(surface: SurfaceKind, compact: bool) -> Div {
    let item = selected_item(surface);
    let theme = get_cached_theme();
    let dim = h(theme.colors.text.dimmed);

    div()
        .px(px(18.0))
        .pb(px(if compact { 4.0 } else { 6.0 }))
        .child(
            div()
                .text_xs()
                .text_color(dim.opacity(if compact { 0.42 } else { HINT }))
                .child(item.description),
        )
}

fn summary_strip(surface: SurfaceKind, show_icon: bool, show_hint: bool, as_header: bool) -> Div {
    let item = selected_item(surface);
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let dim = h(theme.colors.text.dimmed);

    div()
        .px(px(10.0))
        .py(px(if as_header { 8.0 } else { 7.0 }))
        .bg(fg.opacity(if as_header { 0.022 } else { 0.018 }))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .when(show_icon, |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(gold().opacity(0.75))
                                    .child(item.icon),
                            )
                        })
                        .child(
                            div()
                                .text_xs()
                                .text_color(dim.opacity(HINT))
                                .child(if as_header {
                                    "Focused item"
                                } else {
                                    "Selection"
                                }),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(dim.opacity(0.38))
                        .font_family(FONT_MONO)
                        .child(item.trigger),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(fg.opacity(0.86))
                .child(item.description),
        )
        .when(show_hint, |d| {
            d.child(
                div()
                    .text_xs()
                    .text_color(dim.opacity(0.36))
                    .child(match surface {
                        SurfaceKind::Slash => "Enter inserts command",
                        SurfaceKind::Mention => "Enter inserts mention",
                    }),
            )
        })
}

fn sidecar_summary(surface: SurfaceKind, rich: bool) -> Div {
    let item = selected_item(surface);
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let dim = h(theme.colors.text.dimmed);

    div()
        .pt(px(8.0))
        .pr(px(8.0))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(dim.opacity(HINT))
                .child(if rich { item.category } else { "Focused" }),
        )
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(fg)
                .child(item.label),
        )
        .child(
            div()
                .text_xs()
                .text_color(dim.opacity(HINT))
                .child(item.description),
        )
        .when(rich, |d| {
            d.child(
                div()
                    .pt(px(4.0))
                    .text_xs()
                    .text_color(dim.opacity(0.34))
                    .font_family(FONT_MONO)
                    .child(item.trigger),
            )
        })
}

fn floating_note(surface: SurfaceKind) -> Div {
    let item = selected_item(surface);
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let dim = h(theme.colors.text.dimmed);

    div().px(px(18.0)).pb(px(6.0)).child(
        div()
            .px(px(8.0))
            .py(px(6.0))
            .rounded(px(8.0))
            .bg(fg.opacity(0.032))
            .border_1()
            .border_color(gold().opacity(0.18))
            .text_xs()
            .text_color(dim.opacity(HINT))
            .child(item.description),
    )
}

fn selected_expanded_card(surface: SurfaceKind, accent_edge: bool) -> AnyElement {
    let item = selected_item(surface);
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let dim = h(theme.colors.text.dimmed);
    let border = h(theme.colors.ui.border);

    div()
        .px(px(8.0))
        .py(px(6.0))
        .child(
            div()
                .rounded(px(10.0))
                .border_1()
                .border_color(if accent_edge {
                    gold().opacity(0.32)
                } else {
                    border.opacity(0.28)
                })
                .bg(fg.opacity(0.05))
                .child(
                    div()
                        .px(px(10.0))
                        .py(px(8.0))
                        .flex()
                        .items_start()
                        .gap_2()
                        .child(selection_bar(true, 30.0))
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .items_center()
                                        .gap_3()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(fg)
                                                .child(item.label),
                                        )
                                        .child(meta_text(item.trigger, true, true)),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(dim.opacity(HINT))
                                        .child(item.description),
                                ),
                        ),
                ),
        )
        .into_any_element()
}

fn doc_band(surface: SurfaceKind) -> Div {
    let item = selected_item(surface);
    let theme = get_cached_theme();
    let dim = h(theme.colors.text.dimmed);

    div()
        .px(px(10.0))
        .py(px(7.0))
        .bg(gold().opacity(0.045))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(dim.opacity(0.4))
                .child("Focused description"),
        )
        .child(
            div()
                .text_xs()
                .text_color(dim.opacity(HINT))
                .child(item.description),
        )
}

fn selected_chip_summary_row(surface: SurfaceKind) -> AnyElement {
    let item = selected_item(surface);
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);
    let dim = h(theme.colors.text.dimmed);

    div()
        .px(px(8.0))
        .py(px(6.0))
        .bg(fg.opacity(GHOST_HI))
        .child(
            div()
                .flex()
                .items_start()
                .gap_2()
                .child(selection_bar(true, 26.0))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(fg)
                                        .child(item.label),
                                )
                                .child(meta_text(item.trigger, true, true)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(category_chip(item.category))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(dim.opacity(HINT))
                                        .child(item.description),
                                ),
                        ),
                ),
        )
        .into_any_element()
}

fn selected_match_summary_row(surface: SurfaceKind) -> AnyElement {
    let item = selected_item(surface);
    let theme = get_cached_theme();
    let dim = h(theme.colors.text.dimmed);

    div()
        .px(px(8.0))
        .py(px(6.0))
        .bg(gold().opacity(0.06))
        .child(
            div()
                .flex()
                .items_start()
                .gap_2()
                .child(selection_bar(true, 26.0))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .gap_3()
                                .child(highlight_selection_label(item.label))
                                .child(meta_text(item.trigger, true, true)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(dim.opacity(HINT))
                                .child(item.description),
                        ),
                ),
        )
        .into_any_element()
}

fn highlight_selection_label(label: &str) -> Div {
    let theme = get_cached_theme();
    let fg = h(theme.colors.text.primary);

    let lower = label.to_lowercase();
    if let Some(start) = lower.find("sel") {
        let end = start + 3;
        div()
            .flex()
            .items_center()
            .gap(px(0.0))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(fg)
                    .child(label[..start].to_string()),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(gold())
                    .child(label[start..end].to_string()),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(fg)
                    .child(label[end..].to_string()),
            )
    } else {
        div()
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_color(fg)
            .child(label.to_string())
    }
}

fn category_chip(text: &str) -> Div {
    div()
        .px(px(5.0))
        .py(px(1.0))
        .rounded(px(999.0))
        .bg(gold().opacity(0.1))
        .text_xs()
        .text_color(gold().opacity(0.82))
        .child(text.to_string())
}

fn selection_bar(selected: bool, height: f32) -> Div {
    div()
        .w(px(2.0))
        .h(px(height))
        .rounded(px(2.0))
        .bg(if selected {
            gold()
        } else {
            transparent_black()
        })
}

fn meta_text(text: &str, selected: bool, subdued: bool) -> Div {
    let theme = get_cached_theme();
    let dim = h(theme.colors.text.dimmed);

    div()
        .font_family(FONT_MONO)
        .text_xs()
        .text_color(dim.opacity(if subdued {
            0.34
        } else if selected {
            0.42
        } else {
            HINT
        }))
        .child(text.to_string())
}

fn divider(vertical: bool) -> Div {
    let theme = get_cached_theme();
    let border = h(theme.colors.ui.border);

    if vertical {
        div().w(px(1.0)).self_stretch().bg(border.opacity(0.22))
    } else {
        div().h(px(1.0)).bg(border.opacity(0.18))
    }
}

fn items_for(surface: SurfaceKind) -> &'static [PopupItem] {
    match surface {
        SurfaceKind::Slash => SLASH_ITEMS,
        SurfaceKind::Mention => MENTION_ITEMS,
    }
}

fn selected_item(surface: SurfaceKind) -> PopupItem {
    items_for(surface)[SELECTED_INDEX]
}

#[cfg(test)]
mod tests {
    use super::AutocompleteDescriptionTreatmentsStory;
    use crate::storybook::{Story, StorySurface};

    #[test]
    fn autocomplete_description_story_has_21_variants() {
        let story = AutocompleteDescriptionTreatmentsStory;
        assert_eq!(story.surface(), StorySurface::MiniAiChat);
        assert_eq!(story.variants().len(), 21);
    }

    #[test]
    fn autocomplete_description_variant_ids_are_unique() {
        let story = AutocompleteDescriptionTreatmentsStory;
        let ids: Vec<_> = story
            .variants()
            .iter()
            .map(|variant| variant.stable_id())
            .collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len(), "duplicate variant ids: {ids:?}");
    }
}
