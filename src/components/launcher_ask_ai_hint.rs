use gpui::{
    div, px, rgba, ClickEvent, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, StatefulInteractiveElement as _, Styled as _,
};

use crate::components::footer_chrome::render_footer_keycap;
use crate::theme::Theme;

#[allow(dead_code)]
pub(crate) fn render_launcher_ask_ai_hint(
    theme: &Theme,
    on_click: impl Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    // Label uses the old Ask style: 15px, normal weight, soft muted opacity.
    // Keycaps use the shared footer_chrome renderer so they match the footer exactly.
    let label_rgba = (theme.colors.text.muted << 8) | 0x92;

    div()
        .id("agent-hint-button")
        .group("footer-action-button")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(5.0))
        .cursor_pointer()
        .on_click(on_click)
        .child(
            div()
                .text_size(px(15.0))
                .line_height(px(19.0))
                .font_weight(FontWeight::NORMAL)
                .text_color(rgba(label_rgba))
                .child("Agent"),
        )
        .child(render_footer_keycap("⌘".to_string(), None, theme))
        .child(render_footer_keycap("↵".to_string(), None, theme))
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn launcher_agent_hint_shows_cmd_enter_keycaps() {
        let source = fs::read_to_string("src/components/launcher_ask_ai_hint.rs")
            .expect("Failed to read src/components/launcher_ask_ai_hint.rs");

        assert!(
            source.contains(".child(\"Agent\")"),
            "launcher agent hint should show the Agent label"
        );
        assert!(
            source.contains("render_footer_keycap(\"⌘\""),
            "launcher agent hint should render Cmd as a footer keycap"
        );
        assert!(
            source.contains("render_footer_keycap(\"↵\""),
            "launcher agent hint should render Enter as a footer keycap"
        );
        assert!(
            source.contains(".cursor_pointer()"),
            "launcher agent hint should be clickable"
        );
        assert!(
            !source.contains(".child(\"⇥\")"),
            "launcher agent hint should not show old Tab badge"
        );
    }

    #[test]
    fn launcher_agent_hint_uses_soft_label_and_footer_keycaps() {
        let source = fs::read_to_string("src/components/launcher_ask_ai_hint.rs")
            .expect("Failed to read src/components/launcher_ask_ai_hint.rs");
        let render_body = source
            .split("#[cfg(test)]")
            .next()
            .expect("should have render body");

        assert!(
            render_body.contains("text_size(px(15.0))")
                && render_body.contains("FontWeight::NORMAL")
                && render_body.contains("0x92"),
            "label should use the soft Ask-style typography (15px, normal weight, 0x92 opacity)"
        );
        assert!(
            render_body.contains("render_footer_keycap"),
            "keycaps should use the shared footer_chrome renderer"
        );
    }
}
