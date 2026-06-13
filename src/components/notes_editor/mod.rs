//! Shared markdown notes editor component.
//!
//! Hosts markdown editing, formatting operations, preview rendering, and editor
//! focus for the Notes window and future Day Page surface. The host binds
//! document content, wires save/change callbacks, and owns window chrome.

mod component;
mod formatting;
mod ops;
mod render;
pub(crate) mod spine;
mod toolbar;
mod types;

pub use component::NotesEditor;
pub use toolbar::{
    notes_editor_toolbar_action_by_id, notes_editor_toolbar_action_title, run_toolbar_action,
    NotesEditorToolbarAction, NOTES_EDITOR_TOOLBAR_ACTIONS,
};
pub use types::{
    NotesEditorConfig, NotesEditorInputSizing, NotesEditorLayout, NotesEditorMarkdownConfig,
    NotesEditorSurfaceStyle, NOTES_EDITOR_STYLE_OWNER,
};
