//! Source-level contract for cold-start main-window registration.
//!
//! The dev-fast launcher can create a hidden 480x44 panel before the first
//! hotkey expands it. If `find_and_register_main_window` rejects that compact
//! panel, the first show path leaves `window_manager::get_main_window()` as
//! `None` and debug panel invariants panic instead of launching.

const WINDOW_MANAGER: &str = include_str!("../src/window_manager/mod.rs");

#[test]
fn main_window_finder_accepts_cold_start_compact_panel_height() {
    assert!(
        WINDOW_MANAGER.contains("const MIN_HEIGHT: f64 = 40.0;"),
        "find_and_register_main_window must accept the 480x44 hidden \
         cold-start launcher panel; raising MIN_HEIGHT back above 44 \
         reintroduces the first-hotkey panel_invariants panic"
    );
    assert!(
        WINDOW_MANAGER.contains("480x44 in dev-fast"),
        "the compact-height threshold needs an inline comment documenting \
         the dev-fast startup receipt that justifies it"
    );
}
