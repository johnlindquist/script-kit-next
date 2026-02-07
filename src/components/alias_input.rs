//! Alias Input Component
//!
//! A modal overlay for entering command aliases with proper keyboard focus handling.
//! Follows the ShortcutRecorder pattern for GPUI entity + Focusable trait.
//!
//! ## Features
//! - Text input with full keyboard support (typing, backspace, arrows, etc.)
//! - Selection support (shift+arrows, cmd+a)
//! - Clipboard operations (cmd+c, cmd+v, cmd+x)
//! - Cancel with Escape, Save with Enter
//!
//! ## Usage
//! ```rust,ignore
//! let alias_input = cx.new(|cx| {
//!     AliasInput::new(cx, theme)
//!         .with_command_name("My Script")
//!         .with_command_id("my-script-id")
//! });
//! ```

#![allow(dead_code)]

#[path = "alias_input/component.rs"]
mod component;
#[cfg(test)]
#[path = "alias_input/tests.rs"]
mod tests;
#[path = "alias_input/types.rs"]
mod types;

pub use component::AliasInput;
pub use types::{AliasInputAction, AliasInputColors, AliasValidationError};
