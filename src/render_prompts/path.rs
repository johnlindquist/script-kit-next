mod __render_prompts_path_docs {
    //! Path prompt rendering and actions-popup synchronization for filesystem entries.
    //! Key methods include `render_path_prompt`, `handle_show_path_actions`, and close/sync helpers.
    //! It depends on `PathPrompt` state, actions dialog plumbing, and focus handling in `ScriptListApp`.
}

// Path prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

const PATH_PROMPT_KEY_CONTEXT: &str = "path_prompt_outer";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathPromptSharedStateUpdate {
    Updated,
    Unchanged,
    LockError,
}

#[inline]
fn path_prompt_set_actions_showing_state(
    state: &std::sync::Arc<std::sync::Mutex<bool>>,
    showing: bool,
) -> PathPromptSharedStateUpdate {
    match state.lock() {
        Ok(mut guard) => {
            if *guard == showing {
                PathPromptSharedStateUpdate::Unchanged
            } else {
                *guard = showing;
                PathPromptSharedStateUpdate::Updated
            }
        }
        Err(_) => PathPromptSharedStateUpdate::LockError,
    }
}

#[inline]
fn path_prompt_set_actions_search_text(
    state: &std::sync::Arc<std::sync::Mutex<String>>,
    search_text: &str,
) -> PathPromptSharedStateUpdate {
    match state.lock() {
        Ok(mut guard) => {
            if guard.as_str() == search_text {
                PathPromptSharedStateUpdate::Unchanged
            } else {
                *guard = search_text.to_string();
                PathPromptSharedStateUpdate::Updated
            }
        }
        Err(_) => PathPromptSharedStateUpdate::LockError,
    }
}

impl ScriptListApp {
    fn path_prompt_update_actions_showing(&self, showing: bool, source: &str) {
        let correlation_id = logging::current_correlation_id();
        match path_prompt_set_actions_showing_state(&self.path_actions_showing, showing) {
            PathPromptSharedStateUpdate::Updated => {
                logging::log_debug(
                    "ACTIONS",
                    &format!(
                        "{PATH_PROMPT_KEY_CONTEXT}: actions_showing={showing} (source={source}, correlation_id={correlation_id})"
                    ),
                );
            }
            PathPromptSharedStateUpdate::Unchanged => {}
            PathPromptSharedStateUpdate::LockError => {
                logging::log(
                    "ERROR",
                    &format!(
                        "{PATH_PROMPT_KEY_CONTEXT}: failed to update actions_showing mutex (attempted={showing}, source={source}, correlation_id={correlation_id})"
                    ),
                );
            }
        }
    }

    fn path_prompt_update_actions_search_text(&self, search_text: &str, source: &str) {
        let correlation_id = logging::current_correlation_id();
        match path_prompt_set_actions_search_text(&self.path_actions_search_text, search_text) {
            PathPromptSharedStateUpdate::Updated => {
                logging::log_debug(
                    "ACTIONS",
                    &format!(
                        "{PATH_PROMPT_KEY_CONTEXT}: actions_search_text updated (len={}, source={source}, correlation_id={correlation_id})",
                        search_text.chars().count()
                    ),
                );
            }
            PathPromptSharedStateUpdate::Unchanged => {}
            PathPromptSharedStateUpdate::LockError => {
                logging::log(
                    "ERROR",
                    &format!(
                        "{PATH_PROMPT_KEY_CONTEXT}: failed to update actions_search_text mutex (attempted_len={}, source={source}, correlation_id={correlation_id})",
                        search_text.chars().count()
                    ),
                );
            }
        }
    }

    fn path_prompt_sync_actions_search_from_dialog(&self, cx: &mut Context<Self>, source: &str) {
        let next_search_text = self
            .actions_dialog
            .as_ref()
            .map(|dialog| dialog.read(cx).search_text.clone())
            .unwrap_or_default();
        self.path_prompt_update_actions_search_text(&next_search_text, source);
    }

    fn path_prompt_focus_after_actions_close(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let AppView::PathPrompt { focus_handle, .. } = &self.current_view {
            window.focus(focus_handle, cx);
        }
    }

    fn path_prompt_close_actions_popup(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        source: &str,
    ) {
        let correlation_id = logging::current_correlation_id();
        logging::log_debug(
            "ACTIONS",
            &format!(
                "{PATH_PROMPT_KEY_CONTEXT}: closing actions popup (source={source}, correlation_id={correlation_id})"
            ),
        );
        self.handle_close_path_actions(cx);
        self.path_prompt_focus_after_actions_close(window, cx);
    }

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
        self.path_prompt_update_actions_showing(true, "handle_show_path_actions");
        self.path_prompt_sync_actions_search_from_dialog(cx, "handle_show_path_actions");
        cx.notify();
    }

    /// Close path actions dialog - called from PathPrompt callback or key handler.
    fn handle_close_path_actions(&mut self, cx: &mut Context<Self>) {
        logging::log("UI", "handle_close_path_actions called");
        self.show_actions_popup = false;
        self.actions_dialog = None;
        self.path_prompt_update_actions_showing(false, "handle_close_path_actions");
        self.path_prompt_update_actions_search_text("", "handle_close_path_actions");
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
        let design_spacing = tokens.spacing();
        let design_visual = tokens.visual();
        let (actions_dialog_top, actions_dialog_right) =
            prompt_actions_dialog_offsets(design_spacing.padding_sm, design_visual.border_thin);

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

        // Key handler for when actions dialog is showing
        // This intercepts keys and routes them to the dialog (like main menu does)
        let path_entity = entity.clone();
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
                if this.handle_global_shortcut_with_options(event, true, cx) {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let key_str = key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;
                let modifiers = &event.keystroke.modifiers;
                let correlation_id = logging::current_correlation_id();

                logging::log_debug(
                    "KEY",
                    &format!(
                        "{PATH_PROMPT_KEY_CONTEXT}: key='{}' actions_open={} correlation_id={correlation_id}",
                        key_str, this.show_actions_popup
                    ),
                );

                // Cmd+K toggles actions from anywhere
                if has_cmd && crate::ui_foundation::is_key_k(key) {
                    let (current_path, filtered_count) = {
                        let prompt = path_entity.read(cx);
                        (prompt.current_path.clone(), prompt.filtered_entries.len())
                    };
                    logging::log(
                        "KEY",
                        &format!(
                            "{PATH_PROMPT_KEY_CONTEXT}: Cmd+K toggle (actions_open={}, current_path={}, filtered_count={}, correlation_id={correlation_id})",
                            this.show_actions_popup, current_path, filtered_count
                        ),
                    );

                    // Toggle the actions dialog
                    if this.show_actions_popup {
                        this.path_prompt_close_actions_popup(window, cx, "cmd+k");
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
                    let Some(ref dialog) = this.actions_dialog else {
                        logging::log(
                            "WARN",
                            &format!(
                                "{PATH_PROMPT_KEY_CONTEXT}: actions popup open without dialog entity (correlation_id={correlation_id})"
                            ),
                        );
                        return;
                    };

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
                                        "{PATH_PROMPT_KEY_CONTEXT}: action selected via Enter (action_id={action_id}, close={should_close}, correlation_id={correlation_id})"
                                    ),
                                );

                                // Get path info from PathPrompt
                                let path_info = path_entity.read(cx).get_selected_path_info();

                                // Close dialog if action says so (built-in path actions always close)
                                if should_close {
                                    this.path_prompt_close_actions_popup(window, cx, "enter");
                                }

                                // Execute the action if we have path info
                                if let Some(info) = path_info {
                                    this.execute_path_action(&action_id, &info, &path_entity, cx);
                                }
                            }
                        }
                        "escape" | "esc" => {
                            this.path_prompt_close_actions_popup(window, cx, "escape");
                        }
                        "backspace" => {
                            dialog.update(cx, |d, cx| d.handle_backspace(cx));
                            this.path_prompt_sync_actions_search_from_dialog(cx, "backspace");
                        }
                        _ => {
                            // Check for printable character input (only when no modifiers are held)
                            // This prevents Cmd+E from being treated as typing 'e' into the search
                            if !modifiers.platform && !modifiers.control && !modifiers.alt {
                                if let Some(ref key_char) = event.keystroke.key_char {
                                    if let Some(ch) = key_char.chars().next() {
                                        if !ch.is_control() {
                                            dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                            this.path_prompt_sync_actions_search_from_dialog(
                                                cx,
                                                "printable_char",
                                            );
                                        }
                                    }
                                }
                                return;
                            }

                            // Check if keystroke matches any action shortcut in the dialog
                            let keystroke_shortcut =
                                shortcuts::keystroke_to_shortcut(&key_str, modifiers);

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
                                        "{PATH_PROMPT_KEY_CONTEXT}: actions shortcut matched (shortcut={keystroke_shortcut}, action_id={action_id}, correlation_id={correlation_id})"
                                    ),
                                );

                                // Get path info before closing dialog
                                let path_info = path_entity.read(cx).get_selected_path_info();
                                this.path_prompt_close_actions_popup(window, cx, "shortcut");

                                // Execute the action
                                if let Some(info) = path_info {
                                    this.execute_path_action(&action_id, &info, &path_entity, cx);
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
                        .pt(px(actions_dialog_top))
                        .pr(px(actions_dialog_right))
                        .child(dialog),
                )
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod path_prompt_shared_state_tests {
    use super::*;

    #[test]
    fn test_path_prompt_actions_state_sets_showing_only_when_changed() {
        let state = std::sync::Arc::new(std::sync::Mutex::new(false));

        let first = path_prompt_set_actions_showing_state(&state, true);
        let second = path_prompt_set_actions_showing_state(&state, true);
        let third = path_prompt_set_actions_showing_state(&state, false);

        assert_eq!(first, PathPromptSharedStateUpdate::Updated);
        assert_eq!(second, PathPromptSharedStateUpdate::Unchanged);
        assert_eq!(third, PathPromptSharedStateUpdate::Updated);
        assert!(!*state.lock().expect("showing state lock should succeed"));
    }

    #[test]
    fn test_path_prompt_actions_state_sets_search_text_only_when_changed() {
        let state = std::sync::Arc::new(std::sync::Mutex::new(String::new()));

        let first = path_prompt_set_actions_search_text(&state, "open");
        let second = path_prompt_set_actions_search_text(&state, "open");
        let third = path_prompt_set_actions_search_text(&state, "");

        assert_eq!(first, PathPromptSharedStateUpdate::Updated);
        assert_eq!(second, PathPromptSharedStateUpdate::Unchanged);
        assert_eq!(third, PathPromptSharedStateUpdate::Updated);
        assert_eq!(
            state
                .lock()
                .expect("search text state lock should succeed")
                .as_str(),
            ""
        );
    }
}
