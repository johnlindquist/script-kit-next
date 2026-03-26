//! PathPrompt - File/folder picker prompt
//!
//! Features:
//! - Browse file system starting from optional path
//! - Filter files/folders by name
//! - Navigate with keyboard
//! - Submit selected path
//!
//! Uses GPUI EventEmitter pattern for actions dialog communication:
//! - Parent subscribes to PathPromptEvent::ShowActions / CloseActions
//! - No mutex polling in render - events trigger immediate handling

use gpui::{
    div, prelude::*, uniform_list, Context, EventEmitter, FocusHandle, Focusable, Render,
    UniformListScrollHandle, Window,
};
use std::path::Path;
use std::sync::{Arc, Mutex};

// Minimal chrome: uses render_minimal_list_prompt_scaffold + render_hint_strip_leading_text
// from crate::components instead of legacy PromptContainer/PromptHeader
use crate::designs::DesignVariant;
use crate::list_item::{IconKind, ListItem, ListItemColors};
use crate::logging;
use crate::theme;

mod prompt;
mod render;
mod types;

pub use types::{
    CloseActionsCallback, PathEntry, PathInfo, PathPrompt, PathPromptEvent, ShowActionsCallback,
    SubmitCallback,
};
