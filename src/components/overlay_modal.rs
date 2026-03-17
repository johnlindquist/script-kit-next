//! Shared overlay modal shell.
//!
//! Provides constants, animation types, and helpers consumed by both
//! `AliasInput` and `ShortcutRecorder` so the overlay structure is
//! defined once.

#[path = "overlay_modal/animation.rs"]
mod animation;
#[path = "overlay_modal/types.rs"]
mod types;

pub(crate) use animation::OverlayAnimation;
pub(crate) use types::{overlay_color_with_alpha, BUTTON_GAP, MODAL_PADDING, MODAL_WIDTH};

// Used by tests in alias_input and shortcut_recorder modules
#[allow(unused_imports)]
pub(crate) use types::{
    compute_overlay_appear_style, OverlayAppearStyle, OVERLAY_ANIMATION_DURATION_MS,
    OVERLAY_MODAL_ENTRY_OFFSET_PX, OVERLAY_MODAL_START_OPACITY,
};
