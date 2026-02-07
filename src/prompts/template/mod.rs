//! TemplatePrompt - String template with {{placeholder}} syntax
//!
//! Features:
//! - Parse template strings with {{name}} placeholders
//! - Tab through placeholders to fill them in
//! - Live preview of filled template
//! - Submit returns the filled template string

use gpui::{
    div, prelude::*, px, rgb, rgba, Context, FocusHandle, Focusable, Render, SharedString, Window,
};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

use crate::designs::{get_tokens, DesignVariant};
use crate::logging;
use crate::panel::PROMPT_INPUT_FIELD_HEIGHT;
use crate::template_variables;
use crate::theme;
use crate::ui_foundation::get_vibrancy_background;

use super::SubmitCallback;

mod prompt;
mod render;
#[cfg(test)]
mod tests;
mod types;

pub use prompt::TemplatePrompt;
pub use types::TemplateInput;
