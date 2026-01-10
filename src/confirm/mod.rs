//! Confirm Module
//!
//! A modal confirmation dialog that appears as a floating window.
//! Used by the SDK `confirm()` function to get user confirmation for actions.
//!
//! # Example Usage (SDK)
//! ```typescript
//! const confirmed = await confirm("Are you sure you want to delete this?");
//! if (confirmed) {
//!     // proceed with deletion
//! }
//!
//! // With custom button text
//! const proceed = await confirm({
//!     message: "Overwrite existing file?",
//!     confirmText: "Overwrite",
//!     cancelText: "Keep Original"
//! });
//! ```

mod constants;
mod dialog;
mod window;

pub use constants::*;
pub use dialog::{ConfirmCallback, ConfirmDialog};
pub use window::{
    close_confirm_window, get_confirm_window_handle, is_confirm_window_open, notify_confirm_window,
    open_confirm_window, ConfirmWindow,
};
