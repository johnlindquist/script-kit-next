#[allow(dead_code)] // Public API - many methods for future integrations
impl CommandBar {
    /// Create a new CommandBar with actions and configuration
    pub fn new(actions: Vec<Action>, config: CommandBarConfig, theme: Arc<theme::Theme>) -> Self {
        Self {
            dialog: None,
            actions,
            config,
            theme,
            is_open: false,
            on_action: None,
        }
    }

    /// Set the action callback
    pub fn with_on_action(mut self, callback: CommandBarActionCallback) -> Self {
        self.on_action = Some(callback);
        self
    }

    /// Set the action callback (mutable version)
    pub fn set_on_action(&mut self, callback: CommandBarActionCallback) {
        self.on_action = Some(callback);
    }

    /// Update the actions list
    pub fn set_actions(&mut self, actions: Vec<Action>, cx: &mut App) {
        self.actions = actions.clone();

        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| {
                d.actions = actions;
                d.filtered_actions = (0..d.actions.len()).collect();
                d.selected_index = 0;
                d.search_text.clear();
                cx.notify();
            });

            if self.is_open {
                resize_actions_window(cx, dialog);
            }
        }
    }

    /// Update the theme
    pub fn set_theme(&mut self, theme: Arc<theme::Theme>, cx: &mut App) {
        self.theme = theme.clone();

        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| {
                d.update_theme(theme);
                cx.notify();
            });

            if self.is_open {
                notify_actions_window(cx);
            }
        }
    }

    /// Toggle open/close state (for Cmd+K binding)
    pub fn toggle<V: 'static>(&mut self, window: &mut Window, cx: &mut Context<V>) {
        if self.is_open {
            self.close(cx);
        } else {
            self.open(window, cx);
        }
    }

    /// Open the command bar at the default position (bottom-right)
    pub fn open<V: 'static>(&mut self, window: &mut Window, cx: &mut Context<V>) {
        self.open_at_position(window, cx, super::window::WindowPosition::BottomRight);
    }

    /// Open the command bar at top-center (Raycast-style, for Notes window)
    pub fn open_centered<V: 'static>(&mut self, window: &mut Window, cx: &mut Context<V>) {
        self.open_at_position(window, cx, super::window::WindowPosition::TopCenter);
    }

    /// Open the command bar at a specific position
    pub fn open_at_position<V: 'static>(
        &mut self,
        window: &mut Window,
        cx: &mut Context<V>,
        position: super::window::WindowPosition,
    ) {
        if self.is_open {
            return;
        }

        // Create callback for dialog
        let on_select: Arc<dyn Fn(String) + Send + Sync> = Arc::new(|_| {
            // Action handling is done via execute_selected_action()
        });

        // Create the dialog entity
        let theme = self.theme.clone();
        let actions = self.actions.clone();
        let config = self.config.dialog_config.clone();

        // Log what actions we're creating the dialog with
        logging::log(
            "COMMAND_BAR",
            &format!(
                "Creating dialog with {} actions: [{}]",
                actions.len(),
                actions
                    .iter()
                    .take(5)
                    .map(|a| a.title.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );

        let dialog = cx.new(|cx| {
            let mut d =
                ActionsDialog::with_config(cx.focus_handle(), on_select, actions, theme, config);
            // Tell dialog to skip track_focus - ActionsWindow handles focus instead
            // This ensures keyboard events go to ActionsWindow's on_key_down handler
            d.set_skip_track_focus(true);
            d
        });

        // Get window bounds and display for positioning
        let bounds = window.bounds();
        let display_id = window.display(cx).map(|d| d.id());

        // Store dialog and mark as open
        self.dialog = Some(dialog.clone());
        self.is_open = true;

        // Open the vibrancy window at the specified position
        match open_actions_window(cx, bounds, display_id, dialog, position) {
            Ok(_) => {
                logging::log(
                    "COMMAND_BAR",
                    &format!("Command bar opened at {:?}", position),
                );
            }
            Err(e) => {
                logging::log("COMMAND_BAR", &format!("Failed to open command bar: {}", e));
                self.is_open = false;
                self.dialog = None;
            }
        }

        cx.notify();
    }

    /// Close the command bar
    pub fn close<V: 'static>(&mut self, cx: &mut Context<V>) {
        if !self.is_open {
            return;
        }

        close_actions_window(cx);
        self.is_open = false;
        self.dialog = None;
        logging::log("COMMAND_BAR", "Command bar closed");
        cx.notify();
    }

    /// Close the command bar (App context version)
    pub fn close_app(&mut self, cx: &mut App) {
        if !self.is_open {
            return;
        }

        close_actions_window(cx);
        self.is_open = false;
        self.dialog = None;
        logging::log("COMMAND_BAR", "Command bar closed");
    }

    /// Check if the command bar is open
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Get the currently selected action ID
    pub fn get_selected_action_id(&self, cx: &App) -> Option<String> {
        self.dialog
            .as_ref()
            .and_then(|d| d.read(cx).get_selected_action_id())
    }

    /// Get the currently selected action
    pub fn get_selected_action<'a>(&'a self, cx: &'a App) -> Option<&'a Action> {
        self.dialog
            .as_ref()
            .and_then(|d| d.read(cx).get_selected_action())
    }

    /// Execute the selected action and optionally close the command bar
    ///
    /// Returns the action ID if an action was executed, None otherwise.
    pub fn execute_selected_action<V: 'static>(&mut self, cx: &mut Context<V>) -> Option<String> {
        let action_id = self.get_selected_action_id(cx)?;

        // Call the callback if set
        if let Some(callback) = &self.on_action {
            callback(&action_id);
        }

        // Close if configured to do so
        if self.config.close_on_select {
            self.close(cx);
        }

        Some(action_id)
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| d.handle_char(ch, cx));
            resize_actions_window(cx, dialog);
        }
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| d.handle_backspace(cx));
            resize_actions_window(cx, dialog);
        }
    }

    /// Move selection up
    pub fn select_prev(&mut self, cx: &mut App) {
        logging::log(
            "COMMAND_BAR",
            &format!(
                "select_prev called, dialog exists: {}",
                self.dialog.is_some()
            ),
        );
        if let Some(dialog) = &self.dialog {
            let old_idx = dialog.read(cx).selected_index;
            dialog.update(cx, |d, cx| d.move_up(cx));
            let new_idx = dialog.read(cx).selected_index;
            logging::log(
                "COMMAND_BAR",
                &format!("select_prev: index {} -> {}", old_idx, new_idx),
            );
            notify_actions_window(cx);
        }
    }

    /// Move selection down
    pub fn select_next(&mut self, cx: &mut App) {
        logging::log(
            "COMMAND_BAR",
            &format!(
                "select_next called, dialog exists: {}",
                self.dialog.is_some()
            ),
        );
        if let Some(dialog) = &self.dialog {
            let old_idx = dialog.read(cx).selected_index;
            dialog.update(cx, |d, cx| d.move_down(cx));
            let new_idx = dialog.read(cx).selected_index;
            logging::log(
                "COMMAND_BAR",
                &format!("select_next: index {} -> {}", old_idx, new_idx),
            );
            notify_actions_window(cx);
        }
    }

    /// Jump to first action in the list.
    pub fn select_first(&mut self, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| {
                if let Some(first) = first_selectable_index(&d.grouped_items) {
                    d.selected_index = first;
                    d.list_state.scroll_to_reveal_item(d.selected_index);
                    cx.notify();
                }
            });
            notify_actions_window(cx);
        }
    }

    /// Jump to last action in the list.
    pub fn select_last(&mut self, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| {
                if let Some(last) = last_selectable_index(&d.grouped_items) {
                    d.selected_index = last;
                    d.list_state.scroll_to_reveal_item(d.selected_index);
                    cx.notify();
                }
            });
            notify_actions_window(cx);
        }
    }

    /// Move one page up in the list.
    pub fn select_page_up(&mut self, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| {
                if d.grouped_items.is_empty() {
                    return;
                }

                let target = d.selected_index.saturating_sub(COMMAND_BAR_PAGE_JUMP);
                if let Some(next_index) = selectable_index_at_or_before(&d.grouped_items, target)
                    .or_else(|| first_selectable_index(&d.grouped_items))
                {
                    d.selected_index = next_index;
                    d.list_state.scroll_to_reveal_item(d.selected_index);
                    cx.notify();
                }
            });
            notify_actions_window(cx);
        }
    }

    /// Move one page down in the list.
    pub fn select_page_down(&mut self, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, cx| {
                if d.grouped_items.is_empty() {
                    return;
                }

                let last_index = d.grouped_items.len() - 1;
                let target = (d.selected_index + COMMAND_BAR_PAGE_JUMP).min(last_index);
                if let Some(next_index) = selectable_index_at_or_after(&d.grouped_items, target)
                    .or_else(|| last_selectable_index(&d.grouped_items))
                {
                    d.selected_index = next_index;
                    d.list_state.scroll_to_reveal_item(d.selected_index);
                    cx.notify();
                }
            });
            notify_actions_window(cx);
        }
    }

    /// Set cursor visibility (for blink animation)
    pub fn set_cursor_visible(&mut self, visible: bool, cx: &mut App) {
        if let Some(dialog) = &self.dialog {
            dialog.update(cx, |d, _cx| {
                d.set_cursor_visible(visible);
            });
            notify_actions_window(cx);
        }
    }

    /// Get the dialog entity (for advanced use cases)
    pub fn dialog(&self) -> Option<&Entity<ActionsDialog>> {
        self.dialog.as_ref()
    }

    /// Get access to the focus handle of the underlying dialog
    pub fn focus_handle(&self, cx: &App) -> Option<FocusHandle> {
        self.dialog
            .as_ref()
            .map(|d| d.read(cx).focus_handle.clone())
    }
}
