//! Curated actions API surface for consumers that prefer explicit imports.
//!
//! This module keeps action-related imports discoverable without pulling every
//! helper from `actions` root exports.
#![allow(unused_imports)]

pub use super::builders::{
    ChatModelInfo, ChatPromptInfo, NewChatModelInfo, NewChatPresetInfo, NoteSwitcherNoteInfo,
    NotesInfo,
};
pub use super::dialog::ActionsDialog;
pub use super::types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo, SearchPosition,
    SectionStyle,
};
pub use super::window::{
    close_actions_window, is_actions_window_open, notify_actions_window, open_actions_window,
    resize_actions_window, WindowPosition,
};
