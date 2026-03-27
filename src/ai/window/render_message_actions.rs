use super::*;
use crate::theme::opacity::{
    OPACITY_HOVER, OPACITY_ICON_MUTED, OPACITY_SELECTED, OPACITY_STRONG, OPACITY_TEXT_MUTED,
};
use gpui::{Div, Stateful, Svg};

/// Shortcut hint opacity: dimmer than muted text
const SHORTCUT_OPACITY: f32 = OPACITY_TEXT_MUTED * 0.6;

impl AiApp {
    pub(super) fn render_message_actions(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Mini mode: height-collapsed hint strip, revealed on hover of the
        // "mini-last-assistant" group wrapper in render_messages.rs.
        // No layout space is reserved when hidden.
        if self.window_mode.is_mini() {
            let mini_style = mini_ai_chat_style();
            if !mini_style.show_action_hints {
                return div().id("message-actions-mini").into_any_element();
            }
            let has_assistant = self
                .current_messages
                .iter()
                .any(|m| m.role == MessageRole::Assistant);
            let hint: SharedString = if has_assistant {
                "\u{2318}K Actions \u{00b7} \u{2318}\u{21e7}C Copy \u{00b7} \u{2318}\u{21e7}E Export \u{00b7} \u{2318}N New".into()
            } else {
                "\u{2318}K Actions \u{00b7} \u{2318}N New".into()
            };
            return div()
                .id("message-actions-mini")
                .w_full()
                .overflow_hidden()
                .max_h(MINI_ACTION_HINT_COLLAPSED_H)
                .opacity(0.)
                .group_hover("mini-last-assistant", |s| {
                    s.max_h(MINI_ACTION_HINT_REVEALED_H).opacity(1.0)
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .pl(MINI_MESSAGE_PX)
                        .pt(SP_1)
                        .pb(SP_1)
                        .text_xs()
                        .text_color(
                            cx.theme()
                                .muted_foreground
                                .opacity(mini_style.action_hint_reveal_opacity),
                        )
                        .child(hint),
                )
                .into_any_element();
        }

        let muted_fg = cx.theme().muted_foreground;
        let success = cx.theme().success;
        let muted_bg = cx.theme().muted;
        let mouse_mode = self.input_mode == InputMode::Mouse;

        // Show "Generated in Xs · ~N words" until the next message is sent.
        // The label persists so users can reference generation speed context.
        // It is cleared when last_streaming_completed_at is reset (on next submit).
        let completion_label: Option<String> =
            self.last_streaming_completed_at.and_then(|_completed_at| {
                self.last_streaming_duration.map(|dur| {
                    let time_label = {
                        let secs = dur.as_secs();
                        if secs < 1 {
                            format!("{}ms", dur.as_millis())
                        } else {
                            format!("{}s", secs)
                        }
                    };
                    let word_count = self
                        .current_messages
                        .last()
                        .filter(|m| m.role == MessageRole::Assistant)
                        .map(|m| m.content.split_whitespace().count())
                        .unwrap_or(0);
                    if word_count > 0 {
                        let secs = dur.as_secs_f64();
                        if secs > 0.5 {
                            format!(
                                "{} \u{00b7} ~{} words \u{00b7} {:.0} words/s",
                                time_label,
                                word_count,
                                word_count as f64 / secs
                            )
                        } else {
                            format!("{} \u{00b7} ~{} words", time_label, word_count)
                        }
                    } else {
                        time_label
                    }
                })
            });

        // Copy button state
        let last_assistant_id = self
            .current_messages
            .last()
            .filter(|m| m.role == MessageRole::Assistant)
            .map(|m| m.id.clone());
        let is_copied = last_assistant_id
            .as_ref()
            .map(|id| self.is_message_copied(id))
            .unwrap_or(false);

        // Export button state
        let is_exported = self.is_showing_export_feedback();

        div()
            .id("message-actions")
            .flex()
            .flex_col()
            .gap(S1)
            .pl(MSG_PX)
            .mt(S1)
            .mb(S2)
            // Row 1: Regenerate, Copy ⌘⇧C, Export .md ⌘⇧E
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(S2)
                    // Regenerate
                    .child(
                        action_btn_base("regenerate-btn", muted_fg, muted_bg, mouse_mode)
                            .child(action_icon(
                                LocalIconName::Refresh,
                                muted_fg.opacity(OPACITY_ICON_MUTED),
                            ))
                            .child("Regenerate")
                            .on_click(cx.listener(|this, _, window, cx| {
                                tracing::info!(
                                    action = "regenerate",
                                    "action_strip_button_clicked"
                                );
                                this.regenerate_response(window, cx);
                            })),
                    )
                    // Copy ⌘⇧C
                    .child(if is_copied {
                        action_btn_base("copy-response-btn", muted_fg, muted_bg, mouse_mode)
                            .text_color(success.opacity(OPACITY_STRONG))
                            .child(action_icon(
                                LocalIconName::Check,
                                success.opacity(OPACITY_STRONG),
                            ))
                            .child("Copied!")
                            .child(shortcut_hint("\u{2318}\u{21e7}C", muted_fg))
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.copy_last_assistant_response(cx);
                            }))
                    } else {
                        action_btn_base("copy-response-btn", muted_fg, muted_bg, mouse_mode)
                            .child(action_icon(
                                LocalIconName::Copy,
                                muted_fg.opacity(OPACITY_SELECTED),
                            ))
                            .child("Copy")
                            .child(shortcut_hint("\u{2318}\u{21e7}C", muted_fg))
                            .on_click(cx.listener(|this, _, _window, cx| {
                                tracing::info!(
                                    action = "copy_response",
                                    "action_strip_button_clicked"
                                );
                                this.copy_last_assistant_response(cx);
                            }))
                    })
                    // Export .md ⌘⇧E
                    .child(if is_exported {
                        action_btn_base("export-btn", muted_fg, muted_bg, mouse_mode)
                            .text_color(success.opacity(OPACITY_STRONG))
                            .child(action_icon(
                                LocalIconName::Check,
                                success.opacity(OPACITY_STRONG),
                            ))
                            .child("Exported!")
                            .child(shortcut_hint("\u{2318}\u{21e7}E", muted_fg))
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.export_chat_to_clipboard(cx);
                            }))
                    } else {
                        action_btn_base("export-btn", muted_fg, muted_bg, mouse_mode)
                            .child(action_icon(
                                LocalIconName::ArrowDown,
                                muted_fg.opacity(OPACITY_ICON_MUTED),
                            ))
                            .child("Export .md")
                            .child(shortcut_hint("\u{2318}\u{21e7}E", muted_fg))
                            .on_click(cx.listener(|this, _, _window, cx| {
                                tracing::info!(action = "export_md", "action_strip_button_clicked");
                                this.export_chat_to_clipboard(cx);
                            }))
                    }),
            )
            // Row 2: New Chat ⌘N, Search Chats ⌘⇧F, Generated-in label
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(S2)
                    // New Chat ⌘N
                    .child(
                        action_btn_base("new-chat-btn", muted_fg, muted_bg, mouse_mode)
                            .child(action_icon(
                                LocalIconName::Plus,
                                muted_fg.opacity(OPACITY_ICON_MUTED),
                            ))
                            .child("New Chat")
                            .child(shortcut_hint("\u{2318}N", muted_fg))
                            .on_click(cx.listener(|this, _, window, cx| {
                                tracing::info!(action = "new_chat", "action_strip_button_clicked");
                                this.new_conversation(window, cx);
                            })),
                    )
                    // Search Chats ⌘⇧F
                    .child(
                        action_btn_base("search-chats-btn", muted_fg, muted_bg, mouse_mode)
                            .child(action_icon(
                                LocalIconName::MagnifyingGlass,
                                muted_fg.opacity(OPACITY_ICON_MUTED),
                            ))
                            .child("Search Chats")
                            .child(shortcut_hint("\u{2318}\u{21e7}F", muted_fg))
                            .on_click(cx.listener(|this, _, window, cx| {
                                tracing::info!(
                                    action = "search_chats",
                                    "action_strip_button_clicked"
                                );
                                if this.sidebar_collapsed {
                                    this.sidebar_collapsed = false;
                                }
                                this.focus_search(window, cx);
                                cx.notify();
                            })),
                    )
                    // Spacer pushes generated-in label to the end
                    .child(div().flex_1())
                    // Generated-in label (right-aligned)
                    .when_some(completion_label, |el, label| {
                        el.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(S1)
                                .text_xs()
                                .text_color(success.opacity(OPACITY_STRONG))
                                .child(action_icon(
                                    LocalIconName::Check,
                                    success.opacity(OPACITY_SELECTED),
                                ))
                                .child(format!("Generated in {}", label)),
                        )
                    }),
            )
            .into_any_element()
    }
}

/// Base styled div for action strip buttons.
fn action_btn_base(
    id: &'static str,
    muted_fg: gpui::Hsla,
    muted_bg: gpui::Hsla,
    mouse_mode: bool,
) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(S1)
        .px(S2)
        .py(S1)
        .rounded(R_SM)
        .cursor_pointer()
        .text_xs()
        .text_color(muted_fg.opacity(OPACITY_TEXT_MUTED))
        .when(mouse_mode, |d| {
            d.hover(|s| s.bg(muted_bg.opacity(OPACITY_HOVER)).text_color(muted_fg))
        })
}

/// Small SVG icon for action buttons.
fn action_icon(icon: LocalIconName, color: gpui::Hsla) -> Svg {
    svg()
        .external_path(icon.external_path())
        .size(ICON_XS)
        .text_color(color)
}

/// Muted shortcut hint text (e.g. "⌘⇧C").
fn shortcut_hint(text: &'static str, muted_fg: gpui::Hsla) -> Div {
    div()
        .text_xs()
        .text_color(muted_fg.opacity(SHORTCUT_OPACITY))
        .child(text)
}
