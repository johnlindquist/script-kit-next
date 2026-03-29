//! Tab AI Chat — Visual States
//!
//! Exposes the key visual states of the Tab AI chat view for deterministic
//! storybook verification: idle, running, error, and memory-hint.
//! Each variant renders using the same shared HintStrip footer and
//! opacity tokens used in production code.

use gpui::*;

use crate::storybook::{
    story_container, story_item, story_section, Story, StorySurface, StoryVariant,
};

pub struct TabAiChatStory;

impl Story for TabAiChatStory {
    fn id(&self) -> &'static str {
        "tab-ai-chat"
    }

    fn name(&self) -> &'static str {
        "Tab AI Chat States"
    }

    fn category(&self) -> &'static str {
        "Views"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Shell
    }

    fn render(&self) -> AnyElement {
        let theme = crate::theme::get_cached_theme();

        story_container()
            .child(story_section("Tab AI Chat — Visual States").children(
                self.variants().iter().map(|v| {
                    let kind = TabAiChatKind::from_variant_id(&v.stable_id());
                    story_item(&v.name, render_chat_kind(&theme, &kind))
                }),
            ))
            .into_any_element()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let theme = crate::theme::get_cached_theme();
        let kind = TabAiChatKind::from_variant_id(&variant.stable_id());
        render_chat_kind(&theme, &kind).into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant::default_named("idle", "Idle")
                .description("Empty input, placeholder visible, gold cursor"),
            StoryVariant::default_named("running", "Running")
                .description("Generating... placeholder, spinner visible"),
            StoryVariant::default_named("error", "Error")
                .description("Error message displayed below divider"),
            StoryVariant::default_named("memory-hint", "Memory Hint")
                .description("Similar prior automation hint below divider"),
            StoryVariant::default_named("save-offer", "Save Offer")
                .description("Post-execution save confirmation — inline panel, not floating card"),
            StoryVariant::default_named("save-offer-error", "Save Offer Error")
                .description("Save offer with file creation error message"),
        ]
    }
}

/// Visual state kind — main chat vs save-offer.
enum TabAiChatKind {
    Main(TabAiMainState),
    SaveOffer(TabAiSaveOfferVisualState),
}

/// Pure data describing one visual state of the main Tab AI chat.
struct TabAiMainState {
    intent: SharedString,
    placeholder: SharedString,
    is_running: bool,
    error: Option<SharedString>,
    memory_hint: Option<SharedString>,
}

/// Pure data describing one visual state of the save-offer.
struct TabAiSaveOfferVisualState {
    filename_stem: SharedString,
    error: Option<SharedString>,
}

impl TabAiChatKind {
    fn from_variant_id(id: &str) -> Self {
        match id {
            "running" => Self::Main(TabAiMainState {
                intent: "".into(),
                placeholder: "Generating...".into(),
                is_running: true,
                error: None,
                memory_hint: None,
            }),
            "error" => Self::Main(TabAiMainState {
                intent: "force quit this app".into(),
                placeholder: "What do you want to do?".into(),
                is_running: false,
                error: Some(
                    "No AI model configured. Open Settings \u{2192} AI and add a provider API key."
                        .into(),
                ),
                memory_hint: None,
            }),
            "memory-hint" => Self::Main(TabAiMainState {
                intent: "focus on slack".into(),
                placeholder: "What do you want to do?".into(),
                is_running: false,
                error: None,
                memory_hint: Some(
                    "Similar prior automation: focus-slack \u{2014} focus on slack (0.92)".into(),
                ),
            }),
            "save-offer" => Self::SaveOffer(TabAiSaveOfferVisualState {
                filename_stem: "focus-slack".into(),
                error: None,
            }),
            "save-offer-error" => Self::SaveOffer(TabAiSaveOfferVisualState {
                filename_stem: "focus-slack".into(),
                error: Some("Failed to create script: permission denied".into()),
            }),
            _ => Self::Main(TabAiMainState {
                // idle
                intent: "".into(),
                placeholder: "What do you want to do?".into(),
                is_running: false,
                error: None,
                memory_hint: None,
            }),
        }
    }
}

/// Dispatch to the correct renderer based on chat kind.
fn render_chat_kind(theme: &crate::theme::Theme, kind: &TabAiChatKind) -> Div {
    match kind {
        TabAiChatKind::Main(state) => render_tab_ai_main_preview(theme, state),
        TabAiChatKind::SaveOffer(state) => render_tab_ai_save_offer_preview(theme, state),
    }
}

/// Render a static preview of the main Tab AI chat view.
///
/// Uses the same shared primitives (HintStrip, opacity tokens, FONT_MONO)
/// as the production Tab AI chat in `src/app_impl/tab_ai_mode.rs`.
fn render_tab_ai_main_preview(theme: &crate::theme::Theme, state: &TabAiMainState) -> Div {
    let accent = gpui::rgb(theme.colors.accent.selected);
    let bg_scrim = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
        theme.colors.background.main,
        crate::theme::opacity::OPACITY_NEAR_FULL,
    ));
    let text_primary = gpui::rgb(theme.colors.text.primary);
    let text_hint = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
        theme.colors.text.primary,
        crate::theme::opacity::OPACITY_DISABLED,
    ));
    let error_color = gpui::rgb(theme.colors.ui.error);
    let divider_rgba = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
        theme.colors.text.primary,
        crate::theme::opacity::OPACITY_GHOST,
    ));

    let hint_px: f32 = crate::window_resize::mini_layout::HINT_STRIP_PADDING_X;

    let intent_text = &state.intent;
    let placeholder = &state.placeholder;

    // Build the input text cell
    let input_cell = if intent_text.is_empty() {
        div()
            .flex_1()
            .text_size(rems(1.125))
            .font_family(crate::list_item::FONT_MONO)
            .text_color(text_hint)
            .child(placeholder.clone())
    } else {
        div()
            .flex_1()
            .text_size(rems(1.125))
            .font_family(crate::list_item::FONT_MONO)
            .text_color(text_primary)
            .child(intent_text.clone())
    };

    // Build input row
    let mut input_row = div()
        .w_full()
        .px(px(hint_px))
        .py(px(10.))
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .child(input_cell);

    // Gold cursor when empty and not running
    if intent_text.is_empty() && !state.is_running {
        input_row = input_row.child(div().w(px(2.)).h(px(18.)).bg(accent));
    }
    // Spinner when running
    if state.is_running {
        input_row = input_row.child(div().text_sm().text_color(accent).child("\u{25CF}"));
    }

    // Build main container
    let mut container = div()
        .w(px(520.))
        .h(px(200.))
        .flex()
        .flex_col()
        .bg(bg_scrim)
        .rounded(px(4.))
        .overflow_hidden()
        .child(input_row)
        .child(div().w_full().h(px(1.)).bg(divider_rgba));

    // Error message
    if let Some(msg) = &state.error {
        container = container.child(
            div()
                .w_full()
                .px(px(hint_px))
                .py(px(4.))
                .text_xs()
                .text_color(error_color)
                .child(msg.clone()),
        );
    }

    // Memory hint
    if let Some(hint) = &state.memory_hint {
        container = container.child(
            div()
                .w_full()
                .px(px(hint_px))
                .pb(px(4.))
                .text_xs()
                .text_color(text_hint)
                .child(hint.clone()),
        );
    }

    container
        // Spacer pushes footer to bottom
        .child(div().flex_1())
        // Footer — canonical three-key hint strip
        .child(crate::components::HintStrip::new(vec![
            "\u{21B5} Run".into(),
            "\u{2318}K Actions".into(),
            "Tab AI".into(),
        ]))
}

/// Render a static preview of the save-offer.
///
/// Mirrors the inline panel treatment from the production
/// `render_tab_ai_save_offer_overlay` in `src/app_impl/tab_ai_mode.rs`.
fn render_tab_ai_save_offer_preview(
    theme: &crate::theme::Theme,
    state: &TabAiSaveOfferVisualState,
) -> Div {
    let bg_scrim = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
        theme.colors.background.main,
        crate::theme::opacity::OPACITY_NEAR_FULL,
    ));
    let text_primary = gpui::rgb(theme.colors.text.primary);
    let error_color = gpui::rgb(theme.colors.ui.error);
    let divider_rgba = gpui::rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
        theme.colors.text.primary,
        crate::theme::opacity::OPACITY_GHOST,
    ));

    let hint_px: f32 = crate::window_resize::mini_layout::HINT_STRIP_PADDING_X;

    let message = SharedString::from(format!("Save as {}.ts?", state.filename_stem));

    let mut container = div()
        .w(px(520.))
        .h(px(200.))
        .flex()
        .flex_col()
        .bg(bg_scrim)
        .rounded(px(4.))
        .overflow_hidden()
        // Message row — bare text, no card, no accent bar
        .child(
            div().w_full().px(px(hint_px)).py(px(10.)).child(
                div()
                    .text_sm()
                    .font_family(crate::list_item::FONT_MONO)
                    .text_color(text_primary)
                    .child(message),
            ),
        )
        // Hairline divider — ghost opacity
        .child(div().w_full().h(px(1.)).bg(divider_rgba));

    // Error message
    if let Some(msg) = &state.error {
        container = container.child(
            div()
                .w_full()
                .px(px(hint_px))
                .py(px(4.))
                .text_xs()
                .text_color(error_color)
                .child(msg.clone()),
        );
    }

    container
        // Spacer pushes footer to bottom
        .child(div().flex_1())
        // Footer — save-specific hint strip (justified exception: confirmation dialog)
        .child(crate::components::HintStrip::new(vec![
            "\u{21B5} Save".into(),
            "Esc Dismiss".into(),
        ]))
}

#[cfg(test)]
mod tests {
    use super::TabAiChatStory;
    use crate::storybook::{Story, StorySurface};

    #[test]
    fn tab_ai_chat_story_is_discoverable() {
        let story = TabAiChatStory;
        assert_eq!(story.id(), "tab-ai-chat");
        assert_eq!(story.surface(), StorySurface::Shell);
    }

    #[test]
    fn tab_ai_chat_story_has_six_variants() {
        let story = TabAiChatStory;
        let variants = story.variants();
        assert_eq!(variants.len(), 6);
        let ids: Vec<String> = variants.iter().map(|v| v.stable_id()).collect();
        assert!(ids.contains(&"idle".to_string()));
        assert!(ids.contains(&"running".to_string()));
        assert!(ids.contains(&"error".to_string()));
        assert!(ids.contains(&"memory-hint".to_string()));
        assert!(ids.contains(&"save-offer".to_string()));
        assert!(ids.contains(&"save-offer-error".to_string()));
    }

    #[test]
    fn tab_ai_chat_story_is_comparable() {
        let story = TabAiChatStory;
        assert!(
            story.variants().len() > 1,
            "must be comparable (>1 variant)"
        );
    }
}
