//! ACP popup-window facade.
//!
//! Window mechanics (bounds math, no-focus-steal config, child-window attach,
//! AppKit pointer plumbing) moved to the shared
//! [`crate::components::inline_popup_window`] module so ACP and the
//! menu-syntax `:`, `;`, and `!` trigger popups share a single implementation.
//!
//! This file remains as a thin compatibility facade: every `DENSE_PICKER_*`
//! / `dense_picker_*` / `popup_*` name historically exposed by this module is
//! re-exported from the shared implementation under the same ACP-specific
//! alias, so existing ACP call sites in `picker_popup.rs`,
//! `model_selector_popup.rs`, `history_popup.rs`, `view.rs`, and the source-
//! text audit tests in `src/ai/acp/tests.rs` all continue to compile without
//! edits. Add the ACP-flavored `dense_picker_height(item_count)` convenience
//! on top so callers can keep passing a bare item count and get
//! `CONTEXT_PICKER_ROW_HEIGHT` applied automatically.

use crate::components::inline_dropdown::CONTEXT_PICKER_ROW_HEIGHT;

// Re-export constants under ACP-compatible names. Consumers continue to
// reference them via `super::popup_window::DENSE_PICKER_*` without knowing the
// implementation now lives under `components::inline_popup_window`.
pub(crate) use crate::components::inline_popup_window::{
    INLINE_POPUP_DEFAULT_WIDTH as DENSE_PICKER_DEFAULT_WIDTH,
    INLINE_POPUP_EDGE_GUTTER as DENSE_PICKER_EDGE_GUTTER,
    INLINE_POPUP_EMPTY_HEIGHT as DENSE_PICKER_EMPTY_HEIGHT,
    INLINE_POPUP_LEFT_MARGIN as DENSE_PICKER_LEFT_MARGIN,
    INLINE_POPUP_MAX_VISIBLE_ROWS as DENSE_PICKER_MAX_VISIBLE_ROWS,
    INLINE_POPUP_MIN_WIDTH as DENSE_PICKER_MIN_WIDTH,
    INLINE_POPUP_VERTICAL_PADDING as DENSE_PICKER_VERTICAL_PADDING,
};

// Re-export neutral helpers under ACP-compatible names.
pub(crate) use crate::components::inline_popup_window::{
    configure_inline_popup_window as configure_popup_window,
    footer_anchored_inline_popup_top as footer_anchored_popup_top,
    inline_popup_bounds as popup_bounds,
    inline_popup_height_for_row_height as dense_picker_height_for_row_height,
    inline_popup_width_for_labels as dense_picker_width_for_labels,
    inline_popup_width_for_window as dense_picker_width_for_window,
    inline_popup_window_options as popup_window_options,
    set_inline_popup_window_bounds as set_popup_window_bounds,
};

#[cfg(target_os = "macos")]
#[allow(unused_imports)]
pub(crate) use crate::components::inline_popup_window::{
    attach_inline_popup_to_parent_window as attach_popup_to_parent_window,
    inline_popup_ns_window as popup_ns_window,
};

/// ACP-flavored convenience: popup height in rows measured against the ACP
/// context-picker row height. The neutral
/// [`crate::components::inline_popup_window::inline_popup_height_for_row_height`]
/// is what callers use when their row height differs (e.g. the history
/// popup's taller header rows).
pub(crate) fn dense_picker_height(item_count: usize) -> f32 {
    dense_picker_height_for_row_height(item_count, CONTEXT_PICKER_ROW_HEIGHT)
}

#[cfg(test)]
mod tests {
    use super::{dense_picker_height, DENSE_PICKER_EMPTY_HEIGHT};
    use crate::components::inline_popup_window::INLINE_POPUP_MAX_VISIBLE_ROWS;

    #[test]
    fn dense_picker_height_uses_shared_row_contract() {
        assert_eq!(dense_picker_height(0), DENSE_PICKER_EMPTY_HEIGHT);
        // ACP convenience should cap at the shared max-visible-rows value.
        assert_eq!(
            dense_picker_height(INLINE_POPUP_MAX_VISIBLE_ROWS + 4),
            dense_picker_height(INLINE_POPUP_MAX_VISIBLE_ROWS),
        );
    }
}
