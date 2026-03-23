//! Confirm Module
//!
//! Native popup confirmation dialogs rendered in a dedicated GPUI
//! `WindowKind::PopUp` window so macOS vibrancy blur comes from the NSPanel
//! itself instead of a translucent in-window overlay.
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
mod window;

// Used by include!() code in app_actions/handle_action/scripts.rs — clippy
// cannot trace usage through include!() and reports a false-positive dead_code
// warning for the non-lifecycle variant.
#[allow(unused_imports)]
pub(crate) use parent_dialog::{
    confirm_with_parent_dialog, open_parent_confirm_dialog, open_parent_confirm_dialog_for_entity,
    open_parent_confirm_dialog_with_lifecycle, ParentConfirmOptions,
};

#[allow(unused_imports)]
pub(crate) use window::is_confirm_window_open;
#[allow(unused_imports)]
pub(crate) use window::route_key_to_confirm_popup;
