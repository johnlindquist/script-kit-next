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
    Point, Render, Size, Subscription, Window, WindowBounds, WindowHandle, WindowKind,
    WindowOptions,
};
use gpui_component::Root;
use std::sync::{Mutex, OnceLock};

use super::constants::{
    ACTION_ITEM_HEIGHT, HEADER_HEIGHT, POPUP_MAX_HEIGHT, POPUP_WIDTH, SEARCH_INPUT_HEIGHT,
    SECTION_HEADER_HEIGHT,
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
    let mut prev_section: Option<&str> = None;

    for &idx in filtered_indices {
        if let Some(action) = actions.get(idx) {
            // Match header insertion behavior from grouped list rendering:
            // only track non-empty sections so unsectioned rows do not break a section run.
            if let Some(current_section) = action.section.as_deref() {
                if prev_section != Some(current_section) {
                    count += 1;
                    prev_section = Some(current_section);
                }
            }
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

#[inline]
fn should_auto_close_actions_window(
    main_window_focused: bool,
    actions_window_active: bool,
) -> bool {
    !main_window_focused && !actions_window_active
}

#[inline]
fn clear_window_slot<T>(slot: &mut Option<T>) -> bool {
    let had_value = slot.is_some();
    *slot = None;
    had_value
}

fn clear_actions_window_handle(reason: &str) {
    let Some(window_storage) = ACTIONS_WINDOW.get() else {
        crate::logging::log(
            "ACTIONS",
            &format!(
                "ACTIONS_WINDOW_LIFECYCLE clear_actions_window_handle skipped: reason={}, state=uninitialized",
                reason
            ),
        );
        return;
    };

    match window_storage.lock() {
        Ok(mut guard) => {
            let had_handle = clear_window_slot(&mut guard);
            crate::logging::log(
                "ACTIONS",
                &format!(
                    "ACTIONS_WINDOW_LIFECYCLE clear_actions_window_handle: reason={}, had_handle={}",
                    reason, had_handle
                ),
            );
        }
        Err(error) => {
            crate::logging::log(
                "ACTIONS",
                &format!(
                    "ACTIONS_WINDOW_LIFECYCLE clear_actions_window_handle failed: reason={}, error={}",
                    reason, error
                ),
            );
        }
    }
}

/// Actions window width (height is calculated dynamically based on content)
const ACTIONS_WINDOW_WIDTH: f32 = POPUP_WIDTH;
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
    /// Keep activation observer alive so blur-driven auto-close is reliable.
    activation_subscription: Option<Subscription>,
}

impl ActionsWindow {
    pub fn new(dialog: Entity<ActionsDialog>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            dialog,
            focus_handle,
            activation_subscription: None,
        }
    }

    fn defer_close(window: &mut Window, cx: &mut Context<Self>, reason: &'static str) {
        crate::logging::log(
            "ACTIONS",
            &format!("ACTIONS_WINDOW_LIFECYCLE defer_close_scheduled: reason={reason}"),
        );
        window.defer(cx, move |window, _cx| {
            crate::logging::log(
                "ACTIONS",
                &format!("ACTIONS_WINDOW_LIFECYCLE defer_close_executing: reason={reason}"),
            );
            clear_actions_window_handle(reason);
            window.remove_window();
        });
    }

    fn request_close(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
        reason: &'static str,
        activate_main_window: bool,
    ) {
        crate::logging::log(
            "ACTIONS",
            &format!(
                "ACTIONS_WINDOW_LIFECYCLE request_close: reason={reason}, activate_main_window={activate_main_window}"
            ),
        );

        if let Some(on_close) = self.dialog.read(cx).on_close.clone() {
            on_close(cx);
        }

        if activate_main_window {
            platform::activate_main_window();
        }

        Self::defer_close(window, cx, reason);
    }

    fn ensure_activation_subscription(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.activation_subscription.is_some() {
            return;
        }

        crate::logging::log(
            "ACTIONS",
            "ACTIONS_WINDOW_LIFECYCLE activation_subscription_initialized",
        );

        self.activation_subscription = Some(cx.observe_window_activation(window, |this, window, cx| {
            let main_window_focused = platform::is_main_window_focused();
            let actions_window_active = window.is_window_active();
            let should_close =
                should_auto_close_actions_window(main_window_focused, actions_window_active);

            crate::logging::log(
                "ACTIONS",
                &format!(
                    "ACTIONS_WINDOW_LIFECYCLE activation_changed: main_window_focused={}, actions_window_active={}, should_close={}",
                    main_window_focused, actions_window_active, should_close
                ),
            );

            if !should_close {
                return;
            }

            this.request_close(window, cx, "focus_lost", false);
        }));
    }
}

impl Focusable for ActionsWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ActionsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_activation_subscription(window, cx);

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
                        this.request_close(window, cx, "execute_selected", true);
                    }
                }
                Some(ActionsWindowKeyIntent::Close) => {
                    this.request_close(window, cx, "escape", true);
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

#[cfg(test)]
mod window_lifecycle_tests {
    use super::*;

    #[test]
    fn test_should_auto_close_actions_window_returns_true_when_neither_window_is_focused() {
        assert!(should_auto_close_actions_window(false, false));
    }

    #[test]
    fn test_should_auto_close_actions_window_returns_false_when_main_window_is_focused() {
        assert!(!should_auto_close_actions_window(true, false));
    }

    #[test]
    fn test_should_auto_close_actions_window_returns_false_when_actions_window_is_active() {
        assert!(!should_auto_close_actions_window(false, true));
    }

    #[test]
    fn test_clear_window_slot_does_clear_when_value_is_present() {
        let mut slot = Some(42usize);
        let had_value = clear_window_slot(&mut slot);
        assert!(had_value);
        assert_eq!(slot, None);
    }

    #[test]
    fn test_clear_window_slot_is_idempotent_when_called_multiple_times() {
        let mut slot = Some(42usize);
        assert!(clear_window_slot(&mut slot));
        assert!(!clear_window_slot(&mut slot));
        assert_eq!(slot, None);
    }

    fn make_action_for_header_count(id: &str, section: Option<&str>) -> Action {
        let mut action = Action::new(
            id,
            id,
            None,
            crate::actions::types::ActionCategory::ScriptContext,
        );
        if let Some(section) = section {
            action = action.with_section(section);
        }
        action
    }

    #[test]
    fn test_count_section_headers_does_not_reset_on_unsectioned_rows() {
        let actions = vec![
            make_action_for_header_count("a", Some("S1")),
            make_action_for_header_count("b", None),
            make_action_for_header_count("c", Some("S1")),
        ];

        assert_eq!(count_section_headers(&actions, &[0, 1, 2]), 1);
    }

    #[test]
    fn test_count_section_headers_counts_new_section_after_unsectioned_row() {
        let actions = vec![
            make_action_for_header_count("a", Some("S1")),
            make_action_for_header_count("b", None),
            make_action_for_header_count("c", Some("S2")),
        ];

        assert_eq!(count_section_headers(&actions, &[0, 1, 2]), 2);
    }

    #[test]
    fn test_actions_window_key_intent_maps_required_navigation_key_variants() {
        let no_mods = gpui::Modifiers::default();

        assert_eq!(
            actions_window_key_intent("up", &no_mods),
            Some(ActionsWindowKeyIntent::MoveUp)
        );
        assert_eq!(
            actions_window_key_intent("arrowup", &no_mods),
            Some(ActionsWindowKeyIntent::MoveUp)
        );

        assert_eq!(
            actions_window_key_intent("down", &no_mods),
            Some(ActionsWindowKeyIntent::MoveDown)
        );
        assert_eq!(
            actions_window_key_intent("arrowdown", &no_mods),
            Some(ActionsWindowKeyIntent::MoveDown)
        );
    }

    #[test]
    fn test_actions_window_key_intent_maps_required_confirm_and_cancel_key_variants() {
        let no_mods = gpui::Modifiers::default();

        assert_eq!(
            actions_window_key_intent("enter", &no_mods),
            Some(ActionsWindowKeyIntent::ExecuteSelected)
        );
        assert_eq!(
            actions_window_key_intent("Enter", &no_mods),
            Some(ActionsWindowKeyIntent::ExecuteSelected)
        );

        assert_eq!(
            actions_window_key_intent("escape", &no_mods),
            Some(ActionsWindowKeyIntent::Close)
        );
        assert_eq!(
            actions_window_key_intent("Escape", &no_mods),
            Some(ActionsWindowKeyIntent::Close)
        );
    }
}
