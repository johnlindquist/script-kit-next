mod __render_prompts_path_docs {
    //! Path prompt rendering and in-place ⌘K staging for filesystem entries.
    //! Key methods include `render_path_prompt`, `handle_show_path_actions`, and the shared-state sync helpers.
    //! It depends on `PathPrompt` state, the one-window view staging, and focus handling in `ScriptListApp`.
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

    /// ⌘K on a path prompt, from its own key or its event: the highlighted entry goes in the
    /// line above the input and the prompt's list becomes that entry's actions.
    fn handle_show_path_actions(&mut self, path_info: PathInfo, cx: &mut Context<Self>) {
        logging::log(
            "UI",
            &format!(
                "handle_show_path_actions: {} (is_dir={})",
                path_info.path, path_info.is_dir
            ),
        );
        self.one_window_in_window_later(cx, move |app, window, cx| {
            app.stage_path_actions(path_info, window, cx)
        });
    }

    fn stage_path_actions(
        &mut self,
        path_info: PathInfo,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !matches!(self.current_view, AppView::PathPrompt { .. })
            || self.one_window_view_staged().is_some()
        {
            return;
        }
        let theme_arc = std::sync::Arc::clone(&self.theme);
        let dialog = cx.new(|cx| {
            // Action execution is handled by execute_action_for_actions_host.
            let noop_callback: std::sync::Arc<dyn Fn(String) + Send + Sync> =
                std::sync::Arc::new(|_| {});
            let focus_handle = cx.focus_handle();
            let mut dialog =
                ActionsDialog::with_path(focus_handle, noop_callback, &path_info, theme_arc);
            dialog.set_match_main_window_background(true);
            dialog
        });
        self.one_window_stage_in_view(
            ActionsDialogHost::PathPrompt,
            dialog,
            Some(path_info.name.clone()),
            window,
            cx,
        );
        self.path_prompt_update_actions_showing(true, "handle_show_path_actions");
        self.path_prompt_sync_actions_search_from_dialog(cx, "handle_show_path_actions");
    }

    /// The path prompt's own ⌘K while its entry is staged lets go of it.
    fn handle_close_path_actions(&mut self, cx: &mut Context<Self>) {
        logging::log("UI", "handle_close_path_actions called");
        self.one_window_in_window_later(cx, |app, window, cx| {
            if app.one_window_view_staged().is_some() {
                app.close_actions_popup(ActionsDialogHost::PathPrompt, window, cx);
            }
        });
    }

    fn render_path_prompt(
        &mut self,
        entity: Entity<PathPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let context = self.prompt_header_context();
        entity.update(cx, |prompt, _cx| prompt.set_header_context(context));
        tracing::info!(
            surface = "render_prompts::path",
            actions_open = self.show_actions_popup,
            "prompt_surface_rendered"
        );
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "render_prompts::path",
                self.has_nonempty_sdk_actions(),
            ),
        );

        // While an entry's actions are staged the main window draws them in this prompt's
        // place, and their keys route there; this handler only sees the prompt itself.
        let path_entity = entity.clone();
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                if this.handle_global_shortcut_with_options(event, GlobalShortcutEscape::FromDismissPolicy, cx) {
                    return;
                }

                let key = event.keystroke.key.as_str();
                if event.keystroke.modifiers.platform && crate::ui_foundation::is_key_k(key) {
                    let correlation_id = logging::current_correlation_id();
                    logging::log(
                        "KEY",
                        &format!(
                            "{PATH_PROMPT_KEY_CONTEXT}: Cmd+K stages the highlighted entry (correlation_id={correlation_id})"
                        ),
                    );
                    path_entity.update(cx, |prompt, cx| {
                        prompt.toggle_actions(cx);
                    });
                }
            },
        );

        crate::components::prompt_shell_container(
            0.0,
            crate::ui_foundation::get_vibrancy_background(&self.theme),
        )
        .key_context("path_prompt_container")
        .on_key_down(handle_key)
        .child(div().size_full().child(entity))
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

    #[test]
    fn test_path_prompt_actions_stage_in_place() {
        const PATH_RENDER_SOURCE: &str = include_str!("path.rs");

        let render = PATH_RENDER_SOURCE
            .split("fn render_path_prompt(")
            .nth(1)
            .and_then(|rest| rest.split("#[cfg(test)]").next())
            .expect("render_path_prompt should exist");

        assert!(
            PATH_RENDER_SOURCE.contains("self.one_window_stage_in_view(\n            ActionsDialogHost::PathPrompt,"),
            "path prompt Cmd+K should stage the highlighted entry in the main window"
        );
        assert!(
            !render.contains(".child(dialog)"),
            "path prompt should not draw its actions as an overlay"
        );
    }

    #[test]
    fn test_path_prompt_actions_match_main_window_background() {
        const PATH_RENDER_SOURCE: &str = include_str!("path.rs");

        let show_actions_block = PATH_RENDER_SOURCE
            .split("fn handle_show_path_actions(&mut self, path_info: PathInfo, cx: &mut Context<Self>) {")
            .nth(1)
            .expect("handle_show_path_actions should exist");

        assert!(
            show_actions_block.contains("dialog.set_match_main_window_background(true);"),
            "path prompt actions dialog should match the main window background for consistency"
        );
    }
}
