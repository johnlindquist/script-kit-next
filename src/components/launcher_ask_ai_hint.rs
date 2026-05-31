use gpui::{
    div, px, rgba, ClickEvent, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _,
};

use crate::components::footer_chrome::{
    render_footer_hint_content, themed_footer_button_active_rgba, themed_footer_button_hover_rgba,
    themed_footer_button_rest_rgba, FooterHintKeyMode,
};
use crate::theme::Theme;

#[allow(dead_code)]
pub(crate) fn render_launcher_ask_ai_hint(
    theme: &Theme,
    on_click: impl Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    let active_rgba = themed_footer_button_active_rgba(theme);
    let footer_metrics = crate::components::footer_chrome::current_main_menu_footer_metrics();
    let mut button = div()
        .id("agent-hint-button")
        .group("footer-action-button")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(footer_metrics.content_gap))
        .cursor_pointer()
        .on_click(on_click)
        .rounded(px(footer_metrics.button_radius))
        .hover({
            let hover_rgba = themed_footer_button_hover_rgba(theme);
            move |s| s.bg(rgba(hover_rgba))
        })
        .child(render_footer_hint_content(
            "Agent".into(),
            "⌘↵".into(),
            FooterHintKeyMode::Shortcut,
            theme,
        ));

    if let Some(rest_rgba) = themed_footer_button_rest_rgba(theme) {
        button = button.bg(rgba(rest_rgba));
    }

    button.active(move |s| s.bg(rgba(active_rgba)))
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn launcher_agent_hint_shows_cmd_enter_keycaps() {
        let source = fs::read_to_string("src/components/launcher_ask_ai_hint.rs")
            .expect("Failed to read src/components/launcher_ask_ai_hint.rs");

        assert!(
            source.contains("\"Agent\".into()"),
            "launcher agent hint should show the Agent label"
        );
        assert!(
            source.contains("\"⌘↵\".into()"),
            "launcher agent hint should render Cmd as a footer keycap"
        );
        assert!(
            source.contains("\"⌘↵\".into()"),
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
    fn launcher_agent_hint_uses_footer_button_theme_renderer() {
        let source = fs::read_to_string("src/components/launcher_ask_ai_hint.rs")
            .expect("Failed to read src/components/launcher_ask_ai_hint.rs");
        let render_body = source
            .split("#[cfg(test)]")
            .next()
            .expect("should have render body");

        assert!(
            !render_body.contains("text_size(px(15.0))")
                && !render_body.contains("FontWeight::NORMAL")
                && !render_body.contains("0x92"),
            "Agent hint must not carry local typography/opacity that can drift from footer buttons"
        );
        assert!(
            render_body.contains("render_footer_hint_content")
                && render_body.contains("themed_footer_button_rest_rgba")
                && render_body.contains("themed_footer_button_hover_rgba")
                && render_body.contains("themed_footer_button_active_rgba"),
            "Agent hint should use the shared footer button renderer and theme state"
        );
    }
}
