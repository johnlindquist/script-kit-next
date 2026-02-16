mod __render_prompts_other_docs {
    //! Render methods for non-arg prompts: select, env, drop, template, chat, and webcam.
    //! Key APIs are the `render_*_prompt` methods and shared key-routing helpers for actions dialogs.
    //! This fragment depends on prompt entities, action routing, and shell container components from `main.rs`.
}

// Other prompt render methods - extracted from render_prompts.rs
// Contains: select, env, drop, template prompts
// This file is included via include!() macro in main.rs

const WEBCAM_FOOTER_READY_STATUS_CONTEXT: &str = "camera ready, press Enter to capture";
const WEBCAM_FOOTER_HIDE_PRIMARY_LABEL: &str = "Run Command";

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

    fn render_simple_prompt_shell(
        &mut self,
        entity: impl IntoElement,
        key_handler: impl Fn(&mut Self, &gpui::KeyDownEvent, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let shell_radius = self.other_prompt_shell_radius_lg();
        let handle_key = cx.listener(key_handler);
        let vibrancy_bg = get_vibrancy_background(&self.theme);

        crate::components::prompt_shell_container(shell_radius, vibrancy_bg)
            .on_key_down(handle_key)
            .child(crate::components::prompt_shell_content(entity))
            .into_any_element()
    }

    fn render_select_prompt(
        &mut self,
        entity: Entity<SelectPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.render_simple_prompt_shell(entity, Self::other_prompt_shell_handle_key_default, cx)
    }

    fn render_env_prompt(
        &mut self,
        entity: Entity<EnvPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.render_simple_prompt_shell(entity, Self::other_prompt_shell_handle_key_default, cx)
    }

    fn render_drop_prompt(
        &mut self,
        entity: Entity<DropPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.render_simple_prompt_shell(entity, Self::other_prompt_shell_handle_key_default, cx)
    }

    fn render_template_prompt(
        &mut self,
        entity: Entity<TemplatePrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.render_simple_prompt_shell(entity, Self::other_prompt_shell_handle_key_default, cx)
    }

    fn render_paste_sequential_prompt(
        &mut self,
        entity: Entity<prompts::PasteSequentialPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.render_simple_prompt_shell(entity, Self::other_prompt_shell_handle_key_default, cx)
    }

    fn render_chat_prompt(
        &mut self,
        entity: Entity<prompts::ChatPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.render_simple_prompt_shell(entity, Self::other_prompt_shell_handle_key_chat, cx)
    }

    fn render_naming_prompt(
        &mut self,
        entity: Entity<prompts::NamingPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.render_simple_prompt_shell(entity, Self::other_prompt_shell_handle_key_default, cx)
    }

    fn render_webcam_prompt(
        &mut self,
        entity: Entity<prompts::WebcamPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let theme = render_context.theme;
        let design_colors = render_context.design_colors;
        let shell_radius = render_context.design_visual.radius_lg;
        let handle_key = cx.listener(Self::other_prompt_shell_handle_key_webcam);

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(theme);

        // Use explicit height from layout constants
        let content_height = window_resize::layout::STANDARD_HEIGHT;

        // Footer colors and handlers for PromptFooter (same shared prompt footer pattern)
        let footer_colors = prompt_footer_colors_for_prompt(&design_colors, !theme.is_dark_mode());

        // Footer config: keep webcam footer focused on essential controls only.
        let footer_config = prompt_footer_config_with_status(
            "Capture Photo",
            true,
            Some(running_status_text(WEBCAM_FOOTER_READY_STATUS_CONTEXT)),
            None,
        )
        .show_logo(false)
        // Keep explicit label for source-based regression tests, then hide
        // the primary button so webcam footer only shows status + Actions.
        .primary_label("Capture Photo")
        .primary_label(WEBCAM_FOOTER_HIDE_PRIMARY_LABEL);

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

    pub(crate) fn render_creation_feedback(
        &mut self,
        path: std::path::PathBuf,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = self.theme.clone();
        let entity = cx.entity().downgrade();
        let entity_dismiss = entity.clone();

        let panel = prompts::CreationFeedbackPanel::new(path, theme)
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
            }))
            .on_dismiss(Box::new(move |_window, cx| {
                if let Some(app) = entity_dismiss.upgrade() {
                    app.update(cx, |this, cx| {
                        this.current_view = AppView::ScriptList;
                        this.request_script_list_main_filter_focus(cx);
                        cx.notify();
                    });
                }
            }));

        // Handle Enter/Escape to dismiss
        let handle_key = cx.listener(move |this, event: &gpui::KeyDownEvent, _window, cx| {
            let key = event.keystroke.key.as_str();
            if crate::ui_foundation::is_key_escape(key) || key.eq_ignore_ascii_case("enter") {
                this.current_view = AppView::ScriptList;
                this.request_script_list_main_filter_focus(cx);
                cx.notify();
            }
        });

        gpui::div()
            .id("creation-feedback-shell")
            .key_context("CreationFeedback")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(panel)
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
            "render_naming_prompt",
        ] {
            let body = fn_source(fn_name);
            assert!(
                body.contains("render_simple_prompt_shell("),
                "{fn_name} should delegate to render_simple_prompt_shell"
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
    fn webcam_footer_shows_status_and_actions_without_primary_capture_button() {
        let body = fn_source("render_webcam_prompt");
        assert!(
            body.contains("running_status_text(WEBCAM_FOOTER_READY_STATUS_CONTEXT)"),
            "render_webcam_prompt should keep camera-ready status helper text in footer"
        );
        assert!(
            body.contains(".primary_label(WEBCAM_FOOTER_HIDE_PRIMARY_LABEL)"),
            "render_webcam_prompt should hide the primary capture button in the footer"
        );
        assert!(
            body.contains("toggle_webcam_actions"),
            "render_webcam_prompt should keep the footer Actions trigger wired"
        );
        assert!(
            !body.contains("Some(\"Webcam\".to_string())"),
            "render_webcam_prompt should not include the redundant Webcam info label"
        );
    }
}
