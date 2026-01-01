//! Notes Actions Panel
//!
//! Modal overlay panel triggered by Cmd+K in the Notes window.
//! Provides searchable action list for note operations.
//!
//! ## Actions
//! - New Note (⌘N) - Create a new note
//! - Browse Notes (⌘P) - Open note browser/picker
//! - Find in Note (⌘F) - Search within current note
//! - Copy Note (⌘C) - Copy note content to clipboard
//! - Delete Note (⌘D) - Delete current note
//!
//! ## Keyboard Navigation
//! - Arrow Up/Down: Navigate actions
//! - Enter: Execute selected action
//! - Escape: Close panel
//! - Type to search/filter actions

use gpui::{
    div, point, prelude::*, px, uniform_list, App, BoxShadow, Context, FocusHandle, Focusable,
    Hsla, Render, ScrollStrategy, SharedString, UniformListScrollHandle, Window,
};
use gpui_component::{theme::ActiveTheme, Icon, IconName};
use std::sync::Arc;
use tracing::debug;

/// Callback type for action execution
/// The String parameter is the action ID
pub type NotesActionCallback = Arc<dyn Fn(NotesAction) + Send + Sync>;

/// Available actions in the Notes actions panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotesAction {
    /// Create a new note
    NewNote,
    /// Open the note browser/picker
    BrowseNotes,
    /// Search within the current note
    FindInNote,
    /// Copy note content to clipboard
    CopyNote,
    /// Delete the current note
    DeleteNote,
    /// Enable auto-sizing (window grows/shrinks with content)
    EnableAutoSizing,
    /// Panel was cancelled (Escape pressed)
    Cancel,
}

impl NotesAction {
    /// Get all available actions (excluding Cancel)
    pub fn all() -> &'static [NotesAction] {
        &[
            NotesAction::NewNote,
            NotesAction::BrowseNotes,
            NotesAction::FindInNote,
            NotesAction::CopyNote,
            NotesAction::DeleteNote,
            NotesAction::EnableAutoSizing,
        ]
    }

    /// Get the display label for this action
    pub fn label(&self) -> &'static str {
        match self {
            NotesAction::NewNote => "New Note",
            NotesAction::BrowseNotes => "Browse Notes",
            NotesAction::FindInNote => "Find in Note",
            NotesAction::CopyNote => "Copy Note",
            NotesAction::DeleteNote => "Delete Note",
            NotesAction::EnableAutoSizing => "Enable Auto-Sizing",
            NotesAction::Cancel => "Cancel",
        }
    }

    /// Get the keyboard shortcut key (without modifier)
    pub fn shortcut_key(&self) -> &'static str {
        match self {
            NotesAction::NewNote => "N",
            NotesAction::BrowseNotes => "P",
            NotesAction::FindInNote => "F",
            NotesAction::CopyNote => "C",
            NotesAction::DeleteNote => "D",
            NotesAction::EnableAutoSizing => "A",
            NotesAction::Cancel => "Esc",
        }
    }

    /// Get the formatted shortcut display string
    pub fn shortcut_display(&self) -> String {
        match self {
            NotesAction::Cancel => "Esc".to_string(),
            _ => format!("⌘{}", self.shortcut_key()),
        }
    }

    /// Get the icon for this action
    pub fn icon(&self) -> IconName {
        match self {
            NotesAction::NewNote => IconName::Plus,
            NotesAction::BrowseNotes => IconName::File,
            NotesAction::FindInNote => IconName::Search,
            NotesAction::CopyNote => IconName::Copy,
            NotesAction::DeleteNote => IconName::Delete,
            NotesAction::EnableAutoSizing => IconName::Maximize,
            NotesAction::Cancel => IconName::Close,
        }
    }

    /// Get action ID for lookup
    pub fn id(&self) -> &'static str {
        match self {
            NotesAction::NewNote => "new_note",
            NotesAction::BrowseNotes => "browse_notes",
            NotesAction::FindInNote => "find_in_note",
            NotesAction::CopyNote => "copy_note",
            NotesAction::DeleteNote => "delete_note",
            NotesAction::EnableAutoSizing => "enable_auto_sizing",
            NotesAction::Cancel => "cancel",
        }
    }
}

/// Panel dimensions and styling constants (matches main ActionsDialog)
pub const PANEL_WIDTH: f32 = 320.0;
pub const PANEL_MAX_HEIGHT: f32 = 400.0;
pub const PANEL_CORNER_RADIUS: f32 = 12.0;
pub const ACTION_ITEM_HEIGHT: f32 = 42.0;
pub const ACCENT_BAR_WIDTH: f32 = 3.0;

/// Notes Actions Panel - Modal overlay for note operations
pub struct NotesActionsPanel {
    /// Available actions
    actions: Vec<NotesAction>,
    /// Filtered action indices
    filtered_indices: Vec<usize>,
    /// Currently selected index (within filtered)
    selected_index: usize,
    /// Search text
    search_text: String,
    /// Focus handle
    focus_handle: FocusHandle,
    /// Callback for action selection
    on_action: NotesActionCallback,
    /// Scroll handle for virtualization
    scroll_handle: UniformListScrollHandle,
    /// Cursor blink visibility
    cursor_visible: bool,
}

impl NotesActionsPanel {
    /// Create a new NotesActionsPanel
    pub fn new(focus_handle: FocusHandle, on_action: NotesActionCallback) -> Self {
        let actions: Vec<NotesAction> = NotesAction::all().to_vec();
        let filtered_indices: Vec<usize> = (0..actions.len()).collect();

        debug!(action_count = actions.len(), "Notes actions panel created");

        Self {
            actions,
            filtered_indices,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            on_action,
            scroll_handle: UniformListScrollHandle::new(),
            cursor_visible: true,
        }
    }

    /// Set cursor visibility (for blink animation)
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.search_text.push(ch);
        self.refilter();
        cx.notify();
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.search_text.is_empty() {
            self.search_text.pop();
            self.refilter();
            cx.notify();
        }
    }

    /// Move selection up
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.scroll_handle
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
            cx.notify();
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.filtered_indices.len().saturating_sub(1) {
            self.selected_index += 1;
            self.scroll_handle
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
            cx.notify();
        }
    }

    /// Submit the selected action
    pub fn submit_selected(&mut self) {
        if let Some(&action_idx) = self.filtered_indices.get(self.selected_index) {
            if let Some(action) = self.actions.get(action_idx) {
                debug!(action = ?action, "Notes action selected");
                (self.on_action)(*action);
            }
        }
    }

    /// Cancel and close
    pub fn cancel(&mut self) {
        debug!("Notes actions panel cancelled");
        (self.on_action)(NotesAction::Cancel);
    }

    /// Get currently selected action
    pub fn get_selected_action(&self) -> Option<NotesAction> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.actions.get(idx))
            .copied()
    }

    /// Refilter actions based on search text
    fn refilter(&mut self) {
        if self.search_text.is_empty() {
            self.filtered_indices = (0..self.actions.len()).collect();
        } else {
            let search_lower = self.search_text.to_lowercase();
            self.filtered_indices = self
                .actions
                .iter()
                .enumerate()
                .filter(|(_, action)| action.label().to_lowercase().contains(&search_lower))
                .map(|(idx, _)| idx)
                .collect();
        }

        // Reset selection if out of bounds
        if self.selected_index >= self.filtered_indices.len() {
            self.selected_index = 0;
        }

        // Scroll to keep selection visible
        if !self.filtered_indices.is_empty() {
            self.scroll_handle
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
        }
    }

    /// Create box shadow for the overlay
    fn create_shadow() -> Vec<BoxShadow> {
        vec![
            BoxShadow {
                color: Hsla {
                    h: 0.0,
                    s: 0.0,
                    l: 0.0,
                    a: 0.3,
                },
                offset: point(px(0.0), px(4.0)),
                blur_radius: px(16.0),
                spread_radius: px(0.0),
            },
            BoxShadow {
                color: Hsla {
                    h: 0.0,
                    s: 0.0,
                    l: 0.0,
                    a: 0.15,
                },
                offset: point(px(0.0), px(8.0)),
                blur_radius: px(32.0),
                spread_radius: px(-4.0),
            },
        ]
    }
}

impl Focusable for NotesActionsPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NotesActionsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // Colors from gpui-component theme
        let bg_color = theme.background;
        let border_color = theme.border;
        let text_primary = theme.foreground;
        let text_muted = theme.muted_foreground;
        let accent_color = theme.accent;

        // Search display
        let search_display = if self.search_text.is_empty() {
            SharedString::from("Search actions...")
        } else {
            SharedString::from(self.search_text.clone())
        };

        // Build search input row
        let search_input = div()
            .w_full()
            .h(px(44.0))
            .px(px(12.0))
            .py(px(8.0))
            .bg(theme.secondary)
            .border_t_1()
            .border_color(border_color)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            // Cmd+K indicator
            .child(
                div()
                    .w(px(24.0))
                    .text_color(text_muted)
                    .text_xs()
                    .child("⌘K"),
            )
            // Search field
            .child(
                div()
                    .flex_1()
                    .h(px(28.0))
                    .px(px(8.0))
                    .bg(theme.input)
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(if self.search_text.is_empty() {
                        border_color
                    } else {
                        accent_color
                    })
                    .flex()
                    .flex_row()
                    .items_center()
                    .text_sm()
                    .text_color(if self.search_text.is_empty() {
                        text_muted
                    } else {
                        text_primary
                    })
                    // Cursor when empty
                    .when(self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(2.))
                                .h(px(16.))
                                .mr(px(2.))
                                .rounded(px(1.))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    })
                    .child(search_display)
                    // Cursor when has text
                    .when(!self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(2.))
                                .h(px(16.))
                                .ml(px(2.))
                                .rounded(px(1.))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    }),
            );

        // Build actions list
        let selected_index = self.selected_index;
        let filtered_len = self.filtered_indices.len();

        let actions_list = if self.filtered_indices.is_empty() {
            div()
                .flex_1()
                .w_full()
                .py(px(16.0))
                .px(px(12.0))
                .text_color(text_muted)
                .text_sm()
                .child("No actions match your search")
                .into_any_element()
        } else {
            uniform_list(
                "notes-actions-list",
                filtered_len,
                cx.processor(
                    move |this: &mut NotesActionsPanel, visible_range, _window, cx| {
                        let theme = cx.theme();
                        let mut items = Vec::new();

                        for idx in visible_range {
                            if let Some(&action_idx) = this.filtered_indices.get(idx) {
                                if let Some(action) = this.actions.get(action_idx) {
                                    let action: &NotesAction = action;
                                    let is_selected = idx == selected_index;

                                    // Transparent Hsla for unselected state
                                    let transparent = Hsla {
                                        h: 0.0,
                                        s: 0.0,
                                        l: 0.0,
                                        a: 0.0,
                                    };

                                    let action_row = div()
                                        .id(idx)
                                        .w_full()
                                        .h(px(ACTION_ITEM_HEIGHT))
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .bg(if is_selected {
                                            theme.list_active
                                        } else {
                                            transparent
                                        })
                                        .hover(|s| s.bg(theme.list_hover))
                                        .cursor_pointer()
                                        // Left accent bar for selection
                                        .border_l(px(ACCENT_BAR_WIDTH))
                                        .border_color(if is_selected {
                                            theme.accent
                                        } else {
                                            transparent
                                        })
                                        // Content
                                        .child(
                                            div()
                                                .flex_1()
                                                .px(px(12.0))
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                // Left: icon + label
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap(px(8.0))
                                                        // Icon - render using gpui_component Icon
                                                        .child({
                                                            let icon_name: IconName = action.icon();
                                                            Icon::new(icon_name)
                                                                .size_4()
                                                                .text_color(if is_selected {
                                                                    theme.foreground
                                                                } else {
                                                                    theme.muted_foreground
                                                                })
                                                        })
                                                        // Label
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .text_color(if is_selected {
                                                                    theme.foreground
                                                                } else {
                                                                    theme.muted_foreground
                                                                })
                                                                .font_weight(if is_selected {
                                                                    gpui::FontWeight::MEDIUM
                                                                } else {
                                                                    gpui::FontWeight::NORMAL
                                                                })
                                                                .child(action.label()),
                                                        ),
                                                )
                                                // Right: shortcut badge
                                                .child(
                                                    div()
                                                        .px(px(6.0))
                                                        .py(px(2.0))
                                                        .bg(theme.muted)
                                                        .rounded(px(4.0))
                                                        .text_xs()
                                                        .text_color(theme.muted_foreground)
                                                        .child(action.shortcut_display()),
                                                ),
                                        );

                                    items.push(action_row);
                                }
                            }
                        }
                        items
                    },
                ),
            )
            .flex_1()
            .w_full()
            .track_scroll(&self.scroll_handle)
            .into_any_element()
        };

        // Calculate dynamic height
        let items_height = (filtered_len as f32 * ACTION_ITEM_HEIGHT).min(PANEL_MAX_HEIGHT - 60.0);
        let total_height = items_height + 44.0 + 2.0; // search + border

        // Main container
        div()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .h(px(total_height))
            .bg(bg_color)
            .rounded(px(PANEL_CORNER_RADIUS))
            .shadow(Self::create_shadow())
            .border_1()
            .border_color(border_color)
            .overflow_hidden()
            .track_focus(&self.focus_handle)
            .child(actions_list)
            .child(search_input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notes_action_labels() {
        assert_eq!(NotesAction::NewNote.label(), "New Note");
        assert_eq!(NotesAction::BrowseNotes.label(), "Browse Notes");
        assert_eq!(NotesAction::FindInNote.label(), "Find in Note");
        assert_eq!(NotesAction::CopyNote.label(), "Copy Note");
        assert_eq!(NotesAction::DeleteNote.label(), "Delete Note");
        assert_eq!(NotesAction::EnableAutoSizing.label(), "Enable Auto-Sizing");
    }

    #[test]
    fn test_notes_action_shortcuts() {
        assert_eq!(NotesAction::NewNote.shortcut_display(), "⌘N");
        assert_eq!(NotesAction::BrowseNotes.shortcut_display(), "⌘P");
        assert_eq!(NotesAction::FindInNote.shortcut_display(), "⌘F");
        assert_eq!(NotesAction::CopyNote.shortcut_display(), "⌘C");
        assert_eq!(NotesAction::DeleteNote.shortcut_display(), "⌘D");
        assert_eq!(NotesAction::EnableAutoSizing.shortcut_display(), "⌘A");
    }

    #[test]
    fn test_notes_action_all() {
        let all = NotesAction::all();
        assert_eq!(all.len(), 6);
        assert!(all.contains(&NotesAction::NewNote));
        assert!(all.contains(&NotesAction::BrowseNotes));
        assert!(all.contains(&NotesAction::FindInNote));
        assert!(all.contains(&NotesAction::CopyNote));
        assert!(all.contains(&NotesAction::DeleteNote));
        assert!(all.contains(&NotesAction::EnableAutoSizing));
    }

    #[test]
    fn test_notes_action_ids() {
        assert_eq!(NotesAction::NewNote.id(), "new_note");
        assert_eq!(NotesAction::BrowseNotes.id(), "browse_notes");
        assert_eq!(NotesAction::FindInNote.id(), "find_in_note");
        assert_eq!(NotesAction::CopyNote.id(), "copy_note");
        assert_eq!(NotesAction::DeleteNote.id(), "delete_note");
        assert_eq!(NotesAction::EnableAutoSizing.id(), "enable_auto_sizing");
    }

    #[test]
    fn test_panel_constants() {
        // Verify panel matches main ActionsDialog dimensions
        assert_eq!(PANEL_WIDTH, 320.0);
        assert_eq!(PANEL_MAX_HEIGHT, 400.0);
        assert_eq!(PANEL_CORNER_RADIUS, 12.0);
        assert_eq!(ACTION_ITEM_HEIGHT, 42.0);
    }
}
