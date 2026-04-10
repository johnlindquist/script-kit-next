use gpui::{
    div, px, rgb, rgba, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _,
};

#[allow(dead_code)]
pub(crate) fn render_launcher_ask_ai_hint(
    chrome: crate::theme::AppChromeColors,
) -> impl IntoElement {
    div()
        .id("ask-ai-button")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(crate::list_item::ASK_AI_BUTTON_GAP))
        .cursor_default()
        .opacity(0.4)
        .child(
            div()
                .text_sm()
                .text_color(rgb(chrome.badge_text_hex))
                .child("Ask"),
        )
        .child(
            div()
                .px(px(crate::list_item::TAB_BADGE_PADDING_X))
                .py(px(crate::list_item::TAB_BADGE_PADDING_Y))
                .rounded(px(crate::list_item::TAB_BADGE_RADIUS))
                .border_1()
                .border_color(rgba(chrome.badge_bg_rgba))
                .text_xs()
                .text_color(rgb(chrome.badge_text_hex))
                .child("⇥"),
        )
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn launcher_ask_ai_hint_stays_tab_only() {
        let source = fs::read_to_string("src/components/launcher_ask_ai_hint.rs")
            .expect("Failed to read src/components/launcher_ask_ai_hint.rs");

        assert!(
            source.contains(".child(\"Ask\")"),
            "launcher ask-ai hint should keep the Ask label"
        );
        assert!(
            source.contains(".child(\"⇥\")"),
            "launcher ask-ai hint should keep the Tab badge"
        );
        assert!(
            !source.contains(".child(\"⌘↩\")"),
            "launcher ask-ai hint should not duplicate the Cmd+Enter badge"
        );
    }
}
