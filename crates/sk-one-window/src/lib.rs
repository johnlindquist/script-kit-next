//! The one window's rules, with no UI and no clock.
//!
//! Script Kit's main window is the only window: the input never moves, the line above it is always
//! there, every list starts with a header, finished work waits or can be undone from the list, and
//! Enter never runs something that changed under the key. The host feeds [`window::Event`]s with
//! the current time and applies the [`window::Effect`]s it gets back.

pub mod contract;
pub mod enter;
pub mod height;
pub mod history;
pub mod hold;
pub mod ledger;
pub mod list;
pub mod notice;
pub mod row;
pub mod setup;
pub mod slot;
pub mod stage;
pub mod trace;
pub mod view;
pub mod window;

/// Milliseconds of the host's clock.
pub type Ms = u64;

pub use enter::EnterTarget;
pub use height::{HeightDecision, HeightMode};
pub use ledger::{Entry, EntryState, Risk};
pub use list::Item;
pub use row::{Row, RowAct};
pub use slot::{SlotKind, SlotLine, SlotStyle};
pub use stage::{Subject, SubjectKind};
pub use window::{
    Config, Done, Effect, Event, HeldOp, Key, OneWindow, Outcome, RunHow, Snapshot, ViewOp,
    ViewSpec,
};
