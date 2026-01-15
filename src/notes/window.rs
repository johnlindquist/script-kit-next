//! Notes Window
//!
//! A separate floating window for notes, built with gpui-component.
//! This is completely independent from the main Script Kit launcher window.

use anyhow::Result;
use gpui::{
    div, hsla, point, prelude::*, px, rgba, size, App, BoxShadow, Context, Entity, FocusHandle,
    Focusable, IntoElement, KeyDownEvent, ParentElement, Render, Styled, Subscription, Window,
    WindowBounds, WindowOptions,
};

#[cfg(target_os = "macos")]
use cocoa::appkit::NSApp;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState, Search},
    theme::ActiveTheme,
    IconName, Root, Sizable,
};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info};

// Use the unified ActionsDialog/CommandBar system
use crate::actions::{get_notes_command_bar_actions, CommandBar, CommandBarConfig, NotesInfo};
use crate::theme;

// Keep legacy types for backwards compatibility during transition
use super::actions_panel::{
    panel_height_for_rows, NotesAction, NotesActionItem, NotesActionsPanel,
};
use super::browse_panel::{BrowsePanel, NoteAction, NoteListItem};
use super::model::{ExportFormat, Note, NoteId};
use super::storage;

/// Global handle to the notes window
static NOTES_WINDOW: std::sync::OnceLock<std::sync::Mutex<Option<gpui::WindowHandle<Root>>>> =
    std::sync::OnceLock::new();

// NOTE: Theme watching is now centralized in crate::theme::service
// The per-window NOTES_THEME_WATCHER_RUNNING flag has been removed

/// View mode for the notes list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotesViewMode {
    /// Show all active notes
    #[default]
    AllNotes,
    /// Show deleted notes (trash)
    Trash,
}

/// The main notes application view
///
/// Raycast-style single-note view:
/// - No sidebar - displays one note at a time
/// - Titlebar with note title and hover-reveal action icons
/// - Auto-resize: window height grows with content
/// - Footer with type indicator and character count
pub struct NotesApp {
    /// All notes (cached from storage)
    notes: Vec<Note>,

    /// Deleted notes (for trash view)
    deleted_notes: Vec<Note>,

    /// Current view mode
    view_mode: NotesViewMode,

    /// Currently selected note ID
    selected_note_id: Option<NoteId>,

    /// Editor input state (using gpui-component's Input)
    editor_state: Entity<InputState>,

    /// Search input state (for future browse panel)
    search_state: Entity<InputState>,

    /// Current search query (for future browse panel)
    search_query: String,

    /// Whether the titlebar is being hovered (for showing/hiding icons)
    titlebar_hovered: bool,

    /// Whether the entire window is being hovered (for traffic lights)
    window_hovered: bool,

    /// Forces hover chrome for visual tests
    force_hovered: bool,

    /// Whether the formatting toolbar is pinned open
    show_format_toolbar: bool,

    /// Last known content line count for auto-resize
    last_line_count: usize,

    /// Initial window height - used as minimum for auto-resize
    initial_height: f32,

    /// Whether auto-sizing is enabled
    /// When enabled: window grows AND shrinks to fit content (min = initial_height)
    /// When disabled: window size is fixed until user re-enables via actions panel
    /// Disabled automatically when user manually resizes the window
    auto_sizing_enabled: bool,

    /// Last known window height - used to detect manual resize
    last_window_height: f32,

    /// Focus handle for keyboard navigation
    focus_handle: FocusHandle,

    /// Subscriptions to keep alive
    _subscriptions: Vec<Subscription>,

    /// Whether the actions panel is shown (Cmd+K)
    show_actions_panel: bool,

    /// Whether the browse panel is shown (Cmd+P)
    show_browse_panel: bool,

    /// Entity for the actions panel (when shown)
    actions_panel: Option<Entity<NotesActionsPanel>>,

    /// Command bar component (Cmd+K) - uses unified CommandBar wrapper
    /// Opens in a separate vibrancy window for proper macOS blur effect
    command_bar: CommandBar,

    /// Entity for the browse panel (when shown)
    browse_panel: Option<Entity<super::browse_panel::BrowsePanel>>,

    /// Pending action from actions panel clicks
    pending_action: Arc<Mutex<Option<NotesAction>>>,

    /// Previous height before showing the actions panel
    actions_panel_prev_height: Option<f32>,

    /// Cached box shadows from theme (avoid reloading theme on every render)
    cached_box_shadows: Vec<BoxShadow>,

    /// Pending note selection from browse panel
    pending_browse_select: Arc<Mutex<Option<NoteId>>>,

    /// Pending close request from browse panel
    pending_browse_close: Arc<Mutex<bool>>,

    /// Pending action from browse panel (note id + action)
    pending_browse_action: Arc<Mutex<Option<(NoteId, NoteAction)>>>,

    /// Debounce: Whether the current note has unsaved changes
    has_unsaved_changes: bool,

    /// Debounce: Last time we saved (to avoid too-frequent saves)
    last_save_time: Option<Instant>,

    /// Track last persisted bounds for debounced save on close paths
    /// (traffic light, Cmd+W, toggle) that don't go through close_notes_window
    last_persisted_bounds: Option<gpui::WindowBounds>,

    /// Last time we saved bounds (debounce to avoid too-frequent saves)
    last_bounds_save: Instant,

    /// Theme revision seen - used to detect theme changes and recompute cached values
    theme_rev_seen: u64,
}

impl NotesApp {
    /// Create a new NotesApp
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Initialize storage
        if let Err(e) = storage::init_notes_db() {
            tracing::error!(error = %e, "Failed to initialize notes database");
        }

        // Load notes from storage
        let notes = storage::get_all_notes().unwrap_or_default();
        let deleted_notes = storage::get_deleted_notes().unwrap_or_default();
        let selected_note_id = notes.first().map(|n| n.id);

        // Get initial content if we have a selected note
        let initial_content = selected_note_id
            .and_then(|id| notes.iter().find(|n| n.id == id))
            .map(|n| n.content.clone())
            .unwrap_or_default();

        // Calculate initial line count for auto-resize (before moving content)
        let initial_line_count = initial_content.lines().count().max(1);

        // Create input states - use multi_line for the editor
        let editor_state = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
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

        // Pre-compute box shadows from theme (avoid reloading on every render)
        let cached_box_shadows = Self::compute_box_shadows();

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
            force_hovered: false,
            show_format_toolbar: false,
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
            browse_panel: None,
            pending_action: Arc::new(Mutex::new(None)),
            actions_panel_prev_height: None,
            cached_box_shadows,
            pending_browse_select: Arc::new(Mutex::new(None)),
            pending_browse_close: Arc::new(Mutex::new(false)),
            pending_browse_action: Arc::new(Mutex::new(None)),
            has_unsaved_changes: false,
            last_save_time: None,
            last_persisted_bounds: None,
            last_bounds_save: Instant::now(),
            theme_rev_seen: crate::theme::service::theme_revision(),
        }
    }

    /// Debounce interval for saves (in milliseconds)
    const SAVE_DEBOUNCE_MS: u64 = 300;

    /// Debounce interval for bounds persistence (in milliseconds)
    const BOUNDS_DEBOUNCE_MS: u64 = 250;

    /// Update cached theme-derived values if theme revision has changed.
    ///
    /// This is called during render to detect theme hot-reloads and recompute
    /// values like box shadows that are derived from the theme.
    fn maybe_update_theme_cache(&mut self) {
        let current_rev = crate::theme::service::theme_revision();
        if self.theme_rev_seen != current_rev {
            self.theme_rev_seen = current_rev;
            self.cached_box_shadows = Self::compute_box_shadows();
        }
    }

    /// Persist window bounds if they've changed (debounced).
    ///
    /// This ensures bounds are saved even when the window is closed via traffic light
    /// (red close button) which doesn't go through our close handlers.
    fn maybe_persist_bounds(&mut self, window: &gpui::Window) {
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
    fn save_current_note(&mut self) {
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
    }

    /// Check if we should save now (debounce check)
    fn should_save_now(&self) -> bool {
        if !self.has_unsaved_changes {
            return false;
        }

        match self.last_save_time {
            None => true,
            Some(last_save) => last_save.elapsed() >= Duration::from_millis(Self::SAVE_DEBOUNCE_MS),
        }
    }

    /// Handle editor content changes with auto-resize
    fn on_editor_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
    fn update_window_height(
        &mut self,
        window: &mut Window,
        line_count: usize,
        _cx: &mut Context<Self>,
    ) {
        // Skip if auto-sizing is disabled (user manually resized)
        if !self.auto_sizing_enabled {
            return;
        }

        // Constants for layout calculation - adjusted for compact sticky-note style
        const TITLEBAR_HEIGHT: f32 = 32.0;
        const FOOTER_HEIGHT: f32 = 24.0;
        const PADDING: f32 = 24.0; // Top + bottom padding in editor area
        const LINE_HEIGHT: f32 = 20.0; // Approximate line height
        const MAX_HEIGHT: f32 = 600.0; // Don't grow too large

        // Use initial_height as minimum - never shrink below starting size
        let min_height = self.initial_height;

        // Calculate desired height
        let content_height = (line_count as f32) * LINE_HEIGHT;
        let total_height = TITLEBAR_HEIGHT + content_height + FOOTER_HEIGHT + PADDING;
        let clamped_height = total_height.clamp(min_height, MAX_HEIGHT);

        // Get current bounds and update height
        let current_bounds = window.bounds();
        let old_height: f32 = current_bounds.size.height.into();

        // Resize if height needs to change (both grow AND shrink)
        // Use a small threshold to avoid constant tiny adjustments
        const RESIZE_THRESHOLD: f32 = 5.0;
        if (clamped_height - old_height).abs() > RESIZE_THRESHOLD {
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
    fn detect_manual_resize(&mut self, window: &Window) {
        if !self.auto_sizing_enabled {
            return; // Already disabled
        }

        let current_height: f32 = window.bounds().size.height.into();

        // If height differs significantly from what we set, user resized manually
        const MANUAL_RESIZE_THRESHOLD: f32 = 10.0;
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

    /// Handle search query changes
    fn on_search_change(&mut self, cx: &mut Context<Self>) {
        let query = self.search_state.read(cx).value().to_string();
        self.search_query = query.clone();

        // If search is not empty, use FTS search
        if !query.trim().is_empty() {
            match storage::search_notes(&query) {
                Ok(results) => {
                    self.notes = results;
                    // Update selection if current note not in results
                    if let Some(id) = self.selected_note_id {
                        if !self.notes.iter().any(|n| n.id == id) {
                            self.selected_note_id = self.notes.first().map(|n| n.id);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Search failed");
                }
            }
        } else {
            // Reload all notes when search is cleared
            self.notes = storage::get_all_notes().unwrap_or_default();
        }

        cx.notify();
    }

    /// Create a new note
    fn create_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let note = Note::new();
        let id = note.id;

        // Save to storage
        if let Err(e) = storage::save_note(&note) {
            tracing::error!(error = %e, "Failed to create note");
            return;
        }

        // Add to cache and select it
        self.notes.insert(0, note);
        self.select_note(id, window, cx);

        info!(note_id = %id, "New note created");
    }

    /// Select a note for editing
    fn select_note(&mut self, id: NoteId, window: &mut Window, cx: &mut Context<Self>) {
        // Save any unsaved changes to the current note before switching
        self.save_current_note();

        self.selected_note_id = Some(id);

        // Load content into editor
        let note_list = if self.view_mode == NotesViewMode::Trash {
            &self.deleted_notes
        } else {
            &self.notes
        };

        if let Some(note) = note_list.iter().find(|n| n.id == id) {
            let content_len = note.content.len();
            self.editor_state.update(cx, |state, cx| {
                state.set_value(&note.content, window, cx);
                // Move cursor to end of text (set selection to end..end = no selection, cursor at end)
                state.set_selection(content_len, content_len, window, cx);
            });
        }

        // Focus the editor after selecting a note
        self.editor_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });

        cx.notify();
    }

    /// Delete the currently selected note (soft delete)
    fn delete_selected_note(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                note.soft_delete();

                if let Err(e) = storage::save_note(note) {
                    tracing::error!(error = %e, "Failed to delete note");
                }

                // Move to deleted notes
                self.deleted_notes.insert(0, note.clone());
            }

            // Remove from visible list and select next
            self.notes.retain(|n| n.id != id);
            self.selected_note_id = self.notes.first().map(|n| n.id);

            cx.notify();
        }
    }

    /// Permanently delete the selected note from trash
    fn permanently_delete_note(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            if let Err(e) = storage::delete_note_permanently(id) {
                tracing::error!(error = %e, "Failed to permanently delete note");
                return;
            }

            self.deleted_notes.retain(|n| n.id != id);
            self.selected_note_id = self.deleted_notes.first().map(|n| n.id);

            info!(note_id = %id, "Note permanently deleted");
            cx.notify();
        }
    }

    /// Restore the selected note from trash
    fn restore_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            if let Some(note) = self.deleted_notes.iter_mut().find(|n| n.id == id) {
                note.restore();

                if let Err(e) = storage::save_note(note) {
                    tracing::error!(error = %e, "Failed to restore note");
                    return;
                }

                // Move back to active notes
                self.notes.insert(0, note.clone());
            }

            self.deleted_notes.retain(|n| n.id != id);
            self.view_mode = NotesViewMode::AllNotes;
            self.selected_note_id = Some(id);
            self.select_note(id, window, cx);

            info!(note_id = %id, "Note restored");
            cx.notify();
        }
    }

    /// Switch view mode
    fn set_view_mode(&mut self, mode: NotesViewMode, window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = mode;

        // Select first note in new view
        let notes = match mode {
            NotesViewMode::AllNotes => &self.notes,
            NotesViewMode::Trash => &self.deleted_notes,
        };

        if let Some(note) = notes.first() {
            self.select_note(note.id, window, cx);
        } else {
            self.selected_note_id = None;
            self.editor_state.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
        }

        cx.notify();
    }

    /// Export the current note
    fn export_note(&self, format: ExportFormat) {
        if let Some(id) = self.selected_note_id {
            if let Some(note) = self.notes.iter().find(|n| n.id == id) {
                let content = match format {
                    ExportFormat::PlainText => note.content.clone(),
                    // For Markdown, just export the content as-is.
                    // The title is derived from the first line of content,
                    // so prepending it would cause duplication.
                    ExportFormat::Markdown => note.content.clone(),
                    ExportFormat::Html => {
                        // For HTML, we include proper structure with the title
                        // and render the content as preformatted text
                        format!(
                            "<!DOCTYPE html>\n<html>\n<head><title>{}</title></head>\n<body>\n<h1>{}</h1>\n<pre>{}</pre>\n</body>\n</html>",
                            note.title, note.title, note.content
                        )
                    }
                };

                // Copy to clipboard
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let _ = Command::new("pbcopy")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                        .and_then(|mut child| {
                            use std::io::Write;
                            if let Some(stdin) = child.stdin.as_mut() {
                                stdin.write_all(content.as_bytes())?;
                            }
                            child.wait()
                        });
                    info!(format = ?format, "Note exported to clipboard");
                }
            }
        }
    }

    /// Insert markdown formatting at cursor position
    fn insert_formatting(&mut self, prefix: &str, suffix: &str, cx: &mut Context<Self>) {
        let current = self.editor_state.read(cx).value().to_string();
        // For simplicity, append to end. A real implementation would insert at cursor.
        let formatted = format!("{}{}{}", current, prefix, suffix);
        // Note: We can't directly update with cursor position, so this is simplified
        info!(prefix = prefix, "Formatting inserted");
        let _ = formatted; // Would update editor in full implementation
        cx.notify();
    }

    /// Get filtered notes based on search query
    fn get_visible_notes(&self) -> &[Note] {
        match self.view_mode {
            NotesViewMode::AllNotes => &self.notes,
            NotesViewMode::Trash => &self.deleted_notes,
        }
    }

    /// Get the character count of the current note
    fn get_character_count(&self, cx: &Context<Self>) -> usize {
        self.editor_state.read(cx).value().chars().count()
    }

    /// Copy the current note content to clipboard
    fn copy_note_to_clipboard(&self, cx: &Context<Self>) {
        let content = self.editor_state.read(cx).value().to_string();
        self.copy_text_to_clipboard(&content);
    }

    fn copy_text_to_clipboard(&self, content: &str) {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let _ = Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin.write_all(content.as_bytes())?;
                    }
                    child.wait()
                });
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = content; // Avoid unused warning
        }
    }

    fn note_deeplink(&self, id: NoteId) -> String {
        format!("scriptkit://notes/{}", id.as_str())
    }

    fn copy_note_as_markdown(&self) {
        self.export_note(ExportFormat::Markdown);
    }

    fn copy_note_deeplink(&self) {
        if let Some(id) = self.selected_note_id {
            let deeplink = self.note_deeplink(id);
            self.copy_text_to_clipboard(&deeplink);
        }
    }

    fn create_note_quicklink(&self) {
        if let Some(id) = self.selected_note_id {
            let title = self
                .notes
                .iter()
                .find(|note| note.id == id)
                .map(|note| {
                    if note.title.is_empty() {
                        "Untitled Note".to_string()
                    } else {
                        note.title.clone()
                    }
                })
                .unwrap_or_else(|| "Untitled Note".to_string());
            let deeplink = self.note_deeplink(id);
            let quicklink = format!("[{}]({})", title, deeplink);
            self.copy_text_to_clipboard(&quicklink);
        }
    }

    fn duplicate_selected_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.selected_note_id else {
            return;
        };
        let Some(note) = self.notes.iter().find(|note| note.id == id) else {
            return;
        };

        let duplicate = Note::with_content(note.content.clone());
        if let Err(e) = storage::save_note(&duplicate) {
            tracing::error!(error = %e, "Failed to duplicate note");
            return;
        }

        self.notes.insert(0, duplicate.clone());
        self.select_note(duplicate.id, window, cx);
    }

    fn build_action_items(&self) -> Vec<NotesActionItem> {
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

    fn open_actions_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Update command bar actions based on current state (dynamic - depends on selection, etc.)
        let actions = get_notes_command_bar_actions(&NotesInfo {
            has_selection: self.selected_note_id.is_some(),
            is_trash_view: self.view_mode == NotesViewMode::Trash,
            auto_sizing_enabled: self.auto_sizing_enabled,
        });
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

    fn close_actions_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

    fn ensure_actions_panel_height(&mut self, window: &mut Window, row_count: usize) {
        const ACTIONS_PANEL_WINDOW_MARGIN: f32 = 64.0;

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

    fn restore_actions_panel_height(&mut self, window: &mut Window) {
        let Some(prev_height) = self.actions_panel_prev_height.take() else {
            return;
        };

        let current_bounds = window.bounds();
        window.resize(size(current_bounds.size.width, px(prev_height)));
        self.last_window_height = prev_height;
    }

    fn drain_pending_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
    fn drain_pending_browse_actions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
    fn handle_action(&mut self, action: NotesAction, window: &mut Window, cx: &mut Context<Self>) {
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
    fn execute_action(&mut self, action_id: &str, window: &mut Window, cx: &mut Context<Self>) {
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

    /// Open the browse panel with current notes
    fn open_browse_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Create NoteListItems from current notes
        let note_items: Vec<NoteListItem> = self
            .notes
            .iter()
            .map(|note| NoteListItem::from_note(note, Some(note.id) == self.selected_note_id))
            .collect();

        // Clone Arcs for the callbacks
        let pending_select = self.pending_browse_select.clone();
        let pending_close = self.pending_browse_close.clone();
        let pending_action = self.pending_browse_action.clone();

        let browse_panel = cx.new(|cx| {
            BrowsePanel::new(note_items, window, cx)
                .on_select(move |id| {
                    if let Ok(mut guard) = pending_select.lock() {
                        *guard = Some(id);
                    }
                })
                .on_close({
                    let pending_close = pending_close.clone();
                    move || {
                        if let Ok(mut guard) = pending_close.lock() {
                            *guard = true;
                        }
                    }
                })
                .on_action(move |id, action| {
                    if let Ok(mut guard) = pending_action.lock() {
                        *guard = Some((id, action));
                    }
                })
        });

        // Focus the browse panel and its search input
        let panel_focus_handle = browse_panel.read(cx).focus_handle(cx);
        window.focus(&panel_focus_handle, cx);

        // Focus the search input so user can start typing immediately
        browse_panel.update(cx, |panel, cx| {
            panel.focus_search(window, cx);
        });

        self.browse_panel = Some(browse_panel);
        cx.notify();
    }

    /// Handle note selection from browse panel
    fn handle_browse_select(&mut self, id: NoteId, window: &mut Window, cx: &mut Context<Self>) {
        self.show_browse_panel = false;
        self.browse_panel = None;
        // select_note already focuses the editor
        self.select_note(id, window, cx);
        cx.notify();
    }

    /// Handle note action from browse panel
    fn handle_browse_action(&mut self, id: NoteId, action: NoteAction, cx: &mut Context<Self>) {
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

    /// Close the browse panel and refocus the editor
    fn close_browse_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.show_browse_panel = false;
        self.browse_panel = None;

        // Refocus the editor after closing the browse panel
        self.editor_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });

        cx.notify();
    }

    /// Render the search input
    fn render_search(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .px_2()
            .py_1()
            .child(Input::new(&self.search_state).w_full().small())
    }

    /// Render the formatting toolbar
    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_1()
            .py_1()
            .child(
                Button::new("bold")
                    .ghost()
                    .xsmall()
                    .label("B")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("**", "**", cx);
                    })),
            )
            .child(
                Button::new("italic")
                    .ghost()
                    .xsmall()
                    .label("I")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("_", "_", cx);
                    })),
            )
            .child(
                Button::new("heading")
                    .ghost()
                    .xsmall()
                    .label("H")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("\n## ", "", cx);
                    })),
            )
            .child(
                Button::new("list")
                    .ghost()
                    .xsmall()
                    .label("•")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("\n- ", "", cx);
                    })),
            )
            .child(
                Button::new("code")
                    .ghost()
                    .xsmall()
                    .label("</>")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("`", "`", cx);
                    })),
            )
            .child(
                Button::new("codeblock")
                    .ghost()
                    .xsmall()
                    .label("```")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("\n```\n", "\n```", cx);
                    })),
            )
            .child(
                Button::new("link")
                    .ghost()
                    .xsmall()
                    .label("🔗")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.insert_formatting("[", "](url)", cx);
                    })),
            )
    }

    /// Render the export menu
    fn render_export_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .gap_1()
            .child(
                Button::new("export-txt")
                    .ghost()
                    .xsmall()
                    .label("TXT")
                    .on_click(cx.listener(|this, _, _, _cx| {
                        this.export_note(ExportFormat::PlainText);
                    })),
            )
            .child(
                Button::new("export-md")
                    .ghost()
                    .xsmall()
                    .label("MD")
                    .on_click(cx.listener(|this, _, _, _cx| {
                        this.export_note(ExportFormat::Markdown);
                    })),
            )
            .child(
                Button::new("export-html")
                    .ghost()
                    .xsmall()
                    .label("HTML")
                    .on_click(cx.listener(|this, _, _, _cx| {
                        this.export_note(ExportFormat::Html);
                    })),
            )
    }

    // Note: Sidebar removed for Raycast-style single-note view.
    // Browse panel (Cmd+P) will be implemented as a separate overlay in the future.

    /// Render the main editor area with Raycast-style clean UI
    fn render_editor(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_trash = self.view_mode == NotesViewMode::Trash;
        let has_selection = self.selected_note_id.is_some();
        let show_toolbar = self.show_format_toolbar;
        let char_count = self.get_character_count(cx);

        // Get note title - This reads from self.notes which is updated by on_editor_change
        // The title is extracted from the first line of content via Note::set_content()
        let title = self
            .selected_note_id
            .and_then(|id| self.get_visible_notes().iter().find(|n| n.id == id))
            .map(|n| {
                if n.title.is_empty() {
                    "Untitled Note".to_string()
                } else {
                    n.title.clone()
                }
            })
            .unwrap_or_else(|| {
                if is_trash {
                    "No deleted notes".to_string()
                } else {
                    "No note selected".to_string()
                }
            });

        // Raycast-style: titlebar only visible on hover, centered title, right-aligned actions
        let window_hovered = self.window_hovered || self.force_hovered;

        // Get muted foreground color for subtle icons/text
        let muted_color = cx.theme().muted_foreground;

        let titlebar = div()
            .id("notes-titlebar")
            .flex()
            .items_center()
            .justify_center() // Center the title
            .h(px(26.)) // Compact titlebar height (slightly taller for visual balance)
            .px_3()
            .relative() // For absolute positioning of icons
            // NO .bg() - let vibrancy show through from root
            // Only show titlebar elements when window is hovered
            .on_hover(cx.listener(|this, hovered, _, cx| {
                if this.force_hovered {
                    return;
                }

                this.titlebar_hovered = *hovered;
                cx.notify();
            }))
            .child(
                // Note title (truncated) - CENTERED in titlebar
                div()
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_sm()
                    .text_color(muted_color) // Use muted color for subtle title
                    .when(!window_hovered, |d| d.opacity(0.))
                    .child(title),
            )
            // Conditionally show icons based on state - only when window is hovered
            // Raycast-style: icons on the right - settings (actions), panel (browse), + (new)
            // Use absolute positioning to keep title centered
            // Note: "+" and "≡" icons should show even with no notes (so users can create their first note)
            // The "⌘" (actions) icon only shows when a note is selected (needs a note to act on)
            .when(window_hovered && !is_trash, |d| {
                d.child(
                    div()
                        .absolute()
                        .right_3() // Align to right with same padding as px_3
                        .flex()
                        .items_center()
                        .gap_2() // Even spacing between icons
                        // Icon 1: Command key icon - opens actions panel (⌘K)
                        // Only show when a note is selected (actions require a note)
                        .when(has_selection, |d| {
                            d.child(
                                div()
                                    .id("titlebar-cmd-icon")
                                    .text_sm()
                                    .text_color(muted_color.opacity(0.7)) // Muted, subtle icon
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(muted_color)) // Slightly brighter on hover
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        if this.show_actions_panel {
                                            this.close_actions_panel(window, cx);
                                        } else {
                                            this.open_actions_panel(window, cx);
                                        }
                                    }))
                                    .child("⌘"),
                            )
                        })
                        // Icon 2: List icon - for browsing notes (always visible when hovered)
                        .child(
                            div()
                                .id("titlebar-browse-icon")
                                .text_sm()
                                .text_color(muted_color.opacity(0.7)) // Muted, subtle icon
                                .cursor_pointer()
                                .hover(|s| s.text_color(muted_color)) // Slightly brighter on hover
                                .on_click(cx.listener(|this, _, window, cx| {
                                    if this.show_browse_panel {
                                        this.close_browse_panel(window, cx);
                                    } else {
                                        this.close_actions_panel(window, cx);
                                        this.show_browse_panel = true;
                                        this.open_browse_panel(window, cx);
                                    }
                                }))
                                .child("≡"),
                        )
                        // Icon 3: Plus icon - for new note (always visible when hovered)
                        .child(
                            div()
                                .id("titlebar-new-icon")
                                .text_sm()
                                .text_color(muted_color.opacity(0.7)) // Muted, subtle icon
                                .cursor_pointer()
                                .hover(|s| s.text_color(muted_color)) // Slightly brighter on hover
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.create_note(window, cx);
                                }))
                                .child("+"),
                        ),
                )
            })
            .when(has_selection && is_trash, |d| {
                // Trash actions (always visible)
                d.child(
                    div()
                        .flex()
                        .gap_1()
                        .child(
                            Button::new("restore")
                                .ghost()
                                .xsmall()
                                .label("Restore")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.restore_note(window, cx);
                                })),
                        )
                        .child(
                            Button::new("permanent-delete")
                                .ghost()
                                .xsmall()
                                .icon(IconName::Delete)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.permanently_delete_note(cx);
                                })),
                        ),
                )
            });

        // Build character count footer - only visible on hover
        // Raycast style: character count CENTERED, T icon on RIGHT
        // NOTE: No .bg() - let vibrancy show through from root
        let footer = div()
            .flex()
            .items_center()
            .justify_center()
            .relative()
            .h(px(24.))
            .px_3()
            // NO .bg() - let vibrancy show through from root
            // Hide when window not hovered
            .when(!window_hovered, |d| d.opacity(0.))
            .child(
                // Character count CENTERED (Raycast style)
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "{} character{}",
                        char_count,
                        if char_count == 1 { "" } else { "s" }
                    )),
            )
            .child(
                // Type indicator (T for text) on RIGHT (Raycast style)
                div()
                    .absolute()
                    .right_3()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("T"),
            );

        // Build main editor layout - Raycast style: clean, no visible input borders
        // NOTE: Do NOT add .bg() here - the notes-window-root and gpui-component Root
        // already provide the vibrancy background. Adding more semi-transparent backgrounds
        // would compound opacity (0.37 × 0.30 × 0.30 = ~8% transparency left!)
        div()
            .flex_1()
            .flex()
            .flex_col()
            .h_full()
            // NO .bg() - let vibrancy show through from root
            .child(titlebar)
            // Toolbar hidden by default - only show when pinned
            .when(!is_trash && has_selection && show_toolbar, |d| {
                d.child(self.render_toolbar(cx))
            })
            .child(
                div()
                    .flex_1()
                    .p_3()
                    // NO .bg() - let vibrancy show through from root
                    // Use a styled input that blends with background
                    .child(
                        Input::new(&self.editor_state).h_full().appearance(false), // No input styling - blends with background
                    ),
            )
            .when(has_selection && !is_trash, |d| d.child(footer))
    }

    /// Render the actions panel overlay (Cmd+K)
    ///
    /// IMPORTANT: Uses items_start + fixed top padding to keep the search input
    /// at a stable position. Without this, the panel would re-center when items
    /// are filtered out, causing the search input to jump around.
    fn render_actions_panel_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let panel = self
            .actions_panel
            .as_ref()
            .map(|panel| panel.clone().into_any_element())
            .unwrap_or_else(|| div().into_any_element());

        // Fixed top offset so search input stays at same position regardless of item count
        const PANEL_TOP_OFFSET: f32 = 32.0;

        div()
            .id("actions-panel-overlay")
            .absolute()
            .inset_0()
            .bg(gpui::rgba(0x00000080))
            .flex()
            .flex_col()
            .items_center() // Horizontally centered
            .justify_start() // Vertically aligned to top (not centered!)
            .pt(px(PANEL_TOP_OFFSET)) // Fixed offset from top
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.close_actions_panel(window, cx);
                }),
            )
            .child(
                div()
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
                        // Stop propagation - don't close when clicking panel
                    })
                    .child(panel),
            )
    }

    /// Render the browse panel overlay (Cmd+P)
    fn render_browse_panel_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // If we have a browse panel entity, render it
        // Otherwise render an empty container that will close on click
        if let Some(ref browse_panel) = self.browse_panel {
            div()
                .id("browse-panel-overlay")
                .absolute()
                .inset_0()
                .child(browse_panel.clone())
        } else {
            // Fallback: create inline browse panel
            let note_items: Vec<NoteListItem> = self
                .notes
                .iter()
                .map(|note| NoteListItem::from_note(note, Some(note.id) == self.selected_note_id))
                .collect();

            // We need a simple inline version since we can't create entities in render
            div()
                .id("browse-panel-overlay")
                .absolute()
                .inset_0()
                .bg(gpui::rgba(0x00000080))
                .flex()
                .items_center()
                .justify_center()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.close_browse_panel(window, cx);
                }))
                .child(
                    div()
                        .w(px(500.))
                        .max_h(px(400.))
                        .bg(Self::get_vibrancy_background(cx)) // Semi-transparent for vibrancy
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded_lg()
                        .shadow_lg()
                        .p_4()
                        .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
                            // Stop propagation
                        })
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{} notes available", note_items.len())),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .mt_2()
                                .child("Press Escape to close"),
                        ),
                )
        }
    }

    /// Get cached box shadows (computed once at construction)
    fn create_box_shadows(&self) -> Vec<BoxShadow> {
        self.cached_box_shadows.clone()
    }

    // =====================================================
    // Vibrancy Helper Functions
    // =====================================================
    // These use the same approach as the main window (render_script_list.rs)
    // to ensure vibrancy works correctly by using rgba() with hex colors
    // directly from the Script Kit theme.
    // NOTE: hex_to_rgba_with_opacity moved to crate::ui_foundation (centralized)

    /// Get background color with vibrancy opacity applied
    ///
    /// When vibrancy is enabled, backgrounds need to be semi-transparent
    /// to show the blur effect behind them. This helper returns the
    /// theme background color with the appropriate opacity from config.
    fn get_vibrancy_background(_cx: &Context<Self>) -> gpui::Rgba {
        let sk_theme = crate::theme::load_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.main;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.main,
        ))
    }

    /// Get title bar background with vibrancy opacity
    fn get_vibrancy_title_bar_background(_cx: &Context<Self>) -> gpui::Rgba {
        let sk_theme = crate::theme::load_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.title_bar;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.title_bar,
        ))
    }

    /// Get sidebar/panel background with vibrancy opacity
    fn get_vibrancy_sidebar_background() -> gpui::Rgba {
        let sk_theme = crate::theme::load_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.title_bar;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.title_bar,
        ))
    }

    /// Compute box shadows from theme configuration (called once at construction)
    fn compute_box_shadows() -> Vec<BoxShadow> {
        let theme = crate::theme::load_theme();
        let shadow_config = theme.get_drop_shadow();

        if !shadow_config.enabled {
            return vec![];
        }

        // Convert hex color to HSLA
        let r = ((shadow_config.color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((shadow_config.color >> 8) & 0xFF) as f32 / 255.0;
        let b = (shadow_config.color & 0xFF) as f32 / 255.0;

        // Simple RGB to HSL conversion
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        let (h, s) = if max == min {
            (0.0, 0.0)
        } else {
            let d = max - min;
            let s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };
            let h = if max == r {
                (g - b) / d + if g < b { 6.0 } else { 0.0 }
            } else if max == g {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            };
            (h / 6.0, s)
        };

        vec![BoxShadow {
            color: hsla(h, s, l, shadow_config.opacity),
            offset: point(px(shadow_config.offset_x), px(shadow_config.offset_y)),
            blur_radius: px(shadow_config.blur_radius),
            spread_radius: px(shadow_config.spread_radius),
        }]
    }

    /// Update cached box shadows when theme changes
    pub fn update_theme(&mut self, _cx: &mut Context<Self>) {
        self.cached_box_shadows = Self::compute_box_shadows();
    }
}

impl Focusable for NotesApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Drop for NotesApp {
    fn drop(&mut self) {
        // Save any unsaved changes before closing
        if self.has_unsaved_changes {
            if let Some(id) = self.selected_note_id {
                if let Some(note) = self.notes.iter().find(|n| n.id == id) {
                    if let Err(e) = storage::save_note(note) {
                        tracing::error!(error = %e, "Failed to save note on close");
                    } else {
                        debug!(note_id = %id, "Note saved on window close");
                    }
                }
            }
        }

        // Clear the global window handle when NotesApp is dropped
        // This ensures is_notes_window_open() returns false after the window closes
        // regardless of how it was closed (Cmd+W, traffic light, toggle, etc.)
        if let Some(window_handle) = NOTES_WINDOW.get() {
            if let Ok(mut guard) = window_handle.lock() {
                *guard = None;
                debug!("NotesApp dropped - cleared global window handle");
            }
        }
    }
}

impl Render for NotesApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Detect if user manually resized the window (disables auto-sizing)
        self.detect_manual_resize(window);
        self.drain_pending_action(window, cx);
        self.drain_pending_browse_actions(window, cx);

        // Update cached theme values if theme has changed (hot-reload)
        self.maybe_update_theme_cache();

        // Persist bounds on change (ensures bounds saved even on traffic light close)
        self.maybe_persist_bounds(window);

        // Debounced save: check if we should save now
        if self.should_save_now() {
            self.save_current_note();
        }

        // Only show legacy actions panel overlay if it exists AND command bar is NOT open
        // CommandBar renders in its own window, so we don't need the overlay
        let show_actions =
            self.show_actions_panel && self.actions_panel.is_some() && !self.command_bar.is_open();
        let show_browse = self.show_browse_panel;

        // Raycast-style single-note view: no sidebar, editor fills full width
        // Track window hover for traffic lights visibility
        let box_shadows = self.create_box_shadows();

        // NOTE: Do NOT add .bg() here - gpui-component Root already provides
        // .bg(cx.theme().background) with vibrancy alpha. Adding another semi-transparent
        // background would compound opacity and hide the blur effect.
        div()
            .id("notes-window-root")
            .flex()
            .flex_col()
            .size_full()
            .relative()
            // NO .bg() - gpui-component Root provides vibrancy background
            .shadow(box_shadows)
            .text_color(cx.theme().foreground)
            .track_focus(&self.focus_handle)
            // Close command bar when clicking anywhere on the notes window
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    if this.command_bar.is_open() {
                        this.close_actions_panel(window, cx);
                    }
                }),
            )
            // Track window hover for showing/hiding chrome
            .on_hover(cx.listener(|this, hovered, _, cx| {
                if this.force_hovered {
                    return;
                }

                this.window_hovered = *hovered;
                cx.notify();
            }))
            // CRITICAL: Use capture_key_down to intercept keys BEFORE Input component handles them
            // This ensures arrow keys go to CommandBar navigation instead of Input cursor movement
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                // Handle keyboard shortcuts
                let key = event.keystroke.key.to_lowercase();
                let modifiers = &event.keystroke.modifiers;

                // Handle command bar navigation when it's open
                // This routes all relevant keys to the CommandBar
                // CRITICAL: Must stop propagation to prevent Input from consuming the keys
                if this.command_bar.is_open() {
                    match key.as_str() {
                        "escape" | "esc" => {
                            this.close_actions_panel(window, cx);
                            cx.stop_propagation();
                            return;
                        }
                        "up" | "arrowup" => {
                            this.command_bar.select_prev(cx);
                            cx.stop_propagation();
                            return;
                        }
                        "down" | "arrowdown" => {
                            this.command_bar.select_next(cx);
                            cx.stop_propagation();
                            return;
                        }
                        "enter" => {
                            if let Some(action_id) = this.command_bar.execute_selected_action(cx) {
                                this.execute_action(&action_id, window, cx);
                            }
                            cx.stop_propagation();
                            return;
                        }
                        "backspace" | "delete" => {
                            this.command_bar.handle_backspace(cx);
                            cx.stop_propagation();
                            return;
                        }
                        _ => {
                            // Handle printable characters for search (when no modifiers)
                            if !modifiers.platform && !modifiers.control && !modifiers.alt {
                                if let Some(ch) = key.chars().next() {
                                    if ch.is_alphanumeric()
                                        || ch.is_whitespace()
                                        || ch == '-'
                                        || ch == '_'
                                    {
                                        this.command_bar.handle_char(ch, cx);
                                        cx.stop_propagation();
                                        return;
                                    }
                                }
                            }
                            // Cmd+K also closes the command bar
                            if modifiers.platform && key == "k" {
                                this.close_actions_panel(window, cx);
                                cx.stop_propagation();
                                return;
                            }
                        }
                    }
                    // Don't fall through to other handlers when command bar is open
                    return;
                }

                // Legacy: Handle old actions panel (for backwards compatibility during transition)
                if this.show_actions_panel && this.actions_panel.is_some() {
                    if key == "escape" || (modifiers.platform && key == "k") || key == "esc" {
                        this.close_actions_panel(window, cx);
                        return;
                    }

                    if let Some(ref panel) = this.actions_panel {
                        match key.as_str() {
                            "up" | "arrowup" => {
                                panel.update(cx, |panel, cx| panel.move_up(cx));
                            }
                            "down" | "arrowdown" => {
                                panel.update(cx, |panel, cx| panel.move_down(cx));
                            }
                            "enter" => {
                                if let Some(action) = panel.read(cx).get_selected_action() {
                                    this.handle_action(action, window, cx);
                                }
                            }
                            "backspace" => {
                                panel.update(cx, |panel, cx| panel.handle_backspace(cx));
                            }
                            _ => {
                                if let Some(ref key_char) = event.keystroke.key_char {
                                    if let Some(ch) = key_char.chars().next() {
                                        if !ch.is_control() {
                                            panel.update(cx, |panel, cx| {
                                                panel.handle_char(ch, cx);
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }

                    return;
                }

                // Handle browse panel keyboard events
                if this.show_browse_panel {
                    if key == "escape" || (modifiers.platform && key == "p") || key == "esc" {
                        this.close_browse_panel(window, cx);
                        return;
                    }

                    if let Some(ref panel) = this.browse_panel {
                        match key.as_str() {
                            "up" | "arrowup" => {
                                panel.update(cx, |panel, cx| panel.move_up(cx));
                            }
                            "down" | "arrowdown" => {
                                panel.update(cx, |panel, cx| panel.move_down(cx));
                            }
                            "enter" => {
                                if let Some(id) = panel.read(cx).get_selected_note_id() {
                                    this.handle_browse_select(id, window, cx);
                                }
                            }
                            _ => {
                                // Let BrowsePanel handle other keys (like search input)
                            }
                        }
                    }

                    return;
                }

                // Handle Escape to close panels
                if key == "escape" {
                    if this.show_actions_panel {
                        this.close_actions_panel(window, cx);
                        return;
                    }
                    if this.show_browse_panel {
                        this.close_browse_panel(window, cx);
                        return;
                    }
                }

                // platform modifier = Cmd on macOS, Ctrl on Windows/Linux
                if modifiers.platform {
                    match key.as_str() {
                        "k" => {
                            // Toggle actions panel / command bar
                            if this.command_bar.is_open() || this.show_actions_panel {
                                this.close_actions_panel(window, cx);
                            } else {
                                this.open_actions_panel(window, cx);
                            }
                        }
                        "p" => {
                            // Toggle browse panel
                            this.show_browse_panel = !this.show_browse_panel;
                            this.close_actions_panel(window, cx);
                            if this.show_browse_panel {
                                this.open_browse_panel(window, cx);
                            } else {
                                this.browse_panel = None;
                            }
                            cx.notify();
                        }
                        "n" => this.create_note(window, cx),
                        "w" => {
                            // Close the notes window (standard macOS pattern)
                            // Save bounds before closing
                            let wb = window.window_bounds();
                            crate::window_state::save_window_from_gpui(
                                crate::window_state::WindowRole::Notes,
                                wb,
                            );
                            window.remove_window();
                        }
                        "d" => this.duplicate_selected_note(window, cx),
                        "b" => this.insert_formatting("**", "**", cx),
                        "i" => this.insert_formatting("_", "_", cx),
                        _ => {}
                    }
                }
            }))
            // Single note view - editor takes full width
            .child(self.render_editor(cx))
            // Overlay panels
            .when(show_actions, |d| {
                d.child(self.render_actions_panel_overlay(cx))
            })
            .when(show_browse, |d| {
                d.child(self.render_browse_panel_overlay(cx))
            })
    }
}

/// Sync Script Kit theme with gpui-component theme
/// NOTE: Do NOT call gpui_component::init here - it's already called in main.rs
/// and calling it again resets the theme to system defaults (opaque backgrounds),
/// which breaks vibrancy.
fn ensure_theme_initialized(cx: &mut App) {
    // Just sync our theme colors - gpui_component is already initialized in main.rs
    crate::theme::sync_gpui_component_theme(cx);

    info!("Notes window theme synchronized with Script Kit");
}

/// Calculate window bounds positioned in the top-right corner of the display containing the mouse.
fn calculate_top_right_bounds(width: f32, height: f32, padding: f32) -> gpui::Bounds<gpui::Pixels> {
    use crate::platform::{get_global_mouse_position, get_macos_displays};

    let displays = get_macos_displays();

    // Find display containing mouse
    let target_display = if let Some((mouse_x, mouse_y)) = get_global_mouse_position() {
        displays
            .iter()
            .find(|display| {
                mouse_x >= display.origin_x
                    && mouse_x < display.origin_x + display.width
                    && mouse_y >= display.origin_y
                    && mouse_y < display.origin_y + display.height
            })
            .cloned()
    } else {
        None
    };

    // Use found display or fall back to primary
    let display = target_display.or_else(|| displays.first().cloned());

    if let Some(display) = display {
        // Position in top-right corner with padding
        let x = display.origin_x + display.width - width as f64 - padding as f64;
        let y = display.origin_y + padding as f64;

        gpui::Bounds::new(
            gpui::Point::new(px(x as f32), px(y as f32)),
            gpui::Size::new(px(width), px(height)),
        )
    } else {
        // Fallback to centered on primary
        gpui::Bounds::new(
            gpui::Point::new(px(100.0), px(100.0)),
            gpui::Size::new(px(width), px(height)),
        )
    }
}

/// Toggle the notes window (open if closed, close if open)
pub fn open_notes_window(cx: &mut App) -> Result<()> {
    use crate::logging;

    logging::log("PANEL", "open_notes_window called - checking toggle state");

    // Ensure gpui-component theme is initialized before opening window
    ensure_theme_initialized(cx);

    // SAFETY: Release lock BEFORE calling handle.update() to prevent deadlock.
    // We clone the handle (it's just an ID) and release the lock immediately.
    let existing_handle = {
        let slot = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    // Check if window already exists and is valid
    if let Some(handle) = existing_handle {
        // Window exists - check if it's valid and close it (toggle OFF)
        // Lock is released, safe to call handle.update()
        if handle
            .update(cx, |_, window, _cx| {
                // Save bounds before closing (fixes bounds persistence on toggle close)
                let wb = window.window_bounds();
                crate::window_state::save_window_from_gpui(
                    crate::window_state::WindowRole::Notes,
                    wb,
                );
                window.remove_window();
            })
            .is_ok()
        {
            logging::log("PANEL", "Notes window was open - closing (toggle OFF)");
            // Clear the stored handle
            let slot = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
            if let Ok(mut g) = slot.lock() {
                *g = None;
            }

            // NOTE: We intentionally do NOT call cx.hide() here.
            // Closing Notes should not affect the main window's ability to be shown.
            // The main window hotkey handles its own visibility state.
            // If the user wants to hide everything, they can press the main hotkey
            // when the main window is visible.

            return Ok(());
        }
        // Window handle was invalid, fall through to create new window
        logging::log("PANEL", "Notes window handle was invalid - creating new");
    }

    // If main window is visible, hide it (Notes takes focus)
    // Use platform::hide_main_window() to only hide the main window, not the whole app
    // IMPORTANT: Set visibility to false so the main hotkey knows to SHOW (not hide) next time
    if crate::is_main_window_visible() {
        logging::log(
            "PANEL",
            "Main window was visible - hiding it since Notes is opening",
        );
        crate::set_main_window_visible(false);
        crate::platform::hide_main_window();
    }

    // Create new window (toggle ON)
    logging::log("PANEL", "Notes window not open - creating new (toggle ON)");
    info!("Opening new notes window");

    // Calculate position: try saved position first, then top-right default
    let window_width = 350.0_f32;
    let window_height = 280.0_f32;
    let padding = 20.0_f32; // Padding from screen edges

    let default_bounds = calculate_top_right_bounds(window_width, window_height, padding);
    let displays = crate::platform::get_macos_displays();
    let bounds = crate::window_state::get_initial_bounds(
        crate::window_state::WindowRole::Notes,
        default_bounds,
        &displays,
    );

    // Load theme to determine window background appearance (vibrancy)
    let theme = crate::theme::load_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(gpui::TitlebarOptions {
            title: Some("Notes".into()),
            appears_transparent: true,
            traffic_light_position: Some(gpui::Point {
                x: px(8.),
                y: px(7.), // Centered vertically in 26px header
            }),
        }),
        window_background,
        focus: true,
        show: true,
        // Use PopUp for floating panel behavior - allows keyboard input without
        // activating the app (Raycast-like). Creates NSPanel with NonactivatingPanel mask.
        kind: gpui::WindowKind::PopUp,
        ..Default::default()
    };

    // Store the NotesApp entity so we can focus it after window creation
    let notes_app_holder: std::sync::Arc<std::sync::Mutex<Option<Entity<NotesApp>>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    let notes_app_for_closure = notes_app_holder.clone();

    let handle = cx.open_window(window_options, |window, cx| {
        let view = cx.new(|cx| NotesApp::new(window, cx));
        *notes_app_for_closure.lock().unwrap() = Some(view.clone());
        cx.new(|cx| Root::new(view, window, cx))
    })?;

    // NOTE: We do NOT call cx.activate(true) here!
    // Notes is a PopUp window (NSPanel with NonactivatingPanel style), which means
    // it can receive keyboard input without activating the application.
    // Calling activate(true) would bring ALL windows forward (including main window),
    // causing a flash before we could hide it.
    //
    // Instead, we just ensure the main window is hidden (in case it was visible)
    // and let the PopUp window handle focus naturally.
    crate::platform::hide_main_window();

    // Focus the editor input in the Notes window
    // Release lock before calling update
    let notes_app_entity = notes_app_holder.lock().ok().and_then(|mut g| g.take());
    if let Some(notes_app) = notes_app_entity {
        let _ = handle.update(cx, |_root, window, cx| {
            window.activate_window();

            // Focus the NotesApp's editor input and move cursor to end
            notes_app.update(cx, |app, cx| {
                // Get content length for cursor positioning
                let content_len = app.editor_state.read(cx).value().len();

                // Call the InputState's focus method and move cursor to end
                app.editor_state.update(cx, |state, inner_cx| {
                    state.focus(window, inner_cx);
                    // Move cursor to end of text (same as select_note behavior)
                    state.set_selection(content_len, content_len, window, inner_cx);
                });

                if std::env::var("SCRIPT_KIT_TEST_NOTES_HOVERED")
                    .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
                    .unwrap_or(false)
                {
                    app.force_hovered = true;
                    app.window_hovered = true;
                    app.titlebar_hovered = true;
                }

                if std::env::var("SCRIPT_KIT_TEST_NOTES_ACTIONS_PANEL")
                    .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
                    .unwrap_or(false)
                {
                    app.open_actions_panel(window, cx);
                }

                cx.notify();
            });
        });
    }

    // Store the window handle (release lock immediately)
    {
        let slot = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        if let Ok(mut g) = slot.lock() {
            *g = Some(handle);
        }
    }

    // Configure as floating panel (always on top) after window is created
    configure_notes_as_floating_panel();

    // NOTE: Theme hot-reload is now handled by the centralized ThemeService
    // (crate::theme::service::ensure_theme_service) which is started once at app init.
    // This eliminates per-window theme watcher tasks and their potential for leaks.

    Ok(())
}

/// Quick capture - open notes with a new note ready for input
pub fn quick_capture(cx: &mut App) -> Result<()> {
    open_notes_window(cx)?;

    // TODO: Focus the editor and optionally create a new note
    // This requires accessing the NotesApp through the Root wrapper

    Ok(())
}

/// Close the notes window
pub fn close_notes_window(cx: &mut App) {
    // SAFETY: Release lock BEFORE calling handle.update() to prevent deadlock
    // If handle.update() causes Drop to fire synchronously and tries to acquire
    // the same lock, we would deadlock. Taking the handle out first avoids this.
    let handle = {
        let slot = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|mut g| g.take())
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_, window, _| {
            // Save window bounds before closing
            let wb = window.window_bounds();
            crate::window_state::save_window_from_gpui(crate::window_state::WindowRole::Notes, wb);
            window.remove_window();
        });
    }
}

/// Check if the notes window is currently open
///
/// Returns true if the Notes window exists and is valid.
/// This is used by other parts of the app to check if Notes is open
/// without affecting it.
pub fn is_notes_window_open() -> bool {
    let window_handle = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let guard = window_handle.lock().unwrap();
    guard.is_some()
}

/// Configure the Notes window as a floating panel (always on top).
///
/// This sets:
/// - NSFloatingWindowLevel (3) - floats above normal windows
/// - NSWindowCollectionBehaviorMoveToActiveSpace - moves to current space when shown
/// - Disabled window restoration - prevents macOS position caching
#[cfg(target_os = "macos")]
fn configure_notes_as_floating_panel() {
    use crate::logging;
    use std::ffi::CStr;

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex: i];
            let title: id = msg_send![window, title];

            if title != nil {
                let title_cstr: *const i8 = msg_send![title, UTF8String];
                if !title_cstr.is_null() {
                    let title_str = CStr::from_ptr(title_cstr).to_string_lossy();

                    if title_str == "Notes" {
                        // Found the Notes window - configure it

                        // NSFloatingWindowLevel = 3
                        // Use i64 (NSInteger) for proper ABI compatibility on 64-bit macOS
                        let floating_level: i64 = 3;
                        let _: () = msg_send![window, setLevel:floating_level];

                        // Get current collection behavior to preserve existing flags
                        let current: u64 = msg_send![window, collectionBehavior];

                        // Check if window has CanJoinAllSpaces (set by GPUI for PopUp windows)
                        // If so, we can't add MoveToActiveSpace (they're mutually exclusive)
                        let has_can_join_all_spaces = (current & 1) != 0;

                        // OR in FullScreenAuxiliary (256) + IgnoresCycle (64)
                        // IgnoresCycle excludes Notes from Cmd+Tab - it's a utility window
                        // MoveToActiveSpace (2) only if not already CanJoinAllSpaces
                        let desired: u64 = if has_can_join_all_spaces {
                            current | 256 | 64
                        } else {
                            current | 2 | 256 | 64
                        };
                        let _: () = msg_send![window, setCollectionBehavior:desired];

                        // Ensure window content is shareable for captureScreenshot()
                        let sharing_type: i64 = 1; // NSWindowSharingReadOnly
                        let _: () = msg_send![window, setSharingType:sharing_type];

                        // Disable window restoration
                        let _: () = msg_send![window, setRestorable:false];

                        // Disable close/hide animation for instant dismiss (NSWindowAnimationBehaviorNone = 2)
                        let _: () = msg_send![window, setAnimationBehavior: 2i64];

                        // ═══════════════════════════════════════════════════════════════════════════
                        // VIBRANCY CONFIGURATION - Match main window for consistent blur
                        // ═══════════════════════════════════════════════════════════════════════════
                        crate::platform::configure_secondary_window_vibrancy(window, "Notes");

                        // Log detailed breakdown of collection behavior bits
                        let has_can_join = (desired & 1) != 0;
                        let has_ignores = (desired & 64) != 0;
                        let has_move_to_active = (desired & 2) != 0;

                        logging::log(
                            "PANEL",
                            &format!(
                                "Notes window: behavior={}->{} [CanJoinAllSpaces={}, IgnoresCycle={}, MoveToActiveSpace={}]",
                                current, desired, has_can_join, has_ignores, has_move_to_active
                            ),
                        );
                        logging::log(
                            "PANEL",
                            "Notes window: Will NOT appear in Cmd+Tab app switcher (floating utility panel)",
                        );
                        return;
                    }
                }
            }
        }

        logging::log(
            "PANEL",
            "Warning: Notes window not found by title for floating panel config",
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_notes_as_floating_panel() {
    // No-op on non-macOS platforms
}
