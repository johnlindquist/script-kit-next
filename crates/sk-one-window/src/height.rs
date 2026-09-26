//! The window's height.
//!
//! The window grows at once when there is more to show (an animated grow would draw new rows
//! under the floating buttons until the edge caught up). It holds its height while you type and
//! shrinks, animated, once typing pauses for [`SHRINK_HOLD_MS`]. A view that just opened keeps the
//! height for [`VIEW_HOLD_MS`] so a view that is still loading never shrinks and grows back. A key
//! pressed during a shrink stops it where it is.

use serde::{Deserialize, Serialize};

use crate::Ms;
use crate::contract::{SHRINK_ANIM_MS, SHRINK_HOLD_MS, VIEW_HOLD_MS, shrink_ease};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HeightMode {
    /// Set at once.
    Grow,
    /// Pinned at the current height until `until`.
    Hold,
    /// Animated down over [`SHRINK_ANIM_MS`].
    Shrink,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeightDecision {
    pub mode: HeightMode,
    /// Where the window ends up (for a hold, where it stays).
    pub px: f32,
    /// Where a shrink starts.
    pub from: f32,
    /// A hold's end.
    pub until: Option<Ms>,
    /// A shrink's duration.
    pub anim_ms: u64,
}

/// Why the content's natural height changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentCause {
    /// The list changed (typing, results arriving).
    List,
    /// A view just opened in place of the list.
    ViewOpened,
    /// Back on the list after a view: the list the view covered is gone.
    Home,
    /// Something else changed size (the strip at the bottom, a view's own content).
    Other,
}

#[derive(Clone, Debug, Default)]
pub struct HeightGovernor {
    /// Settled or held height, or a shrink's target.
    target: f32,
    /// A shrink's start height and time.
    anim: Option<(f32, Ms)>,
    hold_until: Option<Ms>,
    typed_at: Option<Ms>,
    natural: f32,
    started: bool,
}

impl HeightGovernor {
    pub fn new() -> Self {
        Self::default()
    }

    /// The height on screen at `now`.
    pub fn displayed(&self, now: Ms) -> f32 {
        match self.anim {
            Some((from, start)) => {
                let t = now.saturating_sub(start) as f32 / SHRINK_ANIM_MS as f32;
                if t >= 1.0 {
                    self.target
                } else {
                    from + (self.target - from) * shrink_ease(t)
                }
            }
            None => self.target,
        }
    }

    pub fn natural(&self) -> f32 {
        self.natural
    }

    fn flying(&self, now: Ms) -> bool {
        matches!(self.anim, Some((_, start)) if now < start + SHRINK_ANIM_MS)
    }

    fn typing(&self, now: Ms) -> bool {
        self.typed_at
            .is_some_and(|at| now.saturating_sub(at) < SHRINK_HOLD_MS)
    }

    /// The content now wants `natural` px.
    pub fn content(
        &mut self,
        now: Ms,
        natural: f32,
        cause: ContentCause,
    ) -> Option<HeightDecision> {
        self.natural = natural;
        if !self.started {
            self.started = true;
            self.target = natural;
            return Some(HeightDecision {
                mode: HeightMode::Grow,
                px: natural,
                from: natural,
                until: None,
                anim_ms: 0,
            });
        }
        let typing_left = self
            .typed_at
            .filter(|_| self.typing(now))
            .map(|at| at + SHRINK_HOLD_MS - now)
            .unwrap_or(0);
        let hold = match cause {
            ContentCause::ViewOpened => VIEW_HOLD_MS,
            ContentCause::Home => 0,
            ContentCause::List | ContentCause::Other => typing_left,
        };
        self.resize(now, hold)
    }

    /// A printable key (typing, not Enter/Esc/arrows/shortcuts).
    pub fn typed(&mut self, now: Ms) -> Option<HeightDecision> {
        self.typed_at = Some(now);
        if self.flying(now) {
            return self.resize(now, SHRINK_HOLD_MS);
        }
        if let Some(until) = self.hold_until {
            if until > now {
                self.hold_until = Some(until.max(now + SHRINK_HOLD_MS));
            }
        }
        None
    }

    /// Time passed: a hold may have run out, a shrink may have landed.
    pub fn tick(&mut self, now: Ms) -> Option<HeightDecision> {
        if let Some(until) = self.hold_until {
            if now >= until {
                self.hold_until = None;
                return self.resize(now, 0);
            }
        }
        if matches!(self.anim, Some((_, start)) if now >= start + SHRINK_ANIM_MS) {
            self.anim = None;
        }
        None
    }

    pub fn next_wake(&self) -> Option<Ms> {
        let anim_end = self.anim.map(|(_, start)| start + SHRINK_ANIM_MS);
        match (self.hold_until, anim_end) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }

    fn resize(&mut self, now: Ms, hold: u64) -> Option<HeightDecision> {
        let from = self.displayed(now);
        let to = self.natural;
        let flying = self.flying(now);
        if (from - to).abs() < 2.0 && !flying {
            self.anim = None;
            self.target = from;
            return None;
        }
        if to >= from {
            self.anim = None;
            self.hold_until = None;
            self.target = to;
            return Some(HeightDecision {
                mode: HeightMode::Grow,
                px: to,
                from,
                until: None,
                anim_ms: 0,
            });
        }
        let held_left = self
            .hold_until
            .map(|until| until.saturating_sub(now))
            .unwrap_or(0);
        let wait = hold.max(held_left);
        if wait > 0 {
            self.anim = None;
            self.target = from;
            self.hold_until = Some(now + wait);
            return Some(HeightDecision {
                mode: HeightMode::Hold,
                px: from,
                from,
                until: Some(now + wait),
                anim_ms: 0,
            });
        }
        self.anim = Some((from, now));
        self.target = to;
        Some(HeightDecision {
            mode: HeightMode::Shrink,
            px: to,
            from,
            until: None,
            anim_ms: SHRINK_ANIM_MS,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gov(at: f32) -> HeightGovernor {
        let mut g = HeightGovernor::new();
        g.content(0, at, ContentCause::List);
        g
    }

    #[test]
    fn grows_at_once() {
        let mut g = gov(300.0);
        let d = g.content(10, 420.0, ContentCause::List).unwrap();
        assert_eq!(d.mode, HeightMode::Grow);
        assert_eq!(g.displayed(10), 420.0);
    }

    #[test]
    fn holds_while_typing_then_shrinks_after_the_pause() {
        let mut g = gov(420.0);
        g.typed(100);
        let d = g.content(100, 300.0, ContentCause::List).unwrap();
        assert_eq!(d.mode, HeightMode::Hold);
        assert_eq!(d.until, Some(400));
        g.typed(250);
        assert_eq!(g.next_wake(), Some(550), "each key extends the hold");
        assert!(g.tick(400).is_none());
        let d = g.tick(550).unwrap();
        assert_eq!(d.mode, HeightMode::Shrink);
        assert_eq!(d.px, 300.0);
        assert_eq!(g.displayed(550 + SHRINK_ANIM_MS), 300.0);
    }

    #[test]
    fn shrinks_at_once_when_not_typing() {
        let mut g = gov(420.0);
        let d = g.content(1_000, 300.0, ContentCause::List).unwrap();
        assert_eq!(d.mode, HeightMode::Shrink);
    }

    #[test]
    fn a_key_stops_a_shrink_where_it_is() {
        let mut g = gov(420.0);
        g.content(1_000, 300.0, ContentCause::List);
        let mid = g.displayed(1_100);
        assert!(mid < 420.0 && mid > 300.0);
        let d = g.typed(1_100).unwrap();
        assert_eq!(d.mode, HeightMode::Hold);
        assert!((d.px - mid).abs() < 0.01);
        assert_eq!(g.displayed(1_200), d.px);
    }

    #[test]
    fn a_view_keeps_the_height_then_shrinks_once() {
        let mut g = gov(420.0);
        let d = g.content(2_000, 200.0, ContentCause::ViewOpened).unwrap();
        assert_eq!(d.mode, HeightMode::Hold);
        assert_eq!(d.until, Some(2_000 + VIEW_HOLD_MS));
        let d = g.content(2_100, 260.0, ContentCause::Other).unwrap();
        assert_eq!(d.mode, HeightMode::Hold, "a loading view still holds");
        let d = g.tick(2_300).unwrap();
        assert_eq!(d.mode, HeightMode::Shrink);
        assert_eq!(d.px, 260.0);
    }

    #[test]
    fn a_view_that_grows_while_held_grows_at_once() {
        let mut g = gov(420.0);
        g.content(2_000, 200.0, ContentCause::ViewOpened);
        let d = g.content(2_100, 500.0, ContentCause::Other).unwrap();
        assert_eq!(d.mode, HeightMode::Grow);
        assert!(g.tick(2_300).is_none(), "the hold is gone after a grow");
    }

    #[test]
    fn going_home_does_not_hold() {
        let mut g = gov(420.0);
        let d = g.content(5_000, 200.0, ContentCause::Home).unwrap();
        assert_eq!(d.mode, HeightMode::Shrink);
    }
}
