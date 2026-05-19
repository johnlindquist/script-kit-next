// Confirm popup window — a native GPUI WindowKind::PopUp window with macOS
// vibrancy blur. Replaces the old in-window overlay dialog approach so the
// confirmation surface gets real NSPanel blur instead of plain transparency.

use std::{
    rc::Rc,
    sync::{Mutex, OnceLock},
    time::Duration,
};

use gpui::{
    div, prelude::*, px, AnyWindowHandle, App, Bounds, Context, DisplayId, FocusHandle, Focusable,
    MouseButton, Pixels, Point, Render, SharedString, Size, Task, Window,
    WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind, WindowOptions,
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
    // Unregister from automation registry before destroying the window
    crate::windows::remove_automation_window("confirm-popup");

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

pub(crate) struct ConfirmPopupParentWindow {
    pub(crate) handle: AnyWindowHandle,
    pub(crate) bounds: Bounds<Pixels>,
    pub(crate) display_id: Option<DisplayId>,
    pub(crate) automation_id: Option<String>,
}

/// Read an NSWindow's `title` as a Rust String, or `None` if the title is nil
/// or not valid UTF-8. Safe to call on the AppKit main thread inside the
/// confirm-popup defer block where we already hold raw NSWindow pointers.
#[cfg(target_os = "macos")]
unsafe fn nswindow_title_string(window: cocoa::base::id) -> Option<String> {
    use cocoa::base::nil;
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CStr;
    if window == nil {
        return None;
    }
    let title: cocoa::base::id = msg_send![window, title];
    if title == nil {
        return None;
    }
    let title_cstr: *const std::os::raw::c_char = msg_send![title, UTF8String];
    if title_cstr.is_null() {
        return None;
    }
    Some(CStr::from_ptr(title_cstr).to_string_lossy().into_owned())
}

fn automation_bounds_from_gpui(bounds: Bounds<Pixels>) -> crate::protocol::AutomationWindowBounds {
    crate::protocol::AutomationWindowBounds {
        x: f32::from(bounds.origin.x) as f64,
        y: f32::from(bounds.origin.y) as f64,
        width: f32::from(bounds.size.width) as f64,
        height: f32::from(bounds.size.height) as f64,
    }
}

pub(crate) fn open_confirm_popup_window(
    cx: &mut App,
    parent_window: ConfirmPopupParentWindow,
    options: ConfirmWindowOptions,
    keep_open_while: Rc<dyn Fn() -> bool>,
    result_tx: async_channel::Sender<bool>,
) -> anyhow::Result<WindowHandle<ConfirmPopupWindow>> {
    let parent_automation_id = resolve_confirm_popup_parent_automation_id(
        parent_window.handle,
        parent_window.bounds,
        parent_window.automation_id.as_deref(),
        options.title.as_ref(),
    )?;

    tracing::info!(
        target: "script_kit::confirm",
        event = "open_confirm_popup_window",
        title = %options.title,
        parent_x = ?parent_window.bounds.origin.x,
        parent_y = ?parent_window.bounds.origin.y,
        parent_w = ?parent_window.bounds.size.width,
        parent_h = ?parent_window.bounds.size.height,
        display_id = ?parent_window.display_id,
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
        parent_window.bounds,
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
            display_id: parent_window.display_id,
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

    // Capture intended parent identity + frame so the AppKit attach step can
    // pick the *intended* parent NSWindow deterministically, instead of
    // defaulting to whichever window happens to be `isKeyWindow` when the
    // defer block runs (which is brittle when Notes / ACP / main coexist).
    let parent_automation_id_for_nswindow = parent_automation_id.clone();
    let parent_expected_w: f32 = parent_window.bounds.size.width.into();
    let parent_expected_h: f32 = parent_window.bounds.size.height.into();
    let parent_expected_title =
        crate::windows::automation_window_by_id(&parent_automation_id).and_then(|info| info.title);

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

                        // Attach confirm as child of the parent window so AppKit
                        // keeps it above and moves it with the parent.
                        //
                        // Deterministic match first: prefer the NSWindow whose
                        // frame size matches the parent we computed bounds from,
                        // optionally cross-checked against the registered
                        // automation title. This protects multi-window setups
                        // (Notes / ACP / main coexist) where the key window may
                        // not be the *intended* parent. Fall back to the legacy
                        // isKeyWindow / first-visible heuristic only if the
                        // deterministic match fails.
                        let mut main_ns_window: cocoa::base::id = nil;
                        for idx in 0..count {
                            let w: cocoa::base::id = msg_send![windows, objectAtIndex: idx];
                            if w == nil || w == confirm_ns_window {
                                continue;
                            }
                            let w_visible: bool = msg_send![w, isVisible];
                            if !w_visible {
                                continue;
                            }
                            let frame: cocoa::foundation::NSRect = msg_send![w, frame];
                            let size_matches = (frame.size.width
                                - parent_expected_w as f64)
                                .abs()
                                < 2.0
                                && (frame.size.height - parent_expected_h as f64).abs() < 2.0;
                            let title_opt = nswindow_title_string(w);
                            let expected_title_matches = parent_expected_title
                                .as_deref()
                                .is_some_and(|expected| title_opt.as_deref() == Some(expected));
                            let notes_title_matches = parent_automation_id_for_nswindow == "notes"
                                && title_opt.as_deref() == Some("Notes");
                            if (size_matches && expected_title_matches) || notes_title_matches {
                                main_ns_window = w;
                                tracing::info!(
                                    target: "script_kit::confirm",
                                    event = "confirm_window_parent_matched_by_automation_id",
                                    parent_window_id = %parent_automation_id_for_nswindow,
                                    parent_title = ?title_opt,
                                    parent_w = frame.size.width,
                                    parent_h = frame.size.height,
                                    "Matched confirm popup parent NSWindow deterministically"
                                );
                                break;
                            }
                        }
                        if main_ns_window == nil {
                            tracing::warn!(
                                target: "script_kit::confirm",
                                event = "confirm_window_parent_deterministic_match_failed",
                                parent_window_id = %parent_automation_id_for_nswindow,
                                expected_title = ?parent_expected_title,
                                expected_w = parent_expected_w,
                                expected_h = parent_expected_h,
                                "Falling back to legacy key/visible-window parent search"
                            );
                            let mut fallback_ns_window: cocoa::base::id = nil;
                            for idx in 0..count {
                                let w: cocoa::base::id =
                                    msg_send![windows, objectAtIndex: idx];
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
                        }
                        if main_ns_window != nil {
                            // SAFETY: both pointers verified non-nil and distinct.
                            let _: () = msg_send![main_ns_window, addChildWindow:confirm_ns_window ordered:NS_WINDOW_ABOVE];
                            tracing::info!(
                                target: "script_kit::confirm",
                                event = "confirm_window_attached_to_parent",
                                parent_window_id = %parent_automation_id_for_nswindow,
                                parent = format!("{:?}", main_ns_window),
                                child = format!("{:?}", confirm_ns_window),
                                "Attached confirm popup as native child window"
                            );
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

    // Register in the automation window registry with parent identity.
    // Fail-closed: if registration fails, close the popup and propagate the error.
    if let Err(e) = crate::windows::register_attached_popup(
        "confirm-popup".to_string(),
        crate::protocol::AutomationWindowKind::PromptPopup,
        Some(options.title.to_string()),
        Some("confirmDialog".to_string()),
        Some(automation_bounds_from_gpui(bounds)),
        Some(parent_automation_id.as_str()),
    ) {
        tracing::warn!(
            target: "script_kit::confirm",
            event = "confirm_popup_registry_failed",
            error = %e,
            "Failed to register confirm popup in automation registry — closing popup"
        );
        close_confirm_window(cx);
        return Err(e);
    }

    tracing::info!(
        target: "script_kit::confirm",
        event = "open_confirm_popup_window_ready",
        "open_confirm_popup_window: handle stored, popup ready"
    );

    Ok(handle)
}

/// Validate an explicit `parent_automation_id` (e.g. `"notes"`) against the
/// automation registry. Returns the id back on success; bails when the id is
/// not registered so the popup fails closed instead of attaching to a
/// surprise parent. Extracted from `resolve_confirm_popup_parent_automation_id`
/// so it can be exercised without fabricating a GPUI `AnyWindowHandle` in
/// unit tests.
fn resolve_registered_parent_automation_id(
    parent_automation_id: &str,
    title: &str,
) -> anyhow::Result<String> {
    let Some(parent_info) = crate::windows::automation_window_by_id(parent_automation_id) else {
        tracing::warn!(
            target: "script_kit::confirm",
            event = "confirm_popup_open_blocked_unknown_parent",
            title,
            parent_window_id = parent_automation_id,
            "Confirm popup open blocked: explicit parent automation id is not registered"
        );
        anyhow::bail!(
            "Cannot open confirm popup: parent automation id '{}' is not registered",
            parent_automation_id
        );
    };
    tracing::info!(
        target: "script_kit::confirm",
        event = "confirm_popup_resolved_explicit_parent",
        parent_window_id = %parent_automation_id,
        parent_kind = ?parent_info.kind,
        "Resolved explicit confirm popup parent automation identity"
    );
    Ok(parent_automation_id.to_string())
}

fn resolve_confirm_popup_parent_automation_id(
    parent_window_handle: AnyWindowHandle,
    parent_window_bounds: Bounds<Pixels>,
    parent_automation_id: Option<&str>,
    title: &str,
) -> anyhow::Result<String> {
    if let Some(id) = parent_automation_id {
        let resolved = resolve_registered_parent_automation_id(id, title)?;
        // Refresh the registered runtime handle + live bounds so downstream
        // automation snapshots (and the AppKit child-window lookup) reflect the
        // exact parent we are about to attach to.
        crate::windows::upsert_runtime_window_handle(&resolved, parent_window_handle);
        crate::windows::set_automation_bounds(
            &resolved,
            Some(automation_bounds_from_gpui(parent_window_bounds)),
        );
        return Ok(resolved);
    }

    let Some(main_window_handle) = crate::get_main_window_handle() else {
        tracing::warn!(
            target: "script_kit::confirm",
            event = "confirm_popup_open_blocked_missing_parent",
            title,
            "Confirm popup open blocked: no parent automation identity"
        );
        anyhow::bail!("Cannot open confirm popup: parent automation identity is required");
    };

    if main_window_handle != parent_window_handle {
        tracing::warn!(
            target: "script_kit::confirm",
            event = "confirm_popup_open_blocked_missing_parent",
            title,
            "Confirm popup open blocked: no parent automation identity"
        );
        anyhow::bail!("Cannot open confirm popup: parent automation identity is required");
    }

    let synthesized_parent_id = "main".to_string();
    crate::windows::upsert_runtime_window_handle(&synthesized_parent_id, parent_window_handle);
    crate::windows::upsert_automation_window(crate::protocol::AutomationWindowInfo {
        id: synthesized_parent_id.clone(),
        kind: crate::protocol::AutomationWindowKind::Main,
        title: Some("Script Kit".to_string()),
        focused: true,
        visible: true,
        semantic_surface: Some("scriptList".to_string()),
        bounds: Some(crate::protocol::AutomationWindowBounds {
            x: f32::from(parent_window_bounds.origin.x) as f64,
            y: f32::from(parent_window_bounds.origin.y) as f64,
            width: f32::from(parent_window_bounds.size.width) as f64,
            height: f32::from(parent_window_bounds.size.height) as f64,
        }),
        parent_window_id: None,
        parent_kind: None,
        pid: Some(std::process::id()),
    });
    tracing::info!(
        target: "script_kit::confirm",
        event = "confirm_popup_synthesized_main_parent",
        parent_window_id = %synthesized_parent_id,
        "Synthesized main-window automation identity for confirm popup"
    );

    Ok(synthesized_parent_id)
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
        let panel_bg = gpui::rgba(chrome.popup_surface_rgba);
        let is_danger = matches!(self.confirm_variant, ButtonVariant::Danger);

        // Border color: red-tinted for danger, subtle for normal
        let top_border_color = if is_danger {
            theme.colors.ui.error.with_opacity(0.15)
        } else {
            theme.colors.ui.border.with_opacity(0.30)
        };

        // Read focused button state for visual feedback
        let current_focused = get_confirm_focused_button();
        let cancel_focused = current_focused == FocusedButton::Cancel;
        let confirm_focused = current_focused == FocusedButton::Confirm;

        // Per-button accent: focused button uses theme accent (or error for danger
        // Confirm), unfocused uses muted gray. This makes the active Tab target
        // visually obvious instead of leaving the user guessing which button Enter
        // will hit.
        let accent = theme.colors.accent.selected;
        let muted_keycap_bg = theme.colors.ui.border.with_opacity(0.06);

        let (cancel_keycap_bg, cancel_keycap_color, cancel_label_color) = if cancel_focused {
            (accent.with_opacity(0.06), accent.to_rgb(), title_color)
        } else {
            (muted_keycap_bg, muted_color, muted_color)
        };

        // Both keycaps use theme.accent.selected when focused so the Esc and
        // Enter glyphs share a consistent visual key style. Danger only colors
        // the *label* (the action verb) so the destructive intent still reads,
        // without making the keycap glyph itself diverge from the Cancel side.
        let confirm_label_color_focused = if is_danger {
            theme.colors.ui.error.to_rgb()
        } else {
            title_color
        };
        let (confirm_keycap_bg, confirm_keycap_color, confirm_label_color) = if confirm_focused {
            (
                accent.with_opacity(0.06),
                accent.to_rgb(),
                confirm_label_color_focused,
            )
        } else {
            (muted_keycap_bg, muted_color, muted_color)
        };

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
                            .text_color(cancel_keycap_color)
                            .child("Esc"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cancel_label_color)
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

    #[test]
    fn confirm_window_bounds_bottom_aligned_notes_sized_parent() {
        let parent_bounds = Bounds {
            origin: Point {
                x: px(960.),
                y: px(80.),
            },
            size: Size {
                width: px(350.),
                height: px(280.),
            },
        };

        let bounds = confirm_window_bounds(
            parent_bounds,
            px(326.),
            "Move note to Trash",
            "Move this note to Trash? You can restore it later with \u{2318}\u{21e7}T.",
        );

        let x: f32 = bounds.origin.x.into();
        let y: f32 = bounds.origin.y.into();
        let width: f32 = bounds.size.width.into();
        let height: f32 = bounds.size.height.into();

        assert!((x - 960.0).abs() < 0.5, "popup x must match parent x");
        assert!(
            (width - 350.0).abs() < 0.5,
            "popup width must match parent width"
        );
        assert!(
            ((y + height) - (80.0 + 280.0)).abs() < 0.5,
            "popup must be flush with parent bottom edge"
        );
    }

    #[test]
    fn confirm_popup_parent_search_prefers_automation_id_before_key_window() {
        let source = std::fs::read_to_string("src/confirm/window.rs")
            .expect("Failed to read src/confirm/window.rs");

        let attach_section = source
            .split("Deterministic match first")
            .nth(1)
            .expect("expected AppKit deterministic-match attach section");

        assert!(
            attach_section.contains("parent_automation_id_for_nswindow")
                && attach_section.contains("confirm_window_parent_matched_by_automation_id"),
            "confirm popup parent NSWindow search should prefer the resolved automation id"
        );

        let automation_match_idx = attach_section
            .find("confirm_window_parent_matched_by_automation_id")
            .expect("expected deterministic parent match event");
        let key_window_idx = attach_section
            .find("msg_send![w, isKeyWindow]")
            .expect("legacy fallback may still call msg_send![w, isKeyWindow]");

        assert!(
            automation_match_idx < key_window_idx,
            "automation-id parent matching must happen before key-window fallback"
        );
    }

    #[test]
    fn resolve_registered_parent_accepts_notes_and_rejects_unknown_ids() {
        use crate::protocol::{AutomationWindowInfo, AutomationWindowKind};

        crate::windows::upsert_automation_window(AutomationWindowInfo {
            id: "notes".to_string(),
            kind: AutomationWindowKind::Notes,
            title: Some("Notes".to_string()),
            focused: true,
            visible: true,
            semantic_surface: Some("notes".to_string()),
            bounds: None,
            parent_window_id: None,
            parent_kind: None,
            pid: Some(std::process::id()),
        });

        let resolved = resolve_registered_parent_automation_id("notes", "Move note to Trash")
            .expect("explicit Notes parent id must resolve");
        assert_eq!(resolved, "notes");

        let unknown = resolve_registered_parent_automation_id("nope-unknown-window-id", "x");
        assert!(
            unknown.is_err(),
            "resolver must reject unregistered explicit parent ids"
        );

        crate::windows::remove_automation_window("notes");
    }

    #[test]
    fn confirm_popup_can_synthesize_main_parent_identity() {
        let source = std::fs::read_to_string("src/confirm/window.rs")
            .expect("Failed to read src/confirm/window.rs");

        assert!(
            source.contains("fn resolve_confirm_popup_parent_automation_id("),
            "confirm popup should resolve the parent automation identity through a dedicated helper"
        );
        assert!(
            source.contains("event = \"confirm_popup_synthesized_main_parent\""),
            "confirm popup should log when it synthesizes the main-window automation identity"
        );
        assert!(
            source.contains("crate::windows::upsert_runtime_window_handle(&synthesized_parent_id, parent_window_handle);"),
            "confirm popup should register the synthesized main-window runtime handle"
        );
    }
}
