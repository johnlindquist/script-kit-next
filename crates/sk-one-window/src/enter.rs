//! Enter never changes what it does without a keypress.
//!
//! When Enter's target changes on its own (a form finished, an answer arrived, Jev went quiet),
//! Enter is ignored for [`ENTER_GUARD_MS`], so a press meant for the old target can't run the new
//! one. The Enter that finished something never also runs the first row of what replaces it.

use serde::{Deserialize, Serialize};

use crate::Ms;
use crate::contract::ENTER_GUARD_MS;

/// What Enter would do right now, as the ↵ button says it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnterTarget {
    /// Stable identity of the target; a new id is a new target.
    pub id: String,
    pub label: String,
}

impl EnterTarget {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct EnterGuard {
    armed_until: Ms,
    last_view_id: Option<String>,
}

impl EnterGuard {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ignore Enter for the guard's length from `now`.
    pub fn arm(&mut self, now: Ms) {
        self.armed_until = self.armed_until.max(now + ENTER_GUARD_MS);
    }

    /// Arms only when the change wasn't caused by the user's own key, click, or typing.
    pub fn arm_unless_user(&mut self, now: Ms, user: bool) {
        if !user {
            self.arm(now);
        }
    }

    pub fn armed(&self, now: Ms) -> bool {
        now < self.armed_until
    }

    pub fn armed_until(&self) -> Ms {
        self.armed_until
    }

    /// A view's Enter target, observed each time it may have changed.
    pub fn observe_view(&mut self, now: Ms, id: &str, user: bool) {
        if self.last_view_id.as_deref() != Some(id) {
            self.arm_unless_user(now, user);
            self.last_view_id = Some(id.to_string());
        }
    }

    /// A different view is on top (or none): its first target is not a change.
    pub fn forget_view(&mut self) {
        self.last_view_id = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changes_without_a_keypress_arm_the_guard() {
        let mut g = EnterGuard::new();
        g.observe_view(0, "form:submit", true);
        assert!(!g.armed(1));
        g.observe_view(100, "form:done", false);
        assert!(g.armed(100 + ENTER_GUARD_MS - 1));
        assert!(!g.armed(100 + ENTER_GUARD_MS));
    }

    #[test]
    fn a_keypress_change_does_not_arm() {
        let mut g = EnterGuard::new();
        g.observe_view(0, "a", true);
        g.observe_view(10, "b", true);
        assert!(!g.armed(11));
    }

    #[test]
    fn arming_never_shortens_an_armed_guard() {
        let mut g = EnterGuard::new();
        g.arm(1_000);
        g.arm(500);
        assert_eq!(g.armed_until(), 1_000 + ENTER_GUARD_MS);
    }
}
