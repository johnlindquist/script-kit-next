//! Day Page vertical layout budget — the ONE owner shared by the renderer,
//! DevTools layout receipts, and the design-contract exporter.
//!
//! Moved from `src/main_sections/day_page_layout.rs` (binary target) so the
//! lib-side exporter (`src/design_contract`) and `cargo test --lib` reach the
//! same constants the renderer paints with. The binary keeps consuming these
//! items through the thin compatibility re-export left at the old path.

/// Minimum editor height preserved when the clipboard shelf expands.
pub const DAY_PAGE_MIN_EDITOR_HEIGHT_PX: f32 = 180.0;
/// Vertical padding above the shelf toggle row.
pub const DAY_PAGE_CLIPBOARD_SHELF_TOP_PADDING_PX: f32 = 6.0;
/// Fixed height of the "▸ Clipboard · N kept entries" toggle row.
pub const DAY_PAGE_CLIPBOARD_SHELF_TOGGLE_HEIGHT_PX: f32 = 20.0;
/// Gap between the toggle row and the EXPANDED entry list. This is NOT the
/// toggle's inline glyph/label gap (that is the framework `.gap_1`, a
/// different authority that happens to also equal 4px today).
pub const DAY_PAGE_CLIPBOARD_SHELF_GAP_PX: f32 = 4.0;
/// Fixed slot height of one expanded shelf row wrapper (the Day renderer
/// owns this wrapper; the compact resource row renders inside it).
pub const DAY_PAGE_CLIPBOARD_SHELF_ROW_HEIGHT_PX: f32 = 24.0;
/// Expanded shelf list never consumes more than this fraction of the body.
pub const DAY_PAGE_CLIPBOARD_SHELF_MAX_BODY_FRACTION: f32 = 0.4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DayPageLayoutBudget {
    pub body_height: f32,
    pub editor_height: f32,
    pub shelf_height: f32,
    pub shelf_list_height: f32,
}

/// One vertical owner for the Day Page editor and its clipboard accessory.
/// Rendering, DevTools receipts, and the design contract all consume this
/// calculation.
pub fn day_page_layout_budget(
    viewport_height: f32,
    header_height: f32,
    footer_height: f32,
    shelf_count: usize,
    shelf_expanded: bool,
    accessory_bottom_padding: f32,
) -> DayPageLayoutBudget {
    let body_height = (viewport_height - header_height - footer_height).max(0.0);
    if shelf_count == 0 {
        return DayPageLayoutBudget {
            body_height,
            editor_height: body_height,
            shelf_height: 0.0,
            shelf_list_height: 0.0,
        };
    }

    let shelf_chrome_height = DAY_PAGE_CLIPBOARD_SHELF_TOP_PADDING_PX
        + DAY_PAGE_CLIPBOARD_SHELF_TOGGLE_HEIGHT_PX
        + accessory_bottom_padding;
    let expanded_list_gap = if shelf_expanded {
        DAY_PAGE_CLIPBOARD_SHELF_GAP_PX
    } else {
        0.0
    };
    let available_after_min_editor =
        (body_height - DAY_PAGE_MIN_EDITOR_HEIGHT_PX - shelf_chrome_height - expanded_list_gap)
            .max(0.0);
    let responsive_list_cap =
        available_after_min_editor.min(body_height * DAY_PAGE_CLIPBOARD_SHELF_MAX_BODY_FRACTION);
    let desired_list_height = shelf_count as f32 * DAY_PAGE_CLIPBOARD_SHELF_ROW_HEIGHT_PX;
    let shelf_list_height = if shelf_expanded {
        desired_list_height.min(responsive_list_cap)
    } else {
        0.0
    };
    let shelf_height = shelf_chrome_height
        + shelf_list_height
        + if shelf_list_height > 0.0 {
            expanded_list_gap
        } else {
            0.0
        };

    DayPageLayoutBudget {
        body_height,
        editor_height: (body_height - shelf_height).max(0.0),
        shelf_height,
        shelf_list_height,
    }
}

#[cfg(test)]
mod day_page_layout_budget_tests {
    use super::*;

    #[test]
    fn expanded_shelf_preserves_editor_minimum_at_compact_height() {
        let budget = day_page_layout_budget(360.0, 68.0, 36.0, 20, true, 12.0);

        assert_eq!(budget.body_height, 256.0);
        assert_eq!(budget.editor_height, DAY_PAGE_MIN_EDITOR_HEIGHT_PX);
        assert_eq!(budget.shelf_list_height, 34.0);
        assert_eq!(
            budget.editor_height + budget.shelf_height,
            budget.body_height
        );
    }

    #[test]
    fn shelf_list_budget_responds_to_available_height() {
        let compact = day_page_layout_budget(360.0, 68.0, 36.0, 20, true, 12.0);
        let tall = day_page_layout_budget(640.0, 68.0, 36.0, 20, true, 12.0);

        assert!(compact.shelf_list_height < tall.shelf_list_height);
        assert_ne!(compact.shelf_list_height, 180.0);
        assert!(tall.editor_height >= DAY_PAGE_MIN_EDITOR_HEIGHT_PX);
    }

    #[test]
    fn collapsed_or_absent_shelf_consumes_no_list_budget() {
        let collapsed = day_page_layout_budget(480.0, 68.0, 36.0, 4, false, 12.0);
        let absent = day_page_layout_budget(480.0, 68.0, 36.0, 0, true, 12.0);

        assert_eq!(collapsed.shelf_list_height, 0.0);
        assert_eq!(collapsed.shelf_height, 38.0);
        assert_eq!(absent.shelf_height, 0.0);
        assert_eq!(absent.editor_height, absent.body_height);
    }

    /// Canonical design-contract fixture: 750×480 main window, context-only header 30,
    /// footer 32, ONE kept clipboard entry, shelf COLLAPSED (the shipped
    /// rest state and the state the reference capture reproduces).
    /// Locked per the 2026-07-11 Oracle review of the Day Page slice.
    #[test]
    fn canonical_reference_fixture_collapsed_budget() {
        let budget = day_page_layout_budget(480.0, 30.0, 32.0, 1, false, 12.0);

        assert_eq!(budget.body_height, 418.0);
        assert_eq!(budget.editor_height, 380.0);
        // 6 top + 20 toggle + 12 accessory bottom padding.
        assert_eq!(budget.shelf_height, 38.0);
        assert_eq!(budget.shelf_list_height, 0.0);
    }

    /// The same fixture with the shelf EXPANDED (one row). The geometry is a
    /// legitimate renderer contract even though the expanded raster state is
    /// still unverified (no reference capture yet — no click primitive).
    #[test]
    fn canonical_reference_fixture_one_row_expanded_budget() {
        let budget = day_page_layout_budget(480.0, 30.0, 32.0, 1, true, 12.0);

        assert_eq!(budget.shelf_list_height, 24.0);
        // 6 top + 20 toggle + 4 gap + 24 row + 12 bottom.
        assert_eq!(budget.shelf_height, 66.0);
        assert_eq!(budget.editor_height, 352.0);
    }
}
