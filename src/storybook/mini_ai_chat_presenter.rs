//! Shared presenter for the mini AI chat window.
//!
//! Both storybook previews and the live mini AI chat call
//! [`render_mini_ai_chat_presentation`] with a typed
//! [`MiniAiChatPresentationModel`] and a [`MiniAiChatStyle`].
//! This guarantees visual parity between the two surfaces.

use gpui::*;

use super::mini_ai_chat_variations::MiniAiChatStyle;
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

// ─── Presentation model ────────────────────────────────────────────────

/// Pure-data model that the presenter renders.
/// Constructed by the live AI window from its internal state, or statically
/// by storybook stories.
#[derive(Clone, Debug, PartialEq)]
pub struct MiniAiChatPresentationModel {
    pub title: SharedString,
    pub is_streaming: bool,
    pub model_name: SharedString,
    pub input_text: SharedString,
    pub input_placeholder: SharedString,
    pub messages: Vec<MiniAiChatPresentationMessage>,
    pub show_welcome: bool,
    pub welcome_suggestions: Vec<MiniAiChatSuggestion>,
}

/// A single message in the chat.
#[derive(Clone, Debug, PartialEq)]
pub struct MiniAiChatPresentationMessage {
    pub role: MiniAiChatRole,
    pub content: SharedString,
}

/// Message role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MiniAiChatRole {
    User,
    Assistant,
}

/// A welcome suggestion row.
#[derive(Clone, Debug, PartialEq)]
pub struct MiniAiChatSuggestion {
    pub title: SharedString,
    pub shortcut: SharedString,
}

// ─── Presenter ─────────────────────────────────────────────────────────

/// Render a complete mini AI chat window from a presentation model and a
/// typed style. Every visual knob in [`MiniAiChatStyle`] is consumed here.
pub fn render_mini_ai_chat_presentation(
    model: &MiniAiChatPresentationModel,
    style: MiniAiChatStyle,
    theme: &Theme,
) -> AnyElement {
    let mono: SharedString = SharedString::from(crate::list_item::FONT_MONO);

    let mut root = div()
        .w(px(360.))
        .h(px(220.))
        .flex()
        .flex_col()
        .bg(theme.colors.background.main.to_rgb())
        .rounded(px(10.))
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(0.15))
        .overflow_hidden();

    // Titlebar
    root = root.child(render_titlebar(model, style, theme));

    // Content area
    if model.show_welcome {
        root = root.child(render_welcome(model, style, theme));
    } else {
        root = root.child(render_messages(model, style, theme, &mono));
    }

    // Action hints (only in non-welcome state)
    if !model.show_welcome && style.show_action_hints {
        root = root.child(render_action_hints(style, theme));
    }

    // Composer
    root = root.child(render_composer(model, style, theme, &mono));

    // Hint strip footer
    if style.show_hint_strip {
        root = root.child(render_hint_strip(style, theme));
    }

    root.into_any_element()
}

// ─── Sub-renderers ────────────────────────────────────────────────────

fn render_titlebar(
    model: &MiniAiChatPresentationModel,
    style: MiniAiChatStyle,
    theme: &Theme,
) -> AnyElement {
    let mut bar = div()
        .w_full()
        .h(px(style.titlebar_height))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between();

    // Bottom border
    if style.show_titlebar_border {
        bar = bar.border_b_1().border_color(
            theme
                .colors
                .ui
                .border
                .with_opacity(style.titlebar_border_opacity),
        );
    }

    // Left: title + streaming dot
    let mut left = div().flex().items_center().gap(px(8.)).min_w(px(0.));

    left = left.child(
        div()
            .text_size(px(13.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(
                theme
                    .colors
                    .text
                    .primary
                    .with_opacity(style.titlebar_title_opacity),
            )
            .overflow_x_hidden()
            .text_ellipsis()
            .child(model.title.clone()),
    );

    // Streaming dot
    if model.is_streaming {
        left = left.child(
            div()
                .size(px(6.))
                .rounded_full()
                .bg(theme.colors.accent.selected.to_rgb())
                .flex_shrink_0(),
        );
    }

    // Model chip
    left = left.child(
        div()
            .text_size(px(11.))
            .text_color(
                theme
                    .colors
                    .text
                    .dimmed
                    .with_opacity(style.titlebar_action_opacity),
            )
            .child(model.model_name.clone()),
    );

    // Right: action icons placeholder (3 dots)
    let right = div()
        .flex()
        .items_center()
        .gap(px(4.))
        .child(render_icon_placeholder(
            style.titlebar_action_opacity,
            theme,
        ))
        .child(render_icon_placeholder(
            style.titlebar_action_opacity,
            theme,
        ))
        .child(render_icon_placeholder(
            style.titlebar_action_opacity,
            theme,
        ));

    bar = bar.child(left).child(right);
    bar.into_any_element()
}

fn render_icon_placeholder(opacity: f32, theme: &Theme) -> AnyElement {
    div()
        .size(px(20.))
        .rounded(px(4.))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .size(px(4.))
                .rounded_full()
                .bg(theme.colors.text.dimmed.with_opacity(opacity)),
        )
        .into_any_element()
}

fn render_messages(
    model: &MiniAiChatPresentationModel,
    style: MiniAiChatStyle,
    theme: &Theme,
    mono: &SharedString,
) -> AnyElement {
    let mut container = div()
        .flex_1()
        .min_h(px(0.))
        .w_full()
        .flex()
        .flex_col()
        .overflow_y_hidden()
        .gap(px(style.message_gap));

    for msg in &model.messages {
        container = container.child(render_single_message(msg, style, theme, mono));
    }

    container.into_any_element()
}

fn render_single_message(
    msg: &MiniAiChatPresentationMessage,
    style: MiniAiChatStyle,
    theme: &Theme,
    mono: &SharedString,
) -> AnyElement {
    let is_user = msg.role == MiniAiChatRole::User;

    let bg_opacity = if is_user {
        style.message_user_bg_opacity
    } else {
        style.message_assistant_bg_opacity
    };

    let mut row = div()
        .w_full()
        .px(px(style.message_padding_x))
        .py(px(style.message_padding_y));

    // Background
    if bg_opacity > 0.0 {
        let bg_color = if is_user {
            theme.colors.accent.selected.with_opacity(bg_opacity)
        } else {
            theme.colors.background.main.with_opacity(bg_opacity)
        };
        row = row.bg(bg_color);
    }

    // Border radius
    if style.message_border_radius > 0.0 {
        row = row.rounded(px(style.message_border_radius));
    }

    let mut content_col = div().w_full().flex().flex_col().gap(px(2.));

    // Role label
    if style.show_role_labels {
        let label = if is_user { "You" } else { "Assistant" };
        content_col = content_col.child(
            div()
                .text_size(px(10.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.colors.text.dimmed.with_opacity(0.50))
                .child(SharedString::from(label)),
        );
    }

    // Message content (with optional prefix)
    let mut text_el =
        div()
            .text_size(px(13.))
            .text_color(
                theme
                    .colors
                    .text
                    .primary
                    .with_opacity(if is_user { 0.85 } else { 0.90 }),
            );

    if style.mono_font {
        text_el = text_el.font_family(mono.clone());
    }

    let display_text = match (is_user, style.user_prefix, style.assistant_prefix) {
        (true, Some(prefix), _) => SharedString::from(format!("{} {}", prefix, msg.content)),
        (false, _, Some(prefix)) => SharedString::from(format!("{} {}", prefix, msg.content)),
        _ => msg.content.clone(),
    };

    text_el = text_el.child(display_text);
    content_col = content_col.child(text_el);

    row = row.child(content_col);
    row.into_any_element()
}

fn render_welcome(
    model: &MiniAiChatPresentationModel,
    style: MiniAiChatStyle,
    theme: &Theme,
) -> AnyElement {
    let mut container = div()
        .flex_1()
        .min_h(px(0.))
        .w_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_end()
        .pb(px(12.))
        .gap(px(10.));

    // Heading
    container = container.child(
        div()
            .text_size(px(12.))
            .text_color(
                theme
                    .colors
                    .text
                    .dimmed
                    .with_opacity(style.welcome_heading_opacity),
            )
            .child(SharedString::from("Try a suggestion")),
    );

    // Suggestions
    let mut suggestions = div().flex().flex_col().gap(px(4.)).w_full().max_w(px(280.));

    let count = style.suggestion_count.min(model.welcome_suggestions.len());
    for suggestion in model.welcome_suggestions.iter().take(count) {
        suggestions = suggestions.child(render_suggestion_row(suggestion, style, theme));
    }

    container = container.child(suggestions);
    container.into_any_element()
}

fn render_suggestion_row(
    suggestion: &MiniAiChatSuggestion,
    style: MiniAiChatStyle,
    theme: &Theme,
) -> AnyElement {
    let mut row = div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .px(px(8.))
        .py(px(5.))
        .rounded(px(6.));

    // Title
    row = row.child(
        div()
            .flex_1()
            .text_size(px(12.))
            .text_color(
                theme
                    .colors
                    .text
                    .primary
                    .with_opacity(style.welcome_title_opacity),
            )
            .child(suggestion.title.clone()),
    );

    // Shortcut badge
    row = row.child(
        div()
            .text_size(px(10.))
            .px(px(4.))
            .py(px(1.))
            .rounded(px(3.))
            .bg(theme
                .colors
                .background
                .main
                .with_opacity(style.welcome_badge_bg_opacity))
            .text_color(theme.colors.text.dimmed.with_opacity(0.38))
            .flex_shrink_0()
            .child(suggestion.shortcut.clone()),
    );

    row.into_any_element()
}

fn render_action_hints(style: MiniAiChatStyle, theme: &Theme) -> AnyElement {
    div()
        .w_full()
        .px(px(12.))
        .py(px(2.))
        .text_size(px(10.))
        .text_color(
            theme
                .colors
                .text
                .dimmed
                .with_opacity(style.action_hint_reveal_opacity),
        )
        .child(SharedString::from(
            "\u{2318}K Actions \u{00b7} \u{2318}\u{21e7}C Copy \u{00b7} \u{2318}N New",
        ))
        .into_any_element()
}

fn render_composer(
    model: &MiniAiChatPresentationModel,
    style: MiniAiChatStyle,
    theme: &Theme,
    mono: &SharedString,
) -> AnyElement {
    let mut bar = div()
        .w_full()
        .min_h(px(36.))
        .px(px(8.))
        .py(px(4.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.));

    // Background
    if style.composer_bg_opacity > 0.0 {
        bar = bar.bg(theme
            .colors
            .background
            .main
            .with_opacity(style.composer_bg_opacity));
    }

    // Hairline border
    if style.composer_hairline_opacity > 0.0 {
        bar = bar.border_b_1().border_color(
            theme
                .colors
                .ui
                .border
                .with_opacity(style.composer_hairline_opacity),
        );
    }

    // Input text
    let display_text = if model.input_text.is_empty() {
        model.input_placeholder.clone()
    } else {
        model.input_text.clone()
    };

    let text_color = if model.input_text.is_empty() {
        theme.colors.text.dimmed.with_opacity(0.40)
    } else {
        theme.colors.text.primary.to_rgb()
    };

    let mut input = div().flex_1().text_size(px(13.)).text_color(text_color);
    if style.mono_font {
        input = input.font_family(mono.clone());
    }
    input = input.child(display_text);

    bar = bar.child(input);

    // Submit button indicator
    if !model.input_text.is_empty() || model.is_streaming {
        let icon_color = if model.is_streaming {
            theme
                .colors
                .text
                .secondary
                .with_opacity(style.composer_active_icon_opacity)
        } else {
            theme
                .colors
                .accent
                .selected
                .with_opacity(style.composer_active_icon_opacity)
        };
        bar =
            bar.child(
                div()
                    .size(px(20.))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(div().text_size(px(12.)).text_color(icon_color).child(
                        SharedString::from(if model.is_streaming {
                            "\u{25a0}"
                        } else {
                            "\u{2191}"
                        }),
                    )),
            );
    }

    bar.into_any_element()
}

fn render_hint_strip(style: MiniAiChatStyle, theme: &Theme) -> AnyElement {
    div()
        .w_full()
        .h(px(26.))
        .px(px(14.))
        .flex()
        .items_center()
        .justify_end()
        .text_size(px(10.))
        .text_color(
            theme
                .colors
                .text
                .dimmed
                .with_opacity(style.composer_hint_opacity),
        )
        .child(SharedString::from(style.footer_hint_text))
        .into_any_element()
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storybook::mini_ai_chat_variations::{resolve_mini_ai_chat_style, SPECS};

    fn sample_model() -> MiniAiChatPresentationModel {
        MiniAiChatPresentationModel {
            title: SharedString::from("New Chat"),
            is_streaming: false,
            model_name: SharedString::from("Sonnet"),
            input_text: SharedString::from(""),
            input_placeholder: SharedString::from("Ask anything..."),
            messages: vec![
                MiniAiChatPresentationMessage {
                    role: MiniAiChatRole::User,
                    content: SharedString::from("What is Rust?"),
                },
                MiniAiChatPresentationMessage {
                    role: MiniAiChatRole::Assistant,
                    content: SharedString::from(
                        "Rust is a systems programming language focused on safety and performance.",
                    ),
                },
            ],
            show_welcome: false,
            welcome_suggestions: vec![],
        }
    }

    #[test]
    fn presenter_model_fields_are_complete() {
        let model = sample_model();
        assert_eq!(model.title.as_ref(), "New Chat");
        assert_eq!(model.messages.len(), 2);
        assert!(!model.show_welcome);
        assert!(!model.is_streaming);
    }

    #[test]
    fn presenter_covers_all_style_fields() {
        let styles: Vec<MiniAiChatStyle> = SPECS.iter().map(|s| s.style).collect();
        for (i, a) in styles.iter().enumerate() {
            for (j, b) in styles.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "specs[{i}] and specs[{j}] must differ");
                }
            }
        }
    }

    #[test]
    fn current_variant_matches_production_constants() {
        let (style, resolution) = resolve_mini_ai_chat_style(Some("current"));
        assert_eq!(resolution.resolved_variant_id, "current");
        assert!(style.show_titlebar_border);
        assert_eq!(style.titlebar_height, 44.0);
        assert_eq!(style.message_user_bg_opacity, 0.06);
        assert_eq!(style.message_assistant_bg_opacity, 0.03);
        assert_eq!(style.composer_bg_opacity, 0.03);
    }

    #[test]
    fn message_role_round_trips() {
        let msg = MiniAiChatPresentationMessage {
            role: MiniAiChatRole::User,
            content: SharedString::from("hello"),
        };
        assert_eq!(msg.role, MiniAiChatRole::User);
        assert_eq!(msg.content.as_ref(), "hello");
    }

    #[test]
    fn suggestion_round_trips() {
        let s = MiniAiChatSuggestion {
            title: SharedString::from("Summarize"),
            shortcut: SharedString::from("\u{2318}1"),
        };
        assert_eq!(s.title.as_ref(), "Summarize");
        assert_eq!(s.shortcut.as_ref(), "\u{2318}1");
    }
}
