//! Notes Window
//!
//! A separate floating window for notes, built with gpui-component.
//! This is completely independent from the main Script Kit launcher window.

use anyhow::Result;
use gpui::{
    div, prelude::*, px, rgba, size, AnyElement, App, Context, CursorStyle, Entity, FocusHandle,
    Focusable, IntoElement, KeyDownEvent, MouseMoveEvent, ParentElement, Render, Styled,
    Subscription, Window, WindowBounds, WindowOptions,
};

#[cfg(target_os = "macos")]
use cocoa::appkit::NSApp;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState, Search},
    kbd::Kbd,
    scroll::ScrollableElement,
    theme::ActiveTheme,
    tooltip::Tooltip,
    IconName, Root, Sizable,
};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info};

// Use the unified ActionsDialog/CommandBar system
use crate::actions::{
    get_note_switcher_actions, get_notes_command_bar_actions, CommandBar, CommandBarConfig,
    NoteSwitcherNoteInfo, NotesInfo,
};
use crate::theme;

// Keep legacy types for backwards compatibility during transition
use super::actions_panel::{
    panel_height_for_rows, NotesAction, NotesActionItem, NotesActionsPanel,
};
// Note: BrowsePanel is no longer used - note switcher now uses CommandBar
// Keeping NoteAction and NoteListItem for backwards compatibility during transition
use super::browse_panel::{NoteAction, NoteListItem};
use super::markdown;
use super::markdown_highlighting::register_markdown_highlighter;
use super::model::{ExportFormat, Note, NoteId};
use super::storage;

/// Global handle to the notes window
static NOTES_WINDOW: std::sync::OnceLock<std::sync::Mutex<Option<gpui::WindowHandle<Root>>>> =
    std::sync::OnceLock::new();

/// Global handle to the NotesApp entity for quick_capture access
static NOTES_APP_ENTITY: std::sync::OnceLock<std::sync::Mutex<Option<Entity<NotesApp>>>> =
    std::sync::OnceLock::new();

// NOTE: Theme watching is now centralized in crate::theme::service
// The per-window NOTES_THEME_WATCHER_RUNNING flag has been removed

// =============================================================================
// Layout constants — all on the 4 px micro-grid / 8 px base grid
// =============================================================================

/// Titlebar height — 36 px gives comfortable room for macOS traffic lights.
const TITLEBAR_HEIGHT: f32 = 36.0;

/// Footer / status-bar height — 28 px keeps it compact while readable.
const FOOTER_HEIGHT: f32 = 28.0;

/// Keyboard-shortcuts overlay width.
const SHORTCUTS_PANEL_WIDTH: f32 = 310.0;

/// Keyboard-shortcuts overlay max height.
const SHORTCUTS_PANEL_MAX_HEIGHT: f32 = 480.0;

/// Browse-panel inline fallback width.
const BROWSE_PANEL_WIDTH: f32 = 500.0;

/// Browse-panel inline fallback max height.
const BROWSE_PANEL_MAX_HEIGHT: f32 = 400.0;

/// Actions-panel overlay top offset (keeps search input stable).
const ACTIONS_PANEL_TOP_OFFSET: f32 = 32.0;

// =============================================================================
// Semantic opacity levels — keeps contrast consistent and reduces magic numbers.
//
//   OPACITY_DISABLED  → 0.4  (disabled / inactive — floor for legibility)
//   OPACITY_SUBTLE    → 0.5  (rest-state chrome, de-emphasized metadata)
//   OPACITY_MUTED     → 0.7  (secondary info that should still be easy to read)
//   OPACITY_VISIBLE   → 1.0  (full emphasis)
//
// When applied to `muted_foreground`, the lowest tier (0.4) still exceeds
// WCAG 2.2 AA 3:1 non-text contrast on both dark and light surfaces.
// =============================================================================

/// Opacity for disabled / completely de-emphasized elements.
const OPACITY_DISABLED: f32 = 0.4;

/// Opacity for rest-state chrome (unhovered footer, subtle metadata).
const OPACITY_SUBTLE: f32 = 0.5;

/// Opacity for secondary information (icons, timestamps, line info).
const OPACITY_MUTED: f32 = 0.7;

/// Minimum interactive target size (px) — WCAG 2.2 §2.5.8 (24 × 24 CSS px).
const MIN_TARGET_SIZE: f32 = 24.0;

// =============================================================================
// Derived / contextual opacity tokens — extracted from inline magic numbers
// so every transparency in this file is auditable in one place.
// =============================================================================

/// Near-opaque overlay background (e.g. keyboard-shortcuts help sheet).
const OPACITY_OVERLAY_BG: f32 = 0.96;

/// Very subtle border dividers inside overlays (section separators).
const OPACITY_SECTION_BORDER: f32 = 0.2;

/// Accent-tinted border for trash-view indicator.
const OPACITY_ACCENT_BORDER: f32 = 0.35;

/// Width reserved for macOS traffic-light buttons in the titlebar (px).
const TITLEBAR_TRAFFIC_LIGHT_W: f32 = 60.0;

/// Width reserved for the right-side icon cluster in the titlebar (px).
const TITLEBAR_ICONS_W: f32 = 100.0;

/// Footer separator: Unicode middle-dot used between stats in the footer.
const FOOTER_SEP: &str = " · ";

/// View mode for the notes list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotesViewMode {
    /// Show all active notes
    #[default]
    AllNotes,
    /// Show deleted notes (trash)
    Trash,
}

/// Sort mode for the notes list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotesSortMode {
    /// Sort by last updated (most recent first) — default
    #[default]
    Updated,
    /// Sort by creation date (newest first)
    Created,
    /// Sort alphabetically by title (A→Z)
    Alphabetical,
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

    /// Whether the mouse cursor is currently hidden
    mouse_cursor_hidden: bool,

    /// Forces hover chrome for visual tests
    force_hovered: bool,

    /// Whether the formatting toolbar is pinned open
    show_format_toolbar: bool,

    /// Whether the search bar is shown (Cmd+F)
    show_search: bool,

    /// Whether markdown preview is enabled (Cmd+Shift+P)
    preview_enabled: bool,

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

    /// Note switcher command bar (Cmd+P) - uses unified CommandBar wrapper
    /// Opens in a separate vibrancy window for proper macOS blur effect
    /// This replaces the legacy BrowsePanel for consistent theming and behavior
    note_switcher: CommandBar,

    /// Entity for the browse panel (when shown) - LEGACY, kept for backwards compatibility
    /// Will be removed once note_switcher is fully tested
    browse_panel: Option<Entity<super::browse_panel::BrowsePanel>>,

    /// Pending action from actions panel clicks
    pending_action: Arc<Mutex<Option<NotesAction>>>,

    /// Previous height before showing the actions panel
    actions_panel_prev_height: Option<f32>,

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

    /// History stack for back navigation (Cmd+[)
    /// Stores previously viewed note IDs (most recent at the end)
    history_back: Vec<NoteId>,

    /// History stack for forward navigation (Cmd+])
    /// Populated when user navigates back
    history_forward: Vec<NoteId>,

    /// Flag to suppress history push during back/forward navigation
    navigating_history: bool,

    /// Whether focus mode is enabled (Cmd+.) — hides all chrome for distraction-free writing
    focus_mode: bool,

    /// Current sort mode for notes list
    sort_mode: NotesSortMode,

    /// Whether the keyboard shortcuts help overlay is shown (Cmd+/)
    show_shortcuts_help: bool,

    /// Instant when the last save completed — used for brief "Saved" flash in footer
    last_save_confirmed: Option<Instant>,

    /// Brief action feedback message shown in footer (e.g. "Deleted", "Pinned", "Duplicated")
    /// Tuple of (message, accent_colored, timestamp). Clears after 2 seconds.
    action_feedback: Option<(String, bool, Instant)>,
}

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
    fn maybe_update_theme_cache(&mut self) {
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
        self.last_save_confirmed = Some(Instant::now());
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

        // Layout constants — reuse module-level TITLEBAR_HEIGHT / FOOTER_HEIGHT
        // to avoid divergence.  PADDING and LINE_HEIGHT are auto-resize-specific.
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

    /// Create a new note pre-filled with system clipboard content (Cmd+Shift+N)
    fn create_note_from_clipboard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let clipboard_content = Self::read_clipboard();
        if clipboard_content.is_empty() {
            // Nothing on clipboard, just create an empty note
            self.create_note(window, cx);
            return;
        }

        let note = Note::with_content(clipboard_content);
        let id = note.id;

        if let Err(e) = storage::save_note(&note) {
            tracing::error!(error = %e, "Failed to create note from clipboard");
            return;
        }

        self.notes.insert(0, note);
        self.select_note(id, window, cx);

        info!(note_id = %id, "New note created from clipboard");
    }

    /// Read text from system clipboard
    fn read_clipboard() -> String {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("pbpaste")
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout).ok()
                    } else {
                        None
                    }
                })
                .unwrap_or_default()
        }
        #[cfg(not(target_os = "macos"))]
        {
            String::new()
        }
    }

    /// Select a note for editing
    fn select_note(&mut self, id: NoteId, window: &mut Window, cx: &mut Context<Self>) {
        // Save any unsaved changes to the current note before switching
        self.save_current_note();

        // Push current note onto history stack (unless navigating back/forward)
        if !self.navigating_history {
            if let Some(prev_id) = self.selected_note_id {
                if prev_id != id {
                    self.history_back.push(prev_id);
                    // Clear forward history on new navigation
                    self.history_forward.clear();
                }
            }
        }

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

            self.show_action_feedback("Deleted · ⌘⇧T trash", false);
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

    /// Compute replacement text and resulting selection for formatting insertion.
    fn formatting_replacement(
        value: &str,
        selection: Range<usize>,
        prefix: &str,
        suffix: &str,
    ) -> (String, Range<usize>) {
        let mut start = selection.start.min(value.len());
        let mut end = selection.end.min(value.len());
        if start > end {
            std::mem::swap(&mut start, &mut end);
        }

        debug_assert!(value.is_char_boundary(start));
        debug_assert!(value.is_char_boundary(end));

        let selected_text = if start == end { "" } else { &value[start..end] };

        let replacement = format!("{}{}{}", prefix, selected_text, suffix);
        let selection_start = start + prefix.len();
        let selection_end = if selected_text.is_empty() {
            selection_start
        } else {
            selection_start + selected_text.len()
        };

        (replacement, selection_start..selection_end)
    }

    /// Insert markdown formatting at cursor position
    ///
    /// Inserts prefix+suffix at cursor. If text is selected, it gets replaced
    /// with prefix+suffix via the replace() method.
    fn insert_formatting(
        &mut self,
        prefix: &str,
        suffix: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Get current cursor position before modification
        let current_value = self.editor_state.read(cx).value().to_string();

        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let (replacement, new_selection) =
                Self::formatting_replacement(&value, selection, prefix, suffix);

            state.replace(&replacement, window, cx);
            state.set_selection(new_selection.start, new_selection.end, window, cx);
        });

        // Trigger change detection for autosave
        self.has_unsaved_changes = true;
        let _ = current_value; // Prevent unused variable warning

        info!(prefix = prefix, "Formatting inserted");
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

    /// Get the word count of the current note
    fn get_word_count(&self, cx: &Context<Self>) -> usize {
        self.editor_state
            .read(cx)
            .value()
            .split_whitespace()
            .count()
    }

    /// Get the 1-based index position of the current note in the visible list
    /// Returns (current_position, total_count) or None if no note selected
    fn get_note_position(&self) -> Option<(usize, usize)> {
        let notes = self.get_visible_notes();
        let total = notes.len();
        if total == 0 {
            return None;
        }
        self.selected_note_id.and_then(|id| {
            notes
                .iter()
                .position(|n| n.id == id)
                .map(|idx| (idx + 1, total))
        })
    }

    /// Get the 1-based line number at cursor position, plus total line count
    fn get_cursor_line_info(&self, cx: &Context<Self>) -> Option<(usize, usize)> {
        let value = self.editor_state.read(cx).value().to_string();
        if value.is_empty() {
            return None;
        }
        let selection = self.editor_state.read(cx).selection();
        let cursor = selection.start.min(value.len());
        let current_line = value[..cursor].matches('\n').count() + 1;
        let total_lines = value.lines().count().max(1);
        Some((current_line, total_lines))
    }

    /// Check if the currently selected note is pinned
    fn is_current_note_pinned(&self) -> bool {
        self.selected_note_id
            .and_then(|id| self.get_visible_notes().iter().find(|n| n.id == id))
            .map(|n| n.is_pinned)
            .unwrap_or(false)
    }

    /// Navigate to the previous note in the list
    fn select_prev_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let notes = self.get_visible_notes();
        if notes.is_empty() {
            return;
        }
        if let Some(id) = self.selected_note_id {
            if let Some(idx) = notes.iter().position(|n| n.id == id) {
                if idx > 0 {
                    let prev_id = notes[idx - 1].id;
                    self.select_note(prev_id, window, cx);
                }
            }
        }
    }

    /// Navigate to the next note in the list
    fn select_next_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let notes = self.get_visible_notes();
        if notes.is_empty() {
            return;
        }
        if let Some(id) = self.selected_note_id {
            if let Some(idx) = notes.iter().position(|n| n.id == id) {
                if idx + 1 < notes.len() {
                    let next_id = notes[idx + 1].id;
                    self.select_note(next_id, window, cx);
                }
            }
        }
    }

    /// Jump to the first note in the list (Cmd+Shift+Up)
    fn select_first_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let notes = self.get_visible_notes();
        if let Some(note) = notes.first() {
            let id = note.id;
            self.select_note(id, window, cx);
        }
    }

    /// Jump to the last note in the list (Cmd+Shift+Down)
    fn select_last_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let notes = self.get_visible_notes();
        if let Some(note) = notes.last() {
            let id = note.id;
            self.select_note(id, window, cx);
        }
    }

    /// Navigate back in history (Cmd+[)
    fn navigate_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(prev_id) = self.history_back.pop() {
            // Only navigate if the note still exists
            if self.notes.iter().any(|n| n.id == prev_id) {
                // Push current note onto forward stack
                if let Some(current_id) = self.selected_note_id {
                    self.history_forward.push(current_id);
                }
                self.navigating_history = true;
                self.select_note(prev_id, window, cx);
                self.navigating_history = false;
            }
        }
    }

    /// Navigate forward in history (Cmd+])
    fn navigate_forward(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(next_id) = self.history_forward.pop() {
            // Only navigate if the note still exists
            if self.notes.iter().any(|n| n.id == next_id) {
                // Push current note onto back stack
                if let Some(current_id) = self.selected_note_id {
                    self.history_back.push(current_id);
                }
                self.navigating_history = true;
                self.select_note(next_id, window, cx);
                self.navigating_history = false;
            }
        }
    }

    /// Toggle pin state of the currently selected note (Cmd+Shift+I)
    fn toggle_pin_current_note(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            let mut was_pinned = false;
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                note.is_pinned = !note.is_pinned;
                let pinned = note.is_pinned;
                was_pinned = pinned;
                if let Err(e) = storage::save_note(note) {
                    tracing::error!(error = %e, "Failed to toggle pin state");
                    return;
                }
                info!(note_id = %id, pinned = pinned, "Toggled pin state");
            }
            // Re-sort notes: pinned first, then by updated_at descending
            self.notes.sort_by(|a, b| match (a.is_pinned, b.is_pinned) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.updated_at.cmp(&a.updated_at),
            });
            self.show_action_feedback(if was_pinned { "● Pinned" } else { "Unpinned" }, was_pinned);
            cx.notify();
        }
    }

    /// Get relative time description for when a note was last updated
    fn get_relative_time(&self) -> Option<String> {
        self.selected_note_id
            .and_then(|id| self.get_visible_notes().iter().find(|n| n.id == id))
            .map(|note| {
                let now = chrono::Utc::now();
                let diff = now - note.updated_at;

                if diff.num_seconds() < 5 {
                    "just now".to_string()
                } else if diff.num_seconds() < 60 {
                    format!("{}s ago", diff.num_seconds())
                } else if diff.num_minutes() < 60 {
                    let mins = diff.num_minutes();
                    format!("{}m ago", mins)
                } else if diff.num_hours() < 24 {
                    let hours = diff.num_hours();
                    format!("{}h ago", hours)
                } else if diff.num_days() < 7 {
                    let days = diff.num_days();
                    format!("{}d ago", days)
                } else {
                    note.updated_at.format("%b %d").to_string()
                }
            })
    }

    /// Select a pinned note by its ordinal position (Cmd+1 through Cmd+9)
    fn select_pinned_note_by_index(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let pinned_notes: Vec<NoteId> = self
            .notes
            .iter()
            .filter(|n| n.is_pinned)
            .map(|n| n.id)
            .collect();

        if let Some(&note_id) = pinned_notes.get(index) {
            self.select_note(note_id, window, cx);
        }
    }

    /// Toggle focus mode (Cmd+.) — hides titlebar icons, footer, toolbar for distraction-free writing
    fn toggle_focus_mode(&mut self, cx: &mut Context<Self>) {
        self.focus_mode = !self.focus_mode;
        if self.focus_mode {
            // Also hide search and formatting toolbar in focus mode
            self.show_search = false;
            self.show_format_toolbar = false;
        }
        info!(focus_mode = self.focus_mode, "Toggled focus mode");
        cx.notify();
    }

    /// Get estimated reading time in minutes based on word count (200 wpm average)
    fn get_reading_time(&self, cx: &Context<Self>) -> String {
        let words = self.get_word_count(cx);
        if words < 30 {
            return String::new(); // Too short for meaningful estimate
        }
        let minutes = (words as f64 / 200.0).ceil() as usize;
        if minutes <= 1 {
            "~1 min read".to_string()
        } else {
            format!("~{} min read", minutes)
        }
    }

    /// Get the selected text range stats, if any text is selected
    /// Returns (selected_words, selected_chars) or None if no selection
    fn get_selection_stats(&self, cx: &Context<Self>) -> Option<(usize, usize)> {
        let selection = self.editor_state.read(cx).selection();
        if selection.start == selection.end {
            return None;
        }
        let value = self.editor_state.read(cx).value().to_string();
        let start = selection.start.min(value.len());
        let end = selection.end.min(value.len());
        let selected_text = &value[start..end];
        let words = selected_text.split_whitespace().count();
        let chars = selected_text.chars().count();
        if chars == 0 {
            return None;
        }
        Some((words, chars))
    }

    /// Format a DateTime as a relative time string for the note switcher
    fn format_relative_time(dt: chrono::DateTime<chrono::Utc>) -> String {
        let now = chrono::Utc::now();
        let diff = now - dt;

        if diff.num_seconds() < 5 {
            "just now".to_string()
        } else if diff.num_seconds() < 60 {
            format!("{}s ago", diff.num_seconds())
        } else if diff.num_minutes() < 60 {
            format!("{}m ago", diff.num_minutes())
        } else if diff.num_hours() < 24 {
            format!("{}h ago", diff.num_hours())
        } else if diff.num_days() < 7 {
            format!("{}d ago", diff.num_days())
        } else {
            dt.format("%b %d").to_string()
        }
    }

    /// Strip markdown syntax from a preview string for clean display in the note switcher
    fn strip_markdown_for_preview(s: &str) -> String {
        let mut result = s.to_string();
        // Strip common markdown inline formatting
        result = result.replace("**", "");
        result = result.replace("__", "");
        result = result.replace("~~", "");
        // Strip heading markers
        while result.starts_with('#') {
            result = result.trim_start_matches('#').to_string();
        }
        // Strip list markers and blockquotes
        result = result
            .lines()
            .map(|line| {
                let trimmed = line.trim_start();
                if let Some(rest) = trimmed
                    .strip_prefix("- [ ] ")
                    .or_else(|| trimmed.strip_prefix("- [x] "))
                {
                    rest
                } else if let Some(rest) = trimmed.strip_prefix("- ") {
                    rest
                } else if let Some(rest) = trimmed.strip_prefix("> ") {
                    rest
                } else {
                    trimmed
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        // Collapse whitespace
        result
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    /// Welcome note content for first-time users.
    /// Teaches markdown syntax and key shortcuts through the product itself.
    fn welcome_note_content() -> String {
        [
            "# Welcome to Notes",
            "",
            "A fast, keyboard-first notes app with markdown support.",
            "",
            "## Formatting",
            "",
            "- **Bold** with ⌘B",
            "- *Italic* with ⌘I",
            "- `Code` with ⌘E",
            "- ~~Strikethrough~~ with ⌘⇧X",
            "",
            "## Lists",
            "",
            "- [ ] Checklist item (⌘⇧L)",
            "- Bullet point (⌘⇧8)",
            "1. Numbered list (⌘⇧7)",
            "",
            "## Quick shortcuts",
            "",
            "- ⌘N  new note",
            "- ⌘P  switch notes",
            "- ⌘K  actions",
            "- ⌘.  focus mode",
            "- ⌘/  all shortcuts",
            "",
            "Start typing to make this note your own!",
        ]
        .join("\n")
    }

    /// Show a brief action feedback message in the footer (auto-clears after 2s)
    /// If `accent` is true, the message renders in accent color; otherwise muted.
    fn show_action_feedback(&mut self, msg: impl Into<String>, accent: bool) {
        self.action_feedback = Some((msg.into(), accent, Instant::now()));
    }

    /// Check if action feedback should still be visible (within 2s window)
    fn get_action_feedback(&self) -> Option<(&str, bool)> {
        self.action_feedback.as_ref().and_then(|(msg, accent, t)| {
            if t.elapsed() < Duration::from_secs(2) {
                Some((msg.as_str(), *accent))
            } else {
                None
            }
        })
    }

    /// Toggle keyboard shortcuts help overlay (Cmd+/)
    fn toggle_shortcuts_help(&mut self, cx: &mut Context<Self>) {
        self.show_shortcuts_help = !self.show_shortcuts_help;
        cx.notify();
    }

    /// Insert current date/time at cursor position (Cmd+Shift+D)
    fn insert_date_time(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let now = chrono::Local::now();
        let date_str = now.format("%Y-%m-%d %H:%M").to_string();
        self.editor_state.update(cx, |state, cx| {
            let selection = state.selection();
            let value = state.value().to_string();
            let start = selection.start.min(value.len());
            let end = selection.end.min(value.len());
            let new_value = format!("{}{}{}", &value[..start], date_str, &value[end..]);
            let new_cursor = start + date_str.len();
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Inserted date/time at cursor");
        cx.notify();
    }

    /// Copy note content as markdown to clipboard (Cmd+Shift+C)
    fn copy_as_markdown(&mut self, cx: &Context<Self>) {
        let content = self.editor_state.read(cx).value().to_string();
        self.copy_text_to_clipboard(&content);
        self.show_action_feedback("Copied", false);
        info!("Copied note as markdown to clipboard");
    }

    /// Toggle checklist checkbox on the current line (Cmd+Shift+L)
    ///
    /// Behavior:
    /// - If line starts with "- [ ] " → replace with "- [x] " (check)
    /// - If line starts with "- [x] " → replace with "- [ ] " (uncheck)
    /// - If line starts with "- " (list item) → add checkbox
    /// - Otherwise → prepend "- [ ] " to the line
    fn toggle_checklist(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find the start and end of the current line
            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);
            let line = &value[line_start..line_end];

            let (new_line, cursor_delta): (String, isize) =
                if let Some(rest) = line.strip_prefix("- [x] ") {
                    // Checked → unchecked
                    (format!("- [ ] {}", rest), 0)
                } else if let Some(rest) = line.strip_prefix("- [ ] ") {
                    // Unchecked → checked
                    (format!("- [x] {}", rest), 0)
                } else if let Some(rest) = line.strip_prefix("- ") {
                    // List item without checkbox → add checkbox
                    (format!("- [ ] {}", rest), 4) // "[ ] " is 4 chars
                } else {
                    // Plain line → add full checkbox prefix
                    (format!("- [ ] {}", line), 6) // "- [ ] " is 6 chars
                };

            let new_value = format!("{}{}{}", &value[..line_start], new_line, &value[line_end..]);
            let new_cursor = (cursor as isize + cursor_delta).max(0) as usize;
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Toggled checklist on current line");
        cx.notify();
    }

    /// Insert a horizontal rule (---) at cursor position (Cmd+Shift+-)
    fn insert_horizontal_rule(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Ensure we're on a new line and add the rule
            let needs_newline =
                cursor > 0 && value.as_bytes().get(cursor - 1).is_none_or(|&b| b != b'\n');
            let rule = if needs_newline {
                "\n\n---\n\n"
            } else {
                "\n---\n\n"
            };

            let new_value = format!("{}{}{}", &value[..cursor], rule, &value[cursor..]);
            let new_cursor = cursor + rule.len();
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Inserted horizontal rule");
        cx.notify();
    }

    /// Cycle heading level on the current line (Cmd+Shift+H)
    ///
    /// Behavior:
    /// - Plain text → `# text`
    /// - `# text` → `## text`
    /// - `## text` → `### text`
    /// - `### text` → plain text (strip heading)
    fn cycle_heading(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find the start and end of the current line
            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);
            let line = &value[line_start..line_end];

            let (new_line, cursor_delta): (String, isize) =
                if let Some(rest) = line.strip_prefix("### ") {
                    // ### → plain (remove 4 chars)
                    (rest.to_string(), -4)
                } else if let Some(rest) = line.strip_prefix("## ") {
                    // ## → ### (add 1 char)
                    (format!("### {}", rest), 1)
                } else if let Some(rest) = line.strip_prefix("# ") {
                    // # → ## (add 1 char)
                    (format!("## {}", rest), 1)
                } else {
                    // plain → # (add 2 chars)
                    (format!("# {}", line), 2)
                };

            let new_value = format!("{}{}{}", &value[..line_start], new_line, &value[line_end..]);
            let new_cursor = (cursor as isize + cursor_delta).max(line_start as isize) as usize;
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Cycled heading level on current line");
        cx.notify();
    }

    /// Move the current line up (Alt+Up)
    fn move_line_up(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find current line boundaries
            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);

            // Can't move up if already on first line
            if line_start == 0 {
                return;
            }

            // Find the previous line boundaries
            let prev_line_start = value[..line_start - 1].rfind('\n').map_or(0, |p| p + 1);

            let current_line = &value[line_start..line_end];
            let prev_line = &value[prev_line_start..line_start - 1]; // exclude the \n

            // Build new value: prev_line and current_line swapped
            let new_value = format!(
                "{}{}\n{}{}",
                &value[..prev_line_start],
                current_line,
                prev_line,
                &value[line_end..]
            );

            // Adjust cursor position: move it up by the length of prev_line + newline
            let offset_in_line = cursor - line_start;
            let new_cursor = prev_line_start + offset_in_line;

            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        cx.notify();
    }

    /// Move the current line down (Alt+Down)
    fn move_line_down(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find current line boundaries
            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);

            // Can't move down if already on last line
            if line_end >= value.len() {
                return;
            }

            // Find the next line boundaries
            let next_line_end = value[line_end + 1..]
                .find('\n')
                .map_or(value.len(), |p| line_end + 1 + p);

            let current_line = &value[line_start..line_end];
            let next_line = &value[line_end + 1..next_line_end];

            // Build new value: next_line and current_line swapped
            let new_value = format!(
                "{}{}\n{}{}",
                &value[..line_start],
                next_line,
                current_line,
                &value[next_line_end..]
            );

            // Adjust cursor: it moves down by length of next_line + newline
            let offset_in_line = cursor - line_start;
            let new_line_start = line_start + next_line.len() + 1;
            let new_cursor = new_line_start + offset_in_line;

            state.set_value(&new_value, window, cx);
            state.set_selection(
                new_cursor.min(new_value.len()),
                new_cursor.min(new_value.len()),
                window,
                cx,
            );
        });
        self.has_unsaved_changes = true;
        cx.notify();
    }

    /// Select the entire current line (Cmd+L)
    fn select_current_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);

            state.set_selection(line_start, line_end, window, cx);
        });
        cx.notify();
    }

    /// Smart paste: if text is selected and clipboard contains a URL, wrap as markdown link.
    /// Otherwise, fall through to normal paste behavior.
    /// Returns true if smart paste was handled, false to let default paste proceed.
    fn try_smart_paste(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let clipboard = Self::read_clipboard();
        let trimmed = clipboard.trim();

        // Check if clipboard looks like a URL
        if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
            return false;
        }

        // Check if we have a text selection
        let selection = self.editor_state.read(cx).selection();
        if selection.start == selection.end {
            return false;
        }

        // We have a URL on clipboard and selected text — create a markdown link
        let value = self.editor_state.read(cx).value().to_string();
        let start = selection.start.min(value.len());
        let end = selection.end.min(value.len());
        let (start, end) = if start > end {
            (end, start)
        } else {
            (start, end)
        };
        let selected_text = &value[start..end];
        let link = format!("[{}]({})", selected_text, trimmed);
        let new_value = format!("{}{}{}", &value[..start], link, &value[end..]);
        let new_cursor = start + link.len();

        self.editor_state.update(cx, |state, cx| {
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Smart paste: wrapped selection as markdown link");
        cx.notify();
        true
    }

    /// Wrap selected lines as blockquote (Cmd+Shift+.)
    ///
    /// Prefixes each selected line (or current line if no selection) with "> ".
    /// If all target lines already start with "> ", remove the prefix instead (toggle).
    fn toggle_blockquote(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let sel_start = selection.start.min(value.len());
            let sel_end = selection.end.min(value.len());
            let (sel_start, sel_end) = if sel_start > sel_end {
                (sel_end, sel_start)
            } else {
                (sel_start, sel_end)
            };

            // Expand to full lines
            let region_start = value[..sel_start].rfind('\n').map_or(0, |p| p + 1);
            let region_end = value[sel_end..]
                .find('\n')
                .map_or(value.len(), |p| sel_end + p);

            let region = &value[region_start..region_end];
            let lines: Vec<&str> = region.split('\n').collect();

            // Check if ALL lines already have blockquote prefix
            let all_quoted = lines.iter().all(|l| l.starts_with("> "));

            let new_lines: Vec<String> = if all_quoted {
                // Remove "> " prefix from all lines
                lines
                    .iter()
                    .map(|l| l.strip_prefix("> ").unwrap_or(l).to_string())
                    .collect()
            } else {
                // Add "> " prefix to all lines
                lines.iter().map(|l| format!("> {}", l)).collect()
            };

            let new_region = new_lines.join("\n");
            let new_value = format!(
                "{}{}{}",
                &value[..region_start],
                new_region,
                &value[region_end..]
            );

            // Place cursor at end of modified region
            let new_cursor = region_start + new_region.len();
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Toggled blockquote on selected lines");
        cx.notify();
    }

    /// Duplicate the current line below (Alt+Shift+Down) or above (Alt+Shift+Up)
    fn duplicate_line(
        &mut self,
        direction_down: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find current line boundaries
            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);
            let line = &value[line_start..line_end];

            let (new_value, new_cursor) = if direction_down {
                // Insert copy after current line
                let new_value = format!("{}\n{}{}", &value[..line_end], line, &value[line_end..]);
                // Move cursor to same offset in the duplicated line below
                let offset_in_line = cursor - line_start;
                let new_cursor = line_end + 1 + offset_in_line;
                (new_value, new_cursor)
            } else {
                // Insert copy before current line
                let new_value =
                    format!("{}{}\n{}", &value[..line_start], line, &value[line_start..]);
                // Cursor stays at same absolute position (now on the original line pushed down)
                let offset_in_line = cursor - line_start;
                let new_cursor = line_start + offset_in_line;
                (new_value, new_cursor)
            };

            state.set_value(&new_value, window, cx);
            state.set_selection(
                new_cursor.min(new_value.len()),
                new_cursor.min(new_value.len()),
                window,
                cx,
            );
        });
        self.has_unsaved_changes = true;
        info!(
            direction = if direction_down { "down" } else { "up" },
            "Duplicated current line"
        );
        cx.notify();
    }

    /// Delete the entire current line (Ctrl+Shift+K)
    fn delete_current_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find current line boundaries
            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);

            // Calculate what to remove: include the trailing newline if present, or leading newline
            let (remove_start, remove_end) = if line_end < value.len() {
                // There's a newline after — remove line + newline
                (line_start, line_end + 1)
            } else if line_start > 0 {
                // Last line — remove leading newline + line
                (line_start - 1, line_end)
            } else {
                // Only line — clear everything
                (0, value.len())
            };

            let new_value = format!("{}{}", &value[..remove_start], &value[remove_end..]);
            let new_cursor = remove_start.min(new_value.len());
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Deleted current line");
        cx.notify();
    }

    /// Insert 2 spaces at cursor position (Tab key)
    fn indent_at_cursor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());
            let indent = "  ";
            let new_value = format!("{}{}{}", &value[..cursor], indent, &value[cursor..]);
            let new_cursor = cursor + indent.len();
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        cx.notify();
    }

    /// Remove up to 2 leading spaces from the current line (Shift+Tab)
    fn outdent_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line = &value[line_start..];

            let remove_count = if line.starts_with("  ") {
                2
            } else if line.starts_with(' ') {
                1
            } else {
                return;
            };

            let new_value = format!(
                "{}{}",
                &value[..line_start],
                &value[line_start + remove_count..]
            );
            let new_cursor = cursor.saturating_sub(remove_count).max(line_start);
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        cx.notify();
    }

    /// Toggle bullet list prefix on current line (Cmd+Shift+8)
    ///
    /// Behavior:
    /// - Plain text → `- text`
    /// - `- text` → plain text (strip prefix)
    /// - `- [ ] text` or `- [x] text` → `- text` (strip checkbox, keep bullet)
    fn toggle_bullet_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);
            let line = &value[line_start..line_end];

            let (new_line, cursor_delta): (String, isize) =
                if let Some(rest) = line.strip_prefix("- [x] ") {
                    // Checkbox → bullet only (remove checkbox, keep "- ")
                    (format!("- {}", rest), -4)
                } else if let Some(rest) = line.strip_prefix("- [ ] ") {
                    // Checkbox → bullet only
                    (format!("- {}", rest), -4)
                } else if let Some(rest) = line.strip_prefix("- ") {
                    // Bullet → plain (remove "- ")
                    (rest.to_string(), -2)
                } else {
                    // Plain → bullet (add "- ")
                    (format!("- {}", line), 2)
                };

            let new_value = format!("{}{}{}", &value[..line_start], new_line, &value[line_end..]);
            let new_cursor = (cursor as isize + cursor_delta).max(line_start as isize) as usize;
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Toggled bullet list on current line");
        cx.notify();
    }

    /// Toggle numbered list prefix on current line (Cmd+Shift+7)
    ///
    /// Behavior:
    /// - Plain text → `1. text` (auto-detects sequence from previous line)
    /// - `N. text` → plain text (strip numbered prefix)
    fn toggle_numbered_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            let line_start = value[..cursor].rfind('\n').map_or(0, |p| p + 1);
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);
            let line = &value[line_start..line_end];

            // Check if line already has a numbered list prefix (e.g., "1. ", "12. ")
            let numbered_prefix_len = Self::numbered_list_prefix_len(line);

            let (new_line, cursor_delta): (String, isize) = if numbered_prefix_len > 0 {
                // Remove numbered prefix
                let rest = &line[numbered_prefix_len..];
                (rest.to_string(), -(numbered_prefix_len as isize))
            } else {
                // Add numbered prefix — detect number from previous line
                let num = Self::detect_next_list_number(&value, line_start);
                let prefix = format!("{}. ", num);
                let prefix_len = prefix.len() as isize;
                (format!("{}{}", prefix, line), prefix_len)
            };

            let new_value = format!("{}{}{}", &value[..line_start], new_line, &value[line_end..]);
            let new_cursor = (cursor as isize + cursor_delta).max(line_start as isize) as usize;
            state.set_value(&new_value, window, cx);
            state.set_selection(new_cursor, new_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Toggled numbered list on current line");
        cx.notify();
    }

    /// Get the length of a numbered list prefix (e.g., "1. " → 3, "12. " → 4, "abc" → 0)
    fn numbered_list_prefix_len(line: &str) -> usize {
        let mut chars = line.chars().peekable();
        let mut digit_count = 0;

        // Count leading digits
        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() {
                digit_count += 1;
                chars.next();
            } else {
                break;
            }
        }

        if digit_count == 0 {
            return 0;
        }

        // Must be followed by ". "
        if chars.next() == Some('.') && chars.next() == Some(' ') {
            digit_count + 2 // digits + ". "
        } else {
            0
        }
    }

    /// Detect the next number for a numbered list by looking at the previous line
    fn detect_next_list_number(value: &str, current_line_start: usize) -> usize {
        if current_line_start == 0 {
            return 1;
        }
        // Find previous line
        let prev_line_end = current_line_start - 1; // skip the \n
        let prev_line_start = value[..prev_line_end].rfind('\n').map_or(0, |p| p + 1);
        let prev_line = &value[prev_line_start..prev_line_end];

        // Check if previous line has a numbered prefix
        let prefix_len = Self::numbered_list_prefix_len(prev_line);
        if prefix_len > 0 {
            // Parse the number from the previous line
            let num_str: String = prev_line
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            num_str.parse::<usize>().unwrap_or(1) + 1
        } else {
            1
        }
    }

    /// Join the current line with the next line (Cmd+J)
    ///
    /// Replaces the newline between current and next line with a single space.
    fn join_lines(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let cursor = selection.start.min(value.len());

            // Find end of current line
            let line_end = value[cursor..]
                .find('\n')
                .map_or(value.len(), |p| cursor + p);

            // Can't join if on the last line
            if line_end >= value.len() {
                return;
            }

            // Find the start of actual content on the next line (skip leading whitespace)
            let next_content_start = value[line_end + 1..]
                .find(|c: char| !c.is_whitespace() || c == '\n')
                .map_or(value.len(), |p| line_end + 1 + p);

            // If next line is empty or only whitespace, just remove the newline
            let next_char = value.as_bytes().get(next_content_start);
            let (new_value, join_cursor) = if next_char == Some(&b'\n') || next_char.is_none() {
                // Next line is blank — remove it
                let end = if next_content_start < value.len() {
                    next_content_start + 1 // include the trailing \n
                } else {
                    next_content_start
                };
                let new_value = format!("{}{}", &value[..line_end], &value[end..]);
                (new_value, line_end)
            } else {
                // Join with a space
                let new_value = format!("{} {}", &value[..line_end], &value[next_content_start..]);
                (new_value, line_end)
            };

            state.set_value(&new_value, window, cx);
            state.set_selection(join_cursor, join_cursor, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Joined current line with next");
        cx.notify();
    }

    /// Cycle the case of selected text (Cmd+Shift+U)
    ///
    /// Behavior:
    /// - lowercase → UPPERCASE
    /// - UPPERCASE → Title Case
    /// - Title Case → lowercase
    /// - Mixed → lowercase (then cycles from there)
    fn transform_case(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_state.update(cx, |state, cx| {
            let value = state.value().to_string();
            let selection = state.selection();
            let start = selection.start.min(value.len());
            let end = selection.end.min(value.len());
            let (start, end) = if start > end {
                (end, start)
            } else {
                (start, end)
            };

            if start == end {
                return; // No selection, nothing to transform
            }

            let selected = &value[start..end];

            // Determine current case and cycle
            let transformed = if selected == selected.to_lowercase() {
                // All lowercase → UPPERCASE
                selected.to_uppercase()
            } else if selected == selected.to_uppercase() {
                // All UPPERCASE → Title Case
                Self::to_title_case(selected)
            } else {
                // Mixed/Title Case → lowercase
                selected.to_lowercase()
            };

            let new_value = format!("{}{}{}", &value[..start], transformed, &value[end..]);
            let new_end = start + transformed.len();
            state.set_value(&new_value, window, cx);
            state.set_selection(start, new_end, window, cx);
        });
        self.has_unsaved_changes = true;
        info!("Transformed case of selected text");
        cx.notify();
    }

    /// Convert a string to Title Case (capitalize first letter of each word)
    fn to_title_case(s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase())
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Render the keyboard shortcuts help overlay
    fn render_shortcuts_help(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let accent = cx.theme().accent;
        let border_color = cx.theme().border;
        let bg = cx.theme().background.opacity(OPACITY_OVERLAY_BG);

        let shortcut = |keys: &str, desc: &str| -> AnyElement {
            div()
                .flex()
                .justify_between()
                .w_full()
                .py_1() // 4px — on the spacing grid
                .child(div().text_xs().text_color(muted).child(desc.to_string()))
                .child(div().text_xs().text_color(accent).child(keys.to_string()))
                .into_any_element()
        };

        let section = |title: &str| -> AnyElement {
            div()
                .pt_3()
                .pb_1() // 4px — on the spacing grid
                .mb_1() // 4px — on the spacing grid
                .border_b_1()
                .border_color(border_color.opacity(OPACITY_SECTION_BORDER))
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(muted.opacity(OPACITY_MUTED))
                .child(title.to_string())
                .into_any_element()
        };

        div()
            .id("shortcuts-help-overlay")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(bg)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.show_shortcuts_help = false;
                    cx.notify();
                }),
            )
            .child(
                div()
                    .w(px(SHORTCUTS_PANEL_WIDTH))
                    .max_h(px(SHORTCUTS_PANEL_MAX_HEIGHT))
                    .overflow_y_scrollbar()
                    .rounded(px(10.)) // radius-md — softer card feel
                    .border_1()
                    .border_color(border_color.opacity(OPACITY_SECTION_BORDER))
                    .p_4()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .pb_2()
                            .child("Keyboard Shortcuts"),
                    )
                    .child(section("Notes"))
                    .child(shortcut("⌘N", "New note"))
                    .child(shortcut("⌘⇧N", "New from clipboard"))
                    .child(shortcut("⌘D", "Duplicate note"))
                    .child(shortcut("⌘⌫", "Delete note"))
                    .child(shortcut("⌘⇧I", "Toggle pin"))
                    .child(section("Navigation"))
                    .child(shortcut("⌘↑ / ⌘↓", "Previous / next note"))
                    .child(shortcut("⌘⇧↑ / ⌘⇧↓", "First / last note"))
                    .child(shortcut("⌘[ / ⌘]", "Back / forward"))
                    .child(shortcut("⌘1–9", "Jump to pinned note"))
                    .child(shortcut("⌘P", "Note switcher"))
                    .child(shortcut("⌘K", "Actions"))
                    .child(section("Formatting"))
                    .child(shortcut("⌘B", "Bold"))
                    .child(shortcut("⌘I", "Italic"))
                    .child(shortcut("⌘E", "Inline code"))
                    .child(shortcut("⌘⇧X", "Strikethrough"))
                    .child(shortcut("⌘⇧H", "Cycle heading"))
                    .child(shortcut("⌘⇧L", "Toggle checklist"))
                    .child(shortcut("⌘⇧.", "Toggle blockquote"))
                    .child(shortcut("⌘⇧-", "Horizontal rule"))
                    .child(shortcut("⌘⇧8", "Bullet list"))
                    .child(shortcut("⌘⇧7", "Numbered list"))
                    .child(section("Text"))
                    .child(shortcut("⌘⇧D", "Insert date/time"))
                    .child(shortcut("⌘⇧C", "Copy as markdown"))
                    .child(shortcut("⌘L", "Select line"))
                    .child(shortcut("⌘J", "Join lines"))
                    .child(shortcut("⌘⇧U", "Cycle case"))
                    .child(shortcut("⌥↑ / ⌥↓", "Move line"))
                    .child(shortcut("⌥⇧↑ / ⌥⇧↓", "Duplicate line"))
                    .child(shortcut("⌃⇧K", "Delete line"))
                    .child(shortcut("⌘V", "Smart paste"))
                    .child(shortcut("Tab", "Indent (2 spaces)"))
                    .child(shortcut("⇧Tab", "Outdent"))
                    .child(section("View"))
                    .child(shortcut("⌘.  / Esc", "Focus mode"))
                    .child(shortcut("⌘⇧P", "Markdown preview"))
                    .child(shortcut("⌘F", "Find in note"))
                    .child(shortcut("⌘⇧F", "Search all notes"))
                    .child(shortcut("⌘⇧S", "Cycle sort"))
                    .child(shortcut("⌘⇧T", "Toggle trash"))
                    .child(section("Window"))
                    .child(shortcut("⌘W", "Close"))
                    .child(shortcut("Esc", "Close panel"))
                    .child(shortcut("⌘/", "This help"))
                    .child(
                        div()
                            .pt_3()
                            .text_xs()
                            .text_color(muted.opacity(OPACITY_SUBTLE))
                            .text_center()
                            .child("Click anywhere or press ⌘/ to dismiss"),
                    ),
            )
    }

    /// Cycle sort mode: Updated → Created → Alphabetical → Updated
    fn cycle_sort_mode(&mut self, cx: &mut Context<Self>) {
        self.sort_mode = match self.sort_mode {
            NotesSortMode::Updated => NotesSortMode::Created,
            NotesSortMode::Created => NotesSortMode::Alphabetical,
            NotesSortMode::Alphabetical => NotesSortMode::Updated,
        };
        self.apply_sort(cx);
        info!(sort_mode = ?self.sort_mode, "Cycled sort mode");
    }

    /// Apply current sort mode to the notes list
    fn apply_sort(&mut self, cx: &mut Context<Self>) {
        match self.sort_mode {
            NotesSortMode::Updated => {
                self.notes.sort_by(|a, b| match (a.is_pinned, b.is_pinned) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => b.updated_at.cmp(&a.updated_at),
                });
            }
            NotesSortMode::Created => {
                self.notes.sort_by(|a, b| match (a.is_pinned, b.is_pinned) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => b.created_at.cmp(&a.created_at),
                });
            }
            NotesSortMode::Alphabetical => {
                self.notes.sort_by(|a, b| match (a.is_pinned, b.is_pinned) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                });
            }
        }
        cx.notify();
    }

    /// Empty the entire trash — permanently deletes all trashed notes
    fn empty_trash(&mut self, cx: &mut Context<Self>) {
        let ids: Vec<NoteId> = self.deleted_notes.iter().map(|n| n.id).collect();
        for id in &ids {
            if let Err(e) = storage::delete_note_permanently(*id) {
                tracing::error!(error = %e, note_id = %id, "Failed to permanently delete note");
            }
        }
        self.deleted_notes.clear();
        self.selected_note_id = None;
        info!(count = ids.len(), "Emptied trash");
        cx.notify();
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
        self.show_action_feedback("Duplicated", false);
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

    /// Execute an action from the note switcher (Cmd+P)
    /// Handles note selection when action_id starts with "note_"
    fn execute_note_switcher_action(
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
    fn open_browse_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

    /// Close the browse panel (note switcher) and refocus the editor
    fn close_browse_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
    fn toggle_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
    fn toggle_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

    /// Render the search input bar (shown when Cmd+F is pressed)
    fn render_search(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let has_query = !self.search_query.is_empty();
        let result_count = if has_query {
            self.notes
                .iter()
                .filter(|n| {
                    n.content
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                })
                .count()
        } else {
            self.notes.len()
        };

        div()
            .w_full()
            .px_3()
            .py_1() // 4px — tighter to match toolbar density
            .flex()
            .items_center()
            .gap_2()
            .border_b_1()
            .border_color(theme.border.opacity(OPACITY_SECTION_BORDER))
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground.opacity(OPACITY_MUTED))
                    .child("\u{2315}"), // ⌕ magnifying glass text char
            )
            .child(
                div().flex_1().child(
                    Input::new(&self.search_state)
                        .w_full()
                        .small()
                        .appearance(false),
                ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground.opacity(OPACITY_MUTED))
                    .child(format!("{} notes", result_count)),
            )
    }

    /// Render the formatting toolbar
    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_1()
            .py_1()
            .px_3() // Align horizontally with titlebar & footer
            .border_b_1() // Subtle bottom border — mirrors footer top border
            .border_color(cx.theme().border.opacity(OPACITY_SECTION_BORDER))
            .child(
                Button::new("bold")
                    .ghost()
                    .xsmall()
                    .label("B")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("**", "**", window, cx);
                    })),
            )
            .child(
                Button::new("italic")
                    .ghost()
                    .xsmall()
                    .label("I")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("_", "_", window, cx);
                    })),
            )
            .child(
                Button::new("heading")
                    .ghost()
                    .xsmall()
                    .label("H")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.cycle_heading(window, cx);
                    })),
            )
            .child(
                Button::new("list")
                    .ghost()
                    .xsmall()
                    .label("•")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle_bullet_list(window, cx);
                    })),
            )
            .child(
                Button::new("numbered-list")
                    .ghost()
                    .xsmall()
                    .label("1.")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle_numbered_list(window, cx);
                    })),
            )
            .child(
                Button::new("code")
                    .ghost()
                    .xsmall()
                    .label("</>")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("`", "`", window, cx);
                    })),
            )
            .child(
                Button::new("codeblock")
                    .ghost()
                    .xsmall()
                    .label("```")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("\n```\n", "\n```", window, cx);
                    })),
            )
            .child(
                Button::new("strikethrough")
                    .ghost()
                    .xsmall()
                    .label("S\u{0336}")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("~~", "~~", window, cx);
                    })),
            )
            .child(
                Button::new("checklist")
                    .ghost()
                    .xsmall()
                    .label("\u{2610}")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle_checklist(window, cx);
                    })),
            )
            .child(
                Button::new("link")
                    .ghost()
                    .xsmall()
                    .label("\u{1F517}")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("[", "](url)", window, cx);
                    })),
            )
            .child(
                Button::new("rule")
                    .ghost()
                    .xsmall()
                    .label("\u{2015}")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_horizontal_rule(window, cx);
                    })),
            )
            .child(
                Button::new("blockquote")
                    .ghost()
                    .xsmall()
                    .label(">")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.insert_formatting("\n> ", "", window, cx);
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
        let is_preview = self.preview_enabled;
        let char_count = self.get_character_count(cx);
        let is_pinned = self.is_current_note_pinned();
        let in_focus_mode = self.focus_mode;

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

        // Prepend trash indicator to title when in trash view
        let title = if is_trash {
            format!("🗑 {}", title)
        } else {
            title
        };

        // Raycast-style: titlebar only visible on hover, centered title, right-aligned actions
        let window_hovered = self.window_hovered || self.force_hovered;

        // Get muted foreground color for subtle icons/text
        let muted_color = cx.theme().muted_foreground;
        let accent_color = cx.theme().accent;
        let preview_label = if is_preview { "MD" } else { "TXT" };
        let preview_color = if is_preview {
            accent_color
        } else {
            muted_color.opacity(OPACITY_MUTED)
        };

        // ---------------------------------------------------------------
        // Titlebar: 3-column flex layout
        //   [left spacer (traffic lights)] [title flex-1 center] [icons]
        // This prevents the title from overlapping with the icon cluster
        // regardless of window width.
        // ---------------------------------------------------------------

        // Build the right-column icon cluster (extracted so we can reuse
        // the same width for the left spacer to keep the title centered).
        let titlebar_icons = div()
            .w(px(TITLEBAR_ICONS_W))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_end()
            .gap_2() // 8px — even spacing between icons
            // Normal view icons — only when hovered, not trash, not focus mode
            .when(window_hovered && !is_trash && !in_focus_mode, |d| {
                d
                    // Icon 1: Command key icon - opens actions panel (⌘K)
                    .when(has_selection, |d| {
                        d.child(
                            div()
                                .id("titlebar-cmd-icon")
                                .min_w(px(MIN_TARGET_SIZE))
                                .min_h(px(MIN_TARGET_SIZE))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(muted_color.opacity(OPACITY_MUTED))
                                .cursor_pointer()
                                .hover(|s| s.text_color(muted_color))
                                .tooltip(|window, cx| {
                                    Tooltip::new("Actions")
                                        .key_binding(
                                            gpui::Keystroke::parse("cmd-k").ok().map(Kbd::new),
                                        )
                                        .build(window, cx)
                                })
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
                    // Icon 2: List icon - note switcher
                    .child(
                        div()
                            .id("titlebar-browse-icon")
                            .min_w(px(MIN_TARGET_SIZE))
                            .min_h(px(MIN_TARGET_SIZE))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .text_color(muted_color.opacity(OPACITY_MUTED))
                            .cursor_pointer()
                            .hover(|s| s.text_color(muted_color))
                            .tooltip(|window, cx| {
                                Tooltip::new("Note switcher")
                                    .key_binding(gpui::Keystroke::parse("cmd-p").ok().map(Kbd::new))
                                    .build(window, cx)
                            })
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
                    // Icon 3: Markdown preview toggle
                    .child(
                        div()
                            .id("titlebar-preview-icon")
                            .min_w(px(MIN_TARGET_SIZE))
                            .min_h(px(MIN_TARGET_SIZE))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .text_color(preview_color)
                            .cursor_pointer()
                            .hover(|s| s.text_color(accent_color))
                            .tooltip(|window, cx| {
                                Tooltip::new("Toggle preview")
                                    .key_binding(
                                        gpui::Keystroke::parse("cmd-shift-p").ok().map(Kbd::new),
                                    )
                                    .build(window, cx)
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.toggle_preview(window, cx);
                            }))
                            .child(preview_label),
                    )
                    // Icon 4: Plus icon - new note
                    .child(
                        div()
                            .id("titlebar-new-icon")
                            .min_w(px(MIN_TARGET_SIZE))
                            .min_h(px(MIN_TARGET_SIZE))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .text_color(muted_color.opacity(OPACITY_MUTED))
                            .cursor_pointer()
                            .hover(|s| s.text_color(muted_color))
                            .tooltip(|window, cx| {
                                Tooltip::new("New note")
                                    .key_binding(gpui::Keystroke::parse("cmd-n").ok().map(Kbd::new))
                                    .build(window, cx)
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.create_note(window, cx);
                            }))
                            .child("+"),
                    )
            })
            // Trash view actions (always visible when in trash)
            .when(has_selection && is_trash, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            Button::new("restore")
                                .ghost()
                                .xsmall()
                                .label("Restore (⌘Z)")
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

        let titlebar = div()
            .id("notes-titlebar")
            .flex()
            .items_center()
            .h(px(TITLEBAR_HEIGHT))
            .px_3()
            // NO .bg() — let vibrancy show through from root
            // Trash view: subtle danger-colored bottom border as visual cue
            .when(is_trash, |d| {
                d.border_b_1()
                    .border_color(cx.theme().danger.opacity(OPACITY_ACCENT_BORDER))
            })
            // Track titlebar hover for showing/hiding icons
            .on_hover(cx.listener(|this, hovered, _, cx| {
                if this.force_hovered {
                    return;
                }
                this.titlebar_hovered = *hovered;
                cx.notify();
            }))
            // LEFT column — spacer matching the right icon cluster width
            // to keep the title optically centered between traffic lights and icons.
            .child(div().w(px(TITLEBAR_TRAFFIC_LIGHT_W)).flex_shrink_0())
            // CENTER column — title (flex-1, centered, truncated with ellipsis)
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.)) // allow flex shrinking below content size
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_sm()
                    .text_color(muted_color)
                    // OPACITY_MUTED (0.7) keeps the title legible at rest — WCAG AA.
                    .when(!window_hovered, |d| d.opacity(OPACITY_MUTED))
                    .when(window_hovered, |d| d.opacity(1.0))
                    // In focus mode, hide title completely (distraction-free)
                    .when(in_focus_mode, |d| d.opacity(0.))
                    // Pin indicator — filled dot in accent color
                    .when(is_pinned && !in_focus_mode, |d| {
                        d.child(div().text_xs().text_color(accent_color).child("●"))
                    })
                    .child(title)
                    // Focus mode hint: subtle exit hint shown on hover
                    .when(in_focus_mode && window_hovered, |d| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(muted_color.opacity(OPACITY_DISABLED))
                                .child("esc  or  ⌘.  exit focus"),
                        )
                    }),
            )
            // RIGHT column — icons
            .child(titlebar_icons);

        // Enhanced footer: note position + line on LEFT, word/char count CENTERED, type + time on RIGHT
        // NOTE: No .bg() - let vibrancy show through from root
        let word_count = self.get_word_count(cx);
        let cursor_line_info = self.get_cursor_line_info(cx);
        let selection_stats = self.get_selection_stats(cx);
        let note_position = self.get_note_position();
        let has_unsaved = self.has_unsaved_changes;
        // Show brief "✓" checkmark (1.5s) after a successful save — frictionless feedback
        let show_saved = !has_unsaved
            && self
                .last_save_confirmed
                .map(|t| t.elapsed() < Duration::from_millis(1500))
                .unwrap_or(false);
        let relative_time = self.get_relative_time();
        let has_history_back = !self.history_back.is_empty();
        let has_history_forward = !self.history_forward.is_empty();
        let trash_count = self.deleted_notes.len();
        let is_trash_view = self.view_mode == NotesViewMode::Trash;
        let reading_time = self.get_reading_time(cx);
        let sort_label = match self.sort_mode {
            NotesSortMode::Updated => "updated ↓",
            NotesSortMode::Created => "created ↓",
            NotesSortMode::Alphabetical => "A→Z",
        };
        let auto_sizing_off = !self.auto_sizing_enabled;
        // Action feedback (e.g. "Deleted", "Pinned") — 2s flash
        let action_feedback = self
            .get_action_feedback()
            .map(|(msg, accent)| (msg.to_string(), accent));
        // Created date for the current note
        let created_date = self
            .selected_note_id
            .and_then(|id| self.get_visible_notes().iter().find(|n| n.id == id))
            .map(|note| note.created_at.format("%b %d, %Y").to_string());
        let footer = div()
            .flex()
            .items_center()
            .gap_2() // 8px between left / center / right columns
            .h(px(FOOTER_HEIGHT))
            .px_3()
            // Subtle top border separates footer from editor content
            .border_t_1()
            .border_color(cx.theme().border.opacity(OPACITY_SECTION_BORDER))
            // NO .bg() — let vibrancy show through from root
            // Always visible at readable opacity; full on hover
            // In focus mode: hidden at rest, subtle word count on hover
            .when(in_focus_mode && !window_hovered, |d| d.opacity(0.))
            .when(in_focus_mode && window_hovered, |d| {
                d.opacity(OPACITY_DISABLED)
            })
            .when(!in_focus_mode && !window_hovered, |d| {
                d.opacity(OPACITY_SUBTLE)
            })
            .when(!in_focus_mode && window_hovered, |d| d.opacity(1.0))
            // LEFT column: History arrows + note position + unsaved dot
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1() // 4px — tighter to save horizontal space
                    .overflow_hidden()
                    // History back arrow (Cmd+[) — meets WCAG 2.5.8 target size
                    .child(
                        div()
                            .id("footer-history-back")
                            .min_w(px(MIN_TARGET_SIZE))
                            .min_h(px(MIN_TARGET_SIZE))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(if has_history_back {
                                cx.theme().muted_foreground
                            } else {
                                cx.theme().muted_foreground.opacity(OPACITY_DISABLED)
                            })
                            .when(has_history_back, |d| {
                                d.cursor_pointer()
                                    .hover(|s| s.text_color(cx.theme().foreground))
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate_back(window, cx);
                            }))
                            .child("‹"),
                    )
                    // History forward arrow (Cmd+])
                    .child(
                        div()
                            .id("footer-history-forward")
                            .min_w(px(MIN_TARGET_SIZE))
                            .min_h(px(MIN_TARGET_SIZE))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(if has_history_forward {
                                cx.theme().muted_foreground
                            } else {
                                cx.theme().muted_foreground.opacity(OPACITY_DISABLED)
                            })
                            .when(has_history_forward, |d| {
                                d.cursor_pointer()
                                    .hover(|s| s.text_color(cx.theme().foreground))
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate_forward(window, cx);
                            }))
                            .child("›"),
                    )
                    // Unsaved changes — filled dot in accent color
                    .when(has_unsaved, |d| {
                        d.child(div().text_xs().text_color(cx.theme().accent).child("●"))
                    })
                    // Brief checkmark flash after successful save — frictionless feedback
                    .when(show_saved, |d| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().accent.opacity(OPACITY_MUTED))
                                .child("✓"),
                        )
                    })
                    // Action feedback flash (delete, pin, duplicate, copy) — 2s
                    .when_some(action_feedback.clone(), |d, (msg, accent)| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(if accent {
                                    cx.theme().accent
                                } else {
                                    cx.theme().muted_foreground.opacity(OPACITY_MUTED)
                                })
                                .child(msg),
                        )
                    })
                    .when_some(note_position, |d, (pos, total)| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{}/{}", pos, total)),
                        )
                    }),
            )
            // CENTER (flex-1): Ln position · word count · char count · reading time
            // Uses FOOTER_SEP (" · ") for consistent separator styling.
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.)) // allow flex shrinking
                    .flex()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    // Cursor line position — "Ln 5/42"
                    .when_some(cursor_line_info, |d, (line, total)| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                .child(format!("Ln {}/{}", line, total)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                .child(FOOTER_SEP),
                        )
                    })
                    .child(if let Some((sel_words, sel_chars)) = selection_stats {
                        // Show selection stats in accent color when text is selected
                        div().text_xs().text_color(cx.theme().accent).child(format!(
                            "{}/{} words{}{}/{} chars",
                            sel_words, word_count, FOOTER_SEP, sel_chars, char_count,
                        ))
                    } else {
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "{} words{}{} chars",
                                word_count, FOOTER_SEP, char_count,
                            ))
                    })
                    .when(!reading_time.is_empty(), |d| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                .child(FOOTER_SEP),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                .child(reading_time.clone()),
                        )
                    }),
            )
            // RIGHT column: Sort indicator + trash badge + time + type indicator
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap_1() // 4px — compact to keep right cluster tight
                    // Auto-size disabled indicator — clickable to re-enable
                    .when(auto_sizing_off && !is_trash_view, |d| {
                        d.child(
                            div()
                                .id("footer-auto-size-off")
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                                .cursor_pointer()
                                .hover(|s| s.text_color(cx.theme().accent))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.enable_auto_sizing(window, cx);
                                }))
                                .child("⤢ auto-size"),
                        )
                    })
                    // Sort indicator (always visible, clickable to cycle)
                    .when(!is_trash_view, |d| {
                        d.child(
                            div()
                                .id("footer-sort-indicator")
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                                .cursor_pointer()
                                .hover(|s| s.text_color(cx.theme().muted_foreground))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.cycle_sort_mode(cx);
                                }))
                                .child(sort_label),
                        )
                    })
                    // Trash badge: shows count of deleted notes when not in trash view
                    .when(!is_trash_view && trash_count > 0, |d| {
                        d.child(
                            div()
                                .id("footer-trash-badge")
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                                .cursor_pointer()
                                .hover(|s| s.text_color(cx.theme().muted_foreground))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.set_view_mode(NotesViewMode::Trash, window, cx);
                                }))
                                .child(format!("trash ({})", trash_count)),
                        )
                    })
                    // "Empty Trash" link when in trash view with items
                    .when(is_trash_view && trash_count > 0, |d| {
                        d.child(
                            div()
                                .id("footer-empty-trash")
                                .text_xs()
                                .text_color(cx.theme().danger)
                                .cursor_pointer()
                                .hover(|s| s.text_color(cx.theme().foreground))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.empty_trash(cx);
                                }))
                                .child("empty trash"),
                        )
                    })
                    // "Back to notes" link when in trash view
                    .when(is_trash_view, |d| {
                        d.child(
                            div()
                                .id("footer-back-to-notes")
                                .text_xs()
                                .text_color(cx.theme().accent)
                                .cursor_pointer()
                                .hover(|s| s.text_color(cx.theme().foreground))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.set_view_mode(NotesViewMode::AllNotes, window, cx);
                                }))
                                .child("back to notes"),
                        )
                    })
                    // Created date — only visible on hover
                    .when(window_hovered, |d| {
                        d.when_some(created_date.clone(), |d, date| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                                    .child(date),
                            )
                        })
                    })
                    .when_some(relative_time, |d, time| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                                .child(time),
                        )
                    })
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                            .child(if is_preview { "MD" } else { "TXT" }),
                    ),
            );

        let no_notes = self.get_visible_notes().is_empty();
        let editor_body: AnyElement = if no_notes && !has_selection && is_trash {
            // Empty trash state
            div()
                .id("notes-empty-trash")
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .child(
                    div()
                        .text_base()
                        .text_color(cx.theme().muted_foreground)
                        .child("Trash is empty"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                        .child("Deleted notes will appear here"),
                )
                .child(
                    div()
                        .id("back-to-notes-link")
                        .text_xs()
                        .text_color(cx.theme().accent)
                        .cursor_pointer()
                        .hover(|s| s.text_color(cx.theme().foreground))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.set_view_mode(NotesViewMode::AllNotes, window, cx);
                        }))
                        .child("← Back to Notes"),
                )
                .into_any_element()
        } else if no_notes && !has_selection {
            // Empty state: welcoming instructions when no notes exist
            div()
                .id("notes-empty-state")
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_3()
                .child(
                    div()
                        .text_base()
                        .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                        .child("No notes yet"),
                )
                .child(
                    div()
                        .id("create-first-note")
                        .text_sm()
                        .text_color(cx.theme().accent)
                        .cursor_pointer()
                        .hover(|s| s.text_color(cx.theme().foreground))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.create_note(window, cx);
                        }))
                        .child("Create your first note"),
                )
                .child(
                    div().flex().flex_col().items_center().gap_1().pt_2().child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                    .child("⌘N  new"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                    .child("⌘⇧N  from clipboard"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                    .child("⌘/  shortcuts"),
                            ),
                    ),
                )
                .into_any_element()
        } else if is_preview {
            let content = self.editor_state.read(cx).value().to_string();
            div()
                .id("notes-markdown-preview")
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scrollbar()
                .px_4() // 16px horizontal — match editor content padding
                .py_3() // 12px vertical — match editor content padding
                .child(markdown::render_markdown_preview(&content, cx.theme()))
                .into_any_element()
        } else {
            Input::new(&self.editor_state)
                .h_full()
                .appearance(false)
                .font_family(cx.theme().mono_font_family.clone())
                .text_size(cx.theme().mono_font_size)
                .into_any_element()
        };

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
            // Search bar (Cmd+F to toggle) — hidden in focus mode
            .when(self.show_search && !in_focus_mode, |d| {
                d.child(self.render_search(cx))
            })
            // Toolbar hidden by default - only show when pinned — hidden in focus mode
            .when(
                !is_trash && has_selection && show_toolbar && !in_focus_mode,
                |d| d.child(self.render_toolbar(cx)),
            )
            .child(
                div()
                    .flex_1()
                    .px_4() // 16px horizontal — one step up for comfortable reading
                    .py_3() // 12px vertical — tighter to keep compact feel
                    // NO .bg() — let vibrancy show through from root
                    // Use a styled input that blends with background
                    .child(editor_body),
            )
            .when(has_selection, |d| d.child(footer))
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

        div()
            .id("actions-panel-overlay")
            .absolute()
            .inset_0()
            .bg(Self::get_modal_overlay_background()) // Theme-aware overlay
            .flex()
            .flex_col()
            .items_center() // Horizontally centered
            .justify_start() // Vertically aligned to top (not centered!)
            .pt(px(ACTIONS_PANEL_TOP_OFFSET))
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
                .bg(Self::get_modal_overlay_background()) // Theme-aware overlay
                .flex()
                .items_center()
                .justify_center()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.close_browse_panel(window, cx);
                }))
                .child(
                    div()
                        .w(px(BROWSE_PANEL_WIDTH))
                        .max_h(px(BROWSE_PANEL_MAX_HEIGHT))
                        // NO .bg() - overlay already provides backdrop, avoid double-layering opacity
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded_lg()
                        // Shadow disabled for vibrancy - shadows on transparent elements cause gray fill
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
        let sk_theme = crate::theme::get_cached_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.main;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.main,
        ))
    }

    /// Get title bar background with vibrancy opacity
    fn get_vibrancy_title_bar_background(_cx: &Context<Self>) -> gpui::Rgba {
        let sk_theme = crate::theme::get_cached_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.title_bar;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.title_bar,
        ))
    }

    /// Get sidebar/panel background with vibrancy opacity
    fn get_vibrancy_sidebar_background() -> gpui::Rgba {
        let sk_theme = crate::theme::get_cached_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.title_bar;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.title_bar,
        ))
    }

    /// Get modal overlay background (theme-aware)
    ///
    /// For dark mode: black overlay (darkens content behind)
    /// For light mode: white overlay (keeps content readable on light backgrounds)
    /// 50% opacity (0x80) for good contrast without being too heavy
    fn get_modal_overlay_background() -> gpui::Rgba {
        let sk_theme = crate::theme::get_cached_theme();
        if sk_theme.has_dark_colors() {
            gpui::rgba(0x00000080) // black at 50% for dark mode
        } else {
            gpui::rgba(0xffffff80) // white at 50% for light mode
        }
    }

    fn set_mouse_cursor_hidden_state(mouse_cursor_hidden: &mut bool, hidden: bool) -> bool {
        if *mouse_cursor_hidden == hidden {
            return false;
        }
        *mouse_cursor_hidden = hidden;
        true
    }

    fn hide_mouse_cursor(&mut self, cx: &mut Context<Self>) {
        if Self::set_mouse_cursor_hidden_state(&mut self.mouse_cursor_hidden, true) {
            crate::platform::hide_cursor_until_mouse_moves();
            cx.notify();
        }
    }

    fn show_mouse_cursor(&mut self, cx: &mut Context<Self>) {
        if Self::set_mouse_cursor_hidden_state(&mut self.mouse_cursor_hidden, false) {
            cx.notify();
        }
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

        // Clear the global app entity handle
        if let Some(app_entity) = NOTES_APP_ENTITY.get() {
            if let Ok(mut guard) = app_entity.lock() {
                *guard = None;
                debug!("NotesApp dropped - cleared global app entity handle");
            }
        }
    }
}

impl Render for NotesApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Capture for use in div builder (cursor style applied via .cursor() modifier)
        let mouse_cursor_hidden = self.mouse_cursor_hidden;

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
        // Note: show_browse removed - note_switcher uses CommandBar which renders in its own window

        // Raycast-style single-note view: no sidebar, editor fills full width
        // Track window hover for traffic lights visibility

        // Get vibrancy background - tints the blur effect with theme color
        let vibrancy_bg = crate::ui_foundation::get_window_vibrancy_background();

        div()
            .id("notes-window-root")
            .flex()
            .flex_col()
            .size_full()
            .relative()
            // Apply vibrancy background like POC does - Root no longer provides this
            .bg(vibrancy_bg)
            // Shadow disabled for vibrancy - shadows on transparent elements cause gray fill
            .text_color(cx.theme().foreground)
            .track_focus(&self.focus_handle)
            // Hide mouse cursor while typing
            .when(mouse_cursor_hidden, |d| d.cursor(CursorStyle::None))
            // Close any open CommandBar when clicking anywhere on the notes window
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    if this.command_bar.is_open() {
                        this.close_actions_panel(window, cx);
                    }
                    if this.note_switcher.is_open() {
                        this.close_browse_panel(window, cx);
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
            .on_mouse_move(cx.listener(|this, _: &MouseMoveEvent, _, cx| {
                this.show_mouse_cursor(cx);
            }))
            // CRITICAL: Use capture_key_down to intercept keys BEFORE Input component handles them
            // This ensures arrow keys go to CommandBar navigation instead of Input cursor movement
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                this.hide_mouse_cursor(cx);

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
                        "enter" | "return" => {
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
                            "enter" | "return" => {
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

                // Handle note switcher (Cmd+P) keyboard events - uses CommandBar
                if this.note_switcher.is_open() {
                    match key.as_str() {
                        "escape" | "esc" => {
                            this.close_browse_panel(window, cx);
                            cx.stop_propagation();
                            return;
                        }
                        "up" | "arrowup" => {
                            this.note_switcher.select_prev(cx);
                            cx.stop_propagation();
                            return;
                        }
                        "down" | "arrowdown" => {
                            this.note_switcher.select_next(cx);
                            cx.stop_propagation();
                            return;
                        }
                        "enter" | "return" => {
                            if let Some(action_id) = this.note_switcher.execute_selected_action(cx)
                            {
                                this.execute_note_switcher_action(&action_id, window, cx);
                            }
                            cx.stop_propagation();
                            return;
                        }
                        "backspace" | "delete" => {
                            this.note_switcher.handle_backspace(cx);
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
                                        this.note_switcher.handle_char(ch, cx);
                                        cx.stop_propagation();
                                        return;
                                    }
                                }
                            }
                            // Cmd+P also closes the note switcher
                            if modifiers.platform && key == "p" {
                                this.close_browse_panel(window, cx);
                                cx.stop_propagation();
                                return;
                            }
                        }
                    }
                    // Don't fall through to other handlers when note switcher is open
                    return;
                }

                // Handle Escape to close panels, exit modes, or close window
                if key == "escape" {
                    if this.show_shortcuts_help {
                        this.show_shortcuts_help = false;
                        cx.notify();
                        return;
                    }
                    if this.show_actions_panel || this.command_bar.is_open() {
                        this.close_actions_panel(window, cx);
                        return;
                    }
                    if this.note_switcher.is_open() {
                        this.close_browse_panel(window, cx);
                        return;
                    }
                    if this.show_search {
                        this.toggle_search(window, cx);
                        return;
                    }
                    // Exit focus mode before closing window
                    if this.focus_mode {
                        this.toggle_focus_mode(cx);
                        return;
                    }
                    // In trash view, Escape goes back to all notes
                    if this.view_mode == NotesViewMode::Trash {
                        this.set_view_mode(NotesViewMode::AllNotes, window, cx);
                        return;
                    }
                    // No panels open - close the window (same as Cmd+W)
                    let wb = window.window_bounds();
                    crate::window_state::save_window_from_gpui(
                        crate::window_state::WindowRole::Notes,
                        wb,
                    );
                    window.remove_window();
                    return;
                }

                // Tab: insert 2 spaces (indent); Shift+Tab: outdent
                if key == "tab" && !modifiers.platform && !modifiers.control && !modifiers.alt {
                    if modifiers.shift {
                        this.outdent_line(window, cx);
                    } else {
                        this.indent_at_cursor(window, cx);
                    }
                    cx.stop_propagation();
                    return;
                }

                // Alt+Up/Down: Move current line up/down
                // Alt+Shift+Up/Down: Duplicate current line up/down
                if modifiers.alt && !modifiers.platform {
                    match key.as_str() {
                        "up" | "arrowup" => {
                            if modifiers.shift {
                                this.duplicate_line(false, window, cx);
                            } else {
                                this.move_line_up(window, cx);
                            }
                            cx.stop_propagation();
                            return;
                        }
                        "down" | "arrowdown" => {
                            if modifiers.shift {
                                this.duplicate_line(true, window, cx);
                            } else {
                                this.move_line_down(window, cx);
                            }
                            cx.stop_propagation();
                            return;
                        }
                        _ => {}
                    }
                }

                // Ctrl+Shift+K: Delete current line
                if modifiers.control && modifiers.shift && key == "k" {
                    this.delete_current_line(window, cx);
                    cx.stop_propagation();
                    return;
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
                            if modifiers.shift {
                                // Toggle markdown preview
                                this.toggle_preview(window, cx);
                            } else {
                                // Toggle note switcher (browse panel)
                                this.close_actions_panel(window, cx);
                                if this.note_switcher.is_open() {
                                    this.close_browse_panel(window, cx);
                                } else {
                                    this.open_browse_panel(window, cx);
                                }
                            }
                        }
                        "f" => {
                            if modifiers.shift {
                                // Cmd+Shift+F: Cross-notes search (search across all notes)
                                this.toggle_search(window, cx);
                                cx.stop_propagation();
                            } else {
                                // Cmd+F: In-editor find (uses Input's built-in search)
                                // Focus editor and dispatch Search action
                                this.editor_state.update(cx, |state, cx| {
                                    state.focus(window, cx);
                                });
                                cx.dispatch_action(&Search);
                                cx.stop_propagation();
                            }
                        }
                        "n" => {
                            if modifiers.shift {
                                // Cmd+Shift+N: New note from clipboard
                                this.create_note_from_clipboard(window, cx);
                            } else {
                                this.create_note(window, cx);
                            }
                        }
                        "t" => {
                            if modifiers.shift {
                                // Cmd+Shift+T: Toggle trash view
                                if this.view_mode == NotesViewMode::Trash {
                                    this.set_view_mode(NotesViewMode::AllNotes, window, cx);
                                } else {
                                    this.set_view_mode(NotesViewMode::Trash, window, cx);
                                }
                                cx.stop_propagation();
                            }
                        }
                        "w" => {
                            // Close the notes window (standard macOS pattern)
                            // Close any open CommandBar windows first
                            this.command_bar.close_app(cx);
                            this.note_switcher.close_app(cx);
                            // Save bounds before closing
                            let wb = window.window_bounds();
                            crate::window_state::save_window_from_gpui(
                                crate::window_state::WindowRole::Notes,
                                wb,
                            );
                            window.remove_window();
                        }
                        "." => {
                            if modifiers.shift {
                                // Cmd+Shift+.: Toggle blockquote on selected lines
                                this.toggle_blockquote(window, cx);
                            } else {
                                // Cmd+.: Toggle focus mode
                                this.toggle_focus_mode(cx);
                            }
                            cx.stop_propagation();
                        }
                        "s" => {
                            if modifiers.shift {
                                // Cmd+Shift+S: Cycle sort mode
                                this.cycle_sort_mode(cx);
                                cx.stop_propagation();
                            }
                        }
                        "z" => {
                            // Cmd+Z: Restore from trash (when in trash view)
                            if this.view_mode == NotesViewMode::Trash
                                && this.selected_note_id.is_some()
                            {
                                this.restore_note(window, cx);
                                cx.stop_propagation();
                            }
                        }
                        "d" => {
                            if modifiers.shift {
                                // Cmd+Shift+D: Insert date/time at cursor
                                this.insert_date_time(window, cx);
                                cx.stop_propagation();
                            } else {
                                this.duplicate_selected_note(window, cx);
                            }
                        }
                        "x" => {
                            if modifiers.shift {
                                // Cmd+Shift+X: Strikethrough formatting
                                this.insert_formatting("~~", "~~", window, cx);
                                cx.stop_propagation();
                            }
                            // Let default Cmd+X (cut) pass through to Input
                        }
                        "l" => {
                            if modifiers.shift {
                                // Cmd+Shift+L: Toggle checklist checkbox on current line
                                this.toggle_checklist(window, cx);
                                cx.stop_propagation();
                            } else {
                                // Cmd+L: Select current line
                                this.select_current_line(window, cx);
                                cx.stop_propagation();
                            }
                        }
                        "-" => {
                            if modifiers.shift {
                                // Cmd+Shift+-: Insert horizontal rule
                                this.insert_horizontal_rule(window, cx);
                                cx.stop_propagation();
                            }
                        }
                        "h" => {
                            if modifiers.shift {
                                // Cmd+Shift+H: Cycle heading level on current line
                                this.cycle_heading(window, cx);
                                cx.stop_propagation();
                            }
                        }
                        "v" => {
                            // Cmd+V: Smart paste — if URL on clipboard + text selected,
                            // wrap as [text](url). Otherwise let default paste proceed.
                            if this.try_smart_paste(window, cx) {
                                cx.stop_propagation();
                            }
                            // If not handled, default Cmd+V paste proceeds
                        }
                        "c" => {
                            if modifiers.shift {
                                // Cmd+Shift+C: Copy note content as markdown
                                this.copy_as_markdown(cx);
                                cx.stop_propagation();
                            }
                            // Let default Cmd+C (copy) pass through to Input
                        }
                        "e" => {
                            // Cmd+E: Inline code formatting
                            this.insert_formatting("`", "`", window, cx);
                            cx.stop_propagation();
                        }
                        "/" => {
                            // Cmd+/: Toggle keyboard shortcuts help
                            this.toggle_shortcuts_help(cx);
                            cx.stop_propagation();
                        }
                        "j" => {
                            // Cmd+J: Join current line with next line
                            this.join_lines(window, cx);
                            cx.stop_propagation();
                        }
                        "u" => {
                            if modifiers.shift {
                                // Cmd+Shift+U: Cycle case of selected text
                                this.transform_case(window, cx);
                                cx.stop_propagation();
                            }
                        }
                        "b" => this.insert_formatting("**", "**", window, cx),
                        "i" => {
                            if modifiers.shift {
                                // Cmd+Shift+I: Toggle pin on current note
                                this.toggle_pin_current_note(cx);
                            } else {
                                this.insert_formatting("_", "_", window, cx);
                            }
                        }
                        // Navigate between notes with Cmd+Up/Down (Shift = first/last)
                        "up" | "arrowup" => {
                            if modifiers.shift {
                                this.select_first_note(window, cx);
                            } else {
                                this.select_prev_note(window, cx);
                            }
                            cx.stop_propagation();
                        }
                        "down" | "arrowdown" => {
                            if modifiers.shift {
                                this.select_last_note(window, cx);
                            } else {
                                this.select_next_note(window, cx);
                            }
                            cx.stop_propagation();
                        }
                        // History navigation: Cmd+[ back, Cmd+] forward
                        "[" => {
                            this.navigate_back(window, cx);
                            cx.stop_propagation();
                        }
                        "]" => {
                            this.navigate_forward(window, cx);
                            cx.stop_propagation();
                        }
                        // Delete current note: Cmd+Backspace
                        "backspace" | "delete" => {
                            if this.selected_note_id.is_some() {
                                this.delete_selected_note(cx);
                                // Load the newly selected note into the editor
                                if let Some(id) = this.selected_note_id {
                                    let content = this
                                        .notes
                                        .iter()
                                        .find(|n| n.id == id)
                                        .map(|n| n.content.clone())
                                        .unwrap_or_default();
                                    let content_len = content.len();
                                    this.editor_state.update(cx, |state, cx| {
                                        state.set_value(&content, window, cx);
                                        state.set_selection(content_len, content_len, window, cx);
                                    });
                                } else {
                                    this.editor_state.update(cx, |state, cx| {
                                        state.set_value("", window, cx);
                                    });
                                }
                                cx.stop_propagation();
                            }
                        }
                        // Cmd+Shift+7: Toggle numbered list
                        "7" if modifiers.shift => {
                            this.toggle_numbered_list(window, cx);
                            cx.stop_propagation();
                        }
                        // Cmd+Shift+8: Toggle bullet list
                        "8" if modifiers.shift => {
                            this.toggle_bullet_list(window, cx);
                            cx.stop_propagation();
                        }
                        // Cmd+1 through Cmd+9: Jump to pinned notes (without shift)
                        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                            if !modifiers.shift {
                                if let Ok(num) = key.parse::<usize>() {
                                    this.select_pinned_note_by_index(num - 1, window, cx);
                                    cx.stop_propagation();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }))
            // Single note view - editor takes full width
            .child(self.render_editor(cx))
            // Legacy actions panel overlay (only shown when not using CommandBar)
            .when(show_actions, |d| {
                d.child(self.render_actions_panel_overlay(cx))
            })
            // Keyboard shortcuts help overlay (Cmd+/)
            .when(self.show_shortcuts_help, |d| {
                d.child(self.render_shortcuts_help(cx))
            })
        // Note: browse panel overlay removed - note_switcher now uses CommandBar
        // which renders in its own window, not as an overlay
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
            // Close any open CommandBar windows (command_bar and note_switcher)
            // They use a global singleton, so we close it via the actions module
            crate::actions::close_actions_window(cx);
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
        *notes_app_for_closure
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(view.clone());
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
        // Store the entity globally for quick_capture access
        {
            let slot = NOTES_APP_ENTITY.get_or_init(|| std::sync::Mutex::new(None));
            if let Ok(mut g) = slot.lock() {
                *g = Some(notes_app.clone());
            }
        }

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
///
/// Creates a new empty note and focuses the editor immediately,
/// providing a frictionless capture experience like Apple Quick Note (Fn+Q)
/// or Raycast's Option-click menu bar.
pub fn quick_capture(cx: &mut App) -> Result<()> {
    use crate::logging;

    // Get existing window and app entity
    let existing_handle = {
        let slot = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    let existing_app = {
        let slot = NOTES_APP_ENTITY.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| g.clone())
    };

    // If window exists with valid app entity, create new note in existing window
    if let (Some(handle), Some(notes_app)) = (existing_handle, existing_app) {
        let result = handle.update(cx, |_root, window, cx| {
            notes_app.update(cx, |app, cx| {
                app.create_note(window, cx);
            });
        });

        if result.is_ok() {
            logging::log(
                "PANEL",
                "Quick capture: created new note in existing window",
            );
            return Ok(());
        }
        // Handle was invalid, fall through to create new window
    }

    // Window doesn't exist - create new window with a new note
    open_notes_window(cx)?;

    // After window is created, create a new note using the stored entity
    let handle = {
        let slot = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| *g)
    };

    let notes_app = {
        let slot = NOTES_APP_ENTITY.get_or_init(|| std::sync::Mutex::new(None));
        slot.lock().ok().and_then(|g| g.clone())
    };

    if let (Some(handle), Some(notes_app)) = (handle, notes_app) {
        let _ = handle.update(cx, |_root, window, cx| {
            notes_app.update(cx, |app, cx| {
                app.create_note(window, cx);
            });
        });
        logging::log("PANEL", "Quick capture: created new window with new note");
    }

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
    let guard = window_handle.lock().unwrap_or_else(|e| e.into_inner());
    guard.is_some()
}

/// Check if the given window handle matches the Notes window
///
/// Returns true if the window is the Notes window.
/// Used by keystroke interceptors to avoid handling keys meant for Notes.
pub fn is_notes_window(window: &gpui::Window) -> bool {
    let window_handle = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(guard) = window_handle.lock() {
        if let Some(notes_handle) = guard.as_ref() {
            // Convert WindowHandle<Root> to AnyWindowHandle via Into trait
            let notes_any: gpui::AnyWindowHandle = (*notes_handle).into();
            return window.window_handle() == notes_any;
        }
    }
    false
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
                        let theme = crate::theme::load_theme();
                        let is_dark = theme.should_use_dark_vibrancy();
                        crate::platform::configure_secondary_window_vibrancy(
                            window, "Notes", is_dark,
                        );

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

#[cfg(test)]
mod tests {
    use super::NotesApp;

    #[test]
    fn formatting_replacement_wraps_selected_text() {
        let value = "hello world";
        let selection = 6..11;

        let (replacement, new_selection) =
            NotesApp::formatting_replacement(value, selection.clone(), "**", "**");

        let new_value = format!(
            "{}{}{}",
            &value[..selection.start],
            replacement,
            &value[selection.end..]
        );

        assert_eq!(new_value, "hello **world**");
        assert_eq!(new_selection, 8..13);
    }

    #[test]
    fn formatting_replacement_inserts_and_positions_cursor() {
        let value = "hello";
        let selection = 2..2;

        let (replacement, new_selection) =
            NotesApp::formatting_replacement(value, selection.clone(), "**", "**");

        let new_value = format!(
            "{}{}{}",
            &value[..selection.start],
            replacement,
            &value[selection.end..]
        );

        assert_eq!(new_value, "he****llo");
        assert_eq!(new_selection, 4..4);
    }
}
