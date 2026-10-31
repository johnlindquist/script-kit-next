//! Physical main-shortcut grammar. The first down records ownership, but never takes focus:
//! until the first key-up it may still become dictation into the original field.
use std::time::{Duration, Instant};

pub const HOLD_MS: u64 = 250;
pub const DOUBLE_MS: u64 = 300;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GestureConfig {
    pub hold_ms: u64,
    pub double_ms: u64,
}
impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            hold_ms: HOLD_MS,
            double_ms: DOUBLE_MS,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GestureInput {
    KeyDown(Instant),
    KeyUp(Instant),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GestureEvent {
    BeginOutside,
    BeginInside,
    TapPreview,
    Tap,
    DoubleTap,
    HoldStart,
    TodayHoldStart,
    HoldEnd,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Idle,
    Pressed { since: Instant, second: bool },
    Between(Instant),
    Holding,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GestureClassifier {
    config: GestureConfig,
    shown: bool,
    phase: Phase,
}
impl GestureClassifier {
    pub fn new(config: GestureConfig) -> Self {
        Self {
            config,
            shown: false,
            phase: Phase::Idle,
        }
    }
    pub fn with_defaults() -> Self {
        Self::new(GestureConfig::default())
    }
    pub fn config(&self) -> GestureConfig {
        self.config
    }
    pub fn is_open(&self) -> bool {
        self.shown
    }
    pub fn is_closed(&self) -> bool {
        !self.shown
    }
    pub fn sync_window_shown(&mut self) {
        self.shown = true;
    }
    // A background show/hide must not swallow the release that stops a live microphone.
    pub fn sync_window_hidden(&mut self) {
        self.shown = false;
    }
    pub fn rearm_open_as_closed(&mut self) -> bool {
        if self.phase == Phase::Idle && self.shown {
            self.shown = false;
            true
        } else {
            false
        }
    }
    pub fn commit_typing(&mut self) {
        if matches!(self.phase, Phase::Between(_)) {
            self.phase = Phase::Idle;
        }
    }
    pub fn next_deadline(&self) -> Option<Instant> {
        match self.phase {
            Phase::Pressed { since, .. } => {
                Some(since + Duration::from_millis(self.config.hold_ms))
            }
            Phase::Between(at) => Some(at + Duration::from_millis(self.config.double_ms)),
            _ => None,
        }
    }
    pub fn poll(&mut self, now: Instant) -> Option<GestureEvent> {
        if now < self.next_deadline()? {
            return None;
        }
        match self.phase {
            Phase::Between(_) => {
                self.phase = Phase::Idle;
                Some(GestureEvent::Tap)
            }
            Phase::Pressed { second, .. } => {
                self.phase = Phase::Holding;
                Some(if second {
                    GestureEvent::TodayHoldStart
                } else {
                    GestureEvent::HoldStart
                })
            }
            _ => None,
        }
    }
    pub fn handle_input(&mut self, input: GestureInput) -> Vec<GestureEvent> {
        let at = match input {
            GestureInput::KeyDown(t) | GestureInput::KeyUp(t) => t,
        };
        let mut out = Vec::new();
        if self
            .next_deadline()
            .is_some_and(|d| d < at || (d == at && matches!(self.phase, Phase::Pressed { .. })))
        {
            out.extend(self.poll(at));
        }
        match (self.phase, input) {
            (Phase::Idle, GestureInput::KeyDown(_)) => {
                self.phase = Phase::Pressed {
                    since: at,
                    second: false,
                };
                out.push(if self.shown {
                    GestureEvent::BeginInside
                } else {
                    GestureEvent::BeginOutside
                });
            }
            (Phase::Between(_), GestureInput::KeyDown(_)) => {
                self.phase = Phase::Pressed {
                    since: at,
                    second: true,
                };
                out.push(GestureEvent::DoubleTap);
            }
            (Phase::Pressed { second: false, .. }, GestureInput::KeyUp(_)) => {
                self.phase = Phase::Between(at);
                out.push(GestureEvent::TapPreview);
            }
            (Phase::Pressed { second: true, .. }, GestureInput::KeyUp(_)) => {
                self.phase = Phase::Idle
            }
            (Phase::Holding, GestureInput::KeyUp(_)) => {
                self.phase = Phase::Idle;
                out.push(GestureEvent::HoldEnd);
            }
            _ => {}
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use GestureEvent::*;
    fn key(c: &mut GestureClassifier, t: Instant, ms: u64, down: bool) -> Vec<GestureEvent> {
        let at = t + Duration::from_millis(ms);
        c.handle_input(if down {
            GestureInput::KeyDown(at)
        } else {
            GestureInput::KeyUp(at)
        })
    }
    #[test]
    fn four_gestures_and_boundaries() {
        for (second, hold, start) in [
            (false, false, BeginOutside),
            (false, true, BeginOutside),
            (true, false, DoubleTap),
            (true, true, DoubleTap),
        ] {
            let mut c = GestureClassifier::with_defaults();
            let t = Instant::now();
            if second {
                assert_eq!(key(&mut c, t, 0, true), [BeginOutside]);
                assert_eq!(key(&mut c, t, 40, false), [TapPreview]);
                c.sync_window_shown();
            }
            assert_eq!(key(&mut c, t, 340, true), [start]);
            assert!(key(&mut c, t, 400, true).is_empty());
            let got = key(&mut c, t, if hold { 590 } else { 589 }, false);
            assert_eq!(
                got,
                match (second, hold) {
                    (false, false) => vec![TapPreview],
                    (true, false) => vec![],
                    (false, true) => vec![HoldStart, HoldEnd],
                    (true, true) => vec![TodayHoldStart, HoldEnd],
                }
            );
            if hold || second {
                assert_eq!(c.next_deadline(), None);
            } else {
                assert_eq!(c.poll(t + Duration::from_secs(2)), Some(Tap));
            }
        }
    }
    #[test]
    fn background_today_and_visibility_changes_keep_the_physical_release() {
        let mut c = GestureClassifier::with_defaults();
        let t = Instant::now();
        c.sync_window_shown();
        assert!(c.rearm_open_as_closed());
        assert_eq!(key(&mut c, t, 0, true), [BeginOutside]);
        assert_eq!(c.poll(t + Duration::from_millis(250)), Some(HoldStart));
        c.sync_window_hidden();
        assert_eq!(key(&mut c, t, 400, false), [HoldEnd]);
        assert!(key(&mut c, t, 401, false).is_empty());
    }
    #[test]
    fn typing_commits_first_tap_and_late_second_press_is_fresh() {
        let t = Instant::now();
        for typed in [false, true] {
            let mut c = GestureClassifier::with_defaults();
            key(&mut c, t, 0, true);
            key(&mut c, t, 10, false);
            c.sync_window_shown();
            if typed {
                c.commit_typing();
                assert_eq!(key(&mut c, t, 15, true), [BeginInside]);
            } else {
                assert_eq!(key(&mut c, t, 311, true), [Tap, BeginInside]);
            }
        }
    }
}
