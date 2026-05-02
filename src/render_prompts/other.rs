mod __render_prompts_other_docs {
    //! Render methods for non-arg prompts: select, env, drop, template, chat, and webcam.
    //! Key APIs are the `render_*_prompt` methods and shared key-routing helpers for actions dialogs.
    //! This fragment depends on prompt entities, action routing, and shell container components from `main.rs`.
}

// Other prompt render methods - extracted from render_prompts.rs
// Contains: select, env, drop, template prompts
// This file is included via include!() macro in main.rs

impl ScriptListApp {
    /// Zero-radius shell for whisper chrome (sharp edges per .impeccable.md).
    #[inline]
    fn other_prompt_shell_radius(&self) -> f32 {
        0.0
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

    /// Build the canonical three-key footer with click handlers wired to app actions.
    ///
    /// All three buttons route through `dispatch_main_window_footer_action` so
    /// that prompt footers use the same per-view routing as `Cmd+K` and the
    /// native mini-footer.
    fn clickable_universal_hint_strip(&self, cx: &mut Context<Self>) -> AnyElement {
        let on_run = cx.listener(|this, _: &gpui::ClickEvent, window, cx| {
            this.dispatch_main_window_footer_action(
                crate::footer_popup::FooterAction::Run,
                window,
                cx,
                "gpui_footer",
            );
        });
        let on_actions = cx.listener(|this, _: &gpui::ClickEvent, window, cx| {
            this.dispatch_main_window_footer_action(
                crate::footer_popup::FooterAction::Actions,
                window,
                cx,
                "gpui_footer",
            );
        });
        let on_ai = cx.listener(|this, _: &gpui::ClickEvent, window, cx| {
            this.dispatch_main_window_footer_action(
                crate::footer_popup::FooterAction::Ai,
                window,
                cx,
                "gpui_footer",
            );
        });
        crate::components::render_universal_prompt_hint_strip_clickable(on_run, on_actions, on_ai)
    }

    fn clickable_template_hint_strip(
        &self,
        entity: Entity<TemplatePrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let on_submit = cx.listener(|this, _: &gpui::ClickEvent, window, cx| {
            this.dispatch_main_window_footer_action(
                crate::footer_popup::FooterAction::Run,
                window,
                cx,
                "gpui_footer",
            );
        });
        let on_next_field = move |_: &gpui::ClickEvent, _window: &mut Window, cx: &mut gpui::App| {
            entity.update(cx, |prompt, cx| prompt.next_input(cx));
        };
        let on_actions = cx.listener(|this, _: &gpui::ClickEvent, window, cx| {
            this.dispatch_main_window_footer_action(
                crate::footer_popup::FooterAction::Actions,
                window,
                cx,
                "gpui_footer",
            );
        });

        crate::components::HintStrip::new(crate::components::template_prompt_hints())
            .on_hint_click(0, on_submit)
            .on_hint_click(1, on_next_field)
            .on_hint_click(2, on_actions)
            .into_any_element()
    }

    fn render_wrapped_prompt_entity(
        &mut self,
        entity: impl IntoElement,
        key_handler: impl Fn(&mut Self, &gpui::KeyDownEvent, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let shell_radius = self.other_prompt_shell_radius();
        let handle_key = cx.listener(key_handler);
        let vibrancy_bg = get_vibrancy_background(&self.theme);
        let footer = self.main_window_footer_slot(self.clickable_universal_hint_strip(cx));

        crate::components::render_simple_prompt_shell(shell_radius, vibrancy_bg, entity, footer)
            .on_key_down(handle_key)
            .into_any_element()
    }

    fn render_wrapped_prompt_entity_with_footer(
        &mut self,
        entity: impl IntoElement,
        footer_element: AnyElement,
        key_handler: impl Fn(&mut Self, &gpui::KeyDownEvent, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let shell_radius = self.other_prompt_shell_radius();
        let handle_key = cx.listener(key_handler);
        let vibrancy_bg = get_vibrancy_background(&self.theme);
        let footer = self.main_window_footer_slot(footer_element);

        crate::components::render_simple_prompt_shell(shell_radius, vibrancy_bg, entity, footer)
            .on_key_down(handle_key)
            .into_any_element()
    }

    fn render_select_prompt(
        &mut self,
        entity: Entity<SelectPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        tracing::info!(
            surface = "render_prompts::select",
            row_subtitle = "focus-only",
            row_accent_bar = "focused-only",
            trailing_metadata = "hint-text",
            "prompt_surface_rendered"
        );
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "render_prompts::select",
                self.has_nonempty_sdk_actions(),
            ),
        );
        let hints = crate::components::universal_prompt_hints();
        crate::components::emit_prompt_hint_audit("render_prompts::select", &hints);
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
        let hints = crate::components::universal_prompt_hints();
        crate::components::emit_prompt_hint_audit("render_prompts::env", &hints);
        self.render_wrapped_prompt_entity(entity, Self::other_prompt_shell_handle_key_default, cx)
    }

    fn render_drop_prompt(
        &mut self,
        entity: Entity<DropPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        tracing::info!(surface = "render_prompts::drop", "prompt_surface_rendered");
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "render_prompts::drop",
                self.has_nonempty_sdk_actions(),
            ),
        );
        let hints = crate::components::universal_prompt_hints();
        crate::components::emit_prompt_hint_audit("render_prompts::drop", &hints);
        self.render_wrapped_prompt_entity(entity, Self::other_prompt_shell_handle_key_default, cx)
    }

    fn render_template_prompt(
        &mut self,
        entity: Entity<TemplatePrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        tracing::info!(
            surface = "render_prompts::template",
            shell = "render_simple_prompt_shell",
            "prompt_surface_rendered"
        );
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "render_prompts::template",
                self.has_nonempty_sdk_actions(),
            ),
        );
        let hints = crate::components::template_prompt_hints();
        crate::components::emit_surface_prompt_hint_audit(
            "render_prompts::template",
            &hints,
            "template_submit_next_actions_footer",
        );
        let footer = self.clickable_template_hint_strip(entity.clone(), cx);
        self.render_wrapped_prompt_entity_with_footer(
            entity,
            footer,
            Self::other_prompt_shell_handle_key_default,
            cx,
        )
    }

    fn render_chat_prompt(
        &mut self,
        entity: Entity<prompts::ChatPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        tracing::info!(
            surface = "render_prompts::chat",
            footer_mode = "custom",
            footer_owner = "chat_prompt_internal",
            "prompt_surface_rendered"
        );

        crate::components::emit_prompt_chrome_audit(&crate::components::PromptChromeAudit {
            surface: "render_prompts::chat",
            layout_mode: "mini",
            input_mode: "bare",
            divider_mode: "none",
            footer_mode: "custom",
            header_padding_x: crate::ui::chrome::HEADER_PADDING_X as u16,
            header_padding_y: crate::ui::chrome::HEADER_PADDING_Y as u16,
            hint_count: 3,
            has_leading_status: true,
            has_actions: self.has_nonempty_sdk_actions(),
            exception_reason: Some("chat_prompt_renders_hint_strip_internally"),
        });

        // Chat renders its own hint-strip footer internally,
        // so pass None to avoid a duplicate footer in the shell.
        let shell_radius = self.other_prompt_shell_radius();
        let handle_key = cx.listener(Self::other_prompt_shell_handle_key_chat);
        let vibrancy_bg = get_vibrancy_background(&self.theme);

        crate::components::render_simple_prompt_shell(shell_radius, vibrancy_bg, entity, None)
            .on_key_down(handle_key)
            .into_any_element()
    }

    fn render_naming_prompt(
        &mut self,
        entity: Entity<prompts::NamingPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        tracing::info!(
            surface = "render_prompts::naming",
            shell = "render_simple_prompt_shell",
            "prompt_surface_rendered"
        );
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "render_prompts::naming",
                self.has_nonempty_sdk_actions(),
            ),
        );
        let hints = crate::components::universal_prompt_hints();
        crate::components::emit_prompt_hint_audit("render_prompts::naming", &hints);
        self.render_wrapped_prompt_entity(entity, Self::other_prompt_shell_handle_key_default, cx)
    }

    fn render_webcam_prompt(
        &mut self,
        entity: Entity<prompts::WebcamPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::exception(
                "render_prompts::webcam",
                "media_capture_surface_with_hint_strip",
            ),
        );
        let hints = crate::components::universal_prompt_hints();
        crate::components::emit_prompt_hint_audit("render_prompts::webcam", &hints);
        let theme = PromptRenderContext::new(self.theme.as_ref(), self.current_design).theme;
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
            .rounded(px(0.0))
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
            // Shared three-key hint strip footer (native or GPUI)
            .when_some(
                self.main_window_footer_slot(self.clickable_universal_hint_strip(cx)),
                |d, footer| d.child(footer),
            )
            .into_any_element()
    }

    pub(crate) fn render_script_issues_view(
        &mut self,
        report: std::sync::Arc<crate::scripts::ValidationReport>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "script_issues_view",
                self.has_nonempty_sdk_actions(),
            ),
        );
        let hints = crate::components::universal_prompt_hints();
        crate::components::emit_prompt_hint_audit("render_prompts::script_issues", &hints);

        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let design_spacing = render_context.design_spacing;
        let vibrancy_bg = get_vibrancy_background(render_context.theme);
        let text_color = rgba((self.theme.colors.text.primary << 8) | 0xFF);
        let muted_color = rgba((self.theme.colors.text.muted << 8) | 0xFF);
        let accent_color = rgba((self.theme.colors.accent.selected << 8) | 0xFF);
        let panel_bg = rgba((self.theme.colors.accent.selected_subtle << 8) | 0x22);
        let border_color = rgba((self.theme.colors.ui.border << 8) | 0x40);

        let failed_count = report.failed_scripts.len();
        let header_summary = format!(
            "{} script{} failed · {} fatal · {} warning{}",
            failed_count,
            if failed_count == 1 { "" } else { "s" },
            report.fatal_count,
            report.warning_count,
            if report.warning_count == 1 { "" } else { "s" },
        );

        // Build the per-script blocks.
        let mut blocks: Vec<AnyElement> = Vec::new();
        for failed in report.failed_scripts.iter() {
            let path_str = failed.path.display().to_string();
            let name = failed.name.clone();

            let mut issue_lines: Vec<AnyElement> = Vec::new();
            for issue in failed.fatal.iter() {
                let detail = Self::script_issue_detail_line(issue);
                issue_lines.push(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .child(
                            div()
                                .text_color(text_color)
                                .text_size(px(13.))
                                .child(issue.message.clone()),
                        )
                        .when(!detail.is_empty(), |d| {
                            d.child(
                                div()
                                    .text_color(muted_color)
                                    .text_size(px(11.))
                                    .child(detail),
                            )
                        })
                        .when(!issue.related.is_empty(), |d| {
                            let related: Vec<AnyElement> = issue
                                .related
                                .iter()
                                .map(|r| {
                                    div()
                                        .text_color(muted_color)
                                        .text_size(px(11.))
                                        .child(format!("↔ {} — {}", r.name, r.path.display()))
                                        .into_any_element()
                                })
                                .collect();
                            d.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.))
                                    .pl(px(12.))
                                    .children(related),
                            )
                        })
                        .into_any_element(),
                );
            }

            blocks.push(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .p(px(design_spacing.padding_lg))
                    .bg(panel_bg)
                    .border_1()
                    .border_color(border_color)
                    .rounded(px(6.))
                    .child(
                        div()
                            .text_color(accent_color)
                            .text_size(px(14.))
                            .child(name),
                    )
                    .child(
                        div()
                            .text_color(muted_color)
                            .text_size(px(11.))
                            .child(path_str),
                    )
                    .children(issue_lines)
                    .into_any_element(),
            );
        }

        let empty_state: Option<AnyElement> = if failed_count == 0 {
            Some(
                div()
                    .text_color(muted_color)
                    .text_size(px(13.))
                    .child("No script issues — every script passed validation.")
                    .into_any_element(),
            )
        } else {
            None
        };

        // Key handler: Escape → back to ScriptList; Cmd+C → copy diagnostics text.
        let entity = cx.entity().downgrade();
        let report_for_keys = report.clone();
        let handle_key = cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
            let key = event.keystroke.key.as_str();
            let has_cmd = event.keystroke.modifiers.platform;
            if crate::ui_foundation::is_key_escape(key) {
                this.go_back_or_close(window, cx);
                return;
            }
            if has_cmd && key.eq_ignore_ascii_case("c") {
                let text = Self::format_script_issues_diagnostics(&report_for_keys);
                match crate::platform::copy_text_to_clipboard(&text) {
                    Ok(()) => {
                        if let Some(app) = entity.upgrade() {
                            app.update(cx, |this, cx| {
                                this.show_hud(
                                    "Diagnostics copied".to_string(),
                                    Some(2000),
                                    cx,
                                );
                            });
                        }
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "script_issues copy_text_to_clipboard failed");
                    }
                }
            }
        });

        crate::components::prompt_shell_container(0.0, vibrancy_bg)
            .id("script-issues-shell")
            .h(window_resize::layout::STANDARD_HEIGHT)
            .key_context("ScriptIssuesView")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(design_spacing.padding_md))
                    .flex_1()
                    .w_full()
                    .min_h(px(0.))
                    .overflow_y_scrollbar()
                    .p(px(design_spacing.padding_xl))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(
                                div()
                                    .text_color(text_color)
                                    .text_size(px(16.))
                                    .child("Script Issues"),
                            )
                            .child(
                                div()
                                    .text_color(muted_color)
                                    .text_size(px(12.))
                                    .child(header_summary),
                            ),
                    )
                    .when_some(empty_state, |d, el| d.child(el))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(design_spacing.padding_sm))
                            .children(blocks),
                    ),
            )
            .when_some(
                self.main_window_footer_slot(self.clickable_universal_hint_strip(cx)),
                |d, footer| d.child(footer),
            )
            .into_any_element()
    }

    fn script_issue_detail_line(issue: &crate::scripts::ScriptValidationIssue) -> String {
        let field = issue
            .field
            .map(|f| format!("[{:?}] ", f))
            .unwrap_or_default();
        let kind_detail = match &issue.kind {
            crate::scripts::ScriptValidationKind::MetadataParse { detail }
            | crate::scripts::ScriptValidationKind::SchemaParse { detail } => detail.clone(),
            crate::scripts::ScriptValidationKind::InvalidValue { value, reason } => {
                format!("value={value:?} — {reason}")
            }
            crate::scripts::ScriptValidationKind::DuplicateBinding { binding, value } => {
                format!("{:?} duplicate: {:?}", binding, value)
            }
        };
        if field.is_empty() && kind_detail.is_empty() {
            String::new()
        } else {
            format!("{field}{kind_detail}").trim().to_string()
        }
    }

    pub(crate) fn format_script_issues_diagnostics(
        report: &crate::scripts::ValidationReport,
    ) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "Script Issues — {} failed · {} fatal · {} warning(s)\n",
            report.failed_scripts.len(),
            report.fatal_count,
            report.warning_count,
        ));
        if report.failed_scripts.is_empty() {
            out.push_str("No failing scripts in this report.\n");
            return out;
        }
        for failed in report.failed_scripts.iter() {
            out.push('\n');
            out.push_str(&format!(
                "## {}\n  path: {}\n",
                failed.name,
                failed.path.display()
            ));
            for issue in failed.fatal.iter() {
                let field = issue
                    .field
                    .map(|f| format!("[{:?}] ", f))
                    .unwrap_or_default();
                out.push_str(&format!("  - {field}{}\n", issue.message));
                let kind_detail = match &issue.kind {
                    crate::scripts::ScriptValidationKind::MetadataParse { detail }
                    | crate::scripts::ScriptValidationKind::SchemaParse { detail } => {
                        detail.clone()
                    }
                    crate::scripts::ScriptValidationKind::InvalidValue { value, reason } => {
                        format!("value={value:?} — {reason}")
                    }
                    crate::scripts::ScriptValidationKind::DuplicateBinding { binding, value } => {
                        format!("{:?} duplicate: {:?}", binding, value)
                    }
                };
                if !kind_detail.is_empty() {
                    out.push_str(&format!("      kind: {kind_detail}\n"));
                }
                for related in issue.related.iter() {
                    out.push_str(&format!(
                        "      ↔ {} — {}\n",
                        related.name,
                        related.path.display()
                    ));
                }
            }
        }
        out
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
        let hints = crate::components::universal_prompt_hints();
        crate::components::emit_prompt_hint_audit("render_prompts::creation_feedback", &hints);
        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let theme = self.theme.clone();
        let design_spacing = render_context.design_spacing;
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

        crate::components::prompt_shell_container(0.0, vibrancy_bg)
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
            .when_some(
                self.main_window_footer_slot(self.clickable_universal_hint_strip(cx)),
                |d, footer| d.child(footer),
            )
            .into_any_element()
    }

    /// In-window confirm surface — replaces the popup confirm dialog when the
    /// main window is the active context. Title + body in the main content
    /// area; footer reuses the native AppKit footer with Apply/Close buttons
    /// labeled per [`ParentConfirmOptions`].
    pub(crate) fn render_confirm_prompt(
        &mut self,
        options: crate::confirm::ParentConfirmOptions,
        focused_button: ConfirmFocusedButton,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use gpui_component::button::ButtonVariant;

        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let theme = self.theme.clone();
        let design_spacing = render_context.design_spacing;
        let vibrancy_bg = get_vibrancy_background(render_context.theme);

        let is_danger = matches!(options.confirm_variant, ButtonVariant::Danger);
        let title_color = rgba(
            ((if is_danger {
                theme.colors.ui.error
            } else {
                theme.colors.text.primary
            }) << 8)
                | 0xFF,
        );
        let body_color = rgba((theme.colors.text.secondary << 8) | 0xFF);

        let handle_key = cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
            let key = event.keystroke.key.as_str();
            if crate::ui_foundation::is_key_escape(key) {
                this.resolve_confirm_prompt(false, window, cx);
                cx.stop_propagation();
            } else if key.eq_ignore_ascii_case("enter") {
                let confirm = matches!(
                    this.confirm_prompt_focused_button(),
                    Some(ConfirmFocusedButton::Confirm),
                );
                this.resolve_confirm_prompt(confirm, window, cx);
                cx.stop_propagation();
            } else if key.eq_ignore_ascii_case("tab") {
                this.toggle_confirm_prompt_focus(cx);
                cx.stop_propagation();
            }
        });

        let _ = focused_button; // focus state is reflected via the native footer

        crate::components::prompt_shell_container(0.0, vibrancy_bg)
            .id("confirm-prompt-shell")
            .h(window_resize::layout::STANDARD_HEIGHT)
            .key_context("ConfirmPrompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(design_spacing.padding_md))
                    .p(px(design_spacing.padding_xl))
                    .child(
                        div()
                            .text_color(title_color)
                            .text_size(px(20.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(options.title.clone()),
                    )
                    .child(
                        div()
                            .max_w(px(560.))
                            .text_color(body_color)
                            .text_size(px(14.))
                            .text_center()
                            .child(options.body.clone()),
                    ),
            )
            .when_some(
                self.main_window_footer_slot(self.clickable_universal_hint_strip(cx)),
                |d, footer| d.child(footer),
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
        // Chat is excluded: it uses render_simple_prompt_shell directly (no wrapper footer)
        // because it renders its own footer (mini hint strip or rich interactive footer).
        for fn_name in [
            "render_select_prompt",
            "render_env_prompt",
            "render_drop_prompt",
            "render_naming_prompt",
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
    fn chat_prompt_wrapper_omits_footer_to_avoid_duplicate() {
        let body = fn_source("render_chat_prompt");
        assert!(
            body.contains("render_simple_prompt_shell("),
            "render_chat_prompt should use the shared shell directly"
        );
        assert!(
            body.contains(", None)"),
            "render_chat_prompt should pass None for footer (chat renders its own)"
        );
        assert!(
            !body.contains("render_wrapped_prompt_entity("),
            "render_chat_prompt should not use render_wrapped_prompt_entity (which adds a footer)"
        );
    }

    #[test]
    fn wrapper_surfaces_emit_hint_audits() {
        // Surfaces that emit universal hint audits (chat is excluded — it owns its footer)
        for (fn_name, surface) in [
            ("render_select_prompt", "render_prompts::select"),
            ("render_env_prompt", "render_prompts::env"),
            ("render_drop_prompt", "render_prompts::drop"),
            ("render_naming_prompt", "render_prompts::naming"),
            ("render_webcam_prompt", "render_prompts::webcam"),
            (
                "render_creation_feedback",
                "render_prompts::creation_feedback",
            ),
        ] {
            let body = fn_source(fn_name);
            assert!(
                body.contains("emit_prompt_hint_audit("),
                "{fn_name} should emit a prompt hint audit at the wrapper level"
            );
            assert!(
                body.contains(&format!("\"{}\"", surface)),
                "{fn_name} should emit hint audit with surface name \"{surface}\""
            );
            assert!(
                body.contains("universal_prompt_hints()"),
                "{fn_name} should use universal_prompt_hints() for the hint audit"
            );
        }
    }

    #[test]
    fn template_prompt_emits_surface_specific_truthful_hints() {
        let body = fn_source("render_template_prompt");
        assert!(
            body.contains("emit_surface_prompt_hint_audit("),
            "template prompt should emit a surface-specific hint audit"
        );
        assert!(
            body.contains("template_prompt_hints()"),
            "template prompt should use its truthful submit/next/actions hints"
        );
        assert!(
            !body.contains("universal_prompt_hints()"),
            "template prompt should not advertise the universal AI hint"
        );
    }

    #[test]
    fn render_wrapped_prompt_entity_calls_shared_shell_helper() {
        let body = fn_source("render_wrapped_prompt_entity");
        assert!(
            body.contains("crate::components::render_simple_prompt_shell("),
            "render_wrapped_prompt_entity must call the shared component helper explicitly"
        );
        assert!(
            body.contains("clickable_universal_hint_strip("),
            "render_wrapped_prompt_entity must supply the clickable three-key hint strip footer"
        );
    }

    #[test]
    fn render_template_prompt_uses_shared_wrapper() {
        let body = fn_source("render_template_prompt");
        assert!(
            !body.contains("PromptFooter::new("),
            "render_template_prompt should not use PromptFooter"
        );
        assert!(
            body.contains("render_wrapped_prompt_entity_with_footer("),
            "render_template_prompt should delegate to the footer-aware shared wrapper"
        );
    }

    #[test]
    fn render_naming_prompt_uses_shared_wrapper() {
        let body = fn_source("render_naming_prompt");
        assert!(
            !body.contains("PromptFooter::new("),
            "render_naming_prompt should not use PromptFooter"
        );
        assert!(
            body.contains("render_wrapped_prompt_entity("),
            "render_naming_prompt should delegate to render_wrapped_prompt_entity"
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
            body.contains("clickable_universal_hint_strip("),
            "render_creation_feedback should use the clickable hint strip"
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
            body.contains("clickable_universal_hint_strip("),
            "render_webcam_prompt should use the clickable hint strip"
        );
    }
}

#[cfg(test)]
mod other_prompt_source_tests {
    const SOURCE: &str = include_str!("other.rs");

    fn fn_source(fn_name: &str) -> String {
        let needle = format!("fn {fn_name}(");
        let start = SOURCE.find(&needle).expect("function should exist");
        let rest = &SOURCE[start..];
        let end = rest[1..]
            .find("\n    fn ")
            .map(|ix| ix + 1)
            .unwrap_or(rest.len());
        rest[..end].to_string()
    }

    #[test]
    fn template_prompt_uses_matching_surface_names_for_chrome_and_hint_audits() {
        let body = fn_source("render_template_prompt");
        assert!(
            body.contains("\"render_prompts::template\""),
            "template prompt should use the namespaced surface id in audit calls"
        );
        assert!(
            !body.contains("\"template_prompt\""),
            "template prompt should not use a second surface id spelling"
        );
    }

    #[test]
    fn naming_prompt_uses_matching_surface_names_for_chrome_and_hint_audits() {
        let body = fn_source("render_naming_prompt");
        assert!(
            body.contains("\"render_prompts::naming\""),
            "naming prompt should use the namespaced surface id in audit calls"
        );
        assert!(
            !body.contains("\"naming_prompt\""),
            "naming prompt should not use a second surface id spelling"
        );
    }

    #[test]
    fn webcam_prompt_uses_namespaced_surface_id() {
        let body = fn_source("render_webcam_prompt");
        assert!(
            body.contains("\"render_prompts::webcam\""),
            "webcam prompt should use the namespaced surface id"
        );
        assert!(
            !body.contains("\"webcam_prompt\""),
            "webcam prompt should not use a second surface id spelling"
        );
    }

    #[test]
    fn chat_prompt_wrapper_reports_hint_strip_footer() {
        let body = fn_source("render_chat_prompt");
        assert!(
            body.contains("footer_mode: \"hint_strip\""),
            "chat wrapper should report hint_strip footer mode in chrome audit"
        );
        assert!(
            body.contains("hint_count: 3"),
            "chat wrapper should report 3 hint keys"
        );
        assert!(
            body.contains("has_leading_status: true"),
            "chat wrapper should report leading status text"
        );
        assert!(
            body.contains("exception_reason: Some(\"chat_prompt_renders_hint_strip_internally\")"),
            "chat wrapper should document why it renders the hint strip internally"
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
            source.contains("clickable_universal_hint_strip(")
                || source.contains("render_simple_hint_strip(")
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
                    line.contains("render_prompts::webcam")
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
