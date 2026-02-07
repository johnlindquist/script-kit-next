        // ── Footer with keyboard shortcuts ─────────────────────────
        let shortcut = |key: &str, label: &str| {
            div()
                .flex()
                .flex_row()
                .gap(px(4.0))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_secondary))
                        .child(key.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_dimmed))
                        .child(label.to_string()),
                )
        };
        let footer_border = rgba((ui_border << 8) | 0x30);
        let footer = div()
            .w_full()
            .px(px(design_spacing.padding_lg))
            .py(px(design_spacing.padding_sm))
            .border_t_1()
            .border_color(footer_border)
            .flex()
            .flex_col()
            .gap(px(2.0))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_center()
                    .gap(px(12.0))
                    .child(shortcut("↑↓", "Preview"))
                    .child(shortcut("Enter", "Apply"))
                    .child(shortcut("Esc", "Cancel"))
                    .child(shortcut("PgUp/Dn", "Jump"))
                    .child(shortcut("Type", "Search")),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_center()
                    .gap(px(12.0))
                    .child(shortcut("⌘[]", "Accent"))
                    .child(shortcut("⌘-/=", "Opacity"))
                    .child(shortcut("⌘B", "Vibrancy"))
                    .child(shortcut("⌘M", "Material"))
                    .child(shortcut("⌘R", "Reset")),
            );

        // ── Empty state when filter has no matches ─────────────────
        if filtered_count == 0 {
            return div()
                .flex()
                .flex_col()
                .w_full()
                .h_full()
                .rounded(px(design_visual.radius_lg))
                .text_color(rgb(text_primary))
                .font_family(design_typography.font_family)
                .key_context("theme_chooser")
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key)
                .child(header)
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_muted))
                                .child("No matching themes"),
                        ),
                )
                .child(footer)
                .into_any_element();
        }

        // ── Main layout: list + preview panel ──────────────────────
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("theme_chooser")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(header)
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .flex()
                    .flex_row()
                    .child(div().w_1_2().h_full().child(list))
                    .child(preview_panel),
            )
            .child(footer)
            .into_any_element()
