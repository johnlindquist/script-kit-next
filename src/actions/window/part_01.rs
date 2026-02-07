// Actions Window - Separate vibrancy window for actions panel
//
// This creates a floating popup window with its own vibrancy blur effect,
// similar to Raycast's actions panel. The window is:
// - Non-draggable (fixed position relative to main window)
// - Positioned below the header, at the right edge of main window
// - Auto-closes when app loses focus
// - Shares the ActionsDialog entity with the main app for keyboard routing

use crate::platform;
use crate::theme;
use crate::ui_foundation::{is_key_backspace, is_key_down, is_key_enter, is_key_escape, is_key_up};
use crate::window_resize::layout::FOOTER_HEIGHT;
use gpui::{
    div, prelude::*, px, App, Bounds, Context, DisplayId, Entity, FocusHandle, Focusable, Pixels,
    Point, Render, Size, Window, WindowBounds, WindowHandle, WindowKind, WindowOptions,
};
use gpui_component::Root;
use std::sync::{Mutex, OnceLock};

use super::constants::{
    ACTION_ITEM_HEIGHT, HEADER_HEIGHT, POPUP_MAX_HEIGHT, SEARCH_INPUT_HEIGHT, SECTION_HEADER_HEIGHT,
};
use super::dialog::{ActionsDialog, GroupedActionItem};
use super::types::{Action, SectionStyle};

/// Count the number of section headers in the filtered action list
/// A section header appears when an action's section differs from the previous action's section
pub(super) fn count_section_headers(actions: &[Action], filtered_indices: &[usize]) -> usize {
    if filtered_indices.is_empty() {
        return 0;
    }

    let mut count = 0;
    let mut prev_section: Option<&Option<String>> = None;

    for &idx in filtered_indices {
        if let Some(action) = actions.get(idx) {
            let current_section = &action.section;
            // Count as header if: first item with a section, or section changed
            if current_section.is_some() {
                match prev_section {
                    None => count += 1,                                  // First item with a section
                    Some(prev) if prev != current_section => count += 1, // Section changed
                    _ => {}
                }
            }
            prev_section = Some(current_section);
        }
    }

    count
}

/// Global singleton for the actions window handle
static ACTIONS_WINDOW: OnceLock<Mutex<Option<WindowHandle<Root>>>> = OnceLock::new();

const ACTIONS_WINDOW_PAGE_JUMP: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActionsWindowKeyIntent {
    MoveUp,
    MoveDown,
    MoveHome,
    MoveEnd,
    MovePageUp,
    MovePageDown,
    ExecuteSelected,
    Close,
    Backspace,
    TypeChar(char),
}

#[inline]
fn is_selectable_row(row: &GroupedActionItem) -> bool {
    matches!(row, GroupedActionItem::Item(_))
}

fn first_selectable_index(rows: &[GroupedActionItem]) -> Option<usize> {
    rows.iter().position(is_selectable_row)
}

fn last_selectable_index(rows: &[GroupedActionItem]) -> Option<usize> {
    rows.iter().rposition(is_selectable_row)
}

fn selectable_index_at_or_before(rows: &[GroupedActionItem], start: usize) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }
    let clamped = start.min(rows.len() - 1);
    (0..=clamped).rev().find(|&ix| is_selectable_row(&rows[ix]))
}

fn selectable_index_at_or_after(rows: &[GroupedActionItem], start: usize) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }
    let clamped = start.min(rows.len() - 1);
    (clamped..rows.len()).find(|&ix| is_selectable_row(&rows[ix]))
}

#[inline]
fn actions_window_key_intent(
    key: &str,
    modifiers: &gpui::Modifiers,
) -> Option<ActionsWindowKeyIntent> {
    if is_key_up(key) {
        return Some(ActionsWindowKeyIntent::MoveUp);
    }
    if is_key_down(key) {
        return Some(ActionsWindowKeyIntent::MoveDown);
    }
    if key.eq_ignore_ascii_case("home") {
        return Some(ActionsWindowKeyIntent::MoveHome);
    }
    if key.eq_ignore_ascii_case("end") {
        return Some(ActionsWindowKeyIntent::MoveEnd);
    }
    if key.eq_ignore_ascii_case("pageup") {
        return Some(ActionsWindowKeyIntent::MovePageUp);
    }
    if key.eq_ignore_ascii_case("pagedown") {
        return Some(ActionsWindowKeyIntent::MovePageDown);
    }
    if is_key_enter(key) {
        return Some(ActionsWindowKeyIntent::ExecuteSelected);
    }
    if is_key_escape(key) {
        return Some(ActionsWindowKeyIntent::Close);
    }
    if is_key_backspace(key) || key.eq_ignore_ascii_case("delete") {
        return Some(ActionsWindowKeyIntent::Backspace);
    }
    if !modifiers.platform && !modifiers.control && !modifiers.alt {
        if let Some(ch) = key.chars().next() {
            if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_' {
                return Some(ActionsWindowKeyIntent::TypeChar(ch));
            }
        }
    }
    None
}

/// Actions window width (height is calculated dynamically based on content)
const ACTIONS_WINDOW_WIDTH: f32 = 320.0;
/// Horizontal margin from main window right edge
const ACTIONS_MARGIN_X: f32 = 8.0;
/// Vertical margin from header/footer
const ACTIONS_MARGIN_Y: f32 = 8.0;
/// Titlebar height (for top-anchored positioning)
#[allow(dead_code)] // Reserved for future TopRight positioning
const TITLEBAR_HEIGHT: f32 = 36.0;

/// Window position relative to the parent window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)] // Some variants reserved for future use
pub enum WindowPosition {
    /// Bottom-right, above the footer (default for Cmd+K actions)
    #[default]
    BottomRight,
    /// Top-right, below the titlebar (for new chat dropdown)
    TopRight,
    /// Top-center, below the titlebar, horizontally centered (Raycast-style for Notes)
    TopCenter,
}

/// ActionsWindow wrapper that renders the shared ActionsDialog entity
pub struct ActionsWindow {
    /// The shared dialog entity (created by main app, rendered here)
    pub dialog: Entity<ActionsDialog>,
    /// Focus handle for this window (not actively used - main window keeps focus)
    pub focus_handle: FocusHandle,
}

impl ActionsWindow {
    pub fn new(dialog: Entity<ActionsDialog>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            dialog,
            focus_handle,
        }
    }
}

impl Focusable for ActionsWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ActionsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Log focus state AND window focus state
        let is_focused = self.focus_handle.is_focused(window);
        let window_is_active = window.is_window_active();
        crate::logging::log(
            "ACTIONS",
            &format!(
                "ActionsWindow render: focus_handle.is_focused={}, window_is_active={}",
                is_focused, window_is_active
            ),
        );

        // NOTE: We intentionally do NOT focus this window's focus_handle.
        // The parent window (AI window, Notes window, etc.) keeps keyboard focus
        // and routes events to us via its capture_key_down handler.
        // This approach works better on macOS where popup windows often don't
        // receive keyboard events reliably.
        //
        // The on_key_down handler below is still registered as a fallback for
        // cases where the popup window does receive focus (e.g., user clicks on it).

        // Key handler for the actions window
        // Since this is a separate window, it needs its own key handling
        // (the parent window can't route events to us)
        let handle_key = cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
            let key = event.keystroke.key.as_str();
            let modifiers = &event.keystroke.modifiers;

            crate::logging::log(
                "ACTIONS",
                &format!(
                    "ActionsWindow on_key_down received: key='{}', modifiers={:?}",
                    key, modifiers
                ),
            );

            match actions_window_key_intent(key, modifiers) {
                Some(ActionsWindowKeyIntent::MoveUp) => {
                    crate::logging::log("ACTIONS", "ActionsWindow: handling UP arrow");

                    this.dialog.update(cx, |d, cx| d.move_up(cx));
                    cx.notify();
                }
                Some(ActionsWindowKeyIntent::MoveDown) => {
                    crate::logging::log("ACTIONS", "ActionsWindow: handling DOWN arrow");
                    this.dialog.update(cx, |d, cx| d.move_down(cx));
                    cx.notify();
                }
                Some(ActionsWindowKeyIntent::MoveHome) => {
                    this.dialog.update(cx, |d, cx| {
                        if let Some(first) = first_selectable_index(&d.grouped_items) {
                            d.selected_index = first;
                            d.list_state.scroll_to_reveal_item(d.selected_index);
                            cx.notify();
                        }
                    });
                }
                Some(ActionsWindowKeyIntent::MoveEnd) => {
                    this.dialog.update(cx, |d, cx| {
                        if let Some(last) = last_selectable_index(&d.grouped_items) {
                            d.selected_index = last;
                            d.list_state.scroll_to_reveal_item(d.selected_index);
                            cx.notify();
                        }
                    });
                }
                Some(ActionsWindowKeyIntent::MovePageUp) => {
                    this.dialog.update(cx, |d, cx| {
                        if d.grouped_items.is_empty() {
                            return;
                        }

                        let target = d.selected_index.saturating_sub(ACTIONS_WINDOW_PAGE_JUMP);
                        if let Some(next_index) =
                            selectable_index_at_or_before(&d.grouped_items, target)
                                .or_else(|| first_selectable_index(&d.grouped_items))
                        {
                            d.selected_index = next_index;
                            d.list_state.scroll_to_reveal_item(d.selected_index);
                            cx.notify();
                        }
                    });
                }
                Some(ActionsWindowKeyIntent::MovePageDown) => {
                    this.dialog.update(cx, |d, cx| {
                        if d.grouped_items.is_empty() {
                            return;
                        }

                        let last_index = d.grouped_items.len() - 1;
                        let target = (d.selected_index + ACTIONS_WINDOW_PAGE_JUMP).min(last_index);
                        if let Some(next_index) =
                            selectable_index_at_or_after(&d.grouped_items, target)
                                .or_else(|| last_selectable_index(&d.grouped_items))
                        {
                            d.selected_index = next_index;
                            d.list_state.scroll_to_reveal_item(d.selected_index);
                            cx.notify();
                        }
                    });
                }
                Some(ActionsWindowKeyIntent::ExecuteSelected) => {
                    // Get selected action and execute via callback
                    let action_id = this.dialog.read(cx).get_selected_action_id();
                    if let Some(action_id) = action_id {
                        // Execute the action's callback
                        let callback = this.dialog.read(cx).on_select.clone();
                        callback(action_id.clone());
                        // Notify main app to restore focus before closing
                        let on_close = this.dialog.read(cx).on_close.clone();
                        if let Some(callback) = on_close {
                            callback(cx);
                        }
                        // Activate the main window so it can receive focus
                        platform::activate_main_window();
                        // Defer window removal to give the main window time to become key
                        window.defer(cx, |window, _cx| {
                            window.remove_window();
                        });
                    }
                }
                Some(ActionsWindowKeyIntent::Close) => {
                    // Notify main app to restore focus before closing
                    let on_close = this.dialog.read(cx).on_close.clone();
                    if let Some(callback) = on_close {
                        callback(cx);
                    }
                    // Activate the main window so it can receive focus
                    platform::activate_main_window();
                    // Defer window removal to give the main window time to become key
                    // and process the pending focus. This matches how close_actions_popup
                    // uses cx.spawn() to close the window asynchronously.
                    window.defer(cx, |window, _cx| {
                        window.remove_window();
                    });
                }
                Some(ActionsWindowKeyIntent::Backspace) => {
                    crate::logging::log("ACTIONS", "ActionsWindow: backspace pressed");
                    this.dialog.update(cx, |d, cx| d.handle_backspace(cx));
                    // Schedule resize after filter changes
                    let dialog = this.dialog.clone();
                    window.defer(cx, move |window, cx| {
                        crate::logging::log("ACTIONS", "ActionsWindow: defer - resizing directly");
                        resize_actions_window_direct(window, cx, &dialog);
                    });
                    cx.notify();
                }
                Some(ActionsWindowKeyIntent::TypeChar(ch)) => {
                    crate::logging::log(
                        "ACTIONS",
                        &format!("ActionsWindow: char '{}' pressed", ch),
                    );
                    this.dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                    // Schedule resize after filter changes
                    let dialog = this.dialog.clone();
                    window.defer(cx, move |window, cx| {
                        crate::logging::log("ACTIONS", "ActionsWindow: defer - resizing directly");
                        resize_actions_window_direct(window, cx, &dialog);
                    });
                    cx.notify();
                }
                None => {}
            }
        });

        // Render the shared dialog entity with key handling
        // Don't use size_full() - the dialog calculates its own dynamic height
        // This prevents unused window space from showing as a dark area
        div()
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(self.dialog.clone())
    }
}

