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
        let _design_spacing = tokens.spacing();
        let design_visual = tokens.visual();

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
                    return;
                }

                let key = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && ui_foundation::is_key_k(key) && has_actions_for_handler {
                    logging::log("KEY", "Cmd+K in DivPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
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
                    }
                    ActionsRoute::Handled => {
                        // Key consumed by actions dialog
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

        // Footer colors and handlers for PromptFooter
        let footer_colors = PromptFooterColors {
            accent: design_colors.accent,
            text_muted: design_colors.text_muted,
            border: design_colors.border,
            background: design_colors.background_selected, // Match selected item bg
            is_light_mode: !self.theme.is_dark_mode(),
        };

        // Footer config with Submit as primary action
        let footer_config = PromptFooterConfig::new()
            .primary_label("Submit")
            .primary_shortcut("↵")
            .show_secondary(has_actions);

        // Create click handlers for footer
        let handle_submit = cx.entity().downgrade();
        let handle_actions = cx.entity().downgrade();
        let prompt_id = id.clone();

        div()
            .relative() // Needed for absolute positioned overlays
            .flex()
            .flex_col()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg)) // VIBRANCY: Only apply bg when vibrancy disabled
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
            .w_full()
            .h(content_height)
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .track_focus(&self.focus_handle) // Required to receive key events
            .on_key_down(handle_key)
            // Content area - flex-1 to fill remaining space above footer
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.)) // Critical: allows flex children to size properly
                    .overflow_hidden()
                    .child(entity.clone()),
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
                            .child(div().absolute().top(px(52.)).right(px(8.)).child(dialog)),
                    )
                },
            )
            .into_any_element()
    }
}
