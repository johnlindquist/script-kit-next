use super::*;

impl ScriptListApp {
    pub(crate) fn toggle_logs(&mut self, cx: &mut Context<Self>) {
        self.show_logs = !self.show_logs;
        cx.notify();
    }

    /// Hide the mouse cursor while typing.
    /// The cursor will be shown again when the mouse moves.
    pub(crate) fn hide_mouse_cursor(&mut self, cx: &mut Context<Self>) {
        if !self.mouse_cursor_hidden {
            self.mouse_cursor_hidden = true;
            crate::platform::hide_cursor_until_mouse_moves();
            cx.notify();
        }
    }

    /// Show the mouse cursor (called when mouse moves).
    /// Also switches to Mouse input mode to re-enable hover effects.
    pub(crate) fn show_mouse_cursor(&mut self, cx: &mut Context<Self>) {
        self.input_mode = InputMode::Mouse;
        if self.mouse_cursor_hidden {
            self.mouse_cursor_hidden = false;
        }
        cx.notify();
    }

    /// Calculate view type and item count for window sizing.
    /// Extracted from update_window_size for reuse.
    pub(crate) fn calculate_window_size_params(&mut self) -> Option<(ViewType, usize)> {
        match &self.current_view {
            AppView::ScriptList => {
                // Get grouped results which includes section headers (cached)
                let (grouped_items, _) = self.get_grouped_results_cached();
                let count = grouped_items.len();
                Some((ViewType::ScriptList, count))
            }
            AppView::ArgPrompt { choices, .. } => {
                let filtered = self.get_filtered_arg_choices(choices);
                if filtered.is_empty() && choices.is_empty() {
                    Some((ViewType::ArgPromptNoChoices, 0))
                } else {
                    Some((ViewType::ArgPromptWithChoices, filtered.len()))
                }
            }
            AppView::DivPrompt { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::FormPrompt { .. } => Some((ViewType::DivPrompt, 0)), // Use DivPrompt size for forms
            AppView::EditorPrompt { .. } => Some((ViewType::EditorPrompt, 0)),
            AppView::SelectPrompt { .. } => Some((ViewType::ArgPromptWithChoices, 0)),
            AppView::PathPrompt { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::EnvPrompt { .. } => Some((ViewType::ArgPromptNoChoices, 0)), // Compact: header + footer only
            AppView::DropPrompt { .. } => Some((ViewType::DivPrompt, 0)), // Drop prompt uses div size for drop zone
            AppView::TemplatePrompt { .. } => Some((ViewType::DivPrompt, 0)), // Template prompt uses div size
            AppView::ChatPrompt { .. } => Some((ViewType::DivPrompt, 0)), // Chat prompt uses div size
            AppView::TermPrompt { .. } => Some((ViewType::TermPrompt, 0)),
            AppView::ActionsDialog => {
                // Actions dialog is an overlay, don't resize
                None
            }
            // P0 FIX: Clipboard history and app launcher use standard height (same as script list)
            // View state only - data comes from self fields
            AppView::ClipboardHistoryView { filter, .. } => {
                let entries = &self.cached_clipboard_entries;
                let filtered_count = if filter.is_empty() {
                    entries.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    entries
                        .iter()
                        .filter(|e| e.text_preview.to_lowercase().contains(&filter_lower))
                        .count()
                };
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::PasteSequentiallyView { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::EmojiPickerView {
                filter,
                selected_category,
                ..
            } => {
                let filtered_count = crate::emoji::search_emojis(filter)
                    .into_iter()
                    .filter(|emoji| {
                        selected_category
                            .map(|category| emoji.category == category)
                            .unwrap_or(true)
                    })
                    .count();
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::AppLauncherView { filter, .. } => {
                let apps = &self.apps;
                let filtered_count = if filter.is_empty() {
                    apps.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    apps.iter()
                        .filter(|a| a.name.to_lowercase().contains(&filter_lower))
                        .count()
                };
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::WindowSwitcherView { filter, .. } => {
                let windows = &self.cached_windows;
                let filtered_count = if filter.is_empty() {
                    windows.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    windows
                        .iter()
                        .filter(|w| {
                            w.title.to_lowercase().contains(&filter_lower)
                                || w.app.to_lowercase().contains(&filter_lower)
                        })
                        .count()
                };
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::DesignGalleryView { filter, .. } => {
                // Calculate total gallery items (separators + icons)
                let total_items = designs::separator_variations::SeparatorStyle::count()
                    + designs::icon_variations::total_icon_count();
                let filtered_count = if filter.is_empty() {
                    total_items
                } else {
                    // For now, return total - filtering can be added later
                    total_items
                };
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::ScratchPadView { .. } => Some((ViewType::EditorPrompt, 0)),
            AppView::QuickTerminalView { .. } => Some((ViewType::TermPrompt, 0)),
            AppView::WebcamView { .. } => Some((ViewType::DivPrompt, 0)),
            AppView::FileSearchView { ref query, .. } => {
                let results = &self.cached_file_results;
                let filtered_count = if query.is_empty() {
                    results.len()
                } else {
                    let query_lower = query.to_lowercase();
                    results
                        .iter()
                        .filter(|r| r.name.to_lowercase().contains(&query_lower))
                        .count()
                };
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::ThemeChooserView { ref filter, .. } => {
                let presets = theme::presets::presets_cached();
                let filtered_count = if filter.is_empty() {
                    presets.len()
                } else {
                    let f = filter.to_lowercase();
                    presets
                        .iter()
                        .filter(|p| {
                            p.name.to_lowercase().contains(&f)
                                || p.description.to_lowercase().contains(&f)
                        })
                        .count()
                };
                Some((ViewType::ScriptList, filtered_count))
            }
            AppView::CreationFeedback { .. } => Some((ViewType::ArgPromptNoChoices, 0)),
            AppView::NamingPrompt { .. } => Some((ViewType::ArgPromptNoChoices, 0)),
        }
    }

    /// Update window size using deferred execution (SAFE during render/event cycles).
    ///
    /// Uses Window::defer to schedule the resize at the end of the current effect cycle,
    /// preventing RefCell borrow conflicts that can occur when calling platform APIs
    /// during GPUI's render or event processing.
    ///
    /// Use this version when you have access to `window` and `cx`.
    pub(crate) fn update_window_size_deferred(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((view_type, item_count)) = self.calculate_window_size_params() {
            crate::window_resize::defer_resize_to_view(view_type, item_count, window, &mut *cx);
        }
    }

    /// Update window size synchronously.
    ///
    /// SAFETY: Only call from async handlers (cx.spawn closures, message handlers)
    /// that run OUTSIDE the GPUI render cycle. Calling during render will cause
    /// RefCell borrow panics.
    ///
    /// Prefer `update_window_size_deferred` when you have window/cx access.
    pub(crate) fn update_window_size(&mut self) {
        if let Some((view_type, item_count)) = self.calculate_window_size_params() {
            let target_height = height_for_view(view_type, item_count);
            resize_first_window_to_height(target_height);
        }
    }

    pub(crate) fn set_prompt_input(&mut self, text: String, cx: &mut Context<Self>) {
        match &mut self.current_view {
            AppView::ArgPrompt { .. } => {
                self.arg_input.set_text(text);
                self.arg_selected_index = 0;
                self.arg_list_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
                self.update_window_size();
                cx.notify();
            }
            AppView::PathPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::SelectPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::EnvPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::TemplatePrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::FormPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::FileSearchView {
                query,
                selected_index,
            } => {
                let results = ScriptListApp::resolve_file_search_results(&text);
                logging::log(
                    "EXEC",
                    &format!(
                        "File search setInput '{}' found {} results",
                        text,
                        results.len()
                    ),
                );
                *query = text.clone();
                *selected_index = 0;
                self.update_file_search_results(results);
                self.file_search_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
                // Mark that we need to sync the input text on next render
                self.filter_text = text;
                self.pending_filter_sync = true;
                cx.notify();
            }
            _ => {}
        }
    }

    /// Helper to get filtered arg choices without cloning
    pub(crate) fn get_filtered_arg_choices<'a>(&self, choices: &'a [Choice]) -> Vec<&'a Choice> {
        if self.arg_input.is_empty() {
            choices.iter().collect()
        } else {
            let filter = self.arg_input.text().to_lowercase();
            choices
                .iter()
                .filter(|c| c.name.to_lowercase().contains(&filter))
                .collect()
        }
    }

    pub(crate) fn focus_main_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focused_input = FocusedInput::MainFilter;
        let input_state = self.gpui_input_state.clone();
        input_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });
    }
}
