//! NamingPrompt - Unified naming dialog for script and extension creation
//!
//! Features:
//! - Captures a friendly display name
//! - Shows a live kebab-case filename preview with configurable extension
//! - Validates empty names, invalid path characters, and duplicate filenames

use gpui::{
    div, prelude::*, px, rgb, rgba, Context, FocusHandle, Focusable, Render, SharedString, Window,
};
use std::path::PathBuf;
use std::sync::Arc;

use crate::components::{FocusablePrompt, FocusablePromptInterceptedKey};
use crate::designs::{get_tokens, DesignVariant};
use crate::panel::PROMPT_INPUT_FIELD_HEIGHT;
use crate::theme;
use crate::ui_foundation::{
    get_vibrancy_background, is_key_backspace, is_key_enter, printable_char,
};

use super::SubmitCallback;

mod prompt;
mod render;
mod validation;

pub use prompt::{NamingPrompt, NamingPromptConfig};
pub use validation::{NamingSubmitResult, NamingTarget, NamingValidationError};
