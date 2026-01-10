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
pub use window::open_confirm_window;
