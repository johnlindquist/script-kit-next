#[cfg(test)]
mod tests {
    use super::{page_down_target_index, validated_selection_index, wheel_scroll_target_index};
    use crate::list_item::GroupedListItem;

    #[test]
    fn test_move_selection_page_down_clamps_to_last_item() {
        let rows = vec![
            GroupedListItem::SectionHeader("Suggested".to_string(), None),
            GroupedListItem::Item(0),
            GroupedListItem::Item(1),
            GroupedListItem::SectionHeader("Main".to_string(), None),
            GroupedListItem::Item(2),
        ];

        assert_eq!(page_down_target_index(&rows, 1, 10), 4);
        assert_eq!(page_down_target_index(&rows, 4, 10), 4);
    }

    #[test]
    fn test_handle_scroll_wheel_coalesces_rapid_deltas() {
        let item_count = 20;
        let start = 7;

        let after_first = wheel_scroll_target_index(start, item_count, 0.2);
        let after_second = wheel_scroll_target_index(after_first, item_count, 0.2);
        let after_third = wheel_scroll_target_index(after_second, item_count, -0.2);

        assert_eq!(after_first, start);
        assert_eq!(after_second, start);
        assert_eq!(after_third, start);
    }

    #[test]
    fn test_validate_selection_bounds_recovers_from_out_of_range_index() {
        let rows = vec![
            GroupedListItem::SectionHeader("Suggested".to_string(), None),
            GroupedListItem::Item(0),
            GroupedListItem::Item(1),
            GroupedListItem::SectionHeader("Main".to_string(), None),
            GroupedListItem::Item(2),
        ];
        let headers_only = vec![
            GroupedListItem::SectionHeader("A".to_string(), None),
            GroupedListItem::SectionHeader("B".to_string(), None),
        ];

        assert_eq!(validated_selection_index(&rows, 999), 4);
        assert_eq!(validated_selection_index(&rows, 0), 1);
        assert_eq!(validated_selection_index(&headers_only, 4), 0);
    }
}
