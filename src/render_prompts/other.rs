// Other prompt render methods - extracted from render_prompts.rs
// Contains: select, env, drop, template prompts
// This file is included via include!() macro in main.rs

impl ScriptListApp {
    #[inline]
    fn other_prompt_shell_radius_lg(&self) -> f32 {
        get_tokens(self.current_design).visual().radius_lg
    }

    #[inline]
    fn other_prompt_shell_handle_key_default(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Hide cursor while typing - automatically shows when mouse moves
        self.hide_mouse_cursor(cx);

        let key = event.keystroke.key.as_str();
        let has_cmd = event.keystroke.modifiers.platform;

        if has_cmd && crate::ui_foundation::is_key_k(key) && self.sdk_actions.is_some() {
            self.toggle_arg_actions(cx, window);
            return;
        }

        // Global shortcuts (Cmd+W, ESC for dismissable prompts)
        // Other keys are handled by each prompt entity's own key handler.
        let _ = self.handle_global_shortcut_with_options(event, true, cx);
    }

    #[inline]
    fn other_prompt_shell_handle_key_chat(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Hide cursor while typing - automatically shows when mouse moves
        self.hide_mouse_cursor(cx);

        let key = event.keystroke.key.as_str();
        let key_char = event.keystroke.key_char.as_deref();
        let has_cmd = event.keystroke.modifiers.platform;
        let modifiers = &event.keystroke.modifiers;

        // Check for Cmd+K to toggle actions popup
        if has_cmd && crate::ui_foundation::is_key_k(key) {
            logging::log("KEY", "Cmd+K in ChatPrompt - calling toggle_chat_actions");
            self.toggle_chat_actions(cx, window);
            return;
        }

        // Route to shared actions dialog handler when open
        match self.route_key_to_actions_dialog(
            key,
            key_char,
            modifiers,
            ActionsDialogHost::ChatPrompt,
            window,
            cx,
        ) {
            ActionsRoute::Execute { action_id } => {
                // Handle chat-specific actions
                self.execute_chat_action(&action_id, cx);
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
        // Other keys are handled by the ChatPrompt entity's own key handler.
        let _ = self.handle_global_shortcut_with_options(event, true, cx);
    }

    #[inline]
    fn other_prompt_shell_handle_key_webcam(
        &mut self,
        event: &gpui::KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Hide cursor while typing - automatically shows when mouse moves
        self.hide_mouse_cursor(cx);

        // Global shortcuts (Cmd+W, ESC for dismissable prompts)
        // Note: Escape when actions popup is open is handled by central interceptor
        if !self.show_actions_popup {
            let _ = self.handle_global_shortcut_with_options(event, true, cx);
        }
    }

    fn render_select_prompt(
        &mut self,
        entity: Entity<SelectPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let shell_radius = self.other_prompt_shell_radius_lg();
        let handle_key = cx.listener(Self::other_prompt_shell_handle_key_default);

        let vibrancy_bg = get_vibrancy_background(&self.theme);

        // SelectPrompt entity has its own track_focus and on_key_down in its render method.
        // We wrap with our own handler to intercept Cmd+W and ESC first.
        crate::components::prompt_shell_container(shell_radius, vibrancy_bg)
            .on_key_down(handle_key)
            .child(crate::components::prompt_shell_content(entity))
            .into_any_element()
    }

    fn render_env_prompt(
        &mut self,
        entity: Entity<EnvPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let shell_radius = self.other_prompt_shell_radius_lg();
        let handle_key = cx.listener(Self::other_prompt_shell_handle_key_default);

        let vibrancy_bg = get_vibrancy_background(&self.theme);

        // EnvPrompt entity has its own track_focus and on_key_down in its render method.
        // We wrap with our own handler to intercept Cmd+W and ESC first.
        crate::components::prompt_shell_container(shell_radius, vibrancy_bg)
            .on_key_down(handle_key)
            .child(crate::components::prompt_shell_content(entity))
            .into_any_element()
    }

    fn render_drop_prompt(
        &mut self,
        entity: Entity<DropPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let shell_radius = self.other_prompt_shell_radius_lg();
        let handle_key = cx.listener(Self::other_prompt_shell_handle_key_default);

        let vibrancy_bg = get_vibrancy_background(&self.theme);

        // DropPrompt entity has its own track_focus and on_key_down in its render method.
        // We wrap with our own handler to intercept Cmd+W and ESC first.
        crate::components::prompt_shell_container(shell_radius, vibrancy_bg)
            .on_key_down(handle_key)
            .child(crate::components::prompt_shell_content(entity))
            .into_any_element()
    }

    fn render_template_prompt(
        &mut self,
        entity: Entity<TemplatePrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let shell_radius = self.other_prompt_shell_radius_lg();
        let handle_key = cx.listener(Self::other_prompt_shell_handle_key_default);

        let vibrancy_bg = get_vibrancy_background(&self.theme);

        // TemplatePrompt entity has its own track_focus and on_key_down in its render method.
        // We wrap with our own handler to intercept Cmd+W and ESC first.
        crate::components::prompt_shell_container(shell_radius, vibrancy_bg)
            .on_key_down(handle_key)
            .child(crate::components::prompt_shell_content(entity))
            .into_any_element()
    }

    fn render_chat_prompt(
        &mut self,
        entity: Entity<prompts::ChatPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let shell_radius = self.other_prompt_shell_radius_lg();
        let handle_key = cx.listener(Self::other_prompt_shell_handle_key_chat);

        let vibrancy_bg = get_vibrancy_background(&self.theme);

        // ChatPrompt entity has its own track_focus and on_key_down in its render method.
        // We wrap with our own handler to intercept Cmd+K and route actions first.
        crate::components::prompt_shell_container(shell_radius, vibrancy_bg)
            .on_key_down(handle_key)
            .child(crate::components::prompt_shell_content(entity))
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
        let shell_radius = tokens.visual().radius_lg;
        let handle_key = cx.listener(Self::other_prompt_shell_handle_key_webcam);

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(&self.theme);

        // Use explicit height from layout constants
        let content_height = window_resize::layout::STANDARD_HEIGHT;

        // Footer colors and handlers for PromptFooter (same shared prompt footer pattern)
        let footer_colors =
            prompt_footer_colors_for_prompt(&design_colors, !self.theme.is_dark_mode());

        // Footer config: Capture Photo as primary action, always show Actions button
        let footer_config = prompt_footer_config_with_status(
            "Capture Photo",
            true,
            Some(running_status_text("camera ready, press Enter to capture")),
            Some("Webcam".to_string()),
        )
        // Keep explicit label for source-based regression tests.
        .primary_label("Capture Photo");

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
            .rounded(px(shell_radius))
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
                                if this.capture_webcam_photo(cx) {
                                    this.hide_main_and_reset(cx);
                                }
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

#[cfg(test)]
mod other_prompt_render_wrapper_tests {
    const OTHER_RENDERERS_SOURCE: &str = include_str!("other.rs");

    fn fn_source(name: &str) -> &'static str {
        let marker = format!("fn {}(", name);
        let start = OTHER_RENDERERS_SOURCE
            .find(&marker)
            .unwrap_or_else(|| panic!("missing function: {}", name));
        let tail = &OTHER_RENDERERS_SOURCE[start..];
        let end = tail.find("\n    fn ").unwrap_or(tail.len());
        &tail[..end]
    }

    #[test]
    fn simple_prompt_wrappers_skip_unused_shell_allocations() {
        for fn_name in [
            "render_select_prompt",
            "render_env_prompt",
            "render_drop_prompt",
            "render_template_prompt",
            "render_chat_prompt",
        ] {
            let body = fn_source(fn_name);
            assert!(
                !body.contains("hex_to_rgba_with_opacity"),
                "{fn_name} should not compute unused background opacity in the shell wrapper"
            );
            assert!(
                !body.contains("create_box_shadows"),
                "{fn_name} should not allocate unused box shadows in the shell wrapper"
            );
        }
    }
}
