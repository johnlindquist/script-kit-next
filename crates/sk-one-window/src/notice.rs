//! Word from the app that isn't a run, and runs that can't be taken back.
//!
//! A notice (a script's `toast()` or `hud()`, a failure, a hint) lands in the ledger: the line
//! above the input says it at once while the window shows, and says it when the window comes back
//! if it was hidden. Nothing pops up, and a notice never covers a message that's waiting to go.
//!
//! Something irreversible (emptying the Trash, clearing ranking data) waits in the line like a
//! message does, and the host runs it only when the wait runs out.

/// A notice for the line above the input.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Notice {
    pub icon: String,
    pub title: String,
}

impl Notice {
    pub fn new(icon: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            title: title.into(),
        }
    }
}

/// A run that can't be taken back once it happens.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Irreversible {
    pub icon: String,
    pub title: String,
}

impl Irreversible {
    pub fn new(icon: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            title: title.into(),
        }
    }
}
