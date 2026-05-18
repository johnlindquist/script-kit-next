use super::component::{
    inline_dropdown_clamp_selected_index, inline_dropdown_select_next, inline_dropdown_select_prev,
    inline_dropdown_visible_range, inline_dropdown_visible_range_from_start,
};
use super::row::{CONTEXT_PICKER_ROW_HEIGHT, SOFT_COMPACT_PICKER_ROW_HEIGHT};

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
fn inline_dropdown_rows_match_launcher_row_height() {
    assert_eq!(
        CONTEXT_PICKER_ROW_HEIGHT,
        crate::list_item::LIST_ITEM_HEIGHT
    );
}

#[test]
fn soft_compact_picker_row_is_tighter_than_launcher_rows() {
    assert!(SOFT_COMPACT_PICKER_ROW_HEIGHT < CONTEXT_PICKER_ROW_HEIGHT);
}

#[test]
fn soft_compact_meta_badge_uses_menu_tint_instead_of_dark_badge_surface() {
    let row_source = include_str!("row.rs");
    let badge_start = row_source
        .find("fn render_soft_compact_meta_badge")
        .expect("soft compact meta badge renderer should exist");
    let badge_source = &row_source[badge_start..];

    assert!(
        badge_source.contains(".bg(colors.foreground.opacity(GHOST))"),
        "soft compact metadata badges should use the same low-opacity foreground tint as launcher menu badges"
    );
    assert!(
        !badge_source.contains("chrome.badge_bg_rgba"),
        "soft compact metadata badges must not use the darker global badge surface"
    );
}

#[test]
fn inline_dropdown_visible_range_small_list() {
    // List smaller than max_visible — show everything.
    assert_eq!(inline_dropdown_visible_range(0, 3, 8), 0..3);
}

#[test]
fn inline_dropdown_visible_range_holds_first_page_until_selection_leaves_it() {
    assert_eq!(inline_dropdown_visible_range(6, 20, 8), 0..8);
    assert_eq!(inline_dropdown_visible_range(7, 20, 8), 0..8);
    assert_eq!(inline_dropdown_visible_range(8, 20, 8), 1..9);
}

#[test]
fn inline_dropdown_visible_range_from_start_waits_for_top_and_bottom_edges() {
    assert_eq!(inline_dropdown_visible_range_from_start(8, 8, 20, 8), 8..16);
    assert_eq!(
        inline_dropdown_visible_range_from_start(8, 15, 20, 8),
        8..16
    );
    assert_eq!(inline_dropdown_visible_range_from_start(8, 7, 20, 8), 7..15);
    assert_eq!(
        inline_dropdown_visible_range_from_start(8, 16, 20, 8),
        9..17
    );
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
