//! Browse Panel for Notes
//!
//! A modal overlay component triggered by Cmd+P that displays a searchable list
//! of notes. Follows Raycast's browse panel design pattern.
//!
//! ## Features
//! - Search input at top with "Search for notes..." placeholder
//! - "Notes" section header
//! - Note rows showing: current indicator (red dot), title, character count
//! - Hover reveals pin/delete action icons
//! - Keyboard navigation (arrow keys, enter to select, escape to close)
//! - Filter notes as user types in search

use gpui::{
    div, prelude::*, px, rgba, uniform_list, AnyElement, App, Context, ElementId, Entity,
    FocusHandle, Focusable, IntoElement, KeyDownEvent, MouseButton, ParentElement, Render,
    ScrollStrategy, SharedString, Styled, Subscription, UniformListScrollHandle, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState},
    theme::ActiveTheme,
    tooltip::Tooltip,
    IconName, Sizable,
};
use crate::ui_foundation::{is_key_down, is_key_enter, is_key_escape, is_key_up};

use super::model::{Note, NoteId};

/// Lightweight note data for display in the browse panel
#[derive(Debug, Clone)]
pub struct NoteListItem {
    /// Note identifier
    pub id: NoteId,
    /// Note title (or "Untitled Note" if empty)
    pub title: String,
    /// Character count
    pub char_count: usize,
    /// Whether this is the currently selected note
    pub is_current: bool,
    /// Whether this note is pinned
    pub is_pinned: bool,
}

impl NoteListItem {
    /// Create a NoteListItem from a Note
    pub fn from_note(note: &Note, is_current: bool) -> Self {
        Self {
            id: note.id,
            title: if note.title.is_empty() {
                "Untitled Note".to_string()
            } else {
                note.title.clone()
            },
            char_count: note.char_count(),
            is_current,
            is_pinned: note.is_pinned,
        }
    }
}

/// Callback type for note selection
pub type OnSelectNote = Box<dyn Fn(NoteId) + 'static>;

/// Callback type for panel close
pub type OnClose = Box<dyn Fn() + 'static>;

/// Callback type for note actions (pin, delete)
pub type OnNoteAction = Box<dyn Fn(NoteId, NoteAction) + 'static>;

/// Actions that can be performed on a note from the browse panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteAction {
    /// Toggle pin status
    TogglePin,
    /// Delete the note
    Delete,
}

/// Tracks the user's most recent list-navigation input source.
///
/// Keyboard mode shows only keyboard selection highlight.
/// Mouse mode shows only mouse hover highlight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum InputMode {
    #[default]
    Mouse,
    Keyboard,
}

/// Browse Panel - modal overlay for browsing and selecting notes
///
/// This component is designed to be rendered as an overlay on top of the
/// main notes window. It handles:
/// - Search input with filtering
/// - Arrow key navigation
/// - Enter to select, Escape to close
/// - Pin/delete actions on hover
pub struct BrowsePanel {
    /// All notes (filtered by search)
    notes: Vec<NoteListItem>,
    /// Original unfiltered notes
    all_notes: Vec<NoteListItem>,
    /// Currently highlighted index in the list
    selected_index: usize,
    /// Search input state
    search_state: Entity<InputState>,
    /// Focus handle for keyboard events
    focus_handle: FocusHandle,
    /// Scroll handle for virtualized list scrolling
    scroll_handle: UniformListScrollHandle,
    /// Index of note row being hovered (for showing action icons)
    hovered_index: Option<usize>,
    /// Last input source for list highlighting behavior
    input_mode: InputMode,
    /// Callback when a note is selected
    on_select: Option<OnSelectNote>,
    /// Callback when panel should close
    on_close: Option<OnClose>,
    /// Callback for note actions
    on_action: Option<OnNoteAction>,
    /// Subscriptions to keep alive
    _subscriptions: Vec<Subscription>,
}

impl BrowsePanel {
    /// Create a new BrowsePanel with the given notes
    ///
    /// # Arguments
    /// * `notes` - List of notes to display
    /// * `window` - Window reference for input state
    /// * `cx` - Context for creating entities
    pub fn new(notes: Vec<NoteListItem>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search for notes..."));

        let focus_handle = cx.focus_handle();

        // Subscribe to search input changes
        let search_sub = cx.subscribe_in(&search_state, window, {
            move |this, _, ev: &InputEvent, _window, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.on_search_change(cx);
                }
            }
        });

        Self {
            notes: notes.clone(),
            all_notes: notes,
            selected_index: 0,
            search_state,
            focus_handle,
            scroll_handle: UniformListScrollHandle::new(),
            hovered_index: None,
            input_mode: InputMode::Mouse,
            on_select: None,
            on_close: None,
            on_action: None,
            _subscriptions: vec![search_sub],
        }
    }

    /// Set the callback for note selection
    pub fn on_select(mut self, callback: impl Fn(NoteId) + 'static) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    /// Set the callback for panel close
    pub fn on_close(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_close = Some(Box::new(callback));
        self
    }

    /// Focus the search input
    pub fn focus_search(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.search_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });
    }

    /// Set the callback for note actions
    pub fn on_action(mut self, callback: impl Fn(NoteId, NoteAction) + 'static) -> Self {
        self.on_action = Some(Box::new(callback));
        self
    }

    /// Update the notes list
    pub fn set_notes(&mut self, notes: Vec<NoteListItem>, cx: &mut Context<Self>) {
        self.all_notes = notes.clone();
        self.notes = notes;
        self.selected_index = 0;
        if !self.notes.is_empty() {
            self.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
        }
        cx.notify();
    }

    /// Handle search input changes
    fn on_search_change(&mut self, cx: &mut Context<Self>) {
        let query = self
            .search_state
            .read(cx)
            .value()
            .to_string()
            .to_lowercase();

        if query.is_empty() {
            self.notes = self.all_notes.clone();
        } else {
            self.notes = self
                .all_notes
                .iter()
                .filter(|note| note.title.to_lowercase().contains(&query))
                .cloned()
                .collect();
        }

        // Reset selection to first item
        self.selected_index = 0;
        if !self.notes.is_empty() {
            self.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
        }
        cx.notify();
    }

    /// Switch to keyboard navigation mode and clear stale mouse hover state.
    fn enter_keyboard_mode(&mut self) -> bool {
        let mut changed = false;
        if self.input_mode != InputMode::Keyboard {
            self.input_mode = InputMode::Keyboard;
            changed = true;
        }

        if self.hovered_index.is_some() {
            self.hovered_index = None;
            changed = true;
        }

        changed
    }

    /// Move selection up
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        let mode_changed = self.enter_keyboard_mode();

        if !self.notes.is_empty() {
            self.selected_index = self.selected_index.saturating_sub(1);
            self.scroll_handle
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
            cx.notify();
        } else if mode_changed {
            cx.notify();
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        let mode_changed = self.enter_keyboard_mode();

        if !self.notes.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.notes.len() - 1);
            self.scroll_handle
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
            cx.notify();
        } else if mode_changed {
            cx.notify();
        }
    }

    /// Select the current note
    fn select_current(&mut self, _cx: &mut Context<Self>) {
        if let Some(note) = self.notes.get(self.selected_index) {
            if let Some(ref on_select) = self.on_select {
                on_select(note.id);
            }
        }
    }

    /// Get the currently selected note ID (for parent window keyboard handling)
    pub fn get_selected_note_id(&self) -> Option<NoteId> {
        self.notes.get(self.selected_index).map(|n| n.id)
    }

    /// Close the panel
    fn close(&self) {
        if let Some(ref on_close) = self.on_close {
            on_close();
        }
    }

    /// Handle note action (pin/delete)
    fn handle_action(&self, id: NoteId, action: NoteAction) {
        if let Some(ref on_action) = self.on_action {
            on_action(id, action);
        }
    }

    /// Render the search input
    fn render_search(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .px_3()
            .py_2()
            .child(Input::new(&self.search_state).w_full().small())
    }

    /// Render the section header
    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .px_3()
            .py_1()
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(cx.theme().muted_foreground)
            .child("Notes")
    }

    /// Render a single note row
    fn render_note_row(
        &self,
        index: usize,
        note: &NoteListItem,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_selected = index == self.selected_index && self.input_mode == InputMode::Keyboard;
        let is_hovered = self.hovered_index == Some(index) && self.input_mode == InputMode::Mouse;
        let note_id = note.id;
        let note_title = note.title.clone();

        // Row background based on state
        let bg_color = if is_selected {
            cx.theme().list_active
        } else if is_hovered {
            cx.theme().list_hover
        } else {
            gpui::transparent_black()
        };

        div()
            .id(("note-row", index))
            .w_full()
            .h(px(36.))
            .px_3()
            .flex()
            .items_center()
            .gap_2()
            .bg(bg_color)
            .rounded_sm()
            .cursor_pointer()
            .on_mouse_move(cx.listener(move |this, _, _, cx| {
                if this.input_mode != InputMode::Mouse || this.hovered_index != Some(index) {
                    this.input_mode = InputMode::Mouse;
                    this.hovered_index = Some(index);
                    cx.notify();
                }
            }))
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                if !*hovered && this.hovered_index == Some(index) {
                    this.hovered_index = None;
                    cx.notify();
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.input_mode = InputMode::Mouse;
                    this.hovered_index = Some(index);
                    this.selected_index = index;
                    this.select_current(cx);
                }),
            )
            // Current note indicator (accent dot)
            .child(
                div()
                    .w(px(8.))
                    .h(px(8.))
                    .rounded_full()
                    // Use theme accent color for current note indicator
                    .when(note.is_current, |d| d.bg(cx.theme().accent))
                    .when(!note.is_current, |d| d.bg(gpui::transparent_black())),
            )
            // Title
            .child(
                div()
                    .id(ElementId::NamedInteger(
                        SharedString::from("note-row-title-ellipsis"),
                        index as u64,
                    ))
                    .flex_1()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_sm()
                    .text_color(cx.theme().foreground)
                    .tooltip(move |window, cx| Tooltip::new(note_title.clone()).build(window, cx))
                    .child(note.title.clone()),
            )
            // Character count
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "{} character{}",
                        note.char_count,
                        if note.char_count == 1 { "" } else { "s" }
                    )),
            )
            // Action buttons (visible on hover)
            .when(is_hovered, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            Button::new(("pin", index))
                                .ghost()
                                .xsmall()
                                .icon(IconName::Star)
                                .on_click(cx.listener(move |this, _, _, _cx| {
                                    this.handle_action(note_id, NoteAction::TogglePin);
                                })),
                        )
                        .child(
                            Button::new(("delete", index))
                                .ghost()
                                .xsmall()
                                .icon(IconName::Delete)
                                .on_click(cx.listener(move |this, _, _, _cx| {
                                    this.handle_action(note_id, NoteAction::Delete);
                                })),
                        ),
                )
            })
    }

    // =====================================================
    // Vibrancy Helper Functions
    // =====================================================

    // NOTE: hex_to_rgba_with_opacity moved to crate::ui_foundation (centralized)

    /// Get background color with vibrancy opacity applied
    ///
    /// Uses cached theme to avoid file I/O on every render.
    fn get_vibrancy_background(_cx: &Context<Self>) -> gpui::Rgba {
        let sk_theme = crate::theme::get_cached_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.main;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.main,
        ))
    }

    /// Render the notes list
    fn render_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.notes.is_empty() {
            return div()
                .w_full()
                .py_8()
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("No notes found")
                .into_any_element();
        }

        let note_count = self.notes.len();
        uniform_list(
            "notes-browse-panel-list",
            note_count,
            cx.processor(
                move |this: &mut BrowsePanel,
                      visible_range: std::ops::Range<usize>,
                      _window,
                      cx| {
                    let mut rows: Vec<AnyElement> = Vec::with_capacity(visible_range.len());

                    for index in visible_range {
                        if let Some(note) = this.notes.get(index) {
                            rows.push(this.render_note_row(index, note, cx).into_any_element());
                        }
                    }

                    rows
                },
            ),
        )
        .h_full()
        .w_full()
        .track_scroll(&self.scroll_handle)
        .into_any_element()
    }
}

impl Focusable for BrowsePanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BrowsePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Modal backdrop (semi-transparent overlay - theme-aware)
        div()
            .id("browse-panel-backdrop")
            .absolute()
            .inset_0()
            .bg({
                let sk_theme = crate::theme::get_cached_theme();
                crate::theme::modal_overlay_bg(&sk_theme, 0x80)
            }) // Theme-aware: black for dark, white for light
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _cx| {
                    this.close();
                }),
            )
            // Panel container
            .child({
                // Check vibrancy to conditionally apply shadow
                // Shadows on transparent elements block vibrancy blur
                // Uses cached theme to avoid file I/O on every render
                let sk_theme = crate::theme::get_cached_theme();
                let vibrancy_enabled = sk_theme.is_vibrancy_enabled();

                div()
                    .id("browse-panel")
                    .w(px(500.))
                    .max_h(px(400.))
                    // NO .bg() here - backdrop overlay already provides visual separation
                    // Double-layering causes opacity to compound and makes things darker
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_lg()
                    // Only apply shadow when vibrancy is disabled - matches POC behavior
                    .when(!vibrancy_enabled, |d| d.shadow_lg())
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .track_focus(&self.focus_handle)
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                        let key = event.keystroke.key.as_str();
                        match key {
                            k if is_key_up(k) => this.move_up(cx),
                            k if is_key_down(k) => this.move_down(cx),
                            k if is_key_enter(k) => this.select_current(cx),
                            k if is_key_escape(k) => this.close(),
                            _ => {}
                        }
                    }))
                    // Prevent backdrop click from closing when clicking panel
                    .on_mouse_down(MouseButton::Left, |_, _, _| {})
                    // Search input
                    .child(self.render_search(cx))
                    // Section header
                    .child(self.render_header(cx))
                    // Notes list (scrollable)
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .px_1()
                            .py_1()
                            .on_mouse_move(cx.listener(|this, _, _, cx| {
                                let mut changed = false;
                                if this.input_mode != InputMode::Mouse {
                                    this.input_mode = InputMode::Mouse;
                                    changed = true;
                                }
                                if this.hovered_index.is_some() {
                                    this.hovered_index = None;
                                    changed = true;
                                }
                                if changed {
                                    cx.notify();
                                }
                            }))
                            .child(self.render_list(cx)),
                    )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const BROWSE_PANEL_SOURCE: &str = include_str!("browse_panel.rs");

    #[test]
    fn test_note_list_item_from_note() {
        use chrono::Utc;

        let note = Note {
            id: NoteId::new(),
            title: "Test Note".to_string(),
            content: "Hello, world!".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_pinned: false,
            sort_order: 0,
        };

        let item = NoteListItem::from_note(&note, true);
        assert_eq!(item.title, "Test Note");
        assert_eq!(item.char_count, 13);
        assert!(item.is_current);
        assert!(!item.is_pinned);
    }

    #[test]
    fn test_note_list_item_untitled() {
        use chrono::Utc;

        let note = Note {
            id: NoteId::new(),
            title: "".to_string(),
            content: "Some content".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_pinned: true,
            sort_order: 0,
        };

        let item = NoteListItem::from_note(&note, false);
        assert_eq!(item.title, "Untitled Note");
        assert!(!item.is_current);
        assert!(item.is_pinned);
    }

    #[test]
    fn test_render_list_uses_uniform_list_virtualization() {
        assert!(
            BROWSE_PANEL_SOURCE.contains("uniform_list("),
            "BrowsePanel should use uniform_list for virtualized note rows"
        );
        assert!(
            BROWSE_PANEL_SOURCE.contains(".track_scroll(&self.scroll_handle)"),
            "BrowsePanel list should track the dedicated scroll handle"
        );
    }

    #[test]
    fn test_hover_handlers_clear_stale_row_hover_state() {
        assert!(
            BROWSE_PANEL_SOURCE.contains("this.hovered_index == Some(index)"),
            "Row hover handler should clear hovered row when pointer leaves the row"
        );
    }

    #[test]
    fn test_list_hover_leave_clears_hovered_index() {
        assert!(
            BROWSE_PANEL_SOURCE.contains(".on_mouse_move(cx.listener(|this, _, _, cx| {"),
            "List container should register a mouse move handler"
        );
        assert!(
            BROWSE_PANEL_SOURCE.contains("if this.hovered_index.is_some() {"),
            "List container mouse move handler should detect stale hovered row state"
        );
        assert!(
            BROWSE_PANEL_SOURCE.contains("this.hovered_index = None;"),
            "List container mouse move handler should clear hovered_index when pointer leaves list area"
        );
    }

    #[test]
    fn test_highlight_modes_are_mutually_exclusive() {
        assert!(
            BROWSE_PANEL_SOURCE
                .contains("index == self.selected_index && self.input_mode == InputMode::Keyboard"),
            "Selected row highlight should only render in keyboard mode"
        );
        assert!(
            BROWSE_PANEL_SOURCE
                .contains("self.hovered_index == Some(index) && self.input_mode == InputMode::Mouse"),
            "Hover highlight should only render in mouse mode"
        );
    }

    #[test]
    fn test_arrow_navigation_switches_to_keyboard_mode() {
        assert!(
            BROWSE_PANEL_SOURCE.contains("fn enter_keyboard_mode(&mut self) -> bool"),
            "BrowsePanel should define a keyboard-mode helper"
        );
        assert!(
            BROWSE_PANEL_SOURCE.contains("self.input_mode = InputMode::Keyboard;"),
            "Keyboard navigation should switch input mode to Keyboard"
        );
        assert!(
            BROWSE_PANEL_SOURCE.contains("self.hovered_index = None;"),
            "Keyboard navigation should clear hovered row state"
        );
    }

    #[test]
    fn test_mouse_move_switches_to_mouse_mode() {
        assert!(
            BROWSE_PANEL_SOURCE.contains("this.input_mode = InputMode::Mouse;"),
            "Mouse movement should switch input mode to Mouse"
        );
    }
}
