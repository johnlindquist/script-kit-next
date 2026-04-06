// Confirm popup window — a native GPUI WindowKind::PopUp window with macOS
// vibrancy blur. Replaces the old in-window overlay dialog approach so the
// confirmation surface gets real NSPanel blur instead of plain transparency.

use std::{
    rc::Rc,
    sync::{Mutex, OnceLock},
    time::Duration,
};

use gpui::{
    div, prelude::*, px, App, Bounds, Context, DisplayId, FocusHandle, Focusable, MouseButton,
    Pixels, Point, Render, SharedString, Size, Task, Window, WindowBackgroundAppearance,
    WindowBounds, WindowHandle, WindowKind, WindowOptions,
};
use gpui_component::button::ButtonVariant;

use crate::{
    list_item::FONT_MONO,
    platform,
    theme::get_cached_theme,
    ui_foundation::{is_key_enter, is_key_escape, is_key_left, is_key_tab, HexColorExt},
};

const CONFIRM_PADDING_X: f32 = 10.0;
const CONFIRM_PADDING_Y: f32 = 14.0;
const CONFIRM_SECTION_GAP: f32 = 4.0;
const CONFIRM_BUTTON_GAP: f32 = 12.0;
const CONFIRM_BUTTON_HEIGHT: f32 = 20.0;
const CONFIRM_TITLE_LINE_HEIGHT: f32 = 16.0;
const CONFIRM_BODY_LINE_HEIGHT: f32 = 16.0;
const CONFIRM_MIN_HEIGHT: f32 = 76.0;
const CONFIRM_MAX_HEIGHT: f32 = 128.0;
const CONFIRM_BODY_MAX_LINES: usize = 3;
const CONFIRM_LIFECYCLE_POLL_MS: u64 = 120;
/// NSWindowOrderingMode::NSWindowAbove — place child above parent.
const NS_WINDOW_ABOVE: i64 = 1;

static CONFIRM_WINDOW: OnceLock<Mutex<Option<WindowHandle<ConfirmPopupWindow>>>> = OnceLock::new();
static CONFIRM_RESULT_TX: OnceLock<Mutex<Option<async_channel::Sender<bool>>>> = OnceLock::new();
static CONFIRM_FOCUSED_BUTTON: OnceLock<Mutex<FocusedButton>> = OnceLock::new();

#[derive(Clone)]
pub(crate) struct ConfirmWindowOptions {
    pub title: SharedString,
    pub body: SharedString,
    pub confirm_text: SharedString,
    pub cancel_text: SharedString,
    pub confirm_variant: ButtonVariant,
    pub width: Pixels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedButton {
    Cancel,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfirmWindowKeyIntent {
    FocusNext,
    FocusPrev,
    ActivateFocused,
    Cancel,
}

#[inline]
fn is_key_right(key: &str) -> bool {
    key.eq_ignore_ascii_case("right") || key.eq_ignore_ascii_case("arrowright")
}

#[inline]
fn confirm_window_key_intent(
    key: &str,
    modifiers: &gpui::Modifiers,
) -> Option<ConfirmWindowKeyIntent> {
    if is_key_escape(key) {
        return Some(ConfirmWindowKeyIntent::Cancel);
    }
    if is_key_enter(key) {
        return Some(ConfirmWindowKeyIntent::ActivateFocused);
    }
    if is_key_tab(key) {
        return Some(if modifiers.shift {
            ConfirmWindowKeyIntent::FocusPrev
        } else {
            ConfirmWindowKeyIntent::FocusNext
        });
    }
    if is_key_left(key) {
        return Some(ConfirmWindowKeyIntent::FocusPrev);
    }
    if is_key_right(key) {
        return Some(ConfirmWindowKeyIntent::FocusNext);
    }
    None
}

fn estimate_wrapped_lines(text: &str, approx_chars_per_line: usize) -> usize {
    let approx_chars_per_line = approx_chars_per_line.max(1);
    text.lines()
        .map(|line| {
            let line_len = line.chars().count().max(1);
            line_len.div_ceil(approx_chars_per_line)
        })
        .sum::<usize>()
        .max(1)
}

fn confirm_window_dynamic_height(width: Pixels, title: &str, body: &str) -> f32 {
    let width_px: f32 = width.into();
    let content_width = (width_px - (CONFIRM_PADDING_X * 2.0)).max(160.0);
    let approx_chars_per_line = ((content_width / 7.4).floor() as usize).max(12);

    let title_lines = estimate_wrapped_lines(title, approx_chars_per_line).min(2);
    let title_height = title_lines as f32 * CONFIRM_TITLE_LINE_HEIGHT;

    let has_body = !body.trim().is_empty();
    let body_lines = if has_body {
        estimate_wrapped_lines(body, approx_chars_per_line).min(CONFIRM_BODY_MAX_LINES)
    } else {
        0
    };
    let body_height = body_lines as f32 * CONFIRM_BODY_LINE_HEIGHT;
    let body_gap = if has_body { CONFIRM_SECTION_GAP } else { 0.0 };

    (CONFIRM_PADDING_Y * 2.0
        + title_height
        + body_gap
        + body_height
        + CONFIRM_SECTION_GAP
        + CONFIRM_BUTTON_HEIGHT)
        .clamp(CONFIRM_MIN_HEIGHT, CONFIRM_MAX_HEIGHT)
}

fn confirm_window_bounds(
    parent_bounds: Bounds<Pixels>,
    width: Pixels,
    title: &str,
    body: &str,
) -> Bounds<Pixels> {
    let height = px(confirm_window_dynamic_height(width, title, body));

    // Match parent width and bottom-align flush with parent bottom edge
    let actual_width = parent_bounds.size.width;
    let x = parent_bounds.origin.x;
    let y = parent_bounds.origin.y + parent_bounds.size.height - height;

    Bounds {
        origin: Point { x, y },
        size: Size {
            width: actual_width,
            height,
        },
    }
}

fn clear_confirm_window_handle() {
    if let Some(storage) = CONFIRM_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            *guard = None;
        }
    }
    // Also clear shared state
    if let Some(storage) = CONFIRM_RESULT_TX.get() {
        if let Ok(mut guard) = storage.lock() {
            *guard = None;
        }
    }
}

/// Route a key event to the confirm popup window if it's open.
/// Returns true if the key was handled (confirm popup consumed it).
/// Called from the main window's key handler chain.
#[allow(dead_code)]
pub(crate) fn consume_main_window_key_while_confirm_open(
    key: &str,
    modifiers: &gpui::Modifiers,
    cx: &mut App,
) -> bool {
    if !is_confirm_window_open() {
        return false;
    }

    let intent = confirm_window_key_intent(key, modifiers);

    tracing::info!(
        target: "script_kit::confirm",
        event = "route_key_to_confirm_popup",
        key,
        shift = modifiers.shift,
        platform = modifiers.platform,
        alt = modifiers.alt,
        control = modifiers.control,
        intent = ?intent,
        "Main window routing key to confirm popup"
    );

    match intent {
        Some(ConfirmWindowKeyIntent::Cancel) => {
            tracing::info!(
                target: "script_kit::confirm",
                event = "route_key_confirm_cancel",
                "Routing Escape to confirm popup → cancel"
            );
            send_confirm_result(false);
            // Defer window close so is_confirm_window_open() remains true
            // for the rest of this event processing cycle, blocking
            // PressEnter and other handlers from also firing.
            defer_close_confirm_window(cx);
            true
        }
        Some(ConfirmWindowKeyIntent::ActivateFocused) => {
            let focused = get_confirm_focused_button();
            let confirmed = matches!(focused, FocusedButton::Confirm);
            tracing::info!(
                target: "script_kit::confirm",
                event = "route_key_confirm_enter",
                confirmed,
                focused_button = ?focused,
                "Routing Enter to confirm popup → activate focused"
            );
            send_confirm_result(confirmed);
            defer_close_confirm_window(cx);
            true
        }
        Some(ConfirmWindowKeyIntent::FocusNext) => {
            toggle_confirm_focused_button();
            // Notify the confirm window to re-render with updated focus
            notify_confirm_window(cx);
            true
        }
        Some(ConfirmWindowKeyIntent::FocusPrev) => {
            toggle_confirm_focused_button();
            notify_confirm_window(cx);
            true
        }
        None => {
            tracing::debug!(
                target: "script_kit::confirm",
                event = "route_key_confirm_consume_unhandled",
                key,
                "Confirm popup is open — consuming unhandled key"
            );
            true
        }
    }
}

#[allow(dead_code)]
pub(crate) fn route_key_to_confirm_popup(key: &str, cx: &mut App) -> bool {
    consume_main_window_key_while_confirm_open(key, &gpui::Modifiers::default(), cx)
}

pub(crate) fn send_confirm_result(confirmed: bool) {
    if let Some(storage) = CONFIRM_RESULT_TX.get() {
        if let Ok(mut guard) = storage.lock() {
            if let Some(tx) = guard.take() {
                let _ = tx.try_send(confirmed);
            }
        }
    }
}

/// Select and activate a confirm dialog button by value for batch automation.
///
/// Accepts `"confirm"` or `"cancel"` as the value. Sends the result and closes
/// the dialog. Returns `Some(value)` on success, `None` if the value is invalid
/// or no confirm dialog is open.
#[allow(dead_code)]
pub(crate) fn batch_select_confirm_button_by_value(value: &str) -> Option<String> {
    let confirmed = match value {
        "confirm" => true,
        "cancel" => false,
        _ => return None,
    };
    // Verify a confirm window is actually open
    if !is_confirm_window_open() {
        return None;
    }
    send_confirm_result(confirmed);
    Some(value.to_string())
}

/// Select and activate a confirm dialog button by semantic ID.
///
/// Accepts `"button:0:confirm"` or `"button:1:cancel"`. Returns the semantic ID
/// on success.
#[allow(dead_code)]
pub(crate) fn batch_select_confirm_button_by_semantic_id(semantic_id: &str) -> Option<String> {
    let value = match semantic_id {
        "button:0:confirm" => "confirm",
        "button:1:cancel" => "cancel",
        _ => return None,
    };
    batch_select_confirm_button_by_value(value)?;
    Some(semantic_id.to_string())
}

fn get_confirm_focused_button() -> FocusedButton {
    CONFIRM_FOCUSED_BUTTON
        .get()
        .and_then(|s| s.lock().ok())
        .map_or(FocusedButton::Confirm, |g| *g)
}

fn toggle_confirm_focused_button() {
    if let Some(storage) = CONFIRM_FOCUSED_BUTTON.get() {
        if let Ok(mut guard) = storage.lock() {
            *guard = match *guard {
                FocusedButton::Cancel => FocusedButton::Confirm,
                FocusedButton::Confirm => FocusedButton::Cancel,
            };
        }
    }
}

/// Defer closing the confirm window to the next frame so that
/// `is_confirm_window_open()` remains true for the rest of the current
/// event processing cycle. This prevents PressEnter and other handlers
/// from also processing the same Enter keystroke.
fn defer_close_confirm_window(cx: &mut App) {
    cx.defer(|cx| {
        close_confirm_window(cx);
    });
}

fn notify_confirm_window(cx: &mut App) {
    if let Some(storage) = CONFIRM_WINDOW.get() {
        if let Ok(guard) = storage.lock() {
            if let Some(handle) = guard.as_ref() {
                let _ = handle.update(cx, |_root, _window, cx| {
                    cx.notify();
                });
            }
        }
    }
}

#[allow(dead_code)]
/// Snapshot of the confirm popup's semantic state for automation.
#[derive(Debug, Clone)]
pub(crate) struct ConfirmPopupSnapshot {
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) confirm_text: String,
    pub(crate) cancel_text: String,
    pub(crate) focused_button: &'static str,
}

/// Read the confirm popup snapshot if the popup window is open.
///
/// Used by the automation surface collector to extract semantic elements
/// from the live popup state without needing `&mut App`.
pub(crate) fn get_confirm_popup_snapshot(cx: &gpui::App) -> Option<ConfirmPopupSnapshot> {
    let storage = CONFIRM_WINDOW.get()?;
    let guard = storage.lock().ok()?;
    let handle = (*guard)?;
    handle
        .read_with(cx, |popup, _cx| {
            let focused_button = match popup.focused_button {
                FocusedButton::Confirm => "confirm",
                FocusedButton::Cancel => "cancel",
            };
            ConfirmPopupSnapshot {
                title: popup.title.to_string(),
                body: popup.body.to_string(),
                confirm_text: popup.confirm_text.to_string(),
                cancel_text: popup.cancel_text.to_string(),
                focused_button,
            }
        })
        .ok()
}

pub(crate) fn is_confirm_window_open() -> bool {
    CONFIRM_WINDOW
        .get()
        .and_then(|storage| storage.lock().ok())
        .is_some_and(|guard| guard.is_some())
}

pub(crate) fn close_confirm_window(cx: &mut App) {
    tracing::info!(
        target: "script_kit::confirm",
        event = "close_confirm_window_called",
        "close_confirm_window called"
    );
    if let Some(storage) = CONFIRM_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            if let Some(handle) = guard.take() {
                tracing::info!(
                    target: "script_kit::confirm",
                    event = "close_confirm_window_removing",
                    "close_confirm_window: removing window"
                );
                // remove_window() destroys the NSWindow, which causes AppKit
                // to automatically detach it from its parent (addChildWindow
                // relationship). No manual removeChildWindow: needed.
                let _ = handle.update(cx, |_root, window, _cx| {
                    window.remove_window();
                });
            } else {
                tracing::debug!(
                    target: "script_kit::confirm",
                    event = "close_confirm_window_no_handle",
                    "close_confirm_window: no handle stored"
                );
            }
        }
    }
}

pub(crate) fn open_confirm_popup_window(
    cx: &mut App,
    parent_bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
    options: ConfirmWindowOptions,
    keep_open_while: Rc<dyn Fn() -> bool>,
    result_tx: async_channel::Sender<bool>,
) -> anyhow::Result<WindowHandle<ConfirmPopupWindow>> {
    tracing::info!(
        target: "script_kit::confirm",
        event = "open_confirm_popup_window",
        title = %options.title,
        parent_x = ?parent_bounds.origin.x,
        parent_y = ?parent_bounds.origin.y,
        parent_w = ?parent_bounds.size.width,
        parent_h = ?parent_bounds.size.height,
        display_id = ?display_id,
        "open_confirm_popup_window: opening native confirm popup"
    );
    close_confirm_window(cx);

    let theme = get_cached_theme();
    let is_dark_vibrancy = theme.should_use_dark_vibrancy();
    let vibrancy_enabled = theme.is_vibrancy_enabled();
    let window_background = if vibrancy_enabled {
        WindowBackgroundAppearance::Blurred
    } else {
        WindowBackgroundAppearance::Opaque
    };

    let bounds = confirm_window_bounds(
        parent_bounds,
        options.width,
        options.title.as_ref(),
        options.body.as_ref(),
    );

    tracing::info!(
        target: "script_kit::confirm",
        event = "open_confirm_popup_window_bounds",
        x = ?bounds.origin.x,
        y = ?bounds.origin.y,
        width = ?bounds.size.width,
        height = ?bounds.size.height,
        vibrancy_enabled,
        is_dark_vibrancy,
        "open_confirm_popup_window: calculated bounds"
    );

    let request = options.clone();
    let lifecycle = keep_open_while.clone();
    let sender = result_tx.clone();

    let handle = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            window_background,
            focus: true,
            show: true,
            kind: WindowKind::PopUp,
            display_id,
            ..Default::default()
        },
        move |_window, cx| cx.new(|cx| ConfirmPopupWindow::new(request, lifecycle, sender, cx)),
    )?;

    tracing::info!(
        target: "script_kit::confirm",
        event = "open_confirm_popup_window_created",
        "open_confirm_popup_window: window created successfully"
    );

    // Capture expected frame for NSWindow matching in the deferred callback
    let expected_w: f32 = bounds.size.width.into();
    let expected_h: f32 = bounds.size.height.into();
    let expected_x: f32 = bounds.origin.x.into();
    let expected_y: f32 = bounds.origin.y.into();

    #[cfg(target_os = "macos")]
    {
        let _ = handle.update(cx, move |_root, window, cx| {
            window.defer(cx, move |_window, _cx| {
                use cocoa::appkit::NSApp;
                use cocoa::base::nil;
                use objc::{msg_send, sel, sel_impl};

                // SAFETY: On the AppKit main thread inside GPUI's deferred
                // window callback. We enumerate all NSWindows to find the
                // confirm popup by matching the expected frame bounds, rather
                // than relying on lastObject (which may return the wrong
                // window when other popups coexist).
                unsafe {
                    let app: cocoa::base::id = NSApp();
                    let windows: cocoa::base::id = msg_send![app, windows];
                    let count: usize = msg_send![windows, count];

                    tracing::info!(
                        target: "script_kit::confirm",
                        event = "configure_confirm_nswindow_search",
                        window_count = count,
                        target_x = expected_x,
                        target_y = expected_y,
                        target_w = expected_w,
                        target_h = expected_h,
                        "Searching {} NSWindows for confirm popup by frame bounds",
                        count
                    );

                    if count == 0 {
                        tracing::warn!(
                            target: "script_kit::confirm",
                            event = "configure_confirm_no_windows",
                            "No NSWindows found"
                        );
                        return;
                    }

                    // Log all windows for diagnosis
                    let mut confirm_ns_window: cocoa::base::id = nil;
                    for i in 0..count {
                        let w: cocoa::base::id = msg_send![windows, objectAtIndex: i];
                        if w == nil {
                            continue;
                        }
                        let frame: cocoa::foundation::NSRect = msg_send![w, frame];
                        let level: i64 = msg_send![w, level];
                        let is_visible: bool = msg_send![w, isVisible];
                        let is_key: bool = msg_send![w, isKeyWindow];

                        tracing::info!(
                            target: "script_kit::confirm",
                            event = "configure_confirm_nswindow_enumerate",
                            index = i,
                            ptr = format!("{:?}", w),
                            x = frame.origin.x,
                            y = frame.origin.y,
                            w = frame.size.width,
                            h = frame.size.height,
                            level,
                            is_visible,
                            is_key,
                            "NSWindow[{}]: {:?} frame=({:.0},{:.0} {:.0}x{:.0}) level={} visible={} key={}",
                            i, w, frame.origin.x, frame.origin.y,
                            frame.size.width, frame.size.height,
                            level, is_visible, is_key
                        );

                        // Match by approximate frame bounds (GPUI may apply
                        // slight adjustments, so use 2px tolerance)
                        if (frame.size.width - expected_w as f64).abs() < 2.0
                            && (frame.size.height - expected_h as f64).abs() < 2.0
                            && is_visible
                        {
                            tracing::info!(
                                target: "script_kit::confirm",
                                event = "configure_confirm_nswindow_matched",
                                index = i,
                                ptr = format!("{:?}", w),
                                "Matched confirm popup NSWindow by frame size"
                            );
                            confirm_ns_window = w;
                        }
                    }

                    // Fallback to lastObject if frame matching didn't find it
                    if confirm_ns_window == nil {
                        confirm_ns_window = msg_send![windows, lastObject];
                        tracing::warn!(
                            target: "script_kit::confirm",
                            event = "configure_confirm_nswindow_fallback",
                            ptr = format!("{:?}", confirm_ns_window),
                            "Frame match failed, falling back to lastObject"
                        );
                    }

                    if confirm_ns_window != nil {
                        tracing::info!(
                            target: "script_kit::confirm",
                            event = "configure_confirm_popup_applying",
                            ptr = format!("{:?}", confirm_ns_window),
                            is_dark_vibrancy,
                            "Applying vibrancy + level + makeKey to confirm NSWindow"
                        );
                        platform::configure_confirm_popup_window(confirm_ns_window, is_dark_vibrancy);

                        // Attach confirm as child of the main window so AppKit
                        // guarantees it stays above the parent across render cycles.
                        // This is more robust than orderFrontRegardless alone at the
                        // same window level (both are 101 for PopUp windows).
                        // Find the main window: prefer the current key window,
                        // fall back to the first visible window that isn't the confirm.
                        let mut main_ns_window: cocoa::base::id = nil;
                        let mut fallback_ns_window: cocoa::base::id = nil;
                        for idx in 0..count {
                            let w: cocoa::base::id = msg_send![windows, objectAtIndex: idx];
                            if w != nil && w != confirm_ns_window {
                                let w_visible: bool = msg_send![w, isVisible];
                                if w_visible {
                                    let w_key: bool = msg_send![w, isKeyWindow];
                                    if w_key {
                                        main_ns_window = w;
                                        break;
                                    }
                                    if fallback_ns_window == nil {
                                        fallback_ns_window = w;
                                    }
                                }
                            }
                        }
                        if main_ns_window == nil {
                            main_ns_window = fallback_ns_window;
                            if main_ns_window == nil {
                                tracing::warn!(
                                    target: "script_kit::confirm",
                                    event = "confirm_window.no_parent_found",
                                    "No visible parent window found for addChildWindow"
                                );
                            }
                        }
                        if main_ns_window != nil {
                            // SAFETY: both pointers verified non-nil and distinct.
                            let _: () = msg_send![main_ns_window, addChildWindow:confirm_ns_window ordered:NS_WINDOW_ABOVE];
                        }

                        // Always order front + make key regardless of parent attachment.
                        // orderFrontRegardless is needed for the no-parent fallback and
                        // also ensures the child is visually ordered even if addChildWindow
                        // doesn't immediately reorder on non-activating panels.
                        let _: () = msg_send![confirm_ns_window, orderFrontRegardless];
                        let _: () = msg_send![confirm_ns_window, makeKeyWindow];
                        let is_key: bool = msg_send![confirm_ns_window, isKeyWindow];
                        if !is_key {
                            tracing::warn!(
                                target: "script_kit::confirm",
                                event = "confirm_window.make_key_failed",
                                "makeKeyWindow did not make confirm popup the key window"
                            );
                        }
                    } else {
                        tracing::error!(
                            target: "script_kit::confirm",
                            event = "configure_confirm_popup_no_window",
                            "Cannot configure confirm popup: no NSWindow found"
                        );
                    }
                }
            });
        });
    }

    let storage = CONFIRM_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = storage.lock() {
        *guard = Some(handle);
    }

    // Store result sender and focused button state for key routing from main window
    let tx_storage = CONFIRM_RESULT_TX.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = tx_storage.lock() {
        *guard = Some(result_tx);
    }
    let btn_storage = CONFIRM_FOCUSED_BUTTON.get_or_init(|| Mutex::new(FocusedButton::Confirm));
    if let Ok(mut guard) = btn_storage.lock() {
        *guard = FocusedButton::Confirm;
    }

    tracing::info!(
        target: "script_kit::confirm",
        event = "open_confirm_popup_window_ready",
        "open_confirm_popup_window: handle stored, popup ready"
    );

    Ok(handle)
}

pub(crate) struct ConfirmPopupWindow {
    title: SharedString,
    body: SharedString,
    confirm_text: SharedString,
    cancel_text: SharedString,
    confirm_variant: ButtonVariant,
    focus_handle: FocusHandle,
    focused_button: FocusedButton,
    keep_open_while: Rc<dyn Fn() -> bool>,
    result_tx: async_channel::Sender<bool>,
    lifecycle_task: Option<Task<()>>,
    did_request_focus: bool,
    resolved: bool,
}

impl ConfirmPopupWindow {
    fn new(
        options: ConfirmWindowOptions,
        keep_open_while: Rc<dyn Fn() -> bool>,
        result_tx: async_channel::Sender<bool>,
        cx: &mut Context<Self>,
    ) -> Self {
        tracing::info!(
            target: "script_kit::confirm",
            event = "confirm_popup_window_new",
            title = %options.title,
            body_len = options.body.len(),
            confirm_text = %options.confirm_text,
            cancel_text = %options.cancel_text,
            "ConfirmPopupWindow::new"
        );
        Self {
            title: options.title,
            body: options.body,
            confirm_text: options.confirm_text,
            cancel_text: options.cancel_text,
            confirm_variant: options.confirm_variant,
            focus_handle: cx.focus_handle(),
            focused_button: FocusedButton::Confirm,
            keep_open_while,
            result_tx,
            lifecycle_task: None,
            did_request_focus: false,
            resolved: false,
        }
    }

    fn shift_focus(&mut self, reverse: bool, cx: &mut Context<Self>) {
        self.focused_button = match (self.focused_button, reverse) {
            (FocusedButton::Cancel, false) => FocusedButton::Confirm,
            (FocusedButton::Confirm, false) => FocusedButton::Cancel,
            (FocusedButton::Cancel, true) => FocusedButton::Confirm,
            (FocusedButton::Confirm, true) => FocusedButton::Cancel,
        };
        cx.notify();
    }

    fn ensure_lifecycle_task(&mut self, cx: &mut Context<Self>) {
        if self.lifecycle_task.is_some() {
            return;
        }

        tracing::info!(
            target: "script_kit::confirm",
            event = "lifecycle_task_started",
            poll_interval_ms = CONFIRM_LIFECYCLE_POLL_MS,
            "Starting confirm window lifecycle polling task"
        );

        let keep_open_while = self.keep_open_while.clone();
        let result_tx = self.result_tx.clone();

        self.lifecycle_task = Some(cx.spawn(async move |this, cx| {
            let mut poll_count: u64 = 0;
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(CONFIRM_LIFECYCLE_POLL_MS))
                    .await;

                poll_count += 1;
                let predicate_result = (keep_open_while)();

                if predicate_result {
                    if poll_count.is_multiple_of(50) {
                        tracing::debug!(
                            target: "script_kit::confirm",
                            event = "lifecycle_poll_heartbeat",
                            poll_count,
                            "Lifecycle predicate still true"
                        );
                    }
                    continue;
                }

                tracing::info!(
                    target: "script_kit::confirm",
                    event = "lifecycle_predicate_false",
                    poll_count,
                    "Lifecycle predicate returned false — closing confirm window"
                );

                let _ = this.update(cx, |this, _cx| {
                    if !this.resolved {
                        tracing::info!(
                            target: "script_kit::confirm",
                            event = "lifecycle_auto_cancel",
                            "Auto-cancelling confirm (lifecycle predicate false)"
                        );
                        this.resolved = true;
                        let _ = result_tx.try_send(false);
                    }
                });

                cx.update(|cx| {
                    close_confirm_window(cx);
                });

                break;
            }
        }));
    }

    // NOTE: We intentionally do NOT observe_window_activation here.
    // In Accessory app mode the app is never truly "active" in the macOS
    // sense, so the window would report as inactive immediately and close
    // itself. Instead we rely on the lifecycle polling task and explicit
    // user actions (confirm/cancel/escape) to close the window.

    fn resolve_and_close(&mut self, confirmed: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.resolved {
            tracing::debug!(
                target: "script_kit::confirm",
                event = "resolve_and_close_already_resolved",
                confirmed,
                "resolve_and_close: already resolved, ignoring"
            );
            return;
        }

        tracing::info!(
            target: "script_kit::confirm",
            event = "resolve_and_close",
            confirmed,
            "resolve_and_close: sending result and closing window"
        );

        self.resolved = true;
        let _ = self.result_tx.try_send(confirmed);

        window.defer(cx, |window, _cx| {
            tracing::info!(
                target: "script_kit::confirm",
                event = "resolve_and_close_deferred",
                "resolve_and_close: deferred removal executing"
            );
            clear_confirm_window_handle();
            window.remove_window();
        });
    }
}

impl Drop for ConfirmPopupWindow {
    fn drop(&mut self) {
        tracing::warn!(
            target: "script_kit::confirm",
            event = "confirm_popup_window_DROPPED",
            resolved = self.resolved,
            title = %self.title,
            "ConfirmPopupWindow entity DROPPED — if resolved=false, the window was destroyed externally"
        );
        if !self.resolved {
            tracing::error!(
                target: "script_kit::confirm",
                event = "confirm_popup_window_DROPPED_UNRESOLVED",
                "ConfirmPopupWindow dropped WITHOUT resolving — this will send false to the result channel"
            );
        }
    }
}

impl Focusable for ConfirmPopupWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ConfirmPopupWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_focused = self.focus_handle.is_focused(window);
        let is_active = window.is_window_active();
        tracing::info!(
            target: "script_kit::confirm",
            event = "confirm_popup_render",
            is_focused,
            is_active,
            resolved = self.resolved,
            focused_button = ?self.focused_button,
            did_request_focus = self.did_request_focus,
            "ConfirmPopupWindow::render"
        );

        self.ensure_lifecycle_task(cx);

        if !self.did_request_focus {
            self.did_request_focus = true;
            tracing::info!(
                target: "script_kit::confirm",
                event = "confirm_popup_requesting_focus",
                "Requesting initial focus for confirm popup"
            );
            window.focus(&self.focus_handle, cx);
        }

        let handle_key = cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
            let key = event.keystroke.key.as_str();
            let modifiers = &event.keystroke.modifiers;
            let intent = confirm_window_key_intent(key, modifiers);

            tracing::info!(
                target: "script_kit::confirm",
                event = "confirm_popup_key_down",
                key,
                intent = ?intent,
                "Confirm popup received key"
            );

            match intent {
                Some(ConfirmWindowKeyIntent::Cancel) => {
                    tracing::info!(
                        target: "script_kit::confirm",
                        event = "confirm_popup_escape",
                        "User pressed Escape — cancelling"
                    );
                    this.resolve_and_close(false, window, cx);
                }
                Some(ConfirmWindowKeyIntent::ActivateFocused) => {
                    let confirmed = matches!(this.focused_button, FocusedButton::Confirm);
                    tracing::info!(
                        target: "script_kit::confirm",
                        event = "confirm_popup_enter",
                        confirmed,
                        focused_button = ?this.focused_button,
                        "User pressed Enter — activating focused button"
                    );
                    this.resolve_and_close(confirmed, window, cx);
                }
                Some(ConfirmWindowKeyIntent::FocusNext) => {
                    this.shift_focus(false, cx);
                }
                Some(ConfirmWindowKeyIntent::FocusPrev) => {
                    this.shift_focus(true, cx);
                }
                None => {
                    tracing::debug!(
                        target: "script_kit::confirm",
                        event = "confirm_popup_consume_unhandled_key",
                        key,
                        "Confirm popup consumed unhandled key"
                    );
                }
            }
            cx.stop_propagation();
        });

        let theme = get_cached_theme();
        let chrome = crate::theme::AppChromeColors::from_theme(&theme);
        let title_color = theme.colors.text.primary.to_rgb();
        let body_color = theme.colors.text.secondary.to_rgb();
        let muted_color = theme.colors.text.dimmed.to_rgb();
        let surface_bg = gpui::transparent_black();
        let panel_bg = gpui::rgba(chrome.dialog_surface_rgba);
        let is_danger = matches!(self.confirm_variant, ButtonVariant::Danger);

        // Border color: red-tinted for danger, subtle for normal
        let top_border_color = if is_danger {
            theme.colors.ui.error.with_opacity(0.15)
        } else {
            theme.colors.ui.border.with_opacity(0.30)
        };

        // Confirm action colors
        let (confirm_keycap_bg, confirm_keycap_color, confirm_label_color) = if is_danger {
            let e = theme.colors.ui.error;
            (e.with_opacity(0.06), e.to_rgb(), e.to_rgb())
        } else {
            let a = theme.colors.accent.selected;
            (a.with_opacity(0.06), a.to_rgb(), title_color)
        };

        // Cancel action colors
        let cancel_keycap_bg = theme.colors.ui.border.with_opacity(0.06);

        // Read focused button state for visual feedback
        let current_focused = get_confirm_focused_button();
        let cancel_focused = current_focused == FocusedButton::Cancel;
        let confirm_focused = current_focused == FocusedButton::Confirm;

        let entity = cx.entity();
        let cancel_entity = entity.clone();
        let confirm_entity = entity.clone();

        // ── Title row: optional icon + title ────────────────────
        let title_row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(5.))
            .when(is_danger, |d| {
                d.child(
                    div()
                        .text_xs()
                        .text_color(theme.colors.ui.error.to_rgb())
                        .child("⚠"),
                )
            })
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(title_color)
                    .child(self.title.clone()),
            );

        // ── Keycap action row ───────────────────────────────────
        let action_row = div()
            .w_full()
            .flex()
            .flex_row()
            .justify_end()
            .gap(px(CONFIRM_BUTTON_GAP))
            // Cancel: [Esc] Cancel
            .child(
                div()
                    .id("confirm-cancel-button")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(3.))
                    .cursor_pointer()
                    .when(cancel_focused, |d| d.opacity(1.0))
                    .when(!cancel_focused, |d| d.opacity(0.8))
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        cancel_entity.update(cx, |this: &mut Self, cx| {
                            this.resolve_and_close(false, window, cx);
                        });
                    })
                    .child(
                        div()
                            .px(px(4.))
                            .py(px(1.))
                            .rounded(px(3.))
                            .bg(cancel_keycap_bg)
                            .text_xs()
                            .font_family(FONT_MONO)
                            .text_color(muted_color)
                            .child("Esc"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted_color)
                            .child(self.cancel_text.clone()),
                    ),
            )
            // Confirm: [↵] Clear/Delete
            .child(
                div()
                    .id("confirm-ok-button")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(3.))
                    .cursor_pointer()
                    .when(confirm_focused, |d| d.opacity(1.0))
                    .when(!confirm_focused, |d| d.opacity(0.8))
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        confirm_entity.update(cx, |this: &mut Self, cx| {
                            this.resolve_and_close(true, window, cx);
                        });
                    })
                    .child(
                        div()
                            .px(px(4.))
                            .py(px(1.))
                            .rounded(px(3.))
                            .bg(confirm_keycap_bg)
                            .text_xs()
                            .font_family(FONT_MONO)
                            .text_color(confirm_keycap_color)
                            .child("↵"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(confirm_label_color)
                            .child(self.confirm_text.clone()),
                    ),
            );

        div()
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .bg(surface_bg)
            .overflow_hidden()
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .bg(panel_bg)
                    .px(px(CONFIRM_PADDING_X))
                    .py(px(CONFIRM_PADDING_Y))
                    .gap(px(CONFIRM_SECTION_GAP))
                    .border_t_1()
                    .border_color(top_border_color)
                    // Title row
                    .child(title_row)
                    // Body (if present)
                    .when(!self.body.is_empty(), |d| {
                        d.child(
                            div()
                                .w_full()
                                .text_xs()
                                .text_color(body_color)
                                .child(self.body.clone()),
                        )
                    })
                    // Action row
                    .child(action_row),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_window_key_intent_maps_escape_enter_and_tab_navigation() {
        let no_mods = gpui::Modifiers::default();
        let shift_mods = gpui::Modifiers {
            shift: true,
            ..Default::default()
        };

        assert_eq!(
            confirm_window_key_intent("escape", &no_mods),
            Some(ConfirmWindowKeyIntent::Cancel)
        );
        assert_eq!(
            confirm_window_key_intent("Enter", &no_mods),
            Some(ConfirmWindowKeyIntent::ActivateFocused)
        );
        assert_eq!(
            confirm_window_key_intent("tab", &no_mods),
            Some(ConfirmWindowKeyIntent::FocusNext)
        );
        assert_eq!(
            confirm_window_key_intent("tab", &shift_mods),
            Some(ConfirmWindowKeyIntent::FocusPrev)
        );
        assert_eq!(
            confirm_window_key_intent("arrowleft", &no_mods),
            Some(ConfirmWindowKeyIntent::FocusPrev)
        );
        assert_eq!(
            confirm_window_key_intent("right", &no_mods),
            Some(ConfirmWindowKeyIntent::FocusNext)
        );
    }

    #[test]
    fn confirm_window_dynamic_height_grows_with_body_length() {
        let short = confirm_window_dynamic_height(px(448.), "Confirm", "Short body.");
        let long = confirm_window_dynamic_height(
            px(448.),
            "Confirm",
            &"This is a much longer confirmation body. ".repeat(40),
        );

        assert!(long > short);
        assert!(long <= CONFIRM_MAX_HEIGHT);
    }

    #[test]
    fn confirm_window_bounds_bottom_aligned_over_parent_window() {
        let parent_bounds = Bounds {
            origin: Point {
                x: px(100.),
                y: px(200.),
            },
            size: Size {
                width: px(750.),
                height: px(500.),
            },
        };

        let bounds = confirm_window_bounds(parent_bounds, px(448.), "Confirm", "Body");
        let actual_x: f32 = bounds.origin.x.into();
        let actual_y: f32 = bounds.origin.y.into();
        let actual_w: f32 = bounds.size.width.into();

        let expected_height = confirm_window_dynamic_height(px(448.), "Confirm", "Body");
        // Should match parent x and width
        let expected_x = 100.0;
        // Should bottom-align: parent_y + parent_h - confirm_h
        let expected_y = 200.0 + 500.0 - expected_height;

        assert!((actual_x - expected_x).abs() < 0.5);
        assert!((actual_y - expected_y).abs() < 0.5);
        // Width matches parent
        assert!((actual_w - 750.0).abs() < 0.5);
    }
}
