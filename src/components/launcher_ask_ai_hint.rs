use gpui::{div, px, rgba, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _};

#[allow(dead_code)]
pub(crate) fn render_launcher_ask_ai_hint(
    chrome: crate::theme::AppChromeColors,
) -> impl IntoElement {
    // Storybook Ask+Tab option 22: `tab-glyph-soft-right`.
    // Keep the hint bare and theme-derived so it reads as quiet header chrome.
    let ask_rgba = (chrome.text_muted_hex << 8) | 0x7a;
    let tab_rgba = (chrome.text_muted_hex << 8) | 0x80;

    div()
        .id("ask-ai-button")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(5.0))
        .cursor_default()
        .child(
            div()
                .text_size(px(15.0))
                .line_height(px(19.0))
                .font_weight(gpui::FontWeight::NORMAL)
                .text_color(rgba(ask_rgba))
                .child("Ask"),
        )
        .child(
            div()
                .text_size(px(15.0))
                .line_height(px(19.0))
                .font_weight(gpui::FontWeight::NORMAL)
                .text_color(rgba(tab_rgba))
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

    #[test]
    fn launcher_ask_ai_hint_uses_storybook_option_22_plain_text_tab_glyph() {
        let source = fs::read_to_string("src/components/launcher_ask_ai_hint.rs")
            .expect("Failed to read src/components/launcher_ask_ai_hint.rs");
        // Only inspect the render function body. Assertions below reference the
        // substrings literally, so scanning the whole file would self-match.
        let render_body = source
            .split("#[cfg(test)]")
            .next()
            .expect("launcher ask-ai hint source should contain a render body");

        assert!(
            render_body.contains("tab-glyph-soft-right"),
            "launcher ask-ai hint should document the adopted Storybook option"
        );
        assert!(
            render_body.contains("chrome.text_muted_hex")
                && render_body.contains("0x7a")
                && render_body.contains("0x80"),
            "launcher ask-ai hint should use theme muted text with option 22's separate Ask/tab opacities"
        );
        assert!(
            render_body.matches(".text_size(px(15.0))").count() >= 2,
            "launcher ask-ai hint should use option 22's 15px Ask and tab glyph sizing"
        );
        assert!(
            render_body.contains(".gap(px(5.0))"),
            "launcher ask-ai hint should use option 22's 5px spacing"
        );
        assert!(
            !render_body.contains(".border_1()")
                && !render_body.contains(".bg(rgba(")
                && !render_body.contains(".rounded(px("),
            "launcher ask-ai hint should render the tab glyph without badge chrome"
        );
    }
}
