use gpui::{div, prelude::*, px, App, IntoElement, RenderOnce, Window};

/// Minimum height keeps the input tappable/clickable even when empty (matches standard touch target).
const INLINE_PROMPT_MIN_HEIGHT: f32 = 28.0;

#[derive(IntoElement)]
pub struct InlinePromptInput {
    content: gpui::AnyElement,
}

impl InlinePromptInput {
    pub fn new(content: impl IntoElement) -> Self {
        Self {
            content: content.into_any_element(),
        }
    }
}

impl RenderOnce for InlinePromptInput {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .id("inline-prompt-input")
            .w_full()
            .min_h(px(INLINE_PROMPT_MIN_HEIGHT))
            .flex()
            .flex_row()
            .items_center()
            .child(self.content)
    }
}
