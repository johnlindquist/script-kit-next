use super::*;

impl NotesApp {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_titlebar_icons(
        &self,
        window_hovered: bool,
        has_selection: bool,
        is_trash: bool,
        in_focus_mode: bool,
        preview_label: String,
        preview_color: gpui::Hsla,
        muted_color: gpui::Hsla,
        accent_color: gpui::Hsla,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .w(px(TITLEBAR_ICONS_W))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_end()
            .gap_2()
            .when(window_hovered && !is_trash && !in_focus_mode, |d| {
                d.when(has_selection, |d| {
                    d.child(
                        div()
                            .id("titlebar-cmd-icon")
                            .min_w(px(MIN_TARGET_SIZE))
                            .min_h(px(MIN_TARGET_SIZE))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .text_color(muted_color.opacity(OPACITY_MUTED))
                            .cursor_pointer()
                            .hover(|s| s.text_color(muted_color))
                            .tooltip(|window, cx| {
                                Tooltip::new("Actions")
                                    .key_binding(gpui::Keystroke::parse("cmd-k").ok().map(Kbd::new))
                                    .build(window, cx)
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                if this.show_actions_panel {
                                    this.close_actions_panel(window, cx);
                                } else {
                                    this.open_actions_panel(window, cx);
                                }
                            }))
                            .child("⌘"),
                    )
                })
                .child(
                    div()
                        .id("titlebar-browse-icon")
                        .min_w(px(MIN_TARGET_SIZE))
                        .min_h(px(MIN_TARGET_SIZE))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_sm()
                        .text_color(muted_color.opacity(OPACITY_MUTED))
                        .cursor_pointer()
                        .hover(|s| s.text_color(muted_color))
                        .tooltip(|window, cx| {
                            Tooltip::new("Note switcher")
                                .key_binding(gpui::Keystroke::parse("cmd-p").ok().map(Kbd::new))
                                .build(window, cx)
                        })
                        .on_click(cx.listener(|this, _, window, cx| {
                            if this.show_browse_panel {
                                this.close_browse_panel(window, cx);
                            } else {
                                this.close_actions_panel(window, cx);
                                this.show_browse_panel = true;
                                this.open_browse_panel(window, cx);
                            }
                        }))
                        .child("≡"),
                )
                .child(
                    div()
                        .id("titlebar-preview-icon")
                        .min_w(px(MIN_TARGET_SIZE))
                        .min_h(px(MIN_TARGET_SIZE))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_sm()
                        .text_color(preview_color)
                        .cursor_pointer()
                        .hover(|s| s.text_color(accent_color))
                        .tooltip(|window, cx| {
                            Tooltip::new("Toggle preview")
                                .key_binding(
                                    gpui::Keystroke::parse("cmd-shift-p").ok().map(Kbd::new),
                                )
                                .build(window, cx)
                        })
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.toggle_preview(window, cx);
                        }))
                        .child(preview_label.clone()),
                )
                .child(
                    div()
                        .id("titlebar-new-icon")
                        .min_w(px(MIN_TARGET_SIZE))
                        .min_h(px(MIN_TARGET_SIZE))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_sm()
                        .text_color(muted_color.opacity(OPACITY_MUTED))
                        .cursor_pointer()
                        .hover(|s| s.text_color(muted_color))
                        .tooltip(|window, cx| {
                            Tooltip::new("New note")
                                .key_binding(gpui::Keystroke::parse("cmd-n").ok().map(Kbd::new))
                                .build(window, cx)
                        })
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.create_note(window, cx);
                        }))
                        .child("+"),
                )
            })
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
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.permanently_delete_note(cx);
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
        is_preview: bool,
        is_pinned: bool,
        in_focus_mode: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let muted_color = cx.theme().muted_foreground;
        let accent_color = cx.theme().accent;
        let preview_label = if is_preview { "MD" } else { "TXT" }.to_string();
        let preview_color = if is_preview {
            accent_color
        } else {
            muted_color.opacity(OPACITY_MUTED)
        };

        let titlebar_icons = self.render_titlebar_icons(
            window_hovered,
            has_selection,
            is_trash,
            in_focus_mode,
            preview_label,
            preview_color,
            muted_color,
            accent_color,
            cx,
        );

        div()
            .id("notes-titlebar")
            .flex()
            .items_center()
            .h(px(TITLEBAR_HEIGHT))
            .px_3()
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
            .child(titlebar_icons)
            .into_any_element()
    }
}
