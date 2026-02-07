use super::*;

impl NotesApp {
    /// Create a new NotesApp
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Initialize storage
        if let Err(e) = storage::init_notes_db() {
            tracing::error!(error = %e, "Failed to initialize notes database");
        }

        // Auto-prune trash entries older than 30 days
        match storage::prune_old_deleted_notes(30) {
            Ok(pruned) if pruned > 0 => {
                info!(
                    pruned_count = pruned,
                    "Auto-pruned old trash notes (>30 days)"
                );
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to auto-prune trash");
            }
            _ => {}
        }

        // Load notes from storage
        let mut notes = storage::get_all_notes().unwrap_or_default();
        let deleted_notes = storage::get_deleted_notes().unwrap_or_default();

        // First launch: create a welcome note if no notes exist
        if notes.is_empty() && deleted_notes.is_empty() {
            let welcome = Note::with_content(Self::welcome_note_content());
            if let Err(e) = storage::save_note(&welcome) {
                tracing::error!(error = %e, "Failed to create welcome note");
            } else {
                notes.push(welcome);
                info!("Created welcome note for first launch");
            }
        }

        let selected_note_id = notes.first().map(|n| n.id);

        // Get initial content if we have a selected note
        let initial_content = selected_note_id
            .and_then(|id| notes.iter().find(|n| n.id == id))
            .map(|n| n.content.clone())
            .unwrap_or_default();

        // Calculate initial line count for auto-resize (before moving content)
        let initial_line_count = initial_content.lines().count().max(1);

        // Ensure markdown language is registered before editor initialization
        register_markdown_highlighter();

        // Create input states - use code_editor for markdown highlighting
        let editor_state = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("markdown")
                .line_number(false)
                .searchable(true)
                .rows(20)
                .placeholder("Start typing your note...")
                .default_value(initial_content)
        });

        let search_state = cx.new(|cx| InputState::new(window, cx).placeholder("Search notes..."));

        let focus_handle = cx.focus_handle();

        // Subscribe to editor changes - passes window for auto-resize
        let editor_sub = cx.subscribe_in(&editor_state, window, {
            move |this, _, ev: &InputEvent, window, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.on_editor_change(window, cx);
                }
            }
        });

        // Subscribe to search changes
        let search_sub = cx.subscribe_in(&search_state, window, {
            move |this, _, ev: &InputEvent, _window, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.on_search_change(cx);
                }
            }
        });

        // Get initial window height to use as minimum
        let initial_height: f32 = window.bounds().size.height.into();

        info!(
            note_count = notes.len(),
            initial_height = initial_height,
            "Notes app initialized"
        );

        // Pre-compute note switcher actions before moving notes into struct
        let note_switcher_actions = get_note_switcher_actions(
            &notes
                .iter()
                .map(|n| NoteSwitcherNoteInfo {
                    id: n.id.as_str().to_string(),
                    title: if n.title.is_empty() {
                        "Untitled Note".to_string()
                    } else {
                        n.title.clone()
                    },
                    char_count: n.char_count(),
                    is_current: Some(n.id) == selected_note_id,
                    is_pinned: n.is_pinned,
                    preview: Self::strip_markdown_for_preview(&n.preview()),
                    relative_time: Self::format_relative_time(n.updated_at),
                })
                .collect::<Vec<_>>(),
        );

        Self {
            notes,
            deleted_notes,
            view_mode: NotesViewMode::AllNotes,
            selected_note_id,
            editor_state,
            search_state,
            search_query: String::new(),
            titlebar_hovered: false,
            window_hovered: false,
            mouse_cursor_hidden: false,
            force_hovered: false,
            show_format_toolbar: false,
            show_search: false,
            preview_enabled: false,
            last_line_count: initial_line_count,
            initial_height,
            auto_sizing_enabled: true,          // Auto-sizing ON by default
            last_window_height: initial_height, // Track for manual resize detection
            focus_handle,
            _subscriptions: vec![editor_sub, search_sub],
            show_actions_panel: false,
            show_browse_panel: false,
            actions_panel: None,
            // Initialize CommandBar with notes-specific actions
            command_bar: CommandBar::new(
                get_notes_command_bar_actions(&NotesInfo {
                    has_selection: selected_note_id.is_some(),
                    is_trash_view: false,
                    auto_sizing_enabled: true,
                }),
                CommandBarConfig::notes_style(),
                std::sync::Arc::new(theme::load_theme()),
            ),
            // Initialize note switcher CommandBar (Cmd+P) with note list
            note_switcher: CommandBar::new(
                note_switcher_actions,
                CommandBarConfig::notes_style(),
                std::sync::Arc::new(theme::load_theme()),
            ),
            browse_panel: None,
            pending_action: Arc::new(Mutex::new(None)),
            actions_panel_prev_height: None,
            pending_browse_select: Arc::new(Mutex::new(None)),
            pending_browse_close: Arc::new(Mutex::new(false)),
            pending_browse_action: Arc::new(Mutex::new(None)),
            has_unsaved_changes: false,
            last_save_time: None,
            last_persisted_bounds: None,
            last_bounds_save: Instant::now(),
            theme_rev_seen: crate::theme::service::theme_revision(),
            history_back: Vec::new(),
            history_forward: Vec::new(),
            navigating_history: false,
            focus_mode: false,
            sort_mode: NotesSortMode::default(),
            show_shortcuts_help: false,
            last_save_confirmed: None,
            action_feedback: None,
        }
    }

    /// Debounce interval for saves (in milliseconds)
    const SAVE_DEBOUNCE_MS: u64 = 300;

    /// Debounce interval for bounds persistence (in milliseconds)
    const BOUNDS_DEBOUNCE_MS: u64 = 250;

    /// Update cached theme-derived values if theme revision has changed.
    ///
    /// This is called during render to detect theme hot-reloads.
    /// NOTE: Box shadows were removed for vibrancy compatibility.
    pub(super) fn maybe_update_theme_cache(&mut self) {
        let current_rev = crate::theme::service::theme_revision();
        if self.theme_rev_seen != current_rev {
            self.theme_rev_seen = current_rev;
            // Box shadows disabled for vibrancy - no cached values to update
        }
    }

    /// Persist window bounds if they've changed (debounced).
    ///
    /// This ensures bounds are saved even when the window is closed via traffic light
    /// (red close button) which doesn't go through our close handlers.
    pub(super) fn maybe_persist_bounds(&mut self, window: &gpui::Window) {
        let wb = window.window_bounds();

        // Skip if bounds haven't changed
        if self.last_persisted_bounds.as_ref() == Some(&wb) {
            return;
        }

        // Debounce to avoid too-frequent saves
        if self.last_bounds_save.elapsed()
            < std::time::Duration::from_millis(Self::BOUNDS_DEBOUNCE_MS)
        {
            return;
        }

        // Save bounds
        self.last_persisted_bounds = Some(wb);
        self.last_bounds_save = Instant::now();
        crate::window_state::save_window_from_gpui(crate::window_state::WindowRole::Notes, wb);
    }

    /// Save the current note if it has unsaved changes
    pub(super) fn save_current_note(&mut self) {
        if !self.has_unsaved_changes {
            return;
        }

        if let Some(id) = self.selected_note_id {
            if let Some(note) = self.notes.iter().find(|n| n.id == id) {
                if let Err(e) = storage::save_note(note) {
                    tracing::error!(error = %e, "Failed to save note");
                    return;
                }
                debug!(note_id = %id, "Note saved (debounced)");
            }
        }

        self.has_unsaved_changes = false;
        self.last_save_time = Some(Instant::now());
        self.last_save_confirmed = Some(Instant::now());
    }

    /// Check if we should save now (debounce check)
    pub(super) fn should_save_now(&self) -> bool {
        if !self.has_unsaved_changes {
            return false;
        }

        match self.last_save_time {
            None => true,
            Some(last_save) => last_save.elapsed() >= Duration::from_millis(Self::SAVE_DEBOUNCE_MS),
        }
    }

    /// Handle editor content changes with auto-resize
    pub(super) fn on_editor_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let content = self.editor_state.read(cx).value();
        let content_string = content.to_string();

        // Auto-create a note if user is typing with no note selected
        // This prevents data loss when users start typing immediately
        if self.selected_note_id.is_none() && !content_string.is_empty() {
            info!("Auto-creating note from unselected editor content");
            let note = Note::with_content(content_string.clone());
            let id = note.id;

            // Save to storage
            if let Err(e) = storage::save_note(&note) {
                tracing::error!(error = %e, "Failed to create auto-generated note");
                return;
            }

            // Add to cache and select it
            self.notes.insert(0, note);
            self.selected_note_id = Some(id);
            cx.notify();
            return;
        }

        if let Some(id) = self.selected_note_id {
            // Update the note in our cache (in-memory only)
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                note.set_content(content_string.clone());
                // Mark as dirty - actual save is debounced
                self.has_unsaved_changes = true;
            }

            // Auto-resize: adjust window height based on content
            let new_line_count = content_string.lines().count().max(1);
            if new_line_count != self.last_line_count {
                self.last_line_count = new_line_count;
                self.update_window_height(window, new_line_count, cx);
            }

            cx.notify();
        }
    }

    /// Update window height based on content line count
    /// Raycast-style: window grows AND shrinks to fit content when auto_sizing_enabled
    /// IMPORTANT: Window never shrinks below initial_height (the height at window creation)
    pub(super) fn update_window_height(
        &mut self,
        window: &mut Window,
        line_count: usize,
        _cx: &mut Context<Self>,
    ) {
        // Skip if auto-sizing is disabled (user manually resized)
        if !self.auto_sizing_enabled {
            return;
        }

        // Use initial_height as minimum - never shrink below starting size
        let min_height = self.initial_height;

        // Calculate desired height
        let content_height = (line_count as f32) * AUTO_RESIZE_LINE_HEIGHT;
        let total_height = TITLEBAR_HEIGHT + content_height + FOOTER_HEIGHT + AUTO_RESIZE_PADDING;
        let clamped_height = total_height.clamp(min_height, AUTO_RESIZE_MAX_HEIGHT);

        // Get current bounds and update height
        let current_bounds = window.bounds();
        let old_height: f32 = current_bounds.size.height.into();

        // Resize if height needs to change (both grow AND shrink)
        // Use a small threshold to avoid constant tiny adjustments
        if (clamped_height - old_height).abs() > AUTO_RESIZE_THRESHOLD {
            let new_size = size(current_bounds.size.width, px(clamped_height));

            debug!(
                old_height = old_height,
                new_height = clamped_height,
                min_height = min_height,
                line_count = line_count,
                auto_sizing = self.auto_sizing_enabled,
                "Auto-resize: adjusting window height"
            );

            window.resize(new_size);
            self.last_window_height = clamped_height;
        }
    }

    /// Enable auto-sizing (called from actions panel)
    pub fn enable_auto_sizing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.auto_sizing_enabled = true;
        // Re-calculate and apply the correct height
        let line_count = self.last_line_count;
        self.update_window_height(window, line_count, cx);
        info!("Auto-sizing enabled");
        cx.notify();
    }

    /// Check if user manually resized the window and disable auto-sizing if so
    pub(super) fn detect_manual_resize(&mut self, window: &Window) {
        if !self.auto_sizing_enabled {
            return; // Already disabled
        }

        let current_height: f32 = window.bounds().size.height.into();

        // If height differs significantly from what we set, user resized manually
        if (current_height - self.last_window_height).abs() > MANUAL_RESIZE_THRESHOLD {
            self.auto_sizing_enabled = false;
            self.last_window_height = current_height;
            debug!(
                current_height = current_height,
                last_height = self.last_window_height,
                "Manual resize detected - auto-sizing disabled"
            );
        }
    }
}
