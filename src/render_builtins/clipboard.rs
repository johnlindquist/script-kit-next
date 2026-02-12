impl ScriptListApp {
    /// Render clipboard history view
    /// P0 FIX: Data comes from self.cached_clipboard_entries, view passes only state
    fn render_clipboard_history(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use theme for all colors - consistent with main menu
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Use theme colors for consistency with main menu
        let opacity = self.theme.get_opacity();
        let bg_hex = self.theme.colors.background.main;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        // Removed: box_shadows - shadows on transparent elements block vibrancy
        let _box_shadows = self.create_box_shadows();

        // P0 FIX: Reference data from self instead of taking ownership
        // P1 FIX: NEVER do synchronous SQLite queries or image decoding in render loop!
        // Only copy from global cache (populated async by background prewarm thread).
        // Images not yet cached will show placeholder with dimensions from metadata.
        for entry in &self.cached_clipboard_entries {
            if entry.content_type == clipboard_history::ContentType::Image {
                // Only use already-cached images - NO synchronous fetch/decode
                if !self.clipboard_image_cache.contains_key(&entry.id) {
                    if let Some(cached) = clipboard_history::get_cached_image(&entry.id) {
                        self.clipboard_image_cache.insert(entry.id.clone(), cached);
                    }
                    // If not in global cache yet, background thread will populate it.
                    // We'll show placeholder with dimensions until then.
                }
            }
        }

        // Clone the cache for use in closures
        let image_cache = self.clipboard_image_cache.clone();

        // Filter entries based on current filter
        let filtered_entries: Vec<_> = if filter.is_empty() {
            self.cached_clipboard_entries.iter().enumerate().collect()
        } else {
            let filter_lower = filter.to_lowercase();
            self.cached_clipboard_entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    let matches = e.text_preview.to_lowercase().contains(&filter_lower)
                        || e.ocr_text
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&filter_lower);
                    matches
                })
                .collect()
        };
        let filtered_len = filtered_entries.len();

        // Key handler for clipboard history
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

                let key_str = event.keystroke.key.to_lowercase();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;
                let modifiers = &event.keystroke.modifiers;

                // Route keys to actions dialog first if it's open
                match this.route_key_to_actions_dialog(
                    &key_str,
                    key_char,
                    modifiers,
                    ActionsDialogHost::ClipboardHistory,
                    window,
                    cx,
                ) {
                    ActionsRoute::NotHandled => {}
                    ActionsRoute::Handled => {
                        return;
                    }
                    ActionsRoute::Execute { action_id } => {
                        this.handle_action(action_id, cx);
                        return;
                    }
                }

                // ESC: Clear filter first if present, otherwise go back/close
                if key_str == "escape" && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }

                // Cmd+W always closes window
                if has_cmd && key_str == "w" {
                    logging::log("KEY", "Cmd+W - closing window");
                    this.close_and_reset_window(cx);
                    return;
                }

                logging::log("KEY", &format!("ClipboardHistory key: '{}'", key_str));

                // P0 FIX: View state only - data comes from this.cached_clipboard_entries
                if let AppView::ClipboardHistoryView {
                    filter,
                    selected_index,
                } = &mut this.current_view
                {
                    // Apply filter to get current filtered list
                    // P0 FIX: Reference cached_clipboard_entries from self
                    let filtered_entries: Vec<_> = if filter.is_empty() {
                        this.cached_clipboard_entries.iter().enumerate().collect()
                    } else {
                        let filter_lower = filter.to_lowercase();
                        this.cached_clipboard_entries
                            .iter()
                            .enumerate()
                            .filter(|(_, e)| {
                                let matches = e.text_preview.to_lowercase().contains(&filter_lower)
                                    || e.ocr_text
                                        .as_deref()
                                        .unwrap_or("")
                                        .to_lowercase()
                                        .contains(&filter_lower);
                                matches
                            })
                            .collect()
                    };
                    let filtered_len = filtered_entries.len();
                    let selected_entry = filtered_entries
                        .get(*selected_index)
                        .map(|(_, entry)| (*entry).clone());
                    this.focused_clipboard_entry_id =
                        selected_entry.as_ref().map(|entry| entry.id.clone());

                    // Cmd+P toggles pin state for selected entry
                    if has_cmd && key_str == "p" {
                        if let Some(entry) = selected_entry {
                            drop(filtered_entries);
                            let action_id = if entry.pinned {
                                "clipboard_unpin"
                            } else {
                                "clipboard_pin"
                            };
                            this.handle_action(action_id.to_string(), cx);
                        }
                        return;
                    }

                    // Cmd+K opens clipboard actions dialog
                    if has_cmd && key_str == "k" {
                        if let Some(entry) = selected_entry {
                            drop(filtered_entries);
                            this.toggle_clipboard_actions(entry, window, cx);
                        }
                        return;
                    }

                    // Ctrl+Cmd+A attaches selected entry to AI chat
                    if modifiers.control && has_cmd && key_str == "a" {
                        if let Some(_entry) = selected_entry {
                            drop(filtered_entries);
                            this.handle_action("clipboard_attach_to_ai".to_string(), cx);
                        }
                        return;
                    }

                    // Space opens Quick Look (macOS Finder behavior)
                    if key_str == "space"
                        && filter.is_empty()
                        && !modifiers.platform
                        && !modifiers.control
                        && !modifiers.alt
                        && !modifiers.shift
                    {
                        if let Some(entry) = selected_entry {
                            if let Err(e) = clipboard_history::quick_look_entry(&entry) {
                                logging::log("ERROR", &format!("Quick Look failed: {}", e));
                                this.show_hud(format!("Quick Look failed: {}", e), Some(2500), cx);
                            }
                        }
                        return;
                    }

                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                // Scroll to keep selection visible
                                this.clipboard_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                this.focused_clipboard_entry_id = filtered_entries
                                    .get(*selected_index)
                                    .map(|(_, entry)| entry.id.clone());
                                cx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if *selected_index < filtered_len.saturating_sub(1) {
                                *selected_index += 1;
                                // Scroll to keep selection visible
                                this.clipboard_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                this.focused_clipboard_entry_id = filtered_entries
                                    .get(*selected_index)
                                    .map(|(_, entry)| entry.id.clone());
                                cx.notify();
                            }
                        }
                        "enter" | "return" => {
                            // Copy selected entry to clipboard, hide window, then paste
                            if let Some((_, entry)) = filtered_entries.get(*selected_index) {
                                logging::log(
                                    "EXEC",
                                    &format!("Copying clipboard entry: {}", entry.id),
                                );
                                if let Err(e) =
                                    clipboard_history::copy_entry_to_clipboard(&entry.id)
                                {
                                    logging::log("ERROR", &format!("Failed to copy entry: {}", e));
                                } else {
                                    logging::log("EXEC", "Entry copied to clipboard");
                                    // Hide main window only (not entire app) to keep HUD visible
                                    script_kit_gpui::set_main_window_visible(false);
                                    platform::hide_main_window();
                                    NEEDS_RESET.store(true, Ordering::SeqCst);

                                    // Simulate Cmd+V paste after a brief delay to let focus return
                                    std::thread::spawn(|| {
                                        std::thread::sleep(std::time::Duration::from_millis(100));
                                        if let Err(e) = selected_text::simulate_paste_with_cg() {
                                            logging::log(
                                                "ERROR",
                                                &format!("Failed to simulate paste: {}", e),
                                            );
                                        } else {
                                            logging::log("EXEC", "Simulated Cmd+V paste");
                                        }
                                    });
                                }
                            }
                        }
                        // Note: "escape" is handled by handle_global_shortcut_with_options above
                        // Text input (backspace, characters) is handled by the shared Input component
                        // which syncs via handle_filter_input_change()
                        _ => {}
                    }
                }
            },
        );


        // Pre-compute colors - use theme for consistency with main menu
        let list_colors = ListItemColors::from_theme(&self.theme);
        let text_primary = self.theme.colors.text.primary;
        #[allow(unused_variables)]
        let text_muted = self.theme.colors.text.muted;
        let text_dimmed = self.theme.colors.text.dimmed;
        let ui_border = self.theme.colors.ui.border;

        // Build virtualized list
        let list_element: AnyElement = if filtered_len == 0 {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(self.theme.colors.text.muted))
                .font_family(design_typography.font_family)
                .child(if filter.is_empty() {
                    "No clipboard history"
                } else {
                    "No entries match your filter"
                })
                .into_any_element()
        } else {
            // Clone data for the closure
            let entries_for_closure: Vec<_> = filtered_entries
                .iter()
                .map(|(i, e)| (*i, (*e).clone()))
                .collect();
            let selected = selected_index;
            let hovered = self.hovered_index;
            let current_input_mode = self.input_mode;
            let image_cache_for_list = image_cache.clone();
            let click_entity_handle = cx.entity().downgrade();
            let hover_entity_handle = cx.entity().downgrade();

            uniform_list(
                "clipboard-history",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_, entry)) = entries_for_closure.get(ix) {
                                let is_selected = ix == selected;
                                let is_hovered = hovered == Some(ix) && current_input_mode == InputMode::Mouse;

                                // Get cached thumbnail for images
                                let cached_image = if entry.content_type
                                    == clipboard_history::ContentType::Image
                                {
                                    image_cache_for_list.get(&entry.id).cloned()
                                } else {
                                    None
                                };

                                // Use display_preview() from ClipboardEntryMeta
                                let display_content = entry.display_preview();

                                // Format relative time (entry.timestamp is in milliseconds)
                                let now_ms = chrono::Utc::now().timestamp_millis();
                                let age_secs = (now_ms - entry.timestamp) / 1000;
                                let relative_time = if age_secs < 60 {
                                    "just now".to_string()
                                } else if age_secs < 3600 {
                                    format!("{}m ago", age_secs / 60)
                                } else if age_secs < 86400 {
                                    format!("{}h ago", age_secs / 3600)
                                } else {
                                    format!("{}d ago", age_secs / 86400)
                                };

                                // Add pin indicator
                                let name = if entry.pinned {
                                    format!("📌 {}", display_content)
                                } else {
                                    display_content
                                };

                                // Build list item with optional thumbnail
                                let mut item = ListItem::new(name, list_colors)
                                    .description_opt(Some(relative_time))
                                    .selected(is_selected)
                                    .hovered(is_hovered)
                                    .with_hover_effect(current_input_mode == InputMode::Mouse)
                                    .with_accent_bar(true);

                                // Add thumbnail for images, text icon for text entries
                                if let Some(render_image) = cached_image {
                                    item = item.icon_image(render_image);
                                } else if entry.content_type == clipboard_history::ContentType::Text
                                {
                                    item = item.icon("📄");
                                }

                                // Click handler: select on click, paste on double-click
                                let click_entity = click_entity_handle.clone();
                                let entry_id = entry.id.clone();
                                let click_handler = move |event: &gpui::ClickEvent,
                                                           _window: &mut Window,
                                                           cx: &mut gpui::App| {
                                    if let Some(app) = click_entity.upgrade() {
                                        let entry_id = entry_id.clone();
                                        app.update(cx, |this, cx| {
                                            if let AppView::ClipboardHistoryView {
                                                selected_index, ..
                                            } = &mut this.current_view
                                            {
                                                *selected_index = ix;
                                            }
                                            this.focused_clipboard_entry_id =
                                                Some(entry_id.clone());
                                            cx.notify();

                                            // Double-click: copy and paste
                                            if let gpui::ClickEvent::Mouse(mouse_event) = event {
                                                if mouse_event.down.click_count == 2 {
                                                    logging::log(
                                                        "UI",
                                                        &format!(
                                                            "Double-click paste clipboard entry {}",
                                                            entry_id
                                                        ),
                                                    );
                                                    if clipboard_history::copy_entry_to_clipboard(
                                                        &entry_id,
                                                    )
                                                    .is_ok()
                                                    {
                                                        script_kit_gpui::set_main_window_visible(
                                                            false,
                                                        );
                                                        platform::hide_main_window();
                                                        NEEDS_RESET
                                                            .store(true, Ordering::SeqCst);
                                                        std::thread::spawn(|| {
                                                            std::thread::sleep(
                                                                std::time::Duration::from_millis(
                                                                    100,
                                                                ),
                                                            );
                                                            let _ = selected_text::simulate_paste_with_cg();
                                                        });
                                                    }
                                                }
                                            }
                                        });
                                    }
                                };

                                // Hover handler for mouse tracking
                                let hover_entity = hover_entity_handle.clone();
                                let hover_handler = move |is_hovered: &bool, _window: &mut Window, cx: &mut gpui::App| {
                                    if let Some(app) = hover_entity.upgrade() {
                                        app.update(cx, |this, cx| {
                                            if *is_hovered {
                                                this.input_mode = InputMode::Mouse;
                                                if this.hovered_index != Some(ix) {
                                                    this.hovered_index = Some(ix);
                                                    cx.notify();
                                                }
                                            } else if this.hovered_index == Some(ix) {
                                                this.hovered_index = None;
                                                cx.notify();
                                            }
                                        });
                                    }
                                };

                                div()
                                    .id(ix)
                                    .cursor_pointer()
                                    .on_click(click_handler)
                                    .on_hover(hover_handler)
                                    .child(item)
                            } else {
                                div().id(ix).h(px(LIST_ITEM_HEIGHT))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.clipboard_list_scroll_handle)
            .into_any_element()
        };

        // Build preview panel for selected entry
        let selected_entry = filtered_entries
            .get(selected_index)
            .map(|(_, e)| (*e).clone());
        let has_entry = selected_entry.is_some();
        let selected_entry_for_footer = selected_entry.clone();
        let preview_panel = self.render_clipboard_preview_panel(
            &selected_entry,
            &image_cache,
            &design_spacing,
            &design_typography,
            &design_visual,
        );

        div()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            // Removed: .shadow(box_shadows) - shadows on transparent elements block vibrancy
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("clipboard_history")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header with input - uses shared gpui_input_state for consistent cursor/selection
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    // Search input - shared component with main menu
                    .child(
                        div().flex_1().flex().flex_row().items_center().child(
                            Input::new(&self.gpui_input_state)
                                .w_full()
                                .h(px(28.))
                                .px(px(0.))
                                .py(px(0.))
                                .with_size(Size::Size(px(design_typography.font_size_xl)))
                                .appearance(false)
                                .bordered(false)
                                .focus_bordered(false),
                        ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(format!("{} entries", self.cached_clipboard_entries.len())),
                    ),
            )
            // Divider
            .child(
                div()
                    .mx(px(design_spacing.padding_lg))
                    .h(px(design_visual.border_thin))
                    .bg(rgba((ui_border << 8) | 0x60)),
            )
            // Main content area - 50/50 split: List on left, Preview on right
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h(px(0.))
                    .w_full()
                    .overflow_hidden()
                    // Left side: Clipboard list (50% width)
                    .child(
                        div()
                            .w_1_2()
                            .h_full()
                            .min_h(px(0.))
                            .py(px(design_spacing.padding_xs))
                            .child(list_element),
                    )
                    // Right side: Preview panel (50% width)
                    .child(
                        div()
                            .w_1_2()
                            .h_full()
                            .min_h(px(0.))
                            .overflow_hidden()
                            .child(preview_panel),
                    ),
            )
            // Footer
            .child({
                let handle_actions = cx.entity().downgrade();

                let footer_config = PromptFooterConfig::new()
                    .primary_label("Paste")
                    .primary_shortcut("↵")
                    .show_secondary(has_entry);

                PromptFooter::new(footer_config, PromptFooterColors::from_theme(&self.theme))
                    .on_secondary_click(Box::new(move |_, window, cx| {
                        if let Some(app) = handle_actions.upgrade() {
                            if let Some(entry) = selected_entry_for_footer.clone() {
                                app.update(cx, |this, cx| {
                                    this.toggle_clipboard_actions(entry, window, cx);
                                });
                            }
                        }
                    }))
            })
            .into_any_element()

    }
}
