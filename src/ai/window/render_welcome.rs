use super::*;

impl AiApp {
    pub(super) fn render_welcome(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Show setup card if no providers are configured
        if self.available_models.is_empty() {
            return self.render_setup_card(cx).into_any_element();
        }

        let suggestion_bg = cx.theme().muted.opacity(0.20);
        let suggestion_hover_bg = cx.theme().muted.opacity(0.35);

        let suggestions: Vec<(&str, &str, LocalIconName)> = vec![
            (
                "Write a script",
                "to automate a repetitive task",
                LocalIconName::Terminal,
            ),
            (
                "Explain how",
                "this code works step by step",
                LocalIconName::Code,
            ),
            (
                "Help me debug",
                "an error I'm seeing",
                LocalIconName::Warning,
            ),
            (
                "Generate a function",
                "that processes data",
                LocalIconName::BoltFilled,
            ),
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .gap(S6)
            .px(S4)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(S3)
                    .child(
                        div()
                            .text_xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(cx.theme().foreground)
                            .child("Ask Anything"),
                    )
                    .child({
                        let subtitle: SharedString = self
                            .selected_model
                            .as_ref()
                            .map(|m| {
                                format!(
                                    "Start a conversation with {} or try a suggestion below",
                                    m.display_name
                                )
                            })
                            .unwrap_or_else(|| {
                                "Start a conversation or try a suggestion below".to_string()
                            })
                            .into();
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground.opacity(0.7))
                            .child(subtitle)
                    }),
            )
            // Suggestion cards
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(S2)
                    .w_full()
                    .max_w(px(400.))
                    .children(suggestions.into_iter().enumerate().map(
                        |(i, (title, desc, icon))| {
                            let prompt_text = SharedString::from(format!("{} {}", title, desc));
                            let title_s: SharedString = title.into();
                            let desc_s: SharedString = desc.into();
                            div()
                                .id(SharedString::from(format!("suggestion-{}", i)))
                                .flex()
                                .items_center()
                                .gap(S3)
                                .px(S4)
                                .py(S3)
                                .rounded(R_SM)
                                .bg(suggestion_bg)
                                .cursor_pointer()
                                .hover(move |s| s.bg(suggestion_hover_bg))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.input_state.update(cx, |state, cx| {
                                        state.set_value(prompt_text.to_string(), window, cx);
                                    });
                                    this.focus_input(window, cx);
                                }))
                                .child(
                                    svg()
                                        .external_path(icon.external_path())
                                        .size(ICON_MD)
                                        .text_color(cx.theme().accent.opacity(0.6))
                                        .flex_shrink_0(),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(S1)
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(cx.theme().foreground)
                                                .child(title_s),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(
                                                    cx.theme().muted_foreground.opacity(0.6),
                                                )
                                                .child(desc_s),
                                        ),
                                )
                        },
                    )),
            )
            // Keyboard shortcut hints
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(S3)
                    .mt(S1)
                    .children(
                        [
                            ("\u{2318} Enter", "Send"),
                            ("\u{2318} N", "New Chat"),
                            ("\u{2318} K", "Actions"),
                            ("Esc", "Stop"),
                        ]
                        .into_iter()
                        .map(|(key, label)| {
                            let key_s: SharedString = key.into();
                            let label_s: SharedString = label.into();
                            div()
                                .flex()
                                .items_center()
                                .gap(S1)
                                .child(
                                    div()
                                        .px(S2)
                                        .py(S1)
                                        .rounded(R_SM)
                                        .bg(cx.theme().muted.opacity(0.3))
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground.opacity(0.55))
                                        .child(key_s),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground.opacity(0.4))
                                        .child(label_s),
                                )
                        }),
                    ),
            )
            .into_any_element()
    }
}
