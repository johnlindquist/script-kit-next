// Builtin view render methods - extracted from app_render.rs
// This file is included via include!() macro in main.rs
// Contains: render_clipboard_history, render_app_launcher, render_window_switcher, render_design_gallery

impl ScriptListApp {
    /// Available vibrancy material presets for the theme customizer
    const VIBRANCY_MATERIALS: &[(theme::VibrancyMaterial, &str)] = &[
        (theme::VibrancyMaterial::Hud, "HUD"),
        (theme::VibrancyMaterial::Popover, "Popover"),
        (theme::VibrancyMaterial::Menu, "Menu"),
        (theme::VibrancyMaterial::Sidebar, "Sidebar"),
        (theme::VibrancyMaterial::Content, "Content"),
    ];

    /// Available font size presets for the theme customizer
    const FONT_SIZE_PRESETS: &[(f32, &str)] = &[
        (12.0, "12"),
        (13.0, "13"),
        (14.0, "14"),
        (15.0, "15"),
        (16.0, "16"),
        (18.0, "18"),
        (20.0, "20"),
    ];

    /// Find the index of a vibrancy material in the presets array
    fn find_vibrancy_material_index(material: theme::VibrancyMaterial) -> usize {
        Self::VIBRANCY_MATERIALS
            .iter()
            .position(|(m, _)| *m == material)
            .unwrap_or(0)
    }

    /// Return a human-readable name for a hex accent color
    fn accent_color_name(color: u32) -> &'static str {
        match color {
            0xfbbf24 => "Yellow Gold",
            0xf59e0b => "Amber",
            0xf97316 => "Orange",
            0xef4444 => "Red",
            0xec4899 => "Pink",
            0xa855f7 => "Purple",
            0x6366f1 => "Indigo",
            0x3b82f6 => "Blue",
            0x0078d4 => "Blue",
            0x0ea5e9 => "Sky",
            0x14b8a6 => "Teal",
            0x22c55e => "Green",
            0x84cc16 => "Lime",
            _ => "Custom",
        }
    }

    /// Toggle the actions dialog for file search results
    /// Opens a popup with file-specific actions: Open, Show in Finder, Quick Look, etc.
    fn toggle_file_search_actions(
        &mut self,
        file: &file_search::FileResult,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        logging::log("KEY", "Toggling file search actions popup");

        if self.show_actions_popup || is_actions_window_open() {
            // Close the actions popup
            self.show_actions_popup = false;
            self.actions_dialog = None;
            self.file_search_actions_path = None;

            // Close the actions window via spawn
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                })
                .ok();
            })
            .detach();

            // Use coordinator to restore focus (will pop the overlay and set pending_focus)
            self.pop_focus_overlay(cx);

            // Also directly focus main filter for immediate feedback
            self.focus_main_filter(window, cx);
            logging::log(
                "FOCUS",
                "File search actions closed, focus restored via coordinator",
            );
        } else {
            // Open actions popup for the selected file
            self.show_actions_popup = true;

            // Use coordinator to push overlay - saves current focus state for restore
            self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

            // CRITICAL: Transfer focus from Input to main focus_handle
            // This prevents the Input from receiving text (which would go to file search filter)
            // while keeping keyboard focus in main window for routing to actions dialog
            self.focus_handle.focus(window, cx);
            self.gpui_input_focused = false;
            self.focused_input = FocusedInput::ActionsSearch;

            // Store the file path for action handling
            self.file_search_actions_path = Some(file.path.clone());

            // Create file info from the result
            let file_info = file_search::FileInfo::from_result(file);

            // Create the dialog entity
            let theme_arc = std::sync::Arc::clone(&self.theme);
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                ActionsDialog::with_file(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                    &file_info,
                    theme_arc,
                )
            });

            // Store the dialog entity for keyboard routing
            self.actions_dialog = Some(dialog.clone());

            // Set up the on_close callback to restore focus when escape is pressed in ActionsWindow
            // Match what close_actions_popup does for FileSearch host
            let app_entity = cx.entity().clone();
            dialog.update(cx, |d, _cx| {
                d.set_on_close(std::sync::Arc::new(move |cx| {
                    app_entity.update(cx, |app, cx| {
                        app.show_actions_popup = false;
                        app.actions_dialog = None;
                        app.file_search_actions_path = None;
                        // Use coordinator to pop overlay and restore previous focus
                        app.pop_focus_overlay(cx);
                        logging::log(
                            "FOCUS",
                            "File search actions closed via escape, focus restored via coordinator",
                        );
                    });
                }));
            });

            // Get main window bounds and display_id for positioning
            let main_bounds = window.bounds();
            let display_id = window.display(cx).map(|d| d.id());

            logging::log(
                "ACTIONS",
                &format!(
                    "Opening file search actions for: {} (is_dir={})",
                    file_info.name, file_info.is_dir
                ),
            );

            // Open the actions window
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    match open_actions_window(
                        cx,
                        main_bounds,
                        display_id,
                        dialog,
                        crate::actions::WindowPosition::BottomRight,
                    ) {
                        Ok(_handle) => {
                            logging::log("ACTIONS", "File search actions popup window opened");
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open actions window: {}", e));
                        }
                    }
                })
                .ok();
            })
            .detach();
        }
        cx.notify();
    }

    /// Toggle the actions dialog for a clipboard history entry
    fn toggle_clipboard_actions(
        &mut self,
        entry: clipboard_history::ClipboardEntryMeta,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        logging::log("KEY", "Toggling clipboard actions popup");

        if self.show_actions_popup || is_actions_window_open() {
            // Close the actions popup
            self.show_actions_popup = false;
            self.actions_dialog = None;

            // Close the actions window via spawn
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                })
                .ok();
            })
            .detach();

            // Use coordinator to restore focus (will pop the overlay and set pending_focus)
            self.pop_focus_overlay(cx);

            // Also directly focus main filter for immediate feedback
            self.focus_main_filter(window, cx);
            logging::log(
                "FOCUS",
                "Clipboard actions closed, focus restored via coordinator",
            );
        } else {
            // Open actions popup for the selected clipboard entry
            self.show_actions_popup = true;
            self.focused_clipboard_entry_id = Some(entry.id.clone());

            // Use coordinator to push overlay - saves current focus state for restore
            self.push_focus_overlay(focus_coordinator::FocusRequest::actions_dialog(), cx);

            // Transfer focus from Input to main focus_handle for actions routing
            self.focus_handle.focus(window, cx);
            self.gpui_input_focused = false;
            self.focused_input = FocusedInput::ActionsSearch;

            let entry_content_type = entry.content_type;
            let entry_info = crate::actions::ClipboardEntryInfo {
                id: entry.id.clone(),
                content_type: entry.content_type,
                pinned: entry.pinned,
                preview: entry.display_preview(),
                image_dimensions: entry.image_width.zip(entry.image_height),
                frontmost_app_name: None,
            };

            // Create the dialog entity
            let theme_arc = std::sync::Arc::clone(&self.theme);
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                ActionsDialog::with_clipboard_entry(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                    &entry_info,
                    theme_arc,
                )
            });

            // Store the dialog entity for keyboard routing
            self.actions_dialog = Some(dialog.clone());

            // Set up the on_close callback to restore focus when escape is pressed
            let app_entity = cx.entity().clone();
            dialog.update(cx, |d, _cx| {
                d.set_on_close(std::sync::Arc::new(move |cx| {
                    app_entity.update(cx, |app, cx| {
                        app.show_actions_popup = false;
                        app.actions_dialog = None;
                        // Use coordinator to pop overlay and restore previous focus
                        app.pop_focus_overlay(cx);
                        logging::log(
                            "FOCUS",
                            "Clipboard actions closed via escape, focus restored via coordinator",
                        );
                    });
                }));
            });

            // Get main window bounds and display_id for positioning
            let main_bounds = window.bounds();
            let display_id = window.display(cx).map(|d| d.id());

            logging::log(
                "ACTIONS",
                &format!(
                    "Opening clipboard actions for entry: {} (type={:?}, pinned={})",
                    entry.id, entry_content_type, entry.pinned
                ),
            );

            // Open the actions window
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    match open_actions_window(
                        cx,
                        main_bounds,
                        display_id,
                        dialog,
                        crate::actions::WindowPosition::BottomRight,
                    ) {
                        Ok(_handle) => {
                            logging::log("ACTIONS", "Clipboard actions popup window opened");
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open actions window: {}", e));
                        }
                    }
                })
                .ok();
            })
            .detach();
        }
        cx.notify();
    }

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
                .filter(|(_, e)| e.text_preview.to_lowercase().contains(&filter_lower))
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
                            .filter(|(_, e)| e.text_preview.to_lowercase().contains(&filter_lower))
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
            let image_cache_for_list = image_cache.clone();

            uniform_list(
                "clipboard-history",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_, entry)) = entries_for_closure.get(ix) {
                                let is_selected = ix == selected;

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
                                    .with_accent_bar(true);

                                // Add thumbnail for images, text icon for text entries
                                if let Some(render_image) = cached_image {
                                    item = item.icon_image(render_image);
                                } else if entry.content_type == clipboard_history::ContentType::Text
                                {
                                    item = item.icon("📄");
                                }

                                div().id(ix).child(item)
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

    /// Render the preview panel for clipboard history
    fn render_clipboard_preview_panel(
        &self,
        selected_entry: &Option<clipboard_history::ClipboardEntryMeta>,
        image_cache: &std::collections::HashMap<String, Arc<gpui::RenderImage>>,
        spacing: &designs::DesignSpacing,
        typography: &designs::DesignTypography,
        visual: &designs::DesignVisual,
    ) -> impl IntoElement {
        // Use theme colors for consistency with main menu
        let bg_main = self.theme.colors.background.main;
        let ui_border = self.theme.colors.ui.border;
        let text_primary = self.theme.colors.text.primary;
        let text_muted = self.theme.colors.text.muted;
        let text_secondary = self.theme.colors.text.secondary;
        let bg_search_box = self.theme.colors.background.search_box;
        let accent = self.theme.colors.accent.selected;

        // Use theme opacity for vibrancy-compatible backgrounds
        let opacity = self.theme.get_opacity();
        let panel_alpha = (opacity.main * 255.0 * 0.3) as u8; // 30% of main opacity for subtle tint

        let mut panel = div()
            .w_full()
            .h_full()
            // Semi-transparent background to let vibrancy show through
            .bg(rgba((bg_main << 8) | panel_alpha as u32))
            .border_l_1()
            .border_color(rgba((ui_border << 8) | 0x30)) // Subtle border
            .p(px(spacing.padding_lg))
            .flex()
            .flex_col()
            .overflow_y_hidden()
            .font_family(typography.font_family);

        match selected_entry {
            Some(entry) => {
                // Header with content type
                let content_type_label = match entry.content_type {
                    clipboard_history::ContentType::Text => "Text",
                    clipboard_history::ContentType::Image => "Image",
                };

                panel = panel.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .pb(px(spacing.padding_sm))
                        // Content type badge
                        .child(
                            div()
                                .px(px(spacing.padding_sm))
                                .py(px(spacing.padding_xs / 2.0))
                                .rounded(px(visual.radius_sm))
                                .bg(rgba((accent << 8) | 0x30))
                                .text_xs()
                                .text_color(rgb(accent))
                                .child(content_type_label),
                        )
                        // Pin indicator
                        .when(entry.pinned, |d| {
                            d.child(
                                div()
                                    .px(px(spacing.padding_sm))
                                    .py(px(spacing.padding_xs / 2.0))
                                    .rounded(px(visual.radius_sm))
                                    .bg(rgba((accent << 8) | 0x20))
                                    .text_xs()
                                    .text_color(rgb(accent))
                                    .child("📌 Pinned"),
                            )
                        }),
                );

                // Timestamp
                let now = chrono::Utc::now().timestamp();
                let age_secs = now - entry.timestamp;
                let relative_time = if age_secs < 60 {
                    "just now".to_string()
                } else if age_secs < 3600 {
                    format!("{} minutes ago", age_secs / 60)
                } else if age_secs < 86400 {
                    format!("{} hours ago", age_secs / 3600)
                } else {
                    format!("{} days ago", age_secs / 86400)
                };

                panel = panel.child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .pb(px(spacing.padding_md))
                        .child(relative_time),
                );

                // Divider
                panel = panel.child(
                    div()
                        .w_full()
                        .h(px(visual.border_thin))
                        .bg(rgba((ui_border << 8) | 0x60))
                        .my(px(spacing.padding_sm)),
                );

                // Content preview
                panel = panel.child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .pb(px(spacing.padding_sm))
                        .child("Content Preview"),
                );

                match entry.content_type {
                    clipboard_history::ContentType::Text => {
                        // Fetch full content on-demand for preview
                        let content = clipboard_history::get_entry_content(&entry.id)
                            .unwrap_or_else(|| entry.text_preview.clone());
                        let char_count = content.chars().count();
                        let line_count = content.lines().count();

                        // Use subtle background for content preview - 20% opacity for vibrancy
                        let content_alpha = (opacity.main * 255.0 * 0.5) as u32;
                        panel = panel
                            .child(
                                div()
                                    .w_full()
                                    .flex_1()
                                    .p(px(spacing.padding_md))
                                    .rounded(px(visual.radius_md))
                                    .bg(rgba((bg_search_box << 8) | content_alpha))
                                    .overflow_hidden()
                                    .font_family(typography.font_family_mono)
                                    .text_sm()
                                    .text_color(rgb(text_primary))
                                    .child(content),
                            )
                            // Stats footer
                            .child(
                                div()
                                    .pt(px(spacing.padding_sm))
                                    .text_xs()
                                    .text_color(rgb(text_secondary))
                                    .child(format!(
                                        "{} characters • {} lines",
                                        char_count, line_count
                                    )),
                            );
                    }
                    clipboard_history::ContentType::Image => {
                        // Get image dimensions from metadata
                        let width = entry.image_width.unwrap_or(0);
                        let height = entry.image_height.unwrap_or(0);

                        // Try to get cached render image
                        let cached_image = image_cache.get(&entry.id).cloned();

                        let image_container = if let Some(render_image) = cached_image {
                            // Calculate display size that fits in the preview panel
                            // Max size is 300x300, maintain aspect ratio
                            let max_size: f32 = 300.0;
                            let (display_w, display_h) = if width > 0 && height > 0 {
                                let w = width as f32;
                                let h = height as f32;
                                let scale = (max_size / w).min(max_size / h).min(1.0);
                                (w * scale, h * scale)
                            } else {
                                (max_size, max_size)
                            };

                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_2()
                                // Actual image thumbnail
                                .child(
                                    gpui::img(move |_window: &mut Window, _cx: &mut App| {
                                        Some(Ok(render_image.clone()))
                                    })
                                    .w(px(display_w))
                                    .h(px(display_h))
                                    .object_fit(gpui::ObjectFit::Contain)
                                    .rounded(px(visual.radius_sm)),
                                )
                                // Dimensions label below image
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(format!("{}×{} pixels", width, height)),
                                )
                        } else {
                            // Fallback if image not in cache (shouldn't happen)
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_2()
                                .child(div().text_2xl().child("🖼️"))
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgb(text_primary))
                                        .child(format!("{}×{}", width, height)),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_muted))
                                        .child("Loading image..."),
                                )
                        };

                        // Image container - use very subtle background for vibrancy
                        let image_bg_alpha = (opacity.main * 255.0 * 0.15) as u32; // 15% of main opacity
                        panel = panel.child(
                            div()
                                .w_full()
                                .flex_1()
                                .p(px(spacing.padding_lg))
                                .rounded(px(visual.radius_md))
                                .bg(rgba((bg_search_box << 8) | image_bg_alpha))
                                .flex()
                                .items_center()
                                .justify_center()
                                .overflow_hidden()
                                .child(image_container),
                        );
                    }
                }
            }
            None => {
                // Empty state
                panel = panel.child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(text_muted))
                        .child("No entry selected"),
                );
            }
        }

        panel
    }

    /// Render app launcher view
    /// P0 FIX: Data comes from self.apps, view passes only state
    fn render_app_launcher(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for spacing/typography/visual, theme for colors
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = self.theme.colors.background.main;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        // Removed: box_shadows - shadows on transparent elements block vibrancy
        let _box_shadows = self.create_box_shadows();

        // P0 FIX: Filter apps from self.apps instead of taking ownership
        let filtered_apps: Vec<_> = if filter.is_empty() {
            self.apps.iter().enumerate().collect()
        } else {
            let filter_lower = filter.to_lowercase();
            self.apps
                .iter()
                .enumerate()
                .filter(|(_, a)| a.name.to_lowercase().contains(&filter_lower))
                .collect()
        };
        let filtered_len = filtered_apps.len();

        // Key handler for app launcher
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
                let has_cmd = event.keystroke.modifiers.platform;

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

                logging::log("KEY", &format!("AppLauncher key: '{}'", key_str));

                // P0 FIX: View state only - data comes from this.apps
                if let AppView::AppLauncherView {
                    filter,
                    selected_index,
                } = &mut this.current_view
                {
                    // Apply filter to get current filtered list
                    // P0 FIX: Reference apps from self
                    let filtered_apps: Vec<_> = if filter.is_empty() {
                        this.apps.iter().enumerate().collect()
                    } else {
                        let filter_lower = filter.to_lowercase();
                        this.apps
                            .iter()
                            .enumerate()
                            .filter(|(_, a)| a.name.to_lowercase().contains(&filter_lower))
                            .collect()
                    };
                    let filtered_len = filtered_apps.len();

                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                cx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if *selected_index < filtered_len.saturating_sub(1) {
                                *selected_index += 1;
                                cx.notify();
                            }
                        }
                        "enter" | "return" => {
                            // Launch selected app and hide window
                            if let Some((_, app)) = filtered_apps.get(*selected_index) {
                                logging::log("EXEC", &format!("Launching app: {}", app.name));
                                if let Err(e) = app_launcher::launch_application(app) {
                                    logging::log("ERROR", &format!("Failed to launch app: {}", e));
                                } else {
                                    logging::log("EXEC", &format!("Launched: {}", app.name));
                                    // Hide main window only (not entire app) to keep HUD visible
                                    script_kit_gpui::set_main_window_visible(false);
                                    platform::hide_main_window();
                                    NEEDS_RESET.store(true, Ordering::SeqCst);
                                }
                            }
                        }
                        // Note: "escape" is handled by handle_global_shortcut_with_options above
                        "backspace" => {
                            if !filter.is_empty() {
                                filter.pop();
                                *selected_index = 0;
                                cx.notify();
                            }
                        }
                        _ => {
                            if let Some(ref key_char) = event.keystroke.key_char {
                                if let Some(ch) = key_char.chars().next() {
                                    if !ch.is_control() {
                                        filter.push(ch);
                                        *selected_index = 0;
                                        cx.notify();
                                    }
                                }
                            }
                        }
                    }
                }
            },
        );

        let input_display = if filter.is_empty() {
            SharedString::from("Search applications...")
        } else {
            SharedString::from(filter.clone())
        };
        let input_is_empty = filter.is_empty();

        // Pre-compute colors
        let list_colors = ListItemColors::from_theme(&self.theme);
        let text_primary = self.theme.colors.text.primary;
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
                    "No applications found"
                } else {
                    "No apps match your filter"
                })
                .into_any_element()
        } else {
            // Clone data for the closure
            let apps_for_closure: Vec<_> = filtered_apps
                .iter()
                .map(|(i, a)| (*i, (*a).clone()))
                .collect();
            let selected = selected_index;

            uniform_list(
                "app-launcher",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_, app)) = apps_for_closure.get(ix) {
                                let is_selected = ix == selected;

                                // Format app path for description
                                let path_str = app.path.to_string_lossy();
                                let description = if path_str.starts_with("/Applications") {
                                    None // No need to show path for standard apps
                                } else {
                                    Some(path_str.to_string())
                                };

                                // Use pre-decoded icon if available, fallback to emoji
                                let icon = match &app.icon {
                                    Some(img) => list_item::IconKind::Image(img.clone()),
                                    None => list_item::IconKind::Emoji("📱".to_string()),
                                };

                                div().id(ix).child(
                                    ListItem::new(app.name.clone(), list_colors)
                                        .icon_kind(icon)
                                        .description_opt(description)
                                        .selected(is_selected)
                                        .with_accent_bar(true),
                                )
                            } else {
                                div().id(ix).h(px(LIST_ITEM_HEIGHT))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.list_scroll_handle)
            .into_any_element()
        };

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
            .key_context("app_launcher")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header with input
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    // Title
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child("🚀 Apps"),
                    )
                    // Search input with blinking cursor
                    // ALIGNMENT FIX: Uses canonical cursor constants and negative margin for placeholder
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_lg()
                            .text_color(if input_is_empty {
                                rgb(text_muted)
                            } else {
                                rgb(text_primary)
                            })
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .mr(px(CURSOR_GAP_X))
                                        .when(self.cursor_visible, |d| d.bg(rgb(text_primary))),
                                )
                            })
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .ml(px(-(CURSOR_WIDTH + CURSOR_GAP_X)))
                                        .child(input_display.clone()),
                                )
                            })
                            .when(!input_is_empty, |d| d.child(input_display.clone()))
                            .when(!input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .ml(px(CURSOR_GAP_X))
                                        .when(self.cursor_visible, |d| d.bg(rgb(text_primary))),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(format!("{} apps", self.apps.len())),
                    ),
            )
            // Divider
            .child(
                div()
                    .mx(px(design_spacing.padding_lg))
                    .h(px(design_visual.border_thin))
                    .bg(rgba((ui_border << 8) | 0x60)),
            )
            // App list
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.))
                    .w_full()
                    .py(px(design_spacing.padding_xs))
                    .child(list_element),
            )
            // Footer
            .child(PromptFooter::new(
                PromptFooterConfig::new()
                    .primary_label("Launch")
                    .primary_shortcut("↵")
                    .show_secondary(false),
                PromptFooterColors::from_theme(&self.theme),
            ))
            .into_any_element()
    }

    /// Render window switcher view with 50/50 split layout
    /// P0 FIX: Data comes from self.cached_windows, view passes only state
    fn render_window_switcher(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = self.theme.colors.background.main;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        // Removed: box_shadows - shadows on transparent elements block vibrancy
        let _box_shadows = self.create_box_shadows();

        // P0 FIX: Filter windows from self.cached_windows instead of taking ownership
        let filtered_windows: Vec<_> = if filter.is_empty() {
            self.cached_windows.iter().enumerate().collect()
        } else {
            let filter_lower = filter.to_lowercase();
            self.cached_windows
                .iter()
                .enumerate()
                .filter(|(_, w)| {
                    w.title.to_lowercase().contains(&filter_lower)
                        || w.app.to_lowercase().contains(&filter_lower)
                })
                .collect()
        };
        let filtered_len = filtered_windows.len();

        // Key handler for window switcher
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
                let has_cmd = event.keystroke.modifiers.platform;

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

                logging::log("KEY", &format!("WindowSwitcher key: '{}'", key_str));

                // P0 FIX: View state only - data comes from this.cached_windows
                if let AppView::WindowSwitcherView {
                    filter,
                    selected_index,
                } = &mut this.current_view
                {
                    // Apply filter to get current filtered list
                    // P0 FIX: Reference cached_windows from self
                    let filtered_windows: Vec<_> = if filter.is_empty() {
                        this.cached_windows.iter().enumerate().collect()
                    } else {
                        let filter_lower = filter.to_lowercase();
                        this.cached_windows
                            .iter()
                            .enumerate()
                            .filter(|(_, w)| {
                                w.title.to_lowercase().contains(&filter_lower)
                                    || w.app.to_lowercase().contains(&filter_lower)
                            })
                            .collect()
                    };
                    let filtered_len = filtered_windows.len();

                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                this.window_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if *selected_index < filtered_len.saturating_sub(1) {
                                *selected_index += 1;
                                this.window_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        "enter" | "return" => {
                            // Focus selected window and hide Script Kit
                            if let Some((_, window_info)) = filtered_windows.get(*selected_index) {
                                logging::log(
                                    "EXEC",
                                    &format!("Focusing window: {}", window_info.title),
                                );
                                if let Err(e) = window_control::focus_window(window_info.id) {
                                    logging::log(
                                        "ERROR",
                                        &format!("Failed to focus window: {}", e),
                                    );
                                    this.toast_manager.push(
                                        components::toast::Toast::error(
                                            format!("Failed to focus window: {}", e),
                                            &this.theme,
                                        )
                                        .duration_ms(Some(5000)),
                                    );
                                    cx.notify();
                                } else {
                                    logging::log(
                                        "EXEC",
                                        &format!("Focused window: {}", window_info.title),
                                    );
                                    // Hide main window only (not entire app) to keep HUD visible
                                    script_kit_gpui::set_main_window_visible(false);
                                    platform::hide_main_window();
                                    NEEDS_RESET.store(true, Ordering::SeqCst);
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

        // Pre-compute colors
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
                    "No windows found"
                } else {
                    "No windows match your filter"
                })
                .into_any_element()
        } else {
            // Clone data for the closure
            let windows_for_closure: Vec<_> = filtered_windows
                .iter()
                .map(|(i, w)| (*i, (*w).clone()))
                .collect();
            let selected = selected_index;

            uniform_list(
                "window-switcher",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_, window_info)) = windows_for_closure.get(ix) {
                                let is_selected = ix == selected;

                                // Format: "AppName: Window Title"
                                let name = format!("{}: {}", window_info.app, window_info.title);

                                // Format bounds as description
                                let description = format!(
                                    "{}×{} at ({}, {})",
                                    window_info.bounds.width,
                                    window_info.bounds.height,
                                    window_info.bounds.x,
                                    window_info.bounds.y
                                );

                                div().id(ix).child(
                                    ListItem::new(name, list_colors)
                                        .description_opt(Some(description))
                                        .selected(is_selected)
                                        .with_accent_bar(true),
                                )
                            } else {
                                div().id(ix).h(px(LIST_ITEM_HEIGHT))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.window_list_scroll_handle)
            .into_any_element()
        };

        // Build actions panel for selected window
        let selected_window = filtered_windows
            .get(selected_index)
            .map(|(_, w)| (*w).clone());
        let actions_panel = self.render_window_actions_panel(
            &selected_window,
            &design_colors,
            &design_spacing,
            &design_typography,
            &design_visual,
            cx,
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
            .key_context("window_switcher")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header with input
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    // Search input - uses shared gpui_input_state for consistent cursor/selection
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
                            .child(format!("{} windows", self.cached_windows.len())),
                    ),
            )
            // Divider
            .child(
                div()
                    .mx(px(design_spacing.padding_lg))
                    .h(px(design_visual.border_thin))
                    .bg(rgba((ui_border << 8) | 0x60)),
            )
            // Main content area - 50/50 split: Window list on left, Actions on right
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h(px(0.))
                    .w_full()
                    .overflow_hidden()
                    // Left side: Window list (50% width)
                    .child(
                        div()
                            .w_1_2()
                            .h_full()
                            .min_h(px(0.))
                            .py(px(design_spacing.padding_xs))
                            .child(list_element),
                    )
                    // Right side: Actions panel (50% width)
                    .child(
                        div()
                            .w_1_2()
                            .h_full()
                            .min_h(px(0.))
                            .overflow_hidden()
                            .child(actions_panel),
                    ),
            )
            // Footer
            .child(PromptFooter::new(
                PromptFooterConfig::new()
                    .primary_label("Switch")
                    .primary_shortcut("↵")
                    .show_secondary(false),
                PromptFooterColors::from_theme(&self.theme),
            ))
            .into_any_element()
    }

    /// Render the actions panel for window switcher
    fn render_window_actions_panel(
        &self,
        selected_window: &Option<window_control::WindowInfo>,
        colors: &designs::DesignColors,
        spacing: &designs::DesignSpacing,
        typography: &designs::DesignTypography,
        visual: &designs::DesignVisual,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let bg_main = colors.background;
        let ui_border = colors.border;
        let text_primary = colors.text_primary;
        let text_muted = colors.text_muted;
        let text_secondary = colors.text_secondary;

        let mut panel = div()
            .w_full()
            .h_full()
            .bg(rgb(bg_main))
            .border_l_1()
            .border_color(rgba((ui_border << 8) | 0x80))
            .p(px(spacing.padding_lg))
            .flex()
            .flex_col()
            .overflow_y_hidden()
            .font_family(typography.font_family);

        match selected_window {
            Some(window) => {
                // Window info header
                panel = panel.child(
                    div()
                        .text_lg()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(text_primary))
                        .pb(px(spacing.padding_sm))
                        .child(window.title.clone()),
                );

                // App name
                panel = panel.child(
                    div()
                        .text_sm()
                        .text_color(rgb(text_secondary))
                        .pb(px(spacing.padding_md))
                        .child(window.app.clone()),
                );

                // Bounds info
                panel = panel.child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .pb(px(spacing.padding_lg))
                        .child(format!(
                            "{}×{} at ({}, {})",
                            window.bounds.width,
                            window.bounds.height,
                            window.bounds.x,
                            window.bounds.y
                        )),
                );

                // Divider
                panel = panel.child(
                    div()
                        .w_full()
                        .h(px(visual.border_thin))
                        .bg(rgba((ui_border << 8) | 0x60))
                        .mb(px(spacing.padding_lg)),
                );

                // Actions header
                panel = panel.child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .pb(px(spacing.padding_md))
                        .child("Press Enter to focus window"),
                );
            }
            None => {
                // Empty state
                panel = panel.child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(text_muted))
                        .child("No window selected"),
                );
            }
        }

        panel
    }

    /// Execute a window action (tile, maximize, minimize, close)
    /// NOTE: Currently unused - kept for future when we add action buttons to the actions panel
    #[allow(dead_code)]
    fn execute_window_action(&mut self, window_id: u32, action: &str, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!("Window action: {} on window {}", action, window_id),
        );

        let result = match action {
            "tile_left" => {
                window_control::tile_window(window_id, window_control::TilePosition::LeftHalf)
            }
            "tile_right" => {
                window_control::tile_window(window_id, window_control::TilePosition::RightHalf)
            }
            "tile_top" => {
                window_control::tile_window(window_id, window_control::TilePosition::TopHalf)
            }
            "tile_bottom" => {
                window_control::tile_window(window_id, window_control::TilePosition::BottomHalf)
            }
            "maximize" => window_control::maximize_window(window_id),
            "minimize" => window_control::minimize_window(window_id),
            "close" => window_control::close_window(window_id),
            "focus" => window_control::focus_window(window_id),
            _ => {
                logging::log("ERROR", &format!("Unknown window action: {}", action));
                return;
            }
        };

        match result {
            Ok(()) => {
                logging::log("EXEC", &format!("Window action {} succeeded", action));

                // Show success toast
                self.toast_manager.push(
                    components::toast::Toast::success(
                        format!("Window {}", action.replace("_", " ")),
                        &self.theme,
                    )
                    .duration_ms(Some(2000)),
                );

                // P0 FIX: Refresh window list in self.cached_windows
                if let AppView::WindowSwitcherView { selected_index, .. } = &mut self.current_view {
                    match window_control::list_windows() {
                        Ok(new_windows) => {
                            self.cached_windows = new_windows;
                            // Adjust selected index if needed
                            if *selected_index >= self.cached_windows.len()
                                && !self.cached_windows.is_empty()
                            {
                                *selected_index = self.cached_windows.len() - 1;
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to refresh windows: {}", e));
                        }
                    }
                }
            }
            Err(e) => {
                logging::log("ERROR", &format!("Window action {} failed: {}", action, e));

                // Show error toast
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to {}: {}", action.replace("_", " "), e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
            }
        }

        cx.notify();
    }

    /// Render design gallery view with group header and icon variations
    fn render_design_gallery(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use designs::group_header_variations::{GroupHeaderCategory, GroupHeaderStyle};
        use designs::icon_variations::{IconCategory, IconName, IconStyle};

        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = self.theme.colors.background.main;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        // Removed: box_shadows - shadows on transparent elements block vibrancy
        let _box_shadows = self.create_box_shadows();

        // Build gallery items: group headers grouped by category, then icons grouped by category
        #[derive(Clone)]
        enum GalleryItem {
            GroupHeaderCategory(GroupHeaderCategory),
            GroupHeader(GroupHeaderStyle),
            IconCategoryHeader(IconCategory),
            Icon(IconName, IconStyle),
        }

        let mut gallery_items: Vec<GalleryItem> = Vec::new();

        // Add group headers by category
        for category in GroupHeaderCategory::all() {
            gallery_items.push(GalleryItem::GroupHeaderCategory(*category));
            for style in category.styles() {
                gallery_items.push(GalleryItem::GroupHeader(*style));
            }
        }

        // Add icons by category, showing each icon with default style
        for category in IconCategory::all() {
            gallery_items.push(GalleryItem::IconCategoryHeader(*category));
            for icon in category.icons() {
                gallery_items.push(GalleryItem::Icon(icon, IconStyle::Default));
            }
        }

        // Filter items based on current filter
        let filtered_items: Vec<(usize, GalleryItem)> = if filter.is_empty() {
            gallery_items
                .iter()
                .enumerate()
                .map(|(i, item)| (i, item.clone()))
                .collect()
        } else {
            let filter_lower = filter.to_lowercase();
            gallery_items
                .iter()
                .enumerate()
                .filter(|(_, item)| match item {
                    GalleryItem::GroupHeaderCategory(cat) => {
                        cat.name().to_lowercase().contains(&filter_lower)
                    }
                    GalleryItem::GroupHeader(style) => {
                        style.name().to_lowercase().contains(&filter_lower)
                            || style.description().to_lowercase().contains(&filter_lower)
                    }
                    GalleryItem::IconCategoryHeader(cat) => {
                        cat.name().to_lowercase().contains(&filter_lower)
                    }
                    GalleryItem::Icon(icon, _) => {
                        icon.name().to_lowercase().contains(&filter_lower)
                            || icon.description().to_lowercase().contains(&filter_lower)
                    }
                })
                .map(|(i, item)| (i, item.clone()))
                .collect()
        };
        let filtered_len = filtered_items.len();

        // Key handler for design gallery
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
                let has_cmd = event.keystroke.modifiers.platform;

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

                logging::log("KEY", &format!("DesignGallery key: '{}'", key_str));

                if let AppView::DesignGalleryView {
                    filter,
                    selected_index,
                } = &mut this.current_view
                {
                    // Re-compute filtered_len for this scope
                    let total_items = GroupHeaderStyle::count()
                        + IconName::count()
                        + GroupHeaderCategory::all().len()
                        + IconCategory::all().len();
                    let current_filtered_len = total_items;

                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                this.design_gallery_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if *selected_index < current_filtered_len.saturating_sub(1) {
                                *selected_index += 1;
                                this.design_gallery_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        // Note: "escape" is handled by handle_global_shortcut_with_options above
                        "backspace" => {
                            if !filter.is_empty() {
                                filter.pop();
                                *selected_index = 0;
                                this.design_gallery_scroll_handle
                                    .scroll_to_item(0, ScrollStrategy::Top);
                                cx.notify();
                            }
                        }
                        _ => {
                            if let Some(ref key_char) = event.keystroke.key_char {
                                if let Some(ch) = key_char.chars().next() {
                                    if !ch.is_control() {
                                        filter.push(ch);
                                        *selected_index = 0;
                                        this.design_gallery_scroll_handle
                                            .scroll_to_item(0, ScrollStrategy::Top);
                                        cx.notify();
                                    }
                                }
                            }
                        }
                    }
                }
            },
        );

        let input_display = if filter.is_empty() {
            SharedString::from("Search design variations...")
        } else {
            SharedString::from(filter.clone())
        };
        let input_is_empty = filter.is_empty();

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
                .child("No items match your filter")
                .into_any_element()
        } else {
            // Clone data for the closure
            let items_for_closure = filtered_items.clone();
            let selected = selected_index;
            let _list_colors_clone = list_colors; // Kept for future use
            let design_spacing_clone = design_spacing;
            let design_typography_clone = design_typography;
            let design_visual_clone = design_visual;
            let design_colors_clone = design_colors;

            uniform_list(
                "design-gallery",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_, item)) = items_for_closure.get(ix) {
                                let is_selected = ix == selected;

                                let element: AnyElement = match item {
                                    GalleryItem::GroupHeaderCategory(category) => {
                                        // Category header - styled as section header
                                        div()
                                            .id(ElementId::NamedInteger(
                                                "gallery-header-cat".into(),
                                                ix as u64,
                                            ))
                                            .w_full()
                                            .h(px(32.0))
                                            .px(px(design_spacing_clone.padding_lg))
                                            .flex()
                                            .items_center()
                                            .bg(rgba(
                                                (design_colors_clone.background_secondary << 8)
                                                    | 0x80,
                                            ))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(rgb(design_colors_clone.accent))
                                                    .child(format!(
                                                        "── Group Headers: {} ──",
                                                        category.name()
                                                    )),
                                            )
                                            .into_any_element()
                                    }
                                    GalleryItem::GroupHeader(style) => render_group_header_item(
                                        ix,
                                        is_selected,
                                        style,
                                        &design_spacing_clone,
                                        &design_typography_clone,
                                        &design_visual_clone,
                                        &design_colors_clone,
                                    ),
                                    GalleryItem::IconCategoryHeader(category) => {
                                        // Icon category header
                                        div()
                                            .id(ElementId::NamedInteger(
                                                "gallery-icon-cat".into(),
                                                ix as u64,
                                            ))
                                            .w_full()
                                            .h(px(32.0))
                                            .px(px(design_spacing_clone.padding_lg))
                                            .flex()
                                            .items_center()
                                            .bg(rgba(
                                                (design_colors_clone.background_secondary << 8)
                                                    | 0x80,
                                            ))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(rgb(design_colors_clone.accent))
                                                    .child(format!(
                                                        "── Icons: {} ──",
                                                        category.name()
                                                    )),
                                            )
                                            .into_any_element()
                                    }
                                    GalleryItem::Icon(icon, _style) => {
                                        // Render icon item with SVG
                                        let icon_path = icon.external_path();
                                        let name_owned = icon.name().to_string();
                                        let desc_owned = icon.description().to_string();

                                        let mut item_div = div()
                                            .id(ElementId::NamedInteger(
                                                "gallery-icon".into(),
                                                ix as u64,
                                            ))
                                            .w_full()
                                            .h(px(LIST_ITEM_HEIGHT))
                                            .px(px(design_spacing_clone.padding_lg))
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(design_spacing_clone.gap_md));

                                        if is_selected {
                                            // Use low-opacity for vibrancy support (see VIBRANCY.md)
                                            item_div = item_div.bg(rgba(
                                                (design_colors_clone.background_selected << 8)
                                                    | 0x0f,
                                            )); // ~6% opacity
                                        }

                                        item_div
                                            // Icon preview with SVG
                                            .child(
                                                div()
                                                    .w(px(32.0))
                                                    .h(px(32.0))
                                                    .rounded(px(4.0))
                                                    .bg(rgba(
                                                        (design_colors_clone.background_secondary
                                                            << 8)
                                                            | 0x60,
                                                    ))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        svg()
                                                            .external_path(icon_path)
                                                            .size(px(16.0))
                                                            .text_color(rgb(
                                                                design_colors_clone.text_primary
                                                            )),
                                                    ),
                                            )
                                            // Name and description
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .flex()
                                                    .flex_col()
                                                    .gap(px(2.0))
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .font_weight(gpui::FontWeight::MEDIUM)
                                                            .text_color(rgb(
                                                                design_colors_clone.text_primary
                                                            ))
                                                            .child(name_owned),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(
                                                                design_colors_clone.text_muted
                                                            ))
                                                            .overflow_x_hidden()
                                                            .child(desc_owned),
                                                    ),
                                            )
                                            .into_any_element()
                                    }
                                };
                                element
                            } else {
                                div()
                                    .id(ElementId::NamedInteger("gallery-empty".into(), ix as u64))
                                    .h(px(LIST_ITEM_HEIGHT))
                                    .into_any_element()
                            }
                        })
                        .collect()
                },
            )
            .w_full()
            .h_full()
            .track_scroll(&self.design_gallery_scroll_handle)
            .into_any_element()
        };

        // Build the full view
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
            .key_context("design_gallery")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header with input
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    // Gallery icon
                    .child(div().text_xl().child("🎨"))
                    // Search input with blinking cursor
                    // ALIGNMENT FIX: Uses canonical cursor constants and negative margin for placeholder
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_lg()
                            .text_color(if input_is_empty {
                                rgb(text_muted)
                            } else {
                                rgb(text_primary)
                            })
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .mr(px(CURSOR_GAP_X))
                                        .when(self.cursor_visible, |d| d.bg(rgb(text_primary))),
                                )
                            })
                            .when(input_is_empty, |d| {
                                d.child(
                                    div()
                                        .ml(px(-(CURSOR_WIDTH + CURSOR_GAP_X)))
                                        .child(input_display.clone()),
                                )
                            })
                            .when(!input_is_empty, |d| d.child(input_display.clone()))
                            .when(!input_is_empty, |d| {
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .ml(px(CURSOR_GAP_X))
                                        .when(self.cursor_visible, |d| d.bg(rgb(text_primary))),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(format!("{} items", filtered_len)),
                    ),
            )
            // Divider
            .child(
                div()
                    .mx(px(design_spacing.padding_lg))
                    .h(px(design_visual.border_thin))
                    .bg(rgba((ui_border << 8) | 0x60)),
            )
            // Main content area - just the list (no preview panel for gallery)
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .h_full()
                    .min_h(px(0.))
                    .overflow_hidden()
                    .py(px(design_spacing.padding_xs))
                    .child(list_element),
            )
            // Footer
            .child(PromptFooter::new(
                PromptFooterConfig::new()
                    .primary_label("Select")
                    .primary_shortcut("↵")
                    .show_secondary(false),
                PromptFooterColors::from_theme(&self.theme),
            ))
            .into_any_element()
    }

    /// Helper: compute filtered preset indices from a filter string
    fn theme_chooser_filtered_indices(filter: &str) -> Vec<usize> {
        let presets = theme::presets::all_presets();
        if filter.is_empty() {
            (0..presets.len()).collect()
        } else {
            let f = filter.to_lowercase();
            presets
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    p.name.to_lowercase().contains(&f)
                        || p.description.to_lowercase().contains(&f)
                })
                .map(|(i, _)| i)
                .collect()
        }
    }

    /// Accent color palette for theme customization
    const ACCENT_PALETTE: &'static [(u32, &'static str)] = &[
        (0xFBBF24, "Amber"),
        (0x3B82F6, "Blue"),
        (0x8B5CF6, "Violet"),
        (0xEC4899, "Pink"),
        (0xEF4444, "Red"),
        (0xF97316, "Orange"),
        (0x22C55E, "Green"),
        (0x14B8A6, "Teal"),
        (0x06B6D4, "Cyan"),
        (0x6366F1, "Indigo"),
    ];

    /// Opacity presets for quick selection
    const OPACITY_PRESETS: &'static [(f32, &'static str)] = &[
        (0.10, "10%"),
        (0.30, "30%"),
        (0.50, "50%"),
        (0.80, "80%"),
        (1.00, "100%"),
    ];

    /// Compute on-accent text color based on accent luminance
    fn accent_on_text_color(accent: u32, bg_main: u32) -> u32 {
        let r = ((accent >> 16) & 0xFF) as f32;
        let g = ((accent >> 8) & 0xFF) as f32;
        let b = (accent & 0xFF) as f32;
        if (0.299 * r + 0.587 * g + 0.114 * b) > 128.0 {
            bg_main
        } else {
            0xFFFFFF
        }
    }

    /// Find the closest accent palette index for a given accent color
    fn find_accent_palette_index(accent: u32) -> Option<usize> {
        Self::ACCENT_PALETTE.iter().position(|&(c, _)| c == accent)
    }

    /// Find the closest opacity preset index for a given opacity value
    fn find_opacity_preset_index(opacity: f32) -> usize {
        Self::OPACITY_PRESETS
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.0 - opacity)
                    .abs()
                    .partial_cmp(&(b.0 - opacity).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Render the theme chooser with search, live preview, and preview panel
    pub(crate) fn render_theme_chooser(
        &mut self,
        filter: &str,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();
        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let text_secondary = self.theme.colors.text.secondary;
        let text_muted = self.theme.colors.text.muted;
        let accent_color = self.theme.colors.accent.selected;
        let ui_border = self.theme.colors.ui.border;
        let selection_bg = self.theme.colors.accent.selected_subtle;
        let bg_main = self.theme.colors.background.main;
        let bg_search_box = self.theme.colors.background.search_box;
        let text_on_accent = self.theme.colors.text.on_accent;
        let ui_success = self.theme.colors.ui.success;
        let ui_error = self.theme.colors.ui.error;
        let ui_warning = self.theme.colors.ui.warning;
        let ui_info = self.theme.colors.ui.info;
        let opacity = self.theme.get_opacity();
        let selected_alpha = (opacity.selected * 255.0) as u32;
        let hover_alpha = (opacity.hover * 255.0).max(18.0) as u32;
        let presets = theme::presets::all_presets();
        let preview_colors = theme::presets::all_preset_preview_colors();
        let first_light = theme::presets::first_light_theme_index();
        let original_index = self
            .theme_before_chooser
            .as_ref()
            .map(|t| theme::presets::find_current_preset_index(t))
            .unwrap_or(0);

        // Filter presets by name or description
        let filtered_indices = Self::theme_chooser_filtered_indices(filter);
        let filtered_count = filtered_indices.len();
        let filter_is_empty = filter.is_empty();

        // Count dark/light in filtered results
        let dark_count = filtered_indices
            .iter()
            .filter(|&&i| presets[i].is_dark)
            .count();
        let light_count = filtered_count - dark_count;

        // Terminal colors for preview panel
        let terminal = &self.theme.colors.terminal;
        let term_colors: Vec<u32> = vec![
            terminal.red,
            terminal.green,
            terminal.yellow,
            terminal.blue,
            terminal.magenta,
            terminal.cyan,
            terminal.white,
            terminal.black,
        ];
        let term_bright: Vec<u32> = vec![
            terminal.bright_red,
            terminal.bright_green,
            terminal.bright_yellow,
            terminal.bright_blue,
            terminal.bright_magenta,
            terminal.bright_cyan,
            terminal.bright_white,
            terminal.bright_black,
        ];

        let theme_item_height: f32 = 48.0;
        let entity_handle = cx.entity().downgrade();

        // ── Keyboard handler ───────────────────────────────────────
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                this.hide_mouse_cursor(cx);
                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // Escape: clear filter first if present, otherwise restore original and close
                if key_str == "escape" {
                    if !this.clear_builtin_view_filter(cx) {
                        // No filter to clear — restore original theme and go back
                        if let Some(original) = this.theme_before_chooser.take() {
                            this.theme = original;
                            theme::sync_gpui_component_theme(cx);
                        }
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }
                // Cmd+W: restore and close window
                if has_cmd && key_str == "w" {
                    if let Some(original) = this.theme_before_chooser.take() {
                        this.theme = original;
                        theme::sync_gpui_component_theme(cx);
                    }
                    this.close_and_reset_window(cx);
                    return;
                }
                // Cmd+[ / Cmd+]: cycle accent colors
                if has_cmd && (key_str == "[" || key_str == "bracketleft") {
                    let current = this.theme.colors.accent.selected;
                    let idx = Self::find_accent_palette_index(current).unwrap_or(0);
                    let new_idx = if idx == 0 {
                        Self::ACCENT_PALETTE.len() - 1
                    } else {
                        idx - 1
                    };
                    let (new_accent, _) = Self::ACCENT_PALETTE[new_idx];
                    let mut modified = (*this.theme).clone();
                    modified.colors.accent.selected = new_accent;
                    modified.colors.text.on_accent =
                        Self::accent_on_text_color(new_accent, modified.colors.background.main);
                    this.theme = std::sync::Arc::new(modified);
                    theme::sync_gpui_component_theme(cx);
                    cx.notify();
                    return;
                }
                if has_cmd && (key_str == "]" || key_str == "bracketright") {
                    let current = this.theme.colors.accent.selected;
                    let idx = Self::find_accent_palette_index(current).unwrap_or(0);
                    let new_idx = (idx + 1) % Self::ACCENT_PALETTE.len();
                    let (new_accent, _) = Self::ACCENT_PALETTE[new_idx];
                    let mut modified = (*this.theme).clone();
                    modified.colors.accent.selected = new_accent;
                    modified.colors.text.on_accent =
                        Self::accent_on_text_color(new_accent, modified.colors.background.main);
                    this.theme = std::sync::Arc::new(modified);
                    theme::sync_gpui_component_theme(cx);
                    cx.notify();
                    return;
                }
                // Cmd+- / Cmd+=: adjust opacity
                if has_cmd && key_str == "-" {
                    let current_main = this.theme.get_opacity().main;
                    let idx = Self::find_opacity_preset_index(current_main);
                    if idx > 0 {
                        let target = Self::OPACITY_PRESETS[idx - 1].0;
                        let mut modified = (*this.theme).clone();
                        if let Some(ref mut op) = modified.opacity {
                            op.main = target;
                            op.title_bar = target;
                        }
                        this.theme = std::sync::Arc::new(modified);
                        theme::sync_gpui_component_theme(cx);
                        cx.notify();
                    }
                    return;
                }
                if has_cmd && (key_str == "=" || key_str == "+") {
                    let current_main = this.theme.get_opacity().main;
                    let idx = Self::find_opacity_preset_index(current_main);
                    if idx < Self::OPACITY_PRESETS.len() - 1 {
                        let target = Self::OPACITY_PRESETS[idx + 1].0;
                        let mut modified = (*this.theme).clone();
                        if let Some(ref mut op) = modified.opacity {
                            op.main = target;
                            op.title_bar = target;
                        }
                        this.theme = std::sync::Arc::new(modified);
                        theme::sync_gpui_component_theme(cx);
                        cx.notify();
                    }
                    return;
                }
                // Cmd+B: toggle vibrancy
                if has_cmd && key_str == "b" {
                    let mut modified = (*this.theme).clone();
                    if let Some(ref mut vibrancy) = modified.vibrancy {
                        vibrancy.enabled = !vibrancy.enabled;
                    }
                    this.theme = std::sync::Arc::new(modified);
                    theme::sync_gpui_component_theme(cx);
                    cx.notify();
                    return;
                }
                // Cmd+M: cycle vibrancy material
                if has_cmd && key_str == "m" {
                    let current_material = this
                        .theme
                        .vibrancy
                        .as_ref()
                        .map(|v| v.material)
                        .unwrap_or_default();
                    let idx = Self::find_vibrancy_material_index(current_material);
                    let new_idx = (idx + 1) % Self::VIBRANCY_MATERIALS.len();
                    let (new_material, _) = Self::VIBRANCY_MATERIALS[new_idx];
                    let mut modified = (*this.theme).clone();
                    if let Some(ref mut vibrancy) = modified.vibrancy {
                        vibrancy.material = new_material;
                    }
                    this.theme = std::sync::Arc::new(modified);
                    theme::sync_gpui_component_theme(cx);
                    cx.notify();
                    return;
                }
                // Cmd+R: reset customizations to selected preset defaults
                if has_cmd && key_str == "r" {
                    let current_filter =
                        if let AppView::ThemeChooserView { ref filter, .. } = this.current_view {
                            filter.clone()
                        } else {
                            return;
                        };
                    let presets = theme::presets::all_presets();
                    let filtered = Self::theme_chooser_filtered_indices(&current_filter);
                    if let AppView::ThemeChooserView {
                        ref selected_index, ..
                    } = this.current_view
                    {
                        if let Some(&pidx) = filtered.get(*selected_index) {
                            if pidx < presets.len() {
                                this.theme =
                                    std::sync::Arc::new(presets[pidx].create_theme());
                                theme::sync_gpui_component_theme(cx);
                                cx.notify();
                            }
                        }
                    }
                    return;
                }
                // Enter: apply and close
                if key_str == "enter" {
                    this.theme_before_chooser = None;
                    if let Err(e) = theme::presets::write_theme_to_disk(&this.theme) {
                        logging::log("ERROR", &format!("Failed to save theme: {}", e));
                    }
                    theme::sync_gpui_component_theme(cx);
                    this.go_back_or_close(window, cx);
                    return;
                }

                // Compute filtered indices from current filter
                let current_filter =
                    if let AppView::ThemeChooserView { ref filter, .. } = this.current_view {
                        filter.clone()
                    } else {
                        return;
                    };
                let presets = theme::presets::all_presets();
                let filtered = Self::theme_chooser_filtered_indices(&current_filter);
                let count = filtered.len();
                if count == 0 {
                    return;
                }

                if let AppView::ThemeChooserView {
                    ref mut selected_index,
                    ..
                } = this.current_view
                {
                    let page_size: usize = 5;
                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            *selected_index = if *selected_index == 0 {
                                count - 1
                            } else {
                                *selected_index - 1
                            };
                        }
                        "down" | "arrowdown" => {
                            *selected_index = (*selected_index + 1) % count;
                        }
                        "home" => {
                            *selected_index = 0;
                        }
                        "end" => {
                            *selected_index = count - 1;
                        }
                        "pageup" => {
                            *selected_index = selected_index.saturating_sub(page_size);
                        }
                        "pagedown" => {
                            *selected_index = (*selected_index + page_size).min(count - 1);
                        }
                        _ => return,
                    }
                    // Map to actual preset index and apply theme
                    let preset_idx = filtered[*selected_index];
                    let new_theme = std::sync::Arc::new(presets[preset_idx].create_theme());
                    this.theme = new_theme;
                    this.theme_chooser_scroll_handle
                        .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                    theme::sync_gpui_component_theme(cx);
                    cx.notify();
                }
            },
        );

        // ── Pre-compute data for list closure ──────────────────────
        let preset_names: Vec<String> = presets.iter().map(|p| p.name.to_string()).collect();
        let preset_descs: Vec<String> = presets.iter().map(|p| p.description.to_string()).collect();
        let preset_is_dark: Vec<bool> = presets.iter().map(|p| p.is_dark).collect();
        let selected = selected_index;
        let orig_idx = original_index;
        let first_light_idx = first_light;
        let hover_bg = rgba((selection_bg << 8) | hover_alpha);
        let filtered_indices_for_list = filtered_indices.clone();
        let entity_handle_for_customize = entity_handle.clone();

        // ── Theme list ─────────────────────────────────────────────
        let list = uniform_list(
            "theme-chooser",
            filtered_count,
            move |visible_range, _window, _cx| {
                visible_range
                    .map(|ix| {
                        let preset_idx = filtered_indices_for_list[ix];
                        let is_selected = ix == selected;
                        let is_original = preset_idx == orig_idx;
                        let name = &preset_names[preset_idx];
                        let desc = &preset_descs[preset_idx];
                        let is_dark = preset_is_dark[preset_idx];
                        let colors = &preview_colors[preset_idx];
                        let is_first_light = filter_is_empty
                            && preset_idx == first_light_idx
                            && first_light_idx > 0;

                        // Color swatches
                        let swatch = |color: u32| {
                            div()
                                .w(px(14.0))
                                .h(px(24.0))
                                .rounded(px(3.0))
                                .bg(rgb(color))
                        };
                        let palette = div()
                            .flex()
                            .flex_row()
                            .gap(px(2.0))
                            .mr(px(10.0))
                            .child(swatch(colors.bg))
                            .child(swatch(colors.accent))
                            .child(swatch(colors.text))
                            .child(swatch(colors.secondary))
                            .child(swatch(colors.border));

                        // Checkmark for original (saved) theme
                        let indicator = if is_original {
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(rgb(accent_color))
                                .w(px(16.0))
                                .child("✓")
                        } else {
                            div().w(px(16.0))
                        };

                        // Dark/light badge
                        let badge_text = if is_dark { "dark" } else { "light" };
                        let badge_border = rgba((ui_border << 8) | 0x40);
                        let badge = div()
                            .text_xs()
                            .text_color(rgb(text_dimmed))
                            .ml_auto()
                            .px(px(6.0))
                            .py(px(2.0))
                            .rounded(px(4.0))
                            .border_1()
                            .border_color(badge_border)
                            .child(badge_text.to_string());

                        let sel_bg = rgba((selection_bg << 8) | selected_alpha);
                        let border_rgba = rgba((ui_border << 8) | 0x30);

                        // Section label for light themes (only when unfiltered)
                        let section_label = if is_first_light {
                            Some(
                                div()
                                    .w_full()
                                    .pt(px(8.0))
                                    .pb(px(4.0))
                                    .px(px(16.0))
                                    .border_color(border_rgba)
                                    .border_t_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_dimmed))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .child("LIGHT"),
                                    ),
                            )
                        } else {
                            None
                        };

                        let name_color = if is_selected {
                            accent_color
                        } else {
                            text_primary
                        };

                        // Click handler: select + preview via filtered index
                        let click_entity = entity_handle.clone();
                        let click_handler = move |_event: &gpui::ClickEvent,
                                                   _window: &mut Window,
                                                   cx: &mut gpui::App| {
                            if let Some(app) = click_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    // Recompute filtered indices from current filter
                                    let current_filter = if let AppView::ThemeChooserView {
                                        ref filter,
                                        ..
                                    } = this.current_view
                                    {
                                        filter.clone()
                                    } else {
                                        return;
                                    };
                                    let presets = theme::presets::all_presets();
                                    let filtered =
                                        Self::theme_chooser_filtered_indices(&current_filter);

                                    if let AppView::ThemeChooserView {
                                        ref mut selected_index,
                                        ..
                                    } = this.current_view
                                    {
                                        *selected_index = ix;
                                    }
                                    if let Some(&pidx) = filtered.get(ix) {
                                        if pidx < presets.len() {
                                            this.theme = std::sync::Arc::new(
                                                presets[pidx].create_theme(),
                                            );
                                            theme::sync_gpui_component_theme(cx);
                                            cx.notify();
                                        }
                                    }
                                });
                            }
                        };

                        // Build item row
                        let row = div()
                            .id(ix)
                            .w_full()
                            .h(px(theme_item_height))
                            .px(px(12.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .cursor_pointer()
                            .when(is_selected, |d| {
                                d.bg(sel_bg).border_l_2().border_color(rgb(accent_color))
                            })
                            .when(!is_selected, |d| d.hover(move |s| s.bg(hover_bg)))
                            .on_click(click_handler)
                            .child(indicator)
                            .child(palette)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .overflow_hidden()
                                    .gap(px(1.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .when(is_original || is_selected, |d| {
                                                d.font_weight(gpui::FontWeight::SEMIBOLD)
                                            })
                                            .text_color(rgb(name_color))
                                            .child(name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_secondary))
                                            .child(desc.clone()),
                                    ),
                            )
                            .child(badge);

                        if let Some(label) = section_label {
                            div()
                                .w_full()
                                .flex()
                                .flex_col()
                                .child(label)
                                .child(row)
                                .into_any_element()
                        } else {
                            row.into_any_element()
                        }
                    })
                    .collect()
            },
        )
        .h_full()
        .track_scroll(&self.theme_chooser_scroll_handle)
        .into_any_element();

        // ── Header with search input ───────────────────────────────
        let header = div()
            .w_full()
            .px(px(design_spacing.padding_lg))
            .pt(px(design_spacing.padding_md))
            .pb(px(4.0))
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(text_primary))
                            .child("Themes"),
                    )
                    .child(
                        div().text_xs().text_color(rgb(text_dimmed)).child(format!(
                            "{} dark · {} light",
                            dark_count, light_count
                        )),
                    ),
            )
            // Search input
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
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
                    ),
            )
            // "DARK" section label only when unfiltered
            .when(filter_is_empty, |d| {
                d.child(
                    div()
                        .w_full()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_dimmed))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child("DARK"),
                        ),
                )
            });

        // ── Preview panel with customization controls ─────────────
        let border_rgba = rgba((ui_border << 8) | 0x40);
        let current_opacity_main = opacity.main;
        let vibrancy_enabled = self
            .theme
            .vibrancy
            .as_ref()
            .map(|v| v.enabled)
            .unwrap_or(true);

        // Build accent color swatches (clickable)
        let accent_swatches: Vec<gpui::AnyElement> = Self::ACCENT_PALETTE
            .iter()
            .enumerate()
            .map(|(i, &(color, _name))| {
                let is_current = color == accent_color;
                let click_entity = entity_handle_for_customize.clone();
                let swatch_bg_main = bg_main;
                div()
                    .id(ElementId::NamedInteger("accent-swatch".into(), i as u64))
                    .w(px(20.0))
                    .h(px(20.0))
                    .rounded(px(10.0))
                    .bg(rgb(color))
                    .cursor_pointer()
                    .when(is_current, |d| d.border_2().border_color(rgb(text_primary)))
                    .when(!is_current, |d| {
                        d.border_1()
                            .border_color(border_rgba)
                            .hover(move |s| s.border_color(rgb(text_secondary)))
                    })
                    .on_click(
                        move |_event: &gpui::ClickEvent,
                              _window: &mut Window,
                              cx: &mut gpui::App| {
                            if let Some(app) = click_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    let mut modified = (*this.theme).clone();
                                    modified.colors.accent.selected = color;
                                    modified.colors.text.on_accent =
                                        Self::accent_on_text_color(color, swatch_bg_main);
                                    this.theme = std::sync::Arc::new(modified);
                                    theme::sync_gpui_component_theme(cx);
                                    cx.notify();
                                });
                            }
                        },
                    )
                    .into_any_element()
            })
            .collect();

        // Build opacity preset buttons (clickable)
        let opacity_buttons: Vec<gpui::AnyElement> = Self::OPACITY_PRESETS
            .iter()
            .enumerate()
            .map(|(i, &(value, label))| {
                let is_current = (value - current_opacity_main).abs() < 0.05;
                let click_entity = entity_handle_for_customize.clone();
                div()
                    .id(ElementId::NamedInteger("opacity-btn".into(), i as u64))
                    .px(px(8.0))
                    .py(px(3.0))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .text_xs()
                    .when(is_current, |d| {
                        d.bg(rgb(accent_color))
                            .text_color(rgb(text_on_accent))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                    })
                    .when(!is_current, |d| {
                        d.border_1()
                            .border_color(border_rgba)
                            .text_color(rgb(text_secondary))
                            .hover(move |s| s.bg(rgba((selection_bg << 8) | hover_alpha)))
                    })
                    .on_click(
                        move |_event: &gpui::ClickEvent,
                              _window: &mut Window,
                              cx: &mut gpui::App| {
                            if let Some(app) = click_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    let mut modified = (*this.theme).clone();
                                    if let Some(ref mut op) = modified.opacity {
                                        op.main = value;
                                        op.title_bar = value;
                                    }
                                    this.theme = std::sync::Arc::new(modified);
                                    theme::sync_gpui_component_theme(cx);
                                    cx.notify();
                                });
                            }
                        },
                    )
                    .child(label.to_string())
                    .into_any_element()
            })
            .collect();

        // Build vibrancy toggle (clickable)
        let vibrancy_entity = entity_handle_for_customize.clone();
        let vibrancy_toggle = div()
            .id("vibrancy-toggle")
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            .cursor_pointer()
            .hover(move |s| s.bg(rgba((selection_bg << 8) | hover_alpha)))
            .rounded(px(4.0))
            .px(px(4.0))
            .py(px(2.0))
            .on_click(
                move |_event: &gpui::ClickEvent,
                      _window: &mut Window,
                      cx: &mut gpui::App| {
                    if let Some(app) = vibrancy_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            let mut modified = (*this.theme).clone();
                            if let Some(ref mut vibrancy) = modified.vibrancy {
                                vibrancy.enabled = !vibrancy.enabled;
                            }
                            this.theme = std::sync::Arc::new(modified);
                            theme::sync_gpui_component_theme(cx);
                            cx.notify();
                        });
                    }
                },
            )
            .child(
                div()
                    .w(px(28.0))
                    .h(px(14.0))
                    .rounded(px(7.0))
                    .when(vibrancy_enabled, |d| d.bg(rgb(accent_color)))
                    .when(!vibrancy_enabled, |d| {
                        d.bg(rgba((ui_border << 8) | 0x80))
                    })
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .w(px(10.0))
                            .h(px(10.0))
                            .rounded(px(5.0))
                            .bg(rgb(0xffffff))
                            .when(vibrancy_enabled, |d| d.ml(px(16.0)))
                            .when(!vibrancy_enabled, |d| d.ml(px(2.0))),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(text_secondary))
                    .child(if vibrancy_enabled { "On" } else { "Off" }),
            );

        // Build vibrancy material buttons (clickable, only shown when vibrancy enabled)
        let current_material = self
            .theme
            .vibrancy
            .as_ref()
            .map(|v| v.material)
            .unwrap_or_default();
        let material_buttons: Vec<gpui::AnyElement> = Self::VIBRANCY_MATERIALS
            .iter()
            .enumerate()
            .map(|(i, &(material, label))| {
                let is_current = material == current_material;
                let click_entity = entity_handle_for_customize.clone();
                div()
                    .id(ElementId::NamedInteger("material-btn".into(), i as u64))
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .text_xs()
                    .when(is_current, |d| {
                        d.bg(rgb(accent_color))
                            .text_color(rgb(text_on_accent))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                    })
                    .when(!is_current, |d| {
                        d.border_1()
                            .border_color(border_rgba)
                            .text_color(rgb(text_secondary))
                            .hover(move |s| s.bg(rgba((selection_bg << 8) | hover_alpha)))
                    })
                    .on_click(
                        move |_event: &gpui::ClickEvent,
                              _window: &mut Window,
                              cx: &mut gpui::App| {
                            if let Some(app) = click_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    let mut modified = (*this.theme).clone();
                                    if let Some(ref mut vibrancy) = modified.vibrancy {
                                        vibrancy.material = material;
                                    }
                                    this.theme = std::sync::Arc::new(modified);
                                    theme::sync_gpui_component_theme(cx);
                                    cx.notify();
                                });
                            }
                        },
                    )
                    .child(label.to_string())
                    .into_any_element()
            })
            .collect();

        // Build font size preset buttons (clickable)
        let current_ui_font_size = self.theme.get_fonts().ui_size;
        let font_size_buttons: Vec<gpui::AnyElement> = Self::FONT_SIZE_PRESETS
            .iter()
            .enumerate()
            .map(|(i, &(size, label))| {
                let is_current = (size - current_ui_font_size).abs() < 0.5;
                let click_entity = entity_handle_for_customize.clone();
                div()
                    .id(ElementId::NamedInteger("fontsize-btn".into(), i as u64))
                    .px(px(8.0))
                    .py(px(2.0))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .text_xs()
                    .when(is_current, |d| {
                        d.bg(rgb(accent_color))
                            .text_color(rgb(text_on_accent))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                    })
                    .when(!is_current, |d| {
                        d.border_1()
                            .border_color(border_rgba)
                            .text_color(rgb(text_secondary))
                            .hover(move |s| s.bg(rgba((selection_bg << 8) | hover_alpha)))
                    })
                    .on_click(
                        move |_event: &gpui::ClickEvent,
                              _window: &mut Window,
                              cx: &mut gpui::App| {
                            if let Some(app) = click_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    let mut modified = (*this.theme).clone();
                                    if let Some(ref mut fonts) = modified.fonts {
                                        fonts.ui_size = size;
                                    } else {
                                        modified.fonts = Some(theme::FontConfig {
                                            ui_size: size,
                                            ..Default::default()
                                        });
                                    }
                                    this.theme = std::sync::Arc::new(modified);
                                    theme::sync_gpui_component_theme(cx);
                                    cx.notify();
                                });
                            }
                        },
                    )
                    .child(label.to_string())
                    .into_any_element()
            })
            .collect();

        // Build reset button (clickable)
        let reset_entity = entity_handle_for_customize.clone();
        let reset_button = div()
            .id("reset-to-preset")
            .px(px(10.0))
            .py(px(4.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .text_xs()
            .border_1()
            .border_color(border_rgba)
            .text_color(rgb(text_secondary))
            .hover(move |s| s.bg(rgba((selection_bg << 8) | hover_alpha)))
            .on_click(
                move |_event: &gpui::ClickEvent,
                      _window: &mut Window,
                      cx: &mut gpui::App| {
                    if let Some(app) = reset_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            let current_filter =
                                if let AppView::ThemeChooserView { ref filter, .. } =
                                    this.current_view
                                {
                                    filter.clone()
                                } else {
                                    return;
                                };
                            let presets = theme::presets::all_presets();
                            let filtered =
                                Self::theme_chooser_filtered_indices(&current_filter);
                            if let AppView::ThemeChooserView {
                                ref selected_index, ..
                            } = this.current_view
                            {
                                if let Some(&pidx) = filtered.get(*selected_index) {
                                    if pidx < presets.len() {
                                        this.theme =
                                            std::sync::Arc::new(presets[pidx].create_theme());
                                        theme::sync_gpui_component_theme(cx);
                                        cx.notify();
                                    }
                                }
                            }
                        });
                    }
                },
            )
            .child("Reset to Defaults");

        let accent_name = Self::accent_color_name(accent_color);

        let preview_panel = div()
            .w_1_2()
            .h_full()
            .border_l_1()
            .border_color(border_rgba)
            .px(px(design_spacing.padding_lg))
            .py(px(design_spacing.padding_md))
            .flex()
            .flex_col()
            .gap(px(10.0))
            .overflow_y_hidden()
            // ── Customize section ──────────────────────────────────
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(text_dimmed))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("CUSTOMIZE"),
            )
            // Accent color row (with name)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_muted))
                                    .child("Accent Color"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(accent_color))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child(accent_name.to_string()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(4.0))
                            .flex_wrap()
                            .children(accent_swatches),
                    ),
            )
            // Opacity row (10 steps)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_muted))
                            .child(format!(
                                "Window Opacity  {:.0}%",
                                current_opacity_main * 100.0
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(2.0))
                            .flex_wrap()
                            .children(opacity_buttons),
                    ),
            )
            // Vibrancy toggle + material row
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_muted))
                            .child("Vibrancy Blur"),
                    )
                    .child(vibrancy_toggle)
                    .when(vibrancy_enabled, |d| {
                        d.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .mt(px(4.0))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .child("Material"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap(px(3.0))
                                        .flex_wrap()
                                        .children(material_buttons),
                                ),
                        )
                    }),
            )
            // Font size row
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_muted))
                            .child(format!("UI Font Size  {:.0}px", current_ui_font_size)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(4.0))
                            .children(font_size_buttons),
                    ),
            )
            // Reset button
            .child(reset_button)
            // ── Preview section ────────────────────────────────────
            .child(
                div()
                    .w_full()
                    .mt(px(4.0))
                    .pt(px(8.0))
                    .border_t_1()
                    .border_color(border_rgba)
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_dimmed))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("PREVIEW"),
                    ),
            )
            // Mock search box
            .child(
                div()
                    .w_full()
                    .h(px(28.0))
                    .rounded(px(6.0))
                    .bg(rgb(bg_search_box))
                    .border_1()
                    .border_color(border_rgba)
                    .px(px(10.0))
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_muted))
                            .child("Search scripts..."),
                    ),
            )
            // Mock list items
            .child(
                div()
                    .w_full()
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(border_rgba)
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .w_full()
                            .h(px(28.0))
                            .bg(rgb(accent_color))
                            .px(px(10.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(text_on_accent))
                                    .child("Selected Item"),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .h(px(28.0))
                            .bg(rgb(bg_main))
                            .px(px(10.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_primary))
                                    .child("Regular Item"),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .h(px(28.0))
                            .bg(rgb(bg_main))
                            .px(px(10.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_secondary))
                                    .child("Another Item"),
                            ),
                    ),
            )
            // Terminal + semantic colors
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_dimmed))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("TERMINAL"),
                    )
                    .child(
                        div().flex().flex_row().gap(px(2.0)).children(
                            term_colors
                                .iter()
                                .map(|&c| div().w(px(16.0)).h(px(12.0)).rounded(px(2.0)).bg(rgb(c))),
                        ),
                    )
                    .child(
                        div().flex().flex_row().gap(px(2.0)).children(
                            term_bright
                                .iter()
                                .map(|&c| div().w(px(16.0)).h(px(12.0)).rounded(px(2.0)).bg(rgb(c))),
                        ),
                    ),
            )
            // Semantic colors
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.0))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(3.0))
                            .child(div().w(px(7.0)).h(px(7.0)).rounded(px(4.0)).bg(rgb(ui_success)))
                            .child(div().text_xs().text_color(rgb(ui_success)).child("OK")),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(3.0))
                            .child(div().w(px(7.0)).h(px(7.0)).rounded(px(4.0)).bg(rgb(ui_error)))
                            .child(div().text_xs().text_color(rgb(ui_error)).child("Err")),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(3.0))
                            .child(div().w(px(7.0)).h(px(7.0)).rounded(px(4.0)).bg(rgb(ui_warning)))
                            .child(div().text_xs().text_color(rgb(ui_warning)).child("Warn")),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(3.0))
                            .child(div().w(px(7.0)).h(px(7.0)).rounded(px(4.0)).bg(rgb(ui_info)))
                            .child(div().text_xs().text_color(rgb(ui_info)).child("Info")),
                    ),
            );

        // ── Footer with keyboard shortcuts ─────────────────────────
        let shortcut = |key: &str, label: &str| {
            div()
                .flex()
                .flex_row()
                .gap(px(4.0))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_secondary))
                        .child(key.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_dimmed))
                        .child(label.to_string()),
                )
        };
        let footer_border = rgba((ui_border << 8) | 0x30);
        let footer = div()
            .w_full()
            .px(px(design_spacing.padding_lg))
            .py(px(design_spacing.padding_sm))
            .border_t_1()
            .border_color(footer_border)
            .flex()
            .flex_col()
            .gap(px(2.0))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_center()
                    .gap(px(12.0))
                    .child(shortcut("↑↓", "Preview"))
                    .child(shortcut("Enter", "Apply"))
                    .child(shortcut("Esc", "Cancel"))
                    .child(shortcut("PgUp/Dn", "Jump"))
                    .child(shortcut("Type", "Search")),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_center()
                    .gap(px(12.0))
                    .child(shortcut("⌘[]", "Accent"))
                    .child(shortcut("⌘-/=", "Opacity"))
                    .child(shortcut("⌘B", "Vibrancy"))
                    .child(shortcut("⌘M", "Material"))
                    .child(shortcut("⌘R", "Reset")),
            );

        // ── Empty state when filter has no matches ─────────────────
        if filtered_count == 0 {
            return div()
                .flex()
                .flex_col()
                .w_full()
                .h_full()
                .rounded(px(design_visual.radius_lg))
                .text_color(rgb(text_primary))
                .font_family(design_typography.font_family)
                .key_context("theme_chooser")
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key)
                .child(header)
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_muted))
                                .child("No matching themes"),
                        ),
                )
                .child(footer)
                .into_any_element();
        }

        // ── Main layout: list + preview panel ──────────────────────
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("theme_chooser")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(header)
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .flex()
                    .flex_row()
                    .child(div().w_1_2().h_full().child(list))
                    .child(preview_panel),
            )
            .child(footer)
            .into_any_element()
    }

    /// Render file search view with 50/50 split (list + preview)
    pub(crate) fn render_file_search(
        &mut self,
        query: &str,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use crate::file_search::{self, FileType};

        // Use design tokens for spacing/visual, theme for colors
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let _design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let _opacity = self.theme.get_opacity();
        // bg_with_alpha removed - let vibrancy show through from Root (matches main menu)
        // Removed: box_shadows - shadows on transparent elements block vibrancy
        let _box_shadows = self.create_box_shadows();

        // Color values for use in closures
        let text_primary = self.theme.colors.text.primary;
        let text_muted = self.theme.colors.text.muted;
        let text_dimmed = self.theme.colors.text.dimmed;
        let ui_border = self.theme.colors.ui.border;
        let _accent_color = self.theme.colors.accent.selected;
        let list_hover = self.theme.colors.accent.selected_subtle;
        let list_selected = self.theme.colors.accent.selected_subtle;
        // Use theme opacity for vibrancy-compatible selection/hover (matches main menu)
        let opacity = self.theme.get_opacity();
        let selected_alpha = (opacity.selected * 255.0) as u32;
        let hover_alpha = (opacity.hover * 255.0) as u32;

        // Use pre-computed display indices instead of running Nucleo in render
        // This is CRITICAL for animation performance - render must be cheap
        // The display_indices are computed in recompute_file_search_display_indices()
        // which is called when:
        // 1. Results change (new directory listing or search results)
        // 2. Filter pattern changes (user types in existing directory)
        // 3. Loading completes
        let display_indices = &self.file_search_display_indices;
        let filtered_len = display_indices.len();

        // Log render state (throttled - only when state changes meaningfully)
        static LAST_LOG: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let last = LAST_LOG.load(std::sync::atomic::Ordering::Relaxed);
        if now_ms.saturating_sub(last) > 500 {
            // Log at most every 500ms
            LAST_LOG.store(now_ms, std::sync::atomic::Ordering::Relaxed);
            logging::log(
                "SEARCH",
                &format!(
                    "render_file_search: query='{}' loading={} cached={} display={} selected={}",
                    query,
                    self.file_search_loading,
                    self.cached_file_results.len(),
                    filtered_len,
                    selected_index
                ),
            );
        }

        // Get selected file for preview (if any)
        // Use display_indices to map visible index -> actual result index
        let selected_file = display_indices
            .get(selected_index)
            .and_then(|&result_idx| self.cached_file_results.get(result_idx))
            .cloned();

        // Key handler for file search
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
                    ActionsDialogHost::FileSearch,
                    window,
                    cx,
                ) {
                    ActionsRoute::NotHandled => {
                        // Actions dialog not open - continue to file search key handling
                    }
                    ActionsRoute::Handled => {
                        // Key was consumed by actions dialog
                        return;
                    }
                    ActionsRoute::Execute { action_id } => {
                        // User selected an action - execute it
                        // Use handle_action instead of trigger_action_by_name to support
                        // both built-in actions (open_file, quick_look, etc.) and SDK actions
                        this.handle_action(action_id, cx);
                        return;
                    }
                }

                // ESC: Clear query first if present, otherwise go back/close
                if key_str == "escape" {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }

                // Cmd+W closes window
                if has_cmd && key_str == "w" {
                    logging::log("KEY", "Cmd+W - closing window");
                    this.close_and_reset_window(cx);
                    return;
                }

                if let AppView::FileSearchView {
                    query: _,
                    selected_index,
                } = &mut this.current_view
                {
                    // Use pre-computed display_indices to get the selected file
                    // This is consistent with what render shows and avoids re-running Nucleo
                    let get_selected_file = || {
                        this.file_search_display_indices
                            .get(*selected_index)
                            .and_then(|&idx| this.cached_file_results.get(idx))
                            .cloned()
                    };

                    match key_str.as_str() {
                        // Arrow keys are handled by arrow_interceptor in app_impl.rs
                        // which calls stop_propagation(). This is the single source of truth
                        // for arrow key handling in FileSearchView.
                        "up" | "arrowup" | "down" | "arrowdown" => {
                            // Already handled by interceptor, no-op here
                        }
                        // Tab/Shift+Tab handled by intercept_keystrokes in app_impl.rs
                        // (interceptor fires BEFORE input component can capture Tab)
                        "enter" | "return" => {
                            // Check for Cmd+Enter (reveal in finder) first
                            if has_cmd {
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::reveal_in_finder(&file.path);
                                }
                            } else {
                                // Open file with default app
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::open_file(&file.path);
                                    // Close window after opening file
                                    this.close_and_reset_window(cx);
                                }
                            }
                        }
                        _ => {
                            // Handle Cmd+K (toggle actions)
                            if has_cmd && key_str == "k" {
                                if let Some(file) = get_selected_file() {
                                    this.toggle_file_search_actions(&file, window, cx);
                                }
                                return;
                            }
                            // Handle Cmd+Y (Quick Look) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "y" {
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::quick_look(&file.path);
                                }
                                return;
                            }
                            // Handle Cmd+I (Show Info) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "i" {
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::show_info(&file.path);
                                }
                            }
                            // Handle Cmd+O (Open With) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "o" {
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::open_with(&file.path);
                                }
                            }
                        }
                    }
                }
            },
        );

        // Clone data for the uniform_list closure
        // Use display_indices to get files in the correct order (filtered + sorted)
        // Include the original result index for animation timestamp lookup
        let files_for_closure: Vec<(usize, file_search::FileResult)> = display_indices
            .iter()
            .filter_map(|&idx| self.cached_file_results.get(idx).map(|f| (idx, f.clone())))
            .collect();
        let current_selected = selected_index;
        let is_loading = self.file_search_loading;

        // Use uniform_list for virtualized scrolling
        // Skeleton loading: show placeholder rows while loading and no results yet
        let list_element = if is_loading && filtered_len == 0 {
            // Loading with no results yet - show static skeleton rows
            let skeleton_bg = rgba((ui_border << 8) | 0x30); // ~18% opacity

            // Render 6 skeleton rows
            div()
                .w_full()
                .h_full()
                .flex()
                .flex_col()
                .children((0..6).map(|ix| {
                    div()
                        .id(ix)
                        .w_full()
                        .h(px(52.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .px(px(12.))
                        .gap(px(12.))
                        // Icon placeholder
                        .child(div().w(px(24.)).h(px(24.)).rounded(px(6.)).bg(skeleton_bg))
                        // Text placeholders
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(6.))
                                .child(div().w(px(160.)).h(px(12.)).rounded(px(4.)).bg(skeleton_bg))
                                .child(
                                    div().w(px(240.)).h(px(10.)).rounded(px(4.)).bg(skeleton_bg),
                                ),
                        )
                        // Right side placeholders (size/time)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_end()
                                .gap(px(6.))
                                .child(div().w(px(56.)).h(px(10.)).rounded(px(4.)).bg(skeleton_bg))
                                .child(div().w(px(72.)).h(px(10.)).rounded(px(4.)).bg(skeleton_bg)),
                        )
                }))
                .into_any_element()
        } else if filtered_len == 0 {
            // No results and not loading - show empty state message
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(text_dimmed))
                .child(if query.is_empty() {
                    "Type to search files"
                } else {
                    "No files found"
                })
                .into_any_element()
        } else {
            uniform_list(
                "file-search-list",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_result_idx, file)) = files_for_closure.get(ix) {
                                let is_selected = ix == current_selected;

                                // Use theme opacity for vibrancy-compatible selection
                                let bg = if is_selected {
                                    rgba((list_selected << 8) | selected_alpha)
                                } else {
                                    rgba(0x00000000)
                                };
                                let hover_bg = rgba((list_hover << 8) | hover_alpha);

                                div()
                                    .id(ix)
                                    .w_full()
                                    .h(px(52.))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .px(px(12.))
                                    .gap(px(12.))
                                    .bg(bg)
                                    .hover(move |s| s.bg(hover_bg))
                                    .child(
                                        div()
                                            .text_lg()
                                            .text_color(rgb(text_muted))
                                            .child(file_search::file_type_icon(file.file_type)),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(text_primary))
                                                    .child(file.name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(text_dimmed))
                                                    .child(file_search::shorten_path(&file.path)),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .items_end()
                                            .gap(px(2.))
                                            .child(
                                                div().text_xs().text_color(rgb(text_dimmed)).child(
                                                    file_search::format_file_size(file.size),
                                                ),
                                            )
                                            .child(
                                                div().text_xs().text_color(rgb(text_dimmed)).child(
                                                    file_search::format_relative_time(
                                                        file.modified,
                                                    ),
                                                ),
                                            ),
                                    )
                            } else {
                                div().id(ix).h(px(52.))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.file_search_scroll_handle)
            .into_any_element()
        };

        // Build preview panel content - matching main menu labeled section pattern
        let preview_content = if let Some(file) = &selected_file {
            let file_type_str = match file.file_type {
                FileType::Directory => "Folder",
                FileType::Image => "Image",
                FileType::Audio => "Audio",
                FileType::Video => "Video",
                FileType::Document => "Document",
                FileType::Application => "Application",
                FileType::File => "File",
                FileType::Other => "File",
            };

            div()
                .flex_1()
                .flex()
                .flex_col()
                .p(px(design_spacing.padding_lg))
                .gap(px(design_spacing.gap_md))
                .overflow_y_hidden()
                // Name section (labeled like main menu)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .pb(px(design_spacing.padding_md))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(design_spacing.padding_xs / 2.0))
                                .child("Name"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(design_spacing.gap_sm))
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgb(text_primary))
                                        .child(file.name.clone()),
                                )
                                .child(
                                    div()
                                        .px(px(6.))
                                        .py(px(2.))
                                        .rounded(px(4.))
                                        .bg(rgba((ui_border << 8) | 0x40))
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .child(file_type_str),
                                ),
                        ),
                )
                // Path section (labeled)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .pb(px(design_spacing.padding_md))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(design_spacing.padding_xs / 2.0))
                                .child("Path"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_dimmed))
                                .child(file.path.clone()),
                        ),
                )
                // Divider (like main menu)
                .child(
                    div()
                        .w_full()
                        .h(px(design_visual.border_thin))
                        .bg(rgba((ui_border << 8) | 0x60))
                        .my(px(design_spacing.padding_sm)),
                )
                // Details section (labeled)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(design_spacing.padding_xs / 2.0))
                                .child("Details"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(design_spacing.gap_sm))
                                .child(div().text_sm().text_color(rgb(text_dimmed)).child(format!(
                                    "Size: {}",
                                    file_search::format_file_size(file.size)
                                )))
                                .child(div().text_sm().text_color(rgb(text_dimmed)).child(format!(
                                    "Modified: {}",
                                    file_search::format_relative_time(file.modified)
                                )))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_dimmed))
                                        .child(format!("Type: {}", file_type_str)),
                                ),
                        ),
                )
        } else if is_loading {
            // When loading, show empty preview (no distracting message)
            div().flex_1()
        } else {
            div().flex_1().flex().items_center().justify_center().child(
                div()
                    .text_sm()
                    .text_color(rgb(text_dimmed))
                    .child("No file selected"),
            )
        };

        // Main container - styled to match main menu exactly
        // NOTE: No border to match main menu (border adds visual padding/shift)
        div()
            .key_context("FileSearchView")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            // Removed: .shadow(box_shadows) - shadows on transparent elements block vibrancy
            .rounded(px(design_visual.radius_lg))
            // Header with search input - styled to match main menu exactly
            // Uses shared header constants (HEADER_PADDING_X/Y, CURSOR_HEIGHT_LG) for visual consistency.
            // The right-side element uses same py(4px) padding as main menu's "Ask AI" button
            // to ensure identical flex row height (28px) and input vertical centering.
            .child({
                // Calculate input height using same formula as main menu
                let input_height = CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0);

                div()
                    .w_full()
                    .px(px(HEADER_PADDING_X))
                    .py(px(HEADER_PADDING_Y))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(HEADER_GAP))
                    // Search input - matches main menu Input styling for visual consistency
                    // NOTE: Removed search icon to match main menu alignment exactly
                    .child(
                        div().flex_1().flex().flex_row().items_center().child(
                            Input::new(&self.gpui_input_state)
                                .w_full()
                                .h(px(input_height))
                                .px(px(0.))
                                .py(px(0.))
                                .with_size(Size::Size(px(_design_typography.font_size_xl)))
                                .appearance(false)
                                .bordered(false)
                                .focus_bordered(false),
                        ),
                    )
                    // Right-side element styled to match main menu's "Ask AI" button height
                    // Using fixed width to prevent layout shift when content changes
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_end()
                            .py(px(4.))
                            .w(px(70.)) // Fixed width prevents layout shift
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(text_dimmed))
                                    .child(format!("{} files", filtered_len)),
                            ),
                    )
            })
            // Divider
            .child(
                div()
                    .mx(px(design_spacing.padding_lg))
                    .h(px(design_visual.border_thin))
                    .bg(rgba((ui_border << 8) | 0x60)),
            )
            // Main content: loading state OR empty state OR 50/50 split
            .child(if is_loading && filtered_len == 0 {
                // Loading state: full-width centered (no split, clean appearance)
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .min_h(px(0.))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child("Searching..."),
                    )
            } else if filtered_len == 0 {
                // Empty state: single centered message (no awkward 50/50 split)
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .min_h(px(0.))
                    .child(
                        div().flex().flex_col().items_center().gap(px(8.)).child(
                            div()
                                .text_color(rgb(text_dimmed))
                                .child(if query.is_empty() {
                                    "Type to search files"
                                } else {
                                    "No files found"
                                }),
                        ),
                    )
            } else {
                // Normal state: 50/50 split with list and preview
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .flex_row()
                    .min_h(px(0.))
                    .overflow_hidden()
                    // Left panel: file list (50%)
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .border_r(px(design_visual.border_thin))
                            .border_color(rgba((ui_border << 8) | 0x40))
                            .child(list_element),
                    )
                    // Right panel: preview (50%)
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .child(preview_content),
                    )
            })
            // Footer
            .child(PromptFooter::new(
                PromptFooterConfig::new()
                    .primary_label("Open")
                    .primary_shortcut("↵"),
                // Default config already has secondary_label="Actions", secondary_shortcut="⌘K", show_secondary=true
                PromptFooterColors::from_theme(&self.theme),
            ))
            .into_any_element()
    }
}
