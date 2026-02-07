//! Shortcut Recorder Component
//!
//! A modal overlay for recording keyboard shortcuts. Captures key combinations
//! and displays them using macOS-style symbols (⌘⇧K format).
//!
//! ## Features
//! - Captures modifier keys (Cmd, Ctrl, Alt, Shift) + a single key
//! - Displays shortcuts visually using symbols
//! - Shows conflict warnings when shortcuts are already assigned
//! - Clear, Cancel, and Save buttons
//!
//! ## Usage
//! ```rust,ignore
//! let recorder = ShortcutRecorder::new(focus_handle, theme)
//!     .with_command_name("My Script")
//!     .with_command_description("Does something useful")
//!     .on_save(|shortcut| { /* handle save */ })
//!     .on_cancel(|| { /* handle cancel */ });
//! ```

#![allow(dead_code)]

#[path = "shortcut_recorder/component.rs"]
mod component;
#[path = "shortcut_recorder/render.rs"]
mod render;
#[path = "shortcut_recorder/render_helpers.rs"]
mod render_helpers;
#[cfg(test)]
#[path = "shortcut_recorder/tests.rs"]
mod tests;
#[path = "shortcut_recorder/types.rs"]
mod types;

pub use component::ShortcutRecorder;
pub use types::{
    ConflictChecker, OnCancelCallback, OnSaveCallback, RecordedShortcut, RecorderAction,
    ShortcutConflict, ShortcutRecorderColors,
};
