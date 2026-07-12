use super::*;

impl NotesApp {
    pub(super) fn render_titlebar_trash_actions(
        &self,
        has_selection: bool,
        is_trash: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .w(px(TITLEBAR_ICONS_W))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_end()
            .gap_1()
            .when(has_selection && is_trash, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            Button::new("restore")
                                .ghost()
                                .xsmall()
                                .label("Restore (⌘Z)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.restore_note(window, cx);
                                })),
                        )
                        .child(
                            Button::new("permanent-delete")
                                .ghost()
                                .xsmall()
                                .icon(IconName::Delete)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.permanently_delete_note(window, cx);
                                })),
                        ),
                )
            })
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_editor_titlebar(
        &self,
        title: String,
        window_hovered: bool,
        has_selection: bool,
        is_trash: bool,
        _is_preview: bool,
        is_pinned: bool,
        in_focus_mode: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let muted_color = cx.theme().muted_foreground;
        let accent_color = cx.theme().accent;
        let metrics = style::adopted_metrics();

        let titlebar_actions = self.render_titlebar_trash_actions(has_selection, is_trash, cx);

        div()
            .id("notes-titlebar")
            .flex()
            .items_center()
            .h(px(metrics.titlebar_height))
            // Contract-owned horizontal padding (was an inline `.px_3()`);
            // the design-contract exporter reads the same const.
            .px(px(super::contract::NOTES_TITLEBAR_PADDING_X))
            .when(is_trash, |d| {
                d.border_b_1()
                    .border_color(cx.theme().danger.opacity(OPACITY_ACCENT_BORDER))
            })
            .on_hover(cx.listener(|this, hovered, _, cx| {
                if this.force_hovered {
                    return;
                }
                this.titlebar_hovered = *hovered;
                cx.notify();
            }))
            .child(div().w(px(TITLEBAR_TRAFFIC_LIGHT_W)).flex_shrink_0())
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_sm()
                    .text_color(muted_color)
                    .when(!window_hovered, |d| d.opacity(OPACITY_MUTED))
                    .when(window_hovered, |d| d.opacity(1.0))
                    .when(in_focus_mode, |d| d.opacity(0.))
                    .when(is_pinned && !in_focus_mode, |d| {
                        d.child(div().text_xs().text_color(accent_color).child("●"))
                    })
                    .child(title)
                    .when(in_focus_mode && window_hovered, |d| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(muted_color.opacity(OPACITY_DISABLED))
                                .child("esc  or  ⌘.  exit focus"),
                        )
                    }),
            )
            .child(titlebar_actions)
            .into_any_element()
    }
}
