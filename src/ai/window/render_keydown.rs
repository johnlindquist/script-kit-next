use super::*;

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
        let key = event.keystroke.key.to_lowercase();
        let modifiers = &event.keystroke.modifiers;

        let no_system_modifiers = !modifiers.platform && !modifiers.alt && !modifiers.control;

        // Setup-card keyboard navigation when no providers are configured.
        // Skip while API key input is visible so Enter/typing route to the input.
        let in_setup_mode = self.available_models.is_empty() && !self.showing_api_key_input;

        if no_system_modifiers && in_setup_mode {
            match key.as_str() {
                "tab" => {
                    if modifiers.shift {
                        self.move_setup_button_focus(-1, cx);
                    } else {
                        self.move_setup_button_focus(1, cx);
                    }
                    window.activate_window();
                    cx.stop_propagation();
                    return;
                }
                "up" | "arrowup" => {
                    self.move_setup_button_focus(-1, cx);
                    window.activate_window();
                    cx.stop_propagation();
                    return;
                }
                "down" | "arrowdown" => {
                    self.move_setup_button_focus(1, cx);
                    window.activate_window();
                    cx.stop_propagation();
                    return;
                }
                "enter" | "return" => {
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
            match key.as_str() {
                "up" | "arrowup" => {
                    self.command_bar_select_prev(cx);
                    cx.stop_propagation(); // Prevent Input from handling
                    return;
                }
                "down" | "arrowdown" => {
                    self.command_bar_select_next(cx);
                    cx.stop_propagation(); // Prevent Input from handling
                    return;
                }
                "enter" | "return" => {
                    self.execute_command_bar_action(window, cx);
                    cx.stop_propagation(); // Prevent Input from handling
                    return;
                }
                "escape" => {
                    self.hide_command_bar(cx);
                    cx.stop_propagation(); // Prevent further handling
                    return;
                }
                "backspace" | "delete" => {
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
            match key.as_str() {
                "up" | "arrowup" => {
                    self.presets_select_prev(cx);
                    cx.stop_propagation();
                    return;
                }
                "down" | "arrowdown" => {
                    self.presets_select_next(cx);
                    cx.stop_propagation();
                    return;
                }
                "enter" | "return" => {
                    self.create_chat_with_preset(window, cx);
                    cx.stop_propagation();
                    return;
                }
                "escape" => {
                    self.hide_presets_dropdown(cx);
                    cx.stop_propagation();
                    return;
                }
                _ => {}
            }
        }

        // Handle new chat dropdown navigation (Raycast-style CommandBar)
        if self.new_chat_command_bar.is_open() {
            match key.as_str() {
                "up" | "arrowup" => {
                    self.new_chat_command_bar_select_prev(cx);
                    cx.stop_propagation();
                    return;
                }
                "down" | "arrowdown" => {
                    self.new_chat_command_bar_select_next(cx);
                    cx.stop_propagation();
                    return;
                }
                "enter" | "return" => {
                    self.execute_new_chat_action(window, cx);
                    cx.stop_propagation();
                    return;
                }
                "escape" => {
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
        if self.showing_attachments_picker && key == "escape" {
            self.hide_attachments_picker(cx);
            cx.stop_propagation();
            return;
        }

        // platform modifier = Cmd on macOS, Ctrl on Windows/Linux
        if modifiers.platform {
            match key.as_str() {
                // Cmd+K to toggle command bar (like Raycast)
                "k" => {
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
                        self.create_chat(window, cx);
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
                "enter" | "return" => self.submit_message(window, cx),
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
                "backspace" | "delete" => {
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
        if key == "escape" && self.showing_shortcuts_overlay {
            self.showing_shortcuts_overlay = false;
            cx.notify();
            cx.stop_propagation();
            return;
        }

        // Up arrow in empty input: edit last user message
        if (key == "up" || key == "arrowup")
            && self.input_state.read(cx).value().is_empty()
            && !self.is_streaming
        {
            self.edit_last_user_message(window, cx);
            cx.stop_propagation();
            return;
        }

        // Escape cancels editing mode
        if key == "escape" && self.editing_message_id.is_some() {
            self.editing_message_id = None;
            self.input_state.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
            cx.notify();
            cx.stop_propagation();
            return;
        }

        // Escape cancels rename
        if key == "escape" && self.renaming_chat_id.is_some() {
            self.cancel_rename(cx);
            cx.stop_propagation();
            return;
        }

        // Escape stops streaming if active
        if key == "escape" && self.is_streaming {
            self.stop_streaming(cx);
            cx.stop_propagation();
            return;
        }

        // Escape closes API key input (back to setup card)
        if key == "escape" && self.showing_api_key_input {
            self.hide_api_key_input(window, cx);
            cx.stop_propagation();
            return;
        }

        // Escape closes any open dropdown
        if key == "escape"
            && (self.command_bar.is_open()
                || self.showing_presets_dropdown
                || self.showing_attachments_picker)
        {
            self.hide_all_dropdowns(cx);
        }
    }
}
