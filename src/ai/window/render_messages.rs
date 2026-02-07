use super::*;

impl AiApp {
    pub(super) fn sync_messages_list_and_scroll_to_bottom(&mut self) {
        let item_count = self.messages_list_item_count();
        let old_count = self.messages_list_state.item_count();
        if old_count != item_count {
            self.messages_list_state.splice(0..old_count, item_count);
        }
        // Only auto-scroll if user hasn't scrolled up
        if item_count > 0 && !self.user_has_scrolled_up {
            self.messages_list_state
                .scroll_to_reveal_item(item_count - 1);
        }
    }

    /// Force scroll to the bottom, regardless of user_has_scrolled_up.
    /// Used when user explicitly triggers scroll-to-bottom (clicking the pill
    /// or submitting a new message).
    pub(super) fn force_scroll_to_bottom(&mut self) {
        self.user_has_scrolled_up = false;
        let item_count = self.messages_list_item_count();
        let old_count = self.messages_list_state.item_count();
        if old_count != item_count {
            self.messages_list_state.splice(0..old_count, item_count);
        }
        if item_count > 0 {
            self.messages_list_state
                .scroll_to_reveal_item(item_count - 1);
        }
    }

    /// Total item count for the messages list: messages + optional streaming row.
    pub(super) fn messages_list_item_count(&self) -> usize {
        self.current_messages.len()
            + if self.is_streaming { 1 } else { 0 }
            + if self.streaming_error.is_some() { 1 } else { 0 }
    }

    /// Render the messages area using a virtualized list with native-style scrollbar.
    pub(super) fn render_messages(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let msg_count = self.current_messages.len();
        let is_streaming = self.is_streaming;
        let has_error = self.streaming_error.is_some();

        // Virtualized list: only renders visible messages + overdraw band.
        // Item indices: 0..msg_count = saved messages, msg_count = streaming/error row.
        let messages_list = list(self.messages_list_state.clone(), move |ix, _window, cx| {
            entity.update(cx, |this, cx| {
                if ix < msg_count {
                    let is_last_assistant = !is_streaming
                        && !has_error
                        && ix == msg_count - 1
                        && this.current_messages[ix].role == MessageRole::Assistant;
                    // Compact header when consecutive messages share the same role
                    let is_continuation = ix > 0
                        && this.current_messages[ix].role == this.current_messages[ix - 1].role;
                    let msg_el = this
                        .render_message(&this.current_messages[ix], is_continuation, cx)
                        .into_any_element();
                    if is_last_assistant {
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .child(msg_el)
                            .child(this.render_message_actions(cx))
                            .into_any_element()
                    } else {
                        msg_el
                    }
                } else if is_streaming && ix == msg_count {
                    this.render_streaming_content(cx).into_any_element()
                } else if has_error {
                    this.render_streaming_error(cx).into_any_element()
                } else {
                    div().into_any_element()
                }
            })
        })
        .with_sizing_behavior(ListSizingBehavior::Infer)
        .size_full()
        .px(SP_9)
        .py(SP_8);

        // Track user scroll: show pill when user scrolls up (during streaming or with many messages)
        let show_scroll_pill =
            self.user_has_scrolled_up && (self.is_streaming || self.current_messages.len() > 3);
        let total_items = self.messages_list_item_count();

        // Wrap in a relative container with a native-style scrollbar overlay.
        // The scrollbar uses ListState's ScrollbarHandle impl for position tracking.
        div()
            .relative()
            .size_full()
            // Detect user scroll via scroll wheel events
            .on_scroll_wheel(
                cx.listener(move |this, event: &ScrollWheelEvent, _window, cx| {
                    let delta_y = event.delta.pixel_delta(px(1.0)).y;
                    if delta_y > px(0.) {
                        // Scrolling up - only notify when state actually changes
                        // (avoids redundant re-renders during momentum scroll)
                        if !this.user_has_scrolled_up {
                            this.user_has_scrolled_up = true;
                            cx.notify();
                        }
                    } else if delta_y < px(0.) {
                        // Scrolling down - check if near bottom to reset flag
                        // Use logical_scroll_top to determine position
                        let scroll_top = this.messages_list_state.logical_scroll_top().item_ix;
                        // If we're within 2 items of the bottom, consider it "at bottom"
                        if total_items > 0
                            && scroll_top + 3 >= total_items
                            && this.user_has_scrolled_up
                        {
                            this.user_has_scrolled_up = false;
                            cx.notify();
                        }
                    }
                }),
            )
            .child(messages_list)
            .vertical_scrollbar(&self.messages_list_state)
            // Floating "scroll to bottom" pill when user has scrolled up during streaming
            .when(show_scroll_pill, |el| {
                el.child(
                    div()
                        .id("scroll-to-bottom-pill")
                        .absolute()
                        .bottom(SP_6)
                        .left_0()
                        .right_0()
                        .flex()
                        .justify_center()
                        .child(
                            div()
                                .id("scroll-pill-btn")
                                .flex()
                                .items_center()
                                .gap(SP_2)
                                .px(SP_6)
                                .py(SP_2)
                                .rounded_full()
                                .bg(cx.theme().accent.opacity(0.85))
                                .text_color(cx.theme().accent_foreground)
                                .cursor_pointer()
                                .shadow_md()
                                .hover(|s| s.bg(cx.theme().accent))
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.force_scroll_to_bottom();
                                    cx.notify();
                                }))
                                .child(
                                    svg()
                                        .external_path(LocalIconName::ChevronDown.external_path())
                                        .size(ICON_SM)
                                        .text_color(cx.theme().accent_foreground),
                                )
                                .child({
                                    let pill_label: SharedString = if self.is_streaming {
                                        let new_words =
                                            self.streaming_content.split_whitespace().count();
                                        if new_words > 5 {
                                            format!("\u{2193} ~{} new words", new_words).into()
                                        } else {
                                            "New content below".into()
                                        }
                                    } else {
                                        "Scroll to bottom".into()
                                    };
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(pill_label)
                                }),
                        ),
                )
            })
    }
}
