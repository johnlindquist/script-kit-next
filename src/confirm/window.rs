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
    ui_foundation::{
        get_vibrancy_surface_background, is_key_enter, is_key_escape, is_key_left, is_key_tab,
        HexColorExt,
    },
};

const CONFIRM_PADDING_X: f32 = 20.0;
const CONFIRM_PADDING_Y: f32 = 18.0;
const CONFIRM_SECTION_GAP: f32 = 14.0;
const CONFIRM_BUTTON_GAP: f32 = 10.0;
const CONFIRM_BUTTON_HEIGHT: f32 = 38.0;
const CONFIRM_TITLE_LINE_HEIGHT: f32 = 24.0;
const CONFIRM_BODY_LINE_HEIGHT: f32 = 18.0;
const CONFIRM_MIN_HEIGHT: f32 = 156.0;
const CONFIRM_MAX_HEIGHT: f32 = 360.0;
const CONFIRM_BODY_MAX_LINES: usize = 10;
const CONFIRM_LIFECYCLE_POLL_MS: u64 = 120;
const CONFIRM_RADIUS: f32 = 14.0;
// NSModalPanelWindowLevel = 8 — above NSFloatingWindowLevel (3) so the
// confirm popup appears in front of the main window.
const NS_MODAL_PANEL_WINDOW_LEVEL: i64 = 8;

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

    let x = parent_bounds.origin.x + (parent_bounds.size.width - width) / 2.0;
    let y = parent_bounds.origin.y + (parent_bounds.size.height - height) / 2.0;

    Bounds {
        origin: Point { x, y },
        size: Size { width, height },
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
pub(crate) fn route_key_to_confirm_popup(key: &str, cx: &mut App) -> bool {
    if !is_confirm_window_open() {
        return false;
    }

    let intent = confirm_window_key_intent(key, &gpui::Modifiers::default());

    tracing::info!(
        target: "script_kit::confirm",
        event = "route_key_to_confirm_popup",
        key,
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
        None => false,
    }
}

fn send_confirm_result(confirmed: bool) {
    if let Some(storage) = CONFIRM_RESULT_TX.get() {
        if let Ok(mut guard) = storage.lock() {
            if let Some(tx) = guard.take() {
                let _ = tx.try_send(confirmed);
            }
        }
    }
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
        move |_window, cx| {
            cx.new(|cx| ConfirmPopupWindow::new(request, lifecycle, sender, cx))
        },
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

                        // SAFETY: confirm_ns_window verified non-nil.
                        // Override the level set by configure_actions_popup_window
                        // (which sets NSFloatingWindowLevel=3). The confirm popup
                        // needs to be above the main window, so use modal panel level.
                        let _: () = msg_send![confirm_ns_window, setLevel: NS_MODAL_PANEL_WINDOW_LEVEL];

                        // Bring the confirm popup to front and give it keyboard focus.
                        let _: () = msg_send![confirm_ns_window, orderFrontRegardless];
                        let _: () = msg_send![confirm_ns_window, makeKeyWindow];

                        // Verify key status after makeKeyWindow
                        let is_key_after: bool = msg_send![confirm_ns_window, isKeyWindow];
                        let level_after: i64 = msg_send![confirm_ns_window, level];
                        tracing::info!(
                            target: "script_kit::confirm",
                            event = "configure_confirm_popup_done",
                            is_key_after,
                            level_after,
                            "Confirm popup configured: isKey={}, level={}",
                            is_key_after, level_after
                        );
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
                None => {}
            }
        });

        let theme = get_cached_theme();
        let title_color = theme.colors.text.primary.to_rgb();
        let body_color = theme.colors.text.secondary.to_rgb();
        let muted_color = theme.colors.text.dimmed.to_rgb();
        let border_color = theme.colors.ui.border.with_opacity(0.42);
        let divider_color = theme.colors.ui.border.with_opacity(0.28);
        let surface_bg = gpui::transparent_black();

        // Read focused button from shared state (main window may have toggled it via key routing)
        let current_focused = get_confirm_focused_button();
        let cancel_focused = current_focused == FocusedButton::Cancel;
        let cancel_bg = if cancel_focused {
            theme.colors.accent.selected_subtle.with_opacity(0.62)
        } else {
            theme.colors.background.main.with_opacity(0.18)
        };
        let cancel_border = if cancel_focused {
            theme.colors.accent.selected.with_opacity(0.90)
        } else {
            divider_color
        };
        let cancel_text_color = if cancel_focused {
            title_color
        } else {
            body_color
        };
        let cancel_shortcut_bg = theme.colors.ui.border.with_opacity(0.18);
        let cancel_shortcut_text_color = muted_color;

        let confirm_focused = current_focused == FocusedButton::Confirm;
        let is_danger = matches!(self.confirm_variant, ButtonVariant::Danger);

        let (
            confirm_bg,
            confirm_border,
            confirm_text_color,
            confirm_shortcut_bg,
            confirm_shortcut_text_color,
        ) = if is_danger {
            let base = theme.colors.ui.error;
            (
                base.with_opacity(if confirm_focused { 0.24 } else { 0.14 }),
                base.with_opacity(if confirm_focused { 0.92 } else { 0.58 }),
                base.to_rgb(),
                base.with_opacity(0.14),
                base.to_rgb(),
            )
        } else {
            let base = theme.colors.accent.selected;
            let subtle = theme.colors.accent.selected_subtle;
            (
                if confirm_focused {
                    base.with_opacity(0.22)
                } else {
                    subtle.with_opacity(0.52)
                },
                base.with_opacity(if confirm_focused { 0.92 } else { 0.58 }),
                title_color,
                base.with_opacity(0.14),
                base.to_rgb(),
            )
        };

        let entity = cx.entity();
        let cancel_entity = entity.clone();
        let confirm_entity = entity.clone();

        let cancel_button = div()
            .id("confirm-cancel-button")
            .flex_1()
            .h(px(CONFIRM_BUTTON_HEIGHT))
            .px(px(12.))
            .rounded(px(10.))
            .border_1()
            .border_color(cancel_border)
            .bg(cancel_bg)
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(8.))
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                cancel_entity.update(cx, |this: &mut Self, cx| {
                    this.resolve_and_close(false, window, cx);
                });
            })
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(cancel_text_color)
                    .child(self.cancel_text.clone()),
            )
            .child(
                div()
                    .px(px(6.))
                    .py(px(2.))
                    .rounded(px(4.))
                    .bg(cancel_shortcut_bg)
                    .text_xs()
                    .font_family(FONT_MONO)
                    .text_color(cancel_shortcut_text_color)
                    .child("Esc"),
            );

        let confirm_button = div()
            .id("confirm-ok-button")
            .flex_1()
            .h(px(CONFIRM_BUTTON_HEIGHT))
            .px(px(12.))
            .rounded(px(10.))
            .border_1()
            .border_color(confirm_border)
            .bg(confirm_bg)
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(8.))
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                confirm_entity.update(cx, |this: &mut Self, cx| {
                    this.resolve_and_close(true, window, cx);
                });
            })
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(confirm_text_color)
                    .child(self.confirm_text.clone()),
            )
            .child(
                div()
                    .px(px(6.))
                    .py(px(2.))
                    .rounded(px(4.))
                    .bg(confirm_shortcut_bg)
                    .text_xs()
                    .font_family(FONT_MONO)
                    .text_color(confirm_shortcut_text_color)
                    .child("↵"),
            );

        div()
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .bg(surface_bg)
            .rounded(px(CONFIRM_RADIUS))
            .border_1()
            .border_color(border_color)
            .overflow_hidden()
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .px(px(CONFIRM_PADDING_X))
                    .py(px(CONFIRM_PADDING_Y))
                    .gap(px(CONFIRM_SECTION_GAP))
                    .child(
                        div()
                            .w_full()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(title_color)
                            .child(self.title.clone()),
                    )
                    .when(!self.body.is_empty(), |d| {
                        d.child(
                            div()
                                .w_full()
                                .text_sm()
                                .text_color(body_color)
                                .child(self.body.clone()),
                        )
                    })
                    .child(
                        div()
                            .w_full()
                            .border_t_1()
                            .border_color(divider_color)
                            .pt(px(CONFIRM_SECTION_GAP))
                            .flex()
                            .flex_row()
                            .gap(px(CONFIRM_BUTTON_GAP))
                            .child(cancel_button)
                            .child(confirm_button),
                    ),
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
    fn confirm_window_bounds_center_over_parent_window() {
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

        let expected_height = confirm_window_dynamic_height(px(448.), "Confirm", "Body");
        let expected_x = 100.0 + ((750.0 - 448.0) / 2.0);
        let expected_y = 200.0 + ((500.0 - expected_height) / 2.0);

        assert!((actual_x - expected_x).abs() < 0.5);
        assert!((actual_y - expected_y).abs() < 0.5);
    }
}
