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

use crate::components::{
    PromptContainer, PromptContainerColors, PromptContainerConfig, PromptHeader,
    PromptHeaderColors, PromptHeaderConfig,
};
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
