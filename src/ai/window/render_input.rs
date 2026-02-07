use super::*;

impl AiApp {
    pub(super) fn render_input_with_cursor(
        &self,
        border_color: gpui::Hsla,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let input_bg = cx.theme().muted.opacity(0.4);

        // Make border semi-transparent for vibrancy (50% opacity for better contrast)
        let transparent_border = border_color.opacity(0.5);

        // Wrap input in a styled container for vibrancy support
        // No px padding - let Input component handle text positioning
        div()
            .flex_1()
            .h(px(36.))
            .pl_2() // Small left padding for visual alignment with border
            .rounded(px(10.))
            .border_1()
            .border_color(transparent_border) // Semi-transparent accent border
            .bg(input_bg) // Vibrancy-compatible semi-transparent background
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
                .gap_2()
                .px_2()
                .py(px(2.))
                .rounded_md()
                .cursor_pointer()
                .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.copy_setup_command(cx);
                    window.activate_window();
                }))
                .child(if show_copied {
                    Icon::new(IconName::Check)
                        .size(px(12.))
                        .text_color(cx.theme().success)
                        .into_any_element()
                } else {
                    Icon::new(IconName::TriangleAlert)
                        .size(px(12.))
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
                            .px(px(4.))
                            .py(px(1.))
                            .rounded(px(3.))
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
            .gap_1()
            .px_2()
            .py(px(2.))
            .rounded_md()
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
