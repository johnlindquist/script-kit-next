use super::*;
use crate::components::button::TRANSPARENT;

impl ChatPrompt {
    pub(super) fn render_setup_card(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = &self.prompt_colors;

        let accent_full = rgba((colors.accent_color << 8) | 0xFF);
        let accent_25 = rgba((colors.accent_color << 8) | 0x40);
        let muted_bg = rgba((colors.code_bg << 8) | 0x60);
        let muted_bg_hover = rgba((colors.code_bg << 8) | 0x90);
        let ring_color = rgba((colors.accent_color << 8) | 0x80);
        let kbd_bg = rgba((colors.code_bg << 8) | 0x50);
        let accent_text = rgb(self.theme.colors.text.on_accent);

        let on_configure = self.on_configure.clone();
        let on_claude_code = self.on_claude_code.clone();
        let is_configure_focused = self.setup_focus_index == 0;
        let is_claude_focused = self.setup_focus_index == 1;

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .gap(px(20.))
            .px(px(24.))
            .cursor_default()
            // Icon
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(72.))
                    .rounded(px(18.))
                    .bg(muted_bg)
                    .child(
                        svg()
                            .external_path(IconName::Settings.external_path())
                            .size(px(36.))
                            .text_color(rgb(colors.text_secondary)),
                    ),
            )
            // Title
            .child(
                div()
                    .text_xl()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(colors.text_primary))
                    .child("API Key Required"),
            )
            // Description
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(colors.text_secondary))
                    .text_center()
                    .max_w(px(380.))
                    .child("Set up an AI provider to use the Ask AI feature."),
            )
            // Buttons
            .child(
                div()
                    .id("setup-buttons-container")
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(10.))
                    .mt(px(4.))
                    // Configure Vercel AI Gateway (primary)
                    .child(
                        div()
                            .id("configure-button")
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap(px(8.))
                            .px(px(20.))
                            .py(px(10.))
                            .rounded(px(10.))
                            .bg(if is_configure_focused {
                                accent_25
                            } else {
                                accent_full
                            })
                            .cursor_pointer()
                            .border_2()
                            .border_color(if is_configure_focused {
                                ring_color
                            } else {
                                rgba(TRANSPARENT)
                            })
                            .when(is_configure_focused, |s| {
                                s.shadow(vec![gpui::BoxShadow {
                                    color: ring_color.into(),
                                    offset: gpui::point(px(0.), px(0.)),
                                    blur_radius: px(4.),
                                    spread_radius: px(-1.),
                                }])
                            })
                            .hover(|s| s.bg(accent_25))
                            .on_click(cx.listener(move |_this, _event, _window, _cx| {
                                if let Some(ref cb) = on_configure {
                                    cb();
                                }
                            }))
                            .child(
                                svg()
                                    .external_path(IconName::Settings.external_path())
                                    .size(px(16.))
                                    .text_color(accent_text),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(accent_text)
                                    .child("Configure Vercel AI Gateway"),
                            ),
                    )
                    // "or" separator
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(colors.text_tertiary))
                            .child("or"),
                    )
                    // Connect to Claude Code (secondary)
                    .child(
                        div()
                            .id("configure-claude-code-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap(px(8.))
                            .px(px(20.))
                            .py(px(10.))
                            .rounded(px(10.))
                            .bg(if is_claude_focused {
                                muted_bg_hover
                            } else {
                                muted_bg
                            })
                            .cursor_pointer()
                            .border_2()
                            .border_color(if is_claude_focused {
                                ring_color
                            } else {
                                rgba(TRANSPARENT)
                            })
                            .when(is_claude_focused, |s| {
                                s.shadow(vec![gpui::BoxShadow {
                                    color: ring_color.into(),
                                    offset: gpui::point(px(0.), px(0.)),
                                    blur_radius: px(4.),
                                    spread_radius: px(-1.),
                                }])
                            })
                            .hover(|s| s.bg(muted_bg_hover))
                            .on_click(cx.listener(move |_this, _event, _window, _cx| {
                                if let Some(ref cb) = on_claude_code {
                                    cb();
                                }
                            }))
                            .child(
                                svg()
                                    .external_path(IconName::Terminal.external_path())
                                    .size(px(16.))
                                    .text_color(rgb(colors.text_secondary)),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(colors.text_secondary))
                                    .child("Connect to Claude Code"),
                            ),
                    )
                    // Claude Code caption
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(colors.text_tertiary))
                            .child("Requires Claude Code CLI installed"),
                    ),
            )
            // Keyboard hints
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(16.))
                    .mt(px(12.))
                    .child(Self::render_kbd_hint("Tab", "switch", colors, kbd_bg))
                    .child(Self::render_kbd_hint("Enter", "select", colors, kbd_bg))
                    .child(Self::render_kbd_hint("Esc", "go back", colors, kbd_bg)),
            )
    }

    /// Render a keyboard hint badge with a key and label.
    pub(super) fn render_kbd_hint(
        key: &str,
        label: &str,
        colors: &crate::theme::PromptColors,
        kbd_bg: gpui::Rgba,
    ) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(4.))
            .child(
                div()
                    .px(px(6.))
                    .py(px(2.))
                    .rounded(px(4.))
                    .bg(kbd_bg)
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_tertiary))
                    .child(key.to_string()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_tertiary))
                    .child(label.to_string()),
            )
    }
}
