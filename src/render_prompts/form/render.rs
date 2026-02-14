impl ScriptListApp {
    fn render_form_prompt(
        &mut self,
        entity: Entity<FormPromptState>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let theme = render_context.theme;
        let design_colors = render_context.design_colors;
        let design_spacing = render_context.design_spacing;
        let design_typography = render_context.design_typography;
        let design_visual = render_context.design_visual;
        let actions_dialog_top = render_context.actions_dialog_top;
        let actions_dialog_right = render_context.actions_dialog_right;
        let has_actions = self.has_nonempty_sdk_actions();

        // Get prompt ID and field count from entity
        let (prompt_id, field_count, focused_field_is_textarea) = {
            let form_state = entity.read(cx);
            (
                form_state.id.clone(),
                form_state.fields.len(),
                focused_form_field_is_textarea(form_state),
            )
        };

        // Clone entity for closures
        let entity_for_submit = entity.clone();
        let entity_for_tab = entity.clone();
        let entity_for_shift_tab = entity.clone();
        let entity_for_input = entity.clone();

        let prompt_id_for_key = prompt_id.clone();
        // Key handler for form navigation (Enter/Tab/Escape) and Cmd+K actions
        let has_actions_for_handler = has_actions;
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                if handle_prompt_key_preamble_default(
                    this,
                    event,
                    window,
                    cx,
                    PromptKeyPreambleCfg {
                        is_dismissable: true,
                        stop_propagation_on_global_shortcut: false,
                        stop_propagation_when_handled: false,
                        host: ActionsDialogHost::FormPrompt,
                    },
                    has_actions_for_handler,
                    "FormPrompt",
                ) {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let has_shift = event.keystroke.modifiers.shift;
                let has_cmd = event.keystroke.modifiers.platform;

                // Handle form-level keys (Enter, Escape, Tab) at this level
                // Forward all other keys to the focused form field for text input
                let focused_field_is_textarea = {
                    let form = entity_for_submit.read(cx);
                    focused_form_field_is_textarea(form)
                };
                match form_enter_behavior(key, has_cmd, focused_field_is_textarea) {
                    FormEnterBehavior::Submit => {
                        let validation_errors = {
                            let form = entity_for_submit.read(cx);
                            collect_form_submit_validation_errors(form, cx)
                        };
                        if !validation_errors.is_empty() {
                            let message = form_submit_validation_message(&validation_errors);
                            this.show_hud(message, Some(3000), cx);
                            return;
                        }

                        logging::log("KEY", "Enter in FormPrompt - submitting form");
                        let values = entity_for_submit.read(cx).collect_values(cx);
                        this.submit_prompt_response(prompt_id_for_key.clone(), Some(values), cx);
                        return;
                    }
                    FormEnterBehavior::ForwardToField => {
                        entity_for_input.update(cx, |form, cx| {
                            form.handle_key_input(event, cx);
                        });
                        return;
                    }
                    FormEnterBehavior::Ignore => {}
                }

                // Note: "escape" is handled by handle_global_shortcut_with_options above
                if key.eq_ignore_ascii_case("tab") {
                    // Tab navigation between fields
                    if has_shift {
                        entity_for_shift_tab.update(cx, |form, cx| {
                            form.focus_previous(cx);
                        });
                    } else {
                        entity_for_tab.update(cx, |form, cx| {
                            form.focus_next(cx);
                        });
                    }
                    return;
                }

                // Forward all other keys (characters, backspace, arrows, etc.) to the focused field
                // This is necessary because GPUI requires track_focus() to receive key events,
                // and we need the parent to have focus to handle Enter/Escape/Tab.
                // The form fields' individual on_key_down handlers don't fire when parent has focus.
                entity_for_input.update(cx, |form, cx| {
                    form.handle_key_input(event, cx);
                });
            },
        );

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(theme);

        // Dynamic height based on field count
        // Base height (150px) + per-field height (60px per field)
        // Minimum of calculated height and MAX_HEIGHT (700px)
        let base_height = 150.0;
        let field_height = 60.0;
        let calculated_height = base_height + (field_count as f32 * field_height);
        let max_height = 700.0; // Same as window_resize::layout::MAX_HEIGHT
        let content_height = px(calculated_height.min(max_height));

        // Form fields have their own focus handles and on_key_down handlers.
        // We DO NOT track_focus on the container - the fields handle their own focus.
        // Enter/Escape/Tab are handled by the handle_key listener above.
        div()
            .relative() // Needed for absolute positioned actions dialog overlay
            .flex()
            .flex_col()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg)) // VIBRANCY: Only apply bg when vibrancy disabled
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
            .w_full()
            .h(content_height)
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(design_colors.text_primary))
            .font_family(design_typography.font_family)
            .key_context("form_prompt")
            .on_key_down(handle_key)
            // Content area with form fields
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.))
                    .overflow_y_scrollbar()
                    .p(px(design_spacing.padding_xl))
                    // Render the form entity (contains all fields)
                    .child(entity.clone()),
            )
            // Unified footer with PromptFooter component
            .child({
                let footer_colors = PromptFooterColors::from_theme(theme);
                let footer_config = prompt_footer_config_with_status(
                    "Continue",
                    has_actions,
                    Some(form_footer_status_text(focused_field_is_textarea)),
                    Some(format!("{field_count} fields")),
                );

                let handle_actions = cx.entity().downgrade();

                PromptFooter::new(footer_config, footer_colors).on_secondary_click(Box::new(
                    move |_, window, cx| {
                        if let Some(app) = handle_actions.upgrade() {
                            app.update(cx, |this, cx| {
                                this.toggle_arg_actions(cx, window);
                            });
                        }
                    },
                ))
            })
            // Actions dialog overlay
            .when_some(
                render_actions_backdrop(
                    self.show_actions_popup,
                    self.actions_dialog.clone(),
                    actions_dialog_top,
                    actions_dialog_right,
                    ActionsBackdropConfig {
                        backdrop_id: "form-actions-backdrop",
                        close_host: ActionsDialogHost::FormPrompt,
                        backdrop_log_message: "Form actions backdrop clicked - dismissing dialog",
                        show_pointer_cursor: true,
                    },
                    cx,
                ),
                |d, backdrop_overlay| d.child(backdrop_overlay),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod form_prompt_render_backdrop_tests {
    const FORM_RENDER_SOURCE: &str = include_str!("render.rs");

    #[test]
    fn test_form_actions_backdrop_uses_shared_helper_with_clickable_cursor() {
        assert!(
            FORM_RENDER_SOURCE.contains("render_actions_backdrop("),
            "form render should delegate backdrop overlay creation to shared helper"
        );
        assert!(
            FORM_RENDER_SOURCE.contains("\"form-actions-backdrop\""),
            "form render should pass its backdrop id to shared helper"
        );
        assert!(
            FORM_RENDER_SOURCE.contains("ActionsDialogHost::FormPrompt"),
            "form render should preserve actions host routing when helper is used"
        );
        assert!(
            FORM_RENDER_SOURCE.contains("show_pointer_cursor: true"),
            "form render should keep backdrop cursor pointer enabled"
        );
    }

    #[test]
    fn test_form_key_handler_uses_shared_preamble_helper() {
        assert!(
            FORM_RENDER_SOURCE.contains("handle_prompt_key_preamble("),
            "form key handling should delegate preamble logic to shared helper"
        );
        assert!(
            FORM_RENDER_SOURCE.contains("PromptKeyPreambleCfg"),
            "form key handling should configure the shared helper via PromptKeyPreambleCfg"
        );
    }
}
