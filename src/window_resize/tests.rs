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
fn mini_height_accounts_for_items_and_section_headers() {
    let height = f32::from(height_for_mini_main_window(MiniMainWindowSizing {
        selectable_items: 3,
        visible_section_headers: 2,
        is_empty: false,
    }));

    let expected = MINI_MAIN_WINDOW_HEADER_HEIGHT
        + MINI_MAIN_WINDOW_DIVIDER_HEIGHT
        + MINI_MAIN_WINDOW_HINT_STRIP_HEIGHT
        + (3.0 * LIST_ITEM_HEIGHT)
        + (2.0 * MINI_MAIN_WINDOW_SECTION_HEADER_HEIGHT);

    assert_eq!(height, expected);
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
    // Each header consumes SECTION_HEADER_HEIGHT from the list budget,
    // reducing the number of LIST_ITEM_HEIGHT rows that fit.
    assert_eq!(capped_mini_main_window_selectable_rows(1), 7);
    assert_eq!(capped_mini_main_window_selectable_rows(2), 6);
    // With 4 headers, budget drops further
    assert_eq!(capped_mini_main_window_selectable_rows(4), 5);
    // With enough headers, no selectable rows fit at all
    assert_eq!(capped_mini_main_window_selectable_rows(10), 0);
}
