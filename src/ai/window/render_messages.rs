use super::*;
use crate::theme::opacity::OPACITY_NEAR_FULL;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MessageGroupingState {
    pub(super) is_continuation: bool,
    pub(super) uses_continuation_spacing_after: bool,
}

pub(super) fn message_grouping_state(messages: &[Message], ix: usize) -> MessageGroupingState {
    let role = messages[ix].role;
    let is_continuation = ix > 0 && role == messages[ix - 1].role;
    let uses_continuation_spacing_after = ix + 1 < messages.len() && role == messages[ix + 1].role;

    MessageGroupingState {
        is_continuation,
        uses_continuation_spacing_after,
    }
}

fn is_messages_list_at_bottom(total_items: usize, scroll_top_item_ix: usize) -> bool {
    // `logical_scroll_top` reports the topmost visible item's index.
    // We're at bottom when that topmost item is the last item.
    total_items == 0 || scroll_top_item_ix.saturating_add(1) >= total_items
}

impl AiApp {
    pub(super) fn sync_messages_list_and_scroll_to_bottom(&mut self) {
        let item_count = self.messages_list_item_count();
        let old_count = self.messages_list_state.item_count();
        if old_count != item_count {
            self.messages_list_state.splice(0..old_count, item_count);
        } else if self.is_streaming && item_count > 0 {
            // Content within the streaming item changed — invalidate its cached
            // height so the list re-measures it on the next layout pass.
            let last = item_count - 1;
            self.messages_list_state.splice(last..item_count, 1);
        }
        // Only auto-scroll if user hasn't scrolled up.
        // Use scroll_to with a large offset_in_item to reach the actual bottom
        // of the last (growing) item, not just "reveal" it.
        if item_count > 0 && !self.user_has_scrolled_up {
            self.messages_list_state.scroll_to(ListOffset {
                item_ix: item_count - 1,
                offset_in_item: px(1_000_000.),
            });
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
        } else if item_count > 0 {
            // Invalidate last item height so re-measure picks up new content.
            let last = item_count - 1;
            self.messages_list_state.splice(last..item_count, 1);
        }
        if item_count > 0 {
            self.messages_list_state.scroll_to(ListOffset {
                item_ix: item_count - 1,
                offset_in_item: px(1_000_000.),
            });
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
        let is_mini = self.window_mode.is_mini();

        // Virtualized list: only renders visible messages + overdraw band.
        // Item indices: 0..msg_count = saved messages, msg_count = streaming/error row.
        let messages_list = list(self.messages_list_state.clone(), move |ix, _window, cx| {
            entity.update(cx, |this, cx| {
                if ix < msg_count {
                    let is_last_assistant = !is_streaming
                        && !has_error
                        && ix == msg_count - 1
                        && this.current_messages[ix].role == MessageRole::Assistant;
                    let grouping = message_grouping_state(&this.current_messages, ix);
                    let msg_el = this
                        .render_message(
                            &this.current_messages[ix],
                            grouping.is_continuation,
                            grouping.uses_continuation_spacing_after,
                            cx,
                        )
                        .into_any_element();
                    if is_last_assistant && is_mini {
                        div()
                            .group("mini-last-assistant")
                            .flex()
                            .flex_col()
                            .w_full()
                            .child(msg_el)
                            .child(this.render_message_actions(cx))
                            .into_any_element()
                    } else if is_last_assistant {
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
        .px(MSG_PX)
        .py(S4);

        // Track user scroll: show pill when user scrolls up (during streaming or with many messages)
        let show_scroll_pill =
            self.user_has_scrolled_up && (self.is_streaming || self.current_messages.len() > 3);

        // Wrap in a relative container with a native-style scrollbar overlay.
        // The scrollbar uses ListState's ScrollbarHandle impl for position tracking.
        div()
            .relative()
            .size_full()
            // Detect user scroll via scroll wheel events
            .on_scroll_wheel(
                cx.listener(move |this, event: &ScrollWheelEvent, _window, cx| {
                    let delta_y = event.delta.pixel_delta(px(1.0)).y;
                    if delta_y > S0 {
                        // Scrolling up - only notify when state actually changes
                        // (avoids redundant re-renders during momentum scroll)
                        if !this.user_has_scrolled_up {
                            this.user_has_scrolled_up = true;
                            cx.notify();
                        }
                    } else if delta_y < S0 {
                        // Scrolling down - check if at true bottom to resume auto-scroll.
                        // `logical_scroll_top` gives the topmost visible item index, so
                        // reaching the last item means we're at bottom.
                        let scroll_top = this.messages_list_state.logical_scroll_top();
                        let item_count = this.messages_list_item_count();
                        let at_bottom = is_messages_list_at_bottom(item_count, scroll_top.item_ix);
                        if at_bottom && this.user_has_scrolled_up {
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
                        .bottom(S3)
                        .left_0()
                        .right_0()
                        .flex()
                        .justify_center()
                        .child(
                            div()
                                .id("scroll-pill-btn")
                                .flex()
                                .items_center()
                                .gap(S1)
                                .px(S3)
                                .py(S1)
                                .rounded_full()
                                .bg(cx.theme().accent.opacity(OPACITY_NEAR_FULL))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_messages_list_at_bottom_returns_true_for_empty_list() {
        assert!(is_messages_list_at_bottom(0, 0));
    }

    #[test]
    fn test_is_messages_list_at_bottom_returns_false_when_not_on_last_item() {
        assert!(!is_messages_list_at_bottom(5, 2));
    }

    #[test]
    fn test_is_messages_list_at_bottom_returns_true_when_scroll_top_is_past_last_item() {
        assert!(is_messages_list_at_bottom(5, 5));
        assert!(is_messages_list_at_bottom(5, 6));
    }

    #[test]
    fn test_is_messages_list_at_bottom_returns_true_when_top_item_is_last_item() {
        assert!(
            is_messages_list_at_bottom(5, 4),
            "topmost visible last item should be treated as at bottom"
        );
    }

    #[test]
    fn test_message_grouping_state_decouples_continuation_and_spacing_after() {
        let chat_id = ChatId::new();
        let messages = vec![
            Message::user(chat_id, "u1"),
            Message::user(chat_id, "u2"),
            Message::assistant(chat_id, "a1"),
            Message::assistant(chat_id, "a2"),
            Message::system(chat_id, "s1"),
        ];

        assert_eq!(
            message_grouping_state(&messages, 0),
            MessageGroupingState {
                is_continuation: false,
                uses_continuation_spacing_after: true,
            }
        );
        assert_eq!(
            message_grouping_state(&messages, 1),
            MessageGroupingState {
                is_continuation: true,
                uses_continuation_spacing_after: false,
            }
        );
        assert_eq!(
            message_grouping_state(&messages, 2),
            MessageGroupingState {
                is_continuation: false,
                uses_continuation_spacing_after: true,
            }
        );
        assert_eq!(
            message_grouping_state(&messages, 3),
            MessageGroupingState {
                is_continuation: true,
                uses_continuation_spacing_after: false,
            }
        );
        assert_eq!(
            message_grouping_state(&messages, 4),
            MessageGroupingState {
                is_continuation: false,
                uses_continuation_spacing_after: false,
            }
        );
    }
}
