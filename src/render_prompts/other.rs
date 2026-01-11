// Other prompt render methods - extracted from render_prompts.rs
// Contains: select, env, drop, template prompts
// This file is included via include!() macro in main.rs

impl ScriptListApp {
    fn render_select_prompt(
        &mut self,
        entity: Entity<SelectPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Key handler for global shortcuts (Cmd+W, ESC)
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                // Other keys are handled by the SelectPrompt entity's own key handler
                let _ = this.handle_global_shortcut_with_options(event, true, cx);
            },
        );

        // SelectPrompt entity has its own track_focus and on_key_down in its render method.
        // We wrap with our own handler to intercept Cmd+W and ESC first.
        div()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            .shadow(box_shadows)
            .w_full()
            .h_full()
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .on_key_down(handle_key)
            .child(div().size_full().child(entity))
            .into_any_element()
    }

    fn render_env_prompt(
        &mut self,
        entity: Entity<EnvPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Key handler for global shortcuts (Cmd+W, ESC)
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                // Other keys are handled by the EnvPrompt entity's own key handler
                let _ = this.handle_global_shortcut_with_options(event, true, cx);
            },
        );

        // EnvPrompt entity has its own track_focus and on_key_down in its render method.
        // We wrap with our own handler to intercept Cmd+W and ESC first.
        div()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            .shadow(box_shadows)
            .w_full()
            .h_full()
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .on_key_down(handle_key)
            .child(div().size_full().child(entity))
            .into_any_element()
    }

    fn render_drop_prompt(
        &mut self,
        entity: Entity<DropPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Key handler for global shortcuts (Cmd+W, ESC)
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                // Other keys are handled by the DropPrompt entity's own key handler
                let _ = this.handle_global_shortcut_with_options(event, true, cx);
            },
        );

        // DropPrompt entity has its own track_focus and on_key_down in its render method.
        // We wrap with our own handler to intercept Cmd+W and ESC first.
        div()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            .shadow(box_shadows)
            .w_full()
            .h_full()
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .on_key_down(handle_key)
            .child(div().size_full().child(entity))
            .into_any_element()
    }

    fn render_template_prompt(
        &mut self,
        entity: Entity<TemplatePrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Key handler for global shortcuts (Cmd+W, ESC)
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                // Other keys are handled by the TemplatePrompt entity's own key handler
                let _ = this.handle_global_shortcut_with_options(event, true, cx);
            },
        );

        // TemplatePrompt entity has its own track_focus and on_key_down in its render method.
        // We wrap with our own handler to intercept Cmd+W and ESC first.
        div()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            .shadow(box_shadows)
            .w_full()
            .h_full()
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .on_key_down(handle_key)
            .child(div().size_full().child(entity))
            .into_any_element()
    }

    fn render_chat_prompt(
        &mut self,
        entity: Entity<prompts::ChatPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Key handler for global shortcuts (Cmd+W, ESC)
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                // Other keys are handled by the ChatPrompt entity's own key handler
                let _ = this.handle_global_shortcut_with_options(event, true, cx);
            },
        );

        // ChatPrompt entity has its own track_focus and on_key_down in its render method.
        // We wrap with our own handler to intercept Cmd+W and ESC first.
        div()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            .shadow(box_shadows)
            .w_full()
            .h_full()
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .on_key_down(handle_key)
            .child(div().size_full().child(entity))
            .into_any_element()
    }
}
