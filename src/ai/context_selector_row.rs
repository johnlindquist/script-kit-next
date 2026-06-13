//! Compatibility shim — the canonical implementation now lives in
//! `crate::components::inline_dropdown::row`.  Existing callers under
//! `src/ai/` can keep importing from this path until they are migrated.

pub(crate) use crate::components::inline_dropdown::{
    render_compact_synopsis_strip, render_dense_monoline_picker_row,
    render_dense_monoline_picker_row_with_accessory,
    render_dense_monoline_picker_row_with_leading_visual, render_highlighted_meta,
    render_highlighted_text, render_soft_compact_picker_row, COMMAND_OPACITY,
    CONTEXT_SELECTOR_ROW_HEIGHT, CONTEXT_SELECTOR_SYNOPSIS_HEIGHT, GHOST, HINT, MUTED_OP,
    SOFT_COMPACT_PICKER_ROW_HEIGHT,
};
