// Builtin view render methods - extracted from app_render.rs
// This file is included via include!() macro in main.rs
// Contains: render_clipboard_history, render_app_launcher, render_window_switcher, render_design_gallery

impl ScriptListApp {
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

            // Restore focus state - file search uses the main filter input
            self.focused_input = FocusedInput::MainFilter;
            self.gpui_input_focused = true;

            // Close the actions window via spawn
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                })
                .ok();
            })
            .detach();

            // Refocus the file search input
            self.focus_main_filter(window, cx);
            logging::log(
                "FOCUS",
                "File search actions closed, focus returned to file search input",
            );
        } else {
            // Open actions popup for the selected file
            self.show_actions_popup = true;

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
                cx.update(
                    |cx| match open_actions_window(cx, main_bounds, display_id, dialog) {
                        Ok(_handle) => {
                            logging::log("ACTIONS", "File search actions popup window opened");
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open actions window: {}", e));
                        }
                    },
                )
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
        let box_shadows = self.create_box_shadows();

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
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                // Global shortcuts (Cmd+W, ESC for dismissable views)
                if this.handle_global_shortcut_with_options(event, true, cx) {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
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

                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                // Scroll to keep selection visible
                                this.clipboard_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if *selected_index < filtered_len.saturating_sub(1) {
                                *selected_index += 1;
                                // Scroll to keep selection visible
                                this.clipboard_list_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        "enter" => {
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
                                    // Hide window first
                                    script_kit_gpui::set_main_window_visible(false);
                                    cx.hide();
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
            .shadow(box_shadows)
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
            .child(PromptFooter::new(
                PromptFooterConfig::new()
                    .primary_label("Paste")
                    .primary_shortcut("↵")
                    .show_secondary(false),
                PromptFooterColors::from_theme(&self.theme),
            ))
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
        let box_shadows = self.create_box_shadows();

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
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                // Global shortcuts (Cmd+W) - handled first regardless of view state
                // Global shortcuts (Cmd+W, ESC for dismissable views)
                if this.handle_global_shortcut_with_options(event, true, cx) {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
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
                        "enter" => {
                            // Launch selected app and hide window
                            if let Some((_, app)) = filtered_apps.get(*selected_index) {
                                logging::log("EXEC", &format!("Launching app: {}", app.name));
                                if let Err(e) = app_launcher::launch_application(app) {
                                    logging::log("ERROR", &format!("Failed to launch app: {}", e));
                                } else {
                                    logging::log("EXEC", &format!("Launched: {}", app.name));
                                    // Hide window after launching
                                    script_kit_gpui::set_main_window_visible(false);
                                    cx.hide();
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
            .shadow(box_shadows)
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
        let box_shadows = self.create_box_shadows();

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
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                // Global shortcuts (Cmd+W, ESC for dismissable views)
                if this.handle_global_shortcut_with_options(event, true, cx) {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
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
                        "enter" => {
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
                                    script_kit_gpui::set_main_window_visible(false);
                                    cx.hide();
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
            .shadow(box_shadows)
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
        let box_shadows = self.create_box_shadows();

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
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                // Global shortcuts (Cmd+W) - handled first regardless of view state
                // Global shortcuts (Cmd+W, ESC for dismissable views)
                if this.handle_global_shortcut_with_options(event, true, cx) {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
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
            .shadow(box_shadows)
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
        let box_shadows = self.create_box_shadows();

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

        // Filter results based on query
        // When query is a directory path, extract the filter component for instant filtering
        // e.g., ~/dev/fin -> filter by "fin" on directory contents
        // During directory transitions, use frozen filter to prevent showing wrong results
        let filter_pattern = if let Some(ref frozen) = self.file_search_frozen_filter {
            // Use frozen filter during directory transitions
            // This keeps old results displaying correctly while new results load
            frozen.clone()
        } else if let Some(parsed) = crate::file_search::parse_directory_path(query) {
            parsed.filter // Some("fin") or None
        } else if !query.is_empty() {
            // Not a directory path - use query as filter for search results
            Some(query.to_string())
        } else {
            None
        };

        // Use Nucleo fuzzy matching for filtering - gives better match quality ranking
        let filtered_results: Vec<_> = if let Some(ref pattern) = filter_pattern {
            file_search::filter_results_nucleo_simple(&self.cached_file_results, pattern)
        } else {
            // No filter - show all results
            self.cached_file_results.iter().enumerate().collect()
        };
        let filtered_len = filtered_results.len();

        // Get selected file for preview (if any)
        let selected_file = filtered_results
            .get(selected_index)
            .map(|(_, r)| (*r).clone());

        // Key handler for file search
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;

                // Route keys to actions dialog first if it's open
                match this.route_key_to_actions_dialog(
                    &key_str,
                    key_char,
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

                // ESC goes back to main menu (not close window)
                if key_str == "escape" {
                    logging::log("KEY", "ESC in FileSearch - returning to main menu");
                    // Cancel any pending search
                    this.file_search_debounce_task = None;
                    this.file_search_loading = false;
                    // Clear cached results
                    this.cached_file_results.clear();
                    // Return to main menu
                    this.current_view = AppView::ScriptList;
                    this.filter_text.clear();
                    this.selected_index = 0;
                    // Sync input and reset placeholder to default
                    this.gpui_input_state.update(cx, |state, cx| {
                        state.set_value("", window, cx);
                        // Ensure cursor is at start (empty string, so 0..0)
                        state.set_selection(0, 0, window, cx);
                        state.set_placeholder(DEFAULT_PLACEHOLDER.to_string(), window, cx);
                    });
                    this.update_window_size_deferred(window, cx);
                    cx.notify();
                    return;
                }

                // Cmd+W closes window
                if has_cmd && key_str == "w" {
                    logging::log("KEY", "Cmd+W - closing window");
                    this.close_and_reset_window(cx);
                    return;
                }

                if let AppView::FileSearchView {
                    query,
                    selected_index,
                } = &mut this.current_view
                {
                    // Apply filter to get current filtered list
                    // Use parse_directory_path to extract filter pattern
                    let filter_pattern =
                        if let Some(parsed) = crate::file_search::parse_directory_path(query) {
                            parsed.filter
                        } else if !query.is_empty() {
                            Some(query.clone())
                        } else {
                            None
                        };

                    // Use Nucleo fuzzy matching for filtering
                    let filtered_results: Vec<_> = if let Some(ref pattern) = filter_pattern {
                        crate::file_search::filter_results_nucleo_simple(
                            &this.cached_file_results,
                            pattern,
                        )
                    } else {
                        this.cached_file_results.iter().enumerate().collect()
                    };
                    let _filtered_len = filtered_results.len();

                    match key_str.as_str() {
                        // Arrow keys are handled by arrow_interceptor in app_impl.rs
                        // which calls stop_propagation(). This is the single source of truth
                        // for arrow key handling in FileSearchView.
                        "up" | "arrowup" | "down" | "arrowdown" => {
                            // Already handled by interceptor, no-op here
                        }
                        // Tab/Shift+Tab handled by intercept_keystrokes in app_impl.rs
                        // (interceptor fires BEFORE input component can capture Tab)
                        "enter" => {
                            // Check for Cmd+Enter (reveal in finder) first
                            if has_cmd {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::reveal_in_finder(&file.path);
                                }
                            } else {
                                // Open file with default app
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::open_file(&file.path);
                                    // Close window after opening file
                                    this.close_and_reset_window(cx);
                                }
                            }
                        }
                        _ => {
                            // Handle Cmd+K (toggle actions)
                            if has_cmd && key_str == "k" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    // Clone the file to avoid borrow issues
                                    let file_clone = (*file).clone();
                                    this.toggle_file_search_actions(&file_clone, window, cx);
                                }
                                return;
                            }
                            // Handle Cmd+Y (Quick Look) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "y" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::quick_look(&file.path);
                                }
                                return;
                            }
                            // Handle Cmd+I (Show Info) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "i" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::show_info(&file.path);
                                }
                            }
                            // Handle Cmd+O (Open With) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "o" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::open_with(&file.path);
                                }
                            }
                        }
                    }
                }
            },
        );

        // Clone data for the uniform_list closure
        let files_for_closure: Vec<_> = filtered_results
            .iter()
            .map(|(_, file)| (*file).clone())
            .collect();
        let current_selected = selected_index;
        let is_loading = self.file_search_loading;

        // Use uniform_list for virtualized scrolling
        // Note: Loading state with 0 results is handled by the main content section (full-width spinner)
        // This list_element is only used in the 50/50 split when we have results
        let list_element = if filtered_len == 0 {
            // No results and not loading
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
                            if let Some(file) = files_for_closure.get(ix) {
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
            .shadow(box_shadows)
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
