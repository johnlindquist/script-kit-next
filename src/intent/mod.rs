//! Intent-first launcher: the user says what should happen, Jev reads it against what's
//! on screen, and the engine turns Jev's answers into a plan the main menu can run.
//!
//! The engine never writes text of its own. Every slot (what to remember, the question,
//! the time) is one run of the user's own words, chosen by Jev.

pub mod catalog;
pub mod context;
pub mod engine;
pub mod jev;
pub mod launcher;
pub mod present;
pub mod script;
pub mod spans;

#[cfg(test)]
mod replay_tests;
#[cfg(test)]
mod tests;

pub use catalog::{
    native_window_ops, Catalog, Command, CommandCode, CommandKind, WindowAction, WindowOp,
};
pub use context::{FrontApp, LastAction, LastStep, MenuCommand, OpenWindow, ScreenContext};
pub use engine::{understand, EngineOptions, Pathway, Plan, Reading, Step, COMPLETE_THRESHOLD};
pub use jev::{JevClient, JevConfig, JevError, Oracle};
pub use launcher::{
    Finish, HeldOutcome, IntentLauncher, IntentRow, OracleSlot, RowState, TextChange, Ticket,
};
pub use present::{plan_subtitle, plan_verb};
pub use script::{normalize_shortcut, plan_to_script, DraftScript, ScriptOptions};

/// Requests shorter than this are left to the launcher's own search.
pub const MIN_WORDS: usize = 2;

/// Whether a line is worth asking Jev about. Launcher syntax (captures, sigils, filters)
/// already says exactly what to do, and one word is a search.
pub fn wants_reading(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.split_whitespace().count() < MIN_WORDS {
        return false;
    }
    let first = trimmed.chars().next().unwrap_or(' ');
    if matches!(
        first,
        '@' | '/' | '.' | '|' | '>' | ';' | '+' | ':' | '~' | '!' | '?' | '=' | '#'
    ) {
        return false;
    }
    let head = trimmed.split_whitespace().next().unwrap_or_default();
    if head.ends_with(';') || head.contains("://") {
        return false;
    }
    true
}
