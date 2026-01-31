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
    div, prelude::*, px, rgba, App, Context, Entity, FocusHandle, Focusable, IntoElement,
    KeyDownEvent, MouseButton, ParentElement, Render, Styled, Subscription, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState},
    theme::ActiveTheme,
    IconName, Sizable,
};

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
    /// Index of note row being hovered (for showing action icons)
    hovered_index: Option<usize>,
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
            hovered_index: None,
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
        cx.notify();
    }

    /// Move selection up
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        if !self.notes.is_empty() {
            self.selected_index = self.selected_index.saturating_sub(1);
            cx.notify();
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        if !self.notes.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.notes.len() - 1);
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
        let is_selected = index == self.selected_index;
        let is_hovered = self.hovered_index == Some(index);
        let note_id = note.id;

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
                if this.hovered_index != Some(index) {
                    this.hovered_index = Some(index);
                    cx.notify();
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
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
                    .flex_1()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_sm()
                    .text_color(cx.theme().foreground)
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
    fn get_vibrancy_background(_cx: &Context<Self>) -> gpui::Rgba {
        let sk_theme = crate::theme::load_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.main;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.main,
        ))
    }

    /// Get modal overlay background (theme-aware)
    ///
    /// For dark mode: black overlay (darkens content behind)
    /// For light mode: white overlay (keeps content readable on light backgrounds)
    fn get_modal_overlay_background() -> gpui::Rgba {
        let sk_theme = crate::theme::load_theme();
        if sk_theme.has_dark_colors() {
            rgba(0x00000080) // black at 50% for dark mode
        } else {
            rgba(0xffffff80) // white at 50% for light mode
        }
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

        let mut list = div().w_full().flex().flex_col().gap_px();

        for (index, note) in self.notes.iter().enumerate() {
            list = list.child(self.render_note_row(index, note, cx));
        }

        list.into_any_element()
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
            .bg(Self::get_modal_overlay_background()) // Theme-aware: black for dark, white for light
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
            .child(
                div()
                    .id("browse-panel")
                    .w(px(500.))
                    .max_h(px(400.))
                    .bg(Self::get_vibrancy_background(cx)) // Vibrancy-aware background
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_lg()
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .track_focus(&self.focus_handle)
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                        let key = event.keystroke.key.as_str();
                        match key {
                            "up" | "arrowup" => this.move_up(cx),
                            "down" | "arrowdown" => this.move_down(cx),
                            "enter" | "return" => this.select_current(cx),
                            "escape" | "esc" => this.close(),
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
                                // Clear hover when mouse leaves list area without entering a row
                                if this.hovered_index.is_some() {
                                    // This will be overridden by row hover handlers
                                }
                                let _ = cx;
                            }))
                            .child(self.render_list(cx)),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
