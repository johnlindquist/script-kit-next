impl ActionsDialog {
    pub fn new(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        Self::with_script_and_design(focus_handle, on_select, None, theme, DesignVariant::Default)
    }

    pub fn with_script(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        focused_script: Option<ScriptInfo>,
        theme: Arc<theme::Theme>,
    ) -> Self {
        Self::with_script_and_design(
            focus_handle,
            on_select,
            focused_script,
            theme,
            DesignVariant::Default,
        )
    }

    pub fn with_design(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        theme: Arc<theme::Theme>,
        design_variant: DesignVariant,
    ) -> Self {
        Self::with_script_and_design(focus_handle, on_select, None, theme, design_variant)
    }

    #[allow(clippy::too_many_arguments)]
    fn from_actions_with_context(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        actions: Vec<Action>,
        focused_script: Option<ScriptInfo>,
        focused_scriptlet: Option<Scriptlet>,
        theme: Arc<theme::Theme>,
        design_variant: DesignVariant,
        context_title: Option<String>,
        config: ActionsDialogConfig,
    ) -> Self {
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();
        let grouped_items =
            build_grouped_items_static(&actions, &filtered_actions, config.section_style);
        let list_state = ListState::new(grouped_items.len(), ListAlignment::Top, px(100.));
        let selected_index = initial_selection_index(&grouped_items);

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script,
            focused_scriptlet,
            list_state,
            grouped_items,
            theme,
            design_variant,
            cursor_visible: true,
            hide_search: matches!(config.search_position, SearchPosition::Hidden),
            sdk_actions: None,
            sdk_action_indices: Vec::new(),
            context_title,
            config,
            skip_track_focus: false,
            on_close: None,
        }
    }

    /// Create ActionsDialog for a path (file/folder) with path-specific actions
    pub fn with_path(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        path_info: &PathInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_path_context_actions(path_info);
        let config = ActionsDialogConfig::default();

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for path: {} (is_dir={}) with {} actions",
                path_info.path,
                path_info.is_dir,
                actions.len()
            ),
        );

        Self::from_actions_with_context(
            focus_handle,
            on_select,
            actions,
            None,
            None,
            theme,
            DesignVariant::Default,
            Some(path_info.path.clone()),
            config,
        )
    }

    /// Create ActionsDialog for a file search result with file-specific actions
    /// Actions: Open, Reveal in Finder, Quick Look, Open With..., Show Info, Copy Path
    pub fn with_file(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        file_info: &FileInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_file_context_actions(file_info);
        let config = ActionsDialogConfig::default();

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for file: {} (is_dir={}) with {} actions",
                file_info.path,
                file_info.is_dir,
                actions.len()
            ),
        );

        Self::from_actions_with_context(
            focus_handle,
            on_select,
            actions,
            None,
            None,
            theme,
            DesignVariant::Default,
            Some(file_info.name.clone()),
            config,
        )
    }

    /// Create ActionsDialog for a clipboard history entry with clipboard-specific actions
    /// Actions: Paste, Copy, Paste and Keep Open, Share, Attach to AI, Pin/Unpin, Delete, etc.
    pub fn with_clipboard_entry(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        entry_info: &ClipboardEntryInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_clipboard_history_context_actions(entry_info);
        let config = ActionsDialogConfig::default();

        let context_title = if entry_info.preview.len() > 30 {
            format!("{}...", &entry_info.preview[..27])
        } else {
            entry_info.preview.clone()
        };

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for clipboard entry: {} (type={:?}, pinned={}) with {} actions",
                entry_info.id,
                entry_info.content_type,
                entry_info.pinned,
                actions.len()
            ),
        );

        Self::from_actions_with_context(
            focus_handle,
            on_select,
            actions,
            None,
            None,
            theme,
            DesignVariant::Default,
            Some(context_title),
            config,
        )
    }

    /// Create ActionsDialog for a chat prompt with chat-specific actions
    /// Actions: Model selection, Continue in Chat, Copy Response, Clear Conversation
    pub fn with_chat(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        chat_info: &ChatPromptInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_chat_context_actions(chat_info);
        let config = ActionsDialogConfig::default();

        let context_title = chat_info
            .current_model
            .clone()
            .unwrap_or_else(|| "Chat".to_string());

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for chat prompt: model={:?} with {} actions",
                chat_info.current_model,
                actions.len()
            ),
        );

        Self::from_actions_with_context(
            focus_handle,
            on_select,
            actions,
            None,
            None,
            theme,
            DesignVariant::Default,
            Some(context_title),
            config,
        )
    }

    pub fn with_script_and_design(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        focused_script: Option<ScriptInfo>,
        theme: Arc<theme::Theme>,
        design_variant: DesignVariant,
    ) -> Self {
        let actions = Self::build_actions(&focused_script, &None);
        let config = ActionsDialogConfig::default();

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created with {} actions, script: {:?}, design: {:?}",
                actions.len(),
                focused_script.as_ref().map(|s| &s.name),
                design_variant
            ),
        );

        // Log theme color configuration for debugging
        logging::log("ACTIONS_THEME", &format!(
            "Theme colors applied: bg_main=#{:06x}, bg_search=#{:06x}, text_primary=#{:06x}, accent_selected=#{:06x}",
            theme.colors.background.main,
            theme.colors.background.search_box,
            theme.colors.text.primary,
            theme.colors.accent.selected
        ));

        // Extract context title from focused script if available
        let context_title = focused_script.as_ref().map(|s| s.name.clone());

        Self::from_actions_with_context(
            focus_handle,
            on_select,
            actions,
            focused_script,
            None,
            theme,
            design_variant,
            context_title,
            config,
        )
    }

    /// Update cursor visibility (called from parent's blink timer)
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    /// Hide the search input (for inline mode where header has search)
    pub fn set_hide_search(&mut self, hide: bool) {
        self.hide_search = hide;
    }

    /// Set the context title shown in the header
    pub fn set_context_title(&mut self, title: Option<String>) {
        self.context_title = title;
    }

    /// Set the configuration for appearance and behavior
    pub fn set_config(&mut self, config: ActionsDialogConfig) {
        let should_rebuild = should_rebuild_grouped_items_for_config_change(&self.config, &config);
        let previously_selected_action_index = self.selected_action_index();

        self.config = config;
        // Update hide_search based on config for backwards compatibility
        self.hide_search = matches!(self.config.search_position, SearchPosition::Hidden);

        if should_rebuild {
            self.rebuild_grouped_items();
            self.selected_index = previously_selected_action_index
                .and_then(|action_idx| self.grouped_index_for_action_index(action_idx))
                .unwrap_or_else(|| initial_selection_index(&self.grouped_items));
            if !self.grouped_items.is_empty() {
                self.list_state.scroll_to_reveal_item(self.selected_index);
            }
        }
    }

    /// Set skip_track_focus to let parent handle focus (used by ActionsWindow)
    pub fn set_skip_track_focus(&mut self, skip: bool) {
        self.skip_track_focus = skip;
    }

    /// Set the callback for when the dialog is closed (escape pressed, window dismissed)
    /// Used to notify the main app to restore focus
    pub fn set_on_close(&mut self, callback: CloseCallback) {
        self.on_close = Some(callback);
    }

    /// Call the on_close callback if set
    /// Returns true if a callback was called, false otherwise
    pub fn trigger_on_close(&self, cx: &mut gpui::App) -> bool {
        if let Some(ref callback) = self.on_close {
            callback(cx);
            true
        } else {
            false
        }
    }

    /// Create ActionsDialog with custom configuration and actions
    ///
    /// Use this for contexts like AI chat that need different appearance:
    /// - Search at top instead of bottom
    /// - Section headers instead of separators
    /// - Icons next to actions
    pub fn with_config(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        actions: Vec<Action>,
        theme: Arc<theme::Theme>,
        config: ActionsDialogConfig,
    ) -> Self {
        let filtered_actions_preview: Vec<usize> = (0..actions.len()).collect();
        let grouped_items_preview =
            build_grouped_items_static(&actions, &filtered_actions_preview, config.section_style);
        let initial_selection = initial_selection_index(&grouped_items_preview);

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created with config: {} actions, search={:?}, section_style={:?}, initial_selection={}",
                actions.len(),
                config.search_position,
                config.section_style,
                initial_selection
            ),
        );

        Self::from_actions_with_context(
            focus_handle,
            on_select,
            actions,
            None,
            None,
            theme,
            DesignVariant::Default,
            None,
            config,
        )
    }

    /// Parse a shortcut string into individual keycap characters
    /// e.g., "⌘↵" → vec!["⌘", "↵"], "⌘I" → vec!["⌘", "I"]
    pub(crate) fn parse_shortcut_keycaps(shortcut: &str) -> Vec<String> {
        let mut keycaps = Vec::new();

        for ch in shortcut.chars() {
            // Handle modifier symbols (single character)
            match ch {
                '⌘' | '⌃' | '⌥' | '⇧' | '↵' | '⎋' | '⇥' | '⌫' | '␣' | '↑' | '↓' | '←' | '→' =>
                {
                    keycaps.push(ch.to_string());
                }
                // Regular characters (letters, numbers)
                _ => {
                    keycaps.push(ch.to_uppercase().to_string());
                }
            }
        }

        keycaps
    }
}
