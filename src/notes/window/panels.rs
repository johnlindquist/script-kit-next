use super::*;

impl NotesApp {
    pub(super) fn build_action_items(&self) -> Vec<NotesActionItem> {
        let has_selection = self.selected_note_id.is_some();
        let is_trash = self.view_mode == NotesViewMode::Trash;
        let can_edit = has_selection && !is_trash;

        let mut items: Vec<NotesActionItem> = NotesAction::all()
            .iter()
            .map(|action| {
                let enabled = match action {
                    NotesAction::NewNote | NotesAction::BrowseNotes => true,
                    NotesAction::DuplicateNote
                    | NotesAction::FindInNote
                    | NotesAction::CopyNoteAs
                    | NotesAction::CopyDeeplink
                    | NotesAction::CreateQuicklink
                    | NotesAction::Export
                    | NotesAction::Format => can_edit,
                    NotesAction::MoveListItemUp | NotesAction::MoveListItemDown => false,
                    NotesAction::EnableAutoSizing => !self.auto_sizing_enabled,
                    NotesAction::Cancel => true,
                };

                NotesActionItem {
                    action: *action,
                    enabled,
                }
            })
            .collect();

        if !self.auto_sizing_enabled {
            items.push(NotesActionItem {
                action: NotesAction::EnableAutoSizing,
                enabled: true,
            });
        }

        items
    }

    pub(super) fn open_actions_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Update command bar actions based on current state (dynamic - depends on selection, etc.)
        let actions = get_notes_command_bar_actions(&NotesInfo {
            has_selection: self.selected_note_id.is_some(),
            is_trash_view: self.view_mode == NotesViewMode::Trash,
            auto_sizing_enabled: self.auto_sizing_enabled,
        });

        // Log what actions we're setting
        info!(
            "Notes open_actions_panel: setting {} actions: [{}]",
            actions.len(),
            actions
                .iter()
                .take(5)
                .map(|a| a.title.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        self.command_bar.set_actions(actions, cx);

        // Open the command bar (CommandBar handles window creation internally)
        self.command_bar.open_centered(window, cx);

        // CRITICAL: Focus main focus_handle so keyboard events route to us
        // The ActionsWindow is a visual-only popup - it does NOT take keyboard focus.
        // macOS popup windows often don't receive keyboard events properly.
        self.focus_handle.focus(window, cx);

        // Update state flags
        self.show_actions_panel = true;
        self.show_browse_panel = false;
        self.browse_panel = None;

        cx.notify();
    }

    pub(super) fn close_actions_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Close the command bar window
        self.command_bar.close(cx);

        self.show_actions_panel = false;
        self.actions_panel = None;

        // Refocus the editor after closing the actions panel
        self.editor_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });

        cx.notify();
    }

    pub(super) fn ensure_actions_panel_height(&mut self, window: &mut Window, row_count: usize) {
        let panel_height = panel_height_for_rows(row_count);
        let desired_height = panel_height + ACTIONS_PANEL_WINDOW_MARGIN;
        let current_bounds = window.bounds();
        let current_height: f32 = current_bounds.size.height.into();

        if current_height + 1.0 < desired_height {
            self.actions_panel_prev_height = Some(current_height);
            window.resize(size(current_bounds.size.width, px(desired_height)));
            self.last_window_height = desired_height;
        }
    }

    pub(super) fn restore_actions_panel_height(&mut self, window: &mut Window) {
        let Some(prev_height) = self.actions_panel_prev_height.take() else {
            return;
        };

        let current_bounds = window.bounds();
        window.resize(size(current_bounds.size.width, px(prev_height)));
        self.last_window_height = prev_height;
    }

    pub(super) fn drain_pending_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let pending_action = self
            .pending_action
            .lock()
            .ok()
            .and_then(|mut pending| pending.take());

        if let Some(action) = pending_action {
            self.handle_action(action, window, cx);
        }
    }

    /// Drain pending browse panel actions (select, close, note actions)
    pub(super) fn drain_pending_browse_actions(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Check for pending note selection
        let pending_select = self
            .pending_browse_select
            .lock()
            .ok()
            .and_then(|mut guard| guard.take());

        if let Some(id) = pending_select {
            self.handle_browse_select(id, window, cx);
            return; // Selection closes the panel, so we're done
        }

        // Check for pending close request
        let pending_close = self
            .pending_browse_close
            .lock()
            .ok()
            .map(|mut guard| {
                let val = *guard;
                *guard = false;
                val
            })
            .unwrap_or(false);

        if pending_close {
            self.close_browse_panel(window, cx);
            return;
        }

        // Check for pending note action (pin/delete)
        let pending_action = self
            .pending_browse_action
            .lock()
            .ok()
            .and_then(|mut guard| guard.take());

        if let Some((id, action)) = pending_action {
            self.handle_browse_action(id, action, cx);
        }
    }

    /// Handle action from the actions panel (Cmd+K)
    pub(super) fn handle_action(
        &mut self,
        action: NotesAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        debug!(?action, "Handling notes action");
        match action {
            NotesAction::NewNote => self.create_note(window, cx),
            NotesAction::DuplicateNote => self.duplicate_selected_note(window, cx),
            NotesAction::BrowseNotes => {
                // Close actions panel first, then open browse panel
                // Don't call close_actions_panel here - it refocuses editor
                // Instead, just clear the state and let open_browse_panel handle focus
                self.show_actions_panel = false;
                self.actions_panel = None;
                self.restore_actions_panel_height(window);
                self.show_browse_panel = true;
                self.open_browse_panel(window, cx);
                cx.notify();
                return; // Early return - browse panel handles its own focus
            }
            NotesAction::FindInNote => {
                self.close_actions_panel(window, cx);
                self.editor_state.update(cx, |state, cx| {
                    state.focus(window, cx);
                });
                cx.dispatch_action(&Search);
                return; // Early return - already handled focus
            }
            NotesAction::CopyNoteAs => self.copy_note_as_markdown(),
            NotesAction::CopyDeeplink => self.copy_note_deeplink(),
            NotesAction::CreateQuicklink => self.create_note_quicklink(),
            NotesAction::Export => self.export_note(ExportFormat::Html),
            NotesAction::MoveListItemUp | NotesAction::MoveListItemDown => {}
            NotesAction::Format => {
                self.show_format_toolbar = !self.show_format_toolbar;
            }
            NotesAction::EnableAutoSizing => {
                self.enable_auto_sizing(window, cx);
            }
            NotesAction::Cancel => {
                // Panel was cancelled, nothing to do
            }
        }
        // Default: close actions panel and refocus editor
        self.close_actions_panel(window, cx);
        cx.notify();
    }

    /// Execute an action by ID (from CommandBar)
    /// Maps string action IDs to NotesAction enum values
    pub(super) fn execute_action(
        &mut self,
        action_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        debug!(action_id, "Executing notes action from CommandBar");

        // Map action ID strings to NotesAction enum
        let action = match action_id {
            "new_note" => Some(NotesAction::NewNote),
            "duplicate_note" => Some(NotesAction::DuplicateNote),
            "browse_notes" => Some(NotesAction::BrowseNotes),
            "find_in_note" => Some(NotesAction::FindInNote),
            "format" => Some(NotesAction::Format),
            "copy_note_as" => Some(NotesAction::CopyNoteAs),
            "copy_deeplink" => Some(NotesAction::CopyDeeplink),
            "create_quicklink" => Some(NotesAction::CreateQuicklink),
            "export" => Some(NotesAction::Export),
            "enable_auto_sizing" => Some(NotesAction::EnableAutoSizing),
            _ => {
                tracing::warn!(action_id, "Unknown action ID from CommandBar");
                None
            }
        };

        if let Some(action) = action {
            self.handle_action(action, window, cx);
        } else {
            // Unknown action - just close the command bar
            self.close_actions_panel(window, cx);
        }
    }

    /// Execute an action from the note switcher (Cmd+P)
    /// Handles note selection when action_id starts with "note_"
    pub(super) fn execute_note_switcher_action(
        &mut self,
        action_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        debug!(action_id, "Executing note switcher action");

        // Handle note selection (action_id format: "note_{uuid}")
        if let Some(note_id_str) = action_id.strip_prefix("note_") {
            // Find the note by ID string
            if let Some(note) = self.notes.iter().find(|n| n.id.as_str() == note_id_str) {
                let note_id = note.id;
                self.close_browse_panel(window, cx);
                self.select_note(note_id, window, cx);
                return;
            }
        }

        // Handle "no_notes" placeholder action
        if action_id == "no_notes" {
            self.close_browse_panel(window, cx);
            self.create_note(window, cx);
            return;
        }

        // Unknown action - just close
        tracing::warn!(action_id, "Unknown note switcher action");
        self.close_browse_panel(window, cx);
    }

    /// Open the browse panel (note switcher) with current notes
    /// Uses CommandBar for consistent theming with the Cmd+K actions dialog
    pub(super) fn open_browse_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Update note switcher actions based on current notes
        let note_switcher_actions = get_note_switcher_actions(
            &self
                .notes
                .iter()
                .map(|n| NoteSwitcherNoteInfo {
                    id: n.id.as_str().to_string(),
                    title: if n.title.is_empty() {
                        "Untitled Note".to_string()
                    } else {
                        n.title.clone()
                    },
                    char_count: n.char_count(),
                    is_current: Some(n.id) == self.selected_note_id,
                    is_pinned: n.is_pinned,
                    preview: Self::strip_markdown_for_preview(&n.preview()),
                    relative_time: Self::format_relative_time(n.updated_at),
                })
                .collect::<Vec<_>>(),
        );

        // Log what actions we're setting
        info!(
            "Notes open_browse_panel: setting {} note actions",
            note_switcher_actions.len(),
        );

        self.note_switcher.set_actions(note_switcher_actions, cx);

        // Open the note switcher (CommandBar handles window creation internally)
        self.note_switcher.open_centered(window, cx);

        // CRITICAL: Focus main focus_handle so keyboard events route to us
        // The ActionsWindow is a visual-only popup - it does NOT take keyboard focus.
        self.focus_handle.focus(window, cx);

        // Update state flags
        self.show_browse_panel = true;
        self.show_actions_panel = false;
        self.browse_panel = None; // Clear legacy browse panel

        cx.notify();
    }

    /// Handle note selection from browse panel
    pub(super) fn handle_browse_select(
        &mut self,
        id: NoteId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_browse_panel = false;
        self.browse_panel = None;
        // select_note already focuses the editor
        self.select_note(id, window, cx);
        cx.notify();
    }

    /// Handle note action from browse panel
    pub(super) fn handle_browse_action(
        &mut self,
        id: NoteId,
        action: NoteAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            NoteAction::TogglePin => {
                if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                    note.is_pinned = !note.is_pinned;
                    if let Err(e) = storage::save_note(note) {
                        tracing::error!(error = %e, "Failed to save note pin state");
                    }
                }
                // Re-sort notes: pinned first, then by updated_at descending
                self.notes.sort_by(|a, b| match (a.is_pinned, b.is_pinned) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => b.updated_at.cmp(&a.updated_at),
                });
                cx.notify();
            }
            NoteAction::Delete => {
                let current_id = self.selected_note_id;
                self.selected_note_id = Some(id);
                self.delete_selected_note(cx);
                // Restore selection if different note was deleted
                if current_id != Some(id) {
                    self.selected_note_id = current_id;
                }
            }
        }
        // Update browse panel's note list
        if let Some(ref browse_panel) = self.browse_panel {
            let note_items: Vec<NoteListItem> = self
                .notes
                .iter()
                .map(|note| NoteListItem::from_note(note, Some(note.id) == self.selected_note_id))
                .collect();
            browse_panel.update(cx, |panel, cx| {
                panel.set_notes(note_items, cx);
            });
        }
        cx.notify();
    }

    /// Close the browse panel (note switcher) and refocus the editor
    pub(super) fn close_browse_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Close the note switcher CommandBar window
        self.note_switcher.close(cx);

        self.show_browse_panel = false;
        self.browse_panel = None;

        // Refocus the editor after closing the browse panel
        self.editor_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });

        cx.notify();
    }

    /// Toggle the search bar visibility
    pub(super) fn toggle_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Exit focus mode if active (search requires chrome)
        if self.focus_mode {
            self.focus_mode = false;
        }
        self.show_search = !self.show_search;

        if self.show_search {
            // Focus the search input
            self.search_state.update(cx, |state, cx| {
                state.focus(window, cx);
            });
        } else {
            // Clear search and refocus editor
            self.search_state.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
            self.search_query.clear();
            self.editor_state.update(cx, |state, cx| {
                state.focus(window, cx);
            });
        }

        cx.notify();
    }

    /// Toggle markdown preview mode (Cmd+Shift+P)
    pub(super) fn toggle_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.preview_enabled = !self.preview_enabled;

        if self.preview_enabled {
            // Keep focus on the NotesApp so shortcuts still work while previewing.
            self.focus_handle.focus(window, cx);
        } else {
            // Return focus to editor for editing.
            self.editor_state.update(cx, |state, cx| {
                state.focus(window, cx);
            });
        }

        cx.notify();
    }
}
