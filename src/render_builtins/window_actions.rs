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

        let mut panel = div()
            .w_full()
            .h_full()
            .bg(rgb(bg_main))
            .border_l_1()
            .border_color(rgba((ui_border << 8) | 0x80))
            .p(px(spacing.padding_lg))
            .flex()
            .flex_col()
            .overflow_y_hidden()
            .font_family(typography.font_family);

        match selected_window {
            Some(window) => {
                // Window info header
                panel = panel.child(
                    div()
                        .text_lg()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(text_primary))
                        .pb(px(spacing.padding_sm))
                        .child(window.title.clone()),
                );

                // App name
                panel = panel.child(
                    div()
                        .text_sm()
                        .text_color(rgb(text_secondary))
                        .pb(px(spacing.padding_md))
                        .child(window.app.clone()),
                );

                // Bounds info
                panel = panel.child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .pb(px(spacing.padding_lg))
                        .child(format!(
                            "{}×{} at ({}, {})",
                            window.bounds.width,
                            window.bounds.height,
                            window.bounds.x,
                            window.bounds.y
                        )),
                );

                // Divider
                panel = panel.child(
                    div()
                        .w_full()
                        .h(px(visual.border_thin))
                        .bg(rgba((ui_border << 8) | 0x60))
                        .mb(px(spacing.padding_lg)),
                );

                // Actions header
                panel = panel.child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .pb(px(spacing.padding_md))
                        .child("Press Enter to focus window"),
                );
            }
            None => {
                // Empty state
                panel = panel.child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(text_muted))
                        .child("No window selected"),
                );
            }
        }

        panel
    }

}
