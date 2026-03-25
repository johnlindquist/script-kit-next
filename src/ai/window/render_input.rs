use super::*;

impl AiApp {
    pub(super) fn render_input_with_cursor(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        // Thin wrapper only for max_h constraint (caps multi-line growth).
        // No padding — the composer surface owns all spacing.
        div().flex_1().max_h(COMPOSER_MAX_H).child(
            Input::new(&self.input_state)
                .w_full()
                .appearance(false) // No default styling - we provide our own
                .bordered(false)
                .focus_bordered(false)
                // Override Medium's default 8px vertical padding to 2px so the
                // single-line Input height (~24px) matches the attachment button,
                // giving items_center() a clean alignment target.
                .pt(SP_1)
                .pb(SP_1),
        )
    }

    /// Render the model picker button
    /// Clicking cycles to the next model; shows current model name.
    pub(super) fn render_model_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let button_colors =
            crate::components::ButtonColors::from_theme(&crate::theme::get_cached_theme());
        let entity = cx.entity();

        if self.available_models.is_empty() {
            let show_copied = self.is_showing_copied_feedback();

            // No models available - show actionable setup hint
            let setup_entity = entity.clone();
            return crate::components::Button::new(
                if show_copied {
                    "Copied!"
                } else {
                    "Setup Required"
                },
                button_colors,
            )
            .id("setup-hint")
            .variant(crate::components::ButtonVariant::Ghost)
            .shortcut_opt((!show_copied).then_some("↵".to_string()))
            .on_click(Box::new(move |_, window, cx| {
                setup_entity.update(cx, |this, cx| {
                    this.copy_setup_command(cx);
                    window.activate_window();
                });
            }))
            .into_any_element();
        }

        // Get current model display name, with provider suffix when available
        let model_label: SharedString = self
            .selected_model
            .as_ref()
            .map(|m| {
                let provider_name = self
                    .provider_registry
                    .get_provider(&m.provider)
                    .map(|p| p.display_name().to_string());
                if let Some(ref name) = provider_name {
                    if !name.is_empty() {
                        return format!("{} · {}", m.display_name, name);
                    }
                }
                m.display_name.clone()
            })
            .unwrap_or_else(|| "Select Model".to_string())
            .into();

        let cycle_entity = entity.clone();
        crate::components::Button::new(model_label, button_colors)
            .id("model-display")
            .variant(crate::components::ButtonVariant::Ghost)
            .on_click(Box::new(move |_, _window, cx| {
                cycle_entity.update(cx, |this, cx| {
                    this.cycle_model(cx);
                });
            }))
            .into_any_element()
    }

    /// Render a compact model chip for the mini titlebar.
    /// Matches full mode's button + fallback pattern: click cycles to the
    /// next available model (same as `render_model_picker`). Does NOT open
    /// the generic command bar — model selection is direct.
    pub(super) fn render_mini_model_chip(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let button_colors =
            crate::components::ButtonColors::from_theme(&crate::theme::get_cached_theme());
        let entity = cx.entity();

        if self.available_models.is_empty() {
            let show_copied = self.is_showing_copied_feedback();
            let setup_entity = entity.clone();
            return crate::components::Button::new(
                if show_copied {
                    "Copied!"
                } else {
                    "Setup Required"
                },
                button_colors,
            )
            .id("ai-mini-model-setup")
            .variant(crate::components::ButtonVariant::Ghost)
            .on_click(Box::new(move |_, window, cx| {
                setup_entity.update(cx, |this, cx| {
                    this.copy_setup_command(cx);
                    window.activate_window();
                });
            }))
            .into_any_element();
        }

        // Compact label: model name + provider, same as full mode
        let model_label: SharedString = self
            .selected_model
            .as_ref()
            .map(|m| {
                let provider_name = self
                    .provider_registry
                    .get_provider(&m.provider)
                    .map(|p| p.display_name().to_string());
                if let Some(ref name) = provider_name {
                    if !name.is_empty() {
                        return format!("{} · {}", m.display_name, name);
                    }
                }
                m.display_name.clone()
            })
            .unwrap_or_else(|| "Select Model".to_string())
            .into();

        let click_entity = entity.clone();
        crate::components::Button::new(model_label, button_colors)
            .id("ai-mini-model-chip")
            .variant(crate::components::ButtonVariant::Ghost)
            .on_click(Box::new(move |_, _window, cx| {
                click_entity.update(cx, |this, cx| {
                    tracing::info!(
                        target: "ai",
                        category = "AI_UI",
                        event = "mini_model_chip_cycle",
                        window_mode = ?this.window_mode,
                        "Mini model chip clicked — cycling model"
                    );
                    this.cycle_model(cx);
                });
            }))
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
