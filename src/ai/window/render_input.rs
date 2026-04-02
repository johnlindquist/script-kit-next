use super::*;
use crate::theme::opacity::{OPACITY_STRONG, OPACITY_TEXT_MUTED};

// -- Deterministic model-cycle planning (pure, no side effects) --

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ModelCyclePlan {
    next_index: usize,
    wrapped: bool,
}

/// Compute the next model index given the current selection.
///
/// - `None` current → start at 0 (no wrap).
/// - `Some(last)` → advance by one, wrapping to 0 when past the end.
/// - Empty list → `None` (caller should bail).
fn plan_model_cycle(current_index: Option<usize>, available_len: usize) -> Option<ModelCyclePlan> {
    if available_len == 0 {
        return None;
    }

    let (next_index, wrapped) = match current_index {
        Some(idx) if idx + 1 < available_len => (idx + 1, false),
        Some(_) => (0, true),
        None => (0, false),
    };

    Some(ModelCyclePlan {
        next_index,
        wrapped,
    })
}

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
                // Use Large size to match main menu input font (16px / text_base).
                .with_size(gpui_component::Size::Large)
                // Override Large's default 10px vertical padding to 2px so the
                // single-line Input height matches the attachment button,
                // giving items_center() a clean alignment target.
                .pt(SP_1)
                .pb(SP_1),
        )
    }

    /// Shared model-button label: "Model · Provider" or "Select Model".
    fn current_model_button_label(&self) -> SharedString {
        self.selected_model
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
            .into()
    }

    /// Canonical model-cycle with structured instrumentation.
    ///
    /// Both `render_model_picker` (full) and `render_mini_model_chip` (mini)
    /// route through this single function so the off-by-one bug for
    /// `selected_model == None` is fixed once, and every cycle emits a
    /// machine-readable event payload.
    pub(super) fn cycle_model_from_source(&mut self, source: &'static str, cx: &mut Context<Self>) {
        let had_selected_model = self.selected_model.is_some();
        let current_index = self
            .selected_model
            .as_ref()
            .and_then(|sm| self.available_models.iter().position(|m| m.id == sm.id));
        let selected_model_stale = had_selected_model && current_index.is_none();

        let Some(plan) = plan_model_cycle(current_index, self.available_models.len()) else {
            tracing::warn!(
                target: "ai",
                category = "AI_UI",
                event = "model_cycle_no_available_models",
                source,
                window_mode = ?self.window_mode,
                "Model cycle requested with no available models"
            );
            return;
        };

        let previous_model_id = current_index
            .and_then(|idx| self.available_models.get(idx))
            .map(|m| m.id.clone());
        let previous_provider = current_index
            .and_then(|idx| self.available_models.get(idx))
            .map(|m| m.provider.clone());

        let next_model = &self.available_models[plan.next_index];
        let next_model_id = next_model.id.clone();
        let next_provider = next_model.provider.clone();
        let next_display_name = next_model.display_name.clone();

        tracing::info!(
            target: "ai",
            category = "AI_UI",
            event = "model_cycle_requested",
            source,
            window_mode = ?self.window_mode,
            available_model_count = self.available_models.len(),
            started_unselected = !had_selected_model,
            selected_model_stale,
            wrapped = plan.wrapped,
            previous_model_id = previous_model_id.as_deref().unwrap_or(""),
            previous_provider = previous_provider.as_deref().unwrap_or(""),
            next_model_id = %next_model_id,
            next_provider = %next_provider,
            next_display_name = %next_display_name,
            "Cycling AI model"
        );

        super::observability::emit_ai_ui_event(
            &super::observability::AiUiEvent {
                kind: super::types::AiUiEventKind::CommandLifecycle,
                action: "model_cycle",
                source,
                window_mode: self.window_mode,
                selected_chat_id: self.selected_chat_id.as_ref(),
                overlay_visible: self.showing_mini_history_overlay,
                search_active: !self.search_query.is_empty(),
            },
            Some(serde_json::json!({
                "available_model_count": self.available_models.len(),
                "started_unselected": !had_selected_model,
                "selected_model_stale": selected_model_stale,
                "wrapped": plan.wrapped,
                "previous_model_id": previous_model_id,
                "previous_provider": previous_provider,
                "next_model_id": next_model_id,
                "next_provider": next_provider,
                "next_display_name": next_display_name,
            })),
        );

        self.on_model_change(plan.next_index, cx);
    }

    /// Convenience wrapper — delegates to `cycle_model_from_source`.
    pub(super) fn cycle_model(&mut self, cx: &mut Context<Self>) {
        self.cycle_model_from_source("cycle_model", cx);
    }

    /// Render the model picker button (full mode).
    /// Clicking cycles to the next model; shows current model name.
    pub(super) fn render_model_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
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

        let model_label = self.current_model_button_label();
        let cycle_entity = entity.clone();
        crate::components::Button::new(model_label, button_colors)
            .id("model-display")
            .variant(crate::components::ButtonVariant::Ghost)
            .on_click(Box::new(move |_, _window, cx| {
                cycle_entity.update(cx, |this, cx| {
                    this.cycle_model_from_source("full_model_picker_click", cx);
                });
            }))
            .into_any_element()
    }

    /// Render a Whisper-style plain-text model affordance for the mini titlebar.
    ///
    /// - No-model state: shows "Setup Required" / "Copied!" in accent color,
    ///   clicking copies the setup command.
    /// - With models: shows plain model name, clicking cycles to the next model.
    /// - Hover strengthens opacity (`OPACITY_STRONG`) for discoverability.
    pub(super) fn render_mini_model_chip(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let muted_fg = cx.theme().muted_foreground;
        let fg = cx.theme().foreground;
        let accent = cx.theme().accent;

        if self.available_models.is_empty() {
            let label: SharedString = if self.is_showing_copied_feedback() {
                "Copied!".into()
            } else {
                "Setup Required".into()
            };
            return div()
                .id("ai-mini-model-text")
                .text_xs()
                .cursor_pointer()
                .text_color(accent.opacity(OPACITY_TEXT_MUTED))
                .hover(move |el| el.text_color(accent.opacity(OPACITY_STRONG)))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.copy_setup_command(cx);
                    window.activate_window();
                }))
                .child(label)
                .into_any_element();
        }

        let model_label: SharedString = self
            .selected_model
            .as_ref()
            .map(|m| SharedString::from(m.display_name.clone()))
            .unwrap_or_else(|| "Select Model".into());

        div()
            .id("ai-mini-model-text")
            .text_xs()
            .cursor_pointer()
            .text_color(muted_fg.opacity(OPACITY_TEXT_MUTED))
            .hover(move |el| el.text_color(fg.opacity(OPACITY_STRONG)))
            .on_click(cx.listener(|this, _, _window, cx| {
                this.cycle_model_from_source("mini_model_chip_click", cx);
            }))
            .child(model_label)
            .into_any_element()
    }
}

#[cfg(test)]
mod model_cycle_plan_tests {
    use super::{plan_model_cycle, ModelCyclePlan};

    #[test]
    fn selects_first_model_when_none_is_selected() {
        assert_eq!(
            plan_model_cycle(None, 3),
            Some(ModelCyclePlan {
                next_index: 0,
                wrapped: false,
            })
        );
    }

    #[test]
    fn advances_to_next_model_when_selection_exists() {
        assert_eq!(
            plan_model_cycle(Some(0), 3),
            Some(ModelCyclePlan {
                next_index: 1,
                wrapped: false,
            })
        );
    }

    #[test]
    fn wraps_to_zero_from_last_model() {
        assert_eq!(
            plan_model_cycle(Some(2), 3),
            Some(ModelCyclePlan {
                next_index: 0,
                wrapped: true,
            })
        );
    }

    #[test]
    fn returns_none_when_no_models_exist() {
        assert_eq!(plan_model_cycle(None, 0), None);
        assert_eq!(plan_model_cycle(Some(0), 0), None);
    }

    #[test]
    fn single_model_wraps_immediately() {
        assert_eq!(
            plan_model_cycle(Some(0), 1),
            Some(ModelCyclePlan {
                next_index: 0,
                wrapped: true,
            })
        );
    }
}
