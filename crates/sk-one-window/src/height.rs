//! The window's height.
//!
//! The window grows at once when there is more to show and shrinks at once, never animated, and
//! only when the list has settled: typing has paused for [`SHRINK_HOLD_MS`] and Jev is no longer
//! reading the current words (or has taken [`READING_WAIT_MS`] since the last key). A list that
//! would lose less than [`MIN_SHRINK_PX`] leaves the space under it instead, so rows arriving and
//! going one at a time never move the edge. A view that just opened keeps the height for
//! [`VIEW_HOLD_MS`] so a view that is still loading never shrinks and grows back.

use serde::{Deserialize, Serialize};

use crate::Ms;
use crate::contract::{
    MEASURE_WAIT_MS, MIN_SHRINK_PX, READING_WAIT_MS, SHRINK_HOLD_MS, VIEW_HOLD_MS,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HeightMode {
    /// Set taller at once.
    Grow,
    /// Pinned at the current height until `until`.
    Hold,
    /// Set shorter at once.
    Shrink,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeightDecision {
    pub mode: HeightMode,
    /// Where the window ends up (for a hold, where it stays).
    pub px: f32,
    /// Where it was.
    pub from: f32,
    /// A hold's end.
    pub until: Option<Ms>,
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
    /// The height on screen, or where a hold keeps it.
    target: f32,
    /// While a shrink waits: when the pause after a key or a view opening ends. Jev's reading can
    /// keep it waiting longer ([`Self::wake`]).
    hold_until: Option<Ms>,
    typed_at: Option<Ms>,
    /// When it last grew.
    grew_at: Option<Ms>,
    natural: f32,
    started: bool,
    /// The list, not a view or a new surface, is what changed size: a small drop leaves space.
    loose: bool,
    /// Since when Jev has been reading the current words, while it is.
    reading_since: Option<Ms>,
}

impl HeightGovernor {
    pub fn new() -> Self {
        Self::default()
    }

    /// The height on screen.
    pub fn displayed(&self) -> f32 {
        self.target
    }

    pub fn natural(&self) -> f32 {
        self.natural
    }

    /// When the last typing key arrived.
    pub fn typed_at(&self) -> Option<Ms> {
        self.typed_at
    }

    fn typing(&self, now: Ms) -> bool {
        self.typed_at
            .is_some_and(|at| now.saturating_sub(at) < SHRINK_HOLD_MS)
    }

    /// Until when a shrink waits for Jev's reading of the current words.
    fn reading_until(&self) -> Option<Ms> {
        self.reading_since
            .map(|since| since.max(self.typed_at.unwrap_or(0)) + READING_WAIT_MS)
    }

    /// When a waiting shrink may go.
    fn wake(&self) -> Option<Ms> {
        self.hold_until
            .map(|until| until.max(self.reading_until().unwrap_or(0)))
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
            });
        }
        self.loose = match cause {
            ContentCause::List => true,
            ContentCause::ViewOpened | ContentCause::Home => false,
            ContentCause::Other => self.loose,
        };
        let typing_left = self
            .typed_at
            .filter(|_| self.typing(now))
            .map(|at| at + SHRINK_HOLD_MS - now)
            .unwrap_or(0);
        let hold = match cause {
            ContentCause::ViewOpened => VIEW_HOLD_MS,
            ContentCause::Home => 0,
            ContentCause::List => return self.govern(now),
            ContentCause::Other => typing_left,
        };
        self.resize(now, hold)
    }

    /// Whether Jev is still reading the current words, told after each event. The event that
    /// brings the reading still counts it, so its redraw holds; the measure that follows, which
    /// carries the reading's rows, decides the shrink (within [`MEASURE_WAIT_MS`]).
    pub fn reading(&mut self, now: Ms, reading: bool) {
        match (reading, self.reading_since) {
            (true, None) => self.reading_since = Some(now),
            (false, Some(_)) => {
                self.reading_since = None;
                self.hold_until = self.hold_until.map(|u| u.max(now + MEASURE_WAIT_MS));
            }
            _ => {}
        }
    }

    /// The list drew again at the same height: an arrow, Enter, Jev's rows arriving or
    /// settling. While the window is held above the list, a redraw soon after a key starts the
    /// pause over and a redraw after the pause lets it shrink now, so it never shrinks within
    /// [`SHRINK_HOLD_MS`] of the rows changing under a typing hand.
    pub fn redrawn(&mut self, now: Ms) -> Option<HeightDecision> {
        self.hold_until?;
        self.govern(now)
    }

    fn govern(&mut self, now: Ms) -> Option<HeightDecision> {
        if self.typing(now) {
            self.resize(now, SHRINK_HOLD_MS)
        } else {
            self.hold_until = None;
            self.resize(now, 0)
        }
    }

    /// A printable key (typing, not Enter/Esc/arrows/shortcuts).
    pub fn typed(&mut self, now: Ms) {
        self.typed_at = Some(now);
        if let Some(until) = self.hold_until {
            self.hold_until = Some(until.max(now + SHRINK_HOLD_MS));
        }
    }

    /// Time passed: a hold may have run out.
    pub fn tick(&mut self, now: Ms) -> Option<HeightDecision> {
        match self.wake() {
            Some(at) if now >= at => {
                self.hold_until = None;
                self.resize(now, 0)
            }
            _ => None,
        }
    }

    pub fn next_wake(&self) -> Option<Ms> {
        self.wake()
    }

    fn resize(&mut self, now: Ms, hold: u64) -> Option<HeightDecision> {
        let from = self.target;
        let to = self.natural;
        let held = self.hold_until.is_some();
        if (from - to).abs() < 2.0 && !held {
            return None;
        }
        if to >= from {
            self.hold_until = None;
            self.target = to;
            self.grew_at = Some(now);
            return Some(HeightDecision {
                mode: HeightMode::Grow,
                px: to,
                from,
                until: None,
            });
        }
        if self.loose && from - to < MIN_SHRINK_PX && self.grew_at != Some(now) {
            self.hold_until = None;
            return None;
        }
        let held_left = self
            .hold_until
            .map(|until| until.saturating_sub(now))
            .unwrap_or(0);
        let until = now + hold.max(held_left);
        let wake = until.max(self.reading_until().unwrap_or(0));
        if wake > now {
            self.hold_until = Some(until);
            return Some(HeightDecision {
                mode: HeightMode::Hold,
                px: from,
                from,
                until: Some(wake),
            });
        }
        self.hold_until = None;
        self.target = to;
        let mode = if self.grew_at == Some(now) {
            // Nothing has painted the taller height yet; it is one change, not a grow and a shrink.
            HeightMode::Grow
        } else {
            HeightMode::Shrink
        };
        Some(HeightDecision {
            mode,
            px: to,
            from,
            until: None,
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
        assert_eq!(g.displayed(), 420.0);
    }

    #[test]
    fn holds_while_typing_then_shrinks_at_once_after_the_pause() {
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
        assert_eq!(g.displayed(), 300.0, "no animation: it is there at once");
        assert_eq!(g.next_wake(), None);
    }

    #[test]
    fn shrinks_at_once_when_not_typing() {
        let mut g = gov(420.0);
        let d = g.content(1_000, 300.0, ContentCause::List).unwrap();
        assert_eq!(d.mode, HeightMode::Shrink);
        assert_eq!(g.displayed(), 300.0);
    }

    #[test]
    fn a_list_less_than_two_rows_shorter_leaves_the_space() {
        let mut g = gov(480.0);
        assert!(g.content(1_000, 472.0, ContentCause::List).is_none());
        assert!(
            g.content(2_000, 480.0 - MIN_SHRINK_PX + 1.0, ContentCause::List)
                .is_none()
        );
        assert_eq!(g.displayed(), 480.0);
        let d = g
            .content(3_000, 480.0 - MIN_SHRINK_PX, ContentCause::List)
            .unwrap();
        assert_eq!(d.mode, HeightMode::Shrink);
        assert_eq!(d.px, 480.0 - MIN_SHRINK_PX);
    }

    #[test]
    fn a_small_drop_ends_a_hold_without_moving() {
        let mut g = gov(420.0);
        g.typed(100);
        g.content(100, 300.0, ContentCause::List);
        assert!(g.content(150, 400.0, ContentCause::List).is_none());
        assert_eq!(g.next_wake(), None);
        assert!(g.tick(1_000).is_none());
        assert_eq!(g.displayed(), 420.0);
    }

    #[test]
    fn a_view_or_home_settles_exactly() {
        let mut g = gov(420.0);
        let d = g.content(1_000, 400.0, ContentCause::Home).unwrap();
        assert_eq!(d.mode, HeightMode::Shrink);
        assert_eq!(d.px, 400.0);
        g.content(2_000, 380.0, ContentCause::ViewOpened);
        let d = g.tick(2_000 + VIEW_HOLD_MS).unwrap();
        assert_eq!(d.px, 380.0);
        let d = g.content(3_000, 370.0, ContentCause::Other).unwrap();
        assert_eq!(d.px, 370.0, "a view's own content follows it exactly");
    }

    #[test]
    fn waits_for_the_reading_then_shrinks_once() {
        let mut g = gov(480.0);
        g.typed(100);
        g.reading(100, true);
        g.content(100, 400.0, ContentCause::List);
        let d = g.content(200, 356.0, ContentCause::List).unwrap();
        assert_eq!(d.mode, HeightMode::Hold);
        assert_eq!(d.until, Some(100 + READING_WAIT_MS));
        assert!(
            g.tick(400).is_none(),
            "the typing pause is over, the reading isn't"
        );
        assert!(g.redrawn(500).unwrap().mode == HeightMode::Hold);
        g.reading(600, false);
        assert_eq!(
            g.displayed(),
            480.0,
            "the reading ending alone moves nothing"
        );
        assert!(
            g.tick(600).is_none(),
            "anything before the measure leaves it alone"
        );
        let d = g.content(600, 316.0, ContentCause::List).unwrap();
        assert_eq!(d.mode, HeightMode::Shrink);
        assert_eq!((d.from, d.px), (480.0, 316.0));
    }

    #[test]
    fn a_reading_that_never_comes_waits_only_so_long() {
        let mut g = gov(480.0);
        g.typed(100);
        g.reading(100, true);
        g.content(100, 300.0, ContentCause::List);
        assert_eq!(g.next_wake(), Some(100 + READING_WAIT_MS));
        let d = g.tick(100 + READING_WAIT_MS).unwrap();
        assert_eq!(d.mode, HeightMode::Shrink);
        assert_eq!(d.px, 300.0);
    }

    #[test]
    fn each_key_moves_the_reading_wait() {
        let mut g = gov(480.0);
        g.reading(0, true);
        g.typed(100);
        g.content(100, 300.0, ContentCause::List);
        g.typed(900);
        g.content(900, 300.0, ContentCause::List);
        assert_eq!(g.next_wake(), Some(900 + READING_WAIT_MS));
    }

    #[test]
    fn an_answer_leaves_only_the_pause_and_the_measure_that_carries_it_shrinks() {
        let mut g = gov(480.0);
        g.typed(100);
        g.reading(100, true);
        g.content(150, 300.0, ContentCause::List);
        assert_eq!(g.next_wake(), Some(100 + READING_WAIT_MS));
        g.reading(250, false);
        assert_eq!(g.next_wake(), Some(150 + SHRINK_HOLD_MS));
        let d = g.content(500, 300.0, ContentCause::List).unwrap();
        assert_eq!(d.mode, HeightMode::Shrink);
        assert_eq!(d.px, 300.0);
    }

    #[test]
    fn a_reading_never_holds_a_grow() {
        let mut g = gov(300.0);
        g.typed(100);
        g.reading(100, true);
        let d = g.content(120, 420.0, ContentCause::List).unwrap();
        assert_eq!(d.mode, HeightMode::Grow);
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
    fn a_redraw_soon_after_a_key_starts_the_pause_over() {
        let mut g = gov(420.0);
        g.typed(100);
        g.content(100, 300.0, ContentCause::List);
        let d = g.redrawn(350).unwrap();
        assert_eq!(d.mode, HeightMode::Hold);
        assert_eq!(d.until, Some(350 + SHRINK_HOLD_MS));
        assert!(g.tick(400).is_none(), "the old pause no longer ends it");
        assert_eq!(g.tick(650).unwrap().mode, HeightMode::Shrink);
    }

    #[test]
    fn a_redraw_after_the_pause_shrinks_now() {
        let mut g = gov(420.0);
        g.typed(100);
        g.content(100, 300.0, ContentCause::List);
        g.redrawn(350);
        let d = g.redrawn(420).unwrap();
        assert_eq!(d.mode, HeightMode::Shrink);
        assert_eq!(d.px, 300.0);
    }

    #[test]
    fn a_redraw_leaves_a_settled_window_alone() {
        let mut g = gov(420.0);
        assert!(g.redrawn(50).is_none());
        g.content(1_000, 300.0, ContentCause::List);
        assert!(g.redrawn(1_100).is_none());
    }

    #[test]
    fn rows_arriving_while_typing_hold_from_when_they_arrive() {
        let mut g = gov(420.0);
        g.typed(100);
        let d = g.content(250, 300.0, ContentCause::List).unwrap();
        assert_eq!(d.until, Some(250 + SHRINK_HOLD_MS));
    }

    #[test]
    fn a_held_window_grows_for_a_list_even_a_pixel_taller() {
        let mut g = gov(420.0);
        g.typed(100);
        g.content(100, 300.0, ContentCause::List);
        let d = g.content(150, 421.0, ContentCause::List).unwrap();
        assert_eq!(d.mode, HeightMode::Grow);
        assert_eq!(d.px, 421.0);
        assert!(g.tick(400).is_none(), "a grow ends the hold");
    }

    #[test]
    fn a_shrink_in_the_moment_it_grew_is_one_change() {
        let mut g = gov(394.0);
        g.content(1_000, 551.0, ContentCause::Home);
        let d = g.content(1_000, 512.0, ContentCause::List).unwrap();
        assert_eq!(d.mode, HeightMode::Grow);
        assert_eq!(
            d.px, 512.0,
            "even a small drop: nothing painted the taller height"
        );
        assert_eq!(g.displayed(), 512.0);
    }

    #[test]
    fn going_home_does_not_hold() {
        let mut g = gov(420.0);
        let d = g.content(5_000, 200.0, ContentCause::Home).unwrap();
        assert_eq!(d.mode, HeightMode::Shrink);
    }
}
