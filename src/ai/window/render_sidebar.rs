use super::*;

impl AiApp {
    pub(super) fn render_sidebar_toggle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Use opacity to indicate state - dimmed when collapsed
        let icon_color = if self.sidebar_collapsed {
            cx.theme().muted_foreground.opacity(0.5)
        } else {
            cx.theme().muted_foreground
        };

        div()
            .id("sidebar-toggle")
            .flex()
            .items_center()
            .justify_center()
            .size(px(24.))
            .rounded_md()
            .cursor_pointer()
            .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
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
                    .size(px(16.))
                    .text_color(icon_color),
            )
    }

    pub(super) fn sync_sidebar_list_item_count(&mut self, item_count: usize) {
        let old_count = self.sidebar_list_state.item_count();
        if old_count != item_count {
            self.sidebar_list_state.splice(0..old_count, item_count);
        }
    }

    /// Render the chats sidebar with date groupings
    pub(super) fn render_sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // If sidebar is collapsed, completely hide it (Raycast-style)
        // The toggle button is absolutely positioned in the main container
        if self.sidebar_collapsed {
            return div().w(px(0.)).h_full().into_any_element();
        }

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
        .px_2()
        .pb_2();

        // Build a custom sidebar with date groupings using divs
        // This gives us more control over the layout than SidebarGroup
        div()
            .flex()
            .flex_col()
            .w(SIDEBAR_W)
            .h_full()
            // NO .bg() - let vibrancy show through from root
            .border_r_1()
            .border_color(cx.theme().sidebar_border)
            // Spacer for titlebar height (toggle button is now absolutely positioned in main container)
            .child(div().h(TITLEBAR_H))
            // Header with new chat button and search
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .px_2()
                    .pb_2()
                    .gap_2()
                    // New chat button row with preset dropdown option
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_end()
                            .w_full()
                            .gap_1()
                            // New chat button - use Button's native tooltip (⌘N)
                            .child(
                                Button::new("new-chat")
                                    .ghost()
                                    .xsmall()
                                    .icon(IconName::Plus)
                                    .tooltip("New chat (⌘N)")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.create_chat(window, cx);
                                    })),
                            )
                            // Presets dropdown trigger - use svg directly for better tooltip control
                            .child(
                                div()
                                    .id("presets-trigger")
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .size(SP_9)
                                    .rounded(RADIUS_SM)
                                    .cursor_pointer()
                                    .hover(|el| el.bg(cx.theme().sidebar_accent.opacity(0.5)))
                                    .tooltip(|window, cx| {
                                        Tooltip::new("New chat with preset")
                                            .key_binding(
                                                gpui::Keystroke::parse("cmd-shift-n")
                                                    .ok()
                                                    .map(Kbd::new),
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
                                        Icon::new(IconName::ChevronDown)
                                            .size(ICON_XS)
                                            .text_color(cx.theme().sidebar_foreground.opacity(0.7)),
                                    ),
                            ),
                    )
                    .child(self.render_search(cx))
                    // Search result count (shown when there's an active search query with results)
                    .when(
                        !self.search_query.is_empty() && !self.chats.is_empty(),
                        |d| {
                            let count = self.chats.len();
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(0.6))
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
            // Scrollable chat list with date groups
            // Note: overflow_y_scrollbar() wraps the element in a Scrollable container
            // min_h_0() is critical for flex containers - without it, the element won't shrink
            // below its content size and scrolling won't work properly
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0() // Critical: allows flex child to shrink and enable scrolling
                    .overflow_hidden()
                    .child(if self.chats.is_empty() && !self.search_query.is_empty() {
                        // Empty state when search has no results
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .flex_1()
                            .py_8()
                            .gap_2()
                            .child(
                                svg()
                                    .external_path(LocalIconName::MagnifyingGlass.external_path())
                                    .size(px(24.))
                                    .text_color(cx.theme().muted_foreground.opacity(0.3)),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground.opacity(0.5))
                                    .child("No chats found"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(0.3))
                                    .child(format!("No results for \"{}\"", self.search_query)),
                            )
                            .into_any_element()
                    } else if self.chats.is_empty() && self.search_query.is_empty() {
                        // Empty state when no chats exist at all
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
                                    .size(px(28.))
                                    .text_color(cx.theme().muted_foreground.opacity(0.2)),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground.opacity(0.5))
                                    .child("No conversations yet"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(4.))
                                    .child(
                                        div()
                                            .px(px(5.))
                                            .py(px(1.))
                                            .rounded(px(3.))
                                            .bg(cx.theme().muted.opacity(0.4))
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground.opacity(0.5))
                                            .child("\u{2318}N"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground.opacity(0.4))
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
                    }),
            )
            .into_any_element()
    }
}
