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

    /// Execute a window action (tile, maximize, minimize, close)
    /// NOTE: Currently unused - kept for future when we add action buttons to the actions panel
    #[allow(dead_code)]
    fn execute_window_action(&mut self, window_id: u32, action: &str, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!("Window action: {} on window {}", action, window_id),
        );

        let result = match action {
            "tile_left" => {
                window_control::tile_window(window_id, window_control::TilePosition::LeftHalf)
            }
            "tile_right" => {
                window_control::tile_window(window_id, window_control::TilePosition::RightHalf)
            }
            "tile_top" => {
                window_control::tile_window(window_id, window_control::TilePosition::TopHalf)
            }
            "tile_bottom" => {
                window_control::tile_window(window_id, window_control::TilePosition::BottomHalf)
            }
            "maximize" => window_control::maximize_window(window_id),
            "minimize" => window_control::minimize_window(window_id),
            "close" => window_control::close_window(window_id),
            "focus" => window_control::focus_window(window_id),
            _ => {
                logging::log("ERROR", &format!("Unknown window action: {}", action));
                return;
            }
        };

        match result {
            Ok(()) => {
                logging::log("EXEC", &format!("Window action {} succeeded", action));

                // Show success toast
                self.toast_manager.push(
                    components::toast::Toast::success(
                        format!("Window {}", action.replace("_", " ")),
                        &self.theme,
                    )
                    .duration_ms(Some(2000)),
                );

                // P0 FIX: Refresh window list in self.cached_windows
                if let AppView::WindowSwitcherView { selected_index, .. } = &mut self.current_view {
                    match window_control::list_windows() {
                        Ok(new_windows) => {
                            self.cached_windows = new_windows;
                            // Adjust selected index if needed
                            if *selected_index >= self.cached_windows.len()
                                && !self.cached_windows.is_empty()
                            {
                                *selected_index = self.cached_windows.len() - 1;
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to refresh windows: {}", e));
                        }
                    }
                }
            }
            Err(e) => {
                logging::log("ERROR", &format!("Window action {} failed: {}", action, e));

                // Show error toast
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to {}: {}", action.replace("_", " "), e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
            }
        }

        cx.notify();
    }
}
