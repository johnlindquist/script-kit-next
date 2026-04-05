use super::*;

impl ScriptListApp {
    pub(crate) fn route_key_to_actions_dialog(
        &mut self,
        key: &str,
        key_char: Option<&str>,
        modifiers: &gpui::Modifiers,
        host: ActionsDialogHost,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> ActionsRoute {
        // Not open - let caller handle the key
        if !self.show_actions_popup {
            return ActionsRoute::NotHandled;
        }

        // Defensive: if UI says it's open but dialog is None, don't leak keys
        let Some(ref dialog) = self.actions_dialog else {
            return ActionsRoute::Handled;
        };

        // Use allocation-free key helpers from ui_foundation
        use crate::ui_foundation::{
            is_key_backspace, is_key_down, is_key_enter, is_key_escape, is_key_up, printable_char,
        };

        // Cmd+K toggles the popup closed through the shared close path
        if modifiers.platform
            && !modifiers.shift
            && !modifiers.control
            && !modifiers.alt
            && key.eq_ignore_ascii_case("k")
        {
            self.close_actions_popup(host, window, cx);
            return ActionsRoute::Handled;
        }

        if is_key_up(key) {
            dialog.update(cx, |d, cx| d.move_up(cx));
            crate::actions::notify_actions_window(cx);
            return ActionsRoute::Handled;
        }

        if is_key_down(key) {
            dialog.update(cx, |d, cx| d.move_down(cx));
            crate::actions::notify_actions_window(cx);
            return ActionsRoute::Handled;
        }

        let is_home = key.eq_ignore_ascii_case("home");
        let is_end = key.eq_ignore_ascii_case("end");
        let is_page_up = key.eq_ignore_ascii_case("pageup");
        let is_page_down = key.eq_ignore_ascii_case("pagedown");
        const ACTIONS_PAGE_JUMP: usize = 8;

        if is_home || is_end || is_page_up || is_page_down {
            dialog.update(cx, |d, cx| {
                if d.grouped_items.is_empty() {
                    return;
                }

                if is_home || is_page_up {
                    let steps = if is_home {
                        d.grouped_items.len()
                    } else {
                        ACTIONS_PAGE_JUMP
                    };
                    for _ in 0..steps {
                        let previous = d.selected_index;
                        d.move_up(cx);
                        if d.selected_index == previous {
                            break;
                        }
                    }
                    return;
                }

                let steps = if is_end {
                    d.grouped_items.len()
                } else {
                    ACTIONS_PAGE_JUMP
                };
                for _ in 0..steps {
                    let previous = d.selected_index;
                    d.move_down(cx);
                    if d.selected_index == previous {
                        break;
                    }
                }
            });
            crate::actions::notify_actions_window(cx);
            return ActionsRoute::Handled;
        }

        if is_key_enter(key) {
            let action_id = dialog.read(cx).get_selected_action_id();
            let should_close = dialog.read(cx).selected_action_should_close();

            if let Some(action_id) = action_id {
                logging::log(
                    "ACTIONS",
                    &format!(
                        "Actions dialog executing action: {} (close={}, host={:?})",
                        action_id, should_close, host
                    ),
                );

                if should_close {
                    self.close_actions_popup(host, window, cx);
                }

                return ActionsRoute::Execute { action_id };
            }
            return ActionsRoute::Handled;
        }

        if is_key_escape(key) {
            self.close_actions_popup(host, window, cx);
            return ActionsRoute::Handled;
        }

        if is_key_backspace(key) {
            dialog.update(cx, |d, cx| d.handle_backspace(cx));
            crate::actions::notify_actions_window(cx);
            crate::actions::resize_actions_window(cx, dialog);
            return ActionsRoute::Handled;
        }

        // Check for printable character input (only when no modifiers are held)
        // This prevents Cmd+E from being treated as typing 'e' into the search
        if !modifiers.platform && !modifiers.control && !modifiers.alt {
            if let Some(ch) = printable_char(key_char) {
                dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                crate::actions::notify_actions_window(cx);
                crate::actions::resize_actions_window(cx, dialog);
                return ActionsRoute::Handled;
            }
        }

        // Check if keystroke matches any action shortcut in the dialog
        // This allows Cmd+E, Cmd+L, etc. to execute the corresponding action
        let keystroke_shortcut = shortcuts::keystroke_to_shortcut(key, modifiers);

        // Read dialog actions and look for matching shortcut
        // First pass: find the match (if any) while holding the borrow
        let matched_action_id: Option<String> = {
            let dialog_ref = dialog.read(cx);
            dialog_ref.actions.iter().find_map(|action| {
                action.shortcut.as_ref().and_then(|display_shortcut| {
                    let normalized = Self::normalize_display_shortcut(display_shortcut);
                    if normalized == keystroke_shortcut {
                        Some(action.id.clone())
                    } else {
                        None
                    }
                })
            })
        }; // dialog_ref borrow released here

        // Second pass: execute the action if found (borrow released)
        if let Some(action_id) = matched_action_id {
            logging::log(
                "ACTIONS",
                &format!(
                    "Actions dialog shortcut matched: {} -> {} (host={:?})",
                    keystroke_shortcut, action_id, host
                ),
            );

            // Built-in actions always close the dialog
            self.close_actions_popup(host, window, cx);

            return ActionsRoute::Execute { action_id };
        }

        // Modal behavior: swallow all other keys while popup is open
        ActionsRoute::Handled
    }

    /// Convert a display shortcut (⌘⇧E) to normalized form (cmd+shift+e)
    pub(crate) fn normalize_display_shortcut(hint: &str) -> String {
        crate::components::hint_strip::canonical_shortcut_hint(hint)
    }

    fn should_preserve_main_filter_while_actions_open(&self) -> bool {
        matches!(self.current_view, AppView::ScriptList)
            || self.current_view_uses_shared_filter_input()
    }

    pub(crate) fn mark_filter_resync_after_actions_if_needed(&mut self) {
        if !self.should_preserve_main_filter_while_actions_open() {
            return;
        }

        self.pending_filter_sync = true;
        logging::log(
            "ACTIONS",
            &format!(
                "ACTIONS_FILTER_RESYNC marked pending (show_actions_popup={}, filter_text='{}')",
                self.show_actions_popup, self.filter_text
            ),
        );
    }

    pub(crate) fn resync_filter_input_after_actions_if_needed(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.mark_filter_resync_after_actions_if_needed();
        self.sync_filter_input_if_needed(window, cx);
    }

    pub(crate) fn request_focus_restore_for_actions_host(&mut self, host: ActionsDialogHost) {
        use crate::focus_coordinator::FocusRequest;

        let request = match host {
            ActionsDialogHost::ArgPrompt => FocusRequest::arg_prompt(),
            ActionsDialogHost::ChatPrompt => FocusRequest::chat_prompt(),
            ActionsDialogHost::EditorPrompt => FocusRequest::editor_prompt(),
            ActionsDialogHost::FormPrompt => FocusRequest::form_prompt(),
            ActionsDialogHost::DivPrompt => FocusRequest::div_prompt(),
            ActionsDialogHost::TermPrompt => FocusRequest::term_prompt(),
            ActionsDialogHost::WebcamPrompt => FocusRequest::div_prompt(),
            ActionsDialogHost::AcpChat => FocusRequest::chat_prompt(),
            ActionsDialogHost::MainList
            | ActionsDialogHost::FileSearch
            | ActionsDialogHost::ClipboardHistory
            | ActionsDialogHost::EmojiPicker
            | ActionsDialogHost::AppLauncher => FocusRequest::main_filter(),
        };

        self.focus_coordinator.request(request);
        self.sync_coordinator_to_legacy();
    }

    /// Check if the actions popup was closed very recently (within 300ms).
    ///
    /// This guards against a race where clicking the footer ⌘K button causes
    /// the actions window's activation observer to close the dialog (deferred)
    /// before the click handler fires `toggle_actions`. Without this debounce
    /// the toggle would see the dialog as closed and immediately reopen it.
    pub(crate) fn was_actions_recently_closed(&self) -> bool {
        const ACTIONS_CLOSE_DEBOUNCE: std::time::Duration =
            std::time::Duration::from_millis(300);
        self.actions_closed_at
            .map(|t| t.elapsed() < ACTIONS_CLOSE_DEBOUNCE)
            .unwrap_or(false)
    }

    /// Close the actions popup and restore focus based on host type.
    ///
    /// This centralizes close behavior, ensuring cx.notify() is always called
    /// and focus is correctly restored based on which prompt hosted the dialog.
    pub(crate) fn close_actions_popup(
        &mut self,
        host: ActionsDialogHost,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let overlay_depth_before_on_close = self.focus_coordinator.overlay_depth();
        let on_close_callback = self
            .actions_dialog
            .as_ref()
            .and_then(|dialog| dialog.read(cx).on_close.clone());

        if let Some(on_close) = on_close_callback {
            logging::log(
                "ACTIONS",
                &format!(
                    "ACTIONS_CLOSE_POPUP invoking on_close callback (host={:?}, overlay_depth_before={})",
                    host, overlay_depth_before_on_close
                ),
            );
            on_close(cx);
        }

        let overlay_depth_after_on_close = self.focus_coordinator.overlay_depth();
        let callback_restored_focus = overlay_depth_after_on_close < overlay_depth_before_on_close;

        self.show_actions_popup = false;
        self.actions_closed_at = Some(std::time::Instant::now());
        self.actions_dialog = None;
        self.resync_filter_input_after_actions_if_needed(window, cx);

        // Close the separate actions window if open
        // This ensures consistent behavior whether closing via Cmd+K, Escape, backdrop click,
        // or any other close mechanism
        if is_actions_window_open() {
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                });
            })
            .detach();
        }

        // Use coordinator to pop overlay and restore previous focus.
        // Skip pop when the dialog callback already restored focus to avoid double-pop.
        if !callback_restored_focus {
            self.pop_focus_overlay(cx);
        }

        self.request_focus_restore_for_actions_host(host);

        // Apply restored focus immediately rather than deferring to next render.
        // pop_focus_overlay sets pending_focus to the saved target (e.g. ChatPrompt).
        // Applying it now avoids race conditions with the async window close.
        if !self.apply_pending_focus(window, cx) {
            // Fallback: focus app root if no pending focus was applied
            window.focus(&self.focus_handle, cx);
        }
        logging::log(
            "FOCUS",
            &format!(
                "Actions popup closed (host={:?}), focus restored via coordinator",
                host
            ),
        );
        cx.notify();
    }
}

#[cfg(test)]
mod close_actions_popup_regression_tests {
    use std::fs;

    #[test]
    fn test_close_actions_popup_invokes_on_close_before_clearing_dialog_state() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");
        let close_fn_start = source
            .find("pub(crate) fn close_actions_popup")
            .expect("close_actions_popup function not found");
        let close_fn = &source[close_fn_start..];

        let on_close_pos = close_fn
            .find("on_close(cx);")
            .expect("close_actions_popup must invoke on_close callback");
        let clear_dialog_pos = close_fn
            .find("self.actions_dialog = None;")
            .expect("close_actions_popup must clear actions_dialog state");

        assert!(
            on_close_pos < clear_dialog_pos,
            "close_actions_popup must invoke on_close before clearing actions_dialog state"
        );
    }

    #[test]
    fn test_close_actions_popup_resyncs_filter_input_after_clearing_dialog_state() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");
        let close_fn_start = source
            .find("pub(crate) fn close_actions_popup")
            .expect("close_actions_popup function not found");
        let close_fn = &source[close_fn_start..];

        let clear_dialog_pos = close_fn
            .find("self.actions_dialog = None;")
            .expect("close_actions_popup must clear actions_dialog state");
        let resync_pos = close_fn
            .find("self.resync_filter_input_after_actions_if_needed(window, cx);")
            .expect("close_actions_popup must resync canonical filter input state");

        assert!(
            clear_dialog_pos < resync_pos,
            "close_actions_popup must resync filter input after clearing actions dialog state"
        );
    }

    #[test]
    fn test_close_actions_popup_notifies_after_focus_restore_paths() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");
        let close_fn_start = source
            .find("pub(crate) fn close_actions_popup")
            .expect("close_actions_popup function not found");
        let close_fn = &source[close_fn_start..];

        let fallback_focus_pos = close_fn
            .find("window.focus(&self.focus_handle, cx);")
            .expect("close_actions_popup must keep fallback root focus");
        let notify_pos = close_fn
            .find("cx.notify();")
            .expect("close_actions_popup must notify after closing popup");

        assert!(
            fallback_focus_pos < notify_pos,
            "close_actions_popup must notify after focus restore paths complete"
        );
    }

    #[test]
    fn test_close_actions_popup_restores_host_focus_before_apply_pending_focus() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");
        let close_fn_start = source
            .find("pub(crate) fn close_actions_popup")
            .expect("close_actions_popup function not found");
        let close_fn = &source[close_fn_start..];

        let host_restore_pos = close_fn
            .find("self.request_focus_restore_for_actions_host(host);")
            .expect("close_actions_popup must request host-specific focus restore");
        let apply_pending_pos = close_fn
            .find("if !self.apply_pending_focus(window, cx) {")
            .expect("close_actions_popup must apply pending focus after host restore");

        assert!(
            host_restore_pos < apply_pending_pos,
            "close_actions_popup should request host-specific focus before applying pending focus"
        );
    }

    #[test]
    fn test_actions_host_focus_restore_maps_prompt_hosts() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");
        let helper_start = source
            .find("fn request_focus_restore_for_actions_host")
            .expect("request_focus_restore_for_actions_host function not found");
        let helper_fn = &source[helper_start..];

        for expected in [
            "ActionsDialogHost::ArgPrompt => FocusRequest::arg_prompt()",
            "ActionsDialogHost::ChatPrompt => FocusRequest::chat_prompt()",
            "ActionsDialogHost::EditorPrompt => FocusRequest::editor_prompt()",
            "ActionsDialogHost::FormPrompt => FocusRequest::form_prompt()",
            "ActionsDialogHost::DivPrompt => FocusRequest::div_prompt()",
            "ActionsDialogHost::TermPrompt => FocusRequest::term_prompt()",
            "ActionsDialogHost::WebcamPrompt => FocusRequest::div_prompt()",
            "ActionsDialogHost::MainList",
            "ActionsDialogHost::FileSearch",
            "ActionsDialogHost::ClipboardHistory",
            "ActionsDialogHost::EmojiPicker",
            "ActionsDialogHost::AppLauncher",
            "FocusRequest::main_filter()",
        ] {
            assert!(
                helper_fn.contains(expected),
                "request_focus_restore_for_actions_host should include mapping fragment: {}",
                expected
            );
        }
    }
}

#[cfg(test)]
mod actions_dialog_wiring_regression_tests {
    use std::fs;

    #[test]
    fn render_script_list_routes_popup_keys_before_generic_cmd_shortcuts() {
        let source = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("Failed to read src/render_script_list/mod.rs");

        let route_pos = source
            .find("this.route_key_to_actions_dialog(")
            .expect("render_script_list must use the shared actions router");
        let cmd_pos = source
            .find("if has_cmd")
            .expect("render_script_list cmd shortcut block not found");

        assert!(
            route_pos < cmd_pos,
            "render_script_list must route popup keys before generic Cmd shortcuts"
        );
    }

    #[test]
    fn route_key_to_actions_dialog_notifies_after_arrow_navigation() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");

        let route_start = source
            .find("pub(crate) fn route_key_to_actions_dialog")
            .expect("route_key_to_actions_dialog not found");
        let route_fn = &source[route_start..];

        let up_start = route_fn
            .find("if is_key_up(key)")
            .expect("up branch missing");
        let down_start = route_fn
            .find("if is_key_down(key)")
            .expect("down branch missing");
        let jump_start = route_fn
            .find("let is_home = key.eq_ignore_ascii_case(\"home\")")
            .expect("jump-key section missing");

        assert!(
            route_fn[up_start..down_start]
                .contains("crate::actions::notify_actions_window(cx);"),
            "up branch must notify the actions window"
        );
        assert!(
            route_fn[down_start..jump_start]
                .contains("crate::actions::notify_actions_window(cx);"),
            "down branch must notify the actions window"
        );
    }

    #[test]
    fn route_key_to_actions_dialog_handles_cmd_k_close() {
        let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
            .expect("Failed to read src/app_impl/actions_dialog.rs");

        assert!(
            source.contains("key.eq_ignore_ascii_case(\"k\")")
                && source.contains("self.close_actions_popup(host, window, cx);"),
            "shared actions router should close the popup on Cmd+K"
        );
    }

    #[test]
    fn render_script_list_has_no_duplicate_popup_handler() {
        let source = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("Failed to read src/render_script_list/mod.rs");

        // The old inline popup handler used this pattern - it should be gone
        assert!(
            !source.contains("if this.show_actions_popup {"),
            "render_script_list must not contain a duplicate inline popup key handler"
        );
    }
}
