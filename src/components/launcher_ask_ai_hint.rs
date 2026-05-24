use gpui::{div, px, rgba, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _};

#[allow(dead_code)]
pub(crate) fn render_launcher_ask_ai_hint(
    chrome: crate::theme::AppChromeColors,
) -> impl IntoElement {
    // Keep the hint quieter than footer keycaps while giving Tab a visible key boundary.
    let ask_rgba = (chrome.text_muted_hex << 8) | 0x92;
    let tab_rgba = (chrome.text_muted_hex << 8) | 0x9c;
    let tab_border_rgba = (chrome.text_muted_hex << 8) | 0x44;

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
                .size(px(
                    crate::components::footer_chrome::FOOTER_KEYCAP_HEIGHT_PX,
                ))
                .rounded(px(
                    crate::components::footer_chrome::FOOTER_KEYCAP_RADIUS_PX,
                ))
                .border_1()
                .border_color(rgba(tab_border_rgba))
                .flex()
                .items_center()
                .justify_center()
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
    fn launcher_ask_ai_hint_uses_subtle_tab_keycap_chrome() {
        let source = fs::read_to_string("src/components/launcher_ask_ai_hint.rs")
            .expect("Failed to read src/components/launcher_ask_ai_hint.rs");
        // Only inspect the render function body. Assertions below reference the
        // substrings literally, so scanning the whole file would self-match.
        let render_body = source
            .split("#[cfg(test)]")
            .next()
            .expect("launcher ask-ai hint source should contain a render body");

        assert!(
            render_body.contains("quieter than footer keycaps"),
            "launcher ask-ai hint should document its softer-than-footer visual target"
        );
        assert!(
            render_body.contains("chrome.text_muted_hex")
                && render_body.contains("0x92")
                && render_body.contains("0x9c")
                && render_body.contains("0x44"),
            "launcher ask-ai hint should use theme-muted Ask/tab/border opacities"
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
            render_body.contains(".border_1()")
                && render_body.contains(".border_color(rgba(tab_border_rgba))")
                && render_body.contains("FOOTER_KEYCAP_RADIUS_PX")
                && render_body.contains("FOOTER_KEYCAP_HEIGHT_PX")
                && !render_body.contains(".bg(rgba("),
            "launcher ask-ai hint should render a footer-sized square bordered Tab keycap without a filled background"
        );
    }
}
