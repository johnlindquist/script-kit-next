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
        // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
        // Shadows are handled by app_shell
        let _box_shadows = self.create_box_shadows();

        // Key handler for global shortcuts (Cmd+W, ESC)
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                if has_cmd && crate::ui_foundation::is_key_k(key) && this.sdk_actions.is_some() {
                    this.toggle_arg_actions(cx, window);
                    return;
                }

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
        // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
        // Shadows are handled by app_shell
        let _box_shadows = self.create_box_shadows();

        // Key handler for global shortcuts (Cmd+W, ESC)
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                if has_cmd && crate::ui_foundation::is_key_k(key) && this.sdk_actions.is_some() {
                    this.toggle_arg_actions(cx, window);
                    return;
                }

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
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
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
        // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
        // Shadows are handled by app_shell
        let _box_shadows = self.create_box_shadows();

        // Key handler for global shortcuts (Cmd+W, ESC)
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                if has_cmd && crate::ui_foundation::is_key_k(key) && this.sdk_actions.is_some() {
                    this.toggle_arg_actions(cx, window);
                    return;
                }

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
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
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
        // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
        // Shadows are handled by app_shell
        let _box_shadows = self.create_box_shadows();

        // Key handler for global shortcuts (Cmd+W, ESC)
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                if has_cmd && crate::ui_foundation::is_key_k(key) && this.sdk_actions.is_some() {
                    this.toggle_arg_actions(cx, window);
                    return;
                }

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
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
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
        // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
        // Shadows are handled by app_shell
        let _box_shadows = self.create_box_shadows();

        // Key handler for global shortcuts and ⌘K to open actions
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                let key = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;
                let modifiers = &event.keystroke.modifiers;

                // Check for Cmd+K to toggle actions popup
                if has_cmd && crate::ui_foundation::is_key_k(key) {
                    logging::log("KEY", "Cmd+K in ChatPrompt - calling toggle_chat_actions");
                    this.toggle_chat_actions(cx, window);
                    return;
                }

                // Route to shared actions dialog handler when open
                match this.route_key_to_actions_dialog(
                    key,
                    key_char,
                    modifiers,
                    ActionsDialogHost::ChatPrompt,
                    window,
                    cx,
                ) {
                    ActionsRoute::Execute { action_id } => {
                        // Handle chat-specific actions
                        this.execute_chat_action(&action_id, cx);
                        return;
                    }
                    ActionsRoute::Handled => {
                        // Key consumed by actions dialog
                        return;
                    }
                    ActionsRoute::NotHandled => {
                        // Actions popup not open - continue with normal handling
                    }
                }

                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                // Other keys are handled by the ChatPrompt entity's own key handler
                let _ = this.handle_global_shortcut_with_options(event, true, cx);
            },
        );

        // ChatPrompt entity has its own track_focus and on_key_down in its render method.
        // We wrap with our own handler to intercept Cmd+K and route actions first.
        div()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
            .w_full()
            .h_full()
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .on_key_down(handle_key)
            .child(div().size_full().child(entity))
            .into_any_element()
    }

    fn render_webcam_prompt(
        &mut self,
        entity: Entity<prompts::WebcamPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming (same pattern as DivPrompt)
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_visual = tokens.visual();

        // Key handler for webcam-specific shortcuts
        // Note: Cmd+K and actions dialog key routing are handled centrally in app_impl.rs
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                // Note: Escape when actions popup is open is handled by central interceptor
                if !this.show_actions_popup
                    && this.handle_global_shortcut_with_options(event, true, cx)
                {
                    return;
                }
            },
        );

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(&self.theme);

        // Use explicit height from layout constants
        let content_height = window_resize::layout::STANDARD_HEIGHT;

        // Footer colors and handlers for PromptFooter (same as DivPrompt)
        let footer_colors = PromptFooterColors {
            accent: design_colors.accent,
            text_muted: design_colors.text_muted,
            border: design_colors.border,
            background: design_colors.background_selected,
            is_light_mode: !self.theme.is_dark_mode(),
        };

        // Footer config: Capture as primary action, always show Actions button
        let footer_config = PromptFooterConfig::new()
            .primary_label("Capture")
            .primary_shortcut("↵")
            .helper_text("Built-in")
            .show_secondary(true);

        // Create click handlers for footer
        let handle_submit = cx.entity().downgrade();
        let handle_actions = cx.entity().downgrade();

        div()
            .flex()
            .flex_col()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg))
            .w_full()
            .h(content_height)
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Content area - flex-1 to fill remaining space above footer
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.))
                    .overflow_hidden()
                    .child(entity),
            )
            // Footer with Capture button and Actions (same pattern as all other prompts)
            .child(
                PromptFooter::new(footer_config, footer_colors)
                    .on_primary_click(Box::new(move |_, _window, cx| {
                        if let Some(app) = handle_submit.upgrade() {
                            app.update(cx, |this, cx| {
                                this.close_and_reset_window(cx);
                            });
                        }
                    }))
                    .on_secondary_click(Box::new(move |_, window, cx| {
                        if let Some(app) = handle_actions.upgrade() {
                            app.update(cx, |this, cx| {
                                this.toggle_webcam_actions(cx, window);
                            });
                        }
                    })),
            )
            .into_any_element()
    }
}
