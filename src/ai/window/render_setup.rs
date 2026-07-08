use super::types::*;
use super::*;
use crate::theme::opacity::{OPACITY_ACCENT_MEDIUM, OPACITY_HOVER, OPACITY_SUBTLE};

impl AiApp {
    pub(super) fn render_setup_card(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.showing_api_key_input {
            return self.render_api_key_input(cx).into_any_element();
        }

        let theme = crate::theme::get_cached_theme();
        let setup_info = crate::components::agent_setup_info_spec(
            "Agent Required",
            "Add, install, or authenticate an agent in the catalog or config.ts.",
            None::<String>,
        )
        .footer_note("Supports compatible command-line agents · no restart required · Esc closes");
        let setup_info = crate::components::render_info_state(setup_info, &theme, cx);
        let setup_width =
            crate::components::info_metrics(crate::components::InfoStateDensity::Comfortable)
                .max_width;
        let button_colors = crate::components::ButtonColors::from_theme(&theme);
        let catalog_button = crate::components::Button::new("Open Agent Catalog", button_colors)
            .id("open-agent-catalog-btn")
            .shortcut("↵")
            .focused(self.setup_button_focus_index == 0)
            .on_click(Box::new(cx.listener(|this, _event, window, cx| {
                this.open_agent_chat_agents_catalog(window, cx);
            })));

        div()
            .id("setup-card-container")
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .gap(S6)
            .px(S4)
            .cursor_default()
            .child(div().w_full().max_w(px(setup_width)).child(setup_info))
            .child(catalog_button)
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
            .into_any_element()
    }

    /// Render the API key input view (shown when user clicks Configure)
    pub(super) fn render_api_key_input(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let input_border_color = cx.theme().accent;
        let theme = crate::theme::get_cached_theme();

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
                    .w(SETUP_API_KEY_MAX_W)
                    .child(crate::components::render_back_affordance(
                        "back-btn".into(),
                        "Agent setup".into(),
                        &theme,
                        cx.listener(|this, _, window, cx| {
                            this.hide_api_key_input(window, cx);
                        }),
                    )),
            )
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(cx.theme().foreground)
                    .child("Enter API Key"),
            )
            // Description
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .text_center()
                    .max_w(SETUP_API_KEY_MAX_W)
                    .child("Paste your provider API key below."),
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
