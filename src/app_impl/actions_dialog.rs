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

        if is_key_up(key) {
            dialog.update(cx, |d, cx| d.move_up(cx));
            return ActionsRoute::Handled;
        }

        if is_key_down(key) {
            dialog.update(cx, |d, cx| d.move_down(cx));
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
    pub(crate) fn normalize_display_shortcut(display: &str) -> String {
        let mut parts: Vec<&str> = Vec::new();
        let mut key_char: Option<char> = None;

        for ch in display.chars() {
            match ch {
                '⌘' => parts.push("cmd"),
                '⌃' => parts.push("ctrl"),
                '⌥' => parts.push("alt"),
                '⇧' => parts.push("shift"),
                '↵' => key_char = Some('e'), // Enter - map to 'enter' below
                '⎋' => key_char = Some('`'), // Escape placeholder
                '⇥' => key_char = Some('t'), // Tab placeholder
                '⌫' => key_char = Some('b'), // Backspace placeholder
                _ => key_char = Some(ch),
            }
        }

        // Sort modifiers alphabetically (matches keystroke_to_shortcut order)
        parts.sort();

        let mut result = parts.join("+");
        if let Some(k) = key_char {
            if !result.is_empty() {
                result.push('+');
            }
            // Handle special keys
            match k {
                'e' if display.contains('↵') => result.push_str("enter"),
                '`' if display.contains('⎋') => result.push_str("escape"),
                't' if display.contains('⇥') => result.push_str("tab"),
                'b' if display.contains('⌫') => result.push_str("backspace"),
                _ => result.push_str(&k.to_lowercase().to_string()),
            }
        }

        result
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

    /// Close the actions popup and restore focus based on host type.
    ///
    /// This centralizes close behavior, ensuring cx.notify() is always called
    /// and focus is correctly restored based on which prompt hosted the dialog.
    ///
    /// NOTE: The `host` parameter is now deprecated. Focus restoration is handled
    /// automatically by the FocusCoordinator's overlay stack. The host is kept
    /// for logging purposes only.
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
        self.actions_dialog = None;
        self.resync_filter_input_after_actions_if_needed(window, cx);

        // Close the separate actions window if open
        // This ensures consistent behavior whether closing via Cmd+K, Escape, backdrop click,
        // or any other close mechanism
        if is_actions_window_open() {
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                })
                .ok();
            })
            .detach();
        }

        // Use coordinator to pop overlay and restore previous focus.
        // Skip pop when the dialog callback already restored focus to avoid double-pop.
        if !callback_restored_focus {
            self.pop_focus_overlay(cx);
        }

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
}
