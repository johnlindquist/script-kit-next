// Form prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

impl ScriptListApp {
    fn render_form_prompt(
        &mut self,
        entity: Entity<FormPromptState>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let has_actions =
            self.sdk_actions.is_some() && !self.sdk_actions.as_ref().unwrap().is_empty();

        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Get prompt ID and field count from entity
        let prompt_id = entity.read(cx).id.clone();
        let field_count = entity.read(cx).fields.len();

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
                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                // Note: Escape when actions popup is open should close the popup, not dismiss prompt
                if !this.show_actions_popup
                    && this.handle_global_shortcut_with_options(event, true, cx)
                {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                let has_shift = event.keystroke.modifiers.shift;
                let has_cmd = event.keystroke.modifiers.platform;

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && ui_foundation::is_key_k(key) && has_actions_for_handler {
                    logging::log("KEY", "Cmd+K in FormPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
                    return;
                }

                let modifiers = &event.keystroke.modifiers;

                // Route to shared actions dialog handler (modal when open)
                match this.route_key_to_actions_dialog(
                    key,
                    key_char,
                    modifiers,
                    ActionsDialogHost::FormPrompt,
                    window,
                    cx,
                ) {
                    ActionsRoute::Execute { action_id } => {
                        this.trigger_action_by_name(&action_id, cx);
                        return;
                    }
                    ActionsRoute::Handled => {
                        // Key consumed by actions dialog
                        return;
                    }
                    ActionsRoute::NotHandled => {
                        // Actions popup not open - continue with normal handling
                    }
                }

                // Check for SDK action shortcuts (only when popup is NOT open)
                let key_lower = key.to_lowercase();
                let shortcut_key =
                    shortcuts::keystroke_to_shortcut(&key_lower, &event.keystroke.modifiers);
                if let Some(action_name) = this.action_shortcuts.get(&shortcut_key).cloned() {
                    logging::log(
                        "KEY",
                        &format!("SDK action shortcut matched: {}", action_name),
                    );
                    this.trigger_action_by_name(&action_name, cx);
                    return;
                }

                // Handle form-level keys (Enter, Escape, Tab) at this level
                // Forward all other keys to the focused form field for text input
                if ui_foundation::is_key_enter(key) {
                    // Enter submits the form - collect all field values
                    logging::log("KEY", "Enter in FormPrompt - submitting form");
                    let values = entity_for_submit.read(cx).collect_values(cx);
                    logging::log("FORM", &format!("Form values: {}", values));
                    this.submit_prompt_response(prompt_id_for_key.clone(), Some(values), cx);
                    return;
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

        // Use design tokens for global theming
        let box_shadows = self.create_box_shadows();

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(&self.theme);

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
            .shadow(box_shadows)
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
                    .overflow_y_hidden() // Clip content at container boundary
                    .p(px(design_spacing.padding_xl))
                    // Render the form entity (contains all fields)
                    .child(entity.clone()),
            )
            // Unified footer with PromptFooter component
            .child({
                let footer_colors = PromptFooterColors::from_theme(&self.theme);
                let footer_config = PromptFooterConfig::new()
                    .primary_label("Submit Form")
                    .primary_shortcut("↵")
                    .show_secondary(has_actions);

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
                if self.show_actions_popup {
                    self.actions_dialog.clone()
                } else {
                    None
                },
                |d, dialog| {
                    let backdrop_click = cx.listener(
                        |this: &mut Self,
                         _event: &gpui::ClickEvent,
                         window: &mut Window,
                         cx: &mut Context<Self>| {
                            logging::log(
                                "FOCUS",
                                "Form actions backdrop clicked - dismissing dialog",
                            );
                            this.close_actions_popup(ActionsDialogHost::FormPrompt, window, cx);
                        },
                    );

                    d.child(
                        div()
                            .absolute()
                            .inset_0()
                            .child(
                                div()
                                    .id("form-actions-backdrop")
                                    .absolute()
                                    .inset_0()
                                    .on_click(backdrop_click),
                            )
                            .child(div().absolute().top(px(52.)).right(px(8.)).child(dialog)),
                    )
                },
            )
            .into_any_element()
    }
}
