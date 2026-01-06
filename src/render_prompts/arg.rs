// Arg prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

impl ScriptListApp {
    /// Render the arg input text with cursor and selection highlight
    fn render_arg_input_text(&self, text_primary: u32, accent_color: u32) -> gpui::Div {
        let text = self.arg_input.text();
        let chars: Vec<char> = text.chars().collect();
        let cursor_pos = self.arg_input.cursor();
        let has_selection = self.arg_input.has_selection();
        // Separate focus state from blink state to avoid layout shift
        let is_focused = self.focused_input == FocusedInput::ArgPrompt;
        let is_cursor_visible = is_focused && self.cursor_visible;

        if text.is_empty() {
            // Empty - always reserve cursor space, only show bg when visible
            // Note: height matches the fixed input_height (22px = CURSOR_HEIGHT_LG + 2*CURSOR_MARGIN_Y)
            return div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0)))
                .child(
                    div()
                        .w(px(CURSOR_WIDTH))
                        .h(px(CURSOR_HEIGHT_LG))
                        .when(is_cursor_visible, |d: gpui::Div| d.bg(rgb(text_primary))),
                );
        }

        if has_selection {
            // With selection: before | selected | after (no cursor shown during selection)
            // Use fixed height matching the input container for consistent centering
            let selection = self.arg_input.selection();
            let (start, end) = selection.range();

            let before: String = chars[..start].iter().collect();
            let selected: String = chars[start..end].iter().collect();
            let after: String = chars[end..].iter().collect();

            div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0)))
                .overflow_x_hidden()
                .when(!before.is_empty(), |d: gpui::Div| {
                    d.child(div().child(before))
                })
                .child(
                    div()
                        .bg(rgba((accent_color << 8) | 0x60))
                        .text_color(rgb(0xffffff))
                        .child(selected),
                )
                .when(!after.is_empty(), |d: gpui::Div| {
                    d.child(div().child(after))
                })
        } else {
            // No selection: before cursor | cursor | after cursor
            // Always reserve cursor space to prevent layout shift during blink
            // Use fixed height matching the input container for consistent centering
            let before: String = chars[..cursor_pos].iter().collect();
            let after: String = chars[cursor_pos..].iter().collect();

            div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0)))
                .overflow_x_hidden()
                .when(!before.is_empty(), |d: gpui::Div| {
                    d.child(div().child(before))
                })
                // Always render cursor element, only show bg when visible
                .child(
                    div()
                        .w(px(CURSOR_WIDTH))
                        .h(px(CURSOR_HEIGHT_LG))
                        .when(is_cursor_visible, |d: gpui::Div| d.bg(rgb(text_primary))),
                )
                .when(!after.is_empty(), |d: gpui::Div| {
                    d.child(div().child(after))
                })
        }
    }

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

        // Key handler for arg prompt
        let prompt_id = id.clone();
        let has_actions_for_handler = has_actions;
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                if this.handle_global_shortcut_with_options(event, true, cx) {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;
                logging::log(
                    "KEY",
                    &format!("ArgPrompt key: '{}' cmd={}", key_str, has_cmd),
                );

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && key_str == "k" && has_actions_for_handler {
                    logging::log("KEY", "Cmd+K in ArgPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
                    return;
                }

                // If actions popup is open, route keyboard events to it (same as main menu)
                if this.show_actions_popup {
                    if let Some(ref dialog) = this.actions_dialog {
                        match key_str.as_str() {
                            "up" | "arrowup" => {
                                dialog.update(cx, |d, cx| d.move_up(cx));
                                return;
                            }
                            "down" | "arrowdown" => {
                                dialog.update(cx, |d, cx| d.move_down(cx));
                                return;
                            }
                            "enter" => {
                                // Get the selected action and execute it
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();
                                if let Some(action_id) = action_id {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "ArgPrompt executing action: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );
                                    // Only close if action has close: true (default)
                                    if should_close {
                                        this.show_actions_popup = false;
                                        this.actions_dialog = None;
                                        this.focused_input = FocusedInput::ArgPrompt;
                                        window.focus(&this.focus_handle, cx);
                                    }
                                    // Trigger the SDK action by name
                                    this.trigger_action_by_name(&action_id, cx);
                                }
                                return;
                            }
                            "escape" => {
                                this.show_actions_popup = false;
                                this.actions_dialog = None;
                                this.focused_input = FocusedInput::ArgPrompt;
                                window.focus(&this.focus_handle, cx);
                                cx.notify();
                                return;
                            }
                            "backspace" => {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                                return;
                            }
                            _ => {
                                // Route character input to the dialog for search
                                if let Some(ref key_char) = event.keystroke.key_char {
                                    if let Some(ch) = key_char.chars().next() {
                                        if !ch.is_control() {
                                            dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                        }
                                    }
                                }
                                return;
                            }
                        }
                    }
                }

                // Check for SDK action shortcuts (only when actions popup is NOT open)
                let shortcut_key =
                    shortcuts::keystroke_to_shortcut(&key_str, &event.keystroke.modifiers);
                if let Some(action_name) = this.action_shortcuts.get(&shortcut_key).cloned() {
                    logging::log(
                        "KEY",
                        &format!("SDK action shortcut matched: {}", action_name),
                    );
                    this.trigger_action_by_name(&action_name, cx);
                    return;
                }

                let modifiers = &event.keystroke.modifiers;

                // Arrow up/down: list navigation
                match key_str.as_str() {
                    "up" | "arrowup" if !modifiers.shift => {
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
                    "down" | "arrowdown" if !modifiers.shift => {
                        let filtered = this.filtered_arg_choices();
                        if this.arg_selected_index < filtered.len().saturating_sub(1) {
                            this.arg_selected_index += 1;
                            // P0: Scroll to keep selection visible
                            this.arg_list_scroll_handle
                                .scroll_to_item(this.arg_selected_index, ScrollStrategy::Nearest);
                            logging::log_debug(
                                "SCROLL",
                                &format!(
                                    "P0: Arg down: selected_index={}",
                                    this.arg_selected_index
                                ),
                            );
                            cx.notify();
                        }
                        return;
                    }
                    "enter" => {
                        let filtered = this.filtered_arg_choices();
                        if let Some((_, choice)) = filtered.get(this.arg_selected_index) {
                            // Case 1: There are filtered choices - submit the selected one
                            let value = choice.value.clone();
                            this.submit_prompt_response(prompt_id.clone(), Some(value), cx);
                        } else if !this.arg_input.is_empty() {
                            // Case 2: No choices but user typed something - submit input text
                            let value = this.arg_input.text().to_string();
                            this.submit_prompt_response(prompt_id.clone(), Some(value), cx);
                        }
                        // Case 3: No choices and no input - do nothing (prevent empty submissions)
                        return;
                    }
                    // Note: "escape" is handled by handle_global_shortcut_with_options above
                    _ => {}
                }

                // Delegate all other keys to TextInputState for editing, selection, clipboard
                let key_char = event.keystroke.key_char.as_deref();
                let old_text = this.arg_input.text().to_string();

                let handled = this.arg_input.handle_key(
                    &key_str,
                    key_char,
                    modifiers.platform, // Cmd key on macOS
                    modifiers.alt,
                    modifiers.shift,
                    cx,
                );

                if handled {
                    // If text changed (not just cursor move), reset selection and update
                    if this.arg_input.text() != old_text {
                        this.arg_selected_index = 0;
                        // DEFERRED RESIZE: Avoid RefCell borrow error by deferring window resize
                        // to next frame. The native macOS setFrame:display:animate: call triggers
                        // callbacks that try to borrow the RefCell while GPUI still holds it.
                        let filtered = this.filtered_arg_choices();
                        let (view_type, item_count) = if filtered.is_empty() {
                            // Check if there are any choices at all
                            if let AppView::ArgPrompt { choices, .. } = &this.current_view {
                                if choices.is_empty() {
                                    (ViewType::ArgPromptNoChoices, 0)
                                } else {
                                    (ViewType::ArgPromptWithChoices, 0)
                                }
                            } else {
                                (ViewType::ArgPromptNoChoices, 0)
                            }
                        } else {
                            (ViewType::ArgPromptWithChoices, filtered.len())
                        };
                        // Use window_ops for coalesced resize (avoids Timer::after pattern)
                        let target_height =
                            crate::window_resize::height_for_view(view_type, item_count);
                        crate::window_ops::queue_resize(f32::from(target_height), window, &mut *cx);
                    }
                    cx.notify();
                }
            },
        );

        let input_is_empty = self.arg_input.is_empty();

        // P4: Pre-compute theme values for arg prompt using design tokens for GLOBAL theming
        let arg_list_colors = ListItemColors::from_design(&design_colors);
        let text_primary = design_colors.text_primary;
        let text_muted = design_colors.text_muted;
        let accent_color = design_colors.accent;

        // P0: Clone data needed for uniform_list closure
        let arg_selected_index = self.arg_selected_index;
        let filtered_choices = self.get_filtered_arg_choices_owned();
        let filtered_choices_len = filtered_choices.len();
        logging::log_debug(
            "UI",
            &format!(
                "P0: Arg prompt has {} filtered choices",
                filtered_choices_len
            ),
        );

        // P0: Build virtualized choice list using uniform_list
        let list_element: AnyElement = if filtered_choices_len == 0 {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(design_colors.text_muted))
                .font_family(design_typography.font_family)
                .child("No choices match your filter")
                .into_any_element()
        } else {
            // P0: Use uniform_list for virtualized scrolling of arg choices
            // Now uses shared ListItem component for consistent design with script list
            uniform_list(
                "arg-choices",
                filtered_choices_len,
                move |visible_range, _window, _cx| {
                    logging::log_debug(
                        "SCROLL",
                        &format!("P0: Arg choices visible range: {:?}", visible_range.clone()),
                    );
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
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // P4: Pre-compute more theme values for the main container using design tokens
        let ui_border = design_colors.border;
        let text_dimmed = design_colors.text_dimmed;

        div()
            .relative() // Needed for absolute positioned actions dialog overlay
            .flex()
            .flex_col()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
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
                            .text_xl()
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
                    })
                    // CLS-FREE ACTIONS AREA: Matches main menu pattern exactly
                    // Both states are always rendered at the same position, visibility toggled via opacity
                    .when(has_actions, |d| {
                        let button_colors = ButtonColors::from_theme(&self.theme);
                        let handle_actions = cx.entity().downgrade();
                        let show_actions = self.show_actions_popup;

                        // Get actions search text from the dialog
                        let search_text = self
                            .actions_dialog
                            .as_ref()
                            .map(|dialog| dialog.read(cx).search_text.clone())
                            .unwrap_or_default();
                        let search_is_empty = search_text.is_empty();
                        let search_display: SharedString = if search_is_empty {
                            "Search actions...".into()
                        } else {
                            search_text.into()
                        };
                        let accent_color = design_colors.accent;
                        let search_box_bg = self.theme.colors.background.search_box;
                        let cursor_visible_for_search = self.focused_input
                            == FocusedInput::ActionsSearch
                            && self.cursor_visible;

                        d.child(
                            div()
                                .relative()
                                .h(px(28.)) // Fixed height to prevent vertical CLS
                                .flex()
                                .items_center()
                                // Layer 1: Actions button - visible when NOT showing actions search
                                .child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_end()
                                        .when(show_actions, |d| d.opacity(0.).invisible())
                                        .child(
                                            Button::new("Actions", button_colors)
                                                .variant(ButtonVariant::Ghost)
                                                .shortcut("⌘ K")
                                                .on_click(Box::new(move |_, window, cx| {
                                                    if let Some(app) = handle_actions.upgrade() {
                                                        app.update(cx, |this, cx| {
                                                            this.toggle_arg_actions(cx, window);
                                                        });
                                                    }
                                                })),
                                        )
                                        .child(
                                            div()
                                                .mx(px(4.))
                                                .text_color(rgba((text_dimmed << 8) | 0x60))
                                                .text_sm()
                                                .child("|"),
                                        ),
                                )
                                // Layer 2: Actions search input - visible when showing actions search
                                .child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_end()
                                        .gap(px(8.))
                                        .when(!show_actions, |d| d.opacity(0.).invisible())
                                        // ⌘K indicator
                                        .child(
                                            div()
                                                .text_color(rgb(text_dimmed))
                                                .text_xs()
                                                .child("⌘K"),
                                        )
                                        // Search input display - compact style matching buttons
                                        .child(
                                            div()
                                                .id("arg-actions-search")
                                                .flex_shrink_0()
                                                .w(px(130.))
                                                .min_w(px(130.))
                                                .max_w(px(130.))
                                                .h(px(24.))
                                                .min_h(px(24.))
                                                .max_h(px(24.))
                                                .overflow_hidden()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .px(px(8.))
                                                .rounded(px(4.))
                                                .bg(rgba(
                                                    (search_box_bg << 8)
                                                        | if search_is_empty { 0x40 } else { 0x80 },
                                                ))
                                                .border_1()
                                                .border_color(rgba(
                                                    (accent_color << 8)
                                                        | if search_is_empty { 0x20 } else { 0x40 },
                                                ))
                                                .text_sm()
                                                .text_color(if search_is_empty {
                                                    rgb(text_muted)
                                                } else {
                                                    rgb(text_primary)
                                                })
                                                // Cursor before placeholder when empty
                                                .when(search_is_empty, |d| {
                                                    d.child(
                                                        div()
                                                            .w(px(2.))
                                                            .h(px(14.))
                                                            .mr(px(2.))
                                                            .rounded(px(1.))
                                                            .when(cursor_visible_for_search, |d| {
                                                                d.bg(rgb(accent_color))
                                                            }),
                                                    )
                                                })
                                                .child(search_display)
                                                // Cursor after text when not empty
                                                .when(!search_is_empty, |d| {
                                                    d.child(
                                                        div()
                                                            .w(px(2.))
                                                            .h(px(14.))
                                                            .ml(px(2.))
                                                            .rounded(px(1.))
                                                            .when(cursor_visible_for_search, |d| {
                                                                d.bg(rgb(accent_color))
                                                            }),
                                                    )
                                                }),
                                        )
                                        .child(
                                            div()
                                                .mx(px(4.))
                                                .text_color(rgba((text_dimmed << 8) | 0x60))
                                                .text_sm()
                                                .child("|"),
                                        ),
                                ),
                        )
                    })
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(format!("{} choices", choices.len())),
                    ),
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
                            this.show_actions_popup = false;
                            this.actions_dialog = None;
                            this.focused_input = FocusedInput::ArgPrompt;
                            window.focus(&this.focus_handle, cx);
                            cx.notify();
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
                                    .top(px(52.)) // Clear the header bar (~44px header + 8px margin)
                                    .right(px(8.))
                                    .child(dialog),
                            ),
                    )
                },
            )
            .into_any_element()
    }
}
