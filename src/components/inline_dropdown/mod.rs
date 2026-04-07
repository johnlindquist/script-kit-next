mod component;
mod render;
mod row;
#[cfg(test)]
mod tests;
mod types;

pub(crate) use component::{
    inline_dropdown_clamp_selected_index, inline_dropdown_select_next, inline_dropdown_select_prev,
    inline_dropdown_visible_range, InlineDropdown,
};
pub(crate) use row::{
    render_compact_synopsis_strip, render_dense_monoline_picker_row,
    render_dense_monoline_picker_row_with_accessory,
    render_dense_monoline_picker_row_with_leading_visual, render_highlighted_meta,
    render_highlighted_text, COMMAND_OPACITY, CONTEXT_PICKER_ROW_HEIGHT,
    CONTEXT_PICKER_SYNOPSIS_HEIGHT, GHOST, GOLD, HINT, MUTED_OP,
};
pub(crate) use types::{InlineDropdownColors, InlineDropdownEmptyState, InlineDropdownSynopsis};
