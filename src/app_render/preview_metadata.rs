impl ScriptListApp {
    fn render_actions_dialog(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();
        let ui_border = self.theme.colors.ui.border;

        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                this.hide_mouse_cursor(cx);
                let _ = this.handle_global_shortcut_with_options(event, true, cx);
            },
        );

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .p(px(design_spacing.padding_xl))
            .gap(px(design_spacing.gap_md))
            .text_color(rgb(design_colors.text_primary))
            .font_family(design_typography.font_family)
            .key_context("actions_dialog")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("Actions"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(design_colors.text_muted))
                    .child("Quick actions for Script Kit"),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(design_spacing.gap_sm))
                    .children(
                        actions_dialog_items::ACTIONS_DIALOG_ITEMS.iter().map(|item| {
                            div()
                                .w_full()
                                .px(px(design_spacing.padding_md))
                                .py(px(design_spacing.padding_sm))
                                .rounded(px(design_visual.radius_md))
                                .border_1()
                                .border_color(rgba((ui_border << 8) | 0x24))
                                .bg(rgba((design_colors.background << 8) | 0x18))
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap(px(design_spacing.gap_sm))
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .child(item.title.to_string()),
                                        )
                                        .child(
                                            div()
                                                .ml_auto()
                                                .text_xs()
                                                .text_color(rgb(design_colors.text_dimmed))
                                                .child(item.id.to_string()),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(design_colors.text_muted))
                                        .child(item.description.to_string()),
                                )
                        }),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(design_colors.text_dimmed))
                    .child("Press Esc to close"),
            )
            .into_any_element()
    }
}
