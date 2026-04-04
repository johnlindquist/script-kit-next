use super::super::*;
use super::types::ContextPickerItemKind;
use crate::list_item::FONT_MONO;
use std::collections::HashSet;

/// Gold accent (#fbbf24) — the one warm signature touch.
const GOLD: gpui::Hsla = gpui::Hsla {
    h: 0.1194,
    s: 0.956,
    l: 0.565,
    a: 1.0,
};

/// Impeccable opacity tiers.
const GHOST: f32 = 0.04;
const HINT: f32 = 0.45;
const MUTED_OP: f32 = 0.65;

/// V05 right-side command opacity.
const COMMAND_OPACITY: f32 = 0.30;

impl AiApp {
    /// Render the inline context picker overlay.
    ///
    /// V05 Dense Monoline: text_xs, 3px pad, ghost bg on selected,
    /// gold bar 2x12, FONT_MONO /command at 0.30, vibrancy shell.
    pub(in crate::ai::window) fn render_context_picker(
        &self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let state = match &self.context_picker {
            Some(s) => s,
            None => return div().id("context-picker-empty").into_any_element(),
        };

        // If no items, show empty state with hint chips
        if state.items.is_empty() {
            return self
                .render_context_picker_empty_state(state.trigger, &state.query, cx)
                .into_any_element();
        }

        let fg = cx.theme().foreground;
        let muted_fg = cx.theme().muted_foreground;

        let mut rows: Vec<gpui::AnyElement> = Vec::new();

        for (idx, item) in state.items.iter().enumerate() {
            let is_selected = idx == state.selected_index;
            let label: SharedString = item.label.clone();
            let meta: SharedString = item.meta.clone();
            let label_hits: HashSet<usize> =
                item.label_highlight_indices.iter().copied().collect();
            let meta_hits: HashSet<usize> =
                item.meta_highlight_indices.iter().copied().collect();

            rows.push(
                div()
                    .id(SharedString::from(format!("ctx-picker-{}", idx)))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(3.))
                    .py(px(3.))
                    .cursor_pointer()
                    .bg(if is_selected {
                        fg.opacity(GHOST)
                    } else {
                        gpui::transparent_black()
                    })
                    .hover(|el| el.bg(fg.opacity(GHOST)))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if let Some(picker) = this.context_picker.as_mut() {
                            picker.selected_index = idx;
                        }
                        this.accept_context_picker_selection(window, cx);
                    }))
                    // Left side: gold bar + highlighted label
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S2)
                            .child(
                                div()
                                    .w(px(2.))
                                    .h(px(12.))
                                    .rounded(px(1.))
                                    .bg(if is_selected {
                                        GOLD
                                    } else {
                                        gpui::transparent_black()
                                    }),
                            )
                            .child(render_highlighted_text(
                                &label,
                                &label_hits,
                                if is_selected { fg } else { fg.opacity(MUTED_OP) },
                                GOLD,
                            )),
                    )
                    // Right side: /command in FONT_MONO at COMMAND_OPACITY
                    .when(!meta.is_empty(), |d| {
                        d.child(render_highlighted_meta(
                            &meta,
                            &meta_hits,
                            muted_fg.opacity(COMMAND_OPACITY),
                            GOLD.opacity(HINT),
                        ))
                    })
                    .into_any_element(),
            );
        }

        div()
            .id("context-picker-overlay")
            .w_full()
            .max_h(px(260.))
            .overflow_y_scroll()
            // Near-transparent — vibrancy shows through
            .bg(fg.opacity(0.02))
            .py(SP_1)
            .children(rows)
            .into_any_element()
    }

    /// Render empty state with hint chips when no results match.
    fn render_context_picker_empty_state(
        &self,
        trigger: super::types::ContextPickerTrigger,
        query: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let fg = cx.theme().foreground;
        let muted_fg = cx.theme().muted_foreground;
        let hints = super::empty_state_hints(trigger);

        tracing::info!(
            target: "ai",
            ?trigger,
            query = %query,
            hint_count = hints.len(),
            "ai_context_picker_empty_state"
        );

        let mut chips: Vec<gpui::AnyElement> = Vec::new();
        for hint in hints {
            let hint_str = SharedString::from(*hint);
            let hint_for_click = hint_str.clone();
            chips.push(
                div()
                    .id(SharedString::from(format!("hint-{}", hint)))
                    .px(px(6.))
                    .py(px(2.))
                    .rounded(px(4.))
                    .bg(fg.opacity(GHOST))
                    .hover(|el| el.bg(fg.opacity(0.08)))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.set_composer_value(hint_for_click.to_string(), window, cx);
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_family(FONT_MONO)
                            .text_color(muted_fg.opacity(HINT))
                            .child(hint_str),
                    )
                    .into_any_element(),
            );
        }

        div()
            .id("context-picker-empty-state")
            .w_full()
            .bg(fg.opacity(0.02))
            .py(S3)
            .px(S3)
            .flex()
            .flex_col()
            .gap(S2)
            .child(
                div()
                    .text_xs()
                    .text_color(muted_fg.opacity(MUTED_OP))
                    .child("No matching context"),
            )
            .child(div().flex().items_center().gap(S2).children(chips))
            .into_any_element()
    }
}

/// Render label text with gold highlights on matched characters (V05 text_xs).
fn render_highlighted_text(
    text: &str,
    hits: &HashSet<usize>,
    base: gpui::Hsla,
    accent: gpui::Hsla,
) -> gpui::AnyElement {
    if hits.is_empty() {
        return div()
            .text_xs()
            .text_color(base)
            .text_ellipsis()
            .child(SharedString::from(text.to_string()))
            .into_any_element();
    }

    let mut spans: Vec<gpui::AnyElement> = Vec::new();
    let mut current = String::new();
    let mut current_highlighted = false;

    for (ix, ch) in text.chars().enumerate() {
        let is_hit = hits.contains(&ix);
        if ix > 0 && is_hit != current_highlighted {
            spans.push(
                div()
                    .text_xs()
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
fn render_highlighted_meta(
    text: &str,
    hits: &HashSet<usize>,
    base: gpui::Hsla,
    accent: gpui::Hsla,
) -> gpui::AnyElement {
    if hits.is_empty() {
        return div()
            .text_xs()
            .font_family(FONT_MONO)
            .text_color(base)
            .text_ellipsis()
            .child(SharedString::from(text.to_string()))
            .into_any_element();
    }

    let mut spans: Vec<gpui::AnyElement> = Vec::new();
    let mut current = String::new();
    let mut current_highlighted = false;

    for (ix, ch) in text.chars().enumerate() {
        let is_hit = hits.contains(&ix);
        if ix > 0 && is_hit != current_highlighted {
            spans.push(
                div()
                    .text_xs()
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
