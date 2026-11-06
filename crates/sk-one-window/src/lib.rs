//! The one window's rules, with no UI and no clock.
//!
//! Script Kit's main window is the only window: the input never moves, the line above it is always
//! there, every list starts with a header, finished work waits or can be undone from the list, and
//! Enter never runs something that changed under the key. The host feeds [`window::Event`]s with
//! the current time and applies the [`window::Effect`]s it gets back.

pub mod answer;
pub mod asks;
pub mod chat;
pub mod chat_with;
pub mod clipboard;
pub mod connect;
pub mod contract;
pub mod countdown;
pub mod cursor_dictation;
pub mod display;
pub mod enter;
pub mod font_preference;
pub mod form;
pub mod history;
pub mod hold;
pub mod ledger;
pub mod lessons;
pub mod list;
pub mod marks;
pub mod models;
pub mod notice;
pub mod past_chats;
pub mod personal;
pub mod personal_list;
pub mod quick_tools;
pub mod row;
pub mod setup;
pub mod slot;
pub mod stage;
pub mod steady;
pub mod steps;
pub mod strip;
pub mod talk;
pub mod trace;
pub mod view;
pub mod window;
pub mod work;

/// Milliseconds of the host's clock.
pub type Ms = u64;

pub use enter::EnterTarget;
pub use ledger::{Entry, EntryState, Risk};
pub use lessons::{Lesson, Lessons};
pub use list::Item;
pub use row::{Row, RowAct};
pub use slot::{ContextChip, SlotKind, SlotLine, SlotStyle, SubjectChip};
pub use stage::{Subject, SubjectKind};
pub use talk::{Dest, Reading, TalkEvent};
pub use window::{
    Config, Done, Effect, Event, HeldOp, Key, OneWindow, Outcome, RunHow, Snapshot, ViewOp,
    ViewSpec,
};
