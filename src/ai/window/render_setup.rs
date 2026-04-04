use super::types::*;
use super::*;
use crate::theme::opacity::{
    OPACITY_ACCENT_MEDIUM, OPACITY_ACTIVE, OPACITY_HOVER, OPACITY_PROMINENT, OPACITY_SELECTED,
    OPACITY_STRONG, OPACITY_SUBTLE,
};

impl AiApp {
    pub(super) fn render_setup_card(&self, cx: &mut Context<Self>) -> impl IntoElement {
        debug!(
            setup_icon_size = %SETUP_ICON_CONTAINER_SIZE,
            description_max_w = %SETUP_DESCRIPTION_MAX_W,
            feedback_max_w = %SETUP_FEEDBACK_MAX_W,
            api_key_max_w = %SETUP_API_KEY_MAX_W,
            "render_setup_card layout constants"
        );

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
                    .size(SETUP_ICON_CONTAINER_SIZE)
                    .rounded(R_XL)
                    .bg(cx.theme().muted.opacity(OPACITY_SUBTLE))
                    .child(
                        svg()
                            .external_path(LocalIconName::Settings.external_path())
                            .size(S8)
                            .text_color(cx.theme().muted_foreground.opacity(OPACITY_SELECTED)),
                    ),
            )
            // Title
            .child(
                div()
                    .text_xl()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(cx.theme().foreground)
                    .child("ACP Agent Required"),
            )
            // Description
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .text_center()
                    .max_w(SETUP_DESCRIPTION_MAX_W)
                    .child("Add, install, or authenticate an ACP-compatible agent to continue."),
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
                    .border_color(button_bg.opacity(OPACITY_PROMINENT))
                    .when(configure_button_focused, |s| {
                        s.border_2().border_color(focus_color)
                    })
                    .hover(|s| s.bg(button_bg.opacity(OPACITY_ACTIVE)))
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
                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_ACCENT_MEDIUM))
                    .child("or"),
            )
            // Open ACP agent catalog button
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
                    .bg(cx.theme().muted.opacity(OPACITY_HOVER))
                    .cursor_pointer()
                    .border_1()
                    .border_color(cx.theme().border)
                    .when(claude_button_focused, |s| {
                        s.border_2().border_color(focus_color)
                    })
                    .hover(|s| s.bg(cx.theme().muted.opacity(OPACITY_SELECTED)))
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
                            .child("Open ACP Agent Catalog"),
                    ),
            )
            // ACP setup feedback (shown when setup state changes)
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
                        .bg(cx.theme().accent.opacity(OPACITY_SUBTLE))
                        .border_1()
                        .border_color(cx.theme().accent.opacity(OPACITY_HOVER))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().accent)
                                .text_center()
                                .max_w(SETUP_FEEDBACK_MAX_W)
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
                            .text_color(cx.theme().muted_foreground.opacity(OPACITY_STRONG))
                            .child("Supports any ACP-compatible agent"),
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
                            .hover(|s| s.bg(cx.theme().muted.opacity(OPACITY_HOVER)))
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
                    .max_w(SETUP_API_KEY_MAX_W)
                    .child("Get your API key from Vercel AI Gateway and paste it below."),
            )
            // Input field
            .child(
                div()
                    .w(SETUP_API_KEY_MAX_W)
                    .rounded(R_LG)
                    .border_1()
                    .border_color(input_border_color.opacity(OPACITY_ACCENT_MEDIUM))
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
