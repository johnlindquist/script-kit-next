use super::*;

impl NotesApp {
    /// Render the ACP surface with a thin Notes-owned titlebar containing a
    /// mode switch so the user can toggle back to the Notes editor.
    pub(super) fn render_acp_surface(&self, cx: &mut Context<Self>) -> AnyElement {
        let muted_color = cx.theme().muted_foreground;
        let accent_color = cx.theme().accent;
        let window_hovered = self.window_hovered || self.force_hovered;

        let titlebar = div()
            .id("notes-acp-titlebar")
            .flex()
            .items_center()
            .h(px(TITLEBAR_HEIGHT))
            .px_3()
            // Traffic light padding on the left.
            .child(div().w(px(TITLEBAR_TRAFFIC_LIGHT_W)).flex_shrink_0())
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    // Notes/ACP switch — two clickable labels.
                    .child(
                        div()
                            .id("notes-switch-notes")
                            .cursor_pointer()
                            .text_sm()
                            .text_color(muted_color.opacity(OPACITY_MUTED))
                            .hover(|s| s.text_color(accent_color))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.switch_to_notes_surface(window, cx);
                            }))
                            .child("Notes"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted_color.opacity(OPACITY_SUBTLE))
                            .child("/"),
                    )
                    .child(
                        div()
                            .id("notes-switch-acp")
                            .text_sm()
                            .text_color(accent_color)
                            .child("ACP"),
                    ),
            )
            .child(
                div()
                    .w(px(TITLEBAR_ICONS_W))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap_2()
                    .when(window_hovered, |d| {
                        d.child(
                            div()
                                .id("acp-titlebar-actions-icon")
                                .min_w(px(MIN_TARGET_SIZE))
                                .min_h(px(MIN_TARGET_SIZE))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(muted_color.opacity(OPACITY_MUTED))
                                .cursor_pointer()
                                .hover(|s| s.text_color(muted_color))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.toggle_acp_actions(window, cx);
                                }))
                                .child("⌘"),
                        )
                    }),
            );

        let (acp_body, acp_footer) = if let Some(ref acp_entity) = self.embedded_acp_chat {
            let acp_footer = {
                let view = acp_entity.read(cx);
                view.build_external_host_footer(acp_entity.downgrade(), cx)
            };

            (
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .child(acp_entity.clone())
                    .into_any_element(),
                acp_footer,
            )
        } else {
            (
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .text_color(muted_color.opacity(OPACITY_MUTED))
                    .child("ACP is loading…")
                    .into_any_element(),
                None,
            )
        };

        div()
            .flex_1()
            .flex()
            .flex_col()
            .h_full()
            .child(titlebar)
            .child(acp_body)
            .when_some(acp_footer, |d, footer| d.child(footer))
            .into_any_element()
    }

    fn process_render_side_effects(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.detect_manual_resize(window);
        self.drain_pending_action(window, cx);
        self.drain_pending_browse_actions(window, cx);
        self.drain_pending_focus(window, cx);
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

        let in_acp_mode = self.surface_mode == NotesSurfaceMode::Acp;

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
            // Surface dispatch: Notes editor or embedded ACP chat.
            .when(!in_acp_mode, |d| d.child(self.render_editor(cx)))
            .when(in_acp_mode, |d| d.child(self.render_acp_surface(cx)))
            .when(show_actions && !in_acp_mode, |d| {
                d.child(self.render_actions_panel_overlay(cx))
            })
            .when(self.show_shortcuts_help && !in_acp_mode, |d| {
                d.child(self.render_shortcuts_help(cx))
            })
            .children(gpui_component::Root::render_dialog_layer(window, cx))
            .children(gpui_component::Root::render_notification_layer(window, cx))
    }
}
