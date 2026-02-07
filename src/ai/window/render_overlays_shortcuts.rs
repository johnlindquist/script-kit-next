use super::*;

impl AiApp {
    pub(super) fn render_command_bar_overlay(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        // Command bar now renders in a separate vibrancy window (not inline)
        // See CommandBar component for window management
        div().id("command-bar-overlay-deprecated")
    }

    /// Render the keyboard shortcuts overlay (Cmd+/).
    pub(super) fn render_shortcuts_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let overlay_bg = Self::get_modal_overlay_background();
        let panel_bg = cx.theme().background;
        let border = cx.theme().border;
        let fg = cx.theme().foreground;
        let muted = cx.theme().muted_foreground;
        let accent = cx.theme().accent;

        let shortcuts: Vec<(&str, &str)> = vec![
            ("\u{2318} Enter", "Send message"),
            ("\u{2318} N", "New chat"),
            ("\u{2318} Shift N", "New chat with preset"),
            ("\u{2318} K", "Open actions"),
            ("\u{2318} L", "Focus input"),
            ("\u{2318} B", "Toggle sidebar"),
            ("\u{2318} Shift F", "Search chats"),
            ("\u{2318} Shift C", "Copy last response"),
            ("\u{2318} Shift E", "Export chat as markdown"),
            ("\u{2318} [ / ]", "Previous / next chat"),
            ("\u{2318} Shift \u{232B}", "Delete chat"),
            ("\u{2318} /", "Toggle this overlay"),
            ("Esc", "Stop streaming / close"),
            ("\u{2191}", "Edit last message (empty input)"),
        ];

        div()
            .id("shortcuts-overlay")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(overlay_bg)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.showing_shortcuts_overlay = false;
                    cx.notify();
                }),
            )
            .child(
                div()
                    .id("shortcuts-panel")
                    .w(px(380.))
                    .max_h(px(480.))
                    .rounded_xl()
                    .bg(panel_bg)
                    .border_1()
                    .border_color(border)
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .overflow_y_scroll()
                    // Prevent clicks inside the panel from closing the overlay
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    // Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(fg)
                                    .child("Keyboard Shortcuts"),
                            )
                            .child(
                                div()
                                    .px(px(6.))
                                    .py(px(2.))
                                    .rounded(px(4.))
                                    .bg(cx.theme().muted.opacity(0.3))
                                    .text_xs()
                                    .text_color(muted)
                                    .child("\u{2318} /"),
                            ),
                    )
                    // Divider
                    .child(div().w_full().h(px(1.)).bg(border))
                    // Shortcuts list
                    .children(shortcuts.into_iter().map(|(key, desc)| {
                        let key_s: SharedString = key.into();
                        let desc_s: SharedString = desc.into();
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .py(px(4.))
                            .child(div().text_sm().text_color(fg).child(desc_s))
                            .child(
                                div()
                                    .px(px(8.))
                                    .py(px(2.))
                                    .rounded(px(4.))
                                    .bg(accent.opacity(0.1))
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(accent)
                                    .child(key_s),
                            )
                    })),
            )
    }
}
