use super::*;
use crate::theme::opacity::{
    OPACITY_ACCENT_MEDIUM, OPACITY_DANGER_BG, OPACITY_STRONG, OPACITY_SUGGESTION_HOVER,
};

impl AiApp {
    pub(super) fn render_welcome(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Show setup card if no providers are configured
        if self.available_models.is_empty() {
            return self.render_setup_card(cx).into_any_element();
        }

        let suggestion_bg = cx.theme().muted.opacity(OPACITY_DANGER_BG);
        let suggestion_hover_bg = cx.theme().muted.opacity(OPACITY_SUGGESTION_HOVER);

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
                            .text_color(cx.theme().muted_foreground.opacity(OPACITY_STRONG))
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
                                    info!(
                                        suggestion_text = %prompt_text,
                                        "Welcome suggestion card clicked — auto-submitting"
                                    );
                                    this.input_state.update(cx, |state, cx| {
                                        state.set_value(prompt_text.to_string(), window, cx);
                                    });
                                    this.submit_message(window, cx);
                                }))
                                .child(
                                    svg()
                                        .external_path(icon.external_path())
                                        .size(ICON_MD)
                                        .text_color(
                                            cx.theme().accent.opacity(OPACITY_ACCENT_MEDIUM),
                                        )
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
                                                    cx.theme()
                                                        .muted_foreground
                                                        .opacity(OPACITY_ACCENT_MEDIUM),
                                                )
                                                .child(desc_s),
                                        ),
                                )
                        },
                    )),
            )
            .into_any_element()
    }
}
