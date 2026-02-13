    fn render_arg_prompt(
        &mut self,
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        actions: Option<Vec<ProtocolAction>>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let _theme = &self.theme;
        let _filtered = self.filtered_arg_choices();
        let has_actions = actions.is_some() && !actions.as_ref().unwrap().is_empty();
        let has_choices = !choices.is_empty();

        // Use design tokens for GLOBAL theming - all prompts use current design
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();
        let (actions_dialog_top, actions_dialog_right) =
            prompt_actions_dialog_offsets(design_spacing.padding_sm, design_visual.border_thin);

        // Key handler for arg prompt
        let prompt_id = id.clone();
        let has_actions_for_handler = has_actions;
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                // Note: Escape when actions popup is open should close the popup, not dismiss prompt
                if !this.show_actions_popup
                    && this.handle_global_shortcut_with_options(event, true, cx)
                {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;
                let modifiers = &event.keystroke.modifiers;

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && ui_foundation::is_key_k(key) && has_actions_for_handler {
                    logging::log("KEY", "Cmd+K in ArgPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
                    return;
                }

                // Route to shared actions dialog handler (modal when open)
                match this.route_key_to_actions_dialog(
                    key,
                    key_char,
                    modifiers,
                    ActionsDialogHost::ArgPrompt,
                    window,
                    cx,
                ) {
                    ActionsRoute::Execute { action_id } => {
                        this.trigger_action_by_name(&action_id, cx);
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

                // Check for SDK action shortcuts (only when actions popup is NOT open)
                let key_lower = key.to_lowercase();
                let shortcut_key =
                    shortcuts::keystroke_to_shortcut(&key_lower, &event.keystroke.modifiers);
                if let Some(action_name) = this.action_shortcuts.get(&shortcut_key).cloned() {
                    logging::log(
                        "KEY",
                        &format!("SDK action shortcut matched: {}", action_name),
                    );
                    this.trigger_action_by_name(&action_name, cx);
                    return;
                }

                // Arrow up/down: list navigation (use allocation-free helpers)
                if ui_foundation::is_key_up(key) && !modifiers.shift {
                    if this.arg_selected_index > 0 {
                        this.arg_selected_index -= 1;
                        // P0: Scroll to keep selection visible
                        this.arg_list_scroll_handle
                            .scroll_to_item(this.arg_selected_index, ScrollStrategy::Nearest);
                        logging::log_debug(
                            "SCROLL",
                            &format!("P0: Arg up: selected_index={}", this.arg_selected_index),
                        );
                        cx.notify();
                    }
                    return;
                }

                if ui_foundation::is_key_down(key) && !modifiers.shift {
                    let filtered = this.filtered_arg_choices();
                    if this.arg_selected_index < filtered.len().saturating_sub(1) {
                        this.arg_selected_index += 1;
                        // P0: Scroll to keep selection visible
                        this.arg_list_scroll_handle
                            .scroll_to_item(this.arg_selected_index, ScrollStrategy::Nearest);
                        logging::log_debug(
                            "SCROLL",
                            &format!("P0: Arg down: selected_index={}", this.arg_selected_index),
                        );
                        cx.notify();
                    }
                    return;
                }

                if key.eq_ignore_ascii_case("tab") && !has_cmd && !modifiers.alt && !modifiers.shift
                {
                    this.apply_arg_tab_completion(window, cx);
                    return;
                }

                if ui_foundation::is_key_enter(key) {
                    this.submit_arg_prompt_from_current_state(&prompt_id, cx);
                    return;
                }

                // Delegate all other keys to TextInputState for editing, selection, clipboard
                let old_text = this.arg_input.text().to_string();

                // PRESERVE SELECTION: Capture the original index of the currently selected item
                // BEFORE handle_key changes the text (which changes the filtered results)
                let prev_original_idx = this
                    .filtered_arg_choices()
                    .get(this.arg_selected_index)
                    .map(|(orig_idx, _)| *orig_idx);

                let handled = this.arg_input.handle_key(
                    &key_lower,
                    key_char,
                    modifiers.platform, // Cmd key on macOS
                    modifiers.alt,
                    modifiers.shift,
                    cx,
                );

                if handled {
                    // If text changed (not just cursor move), update selection and resize
                    if this.arg_input.text() != old_text {
                        this.sync_arg_prompt_after_text_change(prev_original_idx, window, cx);
                    }
                    cx.notify();
                }
            },
        );

        let input_is_empty = self.arg_input.is_empty();

        // P4: Pre-compute theme values for arg prompt - use theme for consistent styling
        let arg_list_colors = ListItemColors::from_theme(&self.theme);
        let text_primary = design_colors.text_primary;
        let text_muted = design_colors.text_muted;
        let accent_color = design_colors.accent;

        // P0: Clone data needed for uniform_list closure
        let arg_selected_index = self.arg_selected_index;
        let filtered_choices = self.get_filtered_arg_choices_owned();
        let filtered_choices_len = filtered_choices.len();
        // NOTE: Removed per-render log - fires every render frame during cursor blink

        // P0: Build virtualized choice list using uniform_list
        let list_element: AnyElement = if filtered_choices_len == 0 {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(design_colors.text_muted))
                .font_family(design_typography.font_family)
                .child("No choices match your filter · press Enter to use typed value")
                .into_any_element()
        } else {
            // P0: Use uniform_list for virtualized scrolling of arg choices
            // Now uses shared ListItem component for consistent design with script list
            uniform_list(
                "arg-choices",
                filtered_choices_len,
                move |visible_range, _window, _cx| {
                    // NOTE: Removed visible range log - fires per render frame
                    visible_range
                        .map(|ix| {
                            if let Some((_, choice)) = filtered_choices.get(ix) {
                                let is_selected = ix == arg_selected_index;

                                // Use shared ListItem component for consistent design
                                div().id(ix).child(
                                    ListItem::new(choice.name.clone(), arg_list_colors)
                                        .description_opt(choice.description.clone())
                                        .selected(is_selected)
                                        .with_accent_bar(true)
                                        .index(ix),
                                )
                            } else {
                                div().id(ix).h(px(LIST_ITEM_HEIGHT))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.arg_list_scroll_handle)
            .into_any_element()
        };

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);

        // P4: Pre-compute more theme values for the main container using design tokens
        let ui_border = design_colors.border;

        div()
            .relative() // Needed for absolute positioned actions dialog overlay
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("arg_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header with input - uses shared header constants for visual consistency with main menu
            .child(
                div()
                    .w_full()
                    .px(px(HEADER_PADDING_X))
                    .py(px(HEADER_PADDING_Y))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(HEADER_GAP))
                    // Search input with cursor and selection support
                    // Use explicit height matching main menu: CURSOR_HEIGHT_LG + 2*CURSOR_MARGIN_Y = 22px
                    .child({
                        let input_height = CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0);
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .h(px(input_height)) // Fixed height for consistent vertical centering
                            .text_size(px(design_typography.font_size_lg))
                            .text_color(if input_is_empty {
                                rgb(text_muted)
                            } else {
                                rgb(text_primary)
                            })
                            // When empty: show cursor (always reserve space) + placeholder
                            .when(input_is_empty, |d: gpui::Div| {
                                let is_cursor_visible = self.focused_input
                                    == FocusedInput::ArgPrompt
                                    && self.cursor_visible;
                                // Both cursor and placeholder in same flex container, centered together
                                // Use relative positioning for the placeholder to overlay cursor space
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
                                                .child(placeholder.clone()),
                                        ),
                                )
                            })
                            // When has text: show text with cursor/selection via helper
                            .when(!input_is_empty, |d: gpui::Div| {
                                d.child(self.render_arg_input_text(text_primary, accent_color))
                            })
                    }),
            )
            // Choices list (only when prompt has choices)
            .when(has_choices, |d| {
                d.child(
                    div()
                        .mx(px(design_spacing.padding_lg))
                        .h(px(design_visual.border_thin))
                        .bg(rgba((ui_border << 8) | 0x60)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_h(px(0.)) // P0: Allow flex container to shrink
                        .w_full()
                        .py(px(design_spacing.padding_xs))
                        .child(list_element),
                )
            })
            // Footer with unified actions
            .child({
                let footer_colors =
                    prompt_footer_colors_for_prompt(&design_colors, !self.theme.is_dark_mode());
                let helper_status =
                    resolve_arg_helper_status(has_choices, filtered_choices_len, input_is_empty);
                let helper_text = Some(arg_helper_status_text(helper_status));
                let info_label = if has_choices {
                    Some(format!("{filtered_choices_len} options"))
                } else {
                    None
                };
                let footer_config = prompt_footer_config_with_status(
                    "Continue",
                    has_actions,
                    helper_text,
                    info_label,
                );

                // Create click handlers
                let prompt_id_for_primary = id.clone();
                let handle_primary = cx.entity().downgrade();
                let handle_secondary = cx.entity().downgrade();

                PromptFooter::new(footer_config, footer_colors)
                    .on_primary_click(Box::new(move |_, _window, cx| {
                        if let Some(app) = handle_primary.upgrade() {
                            app.update(cx, |this, cx| {
                                this.submit_arg_prompt_from_current_state(
                                    &prompt_id_for_primary,
                                    cx,
                                );
                            });
                        }
                    }))
                    .on_secondary_click(Box::new(move |_, window, cx| {
                        if let Some(app) = handle_secondary.upgrade() {
                            app.update(cx, |this, cx| {
                                this.toggle_arg_actions(cx, window);
                            });
                        }
                    }))
            })
            // Actions dialog overlay (when Cmd+K is pressed with SDK actions)
            // Uses same pattern as main menu: check BOTH show_actions_popup AND actions_dialog
            .when_some(
                if self.show_actions_popup {
                    self.actions_dialog.clone()
                } else {
                    None
                },
                |d, dialog| {
                    // Create click handler for backdrop to dismiss dialog
                    let backdrop_click = cx.listener(
                        |this: &mut Self,
                         _event: &gpui::ClickEvent,
                         window: &mut Window,
                         cx: &mut Context<Self>| {
                            logging::log(
                                "FOCUS",
                                "Arg actions backdrop clicked - dismissing dialog",
                            );
                            this.close_actions_popup(ActionsDialogHost::ArgPrompt, window, cx);
                        },
                    );

                    d.child(
                        div()
                            .absolute()
                            .inset_0() // Cover entire arg prompt area
                            // Backdrop layer - captures clicks outside the dialog
                            .child(
                                div()
                                    .id("arg-actions-backdrop")
                                    .absolute()
                                    .inset_0()
                                    .on_click(backdrop_click),
                            )
                            // Dialog positioned at top-right
                            .child(
                                div()
                                    .absolute()
                                    .top(px(actions_dialog_top))
                                    .right(px(actions_dialog_right))
                                    .child(dialog),
                            ),
                    )
                },
            )
            .into_any_element()
    }
