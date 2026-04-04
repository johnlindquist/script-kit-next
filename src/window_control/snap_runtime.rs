use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::Result;
use gpui::{App, AsyncApp};

use super::snap_overlay::{hide_snap_overlay, show_snap_overlay};
use super::snap_session::{
    begin_snap_session, build_overlay_model, cancel_snap_session, finish_snap_session,
    poll_window_bounds, tick_snap_session, update_session_display, SnapSession, SnapSessionPhase,
};

/// Polling interval for tracking the dragged window (~60 fps).
const SNAP_POLL_INTERVAL: Duration = Duration::from_millis(16);

// ---------------------------------------------------------------------------
// Active runtime state
// ---------------------------------------------------------------------------

struct ActiveSnapRuntime {
    session: SnapSession,
}

static ACTIVE_SNAP_RUNTIME: Mutex<Option<ActiveSnapRuntime>> = Mutex::new(None);

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Start a live snap runtime: begin tracking the frontmost external window
/// and render the desktop overlay.
pub fn start_snap_runtime(cx: &mut App) -> Result<()> {
    let session = begin_snap_session()?;
    show_snap_overlay(build_overlay_model(&session), cx)?;

    tracing::info!(
        target: "script_kit::snap_runtime",
        event = "snap_runtime_started",
        window_id = session.window_id,
        app_name = %session.app_name,
        title = %session.window_title,
        "started snap runtime"
    );

    {
        let mut guard = ACTIVE_SNAP_RUNTIME
            .lock()
            .map_err(|e| anyhow::anyhow!("snap runtime lock poisoned: {e}"))?;
        *guard = Some(ActiveSnapRuntime { session });
    }

    cx.spawn(async move |cx: &mut AsyncApp| loop {
        cx.background_executor().timer(SNAP_POLL_INTERVAL).await;

        let keep_running = cx.update(|cx| tick_snap_runtime(cx).unwrap_or(false));

        if !keep_running {
            break;
        }
    })
    .detach();

    Ok(())
}

/// Advance the runtime by one tick. Returns `false` when the runtime is done.
///
/// Called from the polling loop; also usable for testing.
pub fn tick_snap_runtime(cx: &mut App) -> Result<bool> {
    let mut guard = ACTIVE_SNAP_RUNTIME
        .lock()
        .map_err(|e| anyhow::anyhow!("snap runtime lock poisoned: {e}"))?;

    let Some(runtime) = guard.as_mut() else {
        return Ok(false);
    };

    let Some(current_bounds) = poll_window_bounds(&runtime.session) else {
        tracing::info!(
            target: "script_kit::snap_runtime",
            event = "snap_runtime_window_gone",
            window_id = runtime.session.window_id,
            "tracked window disappeared"
        );
        let _session = guard.take();
        // Release the lock before calling into overlay (which may acquire its own lock).
        drop(guard);
        hide_snap_overlay(cx)?;
        return Ok(false);
    };

    update_session_display(&mut runtime.session, current_bounds.x, current_bounds.y);
    let phase = tick_snap_session(&mut runtime.session, current_bounds, Instant::now());
    let overlay_model = build_overlay_model(&runtime.session);

    if phase == SnapSessionPhase::Settled {
        let outcome = finish_snap_session(&runtime.session);
        tracing::info!(
            target: "script_kit::snap_runtime",
            event = "snap_runtime_finished",
            ?outcome,
            "finished snap runtime"
        );
        guard.take();
        drop(guard);
        hide_snap_overlay(cx)?;
        return Ok(false);
    }

    // Release lock before overlay update.
    drop(guard);
    show_snap_overlay(overlay_model, cx)?;

    Ok(true)
}

/// Cancel the snap runtime without applying changes.
pub fn cancel_snap_runtime(cx: &mut App) -> Result<()> {
    let mut guard = ACTIVE_SNAP_RUNTIME
        .lock()
        .map_err(|e| anyhow::anyhow!("snap runtime lock poisoned: {e}"))?;

    if let Some(runtime) = guard.take() {
        let outcome = cancel_snap_session(&runtime.session);
        tracing::info!(
            target: "script_kit::snap_runtime",
            event = "snap_runtime_cancelled",
            ?outcome,
            "cancelled snap runtime"
        );
    }

    // Release lock before overlay call.
    drop(guard);
    hide_snap_overlay(cx)?;

    Ok(())
}
