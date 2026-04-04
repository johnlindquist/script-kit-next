use super::super::*;
use super::types::ContextPickerItemKind;

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

impl AiApp {
    /// Render the inline context picker overlay.
    ///
    /// Vibrancy monoline style: near-transparent bg, gold bar on focus,
    /// label left + /command right in hint opacity. No icons, no border.
    pub(in crate::ai::window) fn render_context_picker(
        &self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let state = match &self.context_picker {
            Some(s) if !s.items.is_empty() => s,
            _ => return div().id("context-picker-empty").into_any_element(),
        };

        let fg = cx.theme().foreground;
        let muted_fg = cx.theme().muted_foreground;

        let mut rows: Vec<gpui::AnyElement> = Vec::new();

        for (idx, item) in state.items.iter().enumerate() {
            let is_selected = idx == state.selected_index;
            let label: SharedString = item.label.clone();
            let subtitle: SharedString = item.subtitle.clone();

            rows.push(
                div()
                    .id(SharedString::from(format!("ctx-picker-{}", idx)))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(S3)
                    .py(S1)
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
                    // Left side: gold bar + label
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S2)
                            .child(
                                div()
                                    .w(px(2.))
                                    .h(px(14.))
                                    .rounded(px(1.))
                                    .bg(if is_selected {
                                        GOLD
                                    } else {
                                        gpui::transparent_black()
                                    }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_selected {
                                        fg
                                    } else {
                                        fg.opacity(MUTED_OP)
                                    })
                                    .text_ellipsis()
                                    .child(label),
                            ),
                    )
                    // Right side: /command or subtitle in hint opacity
                    .when(!subtitle.is_empty(), |d| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(muted_fg.opacity(if is_selected {
                                    HINT
                                } else {
                                    0.3
                                }))
                                .text_ellipsis()
                                .child(subtitle),
                        )
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
}
