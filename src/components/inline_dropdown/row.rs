//! Shared dense-monoline picker row for InlineDropdown surfaces.
//!
//! All inline dropdown consumers (ACP popups, AI context pickers, model
//! selectors, presets dropdown) render through these helpers so the chrome
//! stays consistent: footer-aligned row height, 2px gold accent bar on
//! selection, fuzzy-match highlights, and optional leading visual / accessory.

use gpui::{
    div, prelude::FluentBuilder, px, AnyElement, FontWeight, Hsla, InteractiveElement, IntoElement,
    ParentElement, SharedString, Styled,
};
use std::collections::HashSet;

use crate::list_item::FONT_MONO;
use crate::ui::chrome::HINT_STRIP_HEIGHT;

/// Gold accent (#fbbf24) — the one warm signature touch.
pub(crate) const GOLD: Hsla = Hsla {
    h: 0.1194,
    s: 0.956,
    l: 0.565,
    a: 1.0,
};

/// Impeccable opacity tiers.
pub(crate) const GHOST: f32 = 0.04;
pub(crate) const HINT: f32 = 0.45;
pub(crate) const MUTED_OP: f32 = 0.65;

/// Right-side command/meta opacity.
pub(crate) const COMMAND_OPACITY: f32 = 0.30;

/// Picker rows should align with the footer hint strip they sit above.
pub(crate) const CONTEXT_PICKER_ROW_HEIGHT: f32 = HINT_STRIP_HEIGHT;
pub(crate) const CONTEXT_PICKER_SYNOPSIS_HEIGHT: f32 = 64.0;

/// Render a single dense-monoline picker row.
///
/// Shared by ACP popups, inline context pickers, model selectors, and any
/// future smart-input dropdown that wants the same footer-aligned chrome.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_dense_monoline_picker_row(
    id: SharedString,
    label: SharedString,
    meta: SharedString,
    label_highlight_indices: &[usize],
    meta_highlight_indices: &[usize],
    is_selected: bool,
    foreground: Hsla,
    muted_foreground: Hsla,
) -> gpui::Stateful<gpui::Div> {
    render_dense_monoline_picker_row_full(
        id,
        label,
        meta,
        label_highlight_indices,
        meta_highlight_indices,
        is_selected,
        foreground,
        muted_foreground,
        None,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_dense_monoline_picker_row_with_accessory(
    id: SharedString,
    label: SharedString,
    meta: SharedString,
    label_highlight_indices: &[usize],
    meta_highlight_indices: &[usize],
    is_selected: bool,
    foreground: Hsla,
    muted_foreground: Hsla,
    accessory: Option<AnyElement>,
) -> gpui::Stateful<gpui::Div> {
    render_dense_monoline_picker_row_full(
        id,
        label,
        meta,
        label_highlight_indices,
        meta_highlight_indices,
        is_selected,
        foreground,
        muted_foreground,
        None,
        accessory,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_dense_monoline_picker_row_with_leading_visual(
    id: SharedString,
    label: SharedString,
    meta: SharedString,
    label_highlight_indices: &[usize],
    meta_highlight_indices: &[usize],
    is_selected: bool,
    foreground: Hsla,
    muted_foreground: Hsla,
    leading_visual: AnyElement,
) -> gpui::Stateful<gpui::Div> {
    render_dense_monoline_picker_row_full(
        id,
        label,
        meta,
        label_highlight_indices,
        meta_highlight_indices,
        is_selected,
        foreground,
        muted_foreground,
        Some(leading_visual),
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn render_dense_monoline_picker_row_full(
    id: SharedString,
    label: SharedString,
    meta: SharedString,
    label_highlight_indices: &[usize],
    meta_highlight_indices: &[usize],
    is_selected: bool,
    foreground: Hsla,
    muted_foreground: Hsla,
    leading_visual: Option<AnyElement>,
    accessory: Option<AnyElement>,
) -> gpui::Stateful<gpui::Div> {
    let label_hits: HashSet<usize> = label_highlight_indices.iter().copied().collect();
    let meta_hits: HashSet<usize> = meta_highlight_indices.iter().copied().collect();

    let show_meta = accessory.is_none() && !meta.is_empty();

    let mut left = div().flex().items_center().gap(px(6.0)).child(
        div()
            .w(px(2.0))
            .h(px(12.0))
            .rounded(px(1.0))
            .bg(if is_selected {
                GOLD
            } else {
                gpui::transparent_black()
            }),
    );

    if let Some(leading_visual) = leading_visual {
        left = left.child(leading_visual);
    }

    left = left.child(render_highlighted_text(
        &label,
        &label_hits,
        if is_selected {
            foreground
        } else {
            foreground.opacity(MUTED_OP)
        },
        GOLD,
    ));

    let mut row = div()
        .id(id)
        .h(px(CONTEXT_PICKER_ROW_HEIGHT))
        .flex()
        .items_center()
        .justify_between()
        .px(px(3.0))
        .py(px(3.0))
        .bg(if is_selected {
            foreground.opacity(GHOST)
        } else {
            gpui::transparent_black()
        })
        .hover(|el| el.bg(foreground.opacity(GHOST)))
        .child(left)
        .when(show_meta, |d| {
            d.child(render_highlighted_meta(
                &meta,
                &meta_hits,
                muted_foreground.opacity(COMMAND_OPACITY),
                GOLD.opacity(HINT),
            ))
        });

    if let Some(accessory) = accessory {
        row = row.child(accessory);
    }

    row
}

/// Render label text with gold highlights on matched characters (text_xs).
pub(crate) fn render_highlighted_text(
    text: &str,
    hits: &HashSet<usize>,
    base: Hsla,
    accent: Hsla,
) -> AnyElement {
    if hits.is_empty() {
        return div()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(base)
            .text_ellipsis()
            .child(SharedString::from(text.to_string()))
            .into_any_element();
    }

    let mut spans: Vec<AnyElement> = Vec::new();
    let mut current = String::new();
    let mut current_highlighted = false;

    for (ix, ch) in text.chars().enumerate() {
        let is_hit = hits.contains(&ix);
        if ix > 0 && is_hit != current_highlighted {
            spans.push(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(if current_highlighted { accent } else { base })
                    .child(SharedString::from(std::mem::take(&mut current)))
                    .into_any_element(),
            );
        }
        current_highlighted = is_hit;
        current.push(ch);
    }
    if !current.is_empty() {
        spans.push(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(if current_highlighted { accent } else { base })
                .child(SharedString::from(current))
                .into_any_element(),
        );
    }

    div()
        .flex()
        .items_center()
        .text_ellipsis()
        .children(spans)
        .into_any_element()
}

/// Render right-side meta text in FONT_MONO with optional highlights.
pub(crate) fn render_highlighted_meta(
    text: &str,
    hits: &HashSet<usize>,
    base: Hsla,
    accent: Hsla,
) -> AnyElement {
    if hits.is_empty() {
        return div()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .font_family(FONT_MONO)
            .text_color(base)
            .text_ellipsis()
            .child(SharedString::from(text.to_string()))
            .into_any_element();
    }

    let mut spans: Vec<AnyElement> = Vec::new();
    let mut current = String::new();
    let mut current_highlighted = false;

    for (ix, ch) in text.chars().enumerate() {
        let is_hit = hits.contains(&ix);
        if ix > 0 && is_hit != current_highlighted {
            spans.push(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .font_family(FONT_MONO)
                    .text_color(if current_highlighted { accent } else { base })
                    .child(SharedString::from(std::mem::take(&mut current)))
                    .into_any_element(),
            );
        }
        current_highlighted = is_hit;
        current.push(ch);
    }
    if !current.is_empty() {
        spans.push(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .font_family(FONT_MONO)
                .text_color(if current_highlighted { accent } else { base })
                .child(SharedString::from(current))
                .into_any_element(),
        );
    }

    div()
        .flex()
        .items_center()
        .text_ellipsis()
        .children(spans)
        .into_any_element()
}

/// Compact bottom synopsis for the focused picker item.
pub(crate) fn render_compact_synopsis_strip(
    label: SharedString,
    meta: SharedString,
    description: SharedString,
    foreground: Hsla,
    muted_foreground: Hsla,
) -> AnyElement {
    div()
        .px(px(8.0))
        .py(px(5.0))
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .text_xs()
                        .text_color(muted_foreground.opacity(HINT))
                        .child(label),
                )
                .when(!meta.is_empty(), |d| {
                    d.child(
                        div()
                            .text_xs()
                            .font_family(FONT_MONO)
                            .text_color(muted_foreground.opacity(COMMAND_OPACITY))
                            .child(meta),
                    )
                }),
        )
        .child(
            div()
                .text_xs()
                .text_color(foreground.opacity(MUTED_OP))
                .child(description),
        )
        .into_any_element()
}
