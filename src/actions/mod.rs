//! Actions Dialog Module
//!
//! Provides a searchable action menu as a compact overlay popup for quick access
//! to script management and global actions (edit, create, settings, quit, etc.)
//!
//! The dialog can be rendered in two ways:
//! 1. As an inline overlay within the main window (legacy)
//! 2. As a separate floating window with its own vibrancy blur (preferred)
//!
//! ## Module Structure
//! - `types`: Core types (Action, ActionCategory, ScriptInfo)
//! - `builders`: Factory functions for creating action lists
//! - `constants`: Popup dimensions and styling constants
//! - `dialog`: ActionsDialog struct and implementation
//! - `window`: Separate vibrancy window for actions panel

mod builders;
mod command_bar;
mod constants;
mod dialog;
mod types;
mod window;

// Re-export only the public API that is actually used externally:
// - ScriptInfo: used by main.rs for action context
// - ActionsDialog: the main dialog component
// - Window functions for separate vibrancy window

pub use builders::{
    get_ai_command_bar_actions, get_new_chat_actions, get_note_switcher_actions,
    get_notes_command_bar_actions,
};
#[allow(unused_imports)]
pub(crate) use builders::{
    get_global_actions, get_script_context_actions, get_scriptlet_context_actions_with_custom,
};
pub use builders::{
    to_deeplink_name, ChatModelInfo, ChatPromptInfo, ClipboardEntryInfo, NewChatModelInfo,
    NewChatPresetInfo, NoteSwitcherNoteInfo, NotesInfo,
};
pub use command_bar::{CommandBar, CommandBarConfig};
pub use dialog::ActionsDialog;
pub use types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo, SearchPosition,
    SectionStyle,
};

// Window functions for separate vibrancy window
pub use window::{
    close_actions_window, is_actions_window_open, notify_actions_window, open_actions_window,
    resize_actions_window, WindowPosition,
};
// get_actions_window_handle available but not re-exported (use window:: directly if needed)

pub mod prelude;

#[cfg(test)]
include!("tests.rs");
