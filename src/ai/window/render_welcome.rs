use super::*;
use crate::{
    components::{
        non_list_card, non_list_centered_shell, non_list_content_stack, non_list_intro,
        non_list_metrics, non_list_palette, NonListDensity, NonListMetrics, NonListPalette,
    },
    theme,
};

/// Script Kit-specific welcome suggestions shown on the AI chat welcome screen.
/// Each tuple: (title, description, icon).
/// Single source of truth — used by both the rendered cards (render_welcome)
/// and the keyboard shortcuts (Cmd+1-4 in render_keydown).
pub(super) fn script_kit_welcome_suggestions() -> [(&'static str, &'static str, LocalIconName); 4] {
    [
        (
            "Monitor clipboard",
            "Write a script to watch clipboard changes and clean copied text.",
            LocalIconName::BoltFilled,
        ),
        (
            "Menu bar shortcut",
            "Create a menu bar shortcut that launches a Script Kit action instantly.",
            LocalIconName::Code,
        ),
        (
            "Rename downloads",
            "Build a script that organizes Downloads files using simple rules.",
            LocalIconName::Terminal,
        ),
        (
            "Quick launcher",
            "Generate a focused launcher for your most-used Script Kit workflows.",
            LocalIconName::Warning,
        ),
    ]
}

impl AiApp {
    /// Compact welcome surface designed specifically for the mini window.
    ///
    /// Unlike the shared `render_welcome` which branches on `is_mini`, this
    /// renderer is purpose-built for the 720×440 mini panel: tighter spacing,
    /// single-line suggestion rows, smaller icons, and no subtitle. The layout
    /// pushes content toward the composer so the panel feels dense and ready.
    pub(super) fn render_mini_welcome(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mini_style = mini_ai_chat_style();
        if self.available_models.is_empty() {
            info!(
                category = "mini_welcome",
                event = "setup_card_shown",
                "Mini welcome: no models configured, showing setup card"
            );
            return self.render_setup_card(cx).into_any_element();
        }

        let theme = theme::get_cached_theme();
        let palette = non_list_palette(&theme);
        let metrics = non_list_metrics(NonListDensity::Compact);
        let all_suggestions = script_kit_welcome_suggestions();

        info!(
            category = "mini_welcome",
            event = "render",
            suggestion_count = MINI_SUGGESTION_COUNT,
            "Mini welcome surface rendered"
        );

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_end()
            .pb(S4)
            .flex_1()
            .gap(px(metrics.item_gap))
            .px(S4)
            // Heading — whisper-quiet, no subtitle
            .child(
                div()
                    .text_sm()
                    .text_color(palette.hint)
                    .child("Try a suggestion"),
            )
            // Suggestion rows — single-line, whisper chrome
            .child(
                non_list_content_stack(
                    "ai-mini-welcome-suggestions",
                    metrics.max_width,
                    metrics.item_gap,
                )
                .max_w(MINI_WELCOME_MAX_W)
                .children(
                    all_suggestions
                        .into_iter()
                        .take(mini_style.suggestion_count)
                        .enumerate()
                        .map(|(i, (title, _desc, icon))| {
                            let prompt_text =
                                SharedString::from(format!("{} {}", title, all_suggestions[i].1));
                            let title_s: SharedString = title.into();
                            div()
                                .id(SharedString::from(format!("mini-suggestion-{}", i)))
                                .flex()
                                .items_center()
                                .gap(S2)
                                .px(S2)
                                .py(SP_3)
                                .rounded(px(metrics.card_radius))
                                .cursor_pointer()
                                .hover(move |s| s.bg(palette.hover))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    info!(
                                        category = "mini_welcome",
                                        event = "suggestion_clicked",
                                        suggestion_index = i,
                                        suggestion_text = %prompt_text,
                                        "Mini welcome suggestion clicked"
                                    );
                                    this.set_composer_value(prompt_text.to_string(), window, cx);
                                    this.submit_message(window, cx);
                                }))
                                // Compact icon — whisper-quiet accent
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size(MINI_WELCOME_ICON_CONTAINER)
                                        .flex_shrink_0()
                                        .child(
                                            svg()
                                                .path(icon.asset_path())
                                                .size(MINI_WELCOME_ICON_SIZE)
                                                .text_color(palette.accent),
                                        ),
                                )
                                // Title only — subdued foreground
                                .child(
                                    div()
                                        .text_sm()
                                        .flex_1()
                                        .text_color(palette.title)
                                        .child(title_s),
                                )
                                // Keyboard shortcut badge — barely-there
                                .child(
                                    div()
                                        .text_xs()
                                        .px(SP_2)
                                        .py(SP_1)
                                        .rounded(SP_2)
                                        .bg(palette.panel)
                                        .text_color(palette.hint)
                                        .flex_shrink_0()
                                        .child(SharedString::from(format!("\u{2318}{}", i + 1))),
                                )
                        }),
                ),
            )
            .into_any_element()
    }

    /// Full-mode welcome surface with centered layout, subtitle, and all suggestion cards.
    ///
    /// Mini mode uses `render_mini_welcome()` instead — this method is only
    /// called from the full panel path.
    pub(super) fn render_welcome(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Show setup card if no providers are configured
        if self.available_models.is_empty() {
            return self.render_setup_card(cx).into_any_element();
        }

        let theme = theme::get_cached_theme();
        let palette = non_list_palette(&theme);
        let metrics = non_list_metrics(NonListDensity::Comfortable);
        let all_suggestions = script_kit_welcome_suggestions();
        let subtitle: SharedString = self
            .selected_model
            .as_ref()
            .map(|m| {
                format!(
                    "Start a conversation with {} or try a suggestion below",
                    m.display_name
                )
            })
            .unwrap_or_else(|| "Start a conversation or try a suggestion below".to_string())
            .into();

        non_list_centered_shell("ai-welcome-non-list", metrics.max_width, metrics.block_gap)
            .flex_1()
            .child(
                non_list_intro("Ask Anything", subtitle, palette, metrics)
                    .items_center()
                    .text_center(),
            )
            // Suggestion cards
            .child(
                non_list_card("ai-welcome-suggestions", palette, metrics)
                    .flex()
                    .flex_col()
                    .gap(px(metrics.item_gap))
                    .children(
                        all_suggestions
                            .into_iter()
                            .take(FULL_SUGGESTION_COUNT)
                            .enumerate()
                            .map(|(i, (title, desc, icon))| {
                                let prompt_text = SharedString::from(format!("{} {}", title, desc));
                                let title_s: SharedString = title.into();
                                let desc_s: SharedString = desc.into();
                                div()
                                    .id(SharedString::from(format!("suggestion-{}", i)))
                                    .flex()
                                    .items_center()
                                    .gap(px(metrics.item_gap))
                                    .px(px(metrics.card_padding_x))
                                    .py(px(metrics.card_padding_y))
                                    .rounded(px(metrics.card_radius))
                                    .cursor_pointer()
                                    .hover(move |s| s.bg(palette.hover))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        info!(
                                            suggestion_text = %prompt_text,
                                            "Welcome suggestion card clicked — auto-submitting"
                                        );
                                        this.set_composer_value(
                                            prompt_text.to_string(),
                                            window,
                                            cx,
                                        );
                                        this.submit_message(window, cx);
                                    }))
                                    // Icon container — fixed size for consistent alignment
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .size(px(metrics.icon_size))
                                            .rounded(px(metrics.card_radius))
                                            .border_1()
                                            .border_color(palette.border)
                                            .bg(palette.input)
                                            .flex_shrink_0()
                                            .child(
                                                svg()
                                                    .path(icon.asset_path())
                                                    .size(px(metrics.icon_size * 0.45))
                                                    .text_color(palette.accent),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .flex_1()
                                            .gap(SP_1)
                                            .child(
                                                div()
                                                    .text_size(px(metrics.body_size))
                                                    .line_height(px(metrics.body_line))
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(palette.title)
                                                    .child(title_s),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .line_height(px(16.0))
                                                    .text_color(palette.body)
                                                    .child(desc_s),
                                            ),
                                    )
                                    // Keyboard shortcut badge — more visible in mini mode
                                    .child(
                                        div()
                                            .text_xs()
                                            .px(SP_3)
                                            .py(SP_1)
                                            .rounded(SP_2)
                                            .bg(palette.panel)
                                            .text_color(palette.hint)
                                            .flex_shrink_0()
                                            .child(SharedString::from(format!(
                                                "\u{2318}{}",
                                                i + 1
                                            ))),
                                    )
                            }),
                    ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::script_kit_welcome_suggestions;

    #[test]
    fn test_script_kit_welcome_suggestions_reference_clipboard_and_menu_bar() {
        let combined = script_kit_welcome_suggestions()
            .into_iter()
            .flat_map(|(title, desc, _icon)| [title, desc])
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();

        assert!(
            combined.contains("clipboard"),
            "suggestions must reference clipboard"
        );
        assert!(
            combined.contains("menu bar"),
            "suggestions must reference menu bar"
        );
    }

    /// Mini mode only exposes the first MINI_SUGGESTION_COUNT suggestions.
    /// Verify that the array has enough entries and that the mini slice is a strict prefix.
    #[test]
    fn test_mini_only_exposes_first_two_suggestions() {
        let all = script_kit_welcome_suggestions();
        let mini_count = super::MINI_SUGGESTION_COUNT;
        assert!(
            all.len() >= mini_count,
            "need at least {mini_count} suggestions for mini mode"
        );
        // Mini shows a prefix — first mini_count entries must be non-empty
        for (title, desc, _icon) in all.iter().take(mini_count) {
            assert!(!title.is_empty(), "mini suggestion title must be non-empty");
            assert!(!desc.is_empty(), "mini suggestion desc must be non-empty");
        }
        // Full mode shows more
        assert!(
            all.len() > mini_count,
            "full mode should show more suggestions than mini"
        );
    }

    /// Each suggestion must produce a non-empty prompt when formatted.
    #[test]
    fn test_welcome_suggestions_produce_non_empty_prompts() {
        for (title, desc, _icon) in script_kit_welcome_suggestions() {
            let prompt = format!("{} {}", title, desc);
            assert!(
                !prompt.trim().is_empty(),
                "suggestion must produce non-empty prompt"
            );
            assert!(
                prompt.len() > 10,
                "suggestion prompt should be descriptive, got: {}",
                prompt
            );
        }
    }
}
