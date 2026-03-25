use super::*;
use crate::theme::opacity::{
    OPACITY_ACCENT_MEDIUM, OPACITY_CARD_BG, OPACITY_STRONG, OPACITY_SUGGESTION_HOVER,
    OPACITY_TEXT_MUTED,
};

/// Icon container size for suggestion cards (provides consistent hit area around the icon).
const SUGGESTION_ICON_CONTAINER: Pixels = px(36.);
/// Icon size within suggestion cards.
const SUGGESTION_ICON_SIZE: Pixels = px(18.);
/// Maximum width of the suggestion card column.
const SUGGESTION_MAX_W: Pixels = px(540.);

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
        if self.available_models.is_empty() {
            info!(
                category = "mini_welcome",
                event = "setup_card_shown",
                "Mini welcome: no models configured, showing setup card"
            );
            return self.render_setup_card(cx).into_any_element();
        }

        let suggestion_hover_bg = cx.theme().muted.opacity(OPACITY_SUGGESTION_HOVER);
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
            .gap(S3)
            .px(S4)
            // Heading — compact, no subtitle
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_TEXT_MUTED))
                    .child("Try a suggestion"),
            )
            // Suggestion rows — single-line, compact
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(SP_2)
                    .w_full()
                    .max_w(MINI_WELCOME_MAX_W)
                    .children(
                        all_suggestions
                            .into_iter()
                            .take(MINI_SUGGESTION_COUNT)
                            .enumerate()
                            .map(|(i, (title, _desc, icon))| {
                                let prompt_text = SharedString::from(format!(
                                    "{} {}",
                                    title,
                                    all_suggestions[i].1
                                ));
                                let title_s: SharedString = title.into();
                                div()
                                    .id(SharedString::from(format!("mini-suggestion-{}", i)))
                                    .flex()
                                    .items_center()
                                    .gap(S2)
                                    .px(S2)
                                    .py(SP_3)
                                    .rounded(R_SM)
                                    .cursor_pointer()
                                    .hover(move |s| s.bg(suggestion_hover_bg))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        info!(
                                            category = "mini_welcome",
                                            event = "suggestion_clicked",
                                            suggestion_index = i,
                                            suggestion_text = %prompt_text,
                                            "Mini welcome suggestion clicked"
                                        );
                                        this.set_composer_value(
                                            prompt_text.to_string(),
                                            window,
                                            cx,
                                        );
                                        this.submit_message(window, cx);
                                    }))
                                    // Compact icon
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .size(MINI_WELCOME_ICON_CONTAINER)
                                            .rounded(SP_3)
                                            .flex_shrink_0()
                                            .child(
                                                svg()
                                                    .external_path(icon.external_path())
                                                    .size(MINI_WELCOME_ICON_SIZE)
                                                    .text_color(
                                                        cx.theme()
                                                            .accent
                                                            .opacity(OPACITY_ACCENT_MEDIUM),
                                                    ),
                                            ),
                                    )
                                    // Title only — no description in mini
                                    .child(
                                        div()
                                            .text_sm()
                                            .flex_1()
                                            .text_color(cx.theme().foreground)
                                            .child(title_s),
                                    )
                                    // Keyboard shortcut badge
                                    .child(
                                        div()
                                            .text_xs()
                                            .px(SP_2)
                                            .py(SP_1)
                                            .rounded(SP_2)
                                            .bg(cx.theme().muted.opacity(OPACITY_CARD_BG))
                                            .text_color(
                                                cx.theme()
                                                    .muted_foreground
                                                    .opacity(OPACITY_STRONG),
                                            )
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

    /// Full-mode welcome surface with centered layout, subtitle, and all suggestion cards.
    ///
    /// Mini mode uses `render_mini_welcome()` instead — this method is only
    /// called from the full panel path.
    pub(super) fn render_welcome(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Show setup card if no providers are configured
        if self.available_models.is_empty() {
            return self.render_setup_card(cx).into_any_element();
        }

        let suggestion_bg = cx.theme().muted.opacity(OPACITY_CARD_BG);
        let suggestion_hover_bg = cx.theme().muted.opacity(OPACITY_SUGGESTION_HOVER);

        let all_suggestions = script_kit_welcome_suggestions();

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .gap(S7)
            .px(S6)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(S2)
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
                    .gap(S1)
                    .w_full()
                    .max_w(SUGGESTION_MAX_W)
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
                                    .gap(S3)
                                    .pl(S3)
                                    .pr(S4)
                                    .py(S3)
                                    .rounded(R_LG)
                                    .cursor_pointer()
                                    .hover(move |s| s.bg(suggestion_hover_bg))
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
                                            .size(SUGGESTION_ICON_CONTAINER)
                                            .rounded(R_SM)
                                            .bg(suggestion_bg)
                                            .flex_shrink_0()
                                            .child(
                                                svg()
                                                    .external_path(icon.external_path())
                                                    .size(SUGGESTION_ICON_SIZE)
                                                    .text_color(
                                                        cx.theme()
                                                            .accent
                                                            .opacity(OPACITY_ACCENT_MEDIUM),
                                                    ),
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
                                    // Keyboard shortcut badge — more visible in mini mode
                                    .child(
                                        div()
                                            .text_xs()
                                            .px(SP_3)
                                            .py(SP_1)
                                            .rounded(SP_2)
                                            .bg(cx.theme().muted.opacity(OPACITY_CARD_BG))
                                            .text_color(
                                                cx.theme().muted_foreground.opacity(OPACITY_STRONG),
                                            )
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
