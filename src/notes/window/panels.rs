use itertools::Itertools;

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
                    | NotesAction::Format
                    | NotesAction::DeleteNote
                    | NotesAction::SendToAi => can_edit,
                    NotesAction::RestoreNote | NotesAction::PermanentlyDeleteNote => {
                        has_selection && is_trash
                    }
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
            actions.iter().take(5).map(|a| a.title.as_str()).join(", ")
        );

        self.command_bar.set_actions(actions, cx);

        // Open the command bar (CommandBar handles window creation internally)
        self.command_bar.open_centered(window, cx);

        // Update state flags (before focus request so current_focus_surface() reflects the new state)
        self.show_actions_panel = true;
        self.show_browse_panel = false;
        self.browse_panel = None;

        // Route through NotesFocusSurface for structured logging and consistent focus management.
        // The ActionsWindow is a visual-only popup — it does NOT take keyboard focus.
        self.request_focus_surface(focus::NotesFocusSurface::ActionsPanel, window, cx);
    }

    pub(super) fn close_actions_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Close the command bar window
        self.command_bar.close(cx);

        self.show_actions_panel = false;
        self.actions_panel = None;

        // Route through NotesFocusSurface for structured logging and consistent focus management.
        self.request_focus_surface(focus::NotesFocusSurface::Editor, window, cx);
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
                window.dispatch_action(Box::new(Search), cx);
                return; // Early return - already handled focus
            }
            NotesAction::CopyNoteAs => self.copy_note_as_markdown(cx),
            NotesAction::CopyDeeplink => self.copy_note_deeplink(cx),
            NotesAction::CreateQuicklink => self.create_note_quicklink(cx),
            NotesAction::Export => self.export_note(ExportFormat::Html, cx),
            NotesAction::DeleteNote => {
                self.close_actions_panel(window, cx);
                self.request_delete_selected_note(window, cx);
                return;
            }
            NotesAction::RestoreNote => self.restore_note(window, cx),
            NotesAction::PermanentlyDeleteNote => self.permanently_delete_note(window, cx),
            NotesAction::MoveListItemUp | NotesAction::MoveListItemDown => {}
            NotesAction::Format => {
                self.show_format_toolbar = !self.show_format_toolbar;
            }
            NotesAction::EnableAutoSizing => {
                self.enable_auto_sizing(window, cx);
            }
            NotesAction::SendToAi => {
                let opened =
                    self.open_selected_note_cart_in_embedded_acp("NotesAction::SendToAi", cx);
                self.close_actions_panel(window, cx);
                if opened {
                    self.show_action_feedback("Staged in ACP Chat", false);
                }
                return;
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
        info!(action_id, "Executing notes action from CommandBar");

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
            "delete_note" => Some(NotesAction::DeleteNote),
            "restore_note" => Some(NotesAction::RestoreNote),
            "permanently_delete_note" => Some(NotesAction::PermanentlyDeleteNote),
            "enable_auto_sizing" => Some(NotesAction::EnableAutoSizing),
            "send_to_ai" => Some(NotesAction::SendToAi),
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
                if self.replace_active_note_mention_with_note(note_id, window, cx) {
                    return;
                }
                self.close_browse_panel(window, cx);
                self.select_note(note_id, window, cx);
                return;
            }

            tracing::warn!(
                action_id,
                note_id_str,
                selected_note_id = ?self.selected_note_id,
                notes_len = self.notes.len(),
                "notes_note_switcher_selected_note_not_found",
            );
            self.show_selected_note_missing_feedback("execute_note_switcher_action", cx);
            self.close_browse_panel(window, cx);
            return;
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

        let note_count = self.notes.len();
        self.note_switcher.set_actions(note_switcher_actions, cx);

        // Open the note switcher (CommandBar handles window creation internally)
        self.note_switcher.open_centered(window, cx);

        // Set note count as context title header (after open, which creates the dialog)
        if let Some(dialog) = self.note_switcher.dialog() {
            let title = format!(
                "{} note{}",
                note_count,
                if note_count == 1 { "" } else { "s" }
            );
            dialog.update(cx, |d, _cx| {
                d.set_context_title(Some(title));
            });
        }

        // Update state flags (before focus request so current_focus_surface() reflects the new state)
        self.show_browse_panel = true;
        self.show_actions_panel = false;
        self.browse_panel = None; // Clear legacy browse panel

        // Route through NotesFocusSurface for structured logging and consistent focus management.
        // The ActionsWindow is a visual-only popup — it does NOT take keyboard focus.
        self.request_focus_surface(focus::NotesFocusSurface::BrowsePanel, window, cx);
    }

    /// Handle note selection from browse panel
    pub(super) fn handle_browse_select(
        &mut self,
        id: NoteId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.replace_active_note_mention_with_note(id, window, cx) {
            return;
        }
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
                // Soft-delete from browse panel (doesn't switch editor)
                if let Some(idx) = self.notes.iter().position(|n| n.id == id) {
                    let mut note = self.notes.remove(idx);
                    note.soft_delete();
                    if let Err(e) = storage::save_note(&note) {
                        tracing::error!(error = %e, "Failed to delete note");
                    }
                    self.deleted_notes.insert(0, note);
                }
                // If we deleted the currently-selected note, move selection
                if self.selected_note_id == Some(id) {
                    self.selected_note_id = self.notes.first().map(|n| n.id);
                }
                self.show_action_feedback("Deleted · ⌘⇧T trash", false);
                cx.notify();
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
        self.mention_portal_edit = None;

        // Route through NotesFocusSurface for structured logging and consistent focus management.
        self.request_focus_surface(focus::NotesFocusSurface::Editor, window, cx);
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

    /// Build a canonical ACP target for the currently selected note or
    /// unsaved draft content.
    pub(super) fn build_selected_note_target_for_ai(
        &self,
        cx: &Context<Self>,
    ) -> Option<crate::ai::TabAiTargetContext> {
        let content = self.editor_state.read(cx).value().to_string();
        let selected_note = self
            .selected_note_id
            .and_then(|selected_id| self.notes.iter().find(|note| note.id == selected_id));

        // Require either a saved note or non-empty draft content.
        if selected_note.is_none() && content.trim().is_empty() {
            return None;
        }

        let selected_note_id = selected_note.map(|note| note.id.as_str().to_string());
        let semantic_note_id = selected_note_id
            .clone()
            .unwrap_or_else(|| "draft".to_string());

        let title = selected_note
            .map(|note| note.title.trim().to_string())
            .filter(|title| !title.is_empty())
            .unwrap_or_else(|| "Untitled Note".to_string());

        let preview = selected_note
            .map(|note| Self::strip_markdown_for_preview(&note.preview()))
            .unwrap_or_else(|| Self::strip_markdown_for_preview(&content));

        let is_pinned = selected_note.map(|note| note.is_pinned).unwrap_or(false);

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "notes_selected_note_target_built",
            note_id = %semantic_note_id,
            title = %title,
            content_len = content.len(),
            has_saved_note = selected_note_id.is_some(),
        );

        Some(crate::ai::TabAiTargetContext {
            source: "Notes".to_string(),
            kind: "note".to_string(),
            semantic_id: crate::protocol::generate_semantic_id("note", 0, &semantic_note_id),
            label: title.clone(),
            metadata: Some(serde_json::json!({
                "noteId": selected_note_id,
                "title": title,
                "content": content,
                "preview": preview,
                "isPinned": is_pinned,
                "viewMode": format!("{:?}", self.view_mode),
            })),
        })
    }

    /// Stage a canonical note target as a `FocusedTarget` chip into the
    /// Notes-hosted embedded ACP thread.
    fn stage_note_target_in_embedded_acp(
        &mut self,
        source: &'static str,
        target: crate::ai::TabAiTargetContext,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let label = crate::ai::format_explicit_target_chip_label(&target);
        let semantic_id = target.semantic_id.clone();

        let Some(entity) = self.embedded_acp_chat.as_ref().cloned() else {
            return Err("Notes ACP chat entity unavailable".to_string());
        };

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "notes_embedded_acp_target_staged",
            source,
            semantic_id = %semantic_id,
            label = %label,
        );

        entity
            .update(cx, move |chat, cx| {
                if chat.is_setup_mode() {
                    return Err("ACP Chat is in setup mode".to_string());
                }

                // Reused embedded ACP sessions must not keep stale note text
                // in the composer.
                chat.set_input(String::new(), cx);

                chat.live_thread().update(cx, move |thread, cx| {
                    thread.add_context_part(
                        crate::ai::message_parts::AiContextPart::FocusedTarget { target, label },
                        cx,
                    );
                });

                Ok::<(), String>(())
            })
            .map_err(|error| error.to_string())?;

        Ok(())
    }

    pub(crate) fn build_note_text_part_for_ai(
        title: &str,
        semantic_note_id: &str,
        content: &str,
        selection: std::ops::Range<usize>,
    ) -> Option<crate::ai::message_parts::AiContextPart> {
        if selection.start < selection.end && selection.end <= content.len() {
            return Some(crate::ai::message_parts::AiContextPart::TextBlock {
                label: "Selected Text".to_string(),
                source: format!(
                    "notes://{}#selection={}-{}",
                    semantic_note_id, selection.start, selection.end
                ),
                text: content[selection.clone()].to_string(),
                mime_type: Some("text/markdown".to_string()),
            });
        }

        if content.trim().is_empty() {
            return None;
        }

        Some(crate::ai::message_parts::AiContextPart::TextBlock {
            label: title.to_string(),
            source: format!("notes://{}", semantic_note_id),
            text: content.to_string(),
            mime_type: Some("text/markdown".to_string()),
        })
    }

    /// Build a full cart payload for the selected note: the selected note text
    /// as the first `TextBlock` (selection first, otherwise full note body),
    /// then persisted `NoteCartItem`s in sort_order.
    ///
    /// Returns an ordered `Vec<AiContextPart>` ready to stage onto the
    /// Notes-hosted ACP surface as inline `@mentions`.
    pub(super) fn build_selected_note_cart_parts_for_ai(
        &self,
        cx: &Context<Self>,
    ) -> Result<Vec<crate::ai::message_parts::AiContextPart>, String> {
        let editor_state = self.editor_state.read(cx);
        let content = editor_state.value().to_string();
        let selection = editor_state.selection();
        let selected_note = self
            .selected_note_id
            .and_then(|id| self.notes.iter().find(|n| n.id == id));
        let has_selection = selection.start < selection.end;

        // Require either a saved note or non-empty draft content.
        if selected_note.is_none() && content.trim().is_empty() {
            return Err("No note selected and no draft content".to_string());
        }

        let selected_note_id = selected_note.map(|n| n.id.as_str().to_string());
        let semantic_note_id = selected_note_id
            .clone()
            .unwrap_or_else(|| "draft".to_string());
        let title = selected_note
            .map(|n| n.title.trim().to_string())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| "Untitled Note".to_string());

        let mut parts = Vec::new();

        // 1. Selected text as the first TextBlock part, otherwise the full note body.
        if let Some(part) = Self::build_note_text_part_for_ai(
            &title,
            &semantic_note_id,
            &content,
            selection.clone(),
        ) {
            parts.push(part);
        }

        // 2. Persisted cart items in sort_order.
        if let Some(note_id) = selected_note.map(|n| n.id) {
            match crate::notes::storage::list_note_cart_items(note_id) {
                Ok(items) => {
                    for item in &items {
                        parts.push(item.to_ai_context_part());
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        target: "script_kit::tab_ai",
                        event = "notes_cart_items_load_failed",
                        note_id = %note_id,
                        error = %err,
                    );
                }
            }
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "notes_cart_parts_built",
            note_id = %semantic_note_id,
            part_count = parts.len(),
            selected = has_selection,
        );

        Ok(parts)
    }

    /// Open the Notes-hosted ACP surface with the full cart payload from the
    /// selected note.
    ///
    /// Returns `true` if the handoff was initiated, `false` on failure.
    pub(super) fn open_selected_note_cart_in_embedded_acp(
        &mut self,
        source: &'static str,
        cx: &mut Context<Self>,
    ) -> bool {
        let parts = match self.build_selected_note_cart_parts_for_ai(cx) {
            Ok(p) if p.is_empty() => {
                self.show_action_feedback("Nothing to send", true);
                return false;
            }
            Ok(p) => p,
            Err(err) => {
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "notes_cart_handoff_skipped",
                    reason = %err,
                );
                self.show_action_feedback("No note selected", true);
                return false;
            }
        };

        let part_count = parts.len();

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "notes_cart_open_embedded_acp_requested",
            source,
            item_count = part_count,
        );

        if self.embedded_acp_chat.is_none() {
            let requirements = crate::ai::acp::preflight::AcpLaunchRequirements::default();
            let view = match crate::ai::acp::hosted::spawn_hosted_view(None, requirements, cx) {
                Ok(view) => view,
                Err(err) => {
                    tracing::warn!(
                        target: "script_kit::tab_ai",
                        event = "notes_cart_handoff_open_failed",
                        error = %err,
                    );
                    self.show_action_feedback("ACP unavailable", true);
                    return false;
                }
            };
            let notes_entity = cx.entity().downgrade();
            let toggle_entity = notes_entity.clone();
            let close_entity = notes_entity.clone();
            let history_entity = notes_entity;

            view.update(cx, |chat, _cx| {
                chat.set_footer_host(crate::ai::acp::view::AcpFooterHost::External);

                chat.set_on_toggle_actions(move |window, cx| {
                    if let Some(entity) = toggle_entity.upgrade() {
                        entity.update(cx, |app, cx| {
                            app.toggle_acp_actions(window, cx);
                        });
                    }
                });

                chat.set_on_close_requested(move |window, cx| {
                    if let Some(entity) = close_entity.upgrade() {
                        entity.update(cx, |app, cx| {
                            app.switch_to_notes_surface(window, cx);
                        });
                    }
                });

                chat.set_on_open_history_command(move |window, cx| {
                    if let Some(entity) = history_entity.upgrade() {
                        entity.update(cx, |app, cx| {
                            let _ = app.open_embedded_acp_history_popup(window, cx);
                        });
                    }
                });
            });
            self.embedded_acp_chat = Some(view);
        }

        self.surface_mode = NotesSurfaceMode::Acp;
        self.pending_focus_surface = Some(focus::NotesFocusSurface::AcpChat);
        cx.notify();

        let Some(entity) = self.embedded_acp_chat.as_ref().cloned() else {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "notes_cart_handoff_missing_embedded_chat",
            );
            self.show_action_feedback("ACP unavailable", true);
            return false;
        };

        let stage_result = entity.update(cx, move |chat, cx| {
            if chat.is_setup_mode() {
                return Err("ACP Chat is in setup mode".to_string());
            }

            chat.set_input(String::new(), cx);

            let current_text = chat.live_thread().read(cx).input.text().to_string();
            let mut staged_text = current_text;
            let mut staged_aliases = Vec::new();

            for part in parts {
                let inline_token = crate::ai::context_mentions::part_to_inline_token(&part)
                    .unwrap_or_else(|| format!("@{}", part.label()));
                if !staged_text.is_empty() && !staged_text.ends_with(' ') {
                    staged_text.push(' ');
                }
                staged_text.push_str(&inline_token);
                staged_text.push(' ');
                staged_aliases.push((inline_token, part));
            }

            chat.live_thread().update(cx, |thread, cx| {
                for (_, part) in &staged_aliases {
                    thread.add_context_part(part.clone(), cx);
                }
                thread.input.set_text(staged_text.clone());
                thread.input.set_cursor(staged_text.len());
                cx.notify();
            });

            for (inline_token, part) in staged_aliases {
                chat.register_typed_alias(inline_token.clone(), part);
                chat.register_inline_owned_token(inline_token);
            }

            Ok::<(), String>(())
        });

        if let Err(err) = stage_result {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "notes_cart_handoff_stage_failed",
                error = %err,
            );
            self.show_action_feedback("ACP unavailable", true);
            return false;
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "notes_cart_open_embedded_acp_completed",
            source,
            item_count = part_count,
        );

        true
    }

    /// Open the Notes-hosted embedded ACP surface and stage the selected
    /// note (or unsaved draft) as a canonical `FocusedTarget` chip.
    ///
    /// Returns `true` if the embedded ACP was opened and the target staged,
    /// `false` if no note was selected or the ACP surface could not open.
    pub(super) fn open_selected_note_in_embedded_acp(
        &mut self,
        source: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(target) = self.build_selected_note_target_for_ai(cx) else {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "notes_embedded_acp_switch_skipped",
                source,
                reason = "no_selected_note",
            );
            self.show_action_feedback("No note selected", true);
            return false;
        };

        let reused_existing_session = self.embedded_acp_chat.is_some();

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "notes_embedded_acp_switch_requested",
            source,
            reused_existing_session,
            semantic_id = %target.semantic_id,
        );

        let open_result = if reused_existing_session {
            self.relaunch_embedded_acp(None, window, cx)
        } else {
            self.open_or_focus_embedded_acp(None, window, cx)
        };

        match open_result {
            Ok(()) => {
                if let Err(error) = self.stage_note_target_in_embedded_acp(source, target, cx) {
                    tracing::warn!(
                        target: "script_kit::tab_ai",
                        event = "notes_embedded_acp_target_stage_failed",
                        source,
                        error = %error,
                    );
                    self.show_action_feedback("ACP unavailable", true);
                    return false;
                }

                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "notes_embedded_acp_switch_completed",
                    source,
                    reused_existing_session,
                );
                true
            }
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::tab_ai",
                    event = "notes_embedded_acp_switch_failed",
                    source,
                    error = %error,
                );
                self.show_action_feedback("ACP unavailable", true);
                false
            }
        }
    }
}
