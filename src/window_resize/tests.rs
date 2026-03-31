use super::*;

#[test]
fn mini_height_empty_state_clamps_to_min_height() {
    let height = f32::from(height_for_mini_main_window(MiniMainWindowSizing {
        selectable_items: 0,
        visible_section_headers: 0,
        is_empty: true,
    }));

    assert_eq!(height, MINI_MAIN_WINDOW_MIN_HEIGHT);
}

#[test]
fn mini_height_returns_fixed_height_regardless_of_content() {
    let height = f32::from(height_for_mini_main_window(MiniMainWindowSizing {
        selectable_items: 3,
        visible_section_headers: 2,
        is_empty: false,
    }));

    // Fixed height to prevent resize jank
    assert_eq!(height, MINI_MAIN_WINDOW_MAX_HEIGHT);
}

#[test]
fn mini_height_clamps_to_max_height_when_content_exceeds_budget() {
    let height = f32::from(height_for_mini_main_window(MiniMainWindowSizing {
        selectable_items: 8,
        visible_section_headers: 5,
        is_empty: false,
    }));

    assert_eq!(height, MINI_MAIN_WINDOW_MAX_HEIGHT);
}

#[test]
fn capped_rows_drop_as_section_headers_consume_budget() {
    // 0 headers → full 8 rows
    assert_eq!(
        capped_mini_main_window_selectable_rows(0),
        MINI_MAIN_WINDOW_MAX_VISIBLE_ROWS
    );
    // With 440px max, budget = 440-56-1-30 = 353
    // 1 header: (353-32)/40 = 8.025 → 8, min(8, 8) = 8
    assert_eq!(capped_mini_main_window_selectable_rows(1), 8);
    // 2 headers: (353-64)/40 = 7.225 → 7
    assert_eq!(capped_mini_main_window_selectable_rows(2), 7);
    // 4 headers: (353-128)/40 = 5.625 → 5
    assert_eq!(capped_mini_main_window_selectable_rows(4), 5);
    // With enough headers, no selectable rows fit at all
    assert_eq!(capped_mini_main_window_selectable_rows(10), 0);
}

// ---------------------------------------------------------------------------
// Tests for mini_main_window_sizing_from_grouped_items — validates the
// content-aware counting that feeds the height formula.
// ---------------------------------------------------------------------------

use crate::list_item::GroupedListItem;

fn header(label: &str) -> GroupedListItem {
    GroupedListItem::SectionHeader(label.to_string(), None)
}

#[test]
fn grouped_sizing_empty_items_return_empty_sizing() {
    let sizing = mini_main_window_sizing_from_grouped_items(&[]);

    assert_eq!(
        sizing,
        MiniMainWindowSizing {
            selectable_items: 0,
            visible_section_headers: 0,
            is_empty: true,
        }
    );
}

#[test]
fn grouped_sizing_single_section_with_fewer_items_than_cap() {
    let grouped_items = vec![
        header("RECENT"),
        GroupedListItem::Item(0),
        GroupedListItem::Item(1),
        GroupedListItem::Item(2),
    ];

    let sizing = mini_main_window_sizing_from_grouped_items(&grouped_items);

    assert_eq!(sizing.selectable_items, 3);
    assert_eq!(sizing.visible_section_headers, 1);
    assert!(!sizing.is_empty);
}

#[test]
fn grouped_sizing_second_section_header_reduces_selectable_capacity() {
    // RECENT + 3 items + MAIN + 5 items = 10 grouped elements.
    // With 440px max, 2 headers → selectable cap = 7.
    let grouped_items = vec![
        header("RECENT"),
        GroupedListItem::Item(0),
        GroupedListItem::Item(1),
        GroupedListItem::Item(2),
        header("MAIN"),
        GroupedListItem::Item(3),
        GroupedListItem::Item(4),
        GroupedListItem::Item(5),
        GroupedListItem::Item(6),
        GroupedListItem::Item(7),
    ];

    let sizing = mini_main_window_sizing_from_grouped_items(&grouped_items);

    assert_eq!(
        sizing,
        MiniMainWindowSizing {
            selectable_items: 7,
            visible_section_headers: 2,
            is_empty: false,
        }
    );
}

#[test]
fn grouped_sizing_trailing_header_fits_with_larger_budget() {
    // RECENT + 7 items + MAIN header. With 440px max, 1 header → cap of 8,
    // so all 7 items fit and the MAIN header is also counted.
    // Then cap(2 headers) = 7, and selectable=7 >= 7 → break before Item(7).
    let grouped_items = vec![
        header("RECENT"),
        GroupedListItem::Item(0),
        GroupedListItem::Item(1),
        GroupedListItem::Item(2),
        GroupedListItem::Item(3),
        GroupedListItem::Item(4),
        GroupedListItem::Item(5),
        GroupedListItem::Item(6),
        header("MAIN"),
        GroupedListItem::Item(7),
    ];

    let sizing = mini_main_window_sizing_from_grouped_items(&grouped_items);

    assert_eq!(
        sizing,
        MiniMainWindowSizing {
            selectable_items: 7,
            visible_section_headers: 2,
            is_empty: false,
        }
    );
}

#[test]
fn grouped_sizing_items_without_any_section_headers() {
    let grouped_items: Vec<GroupedListItem> = (0..10).map(GroupedListItem::Item).collect();

    let sizing = mini_main_window_sizing_from_grouped_items(&grouped_items);

    // No headers → full cap of MAX_VISIBLE_ROWS (8).
    assert_eq!(sizing.selectable_items, 8);
    assert_eq!(sizing.visible_section_headers, 0);
    assert!(!sizing.is_empty);
}

#[test]
fn grouped_sizing_consecutive_headers_consume_capacity() {
    let grouped_items = vec![
        header("A"),
        header("B"),
        header("C"),
        GroupedListItem::Item(0),
        GroupedListItem::Item(1),
    ];

    let sizing = mini_main_window_sizing_from_grouped_items(&grouped_items);

    // 3 headers eat into the budget; selectable cap = capped_rows(3) = 5.
    assert_eq!(sizing.visible_section_headers, 3);
    assert_eq!(sizing.selectable_items, 2);
    assert!(!sizing.is_empty);
}
