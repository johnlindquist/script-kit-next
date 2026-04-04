use std::sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock, Mutex,
};
use std::time::Duration;

use anyhow::Result;
use gpui::{App, AsyncApp};

use super::query::{get_frontmost_window_of_previous_app, has_accessibility_permission};
use super::snap_runtime::{finish_snap_runtime, is_snap_runtime_active, start_snap_runtime};
use super::types::Bounds;

const SNAP_MONITOR_INTERVAL: Duration = Duration::from_millis(16);

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Snapshot {
    window_id: Option<u32>,
    bounds: Option<Bounds>,
    left_mouse_down: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MonitorAction {
    None,
    StartRuntime,
    FinishRuntime,
}

static INSTALLED: AtomicBool = AtomicBool::new(false);
static LAST_SNAPSHOT: LazyLock<Mutex<Snapshot>> = LazyLock::new(|| Mutex::new(Snapshot::default()));

#[cfg(target_os = "macos")]
fn left_mouse_button_down() -> bool {
    // SAFETY: NSEvent.pressedMouseButtons is a class method that returns a bitmask
    // of currently pressed mouse buttons. Bit 0 = left button. This is safe to call
    // from any thread per Apple documentation.
    unsafe {
        use objc::{class, msg_send, sel, sel_impl};
        let buttons: u64 = msg_send![class!(NSEvent), pressedMouseButtons];
        (buttons & 1) != 0
    }
}

#[cfg(not(target_os = "macos"))]
fn left_mouse_button_down() -> bool {
    false
}

fn origin_changed(previous: Bounds, current: Bounds) -> bool {
    previous.x != current.x || previous.y != current.y
}

fn monitor_action(previous: Snapshot, current: Snapshot, active_runtime: bool) -> MonitorAction {
    let moved_same_window = match (
        previous.window_id,
        previous.bounds,
        current.window_id,
        current.bounds,
    ) {
        (Some(prev_id), Some(prev_bounds), Some(curr_id), Some(curr_bounds))
            if prev_id == curr_id =>
        {
            origin_changed(prev_bounds, curr_bounds)
        }
        _ => false,
    };

    if current.left_mouse_down && moved_same_window && !active_runtime {
        MonitorAction::StartRuntime
    } else if previous.left_mouse_down && !current.left_mouse_down && active_runtime {
        MonitorAction::FinishRuntime
    } else {
        MonitorAction::None
    }
}

/// Install the snap drag monitor that detects external window drags and
/// drives the snap runtime lifecycle (start on drag, finish on release).
pub fn install_snap_drag_monitor(cx: &mut App) -> Result<()> {
    if INSTALLED.swap(true, Ordering::SeqCst) {
        tracing::info!(
            target: "script_kit::snap_monitor",
            event = "snap_drag_monitor_already_installed",
            "snap drag monitor already installed"
        );
        return Ok(());
    }

    tracing::info!(
        target: "script_kit::snap_monitor",
        event = "snap_drag_monitor_installed",
        interval_ms = SNAP_MONITOR_INTERVAL.as_millis() as u64,
        "installed snap drag monitor"
    );

    cx.spawn(async move |cx: &mut AsyncApp| loop {
        cx.background_executor().timer(SNAP_MONITOR_INTERVAL).await;

        cx.update(|cx| {
            if let Err(error) = tick_snap_drag_monitor(cx) {
                tracing::warn!(
                    target: "script_kit::snap_monitor",
                    event = "snap_drag_monitor_tick_failed",
                    %error,
                    "snap drag monitor tick failed"
                );
            }
        });
    })
    .detach();

    Ok(())
}

fn tick_snap_drag_monitor(cx: &mut App) -> Result<()> {
    if !has_accessibility_permission() {
        return Ok(());
    }

    let left_mouse_down = left_mouse_button_down();

    let window_snapshot = get_frontmost_window_of_previous_app()
        .ok()
        .flatten()
        .map(|window| Snapshot {
            window_id: Some(window.id),
            bounds: Some(window.bounds),
            left_mouse_down,
        })
        .unwrap_or(Snapshot {
            window_id: None,
            bounds: None,
            left_mouse_down,
        });

    let active_runtime = is_snap_runtime_active();

    let mut previous = LAST_SNAPSHOT
        .lock()
        .map_err(|e| anyhow::anyhow!("snap monitor lock poisoned: {e}"))?;

    match monitor_action(*previous, window_snapshot, active_runtime) {
        MonitorAction::StartRuntime => {
            tracing::info!(
                target: "script_kit::snap_monitor",
                event = "snap_drag_started",
                window_id = ?window_snapshot.window_id,
                "detected external window drag"
            );
            *previous = window_snapshot;
            drop(previous);
            start_snap_runtime(cx)?;
        }
        MonitorAction::FinishRuntime => {
            tracing::info!(
                target: "script_kit::snap_monitor",
                event = "snap_drag_released",
                window_id = ?window_snapshot.window_id,
                "detected external drag release"
            );
            *previous = window_snapshot;
            drop(previous);
            finish_snap_runtime(cx)?;
        }
        MonitorAction::None => {
            *previous = window_snapshot;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_when_same_window_moves_with_mouse_down() {
        let previous = Snapshot {
            window_id: Some(7),
            bounds: Some(Bounds::new(900, 100, 1200, 800)),
            left_mouse_down: false,
        };
        let current = Snapshot {
            window_id: Some(7),
            bounds: Some(Bounds::new(750, 100, 1200, 800)),
            left_mouse_down: true,
        };
        assert_eq!(
            monitor_action(previous, current, false),
            MonitorAction::StartRuntime
        );
    }

    #[test]
    fn do_not_start_on_size_only_change() {
        let previous = Snapshot {
            window_id: Some(7),
            bounds: Some(Bounds::new(900, 100, 1200, 800)),
            left_mouse_down: false,
        };
        let current = Snapshot {
            window_id: Some(7),
            bounds: Some(Bounds::new(900, 100, 1400, 800)),
            left_mouse_down: true,
        };
        assert_eq!(monitor_action(previous, current, false), MonitorAction::None);
    }

    #[test]
    fn finish_when_mouse_releases_with_active_runtime() {
        let previous = Snapshot {
            window_id: Some(7),
            bounds: Some(Bounds::new(750, 100, 1200, 800)),
            left_mouse_down: true,
        };
        let current = Snapshot {
            window_id: Some(7),
            bounds: Some(Bounds::new(720, 100, 1200, 800)),
            left_mouse_down: false,
        };
        assert_eq!(
            monitor_action(previous, current, true),
            MonitorAction::FinishRuntime
        );
    }

    #[test]
    fn no_action_when_no_window() {
        let previous = Snapshot::default();
        let current = Snapshot {
            left_mouse_down: true,
            ..Default::default()
        };
        assert_eq!(monitor_action(previous, current, false), MonitorAction::None);
    }

    #[test]
    fn no_start_when_runtime_already_active() {
        let previous = Snapshot {
            window_id: Some(7),
            bounds: Some(Bounds::new(900, 100, 1200, 800)),
            left_mouse_down: false,
        };
        let current = Snapshot {
            window_id: Some(7),
            bounds: Some(Bounds::new(750, 100, 1200, 800)),
            left_mouse_down: true,
        };
        // active_runtime = true → should not start again
        assert_eq!(monitor_action(previous, current, true), MonitorAction::None);
    }

    #[test]
    fn no_finish_when_runtime_not_active() {
        let previous = Snapshot {
            window_id: Some(7),
            bounds: Some(Bounds::new(750, 100, 1200, 800)),
            left_mouse_down: true,
        };
        let current = Snapshot {
            window_id: Some(7),
            bounds: Some(Bounds::new(720, 100, 1200, 800)),
            left_mouse_down: false,
        };
        // active_runtime = false → should not finish
        assert_eq!(
            monitor_action(previous, current, false),
            MonitorAction::None
        );
    }
}
