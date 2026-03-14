use std::time::Duration;

use super::*;
use crate::theme::opacity::{
    OPACITY_DANGER_BG, OPACITY_HOVER, OPACITY_MESSAGE_ASSISTANT_BACKGROUND, OPACITY_MUTED,
    OPACITY_NEAR_FULL, OPACITY_SELECTED, OPACITY_STRONG,
};

/// Compute opacity for a thinking dot based on animation delta and dot index.
///
/// Three dots cycle through bright/dim states on a 1200ms loop.
/// Each dot occupies a 1/3 window where it ramps up then fades down,
/// creating a sequential pulse effect.
fn thinking_dot_opacity(delta: f32, dot_index: usize) -> f32 {
    // delta is 0..1 over the full 1200ms period
    // Each dot "owns" a 1/3 slice of the cycle
    let phase_offset = dot_index as f32 / 3.0;
    let local = (delta - phase_offset + 1.0) % 1.0; // 0..1 relative to this dot's start

    if local < 1.0 / 3.0 {
        // This dot's active window: ramp up then down
        let t = local * 3.0; // normalize to 0..1
        let brightness = 1.0 - (t * 2.0 - 1.0).abs(); // triangle wave: 0→1→0
        OPACITY_MUTED + brightness * (OPACITY_NEAR_FULL - OPACITY_MUTED)
    } else {
        OPACITY_MUTED
    }
}

fn ai_should_render_streaming_cursor(
    is_streaming: bool,
    streaming_chat_id: Option<ChatId>,
    selected_chat_id: Option<ChatId>,
    streaming_content: &str,
) -> bool {
    is_streaming
        && streaming_chat_id.is_some()
        && streaming_chat_id == selected_chat_id
        && !streaming_content.trim().is_empty()
}

impl AiApp {
    pub(super) fn render_streaming_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = theme::PromptColors::from_theme(&crate::theme::get_cached_theme());
        // No background for assistant streaming content (transparent, matching message style)
        let show_streaming_cursor = ai_should_render_streaming_cursor(
            self.is_streaming,
            self.streaming_chat_id,
            self.selected_chat_id,
            &self.streaming_content,
        );

        let elapsed_label: SharedString = self
            .streaming_started_at
            .map(|started| {
                let secs = started.elapsed().as_secs();
                if secs < 1 {
                    String::new()
                } else {
                    format!("{}s", secs)
                }
            })
            .unwrap_or_default()
            .into();
        let show_elapsed = !elapsed_label.is_empty();

        let content_element = if self.streaming_content.is_empty() {
            // "Thinking" state with model name and elapsed time
            let thinking_label: SharedString = self
                .selected_model
                .as_ref()
                .map(|m| format!("Thinking with {}", m.display_name))
                .unwrap_or_else(|| "Thinking".to_string())
                .into();
            div()
                .flex()
                .items_center()
                .gap(S2)
                .py(S2)
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(thinking_label),
                )
                .child({
                    // Pulsing dots: each dot cycles bright→dim on a 1200ms loop
                    // with a 1/3-period offset so they light up in sequence.
                    let accent = cx.theme().accent;
                    let dot_duration = Duration::from_millis(ANIM_CYCLE_MS);

                    div()
                        .flex()
                        .items_center()
                        .gap(S1)
                        .child(
                            div()
                                .size(S1)
                                .rounded_full()
                                .bg(accent.opacity(OPACITY_NEAR_FULL))
                                .with_animation(
                                    "thinking-dot-0",
                                    Animation::new(dot_duration).repeat(),
                                    move |el, delta| {
                                        // Phase 0: bright at delta 0..0.33
                                        let opacity = thinking_dot_opacity(delta, 0);
                                        el.bg(accent.opacity(opacity))
                                    },
                                ),
                        )
                        .child(
                            div()
                                .size(S1)
                                .rounded_full()
                                .bg(accent.opacity(OPACITY_MUTED))
                                .with_animation(
                                    "thinking-dot-1",
                                    Animation::new(dot_duration).repeat(),
                                    move |el, delta| {
                                        let opacity = thinking_dot_opacity(delta, 1);
                                        el.bg(accent.opacity(opacity))
                                    },
                                ),
                        )
                        .child(
                            div()
                                .size(S1)
                                .rounded_full()
                                .bg(accent.opacity(OPACITY_MUTED))
                                .with_animation(
                                    "thinking-dot-2",
                                    Animation::new(dot_duration).repeat(),
                                    move |el, delta| {
                                        let opacity = thinking_dot_opacity(delta, 2);
                                        el.bg(accent.opacity(opacity))
                                    },
                                ),
                        )
                })
                .when(show_elapsed, |d| {
                    d.child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground.opacity(OPACITY_SELECTED))
                            .child(elapsed_label.clone()),
                    )
                })
                .into_any_element()
        } else {
            // Render markdown separately from cursor to avoid invalidating
            // the markdown cache on every frame during streaming.
            // Keep cursor absolutely positioned so it does not create an extra line.
            div()
                .relative()
                .w_full()
                .min_w_0()
                .overflow_x_hidden()
                .child(render_markdown(&self.streaming_content, &colors))
                .when(show_streaming_cursor, |d| {
                    let accent = cx.theme().accent;
                    let pulse_duration = Duration::from_millis(ANIM_CYCLE_MS);
                    d.child(
                        div()
                            .absolute()
                            .right(S2)
                            .bottom(S2)
                            .text_sm()
                            .text_color(accent)
                            .child("▌")
                            .with_animation(
                                "streaming-cursor-pulse",
                                Animation::new(pulse_duration).repeat(),
                                move |el, delta| {
                                    // Sine wave: delta 0..1 maps to 0..2π
                                    let sine = (delta * std::f32::consts::PI * 2.0).sin();
                                    // Map sine (-1..1) to opacity (0.4..1.0)
                                    let opacity = CURSOR_OPACITY_BASE + CURSOR_OPACITY_AMP * sine;
                                    el.text_color(accent.opacity(opacity))
                                },
                            ),
                    )
                })
                .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .w_full()
            .mb(S3)
            .child({
                // Model name for display in streaming header
                let model_label: Option<SharedString> = self
                    .selected_model
                    .as_ref()
                    .map(|m| SharedString::from(m.display_name.clone()));

                // Role label matching render_message style
                div().flex().items_center().justify_between().mb(S2).child(
                    div()
                        .flex()
                        .items_center()
                        .gap(S2)
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_STRONG))
                                .child("Assistant"),
                        )
                        .when_some(model_label, |d, label| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(
                                        cx.theme().muted_foreground.opacity(OPACITY_SELECTED),
                                    )
                                    .child(label),
                            )
                        })
                        .when(show_elapsed, |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                    .child(elapsed_label),
                            )
                        }),
                )
            })
            .child(
                div()
                    .w_full()
                    .px(MSG_PX)
                    .py(MSG_PY)
                    .rounded(MSG_RADIUS)
                    .child(content_element),
            )
    }

    /// Render a streaming error row with a retry button and contextual help.
    pub(super) fn render_streaming_error(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let err_msg = self
            .streaming_error
            .clone()
            .unwrap_or_else(|| "Unknown error".to_string());
        let danger = cx.theme().danger;

        // Classify error and provide contextual help
        let err_lower = err_msg.to_lowercase();
        let help_hint: Option<&str> = if err_lower.contains("401")
            || err_lower.contains("unauthorized")
            || err_lower.contains("invalid api key")
            || err_lower.contains("authentication")
        {
            Some("Check your API key in settings")
        } else if err_lower.contains("403") || err_lower.contains("forbidden") {
            Some("Your API key may lack required permissions")
        } else if err_lower.contains("429")
            || err_lower.contains("rate limit")
            || err_lower.contains("too many requests")
        {
            Some("Rate limited \u{2014} wait a moment and retry")
        } else if err_lower.contains("timeout")
            || err_lower.contains("timed out")
            || err_lower.contains("deadline")
        {
            Some("Request timed out \u{2014} check your connection")
        } else if err_lower.contains("network")
            || err_lower.contains("connection")
            || err_lower.contains("dns")
            || err_lower.contains("resolve")
        {
            Some("Network error \u{2014} check your internet connection")
        } else if err_lower.contains("500")
            || err_lower.contains("502")
            || err_lower.contains("503")
            || err_lower.contains("server error")
        {
            Some("Server error \u{2014} the provider may be experiencing issues")
        } else if err_lower.contains("cannot be launched inside another")
            || err_lower.contains("nested sessions")
        {
            Some("Close the outer Claude Code session and retry")
        } else if err_lower.contains("claude")
            && (err_lower.contains("not found")
                || err_lower.contains("no such file")
                || err_lower.contains("command not found"))
        {
            Some("Install the Claude Code CLI to use this provider")
        } else if err_lower.contains("cli exited with status") {
            Some("The AI provider process failed \u{2014} check the error details")
        } else {
            None
        };

        div()
            .flex()
            .flex_col()
            .gap(S1)
            .px(S4)
            .py(S2)
            .rounded(R_MD)
            .bg(danger.opacity(OPACITY_MESSAGE_ASSISTANT_BACKGROUND))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(S2)
                    .child(
                        svg()
                            .external_path(LocalIconName::Warning.external_path())
                            .size(ICON_MD)
                            .text_color(danger),
                    )
                    .child(div().flex_1().text_sm().text_color(danger).child(err_msg))
                    .child(
                        div()
                            .id("retry-btn")
                            .flex()
                            .items_center()
                            .gap(S1)
                            .px(S3)
                            .py(S1)
                            .rounded(R_MD)
                            .bg(danger.opacity(OPACITY_DANGER_BG))
                            .text_sm()
                            .text_color(danger)
                            .cursor_pointer()
                            .hover(|s| s.bg(danger.opacity(OPACITY_HOVER)))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.retry_after_error(window, cx);
                            }))
                            .child(
                                svg()
                                    .external_path(LocalIconName::Refresh.external_path())
                                    .size(ICON_XS)
                                    .text_color(danger),
                            )
                            .child("Retry"),
                    ),
            )
            .when_some(help_hint, |el, hint| {
                el.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground.opacity(OPACITY_STRONG))
                        .pl(S6)
                        .child(hint),
                )
            })
    }

    /// Render the editing indicator bar above the input.
    pub(super) fn render_editing_indicator(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let accent = cx.theme().accent;
        let muted_fg = cx.theme().muted_foreground;
        div()
            .flex()
            .items_center()
            .gap(S2)
            .px(S4)
            .py(S1)
            .bg(accent.opacity(OPACITY_MESSAGE_ASSISTANT_BACKGROUND))
            .rounded_t_md()
            .child(
                svg()
                    .external_path(LocalIconName::Pencil.external_path())
                    .size(ICON_XS)
                    .text_color(accent),
            )
            .child(div().text_xs().text_color(accent).child("Editing message"))
            .child(div().flex_1())
            .child(
                div()
                    .text_xs()
                    .text_color(muted_fg)
                    .child("Esc to cancel  \u{00b7}  Enter to save"),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_should_render_streaming_cursor_returns_true_when_streaming_current_chat_has_content()
    {
        let chat_id = ChatId::new();
        assert!(ai_should_render_streaming_cursor(
            true,
            Some(chat_id),
            Some(chat_id),
            "hello"
        ));
    }

    #[test]
    fn test_ai_should_render_streaming_cursor_returns_false_when_streaming_is_inactive() {
        let chat_id = ChatId::new();
        assert!(!ai_should_render_streaming_cursor(
            false,
            Some(chat_id),
            Some(chat_id),
            "hello"
        ));
    }

    #[test]
    fn test_ai_should_render_streaming_cursor_returns_false_when_content_is_empty_or_whitespace() {
        let chat_id = ChatId::new();
        assert!(!ai_should_render_streaming_cursor(
            true,
            Some(chat_id),
            Some(chat_id),
            ""
        ));
        assert!(!ai_should_render_streaming_cursor(
            true,
            Some(chat_id),
            Some(chat_id),
            "  \n\t"
        ));
    }

    #[test]
    fn test_ai_should_render_streaming_cursor_returns_false_when_streaming_chat_is_not_selected() {
        assert!(!ai_should_render_streaming_cursor(
            true,
            Some(ChatId::new()),
            Some(ChatId::new()),
            "hello"
        ));
    }

    #[test]
    fn test_thinking_dot_opacity_each_dot_peaks_in_its_phase() {
        // At delta=0.0, dot 0 should start its bright phase
        let d0_start = thinking_dot_opacity(0.0, 0);
        assert!(
            d0_start >= OPACITY_MUTED,
            "dot 0 at start should be at least OPACITY_MUTED, got {d0_start}"
        );

        // At delta≈0.17 (middle of dot 0's 1/3 window), dot 0 should be near peak
        let d0_peak = thinking_dot_opacity(0.165, 0);
        assert!(
            d0_peak > OPACITY_SELECTED,
            "dot 0 near peak should be bright, got {d0_peak}"
        );

        // At delta≈0.5 (middle of dot 1's window), dot 1 should be bright
        let d1_peak = thinking_dot_opacity(0.5, 1);
        assert!(
            d1_peak > OPACITY_SELECTED,
            "dot 1 near peak should be bright, got {d1_peak}"
        );

        // At delta≈0.83 (middle of dot 2's window), dot 2 should be bright
        let d2_peak = thinking_dot_opacity(0.83, 2);
        assert!(
            d2_peak > OPACITY_SELECTED,
            "dot 2 near peak should be bright, got {d2_peak}"
        );
    }

    #[test]
    fn test_thinking_dot_opacity_stays_within_bounds() {
        for dot in 0..3 {
            for i in 0..100 {
                let delta = i as f32 / 100.0;
                let opacity = thinking_dot_opacity(delta, dot);
                assert!(
                    (OPACITY_MUTED..=OPACITY_NEAR_FULL).contains(&opacity),
                    "dot {dot} at delta {delta}: opacity {opacity} out of range"
                );
            }
        }
    }

    #[test]
    fn test_thinking_dot_opacity_only_one_dot_bright_at_a_time() {
        // At any given delta, at most one dot should be above OPACITY_MUTED
        let threshold = OPACITY_MUTED + 0.01;
        for i in 0..100 {
            let delta = i as f32 / 100.0;
            let bright_count = (0..3)
                .filter(|&dot| thinking_dot_opacity(delta, dot) > threshold)
                .count();
            assert!(
                bright_count <= 1,
                "at delta {delta}, {bright_count} dots are bright (expected at most 1)"
            );
        }
    }
}
