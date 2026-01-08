// Editor prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

impl ScriptListApp {
    fn render_editor_prompt(
        &mut self,
        entity: Entity<EditorPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let has_actions =
            self.sdk_actions.is_some() && !self.sdk_actions.as_ref().unwrap().is_empty();

        // Sync suppress_keys with actions popup state so editor ignores keys when popup is open
        let show_actions = self.show_actions_popup;
        entity.update(cx, |editor, _| {
            editor.suppress_keys = show_actions;
        });

        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let _design_spacing = tokens.spacing();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let box_shadows = self.create_box_shadows();

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(&self.theme);

        // Use explicit height from layout constants instead of h_full()
        // h_full() doesn't work at the root level because there's no parent to fill
        let content_height = window_resize::layout::MAX_HEIGHT;

        // Key handler for Cmd+K actions toggle (at parent level to intercept before editor)
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

                // Global shortcuts (Cmd+W only - editor is NOT dismissable with ESC)
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
                    logging::log("KEY", "Cmd+K in EditorPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
                    return;
                }

                // Route to shared actions dialog handler (modal when open)
                match this.route_key_to_actions_dialog(
                    key,
                    key_char,
                    ActionsDialogHost::EditorPrompt,
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
                // Let other keys fall through to the editor
            },
        );

        // Clone entity for footer submit button
        let entity_for_footer = entity.clone();

        // Get the prompt ID for submit
        let prompt_id = entity.read(cx).id.clone();

        // Extract editor state for footer display
        let (snippet_helper_text, language_label) = {
            let editor = entity.read(cx);
            let language = editor.language().to_string();

            // Build snippet indicator if in snippet mode
            let snippet_text = editor.snippet_state().map(|state| {
                let current = state.current_tabstop_idx + 1; // 1-based for display
                let total = state.snippet.tabstops.len();

                // Get the current tabstop's display name (placeholder or index)
                let current_name = state
                    .snippet
                    .tabstops
                    .get(state.current_tabstop_idx)
                    .and_then(|ts| {
                        ts.placeholder.clone().or_else(|| {
                            ts.choices
                                .as_ref()
                                .and_then(|c: &Vec<String>| c.first().cloned())
                        })
                    })
                    .unwrap_or_else(|| format!("${}", current));

                format!(
                    "Tab {} of {} · \"{}\" · Tab to continue, Esc to exit",
                    current, total, current_name
                )
            });

            (snippet_text, language)
        };

        // NOTE: The EditorPrompt entity has its own track_focus and on_key_down in its render method.
        // We do NOT add track_focus here to avoid duplicate focus tracking on the same handle.
        //
        // Container with flex layout:
        // - Editor wrapper using flex_1 to fill remaining space above footer
        // - Footer as normal child at bottom (40px fixed height)
        div()
            .relative() // Needed for absolute positioned actions dialog overlay
            .flex()
            .flex_col()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg)) // VIBRANCY: Only apply bg when vibrancy disabled
            .shadow(box_shadows)
            .w_full()
            .h(content_height) // Explicit 700px height (window height for editor view)
            .overflow_hidden() // Clip content to rounded corners
            .rounded(px(design_visual.radius_lg))
            .on_key_down(handle_key)
            // Editor entity fills remaining space (flex_1)
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.)) // Required for flex children to shrink properly
                    .w_full()
                    .overflow_hidden()
                    .child(entity),
            )
            // Unified footer with Submit + Actions
            .child({
                let handle_submit = cx.entity().downgrade();
                let handle_actions = cx.entity().downgrade();
                let entity_weak = entity_for_footer.downgrade();
                let prompt_id_for_submit = prompt_id.clone();

                let footer_colors = PromptFooterColors {
                    accent: design_colors.accent,
                    text_muted: design_colors.text_muted,
                    border: design_colors.border,
                    background: design_colors.background_selected, // Match selected item bg
                };

                // Build footer config with optional helper text and language
                let mut footer_config = PromptFooterConfig::new()
                    .primary_label("Submit")
                    .primary_shortcut("↵")
                    .secondary_label("Actions")
                    .secondary_shortcut("⌘K")
                    .show_secondary(has_actions)
                    .info_label(language_label.clone());

                // Add snippet helper text if in snippet mode
                if let Some(ref helper) = snippet_helper_text {
                    footer_config = footer_config.helper_text(helper.clone());
                }

                let mut footer = PromptFooter::new(footer_config, footer_colors).on_primary_click(
                    Box::new(move |_, _window, cx| {
                        // Get editor content and submit
                        if let Some(editor_entity) = entity_weak.upgrade() {
                            let content = editor_entity.update(cx, |editor, cx| editor.content(cx));
                            if let Some(app) = handle_submit.upgrade() {
                                app.update(cx, |this, cx| {
                                    logging::log("EDITOR", "Footer Submit button clicked");
                                    this.submit_prompt_response(
                                        prompt_id_for_submit.clone(),
                                        Some(content),
                                        cx,
                                    );
                                });
                            }
                        }
                    }),
                );

                if has_actions {
                    footer = footer.on_secondary_click(Box::new(move |_, window, cx| {
                        if let Some(app) = handle_actions.upgrade() {
                            app.update(cx, |this, cx| {
                                this.toggle_arg_actions(cx, window);
                            });
                        }
                    }));
                }

                // Footer as normal flex child (not absolute positioned)
                // The flex_1 editor wrapper above takes remaining space
                footer
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
                                "Editor actions backdrop clicked - dismissing dialog",
                            );
                            this.close_actions_popup(ActionsDialogHost::EditorPrompt, window, cx);
                        },
                    );

                    d.child(
                        div()
                            .absolute()
                            .inset_0()
                            .child(
                                div()
                                    .id("editor-actions-backdrop")
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
