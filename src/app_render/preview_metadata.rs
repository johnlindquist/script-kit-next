impl ScriptListApp {
    fn render_actions_dialog(&mut self, cx: &mut Context<Self>) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        // Removed: box_shadows - shadows on transparent elements block vibrancy
        let _box_shadows = self.create_box_shadows();

        // Key handler for actions dialog
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                // Global shortcuts (Cmd+W, ESC closes window from ActionsDialog too)
                // ActionsDialog has no other key handling, so we just call the global handler
                let _ = this.handle_global_shortcut_with_options(event, true, cx);
            },
        );

        // Simple actions dialog stub with design tokens
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            // Removed: .shadow(box_shadows) - shadows on transparent elements block vibrancy
            .rounded(px(design_visual.radius_lg))
            .p(px(design_spacing.padding_xl))
            .text_color(rgb(design_colors.text_primary))
            .font_family(design_typography.font_family)
            .key_context("actions_dialog")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(div().text_lg().child("Actions (Cmd+K)"))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(design_colors.text_muted))
                    .mt(px(design_spacing.margin_md))
                    .child("• Create script\n• Edit script\n• Reload\n• Settings\n• Quit"),
            )
            .child(
                div()
                    .mt(px(design_spacing.margin_lg))
                    .text_xs()
                    .text_color(rgb(design_colors.text_dimmed))
                    .child("Press Esc to close"),
            )
            .into_any_element()
    }
}
