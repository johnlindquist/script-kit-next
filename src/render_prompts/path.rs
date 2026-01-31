// Path prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

impl ScriptListApp {
    /// Handle showing path actions dialog - called from PathPrompt callback.
    /// This method is called directly instead of polling a mutex in render.
    fn handle_show_path_actions(&mut self, path_info: PathInfo, cx: &mut Context<Self>) {
        logging::log(
            "UI",
            &format!(
                "handle_show_path_actions: {} (is_dir={})",
                path_info.path, path_info.is_dir
            ),
        );

        // Create ActionsDialog for this path
        let theme_arc = std::sync::Arc::clone(&self.theme);
        let dialog = cx.new(|cx| {
            // Use a no-op callback - action execution is handled directly in key handler
            let noop_callback: std::sync::Arc<dyn Fn(String) + Send + Sync> =
                std::sync::Arc::new(|_| {});
            let focus_handle = cx.focus_handle();
            let mut dialog =
                ActionsDialog::with_path(focus_handle, noop_callback, &path_info, theme_arc);
            // Hide search in the dialog - we show it in the header instead
            dialog.set_hide_search(true);
            dialog
        });

        self.actions_dialog = Some(dialog);
        self.show_actions_popup = true;
        // Update shared showing state for toggle behavior
        if let Ok(mut showing_guard) = self.path_actions_showing.lock() {
            *showing_guard = true;
        }
        cx.notify();
    }

    /// Close path actions dialog - called from PathPrompt callback or key handler.
    fn handle_close_path_actions(&mut self, cx: &mut Context<Self>) {
        logging::log("UI", "handle_close_path_actions called");
        self.show_actions_popup = false;
        self.actions_dialog = None;
        if let Ok(mut showing_guard) = self.path_actions_showing.lock() {
            *showing_guard = false;
        }
        cx.notify();
    }

    fn render_path_prompt(
        &mut self,
        entity: Entity<PathPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
        // Shadows are handled by app_shell
        let _box_shadows = self.create_box_shadows();

        // NOTE: No side-effects in render! Dialog creation and action execution
        // are handled by handle_show_path_actions() and execute_path_action()
        // which are called from callbacks and key handlers.

        // Get actions dialog if showing
        let actions_dialog = if self.show_actions_popup {
            self.actions_dialog.clone()
        } else {
            None
        };

        // Sync the actions search text from the dialog to the shared state
        // This is a read-only operation (safe in render)
        if let Some(ref dialog) = actions_dialog {
            let search_text = dialog.read(cx).search_text.clone();
            if let Ok(mut guard) = self.path_actions_search_text.lock() {
                *guard = search_text;
            }
        } else {
            // Clear search text when dialog is not showing
            if let Ok(mut guard) = self.path_actions_search_text.lock() {
                guard.clear();
            }
        }

        // Key handler for when actions dialog is showing
        // This intercepts keys and routes them to the dialog (like main menu does)
        let path_entity = entity.clone();
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
                if this.handle_global_shortcut_with_options(event, true, cx) {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                logging::log(
                    "KEY",
                    &format!(
                        "PathPrompt OUTER handler: key='{}', show_actions_popup={}",
                        key_str, this.show_actions_popup
                    ),
                );

                // Cmd+K toggles actions from anywhere
                if has_cmd && key_str == "k" {
                    // Toggle the actions dialog
                    if this.show_actions_popup {
                        // Close actions
                        this.show_actions_popup = false;
                        this.actions_dialog = None;
                        if let Ok(mut guard) = this.path_actions_showing.lock() {
                            *guard = false;
                        }
                        cx.notify();
                    } else {
                        // Open actions - trigger the callback in PathPrompt
                        path_entity.update(cx, |prompt, cx| {
                            prompt.toggle_actions(cx);
                        });
                    }
                    return;
                }

                // If actions popup is open, route keyboard events to it
                if this.show_actions_popup {
                    if let Some(ref dialog) = this.actions_dialog {
                        match key_str.as_str() {
                            "up" | "arrowup" => {
                                dialog.update(cx, |d, cx| d.move_up(cx));
                            }
                            "down" | "arrowdown" => {
                                dialog.update(cx, |d, cx| d.move_down(cx));
                            }
                            "enter" | "return" => {
                                // Get the selected action and execute it
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();
                                if let Some(action_id) = action_id {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "Path action selected via Enter: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );

                                    // Get path info from PathPrompt
                                    let path_info = path_entity.read(cx).get_selected_path_info();

                                    // Close dialog if action says so (built-in path actions always close)
                                    if should_close {
                                        this.show_actions_popup = false;
                                        this.actions_dialog = None;
                                        if let Ok(mut guard) = this.path_actions_showing.lock() {
                                            *guard = false;
                                        }

                                        // Focus back to PathPrompt
                                        if let AppView::PathPrompt { focus_handle, .. } =
                                            &this.current_view
                                        {
                                            window.focus(focus_handle, cx);
                                        }
                                    }

                                    // Execute the action if we have path info
                                    if let Some(info) = path_info {
                                        this.execute_path_action(
                                            &action_id,
                                            &info,
                                            &path_entity,
                                            cx,
                                        );
                                    }
                                }
                            }
                            "escape" | "esc" => {
                                this.show_actions_popup = false;
                                this.actions_dialog = None;
                                if let Ok(mut guard) = this.path_actions_showing.lock() {
                                    *guard = false;
                                }
                                // Focus back to PathPrompt
                                if let AppView::PathPrompt { focus_handle, .. } = &this.current_view
                                {
                                    window.focus(focus_handle, cx);
                                }
                                cx.notify();
                            }
                            "backspace" => {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                            }
                            _ => {
                                let modifiers = &event.keystroke.modifiers;

                                // Check for printable character input (only when no modifiers are held)
                                // This prevents Cmd+E from being treated as typing 'e' into the search
                                if !modifiers.platform && !modifiers.control && !modifiers.alt {
                                    if let Some(ref key_char) = event.keystroke.key_char {
                                        if let Some(ch) = key_char.chars().next() {
                                            if !ch.is_control() {
                                                dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                            }
                                        }
                                    }
                                    return;
                                }

                                // Check if keystroke matches any action shortcut in the dialog
                                let key_lower = key_str.to_lowercase();
                                let keystroke_shortcut =
                                    shortcuts::keystroke_to_shortcut(&key_lower, modifiers);

                                // Read dialog actions and look for matching shortcut
                                let dialog_ref = dialog.read(cx);
                                let mut matched_action: Option<String> = None;
                                for action in &dialog_ref.actions {
                                    if let Some(ref display_shortcut) = action.shortcut {
                                        let normalized =
                                            Self::normalize_display_shortcut(display_shortcut);
                                        if normalized == keystroke_shortcut {
                                            matched_action = Some(action.id.clone());
                                            break;
                                        }
                                    }
                                }
                                let _ = dialog_ref;

                                if let Some(action_id) = matched_action {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "Path actions dialog shortcut matched: {} -> {}",
                                            keystroke_shortcut, action_id
                                        ),
                                    );

                                    // Get path info before closing dialog
                                    let path_info = path_entity.read(cx).get_selected_path_info();

                                    // Close the dialog
                                    this.show_actions_popup = false;
                                    this.actions_dialog = None;
                                    if let Ok(mut guard) = this.path_actions_showing.lock() {
                                        *guard = false;
                                    }

                                    // Focus back to PathPrompt
                                    if let AppView::PathPrompt { focus_handle, .. } =
                                        &this.current_view
                                    {
                                        window.focus(focus_handle, cx);
                                    }

                                    // Execute the action
                                    if let Some(info) = path_info {
                                        this.execute_path_action(
                                            &action_id,
                                            &info,
                                            &path_entity,
                                            cx,
                                        );
                                    }
                                    cx.notify();
                                }
                            }
                        }
                    }
                }
                // If actions not showing, let PathPrompt handle the keys via its own handler
            },
        );

        // PathPrompt entity has its own track_focus and on_key_down in its render method.
        // We add an outer key handler to intercept events when actions are showing.
        div()
            .relative()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
            .w_full()
            .h_full()
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .key_context("path_prompt_container")
            .on_key_down(handle_key)
            .child(div().size_full().child(entity))
            // Actions dialog overlays on top (upper-right corner, below the header bar)
            .when_some(actions_dialog, |d, dialog| {
                d.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .justify_end()
                        .pt(px(52.)) // Clear the header bar (~44px header + 8px margin)
                        .pr(px(8.))
                        .child(dialog),
                )
            })
            .into_any_element()
    }
}
