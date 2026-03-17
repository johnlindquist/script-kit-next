//! Confirm Module
//!
//! In-window confirmation dialogs using gpui-component's `Dialog`.
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

mod parent_dialog;

// Used by include!() code in app_actions/handle_action/scripts.rs — clippy
// cannot trace usage through include!() and reports a false-positive dead_code
// warning for the non-lifecycle variant.
#[allow(unused_imports)]
pub(crate) use parent_dialog::{
    confirm_with_parent_dialog, open_parent_confirm_dialog, open_parent_confirm_dialog_for_entity,
    open_parent_confirm_dialog_with_lifecycle, ParentConfirmOptions,
};
