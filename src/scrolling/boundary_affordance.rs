//! Pure state and math for the launcher's boundary-scroll affordance.
//!
//! This module deliberately knows nothing about GPUI list state, selection,
//! rendering, or async executors. The owning surface supplies edge eligibility,
//! schedules the watchdog/rebound work requested by [`BoundaryDecision`], and
//! guards each async update with the generation helpers below.

use std::time::{Duration, Instant};

const RAW_PULL_CAP_MULTIPLIER: f32 = 3.0;
const INVERSE_RESISTANCE_RATIO_LIMIT: f32 = 0.999;
const SETTLE_DECAY: f32 = 6.0;
const SETTLE_ZERO_EPSILON_PX: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BoundaryEdge {
    Top,
    Bottom,
}

impl BoundaryEdge {
    #[inline]
    pub(crate) const fn sign(self) -> f32 {
        match self {
            Self::Top => 1.0,
            Self::Bottom => -1.0,
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum BoundaryPhase {
    #[default]
    Idle,
    Tracking,
    Settling,
}

impl BoundaryPhase {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Tracking => "tracking",
            Self::Settling => "settling",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreciseTouchPhase {
    Started,
    Moved,
    Ended,
}

impl PreciseTouchPhase {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Moved => "moved",
            Self::Ended => "ended",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettleReason {
    Ended,
    IdleTimeout,
    ReducedMotion,
    Reset,
}

impl SettleReason {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Ended => "ended",
            Self::IdleTimeout => "idleTimeout",
            Self::ReducedMotion => "reducedMotion",
            Self::Reset => "reset",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct BoundaryEligibility {
    pub top: bool,
    pub bottom: bool,
}

impl BoundaryEligibility {
    /// Resolve a new boundary capture from the event sign. When content fits
    /// and both edges are eligible, the delta direction disambiguates them.
    pub(crate) fn edge_for_delta(self, delta_y_px: f32) -> Option<BoundaryEdge> {
        if delta_y_px > 0.0 && self.top {
            Some(BoundaryEdge::Top)
        } else if delta_y_px < 0.0 && self.bottom {
            Some(BoundaryEdge::Bottom)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BoundaryAffordanceTuning {
    pub max_distance_px: f32,
    pub resistance_px: f32,
    pub settle_duration: Duration,
    pub idle_timeout: Duration,
    pub reduced_motion_max_distance_px: f32,
}

impl BoundaryAffordanceTuning {
    pub(crate) const fn new(
        max_distance_px: f32,
        resistance_px: f32,
        settle_duration: Duration,
        idle_timeout: Duration,
        reduced_motion_max_distance_px: f32,
    ) -> Self {
        Self {
            max_distance_px,
            resistance_px,
            settle_duration,
            idle_timeout,
            reduced_motion_max_distance_px,
        }
    }

    pub(crate) fn active_max_distance_px(self, reduced_motion: bool) -> f32 {
        let value = if reduced_motion {
            self.reduced_motion_max_distance_px
        } else {
            self.max_distance_px
        };
        finite_nonnegative(value)
    }

    fn normalized_resistance_px(self) -> f32 {
        if self.resistance_px.is_finite() {
            self.resistance_px.max(1.0)
        } else {
            1.0
        }
    }

    fn raw_pull_cap_px(self) -> f32 {
        self.normalized_resistance_px() * RAW_PULL_CAP_MULTIPLIER
    }
}

impl Default for BoundaryAffordanceTuning {
    fn default() -> Self {
        Self::new(
            18.0,
            36.0,
            Duration::from_millis(180),
            Duration::from_millis(160),
            4.0,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BoundaryDecision {
    /// Event travel not consumed by unwinding the rubber band. The caller may
    /// pass this through to the existing selection-owned wheel path.
    pub residual_delta_y_px: f32,
    /// Offset, edge, or phase changed and the surface should repaint/report.
    pub visual_changed: bool,
    /// The first tracking event armed a single idle watchdog for this generation.
    pub arm_idle_watchdog: bool,
    /// A non-reduced-motion rebound entered `Settling` and needs frame updates.
    pub start_settle: Option<SettleReason>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IdleWatchdogStatus {
    Cancelled,
    Sleep(Duration),
    TimedOut,
}

#[derive(Debug)]
pub(crate) struct BoundaryAffordanceState {
    pub offset_px: f32,
    raw_pull_px: f32,
    pub edge: Option<BoundaryEdge>,
    pub phase: BoundaryPhase,
    pub generation: u64,
    pub last_touch_phase: Option<PreciseTouchPhase>,
    pub last_settle_reason: Option<SettleReason>,
    pub reduced_motion: bool,
    last_event_at: Option<Instant>,
    idle_watchdog_armed: bool,
}

impl Default for BoundaryAffordanceState {
    fn default() -> Self {
        Self::new(false)
    }
}

impl BoundaryAffordanceState {
    pub(crate) const fn new(reduced_motion: bool) -> Self {
        Self {
            offset_px: 0.0,
            raw_pull_px: 0.0,
            edge: None,
            phase: BoundaryPhase::Idle,
            generation: 0,
            last_touch_phase: None,
            last_settle_reason: None,
            reduced_motion,
            last_event_at: None,
            idle_watchdog_armed: false,
        }
    }

    pub(crate) fn raw_pull_px(&self) -> f32 {
        self.raw_pull_px
    }

    pub(crate) fn last_event_at(&self) -> Option<Instant> {
        self.last_event_at
    }

    pub(crate) fn idle_watchdog_armed(&self) -> bool {
        self.idle_watchdog_armed
    }

    pub(crate) fn generation_is_current(&self, captured_generation: u64) -> bool {
        self.generation == captured_generation
    }

    pub(crate) fn tracking_generation_is_current(&self, captured_generation: u64) -> bool {
        self.generation_is_current(captured_generation) && self.phase == BoundaryPhase::Tracking
    }

    pub(crate) fn settling_generation_is_current(&self, captured_generation: u64) -> bool {
        self.generation_is_current(captured_generation) && self.phase == BoundaryPhase::Settling
    }

    /// Invalidate every task holding the previous generation.
    pub(crate) fn cancel_pending_work(&mut self) -> u64 {
        self.advance_generation()
    }

    /// Consume one precise pixel-scroll event. Positive Y pulls outward at the
    /// top; negative Y pulls outward at the bottom.
    pub(crate) fn handle_precise_scroll(
        &mut self,
        delta_y_px: f32,
        touch_phase: PreciseTouchPhase,
        eligibility: BoundaryEligibility,
        tuning: BoundaryAffordanceTuning,
        reduced_motion: bool,
        now: Instant,
    ) -> BoundaryDecision {
        let before = self.visual_signature();
        let delta_y_px = finite_or_zero(delta_y_px);
        let mut decision = BoundaryDecision {
            residual_delta_y_px: delta_y_px,
            ..BoundaryDecision::default()
        };
        let mut generation_prepared = false;

        self.reduced_motion = reduced_motion;
        self.last_touch_phase = Some(touch_phase);

        if self.phase == BoundaryPhase::Settling
            && matches!(
                touch_phase,
                PreciseTouchPhase::Started | PreciseTouchPhase::Moved
            )
        {
            self.grab_settling_offset(tuning);
            generation_prepared = true;
        } else if touch_phase == PreciseTouchPhase::Started {
            self.advance_generation();
            self.idle_watchdog_armed = false;
            generation_prepared = true;
        }

        if matches!(
            touch_phase,
            PreciseTouchPhase::Started | PreciseTouchPhase::Moved
        ) {
            self.last_event_at = Some(now);
        }

        if self.phase != BoundaryPhase::Settling {
            if self.edge.is_none() {
                if let Some(edge) = eligibility.edge_for_delta(delta_y_px) {
                    if !generation_prepared {
                        self.advance_generation();
                    }
                    self.edge = Some(edge);
                    self.phase = BoundaryPhase::Tracking;
                    self.raw_pull_px = 0.0;
                    self.offset_px = 0.0;
                }
            }

            if self.phase == BoundaryPhase::Tracking {
                decision.residual_delta_y_px = self.apply_tracking_delta(delta_y_px, tuning);

                if self.phase == BoundaryPhase::Tracking
                    && touch_phase != PreciseTouchPhase::Ended
                    && !self.idle_watchdog_armed
                {
                    self.idle_watchdog_armed = true;
                    decision.arm_idle_watchdog = true;
                }
            }
        }

        if touch_phase == PreciseTouchPhase::Ended && self.phase == BoundaryPhase::Tracking {
            let settle = self.begin_settle(SettleReason::Ended, tuning);
            decision.start_settle = settle.start_settle;
            decision.arm_idle_watchdog = false;
        }

        decision.visual_changed = before != self.visual_signature();
        decision
    }

    /// Enter a rebound, or snap to exact idle when reduced motion is enabled.
    pub(crate) fn begin_settle(
        &mut self,
        reason: SettleReason,
        _tuning: BoundaryAffordanceTuning,
    ) -> BoundaryDecision {
        let before = self.visual_signature();
        let mut decision = BoundaryDecision::default();

        if self.phase != BoundaryPhase::Tracking || self.offset_px.abs() < SETTLE_ZERO_EPSILON_PX {
            self.finish_at_zero(reason);
        } else if self.reduced_motion {
            self.advance_generation();
            self.finish_at_zero(SettleReason::ReducedMotion);
        } else {
            self.advance_generation();
            self.phase = BoundaryPhase::Settling;
            self.raw_pull_px = 0.0;
            self.last_settle_reason = Some(reason);
            self.last_event_at = None;
            self.idle_watchdog_armed = false;
            decision.start_settle = Some(reason);
        }

        decision.visual_changed = before != self.visual_signature();
        decision
    }

    /// Return the watchdog's next action without mutating state. `Cancelled`
    /// covers stale generations as well as gestures that already ended/reset.
    pub(crate) fn idle_watchdog_status(
        &self,
        captured_generation: u64,
        now: Instant,
        tuning: BoundaryAffordanceTuning,
    ) -> IdleWatchdogStatus {
        if !self.tracking_generation_is_current(captured_generation) || !self.idle_watchdog_armed {
            return IdleWatchdogStatus::Cancelled;
        }

        let elapsed = self
            .last_event_at
            .map(|last| now.saturating_duration_since(last))
            .unwrap_or(tuning.idle_timeout);
        if elapsed >= tuning.idle_timeout {
            IdleWatchdogStatus::TimedOut
        } else {
            IdleWatchdogStatus::Sleep(tuning.idle_timeout.saturating_sub(elapsed))
        }
    }

    /// Start the timeout rebound only if the watchdog still owns this gesture.
    pub(crate) fn begin_idle_timeout_settle(
        &mut self,
        captured_generation: u64,
        now: Instant,
        tuning: BoundaryAffordanceTuning,
    ) -> BoundaryDecision {
        if self.idle_watchdog_status(captured_generation, now, tuning)
            != IdleWatchdogStatus::TimedOut
        {
            return BoundaryDecision::default();
        }
        self.begin_settle(SettleReason::IdleTimeout, tuning)
    }

    /// Apply one normalized rebound sample. A stale task cannot mutate state.
    /// The accepted terminal sample always produces exact zero/idle state.
    pub(crate) fn apply_settle_sample(
        &mut self,
        captured_generation: u64,
        starting_offset_px: f32,
        normalized_elapsed: f32,
    ) -> bool {
        if !self.settling_generation_is_current(captured_generation) {
            return false;
        }

        let next = settled_offset(starting_offset_px, normalized_elapsed);
        if next == 0.0 {
            self.finish_at_zero_preserving_generation();
        } else {
            let Some(edge) = self.edge else {
                self.finish_at_zero_preserving_generation();
                return true;
            };
            self.offset_px = edge.sign() * next.abs();
        }
        true
    }

    pub(crate) fn finish_settle_if_current(&mut self, captured_generation: u64) -> bool {
        if !self.settling_generation_is_current(captured_generation) {
            return false;
        }
        self.finish_at_zero_preserving_generation();
        true
    }

    /// Central reset seam for view/filter/theme/selection/resize changes.
    pub(crate) fn reset(&mut self, reason: SettleReason) -> bool {
        let before = self.visual_signature();
        self.advance_generation();
        self.finish_at_zero(reason);
        before != self.visual_signature()
    }

    fn apply_tracking_delta(&mut self, delta_y_px: f32, tuning: BoundaryAffordanceTuning) -> f32 {
        let Some(edge) = self.edge else {
            return delta_y_px;
        };
        let outward_component = delta_y_px * edge.sign();

        if outward_component > 0.0 {
            self.raw_pull_px = (self.raw_pull_px + outward_component).min(tuning.raw_pull_cap_px());
            self.recompute_offset(edge, tuning);
            return 0.0;
        }

        if outward_component < 0.0 {
            let inward_px = -outward_component;
            let consumed = inward_px.min(self.raw_pull_px);
            self.raw_pull_px -= consumed;
            let residual_inward_px = inward_px - consumed;

            if self.raw_pull_px <= f32::EPSILON {
                self.finish_at_zero_preserving_generation();
            } else {
                self.recompute_offset(edge, tuning);
            }

            return -edge.sign() * residual_inward_px;
        }

        0.0
    }

    fn recompute_offset(&mut self, edge: BoundaryEdge, tuning: BoundaryAffordanceTuning) {
        let magnitude = resisted_offset(
            self.raw_pull_px,
            tuning.active_max_distance_px(self.reduced_motion),
            tuning.normalized_resistance_px(),
        );
        self.offset_px = edge.sign() * magnitude;
    }

    fn grab_settling_offset(&mut self, tuning: BoundaryAffordanceTuning) {
        self.advance_generation();
        self.idle_watchdog_armed = false;
        self.last_event_at = None;

        let Some(edge) = self.edge else {
            self.finish_at_zero_preserving_generation();
            return;
        };
        if self.offset_px.abs() < SETTLE_ZERO_EPSILON_PX {
            self.finish_at_zero_preserving_generation();
            return;
        }

        self.raw_pull_px = raw_pull_for_offset(
            self.offset_px,
            tuning.active_max_distance_px(self.reduced_motion),
            tuning.normalized_resistance_px(),
        )
        .min(tuning.raw_pull_cap_px());
        self.offset_px = edge.sign()
            * resisted_offset(
                self.raw_pull_px,
                tuning.active_max_distance_px(self.reduced_motion),
                tuning.normalized_resistance_px(),
            );
        self.phase = BoundaryPhase::Tracking;
    }

    fn finish_at_zero(&mut self, reason: SettleReason) {
        self.finish_at_zero_preserving_generation();
        self.last_settle_reason = Some(reason);
        self.last_event_at = None;
    }

    fn finish_at_zero_preserving_generation(&mut self) {
        self.offset_px = 0.0;
        self.raw_pull_px = 0.0;
        self.edge = None;
        self.phase = BoundaryPhase::Idle;
        self.last_event_at = None;
        self.idle_watchdog_armed = false;
    }

    fn advance_generation(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.generation
    }

    fn visual_signature(&self) -> (u32, Option<BoundaryEdge>, BoundaryPhase) {
        (self.offset_px.to_bits(), self.edge, self.phase)
    }
}

pub(crate) fn resisted_offset(raw_pull_px: f32, max_px: f32, resistance_px: f32) -> f32 {
    let raw_pull_px = finite_nonnegative(raw_pull_px);
    let max_px = finite_nonnegative(max_px);
    let resistance_px = if resistance_px.is_finite() {
        resistance_px.max(1.0)
    } else {
        1.0
    };
    max_px * (1.0 - (-raw_pull_px / resistance_px).exp())
}

pub(crate) fn raw_pull_for_offset(offset_px: f32, max_px: f32, resistance_px: f32) -> f32 {
    let max_px = finite_nonnegative(max_px);
    if max_px == 0.0 {
        return 0.0;
    }
    let resistance_px = if resistance_px.is_finite() {
        resistance_px.max(1.0)
    } else {
        1.0
    };
    let ratio =
        (finite_nonnegative(offset_px.abs()) / max_px).clamp(0.0, INVERSE_RESISTANCE_RATIO_LIMIT);
    -resistance_px * (1.0 - ratio).ln()
}

pub(crate) fn settle_factor(normalized_elapsed: f32) -> f32 {
    if !normalized_elapsed.is_finite() || normalized_elapsed >= 1.0 {
        return 0.0;
    }
    let u = normalized_elapsed.max(0.0);
    (1.0 + SETTLE_DECAY * u) * (-SETTLE_DECAY * u).exp()
}

pub(crate) fn settled_offset(starting_offset_px: f32, normalized_elapsed: f32) -> f32 {
    let offset = finite_or_zero(starting_offset_px) * settle_factor(normalized_elapsed);
    if normalized_elapsed >= 1.0 || offset.abs() < SETTLE_ZERO_EPSILON_PX {
        0.0
    } else {
        offset
    }
}

pub(crate) fn normalized_settle_elapsed(elapsed: Duration, duration: Duration) -> f32 {
    if duration.is_zero() {
        return 1.0;
    }
    (elapsed.as_secs_f64() / duration.as_secs_f64()).min(1.0) as f32
}

fn finite_nonnegative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn top_only() -> BoundaryEligibility {
        BoundaryEligibility {
            top: true,
            bottom: false,
        }
    }

    fn bottom_only() -> BoundaryEligibility {
        BoundaryEligibility {
            top: false,
            bottom: true,
        }
    }

    fn moved(
        state: &mut BoundaryAffordanceState,
        delta_y_px: f32,
        eligibility: BoundaryEligibility,
        now: Instant,
    ) -> BoundaryDecision {
        state.handle_precise_scroll(
            delta_y_px,
            PreciseTouchPhase::Moved,
            eligibility,
            BoundaryAffordanceTuning::default(),
            false,
            now,
        )
    }

    #[test]
    fn resistance_is_monotonic_and_bounded() {
        let mut previous = 0.0;
        for raw in [0.0, 1.0, 12.0, 36.0, 72.0, 108.0, 10_000.0] {
            let offset = resisted_offset(raw, 18.0, 36.0);
            assert!(offset >= previous);
            assert!(offset < 18.0 || (18.0 - offset).abs() <= f32::EPSILON);
            previous = offset;
        }
    }

    #[test]
    fn edge_signs_and_direction_disambiguate_dual_edge_content() {
        assert_eq!(BoundaryEdge::Top.sign(), 1.0);
        assert_eq!(BoundaryEdge::Bottom.sign(), -1.0);

        let both = BoundaryEligibility {
            top: true,
            bottom: true,
        };
        assert_eq!(both.edge_for_delta(1.0), Some(BoundaryEdge::Top));
        assert_eq!(both.edge_for_delta(-1.0), Some(BoundaryEdge::Bottom));
        assert_eq!(both.edge_for_delta(0.0), None);
    }

    #[test]
    fn partial_reversal_reduces_offset_without_residual_navigation() {
        let now = Instant::now();
        let mut state = BoundaryAffordanceState::default();
        moved(&mut state, 36.0, top_only(), now);
        let before = state.offset_px;

        let decision = moved(&mut state, -12.0, top_only(), now);

        assert_eq!(decision.residual_delta_y_px, 0.0);
        assert!(state.offset_px > 0.0);
        assert!(state.offset_px < before);
        assert_eq!(state.phase, BoundaryPhase::Tracking);
    }

    #[test]
    fn full_reversal_returns_residual_with_original_inward_sign() {
        let now = Instant::now();
        let mut top = BoundaryAffordanceState::default();
        moved(&mut top, 20.0, top_only(), now);
        let top_decision = moved(&mut top, -32.0, top_only(), now);
        assert_eq!(top_decision.residual_delta_y_px, -12.0);
        assert_idle_exact(&top);

        let mut bottom = BoundaryAffordanceState::default();
        moved(&mut bottom, -20.0, bottom_only(), now);
        let bottom_decision = moved(&mut bottom, 32.0, bottom_only(), now);
        assert_eq!(bottom_decision.residual_delta_y_px, 12.0);
        assert_idle_exact(&bottom);
    }

    #[test]
    fn raw_pull_is_bounded_before_it_can_create_reversal_debt() {
        let now = Instant::now();
        let mut state = BoundaryAffordanceState::default();
        moved(&mut state, 100_000.0, top_only(), now);

        assert_eq!(state.raw_pull_px(), 108.0);
        assert!(state.offset_px < 18.0);
    }

    #[test]
    fn a_new_gesture_grabs_a_settling_offset_without_snapping() {
        let now = Instant::now();
        let tuning = BoundaryAffordanceTuning::default();
        let mut state = BoundaryAffordanceState::default();
        moved(&mut state, 36.0, top_only(), now);
        let ended = state.handle_precise_scroll(
            0.0,
            PreciseTouchPhase::Ended,
            top_only(),
            tuning,
            false,
            now,
        );
        assert_eq!(ended.start_settle, Some(SettleReason::Ended));
        let settling_offset = state.offset_px;
        let settling_generation = state.generation;

        let grabbed = state.handle_precise_scroll(
            0.0,
            PreciseTouchPhase::Started,
            top_only(),
            tuning,
            false,
            now,
        );

        assert!(grabbed.arm_idle_watchdog);
        assert_eq!(state.phase, BoundaryPhase::Tracking);
        assert!(state.generation != settling_generation);
        assert!((state.offset_px - settling_offset).abs() < 0.001);
        assert!((state.raw_pull_px() - 36.0).abs() < 0.001);
    }

    #[test]
    fn stale_settle_generation_cannot_mutate_state() {
        let now = Instant::now();
        let tuning = BoundaryAffordanceTuning::default();
        let mut state = BoundaryAffordanceState::default();
        moved(&mut state, 36.0, top_only(), now);
        state.begin_settle(SettleReason::Ended, tuning);
        let stale_generation = state.generation;
        let starting_offset = state.offset_px;
        state.cancel_pending_work();

        assert!(!state.apply_settle_sample(stale_generation, starting_offset, 0.5));
        assert_eq!(state.offset_px, starting_offset);
    }

    #[test]
    fn critically_damped_settle_decreases_and_finishes_at_exact_zero() {
        let samples = [0.0, 0.15, 0.35, 0.6, 0.9];
        let mut previous = settled_offset(18.0, samples[0]);
        for u in samples.into_iter().skip(1) {
            let next = settled_offset(18.0, u);
            assert!(next < previous, "u={u} next={next} previous={previous}");
            previous = next;
        }
        assert_eq!(settled_offset(18.0, 1.0), 0.0);

        let now = Instant::now();
        let tuning = BoundaryAffordanceTuning::default();
        let mut state = BoundaryAffordanceState::default();
        moved(&mut state, 36.0, top_only(), now);
        state.begin_settle(SettleReason::Ended, tuning);
        let generation = state.generation;
        let starting_offset = state.offset_px;
        assert!(state.apply_settle_sample(generation, starting_offset, 1.0));
        assert_idle_exact(&state);
    }

    #[test]
    fn reduced_motion_uses_small_cap_and_ends_without_autonomous_settle() {
        let now = Instant::now();
        let tuning = BoundaryAffordanceTuning::default();
        let mut state = BoundaryAffordanceState::new(true);
        state.handle_precise_scroll(
            100_000.0,
            PreciseTouchPhase::Moved,
            top_only(),
            tuning,
            true,
            now,
        );
        assert!(state.offset_px > 0.0);
        assert!(state.offset_px <= 4.0);

        let ended = state.handle_precise_scroll(
            0.0,
            PreciseTouchPhase::Ended,
            top_only(),
            tuning,
            true,
            now,
        );
        assert_eq!(ended.start_settle, None);
        assert_eq!(state.last_settle_reason, Some(SettleReason::ReducedMotion));
        assert_idle_exact(&state);
    }

    #[test]
    fn watchdog_reports_remaining_time_and_times_out_once() {
        let start = Instant::now();
        let tuning = BoundaryAffordanceTuning::default();
        let mut state = BoundaryAffordanceState::default();
        let decision = moved(&mut state, 12.0, top_only(), start);
        assert!(decision.arm_idle_watchdog);
        let generation = state.generation;

        assert_eq!(
            state.idle_watchdog_status(generation, start + Duration::from_millis(60), tuning,),
            IdleWatchdogStatus::Sleep(Duration::from_millis(100))
        );
        let settle =
            state.begin_idle_timeout_settle(generation, start + Duration::from_millis(160), tuning);
        assert_eq!(settle.start_settle, Some(SettleReason::IdleTimeout));
        assert_eq!(state.phase, BoundaryPhase::Settling);
        assert_eq!(
            state.idle_watchdog_status(generation, start + Duration::from_secs(1), tuning),
            IdleWatchdogStatus::Cancelled
        );
    }

    #[test]
    fn inverse_mapping_round_trips_resisted_offsets() {
        for raw in [0.0, 4.0, 18.0, 36.0, 72.0, 108.0] {
            let offset = resisted_offset(raw, 18.0, 36.0);
            let reconstructed = raw_pull_for_offset(offset, 18.0, 36.0);
            assert!((reconstructed - raw).abs() < 0.001);
        }
    }

    #[test]
    fn normalized_elapsed_handles_zero_duration() {
        assert_eq!(
            normalized_settle_elapsed(Duration::from_millis(1), Duration::ZERO),
            1.0
        );
        assert_eq!(
            normalized_settle_elapsed(Duration::from_millis(90), Duration::from_millis(180)),
            0.5
        );
    }

    fn assert_idle_exact(state: &BoundaryAffordanceState) {
        assert_eq!(state.offset_px, 0.0);
        assert_eq!(state.raw_pull_px(), 0.0);
        assert_eq!(state.edge, None);
        assert_eq!(state.phase, BoundaryPhase::Idle);
        assert!(!state.idle_watchdog_armed());
    }
}
