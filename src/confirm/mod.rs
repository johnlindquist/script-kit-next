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

pub use dialog::ConfirmCallback;
pub use window::{
    close_confirm_window, dispatch_confirm_key, init_confirm_bindings, is_confirm_window_open,
    open_confirm_window,
};
