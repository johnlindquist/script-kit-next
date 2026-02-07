use super::*;

impl PathPrompt {
    pub fn new(
        id: String,
        start_path: Option<String>,
        hint: Option<String>,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let current_path = start_path.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string())
        });

        logging::log(
            "PROMPTS",
            &format!("PathPrompt::new starting at: {}", current_path),
        );

        // Load entries from current path
        let entries = Self::load_entries(&current_path);
        let filtered_entries = entries.clone();

        PathPrompt {
            id,
            start_path,
            hint,
            current_path,
            filter_text: String::new(),
            selected_index: 0,
            entries,
            filtered_entries,
            focus_handle,
            on_submit,
            theme,
            design_variant: DesignVariant::Default,
            list_scroll_handle: UniformListScrollHandle::new(),
            on_show_actions: None,
            on_close_actions: None,
            actions_showing: Arc::new(Mutex::new(false)),
            actions_search_text: Arc::new(Mutex::new(String::new())),
            cursor_visible: true,
        }
    }

    /// Set the callback for showing actions dialog
    pub fn with_show_actions(mut self, callback: ShowActionsCallback) -> Self {
        self.on_show_actions = Some(callback);
        self
    }

    /// Set the show actions callback (mutable version)
    pub fn set_show_actions(&mut self, callback: ShowActionsCallback) {
        self.on_show_actions = Some(callback);
    }

    /// Set the close actions callback (for toggle behavior)
    pub fn with_close_actions(mut self, callback: CloseActionsCallback) -> Self {
        self.on_close_actions = Some(callback);
        self
    }

    /// Set the shared actions_showing state (for toggle behavior)
    pub fn with_actions_showing(mut self, actions_showing: Arc<Mutex<bool>>) -> Self {
        self.actions_showing = actions_showing;
        self
    }

    /// Set the shared actions_search_text state (for header display)
    pub fn with_actions_search_text(mut self, actions_search_text: Arc<Mutex<String>>) -> Self {
        self.actions_search_text = actions_search_text;
        self
    }

    /// Load directory entries from a path
    pub(super) fn load_entries(dir_path: &str) -> Vec<PathEntry> {
        let path = Path::new(dir_path);
        let mut entries = Vec::new();

        // No ".." entry - use left arrow to navigate to parent

        // Read directory entries
        if let Ok(read_dir) = std::fs::read_dir(path) {
            let mut dirs: Vec<PathEntry> = Vec::new();
            let mut files: Vec<PathEntry> = Vec::new();

            for entry in read_dir.flatten() {
                let entry_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files (starting with .)
                if name.starts_with('.') {
                    continue;
                }

                let is_dir = entry_path.is_dir();
                let path_entry = PathEntry {
                    name,
                    path: entry_path.to_string_lossy().to_string(),
                    is_dir,
                };

                if is_dir {
                    dirs.push(path_entry);
                } else {
                    files.push(path_entry);
                }
            }

            // Sort alphabetically (case insensitive)
            dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            // Add dirs first, then files
            entries.extend(dirs);
            entries.extend(files);
        }

        logging::log(
            "PROMPTS",
            &format!(
                "PathPrompt loaded {} entries from {}",
                entries.len(),
                dir_path
            ),
        );
        entries
    }

    /// Update filtered entries based on filter text
    pub(super) fn update_filtered(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_entries = self.entries.clone();
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            self.filtered_entries = self
                .entries
                .iter()
                .filter(|e| e.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect();
        }

        // Reset selection to 0 if out of bounds
        if self.selected_index >= self.filtered_entries.len() {
            self.selected_index = 0;
        }
    }

    /// Set the current filter text programmatically
    pub fn set_input(&mut self, text: String, cx: &mut Context<Self>) {
        if self.filter_text == text {
            return;
        }

        self.filter_text = text;
        self.update_filtered();
        self.selected_index = 0;
        self.list_scroll_handle
            .scroll_to_item(0, gpui::ScrollStrategy::Top);
        cx.notify();
    }

    /// Navigate into a directory
    pub fn navigate_to(&mut self, path: &str, cx: &mut Context<Self>) {
        self.current_path = path.to_string();
        self.entries = Self::load_entries(path);
        self.filter_text.clear();
        self.filtered_entries = self.entries.clone();
        self.selected_index = 0;
        cx.notify();
    }

    /// Show actions dialog for the selected entry
    /// Emits PathPromptEvent::ShowActions for parent to handle
    pub(super) fn show_actions(&mut self, cx: &mut Context<Self>) {
        if let Some(entry) = self.filtered_entries.get(self.selected_index) {
            let path_info = PathInfo::new(entry.name.clone(), entry.path.clone(), entry.is_dir);
            logging::log(
                "PROMPTS",
                &format!(
                    "PathPrompt emitting ShowActions for: {} (is_dir={})",
                    path_info.path, path_info.is_dir
                ),
            );
            // Emit event for parent to handle (GPUI pattern)
            cx.emit(PathPromptEvent::ShowActions(path_info.clone()));
            // Also call legacy callback if present (backwards compatibility)
            if let Some(ref callback) = self.on_show_actions {
                (callback)(path_info);
            }
            cx.notify();
        }
    }

    /// Close actions dialog (for toggle behavior)
    /// Emits PathPromptEvent::CloseActions for parent to handle
    pub(super) fn close_actions(&mut self, cx: &mut Context<Self>) {
        logging::log("PROMPTS", "PathPrompt emitting CloseActions");
        // Emit event for parent to handle (GPUI pattern)
        cx.emit(PathPromptEvent::CloseActions);
        // Also call legacy callback if present (backwards compatibility)
        if let Some(ref callback) = self.on_close_actions {
            (callback)();
        }
        cx.notify();
    }

    /// Toggle actions dialog - show if hidden, close if showing
    pub fn toggle_actions(&mut self, cx: &mut Context<Self>) {
        let is_showing = self.actions_showing.lock().map(|g| *g).unwrap_or(false);

        if is_showing {
            logging::log(
                "PROMPTS",
                "PathPrompt toggle: closing actions (was showing)",
            );
            self.close_actions(cx);
        } else {
            logging::log("PROMPTS", "PathPrompt toggle: showing actions (was hidden)");
            self.show_actions(cx);
        }
    }

    /// Submit the selected path - always submits, never navigates
    /// For files and directories: submit the path (script will handle it)
    /// Navigation into directories is handled by → and Tab keys
    pub(super) fn submit_selected(&mut self, _cx: &mut Context<Self>) {
        if let Some(entry) = self.filtered_entries.get(self.selected_index) {
            // Always submit the path, whether it's a file or directory
            // The calling script or default handler will decide what to do with it
            logging::log(
                "PROMPTS",
                &format!(
                    "PathPrompt submitting path: {} (is_dir={})",
                    entry.path, entry.is_dir
                ),
            );
            (self.on_submit)(self.id.clone(), Some(entry.path.clone()));
        } else if !self.filter_text.is_empty() {
            // If no entry selected but filter has text, submit the filter as a path
            logging::log(
                "PROMPTS",
                &format!(
                    "PathPrompt submitting filter text as path: {}",
                    self.filter_text
                ),
            );
            (self.on_submit)(self.id.clone(), Some(self.filter_text.clone()));
        }
    }

    /// Handle Enter key - always submit the selected path
    /// The calling code (main.rs) will open it with system default via std::process::Command
    pub fn handle_enter(&mut self, cx: &mut Context<Self>) {
        // Always submit directly - no actions dialog on Enter
        // Actions are available via Cmd+K
        self.submit_selected(cx);
    }

    /// Cancel - submit None
    pub fn submit_cancel(&mut self) {
        logging::log(
            "PROMPTS",
            &format!(
                "PathPrompt submit_cancel called - submitting None for id: {}",
                self.id
            ),
        );
        (self.on_submit)(self.id.clone(), None);
    }

    /// Move selection up
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_scroll_handle
                .scroll_to_item(self.selected_index, gpui::ScrollStrategy::Top);
            cx.notify();
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.filtered_entries.len().saturating_sub(1) {
            self.selected_index += 1;
            self.list_scroll_handle
                .scroll_to_item(self.selected_index, gpui::ScrollStrategy::Top);
            cx.notify();
        }
    }

    /// Handle character input
    pub(super) fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.filter_text.push(ch);
        self.update_filtered();
        cx.notify();
    }

    /// Handle backspace - if filter empty, go up one directory
    pub(super) fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.filter_text.is_empty() {
            self.filter_text.pop();
            self.update_filtered();
            cx.notify();
        } else {
            // If filter is empty, navigate up one directory
            let path = Path::new(&self.current_path);
            if let Some(parent) = path.parent() {
                let parent_path = parent.to_string_lossy().to_string();
                self.navigate_to(&parent_path, cx);
            }
        }
    }

    /// Navigate to parent directory (left arrow / shift+tab)
    pub fn navigate_to_parent(&mut self, cx: &mut Context<Self>) {
        let path = Path::new(&self.current_path);
        if let Some(parent) = path.parent() {
            let parent_path = parent.to_string_lossy().to_string();
            logging::log(
                "PROMPTS",
                &format!("PathPrompt navigating to parent: {}", parent_path),
            );
            self.navigate_to(&parent_path, cx);
        }
        // If at root, do nothing
    }

    /// Navigate into selected directory (right arrow / tab)
    pub fn navigate_into_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(entry) = self.filtered_entries.get(self.selected_index) {
            if entry.is_dir {
                let path = entry.path.clone();
                logging::log("PROMPTS", &format!("PathPrompt navigating into: {}", path));
                self.navigate_to(&path, cx);
            }
            // If selected entry is a file, do nothing
        }
    }

    /// Get the currently selected path info (for actions dialog)
    pub fn get_selected_path_info(&self) -> Option<PathInfo> {
        self.filtered_entries
            .get(self.selected_index)
            .map(|entry| PathInfo::new(entry.name.clone(), entry.path.clone(), entry.is_dir))
    }
}
