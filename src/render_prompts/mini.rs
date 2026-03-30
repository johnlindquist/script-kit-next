mod __render_prompts_mini_docs {
    //! Mini prompt rendering: compact single-line input with reduced chrome.
    //! No list display, minimal padding, optional footer.
    //! This fragment is included via include!() macro in main.rs.
}

impl ScriptListApp {
    fn render_mini_prompt(
        &mut self,
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        tracing::info!(
            prompt_type = "mini",
            prompt_id = %id,
            choice_count = choices.len(),
            "render_mini_prompt: rendering compact single-line prompt"
        );

        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let theme = render_context.theme;
        let design_typography = render_context.design_typography;

        let text_primary = theme.colors.text.primary;
        let text_muted = theme.colors.text.muted;
        let accent_color = theme.colors.accent.selected;
        let input_is_empty = self.arg_input.is_empty();

        // Mini uses shared layout tokens from the resize contract
        let mini_padding_x: f32 = crate::window_resize::mini_layout::HEADER_PADDING_X;
        let mini_padding_y: f32 = crate::window_resize::mini_layout::HEADER_PADDING_Y;

        // Key handler - Enter submits, Escape dismisses, arrow keys navigate hidden selection
        let prompt_id = id.clone();
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing
                this.hide_mouse_cursor(cx);

                let key = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;
                let modifiers = &event.keystroke.modifiers;
                let key_lower = key.to_lowercase();

                // Escape dismisses
                if ui_foundation::is_key_escape(key) {
                    this.go_back_or_close(window, cx);
                    cx.stop_propagation();
                    return;
                }

                // Enter submits
                if ui_foundation::is_key_enter(key) {
                    this.submit_arg_prompt_from_current_state(&prompt_id, cx);
                    cx.stop_propagation();
                    return;
                }

                // Arrow up/down for hidden selection navigation
                if ui_foundation::is_key_up(key) && !modifiers.shift {
                    if this.arg_selected_index > 0 {
                        this.arg_selected_index -= 1;
                        cx.notify();
                    }
                    cx.stop_propagation();
                    return;
                }

                if ui_foundation::is_key_down(key) && !modifiers.shift {
                    let filtered = this.filtered_arg_choices();
                    if this.arg_selected_index < filtered.len().saturating_sub(1) {
                        this.arg_selected_index += 1;
                        cx.notify();
                    }
                    cx.stop_propagation();
                    return;
                }

                // Tab completion
                if key.eq_ignore_ascii_case("tab") && !has_cmd && !modifiers.alt && !modifiers.shift
                {
                    this.apply_arg_tab_completion(window, cx);
                    cx.stop_propagation();
                    return;
                }

                // Cmd+W closes
                if has_cmd && key.eq_ignore_ascii_case("w") {
                    this.go_back_or_close(window, cx);
                    cx.stop_propagation();
                    return;
                }

                // Delegate text editing to TextInputState
                let old_text = this.arg_input.text().to_string();
                let prev_original_idx = this
                    .filtered_arg_choices()
                    .get(this.arg_selected_index)
                    .map(|(orig_idx, _)| *orig_idx);

                let handled = this.arg_input.handle_key(
                    &key_lower,
                    key_char,
                    modifiers.platform,
                    modifiers.alt,
                    modifiers.shift,
                    cx,
                );

                if handled {
                    if this.arg_input.text() != old_text {
                        // Update selection tracking but don't resize (mini is fixed height)
                        let new_selected_idx = {
                            let filtered = this.filtered_arg_choices();
                            if let Some(prev_idx) = prev_original_idx {
                                filtered
                                    .iter()
                                    .position(|(orig_idx, _)| *orig_idx == prev_idx)
                                    .unwrap_or(0)
                            } else {
                                0
                            }
                        };
                        this.arg_selected_index = new_selected_idx;
                    }
                    cx.notify();
                } else {
                    cx.propagate();
                }
            },
        );

        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "render_prompts::mini",
                false,
            ),
        );

        // Build header (compact input row with reduced padding)
        let header = div()
            .w_full()
            .px(px(mini_padding_x))
            .py(px(mini_padding_y))
            .flex()
            .flex_row()
            .items_center()
            .child({
                let input_height = CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0);
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .items_center()
                    .h(px(input_height))
                    .text_size(px(design_typography.font_size_lg))
                    .text_color(if input_is_empty {
                        rgb(text_muted)
                    } else {
                        rgb(text_primary)
                    })
                    .when(input_is_empty, |d: gpui::Div| {
                        let is_cursor_visible = self.focused_input
                            == FocusedInput::ArgPrompt
                            && self.cursor_visible;
                        d.child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .when(is_cursor_visible, |d: gpui::Div| {
                                            d.bg(rgb(text_primary))
                                        }),
                                )
                                .child(
                                    div()
                                        .ml(px(-(CURSOR_WIDTH)))
                                        .text_color(rgb(text_muted))
                                        .child(placeholder),
                                ),
                        )
                    })
                    .when(!input_is_empty, |d: gpui::Div| {
                        d.child(self.render_arg_input_text(text_primary, accent_color))
                    })
            });

        // Compact choice list for mini prompt (e.g. mic picker)
        let content = if choices.is_empty() {
            div()
        } else {
            let arg_list_colors = crate::list_item::ListItemColors::from_theme(theme);
            let arg_selected_index = self.arg_selected_index;
            let filtered_choices = self.get_filtered_arg_choices_owned();
            let filtered_choices_len = filtered_choices.len();

            if filtered_choices_len == 0 {
                div()
                    .w_full()
                    .h(px(crate::list_item::LIST_ITEM_HEIGHT))
                    .px(px(mini_padding_x))
                    .flex()
                    .items_center()
                    .text_color(rgb(text_muted))
                    .child("No matches")
            } else {
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.))
                    .w_full()
                    .child(
                        uniform_list(
                            "mini-choices",
                            filtered_choices_len,
                            move |visible_range, _window, _cx| {
                                visible_range
                                    .map(|ix| {
                                        if let Some((_, choice)) = filtered_choices.get(ix) {
                                            let is_selected = ix == arg_selected_index;
                                            div().id(ix).child(
                                                crate::list_item::ListItem::new(
                                                    choice.name.clone(),
                                                    arg_list_colors,
                                                )
                                                .selected(is_selected)
                                                .with_accent_bar(true)
                                                .index(ix),
                                            )
                                        } else {
                                            div()
                                                .id(ix)
                                                .h(px(crate::list_item::LIST_ITEM_HEIGHT))
                                        }
                                    })
                                    .collect()
                            },
                        )
                        .h_full()
                        .track_scroll(&self.arg_list_scroll_handle),
                    )
            }
        };
        let leading: Option<gpui::AnyElement> = None;

        // Use the same shared shell as arg/render.rs
        crate::components::render_minimal_list_prompt_shell(
            0.0,
            crate::ui_foundation::get_vibrancy_background(&self.theme),
            header,
            content,
            crate::components::universal_prompt_hints(),
            leading,
        )
        .text_color(rgb(text_primary))
        .font_family(design_typography.font_family)
        .key_context("mini_prompt")
        .track_focus(&self.focus_handle)
        .on_key_down(handle_key)
        .into_any_element()
    }
}

#[cfg(test)]
mod mini_prompt_render_tests {
    const MINI_SOURCE: &str = include_str!("mini.rs");

    #[test]
    fn mini_prompt_has_compact_padding() {
        assert!(
            MINI_SOURCE.contains("mini_layout::HEADER_PADDING_X"),
            "mini prompt should use shared mini layout horizontal padding token"
        );
        assert!(
            MINI_SOURCE.contains("mini_layout::HEADER_PADDING_Y"),
            "mini prompt should use shared mini layout vertical padding token"
        );
    }

    #[test]
    fn mini_prompt_renders_compact_choice_list() {
        assert!(
            MINI_SOURCE.contains("uniform_list("),
            "mini prompt should render a compact choice list when choices are present"
        );
        assert!(
            MINI_SOURCE.contains("ListItem::new("),
            "mini prompt should use ListItem for consistent choice rendering"
        );
        assert!(
            MINI_SOURCE.contains("min_h(px(0.))"),
            "mini prompt list container needs min_h to prevent overflow"
        );
    }

    #[test]
    fn mini_prompt_uses_shared_shell_with_universal_hints() {
        assert!(
            MINI_SOURCE.contains("render_minimal_list_prompt_shell("),
            "mini prompt should use the shared prompt shell from arg/render.rs"
        );
        assert!(
            MINI_SOURCE.contains("universal_prompt_hints()"),
            "mini prompt should use the canonical three-key hint strip"
        );
        let render_fn_end = MINI_SOURCE
            .find("#[cfg(test)]")
            .unwrap_or(MINI_SOURCE.len());
        let render_code = &MINI_SOURCE[..render_fn_end];
        let needle = ["PromptFooter", "::new("].concat();
        assert!(
            !render_code.contains(&needle),
            "mini prompt should not use PromptFooter"
        );
    }

    #[test]
    fn mini_prompt_uses_zero_radius_shell() {
        // The shared shell is called with 0.0 radius — no visible rounding
        assert!(
            !MINI_SOURCE.contains("design_visual.radius_lg"),
            "mini prompt should not use radius_lg for shell rounding"
        );
        assert!(
            !MINI_SOURCE.contains("shell_radius"),
            "mini prompt should not use shell_radius for shell rounding"
        );
    }

    #[test]
    fn mini_prompt_emits_chrome_audit() {
        assert!(
            MINI_SOURCE.contains("emit_prompt_chrome_audit("),
            "mini prompt should emit a chrome audit"
        );
        assert!(
            MINI_SOURCE.contains("\"render_prompts::mini\""),
            "mini prompt chrome audit should use correct surface name"
        );
    }

    #[test]
    fn mini_prompt_has_key_handlers() {
        assert!(
            MINI_SOURCE.contains("is_key_escape(key)"),
            "mini prompt should handle Escape"
        );
        assert!(
            MINI_SOURCE.contains("is_key_enter(key)"),
            "mini prompt should handle Enter"
        );
        assert!(
            MINI_SOURCE.contains("stop_propagation()"),
            "mini prompt should stop propagation on handled keys"
        );
    }

    #[test]
    fn mini_prompt_emits_structured_logs() {
        assert!(
            MINI_SOURCE.contains("tracing::info!("),
            "mini prompt should emit structured log at info level"
        );
        assert!(
            MINI_SOURCE.contains("prompt_type = \"mini\""),
            "mini prompt log should include prompt_type field"
        );
    }
}
