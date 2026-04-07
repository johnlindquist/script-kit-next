//! Compatibility shim — the canonical implementation now lives in
//! `crate::components::inline_dropdown::row`.  Existing callers under
//! `src/ai/` can keep importing from this path until they are migrated.

pub(crate) use crate::components::inline_dropdown::{
    render_compact_synopsis_strip, render_dense_monoline_picker_row,
    render_dense_monoline_picker_row_with_accessory,
    render_dense_monoline_picker_row_with_leading_visual, render_highlighted_meta,
    render_highlighted_text, COMMAND_OPACITY, CONTEXT_PICKER_ROW_HEIGHT,
    CONTEXT_PICKER_SYNOPSIS_HEIGHT, GHOST, GOLD, HINT, MUTED_OP,
};
