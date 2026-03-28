// --- merged from part_01.rs ---
// Actions Window - Separate vibrancy window for actions panel
//
// This creates a floating popup window with its own vibrancy blur effect,
// similar to Raycast's actions panel. The window is:
// - Non-draggable (fixed position relative to main window)
// - Positioned below the header, at the right edge of main window
// - Auto-closes when app loses focus
// - Shares the ActionsDialog entity with the main app for keyboard routing

use crate::platform;
use crate::theme::get_cached_theme;
use crate::ui_foundation::{is_key_backspace, is_key_down, is_key_enter, is_key_escape, is_key_up};
use crate::window_resize::layout::FOOTER_HEIGHT;
use gpui::{
    div, prelude::*, px, App, Bounds, Context, DisplayId, Entity, FocusHandle, Focusable, Pixels,
    Point, Render, Size, Subscription, Window, WindowBounds, WindowHandle, WindowKind,
    WindowOptions,
};
// Root intentionally NOT used — its opaque bg blocks NSVisualEffectView vibrancy
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

/// Structured lifecycle events for the actions popup.
///
/// Every significant state transition emits one of these via
/// [`emit_actions_popup_event`] under the `ACTIONS_POPUP` tracing target,
/// giving agentic callers a machine-readable contract for open/route/resize/close.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Variants used from include!()-ed code in app_impl/
pub(crate) enum ActionsPopupEvent {
    /// Toggle or explicit open was requested.
    OpenRequested,
    /// Window was successfully created and stored.
    OpenSucceeded,
    /// Window creation failed (see tracing error for details).
    OpenFailed,
    /// A keyboard event was routed through the popup.
    RoutedKey,
    /// The popup window was resized after filter/content change.
    Resized,
    /// The popup was closed (via Cmd+K, Escape, blur, etc.).
    Closed,
}

/// Emit a structured receipt for an actions popup lifecycle event.
///
/// All fields are optional so callers only supply what is relevant to their
/// transition.  The receipt is emitted at `info` level under
/// `target: "ACTIONS_POPUP"` so log consumers can filter deterministically.
pub(crate) fn emit_actions_popup_event(
    event: ActionsPopupEvent,
    host: Option<&str>,
    position: Option<WindowPosition>,
    num_actions: Option<usize>,
    section_headers: Option<usize>,
    height_px: Option<f32>,
) {
    tracing::info!(
        target: "ACTIONS_POPUP",
        ?event,
        host,
        position = ?position,
        num_actions,
        section_headers,
        height_px,
        "actions popup receipt"
    );
}

/// Global singleton for the actions window handle
static ACTIONS_WINDOW: OnceLock<Mutex<Option<WindowHandle<ActionsWindow>>>> = OnceLock::new();

/// Track the position mode of the current actions window for resize behavior
static ACTIONS_WINDOW_POSITION: OnceLock<Mutex<WindowPosition>> = OnceLock::new();

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

        // Activate the main window BEFORE scheduling focus restoration.
        // macOS window activation is async; starting it early gives the OS
        // more time to make the main window key before the deferred
        // on_close callback runs and sets pending focus.
        if activate_main_window {
            platform::activate_main_window();
        }

        if let Some(on_close) = self.dialog.read(cx).on_close.clone() {
            on_close(cx);
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

// --- merged from part_02.rs ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actions_window_key_intent_supports_aliases_and_jump_keys() {
        let no_mods = gpui::Modifiers::default();

        assert_eq!(
            actions_window_key_intent("return", &no_mods),
            Some(ActionsWindowKeyIntent::ExecuteSelected)
        );
        assert_eq!(
            actions_window_key_intent("esc", &no_mods),
            Some(ActionsWindowKeyIntent::Close)
        );
        assert_eq!(
            actions_window_key_intent("home", &no_mods),
            Some(ActionsWindowKeyIntent::MoveHome)
        );
        assert_eq!(
            actions_window_key_intent("end", &no_mods),
            Some(ActionsWindowKeyIntent::MoveEnd)
        );
        assert_eq!(
            actions_window_key_intent("pageup", &no_mods),
            Some(ActionsWindowKeyIntent::MovePageUp)
        );
        assert_eq!(
            actions_window_key_intent("pagedown", &no_mods),
            Some(ActionsWindowKeyIntent::MovePageDown)
        );
    }

    #[test]
    fn test_actions_window_selectable_index_helpers_skip_section_headers() {
        let rows = vec![
            GroupedActionItem::SectionHeader("One".to_string()),
            GroupedActionItem::Item(0),
            GroupedActionItem::SectionHeader("Two".to_string()),
            GroupedActionItem::Item(1),
        ];

        assert_eq!(first_selectable_index(&rows), Some(1));
        assert_eq!(last_selectable_index(&rows), Some(3));
        assert_eq!(selectable_index_at_or_before(&rows, 2), Some(1));
        assert_eq!(selectable_index_at_or_after(&rows, 2), Some(3));
    }

    #[test]
    fn test_actions_window_dynamic_height_matches_single_row_when_empty() {
        let empty_height = actions_window_dynamic_height(0, 0, false, false);
        let single_row_height = actions_window_dynamic_height(1, 0, false, false);

        assert!(
            (empty_height - single_row_height).abs() < 0.001,
            "empty_height={empty_height}, single_row_height={single_row_height}"
        );
    }
}

#[inline]
fn actions_window_dynamic_height(
    num_actions: usize,
    section_header_count: usize,
    hide_search: bool,
    has_header: bool,
) -> f32 {
    const POPUP_BORDER_HEIGHT: f32 = 2.0;
    let search_box_height = if hide_search {
        0.0
    } else {
        SEARCH_INPUT_HEIGHT
    };
    let header_height = if has_header { HEADER_HEIGHT } else { 0.0 };
    let section_headers_height = section_header_count as f32 * SECTION_HEADER_HEIGHT;
    let min_items_height = if num_actions == 0 {
        ACTION_ITEM_HEIGHT
    } else {
        0.0
    };
    let items_height = (num_actions as f32 * ACTION_ITEM_HEIGHT + section_headers_height)
        .max(min_items_height)
        .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
    let border_height = POPUP_BORDER_HEIGHT;
    items_height + search_box_height + header_height + border_height
}

#[inline]
fn compute_popup_height(dialog: &ActionsDialog) -> f32 {
    let num_actions = dialog.filtered_actions.len();
    let hide_search = dialog.hide_search;
    let has_header = dialog.config.show_context_header && dialog.context_title.is_some();

    let section_header_count = if dialog.config.section_style == SectionStyle::Headers {
        count_section_headers(&dialog.actions, &dialog.filtered_actions)
    } else {
        0
    };

    actions_window_dynamic_height(num_actions, section_header_count, hide_search, has_header)
}

/// Compute the origin point for the actions popup window.
///
/// Pure helper that encapsulates all position-dependent origin math so it can
/// be tested without standing up a real window.
fn actions_popup_origin(
    main_window_bounds: Bounds<Pixels>,
    window_width: Pixels,
    window_height: Pixels,
    position: WindowPosition,
) -> Point<Pixels> {
    let right_aligned_x = main_window_bounds.origin.x + main_window_bounds.size.width
        - window_width
        - px(ACTIONS_MARGIN_X);

    let y = match position {
        WindowPosition::BottomRight => {
            main_window_bounds.origin.y + main_window_bounds.size.height
                - window_height
                - px(FOOTER_HEIGHT)
                - px(ACTIONS_MARGIN_Y)
        }
        WindowPosition::TopRight | WindowPosition::TopCenter => {
            main_window_bounds.origin.y + px(TITLEBAR_HEIGHT) + px(ACTIONS_MARGIN_Y)
        }
    };

    let x = match position {
        WindowPosition::TopCenter => {
            main_window_bounds.origin.x + (main_window_bounds.size.width - window_width) / 2.0
        }
        _ => right_aligned_x,
    };

    Point { x, y }
}

/// Full popup bounds (origin + size) for the actions window.
///
/// Wraps [`actions_popup_origin`] so callers get a single `Bounds` value
/// without reconstructing size separately.
fn actions_popup_bounds(
    main_window_bounds: Bounds<Pixels>,
    window_width: Pixels,
    window_height: Pixels,
    position: WindowPosition,
) -> Bounds<Pixels> {
    Bounds {
        origin: actions_popup_origin(main_window_bounds, window_width, window_height, position),
        size: Size {
            width: window_width,
            height: window_height,
        },
    }
}

/// Structured placement receipt for the actions popup.
///
/// Captures all inputs and computed outputs of a placement decision so that
/// agentic callers can verify geometry deterministically.
#[derive(Debug)]
struct ActionsPopupPlacementReceipt {
    position: WindowPosition,
    display_id: Option<DisplayId>,
    main_window_bounds: Bounds<Pixels>,
    popup_bounds: Bounds<Pixels>,
    anchor_x: Pixels,
    anchor_y: Pixels,
    pinned_edge: &'static str,
}

fn actions_popup_placement_receipt(
    main_window_bounds: Bounds<Pixels>,
    window_width: Pixels,
    window_height: Pixels,
    position: WindowPosition,
    display_id: Option<DisplayId>,
) -> ActionsPopupPlacementReceipt {
    let popup_bounds =
        actions_popup_bounds(main_window_bounds, window_width, window_height, position);

    let (anchor_x, anchor_y, pinned_edge) = match position {
        WindowPosition::BottomRight => (
            main_window_bounds.origin.x + main_window_bounds.size.width - px(ACTIONS_MARGIN_X),
            main_window_bounds.origin.y + main_window_bounds.size.height
                - px(FOOTER_HEIGHT)
                - px(ACTIONS_MARGIN_Y),
            "bottom",
        ),
        WindowPosition::TopRight => (
            main_window_bounds.origin.x + main_window_bounds.size.width - px(ACTIONS_MARGIN_X),
            main_window_bounds.origin.y + px(TITLEBAR_HEIGHT) + px(ACTIONS_MARGIN_Y),
            "top",
        ),
        WindowPosition::TopCenter => (
            main_window_bounds.origin.x + (main_window_bounds.size.width / 2.0),
            main_window_bounds.origin.y + px(TITLEBAR_HEIGHT) + px(ACTIONS_MARGIN_Y),
            "top",
        ),
    };

    ActionsPopupPlacementReceipt {
        position,
        display_id,
        main_window_bounds,
        popup_bounds,
        anchor_x,
        anchor_y,
        pinned_edge,
    }
}

fn log_actions_popup_placement(stage: &'static str, receipt: &ActionsPopupPlacementReceipt) {
    let main_origin_x_px: f32 = receipt.main_window_bounds.origin.x.into();
    let main_origin_y_px: f32 = receipt.main_window_bounds.origin.y.into();
    let main_width_px: f32 = receipt.main_window_bounds.size.width.into();
    let main_height_px: f32 = receipt.main_window_bounds.size.height.into();

    let popup_origin_x_px: f32 = receipt.popup_bounds.origin.x.into();
    let popup_origin_y_px: f32 = receipt.popup_bounds.origin.y.into();
    let popup_width_px: f32 = receipt.popup_bounds.size.width.into();
    let popup_height_px: f32 = receipt.popup_bounds.size.height.into();

    let anchor_x_px: f32 = receipt.anchor_x.into();
    let anchor_y_px: f32 = receipt.anchor_y.into();

    tracing::info!(
        target: "ACTIONS_POPUP",
        stage = stage,
        position = ?receipt.position,
        display_id = ?receipt.display_id,
        pinned_edge = receipt.pinned_edge,
        main_origin_x_px,
        main_origin_y_px,
        main_width_px,
        main_height_px,
        popup_origin_x_px,
        popup_origin_y_px,
        popup_width_px,
        popup_height_px,
        anchor_x_px,
        anchor_y_px,
        "actions popup placement receipt"
    );
}

/// Pure resize origin calculation shared by both resize entry points.
///
/// For bottom-anchored positions the origin stays fixed (bottom edge is pinned).
/// For top-anchored positions the top edge stays fixed, so origin.y shifts.
#[allow(dead_code)]
fn resized_actions_window_origin_y(
    current_origin_y: f64,
    current_height: f64,
    target_height: f64,
    position: WindowPosition,
) -> f64 {
    match position {
        WindowPosition::BottomRight => current_origin_y,
        WindowPosition::TopRight | WindowPosition::TopCenter => {
            let old_top = current_origin_y + current_height;
            old_top - target_height
        }
    }
}

#[cfg(target_os = "macos")]
fn resized_actions_window_frame(
    frame: cocoa::foundation::NSRect,
    new_height_f32: f32,
    position: WindowPosition,
) -> cocoa::foundation::NSRect {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};

    let new_origin_y = resized_actions_window_origin_y(
        frame.origin.y,
        frame.size.height,
        new_height_f32 as f64,
        position,
    );

    NSRect::new(
        NSPoint::new(frame.origin.x, new_origin_y),
        NSSize::new(frame.size.width, new_height_f32 as f64),
    )
}

fn log_actions_popup_resize(
    stage: &'static str,
    position: WindowPosition,
    current_bounds: Bounds<Pixels>,
    target_height_px: f32,
) {
    let current_origin_x_px: f32 = current_bounds.origin.x.into();
    let current_origin_y_px: f32 = current_bounds.origin.y.into();
    let current_width_px: f32 = current_bounds.size.width.into();
    let current_height_px: f32 = current_bounds.size.height.into();

    tracing::info!(
        target: "ACTIONS_POPUP",
        stage = stage,
        position = ?position,
        pinned_edge = match position {
            WindowPosition::BottomRight => "bottom",
            WindowPosition::TopRight | WindowPosition::TopCenter => "top",
        },
        current_origin_x_px,
        current_origin_y_px,
        current_width_px,
        current_height_px,
        target_height_px,
        "actions popup resize receipt"
    );
}

/// Open the actions window as a separate floating window with vibrancy
///
/// The window is positioned at the top-right of the main window, below the header.
/// It does NOT take keyboard focus - the main window keeps focus and routes
/// keyboard events to the shared ActionsDialog entity.
///
/// # Arguments
/// * `cx` - The application context
/// * `main_window_bounds` - The bounds of the main window in SCREEN-RELATIVE coordinates
///   (as returned by GPUI's window.bounds() - top-left origin relative to the window's screen)
/// * `display_id` - The display where the main window is located (actions window will be on same display)
/// * `dialog_entity` - The shared ActionsDialog entity (created by main app)
/// * `position` - Where to position the window relative to the main window
///
/// # Returns
/// The window handle on success
pub fn open_actions_window(
    cx: &mut App,
    main_window_bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
    dialog_entity: Entity<ActionsDialog>,
    position: WindowPosition,
) -> anyhow::Result<WindowHandle<ActionsWindow>> {
    // Close any existing actions window first
    close_actions_window(cx);

    // Load theme for vibrancy settings
    let theme = get_cached_theme();
    let _is_dark_vibrancy = theme.should_use_dark_vibrancy();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    // Calculate dynamic window height based on number of actions.
    // Open and resize paths intentionally share compute_popup_height().
    let dialog = dialog_entity.read(cx);
    let dynamic_height = compute_popup_height(dialog);

    // Calculate window position:
    // - X: Right edge of main window, minus actions width, minus margin
    // - Y: Depends on position parameter:
    //   - BottomRight: Above footer, aligned to bottom
    //   - TopRight: Below titlebar, aligned to top
    //
    // CRITICAL: main_window_bounds must be in SCREEN-RELATIVE coordinates from GPUI's
    // window.bounds(). These are top-left origin, relative to the window's current screen.
    // When we pass display_id to WindowOptions, GPUI will position this window on the
    // same screen as the main window, using these screen-relative coordinates.
    let window_width = px(ACTIONS_WINDOW_WIDTH);
    let window_height = px(dynamic_height);

    let receipt = actions_popup_placement_receipt(
        main_window_bounds,
        window_width,
        window_height,
        position,
        display_id,
    );
    log_actions_popup_placement("open", &receipt);

    let bounds = receipt.popup_bounds;

    crate::logging::log(
        "ACTIONS",
        &format!(
            "Opening actions window at ({:?}, {:?}), size {:?}x{:?}, display_id={:?}, position={:?}",
            bounds.origin.x,
            bounds.origin.y,
            bounds.size.width,
            bounds.size.height,
            display_id,
            position,
        ),
    );

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None, // No titlebar = no drag affordance
        window_background,
        // DON'T take focus - let the parent AI window keep focus and route keys to us
        // macOS popup windows often don't receive keyboard events properly
        focus: false,
        show: true,
        kind: WindowKind::PopUp, // Floating popup window
        display_id,              // CRITICAL: Position on same display as main window
        ..Default::default()
    };

    // Create the window with the shared dialog entity
    // NOTE: We DON'T focus the ActionsWindow's focus_handle here.
    // The parent window (AI window, Notes window, etc.) keeps focus and routes
    // keyboard events to us via its own capture_key_down handler.
    // This avoids focus conflicts where both windows try to handle keys.
    let handle = cx.open_window(window_options, |_window, cx| {
        cx.new(|cx| ActionsWindow::new(dialog_entity.clone(), cx))
    })?;

    // Configure the window as non-movable on macOS
    // Use window.defer() to avoid RefCell borrow conflicts - GPUI may still have
    // internal state borrowed immediately after open_window returns.
    #[cfg(target_os = "macos")]
    {
        let configure_result = handle.update(cx, move |_this, window, cx| {
            window.defer(cx, move |_window, _cx| {
                use cocoa::appkit::NSApp;
                use cocoa::base::nil;
                use objc::{msg_send, sel, sel_impl};

                // Get the NSWindow from the app's windows array
                // The popup window should be the most recently created one
                unsafe {
                    let app: cocoa::base::id = NSApp();
                    let windows: cocoa::base::id = msg_send![app, windows];
                    let count: usize = msg_send![windows, count];
                    if count > 0 {
                        // Get the last window (most recently created)
                        let ns_window: cocoa::base::id = msg_send![windows, lastObject];
                        if ns_window != nil {
                            platform::configure_actions_popup_window(ns_window, is_dark_vibrancy);
                        }
                    }
                }
            });
        });

        if let Err(error) = configure_result {
            crate::logging::log(
                "WARN",
                &format!(
                    "ACTIONS_WINDOW_OP_FAIL configure_popup_window update failed: operation=position_focus error={error:?}"
                ),
            );
            crate::logging::log_debug(
                "ACTIONS",
                &format!(
                    "ACTIONS_WINDOW_OP_FAIL configure_popup_window context: display_id={display_id:?}, position={position:?}"
                ),
            );
        }
    }

    // Store the handle and position globally
    let window_storage = ACTIONS_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = window_storage.lock() {
        *guard = Some(handle);
    }
    let pos_storage = ACTIONS_WINDOW_POSITION.get_or_init(|| Mutex::new(WindowPosition::default()));
    if let Ok(mut guard) = pos_storage.lock() {
        *guard = position;
    }

    crate::logging::log("ACTIONS", "Actions popup window opened with vibrancy");

    // Structured receipt for agentic verification
    let dialog_ref = dialog_entity.read(cx);
    let section_header_count = if dialog_ref.config.section_style == SectionStyle::Headers {
        count_section_headers(&dialog_ref.actions, &dialog_ref.filtered_actions)
    } else {
        0
    };
    emit_actions_popup_event(
        ActionsPopupEvent::OpenSucceeded,
        None,
        Some(position),
        Some(dialog_ref.filtered_actions.len()),
        Some(section_header_count),
        Some(dynamic_height),
    );

    Ok(handle)
}

/// Close the actions window if it's open
pub fn close_actions_window(cx: &mut App) {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(mut guard) = window_storage.lock() {
            if let Some(handle) = guard.take() {
                crate::logging::log("ACTIONS", "Closing actions popup window");
                emit_actions_popup_event(ActionsPopupEvent::Closed, None, None, None, None, None);
                // Close the window
                let close_result = handle.update(cx, |_this, window, _cx| {
                    window.remove_window();
                });
                if let Err(error) = close_result {
                    crate::logging::log(
                        "WARN",
                        &format!(
                            "ACTIONS_WINDOW_OP_FAIL close_actions_window update failed: operation=focus_cleanup error={error:?}"
                        ),
                    );
                    crate::logging::log_debug(
                        "ACTIONS",
                        "ACTIONS_WINDOW_OP_FAIL close_actions_window context: remove_window requested",
                    );
                }
            }
        }
    }
}

/// Check if the given window handle matches the actions window.
///
/// Used by keystroke interceptors to avoid handling keys meant for the
/// actions popup (which manages its own Escape / Enter / arrows).
pub fn is_actions_window(window: &gpui::Window) -> bool {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(guard) = window_storage.lock() {
            if let Some(actions_handle) = guard.as_ref() {
                let actions_any: gpui::AnyWindowHandle = (*actions_handle).into();
                return window.window_handle() == actions_any;
            }
        }
    }
    false
}

/// Check if the actions window is currently open
pub fn is_actions_window_open() -> bool {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(guard) = window_storage.lock() {
            return guard.is_some();
        }
    }
    false
}

/// Get the actions window handle if it exists
pub fn get_actions_window_handle() -> Option<WindowHandle<ActionsWindow>> {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(guard) = window_storage.lock() {
            return *guard;
        }
    }
    None
}

/// Get the current actions window position mode
fn get_actions_window_position() -> WindowPosition {
    if let Some(pos_storage) = ACTIONS_WINDOW_POSITION.get() {
        if let Ok(guard) = pos_storage.lock() {
            return *guard;
        }
    }
    WindowPosition::default()
}

/// Notify the actions window to re-render (call after updating dialog entity)
pub fn notify_actions_window(cx: &mut App) {
    if let Some(handle) = get_actions_window_handle() {
        let notify_result = handle.update(cx, |_this, _window, cx| {
            cx.notify();
        });
        if let Err(error) = notify_result {
            crate::logging::log(
                "WARN",
                &format!(
                    "ACTIONS_WINDOW_OP_FAIL notify_actions_window update failed: operation=focus_refresh error={error:?}"
                ),
            );
            crate::logging::log_debug(
                "ACTIONS",
                "ACTIONS_WINDOW_OP_FAIL notify_actions_window context: cx.notify() skipped",
            );
        }
    }
}

// --- merged from part_03.rs ---

const _ACTIONS_WINDOW_RESIZE_ANIMATE: bool = false;

/// Resize the actions window directly using the window reference
/// Use this from defer callbacks where we already have access to the window
pub fn resize_actions_window_direct(
    window: &mut Window,
    cx: &mut App,
    dialog_entity: &Entity<ActionsDialog>,
) {
    // Read dialog state to calculate new height
    let dialog = dialog_entity.read(cx);
    let num_actions = dialog.filtered_actions.len();
    let hide_search = dialog.hide_search;
    let has_header = dialog.config.show_context_header && dialog.context_title.is_some();

    crate::logging::log(
        "ACTIONS",
        &format!(
            "resize_actions_window_direct: num_actions={}, hide_search={}, has_header={}",
            num_actions, hide_search, has_header
        ),
    );

    let new_height_f32 = compute_popup_height(dialog);

    let current_bounds = window.bounds();
    let current_height_f32: f32 = current_bounds.size.height.into();
    let _current_width_f32: f32 = current_bounds.size.width.into();

    let position = get_actions_window_position();
    log_actions_popup_resize(
        "resize_direct_requested",
        position,
        current_bounds,
        new_height_f32,
    );

    // Skip if height hasn't changed
    if (current_height_f32 - new_height_f32).abs() < 1.0 {
        tracing::debug!(
            target: "ACTIONS_POPUP",
            stage = "resize_direct_skipped",
            position = ?position,
            current_height_px = current_height_f32,
            target_height_px = new_height_f32,
            reason = "height_unchanged",
            "actions popup resize skipped"
        );
        return;
    }

    // Resize via NSWindow using the shared geometry contract
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSScreen;
        use cocoa::base::nil;
        use objc::{msg_send, sel, sel_impl};

        // SAFETY: accessing NSApp and iterating its windows array to find our
        // popup by matching dimensions, then resizing it via setFrame:display:animate:.
        // All ObjC pointers are nil-checked before use.
        unsafe {
            let ns_app: cocoa::base::id = cocoa::appkit::NSApp();
            let windows: cocoa::base::id = msg_send![ns_app, windows];
            let count: usize = msg_send![windows, count];

            let mut found = false;
            for i in 0..count {
                let ns_window: cocoa::base::id = msg_send![windows, objectAtIndex: i];
                if ns_window == nil {
                    continue;
                }

                let frame: cocoa::foundation::NSRect = msg_send![ns_window, frame];

                if (frame.size.width - current_width_f32 as f64).abs() < 2.0
                    && (frame.size.height - current_height_f32 as f64).abs() < 2.0
                {
                    let window_screen: cocoa::base::id = msg_send![ns_window, screen];
                    if window_screen == nil {
                        let screens: cocoa::base::id = NSScreen::screens(nil);
                        let _primary: cocoa::base::id = msg_send![screens, objectAtIndex: 0u64];
                    }

                    let new_frame = resized_actions_window_frame(frame, new_height_f32, position);

                    let _: () = msg_send![
                        ns_window,
                        setFrame:new_frame
                        display:true
                        animate:ACTIONS_WINDOW_RESIZE_ANIMATE
                    ];

                    found = true;
                    break;
                }
            }

            if !found {
                tracing::warn!(
                    target: "ACTIONS_POPUP",
                    stage = "resize_direct_window_match_failed",
                    position = ?position,
                    current_width_px = current_width_f32,
                    current_height_px = current_height_f32,
                    target_height_px = new_height_f32,
                    window_count = count,
                    "actions popup NSWindow lookup failed"
                );
            }
        }
    }

    // Also tell GPUI about the new size
    window.resize(gpui::Size {
        width: current_bounds.size.width,
        height: px(new_height_f32),
    });

    crate::logging::log(
        "ACTIONS",
        &format!(
            "resize_actions_window_direct complete: {} items, height={:.0}",
            num_actions, new_height_f32
        ),
    );

    // Structured receipt for resize
    let dialog_for_receipt = dialog_entity.read(cx);
    let section_header_count = if dialog_for_receipt.config.section_style == SectionStyle::Headers {
        count_section_headers(
            &dialog_for_receipt.actions,
            &dialog_for_receipt.filtered_actions,
        )
    } else {
        0
    };
    emit_actions_popup_event(
        ActionsPopupEvent::Resized,
        None,
        Some(get_actions_window_position()),
        Some(num_actions),
        Some(section_header_count),
        Some(new_height_f32),
    );
}

/// Resize the actions window to fit the current number of filtered actions
/// Call this after filtering changes the action count
///
/// The window is "pinned to bottom" - the search input stays in place and
/// the window shrinks/grows from the top.
pub fn resize_actions_window(cx: &mut App, dialog_entity: &Entity<ActionsDialog>) {
    crate::logging::log("ACTIONS", "resize_actions_window called");
    if let Some(handle) = get_actions_window_handle() {
        // Read dialog state to calculate new height
        let dialog = dialog_entity.read(cx);
        let num_actions = dialog.filtered_actions.len();
        let hide_search = dialog.hide_search;
        let has_header = dialog.config.show_context_header && dialog.context_title.is_some();
        crate::logging::log(
            "ACTIONS",
            &format!(
                "resize_actions_window: num_actions={}, hide_search={}, has_header={}",
                num_actions, hide_search, has_header
            ),
        );

        let new_height_f32 = compute_popup_height(dialog);

        let update_result = handle.update(cx, |_this, window, cx| {
            let current_bounds = window.bounds();
            let current_height_f32: f32 = current_bounds.size.height.into();
            let _current_width_f32: f32 = current_bounds.size.width.into();

            let position = get_actions_window_position();
            log_actions_popup_resize("resize_requested", position, current_bounds, new_height_f32);

            // Skip if height hasn't changed
            if (current_height_f32 - new_height_f32).abs() < 1.0 {
                tracing::debug!(
                    target: "ACTIONS_POPUP",
                    stage = "resize_skipped",
                    position = ?position,
                    current_height_px = current_height_f32,
                    target_height_px = new_height_f32,
                    reason = "height_unchanged",
                    "actions popup resize skipped"
                );
                return;
            }

            #[cfg(target_os = "macos")]
            {
                use cocoa::appkit::NSScreen;
                use cocoa::base::nil;
                use objc::{msg_send, sel, sel_impl};

                // SAFETY: accessing NSApp and iterating its windows array to find our
                // popup by matching dimensions, then resizing via setFrame:display:animate:.
                // All ObjC pointers are nil-checked before use.
                unsafe {
                    let ns_app: cocoa::base::id = cocoa::appkit::NSApp();
                    let windows: cocoa::base::id = msg_send![ns_app, windows];
                    let count: usize = msg_send![windows, count];

                    let mut found = false;
                    for i in 0..count {
                        let ns_window: cocoa::base::id = msg_send![windows, objectAtIndex: i];
                        if ns_window == nil {
                            continue;
                        }

                        let frame: cocoa::foundation::NSRect = msg_send![ns_window, frame];

                        if (frame.size.width - current_width_f32 as f64).abs() < 2.0
                            && (frame.size.height - current_height_f32 as f64).abs() < 2.0
                        {
                            let window_screen: cocoa::base::id = msg_send![ns_window, screen];
                            if window_screen == nil {
                                let screens: cocoa::base::id = NSScreen::screens(nil);
                                let _primary: cocoa::base::id =
                                    msg_send![screens, objectAtIndex: 0u64];
                            }

                            let new_frame =
                                resized_actions_window_frame(frame, new_height_f32, position);

                            let _: () = msg_send![
                                ns_window,
                                setFrame:new_frame
                                display:true
                                animate:ACTIONS_WINDOW_RESIZE_ANIMATE
                            ];

                            found = true;
                            break;
                        }
                    }

                    if !found {
                        tracing::warn!(
                            target: "ACTIONS_POPUP",
                            stage = "resize_window_match_failed",
                            position = ?position,
                            current_width_px = current_width_f32,
                            current_height_px = current_height_f32,
                            target_height_px = new_height_f32,
                            window_count = count,
                            "actions popup NSWindow lookup failed"
                        );
                    }
                }
            }

            // Also tell GPUI about the new size
            window.resize(Size {
                width: current_bounds.size.width,
                height: px(new_height_f32),
            });
            cx.notify();
        });

        if let Err(error) = update_result {
            crate::logging::log(
                "WARN",
                &format!(
                    "ACTIONS_WINDOW_OP_FAIL resize_actions_window update failed: operation=resize error={error:?}"
                ),
            );
            crate::logging::log_debug(
                "ACTIONS",
                &format!(
                    "ACTIONS_WINDOW_OP_FAIL resize_actions_window context: num_actions={}, hide_search={}, has_header={}, target_height={:.0}",
                    num_actions, hide_search, has_header, new_height_f32
                ),
            );
        }

        crate::logging::log(
            "ACTIONS",
            &format!(
                "Resized actions window: {} items, height={:.0}",
                num_actions, new_height_f32
            ),
        );

        let dialog_for_receipt = dialog_entity.read(cx);
        let section_header_count =
            if dialog_for_receipt.config.section_style == SectionStyle::Headers {
                count_section_headers(
                    &dialog_for_receipt.actions,
                    &dialog_for_receipt.filtered_actions,
                )
            } else {
                0
            };
        emit_actions_popup_event(
            ActionsPopupEvent::Resized,
            None,
            Some(get_actions_window_position()),
            Some(num_actions),
            Some(section_header_count),
            Some(new_height_f32),
        );
    }
}

#[cfg(test)]
mod resize_instant_tests {
    use super::ACTIONS_WINDOW_RESIZE_ANIMATE;

    #[test]
    fn test_actions_window_resize_animation_flag_is_disabled() {
        let flag = ACTIONS_WINDOW_RESIZE_ANIMATE;
        assert!(
            !flag,
            "Actions window resize must stay instant with animation disabled"
        );
    }
}

#[cfg(test)]
mod request_close_ordering_tests {
    use std::fs;

    #[test]
    fn test_request_close_activates_main_window_before_on_close_callback() {
        let source = fs::read_to_string("src/actions/window.rs")
            .expect("Failed to read src/actions/window.rs");

        let start = source
            .find("fn request_close")
            .expect("Expected request_close function in src/actions/window.rs");
        let end = source[start..]
            .find("Self::defer_close")
            .map(|idx| start + idx)
            .expect("Expected defer_close call in request_close");
        let body = &source[start..end];

        let activate_idx = body
            .find("platform::activate_main_window")
            .expect("Expected activate_main_window call in request_close");
        let on_close_idx = body
            .find("on_close(cx)")
            .expect("Expected on_close(cx) invocation in request_close");

        assert!(
            activate_idx < on_close_idx,
            "request_close must activate the main window BEFORE scheduling focus restoration \
             via on_close callback. macOS window activation is async — starting it earlier \
             gives the OS time to make the main window key before the deferred callback runs."
        );
    }

    #[test]
    fn test_is_actions_window_function_exists() {
        let source = fs::read_to_string("src/actions/window.rs")
            .expect("Failed to read src/actions/window.rs");

        assert!(
            source.contains("pub fn is_actions_window(window: &gpui::Window) -> bool"),
            "window.rs must export is_actions_window(window) for keystroke interceptor guards"
        );
    }
}

#[cfg(test)]
mod actions_popup_origin_tests {
    use super::*;
    use gpui::{px, Bounds, Point, Size};

    #[test]
    fn top_center_centers_inside_mini_main_window() {
        let origin = actions_popup_origin(
            Bounds {
                origin: Point {
                    x: px(100.0),
                    y: px(50.0),
                },
                size: Size {
                    width: px(480.0),
                    height: px(300.0),
                },
            },
            px(ACTIONS_WINDOW_WIDTH),
            px(220.0),
            WindowPosition::TopCenter,
        );

        assert_eq!(
            f32::from(origin.x),
            100.0 + ((480.0 - ACTIONS_WINDOW_WIDTH) / 2.0),
            "TopCenter must center horizontally within the parent window"
        );
        assert_eq!(
            f32::from(origin.y),
            50.0 + TITLEBAR_HEIGHT + ACTIONS_MARGIN_Y,
            "TopCenter must anchor below the titlebar"
        );
    }

    #[test]
    fn bottom_right_stays_above_footer() {
        let origin = actions_popup_origin(
            Bounds {
                origin: Point {
                    x: px(20.0),
                    y: px(40.0),
                },
                size: Size {
                    width: px(750.0),
                    height: px(500.0),
                },
            },
            px(ACTIONS_WINDOW_WIDTH),
            px(180.0),
            WindowPosition::BottomRight,
        );

        assert_eq!(
            f32::from(origin.x),
            20.0 + 750.0 - ACTIONS_WINDOW_WIDTH - ACTIONS_MARGIN_X,
            "BottomRight must right-align with margin"
        );
        assert_eq!(
            f32::from(origin.y),
            40.0 + 500.0 - 180.0 - FOOTER_HEIGHT - ACTIONS_MARGIN_Y,
            "BottomRight must sit above the footer"
        );
    }

    #[test]
    fn top_right_right_aligns_below_titlebar() {
        let origin = actions_popup_origin(
            Bounds {
                origin: Point {
                    x: px(0.0),
                    y: px(0.0),
                },
                size: Size {
                    width: px(600.0),
                    height: px(400.0),
                },
            },
            px(ACTIONS_WINDOW_WIDTH),
            px(200.0),
            WindowPosition::TopRight,
        );

        assert_eq!(
            f32::from(origin.x),
            600.0 - ACTIONS_WINDOW_WIDTH - ACTIONS_MARGIN_X,
            "TopRight must right-align with margin"
        );
        assert_eq!(
            f32::from(origin.y),
            TITLEBAR_HEIGHT + ACTIONS_MARGIN_Y,
            "TopRight must anchor below the titlebar"
        );
    }

    #[test]
    fn open_actions_window_uses_placement_receipt_helper() {
        let source = std::fs::read_to_string("src/actions/window.rs")
            .expect("Failed to read src/actions/window.rs");

        let fn_start = source
            .find("pub fn open_actions_window(")
            .expect("open_actions_window not found");
        let fn_body = &source[fn_start..];

        assert!(
            fn_body.contains("actions_popup_placement_receipt("),
            "open_actions_window must delegate to actions_popup_placement_receipt helper"
        );
    }
}

#[cfg(test)]
mod actions_popup_geometry_tests {
    use super::*;
    use gpui::{px, Bounds, Point, Size};

    fn test_main_window_bounds() -> Bounds<Pixels> {
        Bounds {
            origin: Point {
                x: px(100.0),
                y: px(50.0),
            },
            size: Size {
                width: px(480.0),
                height: px(220.0),
            },
        }
    }

    #[test]
    fn top_center_bounds_are_centered_below_titlebar() {
        let bounds = actions_popup_bounds(
            test_main_window_bounds(),
            px(320.0),
            px(180.0),
            WindowPosition::TopCenter,
        );

        let x: f32 = bounds.origin.x.into();
        let y: f32 = bounds.origin.y.into();

        assert_eq!(x, 180.0);
        assert_eq!(y, 94.0);
    }

    #[test]
    fn bottom_right_bounds_are_right_aligned_above_footer() {
        let bounds = actions_popup_bounds(
            test_main_window_bounds(),
            px(320.0),
            px(180.0),
            WindowPosition::BottomRight,
        );

        let x: f32 = bounds.origin.x.into();
        let y: f32 = bounds.origin.y.into();

        // x = 100 + 480 - 320 - 8 = 252
        assert_eq!(x, 252.0);
        // y = 50 + 220 - 180 - FOOTER_HEIGHT(30) - ACTIONS_MARGIN_Y(8) = 52
        assert_eq!(y, 52.0);
    }

    #[test]
    fn top_anchored_resize_keeps_top_edge_fixed() {
        assert_eq!(
            resized_actions_window_origin_y(94.0, 180.0, 216.0, WindowPosition::TopCenter),
            58.0
        );
        assert_eq!(
            resized_actions_window_origin_y(94.0, 180.0, 216.0, WindowPosition::TopRight),
            58.0
        );
    }

    #[test]
    fn bottom_anchored_resize_keeps_bottom_edge_fixed() {
        assert_eq!(
            resized_actions_window_origin_y(42.0, 180.0, 216.0, WindowPosition::BottomRight),
            42.0
        );
    }

    #[test]
    fn placement_receipt_captures_correct_pinned_edge() {
        let receipt = actions_popup_placement_receipt(
            test_main_window_bounds(),
            px(320.0),
            px(180.0),
            WindowPosition::TopCenter,
            None,
        );
        assert_eq!(receipt.pinned_edge, "top");

        let receipt = actions_popup_placement_receipt(
            test_main_window_bounds(),
            px(320.0),
            px(180.0),
            WindowPosition::BottomRight,
            None,
        );
        assert_eq!(receipt.pinned_edge, "bottom");
    }

    #[test]
    fn actions_popup_bounds_size_matches_inputs() {
        let bounds = actions_popup_bounds(
            test_main_window_bounds(),
            px(320.0),
            px(180.0),
            WindowPosition::TopCenter,
        );

        let w: f32 = bounds.size.width.into();
        let h: f32 = bounds.size.height.into();
        assert_eq!(w, 320.0);
        assert_eq!(h, 180.0);
    }

    #[test]
    fn resize_paths_use_shared_geometry_helpers() {
        let source = std::fs::read_to_string("src/actions/window.rs")
            .expect("Failed to read src/actions/window.rs");

        let direct_start = source
            .find("pub fn resize_actions_window_direct(")
            .expect("resize_actions_window_direct not found");
        let direct_body = &source[direct_start..];
        assert!(
            direct_body.contains("resized_actions_window_frame("),
            "resize_actions_window_direct must use resized_actions_window_frame helper"
        );

        let indirect_start = source
            .find("pub fn resize_actions_window(")
            .expect("resize_actions_window not found");
        let indirect_body = &source[indirect_start..];
        assert!(
            indirect_body.contains("resized_actions_window_frame("),
            "resize_actions_window must use resized_actions_window_frame helper"
        );
    }
}
