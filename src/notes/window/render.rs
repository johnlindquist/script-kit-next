use super::*;

impl NotesApp {
    fn process_render_side_effects(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.detect_manual_resize(window);
        self.drain_pending_action(window, cx);
        self.drain_pending_browse_actions(window, cx);
        self.maybe_update_theme_cache();
        self.maybe_persist_bounds(window);

        if self.should_save_now() {
            tracing::debug!(
                surface = "notes_window",
                action = "autosave",
                has_selected_note = self.selected_note_id.is_some(),
                has_unsaved_changes = self.has_unsaved_changes,
                show_actions_panel = self.show_actions_panel,
                show_search = self.show_search,
                preview_enabled = self.preview_enabled,
                focus_mode = self.focus_mode,
                "ui_render_decision"
            );
            self.save_current_note();
        }
    }
}

impl Render for NotesApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mouse_cursor_hidden = self.mouse_cursor_hidden;

        self.process_render_side_effects(window, cx);

        let show_actions =
            self.show_actions_panel && self.actions_panel.is_some() && !self.command_bar.is_open();

        let vibrancy_bg =
            crate::ui_foundation::get_vibrancy_background(&crate::theme::get_cached_theme());

        div()
            .id("notes-window-root")
            .flex()
            .flex_col()
            .size_full()
            .relative()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg))
            .text_color(cx.theme().foreground)
            .track_focus(&self.focus_handle)
            .when(mouse_cursor_hidden, |d| d.cursor(CursorStyle::None))
            .on_any_mouse_down(cx.listener(|this, _, window, cx| {
                if this.command_bar.is_open() {
                    this.close_actions_panel(window, cx);
                }
                if this.note_switcher.is_open() {
                    this.close_browse_panel(window, cx);
                }
                if confirm::is_confirm_window_open() {
                    confirm::route_key_to_confirm_popup("escape", cx);
                }
            }))
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
            .children(gpui_component::Root::render_dialog_layer(window, cx))
            .children(gpui_component::Root::render_notification_layer(window, cx))
    }
}
