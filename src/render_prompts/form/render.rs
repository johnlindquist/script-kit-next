impl ScriptListApp {
    fn render_form_prompt(
        &mut self,
        entity: Entity<FormPromptState>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "render_prompts::form",
                self.has_nonempty_sdk_actions(),
            ),
        );
        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let theme = render_context.theme;
        let design_colors = render_context.design_colors;
        let design_typography = render_context.design_typography;
        let has_actions = self.has_nonempty_sdk_actions();

        // Get prompt ID and field count from entity for tracing
        let (prompt_id, prompt_type, field_count) = {
            let form_state = entity.read(cx);
            (
                form_state.id.clone(),
                form_state.prompt_type(),
                form_state.fields.len(),
            )
        };

        tracing::info!(
            surface = "render_prompts::form",
            prompt_id = %prompt_id,
            prompt_type,
            field_count,
            shell = "render_simple_prompt_shell",
            body_insets = "main_menu",
            "prompt_surface_rendered"
        );

        let form_for_key = entity.clone();

        let prompt_id_for_key = prompt_id.clone();
        // Key handler for form navigation (Enter/Tab/Escape) and Cmd+K actions
        let has_actions_for_handler = has_actions;
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                if handle_prompt_key_preamble_default(
                    this,
                    event,
                    window,
                    cx,
                    PromptKeyPreambleCfg {
                        stop_propagation_on_global_shortcut: false,
                        stop_propagation_when_handled: false,
                        host: ActionsDialogHost::FormPrompt,
                    },
                    has_actions_for_handler,
                    "FormPrompt",
                ) {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let has_shift = event.keystroke.modifiers.shift;
                let has_cmd = event.keystroke.modifiers.platform;

                if form_enter_behavior(
                    key,
                    has_cmd,
                    focused_form_field_is_textarea(form_for_key.read(cx)),
                ) == FormEnterBehavior::Submit
                {
                    cx.stop_propagation();
                    let values =
                        form_for_key.update(cx, |form, cx| form.validated_submit_value(Some(window), cx));
                    if let Some(values) = values {
                        this.submit_prompt_response(prompt_id_for_key.clone(), Some(values), cx);
                    }
                    return;
                }

                // Note: "escape" is handled by handle_global_shortcut_with_options above
                if key.eq_ignore_ascii_case("tab") {
                    cx.stop_propagation();
                    form_for_key.update(cx, |form, cx| {
                        if has_shift {
                            form.focus_previous(window, cx);
                        } else {
                            form.focus_next(window, cx);
                        }
                    });
                    return;
                }

                // A focused field already handled the bubbling edit, including
                // plain Return in a textarea. Forward only when the shell owns focus.
                if !form_for_key
                    .read(cx)
                    .current_focus_handle(cx)
                    .is_some_and(|field| field.is_focused(window))
                {
                    form_for_key.update(cx, |form, cx| form.handle_key_input(event, cx));
                }
            },
        );

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(theme);

        // Form fields have their own focus handles and on_key_down handlers.
        // We DO NOT track_focus on the container - the fields handle their own focus.
        // Enter/Escape/Tab are handled by the handle_key listener above.
        crate::components::render_simple_prompt_shell(
            0.0,
            vibrancy_bg,
            crate::components::render_inset_prompt_body(
                "form-prompt-inset-body",
                entity.clone(),
                crate::components::PromptBodyInsets::MainMenu(self.current_design),
            ),
            self.main_window_footer_slot(self.clickable_universal_footer_action_rail(cx)),
        )
        .text_color(rgb(design_colors.text_primary))
        .font_family(design_typography.font_family)
        .key_context("form_prompt")
        .on_key_down(handle_key)
        .into_any_element()
    }
}

#[cfg(test)]
mod form_prompt_render_backdrop_tests {
    const FORM_RENDER_SOURCE: &str = include_str!("render.rs");

    #[test]
    fn test_form_key_handler_uses_shared_preamble_helper() {
        assert!(
            FORM_RENDER_SOURCE.contains("handle_prompt_key_preamble("),
            "form key handling should delegate preamble logic to shared helper"
        );
        assert!(
            FORM_RENDER_SOURCE.contains("PromptKeyPreambleCfg"),
            "form key handling should configure the shared helper via PromptKeyPreambleCfg"
        );
    }
}
