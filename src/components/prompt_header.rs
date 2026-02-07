//! Reusable PromptHeader component for GPUI Script Kit
//!
//! This module provides a theme-aware header component used across all prompt types.
//! It includes a search input with blinking cursor, action buttons, and logo.
//!

#![allow(dead_code)]

#[path = "prompt_header/component.rs"]
mod component;
#[cfg(test)]
#[path = "prompt_header/tests.rs"]
mod tests;
#[path = "prompt_header/types.rs"]
mod types;

pub use component::{HeaderClickCallback, PromptHeader};
pub use types::{
    HeaderActionsDensity, PromptHeaderColors, PromptHeaderConfig,
    HEADER_ACTIONS_MIN_WIDTH_COMPACT_PX, HEADER_ACTIONS_MIN_WIDTH_EXPANDED_PX,
    HEADER_ACTIONS_MIN_WIDTH_NORMAL_PX, HEADER_PATH_PREFIX_MAX_WIDTH_PX,
};
