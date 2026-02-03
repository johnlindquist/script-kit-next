// Term prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

impl ScriptListApp {
    fn render_term_prompt(
        &mut self,
        entity: Entity<term_prompt::TermPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let has_actions =
            self.sdk_actions.is_some() && !self.sdk_actions.as_ref().unwrap().is_empty();

        // Sync suppress_keys with actions popup state so terminal ignores keys when popup is open
        let show_actions = self.show_actions_popup;
        entity.update(cx, |term, _| {
            term.suppress_keys = show_actions;
        });

        // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
        // Shadows are handled by app_shell
        let _box_shadows = self.create_box_shadows();

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(&self.theme);

        // Use explicit height from layout constants instead of h_full()
        // h_full() doesn't work at the root level because there's no parent to fill
        let content_height = window_resize::layout::MAX_HEIGHT;

        // Key handler for Cmd+K actions toggle
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

                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // For QuickTerminalView (built-in utility): ESC returns to main menu or closes window
                // This is different from TermPrompt (SDK prompt) which doesn't respond to ESC
                if matches!(this.current_view, AppView::QuickTerminalView { .. }) {
                    if key_str == "escape" && !this.show_actions_popup {
                        logging::log("KEY", "ESC in QuickTerminalView");
                        this.go_back_or_close(window, cx);
                        return;
                    }

                    if has_cmd && key_str == "w" {
                        logging::log("KEY", "Cmd+W - closing window");
                        this.close_and_reset_window(cx);
                        return;
                    }
                }

                // Global shortcuts (Cmd+W only - term is NOT dismissable with ESC)
                // Note: When actions popup is open, ESC should close the popup
                if !this.show_actions_popup
                    && this.handle_global_shortcut_with_options(event, false, cx)
                {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && ui_foundation::is_key_k(key) && has_actions_for_handler {
                    logging::log("KEY", "Cmd+K in TermPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
                    return;
                }

                let modifiers = &event.keystroke.modifiers;

                // Route to shared actions dialog handler (modal when open)
                match this.route_key_to_actions_dialog(
                    key,
                    key_char,
                    modifiers,
                    ActionsDialogHost::TermPrompt,
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
                }
                // Let other keys fall through to the terminal
            },
        );

        // Footer colors for the terminal prompt - use theme for consistent styling
        let footer_colors = PromptFooterColors::from_theme(&self.theme);

        // Footer configuration - Terminal uses "Close" as primary action (Cmd+W)
        // and Actions as secondary (Cmd+K) when actions are available
        let footer_config = PromptFooterConfig::new()
            .primary_label("Close")
            .primary_shortcut("⌘W")
            .show_secondary(has_actions);

        // Handlers for footer buttons
        let handle_close = cx.entity().downgrade();
        let handle_actions = cx.entity().downgrade();

        // Container with explicit height. We wrap the entity in a sized div because
        // GPUI entities don't automatically inherit parent flex sizing.
        // NOTE: No rounded corners for terminal - it should fill edge-to-edge
        div()
            .relative() // Needed for absolute positioned actions dialog overlay
            .flex()
            .flex_col()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg)) // VIBRANCY: Only apply bg when vibrancy disabled
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
            .w_full()
            .h(content_height)
            .overflow_hidden()
            .on_key_down(handle_key)
            // Terminal content takes remaining space
            .child(div().flex_1().min_h(px(0.)).overflow_hidden().child(entity))
            // Footer at the bottom
            .child(
                PromptFooter::new(footer_config, footer_colors)
                    .on_primary_click(Box::new(move |_, _window, cx| {
                        if let Some(app) = handle_close.upgrade() {
                            app.update(cx, |this, cx| {
                                this.close_and_reset_window(cx);
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
                                "Term actions backdrop clicked - dismissing dialog",
                            );
                            this.close_actions_popup(ActionsDialogHost::TermPrompt, window, cx);
                        },
                    );

                    d.child(
                        div()
                            .absolute()
                            .inset_0()
                            .child(
                                div()
                                    .id("term-actions-backdrop")
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
