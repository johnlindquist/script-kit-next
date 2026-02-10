//! SelectPrompt - Multi-select from choices
//!
//! Features:
//! - Select multiple items from a list
//! - Toggle selection with Cmd/Ctrl+Space
//! - Filter choices by typing
//! - Submit selected items

use gpui::{
    div, prelude::*, px, rgb, rgba, uniform_list, AnyElement, Context, FocusHandle, Focusable,
    Render, ScrollStrategy, SharedString, UniformListScrollHandle, Window,
};
use std::collections::HashSet;
use std::ops::Range;
use std::sync::Arc;

use crate::components::{
    Density, ItemState, LeadingContent, TextContent, TrailingContent, UnifiedListItem,
    UnifiedListItemColors,
};
use crate::designs::{get_tokens, DesignColors, DesignVariant};
use crate::list_item::{IconKind, LIST_ITEM_HEIGHT};
use crate::logging;
use crate::panel::PROMPT_INPUT_FIELD_HEIGHT;
use crate::protocol::{generate_semantic_id, Choice};
use crate::scripts;
use crate::theme;
use crate::ui_foundation::get_vibrancy_background;

use super::SubmitCallback;

mod prompt;
mod render;
mod search;
#[cfg(test)]
mod tests;
mod types;

pub use prompt::SelectPrompt;
use search::*;
use types::*;
