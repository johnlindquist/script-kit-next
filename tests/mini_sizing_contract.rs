//! Source-contract test: mini main window sizing must use header-aware row cap.
//!
//! The canonical implementation lives in `src/window_resize/mod.rs` and is
//! delegated-to from `src/app_impl/ui_window.rs`.  This test verifies:
//! 1. The `window_resize` implementation uses `capped_mini_main_window_selectable_rows`
//!    and tracks both `visible_section_headers` and `selectable_items`.
//! 2. The `ui_window` wrapper delegates to the canonical implementation.
//!
//! Run: `cargo test --test mini_sizing_contract`

use std::fs;

/// Read a source file or panic with a clear message.
fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"))
}

/// Extract the text between `start` marker and `end` marker within `source`.
fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_ix = source
        .find(start)
        .unwrap_or_else(|| panic!("marker not found: '{start}'"));
    let tail = &source[start_ix..];
    let end_rel = tail.find(end).unwrap_or(tail.len());
    &tail[..end_rel]
}

#[test]
fn mini_sizing_helper_uses_header_aware_row_cap() {
    // The canonical implementation now lives in window_resize/mod.rs.
    let source = read("src/window_resize/mod.rs");
    let helper = section_between(
        &source,
        "fn mini_main_window_sizing_from_grouped_items",
        "pub(crate) struct MiniMainWindowSizing",
    );

    assert!(
        helper.contains("capped_mini_main_window_selectable_rows"),
        "mini_main_window_sizing_from_grouped_items must delegate to \
         capped_mini_main_window_selectable_rows so section headers reduce visible rows"
    );

    assert!(
        helper.contains("visible_section_headers"),
        "mini_main_window_sizing_from_grouped_items must track visible_section_headers"
    );

    assert!(
        helper.contains("selectable_items"),
        "mini_main_window_sizing_from_grouped_items must track selectable_items"
    );
}

#[test]
fn ui_window_wrapper_delegates_to_canonical_implementation() {
    let source = read("src/app_impl/ui_window.rs");
    let helper = section_between(
        &source,
        "fn mini_main_window_sizing_from_grouped_items",
        "impl ScriptListApp",
    );

    assert!(
        helper.contains("crate::window_resize::mini_main_window_sizing_from_grouped_items"),
        "ui_window wrapper must delegate to crate::window_resize::mini_main_window_sizing_from_grouped_items"
    );
}
