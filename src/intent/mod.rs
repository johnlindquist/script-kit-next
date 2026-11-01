//! Intent-first launcher: the user says what should happen, Jev reads it against what's
//! on screen, and the engine turns Jev's answers into a plan the main menu can run.
//!
//! The engine never writes text of its own. Every slot (what to remember, the question,
//! the time) is one run of the user's own words, chosen by Jev.

pub mod catalog;
pub mod context;
pub mod engine;
pub mod filename;
pub mod fixture_oracle;
pub mod jev;
pub mod launcher;
pub mod present;
pub mod script;
pub mod spans;
pub mod words;

#[cfg(test)]
mod holdout_tests;
#[cfg(test)]
mod replay_tests;
#[cfg(test)]
mod tests;

pub use catalog::{
    native_window_ops, Catalog, Command, CommandCode, CommandKind, WindowAction, WindowOp,
};
pub use context::{FrontApp, LastAction, LastStep, MenuCommand, OpenWindow, ScreenContext};
pub use engine::{understand, EngineOptions, Pathway, Plan, Reading, Step, COMPLETE_THRESHOLD};
pub use fixture_oracle::FixtureWorld;
pub use jev::{JevClient, JevConfig, JevError, Oracle};
pub use launcher::{readings, IntentLauncher, JevRow, OracleSlot, READINGS};
pub use present::{plan_subtitle, plan_verb};
pub use script::{normalize_shortcut, plan_to_script, DraftScript, ScriptOptions};
pub use words::{WordKind, WordRead, WordSource};

/// A second line of defense, not a guarantee of secret detection. Local-only mode
/// blocks *all* remote readings, including partial voice transcripts.
pub fn sensitive_words(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "password",
        "passphrase",
        "api key",
        "api_key",
        "private key",
        "access token",
        "recovery phrase",
    ]
    .iter()
    .any(|label| lower.contains(label))
        || crate::clipboard_history::matches_secret_content(text)
}

/// Whether a line is worth asking Jev about. Launcher syntax (captures, sigils, filters)
/// already says exactly what to do.
pub fn wants_reading(text: &str) -> bool {
    let trimmed = text.trim();
    let Some(first) = trimmed.chars().next() else {
        return false;
    };
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
