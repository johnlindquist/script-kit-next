mod __render_prompts_div_docs {
    //! Div prompt rendering integration for `ScriptListApp::render_div_prompt`.
    //! The key surface is the single render method that wires keyboard handling and action popups.
    //! It depends on prompt-shell/components/theme tokens and is included into the main app module.
}

// Div prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

impl ScriptListApp {
    fn render_div_prompt(
        &mut self,
        id: String,
        entity: Entity<DivPrompt>,
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
        let (actions_dialog_top, actions_dialog_right) =
            prompt_actions_dialog_offsets(design_spacing.padding_sm, design_visual.border_thin);

        // Key handler for Cmd+K actions toggle (at parent level to intercept before DivPrompt)
        let has_actions_for_handler = has_actions;
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                // Note: Escape when actions popup is open should close the popup, not dismiss prompt
                // So only handle global shortcuts when popup is closed
                if !this.show_actions_popup
                    && this.handle_global_shortcut_with_options(event, true, cx)
                {
                    cx.stop_propagation();
                    return;
                }

                let key = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && ui_foundation::is_key_k(key) && has_actions_for_handler {
                    logging::log("KEY", "Cmd+K in DivPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
                    cx.stop_propagation();
                    return;
                }

                let modifiers = &event.keystroke.modifiers;

                // Route to shared actions dialog handler (modal when open)
                match this.route_key_to_actions_dialog(
                    key,
                    key_char,
                    modifiers,
                    ActionsDialogHost::DivPrompt,
                    window,
                    cx,
                ) {
                    ActionsRoute::Execute { action_id } => {
                        this.trigger_action_by_name(&action_id, cx);
                        cx.stop_propagation();
                        return;
                    }
                    ActionsRoute::Handled => {
                        // Key consumed by actions dialog
                        cx.stop_propagation();
                        return;
                    }
                    ActionsRoute::NotHandled => {
                        // Actions popup not open - check SDK action shortcuts
                        let key_lower = key.to_lowercase();
                        let shortcut_key = shortcuts::keystroke_to_shortcut(
                            &key_lower,
                            &event.keystroke.modifiers,
                        );
                        if let Some(action_name) = this.action_shortcuts.get(&shortcut_key).cloned()
                        {
                            logging::log(
                                "KEY",
                                &format!(
                                    "SDK action shortcut matched in DivPrompt: {}",
                                    action_name
                                ),
                            );
                            this.trigger_action_by_name(&action_name, cx);
                            cx.stop_propagation();
                            return;
                        }
                    }
                }
            },
        );

        // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
        // Shadows are handled by app_shell
        let _ = self.create_box_shadows();

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(&self.theme);

        // Use explicit height from layout constants
        let content_height = window_resize::layout::STANDARD_HEIGHT;

        // Footer colors and config aligned with other interactive prompts.
        let footer_colors =
            prompt_footer_colors_for_prompt(&design_colors, !self.theme.is_dark_mode());
        let footer_config = prompt_footer_config_with_status(
            "Continue",
            has_actions,
            Some(running_status_text("review output and press Enter")),
            Some("Output".to_string()),
        );

        // Create click handlers for footer
        let handle_submit = cx.entity().downgrade();
        let handle_actions = cx.entity().downgrade();
        let prompt_id = id.clone();

        crate::components::prompt_shell_container(design_visual.radius_lg, vibrancy_bg)
            .h(content_height)
            .track_focus(&self.focus_handle) // Required to receive key events
            .on_key_down(handle_key)
            // Header + content shell
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.)) // Critical: allows flex children to size properly
                    .overflow_hidden()
                    .child(
                        div()
                            .w_full()
                            .px(px(HEADER_PADDING_X))
                            .py(px(HEADER_PADDING_Y))
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .gap(px(HEADER_GAP))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(design_colors.text_primary))
                                    .font_family(design_typography.font_family)
                                    .child("Script Output"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(design_colors.text_muted))
                                    .font_family(design_typography.font_family)
                                    .child("Enter to continue"),
                            ),
                    )
                    .child(
                        div()
                            .mx(px(design_spacing.padding_lg))
                            .h(px(design_visual.border_thin))
                            .bg(rgba((design_colors.border << 8) | 0x60)),
                    )
                    .child(crate::components::prompt_shell_content(entity.clone())),
            )
            // Footer with Submit button and Actions
            .child(
                PromptFooter::new(footer_config, footer_colors)
                    .on_primary_click(Box::new(move |_, _window, cx| {
                        if let Some(app) = handle_submit.upgrade() {
                            let id = prompt_id.clone();
                            app.update(cx, |this, cx| {
                                // Submit the div prompt - send empty value to continue
                                this.submit_prompt_response(id, None, cx);
                            });
                        }
                    }))
                    .on_secondary_click(Box::new(move |_, window, cx| {
                        if let Some(app) = handle_actions.upgrade() {
                            app.update(cx, |this, cx| {
                                this.toggle_arg_actions(cx, window);
                            });
                        }
                    })),
            )
            // Actions dialog overlay (when Cmd+K is pressed with SDK actions)
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
                                "Div actions backdrop clicked - dismissing dialog",
                            );
                            this.close_actions_popup(ActionsDialogHost::DivPrompt, window, cx);
                        },
                    );

                    d.child(
                        div()
                            .absolute()
                            .inset_0()
                            .child(
                                div()
                                    .id("div-actions-backdrop")
                                    .absolute()
                                    .inset_0()
                                    .on_click(backdrop_click),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .top(px(actions_dialog_top))
                                    .right(px(actions_dialog_right))
                                    .child(dialog),
                            ),
                    )
                },
            )
            .into_any_element()
    }
}
