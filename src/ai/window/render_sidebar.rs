use super::types::*;
use super::*;
use crate::theme::opacity::{
    OPACITY_ACCENT_MEDIUM, OPACITY_DANGER_BG, OPACITY_DISABLED, OPACITY_HOVER, OPACITY_SELECTED,
    OPACITY_STRONG,
};

impl AiApp {
    pub(super) fn sidebar_list_splice_plan(
        old_count: usize,
        item_count: usize,
    ) -> Option<(std::ops::Range<usize>, usize)> {
        if item_count > old_count {
            Some((old_count..old_count, item_count - old_count))
        } else if item_count < old_count {
            Some((item_count..old_count, 0))
        } else {
            None
        }
    }

    pub(super) fn render_sidebar_toggle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Use opacity to indicate state - dimmed when collapsed
        let icon_color = if self.sidebar_collapsed {
            cx.theme().muted_foreground.opacity(OPACITY_SELECTED)
        } else {
            cx.theme().muted_foreground
        };

        div()
            .id("sidebar-toggle")
            .flex()
            .items_center()
            .gap(S1)
            .child(
                div()
                    .id("sidebar-toggle-icon")
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(S6)
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|s| s.bg(cx.theme().muted.opacity(OPACITY_HOVER)))
                    .tooltip(|window, cx| {
                        Tooltip::new("Toggle sidebar")
                            .key_binding(gpui::Keystroke::parse("cmd-b").ok().map(Kbd::new))
                            .build(window, cx)
                    })
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.toggle_sidebar(cx);
                        }),
                    )
                    .child(
                        svg()
                            .external_path(LocalIconName::Sidebar.external_path())
                            .size(ICON_MD)
                            .text_color(icon_color),
                    ),
            )
    }

    pub(super) fn sync_sidebar_list_item_count(&mut self, item_count: usize) {
        let old_count = self.sidebar_list_state.item_count();
        if let Some((range, insert_count)) = Self::sidebar_list_splice_plan(old_count, item_count) {
            self.sidebar_list_state.splice(range, insert_count);
        }
    }

    /// Reusable sidebar body: search + chat list with empty states.
    /// Used by both the full sidebar and the mini history overlay.
    pub(super) fn render_sidebar_body(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_id = self.selected_chat_id;
        let sidebar_rows = build_sidebar_rows_for_chats(&self.chats);
        self.sync_sidebar_list_item_count(sidebar_rows.len());
        let sidebar_entity = cx.entity();
        let sidebar_list = list(self.sidebar_list_state.clone(), move |ix, _window, cx| {
            let row = sidebar_rows.get(ix).copied();
            sidebar_entity.update(cx, |this, cx| match row {
                Some(SidebarRow::Header { group, is_first }) => this
                    .render_sidebar_group_header(group, is_first, cx)
                    .into_any_element(),
                Some(SidebarRow::Chat { chat_id }) => this
                    .chats
                    .iter()
                    .find(|chat| chat.id == chat_id)
                    .map(|chat| {
                        this.render_chat_item(chat, selected_id, cx)
                            .into_any_element()
                    })
                    .unwrap_or_else(|| div().into_any_element()),
                None => div().into_any_element(),
            })
        })
        .with_sizing_behavior(ListSizingBehavior::Infer)
        .size_full()
        .px(SIDEBAR_INSET_X)
        .pb(S2);

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            // Search header
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .px(SIDEBAR_INSET_X)
                    .pb(S2)
                    .gap(S2)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S2)
                            .child(div().flex_1().child(self.render_search(cx))),
                    )
                    .when(
                        !self.search_query.is_empty() && !self.chats.is_empty(),
                        |d| {
                            let count = self.chats.len();
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(
                                        cx.theme().muted_foreground.opacity(OPACITY_ACCENT_MEDIUM),
                                    )
                                    .px_1()
                                    .child(format!(
                                        "{} {}",
                                        count,
                                        if count == 1 { "result" } else { "results" }
                                    )),
                            )
                        },
                    ),
            )
            // Scrollable chat list
            .child(div().relative().flex_1().min_h_0().overflow_hidden().child(
                if self.chats.is_empty() && !self.search_query.is_empty() {
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .flex_1()
                        .py_8()
                        .gap(S2)
                        .child(
                            svg()
                                .external_path(LocalIconName::MagnifyingGlass.external_path())
                                .size(S6)
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_HOVER)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_SELECTED))
                                .child("No chats found"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_HOVER))
                                .child(format!("No results for \"{}\"", self.search_query)),
                        )
                        .child(
                            div()
                                .mt(S2)
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_HOVER))
                                .child("Press Esc to clear search"),
                        )
                        .into_any_element()
                } else if self.chats.is_empty() && self.search_query.is_empty() {
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .flex_1()
                        .py_8()
                        .gap_3()
                        .child(
                            svg()
                                .external_path(LocalIconName::MessageCircle.external_path())
                                .size(MINI_BTN_SIZE)
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_DANGER_BG)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_SELECTED))
                                .child("No conversations yet"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(S1)
                                .child(
                                    div()
                                        .px(S2)
                                        .py(S1)
                                        .rounded(R_SM)
                                        .bg(cx.theme().muted.opacity(OPACITY_DISABLED))
                                        .text_xs()
                                        .text_color(
                                            cx.theme().muted_foreground.opacity(OPACITY_SELECTED),
                                        )
                                        .child("\u{2318}N"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(
                                            cx.theme().muted_foreground.opacity(OPACITY_DISABLED),
                                        )
                                        .child("to start a new chat"),
                                ),
                        )
                        .into_any_element()
                } else {
                    div()
                        .relative()
                        .size_full()
                        .child(sidebar_list)
                        .vertical_scrollbar(&self.sidebar_list_state)
                        .into_any_element()
                },
            ))
    }

    /// Render the chats sidebar with date groupings
    pub(super) fn render_sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // If sidebar is collapsed, completely hide it (Raycast-style)
        if self.sidebar_collapsed {
            return div().w(S0).h_full().into_any_element();
        }

        div()
            .flex()
            .flex_col()
            .w(SIDEBAR_W)
            .h_full()
            // NO .bg() - let vibrancy show through from root
            .border_r_1()
            .border_color(cx.theme().sidebar_border)
            // New chat + presets header (only in full sidebar, not overlay)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .w_full()
                    .px(SIDEBAR_INSET_X)
                    .gap_1()
                    .child(
                        div()
                            .id("new-chat-tooltip-trigger")
                            .flex()
                            .items_center()
                            .gap(S1)
                            .hover(|el| el)
                            .tooltip(|window, cx| {
                                Tooltip::new("New chat")
                                    .key_binding(gpui::Keystroke::parse("cmd-n").ok().map(Kbd::new))
                                    .build(window, cx)
                            })
                            .child(
                                Button::new("new-chat")
                                    .ghost()
                                    .xsmall()
                                    .icon(IconName::Plus)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.new_conversation(window, cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .id("presets-trigger")
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(SP_9)
                            .rounded(R_SM)
                            .cursor_pointer()
                            .hover(|el| el.bg(cx.theme().sidebar_accent.opacity(OPACITY_SELECTED)))
                            .tooltip(|window, cx| {
                                Tooltip::new("New chat with preset")
                                    .key_binding(
                                        gpui::Keystroke::parse("cmd-shift-n").ok().map(Kbd::new),
                                    )
                                    .build(window, cx)
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                if this.showing_presets_dropdown {
                                    this.hide_presets_dropdown(cx);
                                } else {
                                    this.hide_all_dropdowns(cx);
                                    this.show_presets_dropdown(window, cx);
                                }
                            }))
                            .child(
                                Icon::new(IconName::ChevronDown).size(ICON_XS).text_color(
                                    cx.theme().sidebar_foreground.opacity(OPACITY_STRONG),
                                ),
                            ),
                    ),
            )
            // Shared sidebar body (search + chat list)
            .child(self.render_sidebar_body(cx))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::AiApp;

    #[test]
    fn test_sidebar_list_splice_plan_appends_when_item_count_grows() {
        let plan = AiApp::sidebar_list_splice_plan(3, 6);
        assert_eq!(plan, Some((3..3, 3)));
    }

    #[test]
    fn test_sidebar_list_splice_plan_truncates_when_item_count_shrinks() {
        let plan = AiApp::sidebar_list_splice_plan(6, 3);
        assert_eq!(plan, Some((3..6, 0)));
    }

    #[test]
    fn test_sidebar_list_splice_plan_returns_none_when_count_unchanged() {
        let plan = AiApp::sidebar_list_splice_plan(4, 4);
        assert_eq!(plan, None);
    }
}
