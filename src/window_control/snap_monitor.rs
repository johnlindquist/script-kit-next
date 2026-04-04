use std::sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock, Mutex,
};

use anyhow::{anyhow, bail, Result};
use gpui::{App, AsyncApp};

use super::query::{get_frontmost_window_of_previous_app, has_accessibility_permission};
use super::snap_runtime::{finish_snap_runtime, is_snap_runtime_active, start_snap_runtime};
use super::types::{Bounds, WindowInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SnapMonitorEvent {
    Pressed,
    Dragged,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DragArmState {
    window_id: u32,
    bounds: Bounds,
}

static INSTALLED: AtomicBool = AtomicBool::new(false);
static DRAG_ARM_STATE: LazyLock<Mutex<Option<DragArmState>>> = LazyLock::new(|| Mutex::new(None));
static SNAP_MONITOR_CHANNEL: LazyLock<(
    async_channel::Sender<SnapMonitorEvent>,
    async_channel::Receiver<SnapMonitorEvent>,
)> = LazyLock::new(|| async_channel::bounded(128));

fn origin_changed(previous: Bounds, current: Bounds) -> bool {
    previous.x != current.x || previous.y != current.y
}

fn arm_state_for_window(window: &WindowInfo) -> DragArmState {
    DragArmState {
        window_id: window.id,
        bounds: window.bounds,
    }
}

fn should_start_runtime(armed: DragArmState, current: Option<&WindowInfo>) -> bool {
    matches!(
        current,
        Some(window)
            if window.id == armed.window_id
                && origin_changed(armed.bounds, window.bounds)
    )
}

fn handle_snap_monitor_event(event: SnapMonitorEvent, cx: &mut App) -> Result<()> {
    if !has_accessibility_permission() {
        return Ok(());
    }

    match event {
        SnapMonitorEvent::Pressed => {
            if is_snap_runtime_active() {
                return Ok(());
            }

            let armed = get_frontmost_window_of_previous_app()?
                .as_ref()
                .map(arm_state_for_window);
            *DRAG_ARM_STATE
                .lock()
                .map_err(|e| anyhow!("snap monitor arm lock poisoned: {e}"))? = armed;
        }
        SnapMonitorEvent::Dragged => {
            if is_snap_runtime_active() {
                return Ok(());
            }

            let armed = *DRAG_ARM_STATE
                .lock()
                .map_err(|e| anyhow!("snap monitor arm lock poisoned: {e}"))?;
            let Some(armed) = armed else {
                return Ok(());
            };

            let current = get_frontmost_window_of_previous_app()?;

            if should_start_runtime(armed, current.as_ref()) {
                *DRAG_ARM_STATE
                    .lock()
                    .map_err(|e| anyhow!("snap monitor arm lock poisoned: {e}"))? = None;
                tracing::info!(
                    target: "script_kit::snap_monitor",
                    event = "snap_drag_started",
                    window_id = armed.window_id,
                    "detected external window drag"
                );
                start_snap_runtime(cx)?;
            } else if !matches!(current.as_ref(), Some(window) if window.id == armed.window_id) {
                *DRAG_ARM_STATE
                    .lock()
                    .map_err(|e| anyhow!("snap monitor arm lock poisoned: {e}"))? = None;
            }
        }
        SnapMonitorEvent::Released => {
            *DRAG_ARM_STATE
                .lock()
                .map_err(|e| anyhow!("snap monitor arm lock poisoned: {e}"))? = None;

            if is_snap_runtime_active() {
                tracing::info!(
                    target: "script_kit::snap_monitor",
                    event = "snap_drag_released",
                    "detected external drag release"
                );
                finish_snap_runtime(cx)?;
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
struct SendableId(cocoa::base::id);

#[cfg(target_os = "macos")]
// SAFETY: AppKit monitor IDs are opaque Objective-C objects managed on the main thread.
unsafe impl Send for SendableId {}

#[cfg(target_os = "macos")]
// SAFETY: The opaque monitor handle is only stored for lifetime management.
unsafe impl Sync for SendableId {}

#[cfg(target_os = "macos")]
static GLOBAL_MOUSE_MONITOR: LazyLock<Mutex<Option<SendableId>>> =
    LazyLock::new(|| Mutex::new(None));

#[cfg(target_os = "macos")]
fn install_global_mouse_monitor() -> Result<()> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    if crate::platform::require_main_thread("install_global_mouse_monitor") {
        return Ok(());
    }

    if GLOBAL_MOUSE_MONITOR
        .lock()
        .map_err(|e| anyhow!("snap mouse monitor lock poisoned: {e}"))?
        .is_some()
    {
        return Ok(());
    }

    // NSEventMaskLeftMouseDown = 1 << 1
    // NSEventMaskLeftMouseUp = 1 << 2
    // NSEventMaskLeftMouseDragged = 1 << 6
    let mask: u64 = (1 << 1) | (1 << 2) | (1 << 6);

    let block = block::ConcreteBlock::new(move |event: id| {
        // SAFETY: `event` is a valid NSEvent delivered by AppKit.
        let event_type: usize = unsafe { msg_send![event, type] };
        let snap_event = match event_type {
            1 => Some(SnapMonitorEvent::Pressed),
            2 => Some(SnapMonitorEvent::Released),
            6 => Some(SnapMonitorEvent::Dragged),
            _ => None,
        };

        if let Some(snap_event) = snap_event {
            let _ = SNAP_MONITOR_CHANNEL.0.try_send(snap_event);
        }
    });
    let block = block.copy();

    // SAFETY: NSEvent is a valid AppKit class on macOS and the monitor is installed
    // on the main thread. The returned opaque monitor handle is retained by AppKit
    // until removed or process exit.
    let monitor: id = unsafe {
        msg_send![
            class!(NSEvent),
            addGlobalMonitorForEventsMatchingMask: mask
            handler: &*block
        ]
    };

    if monitor == nil {
        bail!("Failed to install global snap mouse monitor");
    }

    *GLOBAL_MOUSE_MONITOR
        .lock()
        .map_err(|e| anyhow!("snap mouse monitor lock poisoned: {e}"))? = Some(SendableId(monitor));

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn install_global_mouse_monitor() -> Result<()> {
    Ok(())
}

/// Install the snap drag monitor that detects external window drags and
/// drives the desktop snap overlay lifecycle (start on drag, finish on release).
pub fn install_snap_drag_monitor(cx: &mut App) -> Result<()> {
    if INSTALLED.swap(true, Ordering::SeqCst) {
        tracing::info!(
            target: "script_kit::snap_monitor",
            event = "snap_drag_monitor_already_installed",
            "snap drag monitor already installed"
        );
        return Ok(());
    }

    install_global_mouse_monitor()?;

    tracing::info!(
        target: "script_kit::snap_monitor",
        event = "snap_drag_monitor_installed",
        mode = "global_mouse_monitor",
        "installed snap drag monitor"
    );

    let rx = SNAP_MONITOR_CHANNEL.1.clone();
    cx.spawn(async move |cx: &mut AsyncApp| {
        while let Ok(event) = rx.recv().await {
            cx.update(|cx| {
                if let Err(error) = handle_snap_monitor_event(event, cx) {
                    tracing::warn!(
                        target: "script_kit::snap_monitor",
                        event = "snap_drag_monitor_event_failed",
                        ?event,
                        %error,
                        "snap drag monitor event failed"
                    );
                }
            });
        }
    })
    .detach();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_window(id: u32, bounds: Bounds) -> WindowInfo {
        WindowInfo::for_test(id, "TestApp".to_string(), "Window".to_string(), bounds, 123)
    }

    #[test]
    fn starts_when_same_window_origin_changes() {
        let armed = DragArmState {
            window_id: 7,
            bounds: Bounds::new(900, 100, 1200, 800),
        };
        let current = make_window(7, Bounds::new(750, 100, 1200, 800));
        assert!(should_start_runtime(armed, Some(&current)));
    }

    #[test]
    fn does_not_start_on_size_only_change() {
        let armed = DragArmState {
            window_id: 7,
            bounds: Bounds::new(900, 100, 1200, 800),
        };
        let current = make_window(7, Bounds::new(900, 100, 1400, 800));
        assert!(!should_start_runtime(armed, Some(&current)));
    }

    #[test]
    fn does_not_start_for_different_window() {
        let armed = DragArmState {
            window_id: 7,
            bounds: Bounds::new(900, 100, 1200, 800),
        };
        let current = make_window(9, Bounds::new(750, 100, 1200, 800));
        assert!(!should_start_runtime(armed, Some(&current)));
    }

    #[test]
    fn does_not_start_without_window() {
        let armed = DragArmState {
            window_id: 7,
            bounds: Bounds::new(900, 100, 1200, 800),
        };
        assert!(!should_start_runtime(armed, None));
    }
}
