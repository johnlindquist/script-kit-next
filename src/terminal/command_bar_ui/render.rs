use gpui::{
    div, prelude::*, px, rgb, rgba, App, BoxShadow, Context, ElementId, FocusHandle, Focusable,
    Hsla, Render, SharedString, Window,
};

use super::*;

impl TerminalCommandBar {
    /// Create box shadow for the popup
    fn create_popup_shadow(&self) -> Vec<BoxShadow> {
        let is_dark = self.theme.has_dark_colors();
        let shadow_color = if is_dark {
            rgba(0x00000080)
        } else {
            rgba(0x00000040)
        };

        vec![
            BoxShadow {
                color: Hsla::from(shadow_color),
                offset: gpui::point(px(0.), px(4.)),
                blur_radius: px(16.),
                spread_radius: px(0.),
            },
            BoxShadow {
                color: Hsla::from(rgba(0x00000020)),
                offset: gpui::point(px(0.), px(2.)),
                blur_radius: px(8.),
                spread_radius: px(0.),
            },
        ]
    }

    /// Parse shortcut string into individual keycaps
    pub(super) fn parse_shortcut_keycaps(shortcut: &str) -> Vec<String> {
        shortcut.chars().map(|c| c.to_string()).collect()
    }

    /// Render a single keycap
    fn render_keycap(&self, key: &str, is_dark: bool) -> impl IntoElement {
        let keycap_bg = if is_dark {
            rgba(0xffffff18)
        } else {
            rgba(0x00000010)
        };
        let keycap_text = rgb(self.theme.colors.text.dimmed);
        let keycap_border = if is_dark {
            rgba(0xffffff20)
        } else {
            rgba(0x00000020)
        };

        div()
            .h(px(KEYCAP_HEIGHT))
            .min_w(px(KEYCAP_MIN_WIDTH))
            .px(px(6.))
            .flex()
            .items_center()
            .justify_center()
            .bg(keycap_bg)
            .border_1()
            .border_color(keycap_border)
            .rounded(px(4.))
            .text_xs()
            .text_color(keycap_text)
            .child(key.to_string())
    }

    /// Render a command item
    fn render_command_item(
        &self,
        idx: usize,
        cmd: &TerminalCommandItem,
        is_selected: bool,
    ) -> impl IntoElement {
        let is_dark = self.theme.has_dark_colors();

        let opacity = self.theme.get_opacity();
        let selected_bg = {
            let alpha = (opacity.selected * 255.0) as u32;
            rgba((self.theme.colors.accent.selected_subtle << 8) | alpha)
        };

        let hover_bg = {
            let alpha = (opacity.hover * 255.0) as u32;
            rgba((self.theme.colors.accent.selected_subtle << 8) | alpha)
        };

        let primary_text = rgb(self.theme.colors.text.primary);
        let secondary_text = rgb(self.theme.colors.text.secondary);

        let shortcut_element = cmd.shortcut.as_ref().map(|shortcut| {
            let keycaps = Self::parse_shortcut_keycaps(shortcut);
            div()
                .flex()
                .flex_row()
                .gap(px(2.))
                .children(keycaps.into_iter().map(|k| self.render_keycap(&k, is_dark)))
        });

        div()
            .id(ElementId::NamedInteger("cmd-item".into(), idx as u64))
            .h(px(COMMAND_ITEM_HEIGHT))
            .w_full()
            .px(px(ITEM_PADDING_X))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .rounded(px(8.))
            .mx(px(8.))
            .when(is_selected, |d| d.bg(selected_bg))
            .when(!is_selected, |d| d.hover(|d| d.bg(hover_bg)))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        div()
                            .text_sm()
                            .text_color(primary_text)
                            .child(cmd.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(secondary_text)
                            .child(cmd.description.clone()),
                    ),
            )
            .when_some(shortcut_element, |d, shortcut| d.child(shortcut))
    }
}

impl Focusable for TerminalCommandBar {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TerminalCommandBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let dialog_bg = {
            let opacity = if self.theme.is_vibrancy_enabled() {
                0.50
            } else {
                0.95
            };
            let alpha = (opacity * 255.0) as u32;
            rgba((self.theme.colors.background.main << 8) | alpha)
        };

        let border_color = rgba((self.theme.colors.ui.border << 8) | 0x80);
        let hint_text_color = rgb(self.theme.colors.text.dimmed);
        let input_text_color = rgb(self.theme.colors.text.primary);
        let accent_color = rgb(self.theme.colors.accent.selected);

        let search_display = if self.search_text.is_empty() {
            SharedString::from("Search commands...")
        } else {
            SharedString::from(self.search_text.clone())
        };

        let item_count = self.filtered_indices.len();
        let content_height = Self::command_list_height(item_count);

        let items: Vec<_> = self
            .filtered_indices
            .iter()
            .enumerate()
            .filter_map(|(visual_idx, &cmd_idx)| {
                self.commands.get(cmd_idx).map(|cmd| {
                    let is_selected = visual_idx == self.selected_index;
                    self.render_command_item(visual_idx, cmd, is_selected)
                })
            })
            .collect();

        let separator_color = border_color;

        div()
            .track_focus(&self.focus_handle)
            .w(px(COMMAND_BAR_WIDTH))
            .max_h(px(COMMAND_BAR_MAX_HEIGHT))
            .bg(dialog_bg)
            .border_1()
            .border_color(border_color)
            .rounded(px(POPUP_RADIUS))
            .shadow(self.create_popup_shadow())
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                div()
                    .id("command-bar-list")
                    .h(px(content_height))
                    .overflow_y_scroll()
                    .py(px(8.))
                    .when(self.filtered_indices.is_empty(), |d| {
                        d.child(
                            div()
                                .h(px(COMMAND_ITEM_HEIGHT))
                                .w_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(hint_text_color)
                                .child("No commands match"),
                        )
                    })
                    .when(!self.filtered_indices.is_empty(), |d| d.children(items)),
            )
            .child(
                div()
                    .h(px(SEARCH_INPUT_HEIGHT))
                    .w_full()
                    .px(px(ITEM_PADDING_X))
                    .border_t_1()
                    .border_color(separator_color)
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_sm()
                            .text_color(if self.search_text.is_empty() {
                                hint_text_color
                            } else {
                                input_text_color
                            })
                            .when(self.search_text.is_empty(), |d| {
                                d.child(
                                    div()
                                        .w(px(2.))
                                        .h(px(16.))
                                        .mr(px(2.))
                                        .rounded(px(1.))
                                        .when(self.cursor_visible, |d| d.bg(accent_color)),
                                )
                            })
                            .child(search_display)
                            .when(!self.search_text.is_empty(), |d| {
                                d.child(
                                    div()
                                        .w(px(2.))
                                        .h(px(16.))
                                        .ml(px(2.))
                                        .rounded(px(1.))
                                        .when(self.cursor_visible, |d| d.bg(accent_color)),
                                )
                            }),
                    ),
            )
    }
}
