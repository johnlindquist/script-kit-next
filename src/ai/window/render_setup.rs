use super::*;

impl AiApp {
    pub(super) fn render_setup_card(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // If showing API key input mode, render that instead
        if self.showing_api_key_input {
            return self.render_api_key_input(cx).into_any_element();
        }

        // Theme-aware accent color for the button (Raycast style)
        let button_bg = cx.theme().accent;
        let button_text = cx.theme().primary_foreground;
        let configure_button_focused = self.setup_button_focus_index == 0;
        let claude_button_focused = self.setup_button_focus_index == 1;
        let focus_color = cx.theme().ring;

        div()
            .id("setup-card-container")
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .gap(S6)
            .px(S4)
            // Default cursor for the container (buttons will override with pointer)
            .cursor_default()
            // Icon - muted settings icon at top
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(80.))
                    .rounded(R_XL)
                    .bg(cx.theme().muted.opacity(0.2))
                    .child(
                        svg()
                            .external_path(LocalIconName::Settings.external_path())
                            .size(S8)
                            .text_color(cx.theme().muted_foreground.opacity(0.5)),
                    ),
            )
            // Title
            .child(
                div()
                    .text_xl()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(cx.theme().foreground)
                    .child("API Key Required"),
            )
            // Description
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .text_center()
                    .max_w(px(380.))
                    .child("Set up an AI provider to use the Ask AI feature."),
            )
            // Configure Vercel AI Gateway button
            .child(
                div()
                    .id("configure-vercel-btn")
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(S2)
                    .px(S5)
                    .py(S2)
                    .rounded(R_LG)
                    .bg(button_bg)
                    .cursor_pointer()
                    .border_1()
                    .border_color(button_bg.opacity(0.8))
                    .when(configure_button_focused, |s| {
                        s.border_2().border_color(focus_color)
                    })
                    .hover(|s| s.bg(button_bg.opacity(0.9)))
                    .on_click(cx.listener(|this, _, window, cx| {
                        info!("Vercel button clicked in AI window");
                        this.show_api_key_input(window, cx);
                    }))
                    .child(
                        svg()
                            .external_path(LocalIconName::Settings.external_path())
                            .size(ICON_MD)
                            .text_color(button_text),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(button_text)
                            .child("Configure Vercel AI Gateway"),
                    ),
            )
            // "or" separator
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground.opacity(0.6))
                    .child("or"),
            )
            // Connect to Claude Code button
            .child(
                div()
                    .id("connect-claude-code-btn")
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(S2)
                    .px(S5)
                    .py(S2)
                    .rounded(R_LG)
                    .bg(cx.theme().muted.opacity(0.3))
                    .cursor_pointer()
                    .border_1()
                    .border_color(cx.theme().border)
                    .when(claude_button_focused, |s| {
                        s.border_2().border_color(focus_color)
                    })
                    .hover(|s| s.bg(cx.theme().muted.opacity(0.5)))
                    .on_click(cx.listener(|this, _event, window, cx| {
                        info!("Claude Code button clicked in AI window");
                        this.enable_claude_code(window, cx);
                    }))
                    .child(
                        svg()
                            .external_path(LocalIconName::Terminal.external_path())
                            .size(ICON_MD)
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(cx.theme().muted_foreground)
                            .child("Connect to Claude Code"),
                    ),
            )
            // Claude Code setup feedback (shown when config saved but CLI not found)
            .when_some(self.claude_code_setup_feedback.clone(), |el, feedback| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .px(S4)
                        .py(S2)
                        .mt(S2)
                        .rounded(R_MD)
                        .bg(cx.theme().accent.opacity(0.15))
                        .border_1()
                        .border_color(cx.theme().accent.opacity(0.3))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().accent)
                                .text_center()
                                .max_w(px(340.))
                                .child(feedback),
                        ),
                )
            })
            // Info text
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(S1)
                    .mt(S2)
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground.opacity(0.7))
                            .child("Requires Claude Code CLI installed"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("No restart required"),
                    ),
            )
            // Keyboard hints
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(S4)
                    .mt(S4)
                    // Esc to go back
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S2)
                            .child(
                                div()
                                    .px(S2)
                                    .py(S1)
                                    .rounded(R_SM)
                                    .bg(cx.theme().muted)
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Esc"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("to go back"),
                            ),
                    ),
            )
            .into_any_element()
    }

    /// Render the API key input view (shown when user clicks Configure)
    pub(super) fn render_api_key_input(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let input_border_color = cx.theme().accent;

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .gap(S6)
            .px(S4)
            // Back arrow + title
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(S2)
                    .child(
                        div()
                            .id("back-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(S6)
                            .rounded(R_MD)
                            .cursor_pointer()
                            .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.hide_api_key_input(window, cx);
                            }))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("←"),
                            ),
                    )
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(cx.theme().foreground)
                            .child("Enter Vercel API Key"),
                    ),
            )
            // Description
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .text_center()
                    .max_w(px(400.))
                    .child("Get your API key from Vercel AI Gateway and paste it below."),
            )
            // Input field
            .child(
                div()
                    .w(px(400.))
                    .rounded(R_LG)
                    .border_1()
                    .border_color(input_border_color.opacity(0.6))
                    .overflow_hidden()
                    .child(
                        Input::new(&self.api_key_input_state)
                            .w_full()
                            .appearance(false)
                            .focus_bordered(false),
                    ),
            )
            // Keyboard hints
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(S4)
                    .mt(S2)
                    // Enter to save
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S2)
                            .child(
                                div()
                                    .px(S2)
                                    .py(S1)
                                    .rounded(R_SM)
                                    .bg(cx.theme().muted)
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Enter"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("to save"),
                            ),
                    )
                    // Esc to go back
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S2)
                            .child(
                                div()
                                    .px(S2)
                                    .py(S1)
                                    .rounded(R_SM)
                                    .bg(cx.theme().muted)
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Esc"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("to go back"),
                            ),
                    ),
            )
    }
}
