use super::*;

impl AiApp {
    pub(super) fn render_input_with_cursor(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        // Keep this focused on text alignment; the outer composer surface owns border/radius.
        div()
            .flex_1()
            .h(COMPOSER_H)
            .pl(S3)
            .rounded(R_MD)
            .flex()
            .items_center()
            .child(
                Input::new(&self.input_state)
                    .w_full()
                    .appearance(false) // No default styling - we provide our own
                    .bordered(false)
                    .focus_bordered(false),
            )
    }

    /// Render the model picker button
    /// Clicking cycles to the next model; shows current model name
    pub(super) fn render_model_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.available_models.is_empty() {
            let show_copied = self.is_showing_copied_feedback();

            // No models available - show actionable setup hint
            return div()
                .id("setup-hint")
                .flex()
                .items_center()
                .gap(S2)
                .px(S2)
                .py(S1)
                .rounded(R_SM)
                .cursor_pointer()
                .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.copy_setup_command(cx);
                    window.activate_window();
                }))
                .child(if show_copied {
                    Icon::new(IconName::Check)
                        .size(ICON_XS)
                        .text_color(cx.theme().success)
                        .into_any_element()
                } else {
                    Icon::new(IconName::TriangleAlert)
                        .size(ICON_XS)
                        .text_color(cx.theme().warning)
                        .into_any_element()
                })
                .child(
                    div()
                        .text_xs()
                        .text_color(if show_copied {
                            cx.theme().success
                        } else {
                            cx.theme().muted_foreground
                        })
                        .child(if show_copied {
                            "Copied!"
                        } else {
                            "Setup Required"
                        }),
                )
                .when(!show_copied, |d| {
                    d.child(
                        div()
                            .px(S1)
                            .py(S0)
                            .rounded(RADIUS_SM)
                            .bg(cx.theme().muted)
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("↵"),
                    )
                })
                .into_any_element();
        }

        // Get current model display name
        let model_label: SharedString = self
            .selected_model
            .as_ref()
            .map(|m| m.display_name.clone())
            .unwrap_or_else(|| "Select Model".to_string())
            .into();

        // Model display (read-only) - model selection now available via Actions (Cmd+K)
        div()
            .id("model-display")
            .flex()
            .items_center()
            .gap(S1)
            .px(S2)
            .py(S1)
            .rounded(R_SM)
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(model_label)
            .into_any_element()
    }

    /// Cycle to the next model in the list
    pub(super) fn cycle_model(&mut self, cx: &mut Context<Self>) {
        if self.available_models.is_empty() {
            return;
        }

        // Find current index
        let current_idx = self
            .selected_model
            .as_ref()
            .and_then(|sm| self.available_models.iter().position(|m| m.id == sm.id))
            .unwrap_or(0);

        // Cycle to next
        let next_idx = (current_idx + 1) % self.available_models.len();
        self.on_model_change(next_idx, cx);
    }
}
