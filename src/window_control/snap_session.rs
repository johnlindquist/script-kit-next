use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use super::ax::{get_window_position, get_window_size};
use super::cache::get_cached_window;
use super::display::get_visible_display_bounds;
use super::snap::{best_snap_match, build_snap_targets, SnapMatch, SnapTarget};
use super::types::{Bounds, TilePosition};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Minimum overlap ratio to consider a snap target as active.
const MIN_OVERLAP_RATIO: f64 = 0.35;

/// Duration of inactivity (no window movement) before settling.
const SETTLE_DURATION: Duration = Duration::from_millis(120);

// ---------------------------------------------------------------------------
// Session types
// ---------------------------------------------------------------------------

/// Phase of the snap session state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapSessionPhase {
    /// Tracking window movement, haven't seen any movement yet.
    WaitingForMovement,
    /// Window has moved at least once; actively tracking and updating.
    Tracking,
    /// Movement settled; session is complete.
    Settled,
}

/// Outcome of a completed snap session.
#[derive(Debug, Clone)]
pub enum SnapSessionOutcome {
    /// Session committed: the window was snapped to a target.
    Committed { tile: TilePosition, bounds: Bounds },
    /// Session cancelled: no active target when movement settled.
    Cancelled,
}

/// Data model for the desktop snap overlay (consumed by the renderer).
#[derive(Debug, Clone, PartialEq)]
pub struct SnapOverlayModel {
    /// Visible display bounds for positioning the overlay window.
    pub display_bounds: Bounds,
    /// All snap targets with their active/inactive state.
    pub targets: Vec<SnapOverlayTarget>,
}

/// A single snap target for overlay rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapOverlayTarget {
    pub tile: TilePosition,
    pub bounds: Bounds,
    pub active: bool,
}

/// The live snap session state.
#[derive(Debug, Clone)]
pub struct SnapSession {
    /// The tracked external window's ID.
    pub window_id: u32,
    /// Application name of the tracked window.
    pub app_name: String,
    /// Window title.
    pub window_title: String,
    /// Visible display bounds where snap targets are computed.
    pub display: Bounds,
    /// Pre-computed snap targets for the display.
    pub targets: Vec<SnapTarget>,
    /// Current best snap match (if any target exceeds the overlap threshold).
    pub active_match: Option<SnapMatch>,
    /// Last known window bounds (updated on each tick).
    pub last_window_bounds: Bounds,
    /// Session phase.
    pub phase: SnapSessionPhase,
    /// Timestamp of the last detected window movement.
    last_movement_time: Option<Instant>,
    /// Whether we've seen at least one movement during this session.
    has_moved: bool,
    /// Unique session ID for trace correlation.
    session_id: u64,
}

/// Monotonic session ID counter.
static SESSION_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

// ---------------------------------------------------------------------------
// Session lifecycle
// ---------------------------------------------------------------------------

/// Begin a new snap session from the frontmost external window.
///
/// Resolves the menu-bar-owning app's focused window, reads its bounds,
/// determines the display it lives on, and pre-computes snap targets.
pub fn begin_snap_session() -> Result<SnapSession> {
    let window = super::query::get_frontmost_window_of_previous_app()?
        .context("No frontmost external window available for snap session")?;

    let display_bounds = get_visible_display_bounds(window.bounds.x, window.bounds.y);
    let targets = build_snap_targets(&display_bounds);
    let session_id = SESSION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    tracing::info!(
        target: "script_kit::snap_session",
        event = "snap_session_started",
        session_id,
        window_id = window.id,
        app = %window.app,
        title = %window.title,
        display_x = display_bounds.x,
        display_y = display_bounds.y,
        display_w = display_bounds.width,
        display_h = display_bounds.height,
        target_count = targets.len(),
        "began snap session"
    );

    Ok(SnapSession {
        window_id: window.id,
        app_name: window.app,
        window_title: window.title,
        display: display_bounds,
        targets,
        active_match: None,
        last_window_bounds: window.bounds,
        phase: SnapSessionPhase::WaitingForMovement,
        last_movement_time: None,
        has_moved: false,
        session_id,
    })
}

/// Poll the tracked window's current AX bounds.
///
/// Returns `None` if the window is no longer accessible (e.g., closed).
pub fn poll_window_bounds(session: &SnapSession) -> Option<Bounds> {
    let window = get_cached_window(session.window_id)?;
    let (x, y) = get_window_position(window.as_ptr()).ok()?;
    let (w, h) = get_window_size(window.as_ptr()).ok()?;
    Some(Bounds::new(x, y, w, h))
}

/// Re-evaluate the display for the session based on the current window position.
///
/// Call this from the GPUI polling loop when the window may have moved to a
/// different display. This is separate from `tick_snap_session` because it
/// touches macOS NSScreen APIs that require the main thread.
pub fn update_session_display(session: &mut SnapSession, window_x: i32, window_y: i32) {
    let new_display = get_visible_display_bounds(window_x, window_y);
    if new_display != session.display {
        session.display = new_display;
        session.targets = build_snap_targets(&new_display);
        tracing::info!(
            target: "script_kit::snap_session",
            event = "snap_session_display_changed",
            session_id = session.session_id,
            display_x = new_display.x,
            display_y = new_display.y,
            display_w = new_display.width,
            display_h = new_display.height,
            "snap session display changed"
        );
    }
}

/// Advance the session by one tick with the current window bounds.
///
/// Updates the active snap match and phase. Returns the updated phase.
/// This function is pure computation — no macOS API calls.
pub fn tick_snap_session(
    session: &mut SnapSession,
    current_bounds: Bounds,
    now: Instant,
) -> SnapSessionPhase {
    let moved = current_bounds != session.last_window_bounds;

    if moved {
        session.has_moved = true;
        session.last_movement_time = Some(now);
        session.phase = SnapSessionPhase::Tracking;
    }

    // Recompute snap match when tracking.
    if session.has_moved {
        session.active_match =
            best_snap_match(&current_bounds, &session.targets, MIN_OVERLAP_RATIO);
    }

    session.last_window_bounds = current_bounds;

    // Check settling: movement burst happened, then no movement for SETTLE_DURATION.
    if session.has_moved && !moved {
        if let Some(last_move) = session.last_movement_time {
            if now.duration_since(last_move) >= SETTLE_DURATION {
                session.phase = SnapSessionPhase::Settled;
            }
        }
    }

    tracing::info!(
        target: "script_kit::snap_session",
        event = "snap_session_updated",
        session_id = session.session_id,
        window_id = session.window_id,
        x = current_bounds.x,
        y = current_bounds.y,
        w = current_bounds.width,
        h = current_bounds.height,
        moved,
        phase = ?session.phase,
        matched = session.active_match.is_some(),
        matched_tile = session.active_match.map(|m| format!("{:?}", m.target.tile)),
        overlap = session.active_match.map(|m| m.overlap_ratio),
        "tick snap session"
    );

    session.phase
}

/// Build an overlay model from the current session state.
///
/// The caller is responsible for passing this model to the overlay renderer.
pub fn build_overlay_model(session: &SnapSession) -> SnapOverlayModel {
    let active_tile = session.active_match.map(|m| m.target.tile);

    SnapOverlayModel {
        display_bounds: session.display,
        targets: session
            .targets
            .iter()
            .map(|t| SnapOverlayTarget {
                tile: t.tile,
                bounds: t.bounds,
                active: Some(t.tile) == active_tile,
            })
            .collect(),
    }
}

/// Commit the snap session: apply the active match bounds to the tracked window.
///
/// Returns `SnapSessionOutcome::Committed` on success.
pub fn commit_snap_session(session: &SnapSession) -> Result<SnapSessionOutcome> {
    let active = session
        .active_match
        .context("No active snap target to commit")?;

    tracing::info!(
        target: "script_kit::snap_session",
        event = "snap_session_committed",
        session_id = session.session_id,
        window_id = session.window_id,
        tile = ?active.target.tile,
        target_x = active.target.bounds.x,
        target_y = active.target.bounds.y,
        target_w = active.target.bounds.width,
        target_h = active.target.bounds.height,
        overlap = active.overlap_ratio,
        "committing snap session"
    );

    super::actions::set_window_bounds(session.window_id, active.target.bounds)?;

    Ok(SnapSessionOutcome::Committed {
        tile: active.target.tile,
        bounds: active.target.bounds,
    })
}

/// Cancel the snap session without applying any changes.
pub fn cancel_snap_session(session: &SnapSession) -> SnapSessionOutcome {
    tracing::info!(
        target: "script_kit::snap_session",
        event = "snap_session_cancelled",
        session_id = session.session_id,
        window_id = session.window_id,
        had_match = session.active_match.is_some(),
        "cancelled snap session"
    );

    SnapSessionOutcome::Cancelled
}

/// Prime a session from the first observed drag frame so the overlay is
/// visible immediately instead of waiting for a second movement tick.
pub fn prime_snap_session(session: &mut SnapSession, now: Instant) {
    session.has_moved = true;
    session.last_movement_time = Some(now);
    session.phase = SnapSessionPhase::Tracking;
    session.active_match = best_snap_match(
        &session.last_window_bounds,
        &session.targets,
        MIN_OVERLAP_RATIO,
    );

    tracing::info!(
        target: "script_kit::snap_session",
        event = "snap_session_primed",
        session_id = session.session_id,
        window_id = session.window_id,
        x = session.last_window_bounds.x,
        y = session.last_window_bounds.y,
        w = session.last_window_bounds.width,
        h = session.last_window_bounds.height,
        matched = session.active_match.is_some(),
        matched_tile = session.active_match.map(|m| format!("{:?}", m.target.tile)),
        overlap = session.active_match.map(|m| m.overlap_ratio),
        "primed snap session from initial drag frame"
    );
}

/// Finish a settled session: commit if there's an active match, otherwise cancel.
pub fn finish_snap_session(session: &SnapSession) -> Result<SnapSessionOutcome> {
    if session.active_match.is_some() {
        commit_snap_session(session)
    } else {
        Ok(cancel_snap_session(session))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_display() -> Bounds {
        Bounds::new(0, 24, 1440, 876)
    }

    fn make_session() -> SnapSession {
        let display_val = make_display();
        let targets = build_snap_targets(&display_val);
        SnapSession {
            window_id: 0x1234_0000,
            app_name: "TestApp".to_string(),
            window_title: "Test Window".to_string(),
            display: display_val,
            targets,
            active_match: None,
            last_window_bounds: Bounds::new(200, 100, 800, 600),
            phase: SnapSessionPhase::WaitingForMovement,
            last_movement_time: None,
            has_moved: false,
            session_id: 0,
        }
    }

    #[test]
    fn tick_detects_movement_and_enters_tracking() {
        let mut session = make_session();
        let now = Instant::now();

        // First tick with same bounds — stays in WaitingForMovement.
        let phase = tick_snap_session(&mut session, Bounds::new(200, 100, 800, 600), now);
        assert_eq!(phase, SnapSessionPhase::WaitingForMovement);

        // Move the window — enters Tracking.
        let phase = tick_snap_session(&mut session, Bounds::new(0, 30, 800, 600), now);
        assert_eq!(phase, SnapSessionPhase::Tracking);
        assert!(session.has_moved);
    }

    #[test]
    fn tick_settles_after_inactivity() {
        let mut session = make_session();
        let t0 = Instant::now();

        // Move at t0.
        tick_snap_session(&mut session, Bounds::new(0, 30, 800, 600), t0);
        assert_eq!(session.phase, SnapSessionPhase::Tracking);

        // No movement, but not enough time — still Tracking.
        let t1 = t0 + Duration::from_millis(50);
        tick_snap_session(&mut session, Bounds::new(0, 30, 800, 600), t1);
        assert_eq!(session.phase, SnapSessionPhase::Tracking);

        // After settle duration — Settled.
        let t2 = t0 + SETTLE_DURATION + Duration::from_millis(1);
        tick_snap_session(&mut session, Bounds::new(0, 30, 800, 600), t2);
        assert_eq!(session.phase, SnapSessionPhase::Settled);
    }

    #[test]
    fn tick_finds_snap_match_for_left_half() {
        let mut session = make_session();
        let now = Instant::now();

        // Drag to left half of display.
        let left_bounds = Bounds::new(0, 24, 720, 876);
        tick_snap_session(&mut session, left_bounds, now);

        let active = session.active_match.expect("should match left half");
        assert_eq!(active.target.tile, TilePosition::LeftHalf);
    }

    #[test]
    fn tick_clears_match_when_window_moves_away() {
        let mut session = make_session();
        let now = Instant::now();

        // First: snap to left half.
        tick_snap_session(&mut session, Bounds::new(0, 24, 720, 876), now);
        assert!(session.active_match.is_some());

        // Move to a position that doesn't overlap enough.
        let t1 = now + Duration::from_millis(16);
        tick_snap_session(&mut session, Bounds::new(500, 300, 100, 100), t1);
        assert!(session.active_match.is_none());
    }

    #[test]
    fn build_overlay_model_marks_active_target() {
        let mut session = make_session();
        let now = Instant::now();

        // Snap to left half.
        tick_snap_session(&mut session, Bounds::new(0, 24, 720, 876), now);

        let model = build_overlay_model(&session);
        assert_eq!(model.display_bounds, session.display);

        let active_targets: Vec<_> = model.targets.iter().filter(|t| t.active).collect();
        assert_eq!(active_targets.len(), 1);
        assert_eq!(active_targets[0].tile, TilePosition::LeftHalf);

        let inactive_count = model.targets.iter().filter(|t| !t.active).count();
        assert_eq!(inactive_count, model.targets.len() - 1);
    }

    #[test]
    fn build_overlay_model_no_active_when_no_match() {
        let session = make_session();
        let model = build_overlay_model(&session);
        assert!(model.targets.iter().all(|t| !t.active));
    }

    #[test]
    fn finish_cancels_when_no_active_match() {
        let session = make_session();
        let outcome = finish_snap_session(&session).expect("cancel should not fail");
        assert!(matches!(outcome, SnapSessionOutcome::Cancelled));
    }

    #[test]
    fn prime_session_marks_tracking_and_computes_current_match() {
        let mut session = make_session();
        session.last_window_bounds = Bounds::new(0, 24, 720, 876);
        prime_snap_session(&mut session, Instant::now());

        assert_eq!(session.phase, SnapSessionPhase::Tracking);
        assert!(session.has_moved);
        assert!(session.last_movement_time.is_some());

        let active = session
            .active_match
            .expect("expected active match after priming");
        assert_eq!(active.target.tile, TilePosition::LeftHalf);
    }

    #[test]
    fn cancel_returns_cancelled_outcome() {
        let session = make_session();
        let outcome = cancel_snap_session(&session);
        assert!(matches!(outcome, SnapSessionOutcome::Cancelled));
    }

    #[test]
    fn session_phase_progression() {
        let mut session = make_session();
        let t0 = Instant::now();

        // Phase 1: WaitingForMovement.
        assert_eq!(session.phase, SnapSessionPhase::WaitingForMovement);

        // Phase 2: Tracking (after movement).
        tick_snap_session(&mut session, Bounds::new(10, 30, 800, 600), t0);
        assert_eq!(session.phase, SnapSessionPhase::Tracking);

        // Phase 3: Settled (after settle duration with no movement).
        let t1 = t0 + SETTLE_DURATION + Duration::from_millis(10);
        tick_snap_session(&mut session, Bounds::new(10, 30, 800, 600), t1);
        assert_eq!(session.phase, SnapSessionPhase::Settled);
    }

    #[test]
    fn movement_resets_settle_timer() {
        let mut session = make_session();
        let t0 = Instant::now();

        // Initial movement.
        tick_snap_session(&mut session, Bounds::new(10, 30, 800, 600), t0);

        // Almost settled...
        let t1 = t0 + SETTLE_DURATION - Duration::from_millis(10);
        tick_snap_session(&mut session, Bounds::new(10, 30, 800, 600), t1);
        assert_eq!(session.phase, SnapSessionPhase::Tracking);

        // New movement resets the timer.
        let t2 = t0 + SETTLE_DURATION;
        tick_snap_session(&mut session, Bounds::new(20, 30, 800, 600), t2);
        assert_eq!(session.phase, SnapSessionPhase::Tracking);

        // Not enough time since the second movement — still Tracking.
        let t3 = t2 + Duration::from_millis(50);
        tick_snap_session(&mut session, Bounds::new(20, 30, 800, 600), t3);
        assert_eq!(session.phase, SnapSessionPhase::Tracking);

        // Now enough time after second movement — Settled.
        let t4 = t2 + SETTLE_DURATION + Duration::from_millis(1);
        tick_snap_session(&mut session, Bounds::new(20, 30, 800, 600), t4);
        assert_eq!(session.phase, SnapSessionPhase::Settled);
    }

    #[test]
    fn right_half_snap_match() {
        let mut session = make_session();
        let now = Instant::now();

        // Drag to right half of display.
        let right_bounds = Bounds::new(720, 24, 720, 876);
        tick_snap_session(&mut session, right_bounds, now);

        let active = session.active_match.expect("should match right half");
        assert_eq!(active.target.tile, TilePosition::RightHalf);
    }

    #[test]
    fn quadrant_snap_match() {
        let mut session = make_session();
        let now = Instant::now();

        // Drag to top-left quadrant.
        let tl_bounds = Bounds::new(0, 24, 720, 438);
        tick_snap_session(&mut session, tl_bounds, now);

        let active = session.active_match.expect("should match a quadrant");
        assert_eq!(active.target.tile, TilePosition::TopLeft);
    }
}
