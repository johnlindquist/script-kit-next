// App navigation methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: move_selection_up, move_selection_down, scroll_to_selected, etc.

#[inline]
fn page_down_target_index(
    grouped_items: &[GroupedListItem],
    selected_index: usize,
    page_size: usize,
) -> usize {
    let Some(last_selectable) = grouped_items
        .iter()
        .rposition(|item| matches!(item, GroupedListItem::Item(_)))
    else {
        return selected_index;
    };

    if selected_index >= last_selectable {
        return selected_index;
    }

    let mut remaining = page_size;
    let mut target = selected_index;
    for i in (selected_index + 1)..=last_selectable {
        if matches!(grouped_items.get(i), Some(GroupedListItem::Item(_))) {
            target = i;
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
    }

    target
}

#[inline]
fn wheel_scroll_target_index(current_item: usize, item_count: usize, delta_lines: f32) -> usize {
    if item_count == 0 {
        return 0;
    }

    let max_item = item_count.saturating_sub(1);
    let items_to_scroll = (-delta_lines).round() as i32;
    (current_item as i32 + items_to_scroll).clamp(0, max_item as i32) as usize
}

#[inline]
fn validated_selection_index(grouped_items: &[GroupedListItem], selected_index: usize) -> usize {
    list_item::coerce_selection(grouped_items, selected_index).unwrap_or(0)
}

