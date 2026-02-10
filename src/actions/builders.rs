//! Action builders
//!
//! Factory functions for creating context-specific action lists.

// Re-export action types into the builders module namespace so
// submodules can import them as `super::types::*`.
pub(super) mod types {
    pub use crate::actions::types::{Action, ActionCategory, ScriptInfo};
}

mod chat;
mod clipboard;
mod file_path;
mod notes;
mod script_context;
mod scriptlet;
mod shared;

pub use chat::{ChatModelInfo, ChatPromptInfo};
pub use clipboard::ClipboardEntryInfo;
pub use notes::{NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo};
pub use shared::to_deeplink_name;

#[allow(unused_imports)]
pub(crate) use scriptlet::get_scriptlet_defined_actions;
#[allow(unused_imports)]
pub(crate) use shared::format_shortcut_hint;

pub use chat::{get_ai_command_bar_actions, get_chat_context_actions};
pub use clipboard::get_clipboard_history_context_actions;
pub use file_path::{get_file_context_actions, get_path_context_actions};
pub use notes::{get_new_chat_actions, get_note_switcher_actions, get_notes_command_bar_actions};
pub use script_context::{get_global_actions, get_script_context_actions};
pub use scriptlet::get_scriptlet_context_actions_with_custom;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "builders_tests.rs"]
mod builders_extended_tests;
