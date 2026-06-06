use crate::list_item::{coerce_selection, GroupedListItem};
use gpui::{ListOffset, Pixels};

#[allow(dead_code)] // Shared with the launcher binary; the library target compiles this module without the owning call sites.
fn row_height_for_theme(
    row: &GroupedListItem,
    ix: usize,
    theme: crate::designs::MainMenuThemeVariant,
) -> f32 {
    match row {
        GroupedListItem::SectionHeader(..) => {
            if ix == 0 {
                crate::list_item::effective_first_section_header_height_for_theme(theme)
            } else {
                crate::list_item::effective_section_header_height_for_theme(theme)
            }
        }
        GroupedListItem::Status(..) => {
            crate::list_item::effective_source_status_row_height_for_theme(theme)
        }
        GroupedListItem::Item(..) => crate::list_item::effective_list_item_height_for_theme(theme),
    }
}

#[allow(dead_code)] // Shared with the launcher binary; the library target compiles this module without the owning call sites.
pub(crate) fn visible_grouped_row_range(
    rows: &[GroupedListItem],
    scroll_top: ListOffset,
    viewport_height: Pixels,
) -> Option<(usize, usize)> {
    if rows.is_empty() || viewport_height <= gpui::px(0.0) {
        return None;
    }

    let theme = crate::designs::current_main_menu_theme();
    let first = scroll_top.item_ix.min(rows.len().saturating_sub(1));
    let mut consumed = -scroll_top.offset_in_item.as_f32().max(0.0);
    let mut last = first;

    while last < rows.len() {
        consumed += row_height_for_theme(&rows[last], last, theme);
        if consumed >= viewport_height.as_f32() || last == rows.len() - 1 {
            break;
        }
        last += 1;
    }

    Some((first, last))
}

#[allow(dead_code)] // Shared with the launcher binary; the library target compiles this module without the owning call sites.
pub(crate) fn reanchor_grouped_selection(
    rows: &[GroupedListItem],
    current_selected: usize,
    scroll_top: ListOffset,
    viewport_height: Pixels,
) -> Option<usize> {
    let (first, last) = visible_grouped_row_range(rows, scroll_top, viewport_height)?;
    if current_selected >= first
        && current_selected <= last
        && matches!(rows.get(current_selected), Some(GroupedListItem::Item(_)))
    {
        return None;
    }

    coerce_selection(rows, first).or_else(|| coerce_selection(rows, last))
}

#[allow(dead_code)] // Shared with the launcher binary; the library target compiles this module without the owning call sites.
pub(crate) fn reanchor_uniform_selection(
    current_selected: usize,
    first_visible: usize,
    visible_items: usize,
    total_items: usize,
) -> Option<usize> {
    if total_items == 0 {
        return None;
    }

    let last_visible = (first_visible + visible_items.saturating_sub(1)).min(total_items - 1);
    if current_selected >= first_visible && current_selected <= last_visible {
        None
    } else {
        Some(first_visible.min(total_items - 1))
    }
}
