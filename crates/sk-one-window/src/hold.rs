//! Messages that wait before they go.
//!
//! A held message counts down [`MESSAGE_HOLD_MS`] in the line above the input. Typing in the list
//! pauses it, `⌘Z` stops it, a rewrite gives it new words and a fresh wait, and it commits
//! exactly once.

use crate::Ms;
use crate::contract::MESSAGE_HOLD_MS;

#[derive(Clone, Debug, PartialEq)]
pub struct Held {
    pub id: u64,
    pub label: String,
    pub request: String,
    /// Milliseconds left before it goes.
    pub left: u64,
    pub total: u64,
    pub entry: Option<u64>,
    /// The run that sent it, while its words are still the run's: sending it is the run finishing.
    pub run: Option<u64>,
    pub state: HeldState,
    /// Waiting while you type. A rewritten message starts over unpaused.
    pub paused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeldState {
    Waiting,
    Sent,
    Unsent,
}

impl Held {
    pub fn live(&self) -> bool {
        self.state == HeldState::Waiting
    }

    /// Whole seconds left, as the line shows them.
    pub fn seconds_left(&self) -> u64 {
        self.left.div_ceil(1000)
    }

    pub fn progress(&self) -> f32 {
        if self.total == 0 {
            1.0
        } else {
            1.0 - self.left as f32 / self.total as f32
        }
    }
}

/// What happened to held messages as time passed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeldChange {
    Sent { id: u64 },
    Paused { id: u64 },
    Resumed { id: u64 },
}

#[derive(Clone, Debug)]
pub struct Holds {
    items: Vec<Held>,
    next_id: u64,
    hold_ms: u64,
    advanced_to: Ms,
    paused: bool,
}

impl Default for Holds {
    fn default() -> Self {
        Self::new(MESSAGE_HOLD_MS)
    }
}

impl Holds {
    pub fn new(hold_ms: u64) -> Self {
        Self {
            items: Vec::new(),
            next_id: 0,
            hold_ms,
            advanced_to: 0,
            paused: false,
        }
    }

    pub fn hold_ms(&self) -> u64 {
        self.hold_ms
    }

    pub fn set_hold_ms(&mut self, ms: u64) {
        self.hold_ms = ms;
    }

    pub fn start(&mut self, now: Ms, label: String, request: String) -> u64 {
        self.advance(now);
        self.next_id += 1;
        self.items.push(Held {
            id: self.next_id,
            label,
            request,
            left: self.hold_ms,
            total: self.hold_ms,
            entry: None,
            run: None,
            state: HeldState::Waiting,
            paused: false,
        });
        self.next_id
    }

    pub fn set_entry(&mut self, id: u64, entry: u64) {
        if let Some(h) = self.get_mut(id) {
            h.entry = Some(entry);
        }
    }

    pub fn set_run(&mut self, id: u64, run: u64) {
        if let Some(h) = self.get_mut(id) {
            h.run = Some(run);
        }
    }

    pub fn get(&self, id: u64) -> Option<&Held> {
        self.items.iter().find(|h| h.id == id)
    }

    fn get_mut(&mut self, id: u64) -> Option<&mut Held> {
        self.items.iter_mut().find(|h| h.id == id)
    }

    pub fn live(&self) -> impl Iterator<Item = &Held> {
        self.items.iter().filter(|h| h.live())
    }

    pub fn live_count(&self) -> usize {
        self.live().count()
    }

    /// The newest message still waiting: the one the line shows and `⌘Z` stops.
    pub fn newest(&self) -> Option<&Held> {
        self.items.iter().rev().find(|h| h.live())
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Moves every waiting message forward to `now`, unless paused. Messages whose wait ran out
    /// are sent, each exactly once.
    pub fn advance(&mut self, now: Ms) -> Vec<HeldChange> {
        let dt = now.saturating_sub(self.advanced_to);
        self.advanced_to = self.advanced_to.max(now);
        let mut changes = Vec::new();
        if self.paused || dt == 0 {
            return changes;
        }
        for h in self
            .items
            .iter_mut()
            .filter(|h| h.state == HeldState::Waiting)
        {
            h.left = h.left.saturating_sub(dt);
            if h.left == 0 {
                h.state = HeldState::Sent;
                changes.push(HeldChange::Sent { id: h.id });
            }
        }
        changes
    }

    /// Typing with the list showing pauses every wait; clearing the words resumes the ones it paused.
    pub fn set_paused(&mut self, now: Ms, paused: bool) -> Vec<HeldChange> {
        let mut changes = self.advance(now);
        self.paused = paused;
        for h in self
            .items
            .iter_mut()
            .filter(|h| h.live() && h.paused != paused)
        {
            h.paused = paused;
            changes.push(if paused {
                HeldChange::Paused { id: h.id }
            } else {
                HeldChange::Resumed { id: h.id }
            });
        }
        changes
    }

    /// Stops a waiting message. Returns it if it was still waiting.
    pub fn unsend(&mut self, now: Ms, id: u64) -> Option<Held> {
        self.advance(now);
        let h = self.get_mut(id)?;
        if !h.live() {
            return None;
        }
        h.state = HeldState::Unsent;
        Some(h.clone())
    }

    /// Sends a waiting message now.
    pub fn send_now(&mut self, now: Ms, id: u64) -> Option<Held> {
        self.advance(now);
        let h = self.get_mut(id)?;
        if !h.live() {
            return None;
        }
        h.left = 0;
        h.state = HeldState::Sent;
        Some(h.clone())
    }

    /// New words for a waiting message, and a fresh wait so the change can be taken back too.
    pub fn rewrite(&mut self, now: Ms, id: u64, label: String, request: String) -> bool {
        self.advance(now);
        let hold_ms = self.hold_ms;
        match self.get_mut(id) {
            Some(h) if h.live() => {
                h.label = label;
                h.request = request;
                h.left = hold_ms;
                h.total = hold_ms;
                h.paused = false;
                h.run = None;
                true
            }
            _ => false,
        }
    }

    pub fn find_live_by_label(&self, label: &str) -> Option<&Held> {
        self.items
            .iter()
            .rev()
            .find(|h| h.live() && h.label == label)
    }

    /// When something visible changes next: a send, or the line's seconds ticking down.
    pub fn next_wake(&self, now: Ms) -> Option<Ms> {
        if self.paused {
            return None;
        }
        self.live()
            .map(|h| {
                let to_second = h.left % 1000;
                let step = if to_second == 0 {
                    1000.min(h.left)
                } else {
                    to_second
                };
                now + step.max(1)
            })
            .min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_down_and_sends_once() {
        let mut h = Holds::new(8_000);
        let id = h.start(0, "To Sarah".into(), "tell sarah hi".into());
        assert_eq!(h.newest().unwrap().seconds_left(), 8);
        assert!(h.advance(2_500).is_empty());
        assert_eq!(h.newest().unwrap().seconds_left(), 6);
        assert_eq!(h.advance(8_000), vec![HeldChange::Sent { id }]);
        assert!(h.advance(9_000).is_empty(), "never sends twice");
        assert!(h.newest().is_none());
    }

    #[test]
    fn typing_pauses_the_wait() {
        let mut h = Holds::new(8_000);
        h.start(0, "To Sarah".into(), String::new());
        h.set_paused(1_000, true);
        h.advance(20_000);
        assert_eq!(h.newest().unwrap().left, 7_000);
        h.set_paused(20_000, false);
        h.advance(21_000);
        assert_eq!(h.newest().unwrap().left, 6_000);
    }

    #[test]
    fn unsend_stops_it_and_it_never_sends() {
        let mut h = Holds::new(8_000);
        let id = h.start(0, "To Sarah".into(), String::new());
        assert!(h.unsend(1_000, id).is_some());
        assert!(h.unsend(1_100, id).is_none());
        assert!(h.advance(10_000).is_empty());
    }

    #[test]
    fn rewrite_restarts_the_wait() {
        let mut h = Holds::new(8_000);
        let id = h.start(0, "To Sarah: “hi”".into(), String::new());
        h.advance(7_000);
        assert!(h.rewrite(7_000, id, "To Sarah: “thursday”".into(), String::new()));
        assert_eq!(h.newest().unwrap().left, 8_000);
    }

    #[test]
    fn a_rewritten_message_starts_over_unpaused() {
        let mut h = Holds::new(8_000);
        let a = h.start(0, "To Sarah: “friday”".into(), String::new());
        let b = h.start(0, "To #launcher: “slipped”".into(), String::new());
        h.set_run(a, 9);
        assert_eq!(
            h.set_paused(100, true),
            vec![HeldChange::Paused { id: a }, HeldChange::Paused { id: b }]
        );
        h.rewrite(200, a, "To Sarah: “thursday”".into(), String::new());
        assert_eq!(
            h.get(a).unwrap().run,
            None,
            "its words aren't the run's any more"
        );
        assert_eq!(
            h.set_paused(300, false),
            vec![HeldChange::Resumed { id: b }],
            "only what typing paused resumes"
        );
    }

    #[test]
    fn wakes_on_each_second_boundary() {
        let mut h = Holds::new(8_000);
        h.start(0, "m".into(), String::new());
        assert_eq!(h.next_wake(0), Some(1_000));
        h.advance(250);
        assert_eq!(h.next_wake(250), Some(1_000));
    }
}
