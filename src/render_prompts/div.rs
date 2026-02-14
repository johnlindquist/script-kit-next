mod __render_prompts_div_docs {
    //! Div prompt rendering integration for `ScriptListApp::render_div_prompt`.
    //! The key surface is the single render method that wires keyboard handling and action popups.
    //! It depends on prompt-shell/components/theme tokens and is included into the main app module.
}

// Div prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

impl ScriptListApp {
    fn render_div_prompt(
        &mut self,
        id: String,
        entity: Entity<DivPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let theme = render_context.theme;
        let design_colors = render_context.design_colors;
        let design_spacing = render_context.design_spacing;
        let design_typography = render_context.design_typography;
        let design_visual = render_context.design_visual;
        let actions_dialog_top = render_context.actions_dialog_top;
        let actions_dialog_right = render_context.actions_dialog_right;
        let has_actions = self.has_nonempty_sdk_actions();

        // Key handler for Cmd+K actions toggle (at parent level to intercept before DivPrompt)
        let has_actions_for_handler = has_actions;
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                if handle_prompt_key_preamble(
                    this,
                    event,
                    window,
                    cx,
                    PromptKeyPreambleCfg {
                        is_dismissable: true,
                        stop_propagation_on_global_shortcut: true,
                        stop_propagation_when_handled: true,
                        host: ActionsDialogHost::DivPrompt,
                    },
                    |_this, _event, _window, _cx| false,
                    |key, _key_char, modifiers| {
                        modifiers.platform && ui_foundation::is_key_k(key) && has_actions_for_handler
                    },
                    |this, window, cx| {
                        logging::log("KEY", "Cmd+K in DivPrompt - calling toggle_arg_actions");
                        this.toggle_arg_actions(cx, window);
                    },
                    |this, action_id, cx| {
                        this.trigger_action_by_name(action_id, cx);
                    },
                    |_key, _key_char, _modifiers| true,
                    |this, matched_shortcut, cx| {
                        logging::log(
                            "KEY",
                            &format!(
                                "SDK action shortcut matched in DivPrompt: {}",
                                matched_shortcut.action_name
                            ),
                        );
                        this.trigger_action_by_name(&matched_shortcut.action_name, cx);
                    },
                ) {}
                // Fall through to DivPrompt entity key handling.
            },
        );

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(theme);

        // Use explicit height from layout constants
        let content_height = window_resize::layout::STANDARD_HEIGHT;

        // Footer colors and config aligned with other interactive prompts.
        let footer_colors = PromptFooterColors::from_theme(theme);
        let footer_config = prompt_footer_config_with_status(
            "Continue",
            has_actions,
            Some(running_status_text("review output and press Enter")),
            Some("Output".to_string()),
        );

        // Create click handlers for footer
        let handle_submit = cx.entity().downgrade();
        let handle_actions = cx.entity().downgrade();
        let prompt_id = id.clone();

        crate::components::prompt_shell_container(design_visual.radius_lg, vibrancy_bg)
            .h(content_height)
            .track_focus(&self.focus_handle) // Required to receive key events
            .on_key_down(handle_key)
            // Header + content shell
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.)) // Critical: allows flex children to size properly
                    .overflow_hidden()
                    .child(
                        div()
                            .w_full()
                            .px(px(HEADER_PADDING_X))
                            .py(px(HEADER_PADDING_Y))
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .gap(px(HEADER_GAP))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(design_colors.text_primary))
                                    .font_family(design_typography.font_family)
                                    .child("Script Output"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(design_colors.text_muted))
                                    .font_family(design_typography.font_family)
                                    .child("Enter to continue"),
                            ),
                    )
                    .child(
                        div()
                            .mx(px(design_spacing.padding_lg))
                            .h(px(design_visual.border_thin))
                            .bg(rgba((design_colors.border << 8) | 0x60)),
                    )
                    .child(crate::components::prompt_shell_content(entity.clone())),
            )
            // Footer with Submit button and Actions
            .child(
                PromptFooter::new(footer_config, footer_colors)
                    .on_primary_click(Box::new(move |_, _window, cx| {
                        if let Some(app) = handle_submit.upgrade() {
                            let id = prompt_id.clone();
                            app.update(cx, |this, cx| {
                                // Submit the div prompt - send empty value to continue
                                this.submit_prompt_response(id, None, cx);
                            });
                        }
                    }))
                    .on_secondary_click(Box::new(move |_, window, cx| {
                        if let Some(app) = handle_actions.upgrade() {
                            app.update(cx, |this, cx| {
                                this.toggle_arg_actions(cx, window);
                            });
                        }
                    })),
            )
            // Actions dialog overlay (when Cmd+K is pressed with SDK actions)
            .when_some(
                render_actions_backdrop(
                    self.show_actions_popup,
                    self.actions_dialog.clone(),
                    actions_dialog_top,
                    actions_dialog_right,
                    ActionsBackdropConfig {
                        backdrop_id: "div-actions-backdrop",
                        close_host: ActionsDialogHost::DivPrompt,
                        backdrop_log_message: "Div actions backdrop clicked - dismissing dialog",
                        show_pointer_cursor: true,
                    },
                    cx,
                ),
                |d, backdrop_overlay| d.child(backdrop_overlay),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod div_prompt_render_backdrop_tests {
    const DIV_RENDER_SOURCE: &str = include_str!("div.rs");

    #[test]
    fn test_div_actions_backdrop_uses_shared_helper_with_clickable_cursor() {
        assert!(
            DIV_RENDER_SOURCE.contains("render_actions_backdrop("),
            "div render should delegate backdrop overlay creation to shared helper"
        );
        assert!(
            DIV_RENDER_SOURCE.contains("\"div-actions-backdrop\""),
            "div render should pass its backdrop id to shared helper"
        );
        assert!(
            DIV_RENDER_SOURCE.contains("ActionsDialogHost::DivPrompt"),
            "div render should preserve actions host routing when helper is used"
        );
        assert!(
            DIV_RENDER_SOURCE.contains("show_pointer_cursor: true"),
            "div render should keep backdrop cursor pointer enabled"
        );
    }

    #[test]
    fn test_div_key_handling_uses_preamble_helper_with_handled_propagation_stop() {
        assert!(
            DIV_RENDER_SOURCE.contains("handle_prompt_key_preamble("),
            "div key handling should delegate shared preamble behavior to helper"
        );
        assert!(
            DIV_RENDER_SOURCE.contains("stop_propagation_on_global_shortcut: true"),
            "div key preamble should stop propagation when global shortcut consumes the key"
        );
        assert!(
            DIV_RENDER_SOURCE.contains("stop_propagation_when_handled: true"),
            "div key preamble should stop propagation when helper branches handle the key"
        );
    }
}
