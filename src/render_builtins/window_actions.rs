fn format_window_bounds(
    width: impl std::fmt::Display,
    height: impl std::fmt::Display,
    x: impl std::fmt::Display,
    y: impl std::fmt::Display,
) -> gpui::SharedString {
    gpui::SharedString::from(format!("{width}\u{00d7}{height} \u{00b7} {x}, {y}"))
}

impl ScriptListApp {
    /// Render the actions panel for window switcher
    fn render_window_actions_panel(
        &self,
        selected_window: &Option<window_control::WindowInfo>,
        colors: &designs::DesignColors,
        spacing: &designs::DesignSpacing,
        typography: &designs::DesignTypography,
        visual: &designs::DesignVisual,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let bg_main = colors.background;
        let ui_border = colors.border;
        let text_primary = colors.text_primary;
        let text_muted = colors.text_muted;
        let text_secondary = colors.text_secondary;
        let opacity = self.theme.get_opacity();
        let panel_alpha = (opacity.main * 255.0 * 0.30) as u32;

        let section_label = |label: &'static str| {
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(text_muted))
                .child(label)
        };

        let meta_row = |label: &'static str, value: gpui::SharedString| {
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .child(label),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(text_secondary))
                        .child(value),
                )
        };

        let shortcut_chip = |key: &'static str, label: &'static str| {
            div()
                .px(px(8.0))
                .py(px(4.0))
                .rounded(px(visual.radius_sm))
                .bg(rgba(
                    (ui_border << 8) | u32::from(ui_foundation::ALPHA_TINT_SUBTLE),
                ))
                .border_1()
                .border_color(rgba(
                    (ui_border << 8) | u32::from(ui_foundation::ALPHA_BORDER_SUBTLE),
                ))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(6.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_secondary))
                                .child(key),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .child(label),
                        ),
                )
        };

        let mut panel = div()
            .w_full()
            .h_full()
            .bg(rgba((bg_main << 8) | panel_alpha))
            .border_l_1()
            .border_color(rgba(
                (ui_border << 8) | u32::from(ui_foundation::ALPHA_BORDER_SUBTLE),
            ))
            .p(px(spacing.padding_lg))
            .flex()
            .flex_col()
            .gap(px(spacing.gap_md))
            .overflow_y_hidden()
            .font_family(typography.font_family);

        match selected_window {
            Some(window) => {
                panel = panel
                    .child(section_label("WINDOW"))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(text_primary))
                                    .child(window.title.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(text_secondary))
                                    .child(window.app.clone()),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .h(px(visual.border_thin))
                            .bg(rgba(
                                (ui_border << 8) | u32::from(ui_foundation::ALPHA_DIVIDER),
                            )),
                    )
                    .child(section_label("DETAILS"))
                    .child(meta_row(
                        "Bounds",
                        format_window_bounds(
                            window.bounds.width,
                            window.bounds.height,
                            window.bounds.x,
                            window.bounds.y,
                        ),
                    ))
                    .child(section_label("SHORTCUTS"))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(6.0))
                            .flex_wrap()
                            .child(shortcut_chip("\u{21b5}", "Switch"))
                            .child(shortcut_chip("Esc", "Back")),
                    );
            }
            None => {
                panel = panel.child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_secondary))
                                .child("No window selected"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .child("Use \u{2191} \u{2193} to choose a window."),
                        ),
                );
            }
        }

        panel
    }
}

#[cfg(test)]
mod window_actions_tests {
    use super::format_window_bounds;

    #[test]
    fn format_window_bounds_uses_compact_preview_format() {
        let result = format_window_bounds(1440, 900, 96, 54);
        assert_eq!(result.to_string(), "1440\u{00d7}900 \u{00b7} 96, 54");
    }
}
