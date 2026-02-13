                            ExternalCommand::SimulateKey { ref key, ref modifiers } => {
                                logging::log("STDIN", &format!("Simulating key: '{}' with modifiers: {:?}", key, modifiers));

                                // Parse modifiers
                                let has_cmd = modifiers.contains(&KeyModifier::Cmd);
                                let has_shift = modifiers.contains(&KeyModifier::Shift);
                                let _has_alt = modifiers.contains(&KeyModifier::Alt);
                                let _has_ctrl = modifiers.contains(&KeyModifier::Ctrl);

                                // Handle key based on current view
                                let key_lower = key.to_lowercase();

                                match &view.current_view {
                                    AppView::ScriptList => {
                                        // Main script list key handling
                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle actions");
                                            view.toggle_actions(ctx, window);
                                        } else if view.fallback_mode && !view.cached_fallbacks.is_empty() {
                                            // Handle keys in fallback mode
                                            match key_lower.as_str() {
                                                "tab" => {
                                                    // Tab with filter text opens inline AI chat (even in fallback mode)
                                                    if !view.filter_text.is_empty() && !view.show_actions_popup {
                                                        let query = view.filter_text.clone();
                                                        view.filter_text.clear();
                                                        view.show_inline_ai_chat(Some(query), ctx);
                                                    }
                                                }
                                                "up" | "arrowup" => {
                                                    if view.fallback_selected_index > 0 {
                                                        view.fallback_selected_index -= 1;
                                                        ctx.notify();
                                                    }
                                                }
                                                "down" | "arrowdown" => {
                                                    if view.fallback_selected_index < view.cached_fallbacks.len().saturating_sub(1) {
                                                        view.fallback_selected_index += 1;
                                                        ctx.notify();
                                                    }
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - execute fallback");
                                                    view.execute_selected_fallback(ctx);
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - clear filter (exit fallback mode)");
                                                    view.clear_filter(window, ctx);
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in fallback mode", key_lower));
                                                }
                                            }
                                        } else {
                                            match key_lower.as_str() {
                                                "tab" => {
                                                    // Tab with filter text opens inline AI chat
                                                    if !view.filter_text.is_empty() && !view.show_actions_popup {
                                                        let query = view.filter_text.clone();
                                                        view.filter_text.clear();
                                                        view.show_inline_ai_chat(Some(query), ctx);
                                                    }
                                                }
                                                "up" | "arrowup" => {
                                                    // Use move_selection_up to properly skip section headers
                                                    view.move_selection_up(ctx);
                                                }
                                                "down" | "arrowdown" => {
                                                    // Use move_selection_down to properly skip section headers
                                                    view.move_selection_down(ctx);
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - execute selected");
                                                    view.execute_selected(ctx);
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - clear filter or hide");
                                                    if !view.filter_text.is_empty() {
                                                        view.clear_filter(window, ctx);
                                                    } else {
                                                        // Save window position for the current display BEFORE hiding
                                                        if let Some((x, y, width, height)) = platform::get_main_window_bounds() {
                                                            let displays = platform::get_macos_displays();
                                                            let bounds = window_state::PersistedWindowBounds::new(x, y, width, height);
                                                            if let Some(display) = window_state::find_display_for_bounds(&bounds, &displays) {
                                                                window_state::save_main_position_for_display(display, bounds);
                                                            }
                                                        }
                                                        script_kit_gpui::set_main_window_visible(false);
                                                        ctx.hide();
                                                    }
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ScriptList", key_lower));
                                                }
                                            }
                                        }
                                    }
                                    AppView::PathPrompt { entity, .. } => {
                                        // Path prompt key handling
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to PathPrompt", key_lower));
                                        let entity_clone = entity.clone();
                                        entity_clone.update(ctx, |path_prompt: &mut PathPrompt, path_cx| {
                                            if has_cmd && key_lower == "k" {
                                                path_prompt.toggle_actions(path_cx);
                                            } else {
                                                match key_lower.as_str() {
                                                    "up" | "arrowup" => path_prompt.move_up(path_cx),
                                                    "down" | "arrowdown" => path_prompt.move_down(path_cx),
                                                    "enter" => path_prompt.handle_enter(path_cx),
                                                    "escape" => path_prompt.submit_cancel(),
                                                    "left" | "arrowleft" => path_prompt.navigate_to_parent(path_cx),
                                                    "right" | "arrowright" => path_prompt.navigate_into_selected(path_cx),
                                                    _ => {
                                                        logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in PathPrompt", key_lower));
                                                    }
                                                }
                                            }
                                        });
                                    }
                                    AppView::ArgPrompt { id, .. } => {
                                        // Arg prompt key handling via SimulateKey
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to ArgPrompt (actions_popup={})", key_lower, view.show_actions_popup));

                                        // Check for Cmd+K to toggle actions popup
                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle arg actions");
                                            view.toggle_arg_actions(ctx, window);
                                        } else if view.show_actions_popup {
                                            // If actions popup is open, route to it
                                            if let Some(ref dialog) = view.actions_dialog {
                                                match key_lower.as_str() {
                                                    "up" | "arrowup" => {
                                                        logging::log("STDIN", "SimulateKey: Up in actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_up(cx));
                                                    }
                                                    "down" | "arrowdown" => {
                                                        logging::log("STDIN", "SimulateKey: Down in actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_down(cx));
                                                    }
                                                    "enter" => {
                                                        logging::log("STDIN", "SimulateKey: Enter in actions dialog");
                                                        let action_id = dialog.read(ctx).get_selected_action_id();
                                                        let should_close = dialog.read(ctx).selected_action_should_close();
                                                        if let Some(action_id) = action_id {
                                                            logging::log("ACTIONS", &format!("SimulateKey: Executing action: {} (close={})", action_id, should_close));
                                                            if should_close {
                                                                view.show_actions_popup = false;
                                                                view.actions_dialog = None;
                                                                view.focused_input = FocusedInput::ArgPrompt;
                                                                window.focus(&view.focus_handle, ctx);
                                                            }
                                                            view.trigger_action_by_name(&action_id, ctx);
                                                        }
                                                    }
                                                    "escape" => {
                                                        logging::log("STDIN", "SimulateKey: Escape - close actions dialog");
                                                        view.show_actions_popup = false;
                                                        view.actions_dialog = None;
                                                        view.focused_input = FocusedInput::ArgPrompt;
                                                        window.focus(&view.focus_handle, ctx);
                                                    }
                                                    _ => {
                                                        logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ArgPrompt actions dialog", key_lower));
                                                    }
                                                }
                                            }
                                        } else {
                                            // Normal arg prompt key handling
                                            let prompt_id = id.clone();
                                            match key_lower.as_str() {
                                                "up" | "arrowup" => {
                                                    if view.arg_selected_index > 0 {
                                                        view.arg_selected_index -= 1;
                                                        view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
                                                        logging::log("STDIN", &format!("SimulateKey: Arg up, index={}", view.arg_selected_index));
                                                    }
                                                }
                                                "down" | "arrowdown" => {
                                                    let filtered = view.filtered_arg_choices();
                                                    if view.arg_selected_index < filtered.len().saturating_sub(1) {
                                                        view.arg_selected_index += 1;
                                                        view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
                                                        logging::log("STDIN", &format!("SimulateKey: Arg down, index={}", view.arg_selected_index));
                                                    }
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - submit selection");
                                                    let filtered = view.filtered_arg_choices();
                                                    if let Some((_, choice)) = filtered.get(view.arg_selected_index) {
                                                        let value = choice.value.clone();
                                                        view.submit_prompt_response(prompt_id, Some(value), ctx);
                                                    } else if !view.arg_input.is_empty() {
                                                        let value = view.arg_input.text().to_string();
                                                        view.submit_prompt_response(prompt_id, Some(value), ctx);
                                                    }
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - cancel script");
                                                    view.submit_prompt_response(prompt_id, None, ctx);
                                                    view.cancel_script_execution(ctx);
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ArgPrompt", key_lower));
                                                }
                                            }
                                        }
                                    }
                                    AppView::EditorPrompt { entity, id, .. } => {
                                        // Editor prompt key handling for template/snippet navigation and choice popup
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to EditorPrompt", key_lower));
                                        let entity_clone = entity.clone();
                                        let prompt_id_clone = id.clone();

                                        // Check if choice popup is visible
                                        let has_choice_popup = entity_clone.update(ctx, |editor: &mut EditorPrompt, _| {
                                            editor.is_choice_popup_visible()
                                        });

                                        if has_choice_popup {
                                            // Handle choice popup navigation
                                            match key_lower.as_str() {
                                                "up" | "arrowup" => {
                                                    logging::log("STDIN", "SimulateKey: Up in choice popup");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_up_public(cx);
                                                    });
                                                }
                                                "down" | "arrowdown" => {
                                                    logging::log("STDIN", "SimulateKey: Down in choice popup");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_down_public(cx);
                                                    });
                                                }
                                                "enter" if !has_cmd => {
                                                    logging::log("STDIN", "SimulateKey: Enter in choice popup - confirming");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_confirm_public(window, cx);
                                                    });
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape in choice popup - cancelling");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_cancel_public(cx);
                                                    });
                                                }
                                                "tab" if !has_shift => {
                                                    logging::log("STDIN", "SimulateKey: Tab in choice popup - confirm and next");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_confirm_public(window, cx);
                                                        editor.next_tabstop_public(window, cx);
                                                    });
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in choice popup", key_lower));
                                                }
                                            }
                                        } else if key_lower == "tab" && !has_cmd {
                                            // Handle Tab key for snippet navigation
                                            entity_clone.update(ctx, |editor: &mut EditorPrompt, editor_cx| {
                                                logging::log("STDIN", "SimulateKey: Tab in EditorPrompt - calling next_tabstop");
                                                if editor.in_snippet_mode() {
                                                    editor.next_tabstop_public(window, editor_cx);
                                                } else {
                                                    logging::log("STDIN", "SimulateKey: Tab - not in snippet mode");
                                                }
                                            });
                                        } else if key_lower == "enter" && has_cmd {
                                            // Cmd+Enter submits - get content from editor
                                            logging::log("STDIN", "SimulateKey: Cmd+Enter in EditorPrompt - submitting");
                                            let content = entity_clone.update(ctx, |editor, editor_cx| {
                                                editor.content(editor_cx)
                                            });
                                            view.submit_prompt_response(prompt_id_clone.clone(), Some(content), ctx);
                                        } else if key_lower == "escape" && !has_cmd {
                                            logging::log("STDIN", "SimulateKey: Escape in EditorPrompt - cancelling");
                                            view.submit_prompt_response(prompt_id_clone.clone(), None, ctx);
                                            view.cancel_script_execution(ctx);
                                        } else {
                                            logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in EditorPrompt", key_lower));
                                        }
                                    }
                                    AppView::ChatPrompt { entity, .. } => {
                                        // ChatPrompt key handling
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to ChatPrompt (actions_popup={})", key_lower, view.show_actions_popup));

                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle chat actions");
                                            view.toggle_chat_actions(ctx, window);
                                        } else if view.show_actions_popup {
                                            // If actions popup is open, route to it
                                            if let Some(ref dialog) = view.actions_dialog {
                                                match key_lower.as_str() {
                                                    "up" | "arrowup" => {
                                                        logging::log("STDIN", "SimulateKey: Up in chat actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_up(cx));
                                                    }
                                                    "down" | "arrowdown" => {
                                                        logging::log("STDIN", "SimulateKey: Down in chat actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_down(cx));
                                                    }
                                                    "enter" => {
                                                        logging::log("STDIN", "SimulateKey: Enter in chat actions dialog");
                                                        let action_id = dialog.read(ctx).get_selected_action_id();
                                                        let should_close = dialog.read(ctx).selected_action_should_close();
                                                        if let Some(action_id) = action_id {
                                                            logging::log("ACTIONS", &format!("SimulateKey: Executing chat action: {} (close={})", action_id, should_close));
                                                            if should_close {
                                                                view.close_actions_popup(ActionsDialogHost::ChatPrompt, window, ctx);
                                                            }
                                                            view.execute_chat_action(&action_id, ctx);
                                                        }
                                                    }
                                                    "escape" => {
                                                        logging::log("STDIN", "SimulateKey: Escape - close chat actions dialog");
                                                        view.close_actions_popup(ActionsDialogHost::ChatPrompt, window, ctx);
                                                    }
                                                    _ => {
                                                        // Handle printable characters for search
                                                        if let Some(ch) = key_lower.chars().next() {
                                                            if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_' {
                                                                logging::log("STDIN", &format!("SimulateKey: Char '{}' in chat actions dialog", ch));
                                                                dialog.update(ctx, |d, cx| d.handle_char(ch, cx));
                                                            } else {
                                                                logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ChatPrompt actions dialog", key_lower));
                                                            }
                                                        }
                                                    }
                                                }
                                                // Notify the actions window to re-render
                                                crate::actions::notify_actions_window(ctx);
                                            }
                                        } else {
                                            // Route setup keys (tab, arrows, enter, escape) to ChatPrompt
                                            entity.update(ctx, |chat, cx| {
                                                if chat.handle_setup_key(&key_lower, has_shift, cx) {
                                                    logging::log("STDIN", &format!("SimulateKey: Setup handled '{}'", key_lower));
                                                } else {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled '{}' in ChatPrompt", key_lower));
                                                }
                                            });
                                        }
                                    }
                                    AppView::EmojiPickerView { ref mut selected_index, ref filter, ref selected_category } => {
                                        let ordered = crate::emoji::filtered_ordered_emojis(filter, *selected_category);
                                        let filtered_len = ordered.len();
                                        if filtered_len == 0 {
                                            return;
                                        }
                                        let cols = crate::emoji::GRID_COLS;
                                        match key_lower.as_str() {
                                            "up" | "arrowup" => {
                                                *selected_index = (*selected_index).saturating_sub(cols);
                                            }
                                            "down" | "arrowdown" => {
                                                *selected_index = (*selected_index + cols).min(filtered_len.saturating_sub(1));
                                            }
                                            "left" | "arrowleft" => {
                                                *selected_index = (*selected_index).saturating_sub(1);
                                            }
                                            "right" | "arrowright" => {
                                                *selected_index = (*selected_index + 1).min(filtered_len.saturating_sub(1));
                                            }
                                            "enter" => {
                                                if let Some(emoji) = ordered.get(*selected_index) {
                                                    ctx.write_to_clipboard(gpui::ClipboardItem::new_string(emoji.emoji.to_string()));
                                                    view.close_and_reset_window(ctx);
                                                }
                                                return;
                                            }
                                            "escape" => {
                                                view.close_and_reset_window(ctx);
                                                return;
                                            }
                                            _ => {
                                                logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in EmojiPicker", key_lower));
                                                return;
                                            }
                                        }
                                        // Scroll to row containing selected emoji
                                        let mut flat_offset: usize = 0;
                                        let mut row_offset: usize = 0;
                                        for cat in crate::emoji::ALL_CATEGORIES.iter().copied() {
                                            let cat_count = ordered.iter().filter(|e| e.category == cat).count();
                                            if cat_count == 0 { continue; }
                                            if flat_offset + cat_count > *selected_index {
                                                let idx_in_cat = *selected_index - flat_offset;
                                                row_offset += 1 + idx_in_cat / cols;
                                                break;
                                            }
                                            row_offset += 1 + cat_count.div_ceil(cols);
                                            flat_offset += cat_count;
                                        }
                                        view.emoji_scroll_handle.scroll_to_item(row_offset, gpui::ScrollStrategy::Nearest);
                                        view.input_mode = InputMode::Keyboard;
                                        view.hovered_index = None;
                                        ctx.notify();
                                    }
                                    _ => {
                                        logging::log("STDIN", &format!("SimulateKey: View {:?} not supported for key simulation", std::mem::discriminant(&view.current_view)));
                                    }
                                }
                            }
