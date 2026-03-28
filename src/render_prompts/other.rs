mod __render_prompts_other_docs {
    //! Render methods for non-arg prompts: select, env, drop, template, chat, and webcam.
    //! Key APIs are the `render_*_prompt` methods and shared key-routing helpers for actions dialogs.
    //! This fragment depends on prompt entities, action routing, and shell container components from `main.rs`.
}

// Other prompt render methods - extracted from render_prompts.rs
// Contains: select, env, drop, template prompts
// This file is included via include!() macro in main.rs


impl ScriptListApp {
    #[inline]
    fn other_prompt_shell_radius_lg(&self) -> f32 {
        PromptRenderContext::new(self.theme.as_ref(), self.current_design)
            .design_visual
            .radius_lg
    }

    #[inline]
    fn has_nonempty_sdk_actions(&self) -> bool {
        self.sdk_actions.as_ref().is_some_and(|a| !a.is_empty())
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

    fn render_wrapped_prompt_entity(
        &mut self,
        entity: impl IntoElement,
        key_handler: impl Fn(&mut Self, &gpui::KeyDownEvent, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let shell_radius = self.other_prompt_shell_radius_lg();
        let handle_key = cx.listener(key_handler);
        let vibrancy_bg = get_vibrancy_background(&self.theme);

        crate::components::render_simple_prompt_shell(shell_radius, vibrancy_bg, entity, None)
            .on_key_down(handle_key)
            .into_any_element()
    }

    fn render_select_prompt(
        &mut self,
        entity: Entity<SelectPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "render_prompts::select",
                self.has_nonempty_sdk_actions(),
            ),
        );
        self.render_wrapped_prompt_entity(entity, Self::other_prompt_shell_handle_key_default, cx)
    }

    fn render_env_prompt(
        &mut self,
        entity: Entity<EnvPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "render_prompts::env",
                self.has_nonempty_sdk_actions(),
            ),
        );
        self.render_wrapped_prompt_entity(entity, Self::other_prompt_shell_handle_key_default, cx)
    }

    fn render_drop_prompt(
        &mut self,
        entity: Entity<DropPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "render_prompts::drop",
                self.has_nonempty_sdk_actions(),
            ),
        );
        self.render_wrapped_prompt_entity(entity, Self::other_prompt_shell_handle_key_default, cx)
    }

    fn render_template_prompt(
        &mut self,
        entity: Entity<TemplatePrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "template_prompt",
                self.has_nonempty_sdk_actions(),
            ),
        );
        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let theme = render_context.theme;
        let design_spacing = render_context.design_spacing;
        let shell_radius = render_context.design_visual.radius_lg;
        let vibrancy_bg = get_vibrancy_background(theme);

        crate::components::prompt_shell_container(shell_radius, vibrancy_bg)
            .h(window_resize::layout::STANDARD_HEIGHT)
            .on_key_down(cx.listener(Self::other_prompt_shell_handle_key_default))
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.))
                    .overflow_y_scrollbar()
                    .p(px(design_spacing.padding_xl))
                    .child(entity),
            )
            .child(crate::components::render_simple_hint_strip(
                crate::components::universal_prompt_hints(),
                None,
            ))
            .into_any_element()
    }

    fn render_chat_prompt(
        &mut self,
        entity: Entity<prompts::ChatPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "render_prompts::chat",
                self.has_nonempty_sdk_actions(),
            ),
        );
        self.render_wrapped_prompt_entity(entity, Self::other_prompt_shell_handle_key_chat, cx)
    }

    fn render_naming_prompt(
        &mut self,
        entity: Entity<prompts::NamingPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "naming_prompt",
                self.has_nonempty_sdk_actions(),
            ),
        );
        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let theme = render_context.theme;
        let design_spacing = render_context.design_spacing;
        let shell_radius = render_context.design_visual.radius_lg;
        let vibrancy_bg = get_vibrancy_background(theme);

        crate::components::prompt_shell_container(shell_radius, vibrancy_bg)
            .h(window_resize::layout::STANDARD_HEIGHT)
            .on_key_down(cx.listener(Self::other_prompt_shell_handle_key_default))
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.))
                    .overflow_y_scrollbar()
                    .p(px(design_spacing.padding_xl))
                    .child(entity),
            )
            .child(crate::components::render_simple_hint_strip(
                crate::components::universal_prompt_hints(),
                None,
            ))
            .into_any_element()
    }

    fn render_webcam_prompt(
        &mut self,
        entity: Entity<prompts::WebcamPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::exception(
                "webcam_prompt",
                "media_capture_surface_with_hint_strip",
            ),
        );
        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let theme = render_context.theme;
        let shell_radius = render_context.design_visual.radius_lg;
        let handle_key = cx.listener(Self::other_prompt_shell_handle_key_webcam);

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(theme);

        // Use explicit height from layout constants
        let content_height = window_resize::layout::STANDARD_HEIGHT;

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
            // Shared three-key hint strip footer
            .child(crate::components::render_simple_hint_strip(
                crate::components::universal_prompt_hints(),
                None,
            ))
            .into_any_element()
    }

    pub(crate) fn render_creation_feedback(
        &mut self,
        path: std::path::PathBuf,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "creation_feedback",
                self.has_nonempty_sdk_actions(),
            ),
        );
        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let theme = self.theme.clone();
        let design_spacing = render_context.design_spacing;
        let shell_radius = render_context.design_visual.radius_lg;
        let vibrancy_bg = get_vibrancy_background(render_context.theme);

        let entity = cx.entity().downgrade();

        let panel = prompts::CreationFeedbackPanel::new(path, theme)
            .design_variant(self.current_design)
            .on_reveal_in_finder(Box::new(move |p, _window, _cx| {
                if let Err(e) = crate::platform::reveal_in_finder(p) {
                    tracing::warn!(error = %e, "reveal_in_finder failed");
                }
            }))
            .on_copy_path(Box::new(move |p, _window, cx| {
                if let Err(e) = crate::platform::copy_text_to_clipboard(&p.to_string_lossy()) {
                    tracing::warn!(error = %e, "copy_text_to_clipboard failed");
                } else if let Some(app) = entity.upgrade() {
                    app.update(cx, |this, cx| {
                        this.show_hud("Path copied".to_string(), None, cx);
                    });
                }
            }))
            .on_open(Box::new(move |p, _window, _cx| {
                if let Err(e) = crate::platform::open_in_default_app(p) {
                    tracing::warn!(error = %e, "open_in_default_app failed");
                }
            }));

        // Handle Enter/Escape to dismiss
        let handle_key = cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
            let key = event.keystroke.key.as_str();
            if crate::ui_foundation::is_key_escape(key) || key.eq_ignore_ascii_case("enter") {
                this.go_back_or_close(window, cx);
            }
        });

        crate::components::prompt_shell_container(shell_radius, vibrancy_bg)
            .id("creation-feedback-shell")
            .h(window_resize::layout::STANDARD_HEIGHT)
            .key_context("CreationFeedback")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.))
                    .overflow_y_scrollbar()
                    .p(px(design_spacing.padding_xl))
                    .child(panel),
            )
            .child(crate::components::render_simple_hint_strip(
                crate::components::universal_prompt_hints(),
                None,
            ))
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
            "render_chat_prompt",
        ] {
            let body = fn_source(fn_name);
            assert!(
                body.contains("render_wrapped_prompt_entity("),
                "{fn_name} should delegate to render_wrapped_prompt_entity"
            );
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

    #[test]
    fn render_wrapped_prompt_entity_calls_shared_shell_helper() {
        let body = fn_source("render_wrapped_prompt_entity");
        assert!(
            body.contains("crate::components::render_simple_prompt_shell("),
            "render_wrapped_prompt_entity must call the shared component helper explicitly"
        );
    }

    #[test]
    fn render_template_prompt_uses_hint_strip() {
        let body = fn_source("render_template_prompt");
        assert!(
            !body.contains("PromptFooter::new("),
            "render_template_prompt should not use PromptFooter"
        );
        assert!(
            body.contains("render_simple_hint_strip("),
            "render_template_prompt should use the shared hint strip"
        );
        assert!(
            body.contains("window_resize::layout::STANDARD_HEIGHT"),
            "render_template_prompt should use STANDARD_HEIGHT"
        );
    }

    #[test]
    fn render_naming_prompt_uses_hint_strip() {
        let body = fn_source("render_naming_prompt");
        assert!(
            !body.contains("PromptFooter::new("),
            "render_naming_prompt should not use PromptFooter"
        );
        assert!(
            body.contains("render_simple_hint_strip("),
            "render_naming_prompt should use the shared hint strip"
        );
        assert!(
            body.contains("window_resize::layout::STANDARD_HEIGHT"),
            "render_naming_prompt should use STANDARD_HEIGHT"
        );
    }

    #[test]
    fn render_creation_feedback_uses_hint_strip() {
        let body = fn_source("render_creation_feedback");
        assert!(
            body.contains("window_resize::layout::STANDARD_HEIGHT"),
            "render_creation_feedback should use STANDARD_HEIGHT"
        );
        assert!(
            body.contains("prompt_shell_container("),
            "render_creation_feedback should use prompt_shell_container"
        );
        assert!(
            !body.contains("PromptFooter::new("),
            "render_creation_feedback should not use PromptFooter"
        );
        assert!(
            body.contains("render_simple_hint_strip("),
            "render_creation_feedback should use the shared hint strip"
        );
    }

    #[test]
    fn render_webcam_prompt_uses_hint_strip() {
        let body = fn_source("render_webcam_prompt");
        assert!(
            !body.contains("PromptFooter::new("),
            "render_webcam_prompt should not use PromptFooter"
        );
        assert!(
            body.contains("render_simple_hint_strip("),
            "render_webcam_prompt should use the shared hint strip"
        );
    }
}

#[cfg(test)]
mod prompt_footer_regression_tests {
    use std::fs;

    #[test]
    fn form_prompt_no_longer_uses_prompt_footer() {
        let source = fs::read_to_string("src/render_prompts/form/render.rs")
            .expect("Failed to read src/render_prompts/form/render.rs");
        assert!(
            source.contains("render_simple_hint_strip(")
                || source.contains("render_minimal_list_prompt_shell("),
            "form prompt should render the shared hint strip"
        );
        assert!(
            !source.contains("PromptFooter::new("),
            "form prompt should not keep PromptFooter after migration"
        );
    }

    #[test]
    fn term_prompt_no_longer_uses_prompt_footer() {
        let source = fs::read_to_string("src/render_prompts/term.rs")
            .expect("Failed to read src/render_prompts/term.rs");
        assert!(
            source.contains("render_simple_hint_strip("),
            "term prompt should render the shared hint strip"
        );
        assert!(
            !source.contains("PromptFooter::new("),
            "term prompt should not keep PromptFooter after migration"
        );
    }

    #[test]
    fn other_rs_exceptions_are_spec_blessed_only() {
        let source = fs::read_to_string("src/render_prompts/other.rs")
            .expect("Failed to read src/render_prompts/other.rs");
        assert!(
            !source.contains("PromptFooter::new("),
            "other.rs should not contain any PromptFooter::new after migration"
        );
        // Remaining exceptions must be spec-blessed surfaces (terminal, webcam media)
        for line in source.lines() {
            if line.contains("PromptChromeAudit::exception(") {
                assert!(
                    line.contains("webcam_prompt")
                        || line.contains("terminal")
                        || line.contains("editor")
                        || line.contains("grid")
                        || line.contains("expanded"),
                    "non-blessed exception found in other.rs: {line}"
                );
            }
        }
    }
}
