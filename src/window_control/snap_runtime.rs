use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::Result;
use gpui::{App, AsyncApp};

use super::snap::build_snap_targets_for_mode;
use super::snap_mode::{current_snap_mode, SnapMode};
use super::snap_overlay::{hide_snap_overlay, show_snap_overlay};
use super::snap_session::{
    begin_snap_session, build_overlay_scene, cancel_snap_session, finish_snap_session,
    poll_window_bounds, prime_snap_session, tick_snap_session, update_session_display,
    SnapDisplayTargets, SnapSession,
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

/// Check whether a snap runtime is currently active.
pub fn is_snap_runtime_active() -> bool {
    ACTIVE_SNAP_RUNTIME
        .lock()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

/// Start a live snap runtime: begin tracking the frontmost external window
/// and render the desktop overlay.
pub fn start_snap_runtime(cx: &mut App) -> Result<()> {
    if current_snap_mode() == SnapMode::Off {
        tracing::info!(
            target: "script_kit::snap_runtime",
            event = "snap_runtime_start_blocked_mode_off",
            "snap runtime not started because snap mode is Off"
        );
        return Ok(());
    }

    if is_snap_runtime_active() {
        tracing::info!(
            target: "script_kit::snap_runtime",
            event = "snap_runtime_start_skipped_already_active",
            "snap runtime already active"
        );
        return Ok(());
    }

    let mut session = begin_snap_session()?;
    prime_snap_session(&mut session, Instant::now());
    show_snap_overlay(build_overlay_scene(&session), cx)?;

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

/// Advance the runtime by one tick. Returns `false` only when the runtime
/// is no longer active (for example the tracked window disappeared).
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
        drop(guard);
        hide_snap_overlay(cx)?;
        return Ok(false);
    };

    update_session_display(&mut runtime.session, &current_bounds);
    let phase = tick_snap_session(&mut runtime.session, current_bounds, Instant::now());
    let overlay_scene = build_overlay_scene(&runtime.session);

    tracing::info!(
        target: "script_kit::snap_runtime",
        event = "snap_runtime_tick",
        window_id = runtime.session.window_id,
        ?phase,
        matched = runtime.session.active_match.is_some(),
        matched_tile = runtime
            .session
            .active_match
            .map(|m| format!("{:?}", m.target.tile)),
        "updated snap runtime"
    );

    // Release lock before overlay update.
    drop(guard);
    show_snap_overlay(overlay_scene, cx)?;

    Ok(true)
}

/// Finish the snap runtime on mouse-up. Commits when there is an active match,
/// otherwise cancels cleanly.
pub fn finish_snap_runtime(cx: &mut App) -> Result<()> {
    let mut guard = ACTIVE_SNAP_RUNTIME
        .lock()
        .map_err(|e| anyhow::anyhow!("snap runtime lock poisoned: {e}"))?;

    let Some(runtime) = guard.take() else {
        return Ok(());
    };

    let outcome = finish_snap_session(&runtime.session)?;
    tracing::info!(
        target: "script_kit::snap_runtime",
        event = "snap_runtime_finished",
        ?outcome,
        "finished snap runtime"
    );

    drop(guard);
    hide_snap_overlay(cx)?;

    Ok(())
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

/// Refresh the active runtime after a snap-mode change without losing the
/// currently tracked window or overlay lifecycle.
pub fn refresh_snap_runtime_for_mode(cx: &mut App) -> Result<()> {
    let mut guard = ACTIVE_SNAP_RUNTIME
        .lock()
        .map_err(|e| anyhow::anyhow!("snap runtime lock poisoned: {e}"))?;

    let Some(runtime) = guard.as_mut() else {
        return Ok(());
    };

    let mode = current_snap_mode();
    runtime.session.mode = mode;
    runtime.session.all_display_targets = runtime
        .session
        .all_display_targets
        .iter()
        .map(|dt| SnapDisplayTargets {
            display: dt.display,
            targets: build_snap_targets_for_mode(&dt.display, mode),
        })
        .collect();

    if let Some(dt) = runtime
        .session
        .all_display_targets
        .iter()
        .find(|dt| dt.display == runtime.session.display)
    {
        runtime.session.targets = dt.targets.clone();
    } else if let Some(first) = runtime.session.all_display_targets.first() {
        runtime.session.display = first.display;
        runtime.session.targets = first.targets.clone();
    } else {
        runtime.session.targets.clear();
    }

    let current_bounds = runtime.session.last_window_bounds;
    update_session_display(&mut runtime.session, &current_bounds);
    let _ = tick_snap_session(&mut runtime.session, current_bounds, Instant::now());
    let scene = build_overlay_scene(&runtime.session);

    tracing::info!(
        target: "script_kit::snap_runtime",
        event = "snap_runtime_mode_refreshed",
        window_id = runtime.session.window_id,
        ?mode,
        target_count = runtime.session.targets.len(),
        display_count = runtime.session.all_display_targets.len(),
        "refreshed snap runtime for snap mode change"
    );

    drop(guard);
    show_snap_overlay(scene, cx)?;

    Ok(())
}
