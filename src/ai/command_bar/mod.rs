//! AI Command Bar Module
//!
//! A searchable, keyboard-driven command menu following the patterns
//! documented in DESIGNING_POPUP_WINDOWS.md.
//!
//! # Architecture
//!
//! - `constants.rs` - Layout dimensions matching the main actions dialog
//! - `dialog.rs` - CommandBarDialog component with uniform_list virtualization
//!
//! # Usage
//!
//! The dialog is created by the parent (AiApp) and rendered as an overlay.
//! The parent routes keyboard events to the dialog's methods.
//!
//! ```rust,ignore
//! // Create the dialog
//! let dialog = cx.new(|cx| CommandBarDialog::new(cx, |action_id| {
//!     // Handle action selection
//! }));
//!
//! // Route keyboard in parent
//! match key {
//!     "up" | "arrowup" => dialog.update(cx, |d, cx| d.move_up(cx)),
//!     "down" | "arrowdown" => dialog.update(cx, |d, cx| d.move_down(cx)),
//!     "enter" => dialog.read(cx).submit_selected(),
//!     "escape" => dialog.read(cx).submit_cancel(),
//!     _ => {}
//! }
//! ```

mod constants;
mod dialog;

pub use constants::*;
pub use dialog::{CommandBarAction, CommandBarColors, CommandBarDialog};
