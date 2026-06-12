//! Shared markdown notes editor component.
//!
//! Hosts markdown editing, formatting operations, preview rendering, and editor
//! focus for the Notes window and future Day Page surface. The host binds
//! document content, wires save/change callbacks, and owns window chrome.

mod component;
mod formatting;
mod ops;
mod render;
mod toolbar;
mod types;

pub use component::NotesEditor;
pub use toolbar::{run_toolbar_action, NotesEditorToolbarAction, NOTES_EDITOR_TOOLBAR_ACTIONS};
pub use types::{NotesEditorConfig, NotesEditorLayout, NotesEditorSurfaceStyle};
