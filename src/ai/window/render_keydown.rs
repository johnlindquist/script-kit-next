use super::*;
use crate::ui_foundation::{
    is_key_backspace, is_key_delete, is_key_down, is_key_enter, is_key_escape, is_key_k,
    is_key_tab, is_key_up,
};

impl AiApp {
    pub(super) fn handle_root_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Hide mouse cursor on any keyboard interaction
        self.hide_mouse_cursor(cx);

        // Handle keyboard shortcuts
        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;

        let no_system_modifiers = !modifiers.platform && !modifiers.alt && !modifiers.control;

        // Setup-card keyboard navigation when no providers are configured.
        // Skip while API key input is visible so Enter/typing route to the input.
        let in_setup_mode = self.available_models.is_empty() && !self.showing_api_key_input;

        if no_system_modifiers && in_setup_mode {
            match key {
                k if is_key_tab(k) => {
                    if modifiers.shift {
                        self.move_setup_button_focus(-1, cx);
                    } else {
                        self.move_setup_button_focus(1, cx);
                    }
                    window.activate_window();
                    cx.stop_propagation();
                    return;
                }
                k if is_key_up(k) => {
                    self.move_setup_button_focus(-1, cx);
                    window.activate_window();
                    cx.stop_propagation();
                    return;
                }
                k if is_key_down(k) => {
                    self.move_setup_button_focus(1, cx);
                    window.activate_window();
                    cx.stop_propagation();
                    return;
                }
                k if is_key_enter(k) => {
                    match self.setup_button_focus_index {
                        0 => self.show_api_key_input(window, cx),
                        1 => self.enable_claude_code(window, cx),
                        _ => {}
                    }
                    window.activate_window();
                    cx.stop_propagation();
                    return;
                }
                _ => {}
            }
        }

        // Handle command bar navigation when it's open
        // This routes all relevant keys to the CommandBar
        // CRITICAL: Must stop propagation to prevent Input from consuming the keys
        if self.command_bar.is_open() {
            match key {
                k if is_key_up(k) => {
                    self.command_bar_select_prev(cx);
                    cx.stop_propagation(); // Prevent Input from handling
                    return;
                }
                k if is_key_down(k) => {
                    self.command_bar_select_next(cx);
                    cx.stop_propagation(); // Prevent Input from handling
                    return;
                }
                k if is_key_enter(k) => {
                    self.execute_command_bar_action(window, cx);
                    cx.stop_propagation(); // Prevent Input from handling
                    return;
                }
                k if is_key_escape(k) => {
                    self.hide_command_bar(cx);
                    cx.stop_propagation(); // Prevent further handling
                    return;
                }
                k if is_key_backspace(k) || is_key_delete(k) => {
                    self.command_bar_handle_backspace(cx);
                    cx.stop_propagation(); // Prevent Input from handling
                    return;
                }
                _ => {
                    // Handle printable characters for search (when no modifiers)
                    if !modifiers.platform && !modifiers.control && !modifiers.alt {
                        // Get the character from the keystroke
                        if let Some(ch) = key.chars().next() {
                            if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_'
                            {
                                self.command_bar_handle_char(ch, cx);
                                cx.stop_propagation(); // Prevent Input from handling
                                return;
                            }
                        }
                    }
                }
            }
            // Don't fall through to other handlers when command bar is open
            return;
        }

        // Handle presets dropdown navigation
        if self.showing_presets_dropdown {
            match key {
                k if is_key_up(k) => {
                    self.presets_select_prev(cx);
                    cx.stop_propagation();
                    return;
                }
                k if is_key_down(k) => {
                    self.presets_select_next(cx);
                    cx.stop_propagation();
                    return;
                }
                k if is_key_enter(k) => {
                    self.create_chat_with_preset(window, cx);
                    cx.stop_propagation();
                    return;
                }
                k if is_key_escape(k) => {
                    self.hide_presets_dropdown(cx);
                    cx.stop_propagation();
                    return;
                }
                _ => {}
            }
        }

        // Handle new chat dropdown navigation (Raycast-style CommandBar)
        if self.new_chat_command_bar.is_open() {
            match key {
                k if is_key_up(k) => {
                    self.new_chat_command_bar_select_prev(cx);
                    cx.stop_propagation();
                    return;
                }
                k if is_key_down(k) => {
                    self.new_chat_command_bar_select_next(cx);
                    cx.stop_propagation();
                    return;
                }
                k if is_key_enter(k) => {
                    self.execute_new_chat_action(window, cx);
                    cx.stop_propagation();
                    return;
                }
                k if is_key_escape(k) => {
                    self.hide_new_chat_command_bar(cx);
                    cx.stop_propagation();
                    return;
                }
                _ => {
                    // Let printable characters fall through to the search input
                }
            }
        }

        // Handle attachments picker
        if self.showing_attachments_picker && is_key_escape(key) {
            self.hide_attachments_picker(cx);
            cx.stop_propagation();
            return;
        }

        // Cmd+1-4: submit welcome suggestion cards (only when welcome screen is visible)
        if modifiers.platform
            && !modifiers.shift
            && !modifiers.alt
            && !modifiers.control
            && self.current_messages.is_empty()
            && !self.is_streaming
            && !self.available_models.is_empty()
        {
            let idx = match key {
                "1" => Some(0),
                "2" => Some(1),
                "3" => Some(2),
                "4" => Some(3),
                _ => None,
            };
            if let Some(i) = idx {
                let (title, desc) = WELCOME_SUGGESTIONS[i];
                let prompt = format!("{} {}", title, desc);
                info!(
                    shortcut = i + 1,
                    prompt = %prompt,
                    "welcome_suggestion_shortcut"
                );
                self.input_state.update(cx, |state, cx| {
                    state.set_value(&prompt, window, cx);
                });
                self.submit_message(window, cx);
                cx.stop_propagation();
                return;
            }
        }

        // platform modifier = Cmd on macOS, Ctrl on Windows/Linux
        if modifiers.platform {
            match key {
                // Cmd+K to toggle command bar (like Raycast)
                k if is_key_k(k) => {
                    if self.command_bar.is_open() {
                        self.hide_command_bar(cx);
                    } else {
                        self.hide_all_dropdowns(cx);
                        self.show_command_bar(window, cx);
                    }
                }
                // Cmd+N for new chat (with Shift for presets)
                "n" => {
                    if modifiers.shift {
                        // Cmd+Shift+N opens presets dropdown
                        self.hide_all_dropdowns(cx);
                        self.show_presets_dropdown(window, cx);
                    } else {
                        self.new_conversation(window, cx);
                    }
                }
                // Cmd+Shift+F to focus search (expand sidebar if collapsed)
                "f" => {
                    if modifiers.shift {
                        // Expand sidebar if collapsed before focusing search
                        if self.sidebar_collapsed {
                            self.sidebar_collapsed = false;
                        }
                        self.hide_all_dropdowns(cx);
                        self.focus_search(window, cx);
                        cx.stop_propagation();
                    }
                }
                k if is_key_enter(k) => self.submit_message(window, cx),
                // Cmd+\ to toggle sidebar (like Raycast)
                "\\" | "backslash" => self.toggle_sidebar(cx),
                // Cmd+B also toggles sidebar (common convention)
                "b" => self.toggle_sidebar(cx),
                // Cmd+V for paste - check for images first
                "v" => {
                    // Try to paste an image; if not found, let normal text paste happen
                    // We don't need to prevent the event since the Input handles text paste
                    self.handle_paste_for_image(cx);
                }
                // Cmd+L to focus input (standard shortcut)
                "l" => {
                    self.focus_input(window, cx);
                    cx.stop_propagation();
                }
                // Cmd+Shift+C to copy last assistant response
                "c" => {
                    if modifiers.shift {
                        self.copy_last_assistant_response(cx);
                        cx.stop_propagation();
                    }
                }
                // Cmd+[ to navigate to previous chat, Cmd+] to next chat
                "[" | "bracketleft" => {
                    self.navigate_chat(-1, window, cx);
                    cx.stop_propagation();
                }
                "]" | "bracketright" => {
                    self.navigate_chat(1, window, cx);
                    cx.stop_propagation();
                }
                // Cmd+Shift+Backspace to delete current chat
                k if is_key_backspace(k) || is_key_delete(k) => {
                    if modifiers.shift {
                        self.delete_current_chat(cx);
                        cx.stop_propagation();
                    }
                }
                // Cmd+Shift+E to export chat to clipboard as markdown
                "e" => {
                    if modifiers.shift {
                        self.export_chat_to_clipboard(cx);
                        cx.stop_propagation();
                    }
                }
                // Cmd+/ to toggle keyboard shortcuts overlay
                "/" | "slash" => {
                    self.toggle_shortcuts_overlay(cx);
                    cx.stop_propagation();
                }
                // Cmd+W closes the AI window (standard macOS pattern)
                "w" => {
                    // Save bounds before closing
                    let wb = window.window_bounds();
                    crate::window_state::save_window_from_gpui(
                        crate::window_state::WindowRole::Ai,
                        wb,
                    );
                    window.remove_window();
                }
                _ => {}
            }
        }

        // Escape closes shortcuts overlay
        if is_key_escape(key) && self.showing_shortcuts_overlay {
            self.showing_shortcuts_overlay = false;
            cx.notify();
            cx.stop_propagation();
            return;
        }

        // Escape clears active search before falling through to close/stop handlers
        if is_key_escape(key) && !self.search_query.is_empty() {
            info!(
                previous_query = %self.search_query,
                "escape_clear_search"
            );
            self.search_query.clear();
            self.search_generation += 1;
            self.search_snippets.clear();
            self.search_matched_title.clear();
            self.chats = crate::ai::storage::get_all_chats().unwrap_or_default();
            // Clear the search input text
            self.search_state.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
            // Return focus to chat input
            self.focus_input(window, cx);
            cx.notify();
            cx.stop_propagation();
            return;
        }

        // Up arrow in empty input: edit last user message
        if is_key_up(key) && self.input_state.read(cx).value().is_empty() && !self.is_streaming {
            self.edit_last_user_message(window, cx);
            cx.stop_propagation();
            return;
        }

        // Escape cancels editing mode
        if is_key_escape(key) && self.editing_message_id.is_some() {
            self.editing_message_id = None;
            self.input_state.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
            cx.notify();
            cx.stop_propagation();
            return;
        }

        // Escape cancels rename
        if is_key_escape(key) && self.renaming_chat_id.is_some() {
            self.cancel_rename(cx);
            cx.stop_propagation();
            return;
        }

        // Escape stops streaming if active
        if is_key_escape(key) && self.is_streaming {
            self.stop_streaming(cx);
            cx.stop_propagation();
            return;
        }

        // Escape closes API key input (back to setup card)
        if is_key_escape(key) && self.showing_api_key_input {
            self.hide_api_key_input(window, cx);
            cx.stop_propagation();
            return;
        }

        // Escape closes any open dropdown
        if is_key_escape(key)
            && (self.command_bar.is_open()
                || self.showing_presets_dropdown
                || self.showing_attachments_picker)
        {
            self.hide_all_dropdowns(cx);
        }
    }
}
