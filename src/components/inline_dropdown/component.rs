use gpui::{AnyElement, IntoElement, SharedString};

use super::types::{InlineDropdownColors, InlineDropdownEmptyState, InlineDropdownSynopsis};

#[derive(IntoElement)]
pub(crate) struct InlineDropdown {
    pub(crate) id: SharedString,
    pub(crate) body: AnyElement,
    pub(crate) empty_state: Option<InlineDropdownEmptyState>,
    pub(crate) synopsis: Option<InlineDropdownSynopsis>,
    pub(crate) colors: InlineDropdownColors,
    pub(crate) vertical_padding: f32,
    pub(crate) horizontal_padding: f32,
}

impl InlineDropdown {
    pub(crate) fn new(id: SharedString, body: AnyElement, colors: InlineDropdownColors) -> Self {
        Self {
            id,
            body,
            empty_state: None,
            synopsis: None,
            colors,
            vertical_padding: 4.0,
            horizontal_padding: 6.0,
        }
    }

    pub(crate) fn empty_state(mut self, empty_state: InlineDropdownEmptyState) -> Self {
        self.empty_state = Some(empty_state);
        self
    }

    pub(crate) fn empty_state_opt(mut self, empty_state: Option<InlineDropdownEmptyState>) -> Self {
        self.empty_state = empty_state;
        self
    }

    pub(crate) fn synopsis(mut self, synopsis: Option<InlineDropdownSynopsis>) -> Self {
        self.synopsis = synopsis;
        self
    }

    pub(crate) fn vertical_padding(mut self, vertical_padding: f32) -> Self {
        self.vertical_padding = vertical_padding;
        self
    }

    #[allow(dead_code)] // Builder API — available for future consumers.
    pub(crate) fn horizontal_padding(mut self, horizontal_padding: f32) -> Self {
        self.horizontal_padding = horizontal_padding;
        self
    }
}

/// Clamp `selected_index` to a valid position within `item_count` items.
/// Returns `0` if the list is empty.
pub(crate) fn inline_dropdown_clamp_selected_index(
    selected_index: usize,
    item_count: usize,
) -> usize {
    if item_count == 0 {
        0
    } else {
        selected_index.min(item_count - 1)
    }
}

/// Move selection up (wrapping from top to bottom).
pub(crate) fn inline_dropdown_select_prev(selected_index: usize, item_count: usize) -> usize {
    if item_count == 0 {
        0
    } else if selected_index == 0 {
        item_count - 1
    } else {
        selected_index - 1
    }
}

/// Move selection down (wrapping from bottom to top).
pub(crate) fn inline_dropdown_select_next(selected_index: usize, item_count: usize) -> usize {
    if item_count == 0 {
        0
    } else {
        (selected_index + 1) % item_count
    }
}

/// Compute the visible row range centered on `selected_index`.
pub(crate) fn inline_dropdown_visible_range(
    selected_index: usize,
    item_count: usize,
    max_visible_rows: usize,
) -> std::ops::Range<usize> {
    if item_count <= max_visible_rows {
        return 0..item_count;
    }
    let half = max_visible_rows / 2;
    let mut start = selected_index.saturating_sub(half);
    let max_start = item_count.saturating_sub(max_visible_rows);
    if start > max_start {
        start = max_start;
    }
    start..(start + max_visible_rows).min(item_count)
}
