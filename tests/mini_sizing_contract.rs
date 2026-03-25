//! Source-contract test: mini main window sizing must use header-aware row cap.
//!
//! This test reads `src/app_impl/ui_window.rs` at the source level and asserts
//! that `mini_main_window_sizing_from_grouped_items` delegates to
//! `capped_mini_main_window_selectable_rows` and tracks both
//! `visible_section_headers` and `selectable_items`.
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
    let source = read("src/app_impl/ui_window.rs");
    let helper = section_between(
        &source,
        "fn mini_main_window_sizing_from_grouped_items",
        "impl ScriptListApp",
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
