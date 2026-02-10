//! DivPrompt - HTML content display
//!
//! Features:
//! - Parse and render HTML elements as native GPUI components
//! - Support for headers, paragraphs, bold, italic, code, lists, blockquotes
//! - Theme-aware styling
//! - Simple keyboard: Enter or Escape to submit

use gpui::{
    div, prelude::*, px, rgb, rgba, Context, Div, FocusHandle, Focusable, FontWeight, Hsla, Render,
    ScrollHandle, Window,
};
use std::sync::Arc;

use crate::designs::{get_tokens, DesignVariant};
use crate::logging;
use crate::theme;
use crate::ui_foundation::{get_vibrancy_background, is_key_enter, is_key_escape};
use crate::utils::{parse_color, parse_html, HtmlElement, TailwindStyles};

use super::SubmitCallback;

mod inline;
mod prompt;
mod render;
mod render_html;
mod tailwind;
#[cfg(test)]
mod tests;
mod types;

use inline::*;
pub use prompt::DivPrompt;
use render_html::*;
use tailwind::*;
use types::*;
pub use types::{ContainerOptions, ContainerPadding};
