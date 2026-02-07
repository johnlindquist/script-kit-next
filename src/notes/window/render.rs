use super::*;

impl Render for NotesApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mouse_cursor_hidden = self.mouse_cursor_hidden;

        self.detect_manual_resize(window);
        self.drain_pending_action(window, cx);
        self.drain_pending_browse_actions(window, cx);
        self.maybe_update_theme_cache();
        self.maybe_persist_bounds(window);

        if self.should_save_now() {
            self.save_current_note();
        }

        let show_actions =
            self.show_actions_panel && self.actions_panel.is_some() && !self.command_bar.is_open();

        let vibrancy_bg = crate::ui_foundation::get_window_vibrancy_background();

        div()
            .id("notes-window-root")
            .flex()
            .flex_col()
            .size_full()
            .relative()
            .bg(vibrancy_bg)
            .text_color(cx.theme().foreground)
            .track_focus(&self.focus_handle)
            .when(mouse_cursor_hidden, |d| d.cursor(CursorStyle::None))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    if this.command_bar.is_open() {
                        this.close_actions_panel(window, cx);
                    }
                    if this.note_switcher.is_open() {
                        this.close_browse_panel(window, cx);
                    }
                }),
            )
            .on_hover(cx.listener(|this, hovered, _, cx| {
                if this.force_hovered {
                    return;
                }

                this.window_hovered = *hovered;
                cx.notify();
            }))
            .on_mouse_move(cx.listener(|this, _: &MouseMoveEvent, _, cx| {
                this.show_mouse_cursor(cx);
            }))
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                this.handle_key_down(event, window, cx);
            }))
            .child(self.render_editor(cx))
            .when(show_actions, |d| {
                d.child(self.render_actions_panel_overlay(cx))
            })
            .when(self.show_shortcuts_help, |d| {
                d.child(self.render_shortcuts_help(cx))
            })
    }
}
