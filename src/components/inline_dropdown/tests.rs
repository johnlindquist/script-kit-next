use super::component::{
    inline_dropdown_clamp_selected_index, inline_dropdown_select_next, inline_dropdown_select_prev,
    inline_dropdown_visible_range,
};

#[test]
fn inline_dropdown_navigation_wraps() {
    assert_eq!(inline_dropdown_select_prev(0, 4), 3);
    assert_eq!(inline_dropdown_select_next(3, 4), 0);
}

#[test]
fn inline_dropdown_navigation_empty_list() {
    assert_eq!(inline_dropdown_select_prev(0, 0), 0);
    assert_eq!(inline_dropdown_select_next(0, 0), 0);
}

#[test]
fn inline_dropdown_navigation_single_item() {
    assert_eq!(inline_dropdown_select_prev(0, 1), 0);
    assert_eq!(inline_dropdown_select_next(0, 1), 0);
}

#[test]
fn inline_dropdown_clamps_after_filter_shrink() {
    assert_eq!(inline_dropdown_clamp_selected_index(5, 0), 0);
    assert_eq!(inline_dropdown_clamp_selected_index(5, 2), 1);
    assert_eq!(inline_dropdown_clamp_selected_index(0, 3), 0);
    assert_eq!(inline_dropdown_clamp_selected_index(2, 3), 2);
}

#[test]
fn inline_dropdown_visible_range_small_list() {
    // List smaller than max_visible — show everything.
    assert_eq!(inline_dropdown_visible_range(0, 3, 8), 0..3);
}

#[test]
fn inline_dropdown_visible_range_centers_selection() {
    assert_eq!(inline_dropdown_visible_range(5, 20, 8), 1..9);
}

#[test]
fn inline_dropdown_visible_range_clamps_to_end() {
    assert_eq!(inline_dropdown_visible_range(19, 20, 8), 12..20);
}

#[test]
fn inline_dropdown_visible_range_clamps_to_start() {
    assert_eq!(inline_dropdown_visible_range(0, 20, 8), 0..8);
}

#[test]
fn inline_dropdown_visible_range_empty_list() {
    assert_eq!(inline_dropdown_visible_range(0, 0, 8), 0..0);
}
