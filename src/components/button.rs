//! Reusable Button component for GPUI Script Kit
//!
//! This module provides a theme-aware button component with multiple variants
//! and support for hover states, click handlers, and keyboard shortcuts.

#![allow(dead_code)]

#[path = "button/component.rs"]
mod component;
#[cfg(test)]
#[path = "button/tests.rs"]
mod tests;
#[path = "button/types.rs"]
mod types;

pub use component::Button;
pub use types::{
    ButtonColors, ButtonVariant, BUTTON_GHOST_HEIGHT, BUTTON_GHOST_PADDING_X,
    BUTTON_GHOST_PADDING_Y,
};
