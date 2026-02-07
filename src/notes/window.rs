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

// =============================================================================
// Auto-resize constants — govern how the window grows / shrinks with content.
// =============================================================================

/// Combined top + bottom padding in the editor area (px).
const AUTO_RESIZE_PADDING: f32 = 24.0;

/// Approximate per-line height used by the text layout (px).
const AUTO_RESIZE_LINE_HEIGHT: f32 = 20.0;

/// Absolute ceiling — the window will never auto-grow beyond this (px).
const AUTO_RESIZE_MAX_HEIGHT: f32 = 600.0;

/// Minimum delta before we bother resizing (avoids 1-px jitter).
const AUTO_RESIZE_THRESHOLD: f32 = 5.0;

/// Delta beyond which we assume the user grabbed the edge to resize manually.
const MANUAL_RESIZE_THRESHOLD: f32 = 10.0;

// =============================================================================
// Timing constants
// =============================================================================

/// Duration (ms) of the brief "✓ Saved" flash after a successful save.
const SAVED_FLASH_MS: u64 = 1500;

/// Duration (ms) of action feedback flash ("Deleted", "Pinned", etc.).
const ACTION_FEEDBACK_MS: u64 = 2000;

// =============================================================================
// Modal overlay tokens (same palette as browse_panel.rs)
// =============================================================================

/// Dark-mode modal overlay: black at 50 % alpha.
const MODAL_OVERLAY_DARK: u32 = 0x00000080;

/// Light-mode modal overlay: white at 50 % alpha.
const MODAL_OVERLAY_LIGHT: u32 = 0xffffff80;

/// Corner radius for shortcuts panel card (px).
const SHORTCUTS_PANEL_RADIUS: f32 = 10.0;

/// Extra vertical space reserved around the actions panel overlay (px).
const ACTIONS_PANEL_WINDOW_MARGIN: f32 = 64.0;

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

mod clipboard_ops;
mod editor_formatting;
mod editor_ops_a;
mod editor_ops_b;
mod init;
mod keyboard;
mod navigation;
mod notes;
mod notes_actions;
mod panels;
mod render;
mod render_editor;
mod render_editor_body;
mod render_editor_footer;
mod render_editor_titlebar;
mod render_overlays;
mod render_shortcuts;
mod render_ui;
mod traits;
mod vibrancy;
mod window_ops;

pub use window_ops::{
    close_notes_window, is_notes_window, is_notes_window_open, open_notes_window, quick_capture,
};

#[cfg(test)]
mod tests;
