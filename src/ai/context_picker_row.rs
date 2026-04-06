//! Shared dense-monoline picker row for both ACP @-mention and AI window
//! context picker surfaces.
//!
//! Both surfaces render footer-aligned rows with a 2px gold accent bar on
//! selection, left-aligned label text with fuzzy-match highlights, and
//! right-aligned monospace meta text.

use gpui::{
    div, prelude::FluentBuilder, px, AnyElement, FontWeight, Hsla, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled,
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

/// V05 right-side command opacity.
pub(crate) const COMMAND_OPACITY: f32 = 0.30;

/// Picker rows should align with the footer hint strip they sit above.
pub(crate) const CONTEXT_PICKER_ROW_HEIGHT: f32 = HINT_STRIP_HEIGHT;

/// Render a single dense-monoline picker row.
///
/// Used by both the AI window context picker and the ACP @-mention picker
/// to ensure identical row chrome: footer-aligned height, 2px gold bar,
/// and fuzzy highlights in both label and meta text.
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
    render_dense_monoline_picker_row_with_accessory(
        id,
        label,
        meta,
        label_highlight_indices,
        meta_highlight_indices,
        is_selected,
        foreground,
        muted_foreground,
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
    let label_hits: HashSet<usize> = label_highlight_indices.iter().copied().collect();
    let meta_hits: HashSet<usize> = meta_highlight_indices.iter().copied().collect();

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
                        .bg(if is_selected {
                            GOLD
                        } else {
                            gpui::transparent_black()
                        }),
                )
                .child(render_highlighted_text(
                    &label,
                    &label_hits,
                    if is_selected {
                        foreground
                    } else {
                        foreground.opacity(MUTED_OP)
                    },
                    GOLD,
                )),
        )
        .when(accessory.is_none() && !meta.is_empty(), |d| {
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
