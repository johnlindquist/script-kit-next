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

// ── Presets adoption regression ─────────────────────────────────────
// These tests lock the shared helper contract that the presets dropdown
// depends on after migrating from bespoke selection logic.

#[test]
fn presets_adoption_select_next_wraps_at_boundary() {
    // Presets dropdown with 3 items: selecting next from last wraps to 0.
    assert_eq!(inline_dropdown_select_next(2, 3), 0);
    assert_eq!(inline_dropdown_select_next(0, 3), 1);
}

#[test]
fn presets_adoption_select_prev_wraps_at_boundary() {
    // Presets dropdown with 3 items: selecting prev from first wraps to last.
    assert_eq!(inline_dropdown_select_prev(0, 3), 2);
    assert_eq!(inline_dropdown_select_prev(1, 3), 0);
}

#[test]
fn presets_adoption_clamp_survives_empty_preset_list() {
    // When all presets are removed, clamp must return 0 (not panic).
    assert_eq!(inline_dropdown_clamp_selected_index(5, 0), 0);
}

#[test]
fn presets_adoption_visible_range_with_max_8() {
    // Presets dropdown uses max_visible=8. Verify the window slides correctly.
    // Selected at index 10 out of 15 items → window should include index 10.
    let range = inline_dropdown_visible_range(10, 15, 8);
    assert!(
        range.contains(&10),
        "selected index must be in visible range"
    );
    assert_eq!(range.len(), 8, "visible range must be exactly max_visible");
}
