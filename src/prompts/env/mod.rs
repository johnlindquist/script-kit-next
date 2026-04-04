//! EnvPrompt - Environment variable prompt with encrypted storage
//!
//! Features:
//! - Prompt for environment variable values
//! - Secure storage via age-encrypted secrets (see crate::secrets)
//! - Mask input for secret values
//! - Remember values for future sessions
//! - Full text selection and clipboard support (cmd+c/v/x, shift+arrows)
//!
//! Design: Full-window centered input with clear visual hierarchy

use chrono::{DateTime, Utc};
use gpui::{
    div, prelude::*, px, rgb, rgba, Context, Div, FocusHandle, Focusable, Render, SharedString,
    Window,
};
use std::sync::Arc;

use crate::components::TextInputState;
use crate::designs::DesignVariant;
use crate::logging;
use crate::panel::{CURSOR_HEIGHT_LG, CURSOR_WIDTH};
use crate::secrets;
use crate::theme;
use crate::ui_foundation::{is_key_enter, is_key_escape};

use super::SubmitCallback;

mod helpers;
mod prompt;
mod render;
#[cfg(test)]
mod tests;

use helpers::*;
pub use prompt::EnvPrompt;
