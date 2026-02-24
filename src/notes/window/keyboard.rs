use super::*;
use crate::ui_foundation::{
    is_key_backspace, is_key_delete, is_key_down, is_key_enter, is_key_escape, is_key_tab,
    is_key_up,
};

impl NotesApp {
    pub(super) fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.hide_mouse_cursor(cx);

        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;

        if self.command_bar.is_open() {
            match key {
                key if is_key_escape(key) => {
                    self.close_actions_panel(window, cx);
                    cx.stop_propagation();
                    return;
                }
                key if is_key_up(key) => {
                    self.command_bar.select_prev(cx);
                    cx.stop_propagation();
                    return;
                }
                key if is_key_down(key) => {
                    self.command_bar.select_next(cx);
                    cx.stop_propagation();
                    return;
                }
                key if is_key_enter(key) => {
                    if let Some(action_id) = self.command_bar.execute_selected_action(cx) {
                        self.execute_action(&action_id, window, cx);
                    }
                    cx.stop_propagation();
                    return;
                }
                key if is_key_backspace(key) || is_key_delete(key) => {
                    self.command_bar.handle_backspace(cx);
                    cx.stop_propagation();
                    return;
                }
                _ => {
                    if !modifiers.platform && !modifiers.control && !modifiers.alt {
                        if let Some(ch) = key.chars().next() {
                            let ch = ch.to_ascii_lowercase();
                            if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_'
                            {
                                self.command_bar.handle_char(ch, cx);
                                cx.stop_propagation();
                                return;
                            }
                        }
                    }
                    if modifiers.platform && key.eq_ignore_ascii_case("k") {
                        self.close_actions_panel(window, cx);
                        cx.stop_propagation();
                        return;
                    }
                }
            }
            return;
        }

        if self.show_actions_panel && self.actions_panel.is_some() {
            if is_key_escape(key) || (modifiers.platform && key.eq_ignore_ascii_case("k")) {
                self.close_actions_panel(window, cx);
                return;
            }

            if let Some(ref panel) = self.actions_panel {
                match key {
                    key if is_key_up(key) => {
                        panel.update(cx, |panel, cx| panel.move_up(cx));
                    }
                    key if is_key_down(key) => {
                        panel.update(cx, |panel, cx| panel.move_down(cx));
                    }
                    key if is_key_enter(key) => {
                        panel.update(cx, |panel, _| panel.submit_selected());
                    }
                    key if is_key_backspace(key) => {
                        panel.update(cx, |panel, cx| panel.handle_backspace(cx));
                    }
                    _ => {
                        let handled_navigation =
                            panel.update(cx, |panel, cx| panel.handle_navigation_key(key, cx));
                        if handled_navigation {
                            return;
                        }
                        if let Some(ref key_char) = event.keystroke.key_char {
                            if let Some(ch) = key_char.chars().next() {
                                if !ch.is_control() {
                                    panel.update(cx, |panel, cx| {
                                        panel.handle_char(ch, cx);
                                    });
                                }
                            }
                        }
                    }
                }
            }

            return;
        }

        if self.note_switcher.is_open() {
            match key {
                key if is_key_escape(key) => {
                    self.close_browse_panel(window, cx);
                    cx.stop_propagation();
                    return;
                }
                key if is_key_up(key) => {
                    self.note_switcher.select_prev(cx);
                    cx.stop_propagation();
                    return;
                }
                key if is_key_down(key) => {
                    self.note_switcher.select_next(cx);
                    cx.stop_propagation();
                    return;
                }
                key if is_key_enter(key) => {
                    if let Some(action_id) = self.note_switcher.execute_selected_action(cx) {
                        self.execute_note_switcher_action(&action_id, window, cx);
                    }
                    cx.stop_propagation();
                    return;
                }
                key if is_key_backspace(key) || is_key_delete(key) => {
                    self.note_switcher.handle_backspace(cx);
                    cx.stop_propagation();
                    return;
                }
                _ => {
                    if !modifiers.platform && !modifiers.control && !modifiers.alt {
                        if let Some(ch) = key.chars().next() {
                            let ch = ch.to_ascii_lowercase();
                            if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_'
                            {
                                self.note_switcher.handle_char(ch, cx);
                                cx.stop_propagation();
                                return;
                            }
                        }
                    }
                    if modifiers.platform && key.eq_ignore_ascii_case("p") {
                        self.close_browse_panel(window, cx);
                        cx.stop_propagation();
                        return;
                    }
                }
            }
            return;
        }

        if is_key_escape(key) {
            if self.show_shortcuts_help {
                self.show_shortcuts_help = false;
                cx.notify();
                return;
            }
            if self.show_actions_panel || self.command_bar.is_open() {
                self.close_actions_panel(window, cx);
                return;
            }
            if self.note_switcher.is_open() {
                self.close_browse_panel(window, cx);
                return;
            }
            if self.show_search {
                self.toggle_search(window, cx);
                return;
            }
            if self.focus_mode {
                self.toggle_focus_mode(cx);
                return;
            }
            if self.view_mode == NotesViewMode::Trash {
                self.set_view_mode(NotesViewMode::AllNotes, window, cx);
                return;
            }
            let wb = window.window_bounds();
            crate::window_state::save_window_from_gpui(crate::window_state::WindowRole::Notes, wb);
            window.remove_window();
            return;
        }

        if is_key_tab(key) && !modifiers.platform && !modifiers.control && !modifiers.alt {
            if modifiers.shift {
                self.outdent_line(window, cx);
            } else {
                self.indent_at_cursor(window, cx);
            }
            cx.stop_propagation();
            return;
        }

        if modifiers.alt && !modifiers.platform {
            match key {
                key if is_key_up(key) => {
                    if modifiers.shift {
                        self.duplicate_line(false, window, cx);
                    } else {
                        self.move_line_up(window, cx);
                    }
                    cx.stop_propagation();
                    return;
                }
                key if is_key_down(key) => {
                    if modifiers.shift {
                        self.duplicate_line(true, window, cx);
                    } else {
                        self.move_line_down(window, cx);
                    }
                    cx.stop_propagation();
                    return;
                }
                _ => {}
            }
        }

        if modifiers.control && modifiers.shift && key.eq_ignore_ascii_case("k") {
            self.delete_current_line(window, cx);
            cx.stop_propagation();
            return;
        }

        if modifiers.platform {
            match key {
                key if key.eq_ignore_ascii_case("k") => {
                    if self.command_bar.is_open() || self.show_actions_panel {
                        self.close_actions_panel(window, cx);
                    } else {
                        self.open_actions_panel(window, cx);
                    }
                }
                key if key.eq_ignore_ascii_case("p") => {
                    if modifiers.shift {
                        self.toggle_preview(window, cx);
                    } else {
                        self.close_actions_panel(window, cx);
                        if self.note_switcher.is_open() {
                            self.close_browse_panel(window, cx);
                        } else {
                            self.open_browse_panel(window, cx);
                        }
                    }
                }
                key if key.eq_ignore_ascii_case("f") => {
                    if modifiers.shift {
                        self.toggle_search(window, cx);
                        cx.stop_propagation();
                    } else {
                        self.editor_state.update(cx, |state, cx| {
                            state.focus(window, cx);
                        });
                        // Route Search directly through this window so Notes find works
                        // even when other app windows are open.
                        window.dispatch_action(Box::new(Search), cx);
                        cx.stop_propagation();
                    }
                }
                key if key.eq_ignore_ascii_case("n") => {
                    if modifiers.shift {
                        self.create_note_from_clipboard(window, cx);
                    } else {
                        self.create_note(window, cx);
                    }
                }
                key if key.eq_ignore_ascii_case("t") => {
                    if modifiers.shift {
                        if self.view_mode == NotesViewMode::Trash {
                            self.set_view_mode(NotesViewMode::AllNotes, window, cx);
                        } else {
                            self.set_view_mode(NotesViewMode::Trash, window, cx);
                        }
                        cx.stop_propagation();
                    }
                }
                key if key.eq_ignore_ascii_case("w") => {
                    self.command_bar.close_app(cx);
                    self.note_switcher.close_app(cx);
                    let wb = window.window_bounds();
                    crate::window_state::save_window_from_gpui(
                        crate::window_state::WindowRole::Notes,
                        wb,
                    );
                    window.remove_window();
                }
                "." => {
                    if modifiers.shift {
                        self.toggle_blockquote(window, cx);
                    } else {
                        self.toggle_focus_mode(cx);
                    }
                    cx.stop_propagation();
                }
                key if key.eq_ignore_ascii_case("s") => {
                    if modifiers.shift {
                        self.cycle_sort_mode(cx);
                        cx.stop_propagation();
                    }
                }
                key if key.eq_ignore_ascii_case("z") => {
                    if self.view_mode == NotesViewMode::Trash && self.selected_note_id.is_some() {
                        self.restore_note(window, cx);
                        cx.stop_propagation();
                    }
                }
                key if key.eq_ignore_ascii_case("d") => {
                    if modifiers.shift {
                        self.insert_date_time(window, cx);
                        cx.stop_propagation();
                    } else {
                        self.duplicate_selected_note(window, cx);
                    }
                }
                key if key.eq_ignore_ascii_case("x") => {
                    if modifiers.shift {
                        self.insert_formatting("~~", "~~", window, cx);
                        cx.stop_propagation();
                    }
                }
                key if key.eq_ignore_ascii_case("l") => {
                    if modifiers.shift {
                        self.toggle_checklist(window, cx);
                        cx.stop_propagation();
                    } else {
                        self.select_current_line(window, cx);
                        cx.stop_propagation();
                    }
                }
                "-" => {
                    if modifiers.shift {
                        self.insert_horizontal_rule(window, cx);
                        cx.stop_propagation();
                    }
                }
                key if key.eq_ignore_ascii_case("h") => {
                    if modifiers.shift {
                        self.cycle_heading(window, cx);
                        cx.stop_propagation();
                    }
                }
                key if key.eq_ignore_ascii_case("v") => {
                    if self.try_smart_paste(window, cx) {
                        cx.stop_propagation();
                    }
                }
                key if key.eq_ignore_ascii_case("c") => {
                    if modifiers.shift {
                        self.copy_as_markdown(cx);
                        cx.stop_propagation();
                    }
                }
                key if key.eq_ignore_ascii_case("e") => {
                    self.insert_formatting("`", "`", window, cx);
                    cx.stop_propagation();
                }
                "/" => {
                    self.toggle_shortcuts_help(cx);
                    cx.stop_propagation();
                }
                key if key.eq_ignore_ascii_case("j") => {
                    self.join_lines(window, cx);
                    cx.stop_propagation();
                }
                key if key.eq_ignore_ascii_case("u") => {
                    if modifiers.shift {
                        self.transform_case(window, cx);
                        cx.stop_propagation();
                    }
                }
                key if key.eq_ignore_ascii_case("b") => {
                    self.insert_formatting("**", "**", window, cx)
                }
                key if key.eq_ignore_ascii_case("i") => {
                    if modifiers.shift {
                        self.toggle_pin_current_note(cx);
                    } else {
                        self.insert_formatting("_", "_", window, cx);
                    }
                }
                key if is_key_up(key) => {
                    let editor_is_focused = self
                        .editor_state
                        .read(cx)
                        .focus_handle(cx)
                        .is_focused(window);
                    if !editor_is_focused {
                        if modifiers.shift {
                            self.select_first_note(window, cx);
                        } else {
                            self.select_prev_note(window, cx);
                        }
                        cx.stop_propagation();
                    }
                }
                key if is_key_down(key) => {
                    let editor_is_focused = self
                        .editor_state
                        .read(cx)
                        .focus_handle(cx)
                        .is_focused(window);
                    if !editor_is_focused {
                        if modifiers.shift {
                            self.select_last_note(window, cx);
                        } else {
                            self.select_next_note(window, cx);
                        }
                        cx.stop_propagation();
                    }
                }
                "[" => {
                    self.navigate_back(window, cx);
                    cx.stop_propagation();
                }
                "]" => {
                    self.navigate_forward(window, cx);
                    cx.stop_propagation();
                }
                key if is_key_backspace(key) || is_key_delete(key) => {
                    if self.selected_note_id.is_some() {
                        self.delete_selected_note(cx);
                        if let Some(id) = self.selected_note_id {
                            let content = self
                                .notes
                                .iter()
                                .find(|n| n.id == id)
                                .map(|n| n.content.clone())
                                .unwrap_or_default();
                            let content_len = content.len();
                            self.editor_state.update(cx, |state, cx| {
                                state.set_value(&content, window, cx);
                                state.set_selection(content_len, content_len, window, cx);
                            });
                        } else {
                            self.editor_state.update(cx, |state, cx| {
                                state.set_value("", window, cx);
                            });
                        }
                        cx.stop_propagation();
                    }
                }
                "7" if modifiers.shift => {
                    self.toggle_numbered_list(window, cx);
                    cx.stop_propagation();
                }
                "8" if modifiers.shift => {
                    self.toggle_bullet_list(window, cx);
                    cx.stop_propagation();
                }
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                    if !modifiers.shift {
                        if let Ok(num) = key.parse::<usize>() {
                            self.select_pinned_note_by_index(num - 1, window, cx);
                            cx.stop_propagation();
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
