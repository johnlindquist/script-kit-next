//! Source-contract tests locking the Tab AI overlay-to-chat cutover.
//!
//! These tests ensure that:
//! 1. No production source references overlay-era symbols
//! 2. Tab routing targets `open_tab_ai_chat(cx)`
//! 3. The storybook has been renamed from overlay to chat
//! 4. The new TabAiChat entity owns proper ChatPrompt-style state

use std::fs;
use std::path::Path;

/// Collect all `.rs` files under `src/` (production code only).
fn collect_production_sources() -> String {
    let mut combined = String::new();
    collect_rs_files(Path::new("src"), &mut combined);
    combined
}

fn collect_rs_files(dir: &Path, buf: &mut String) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, buf);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(content) = fs::read_to_string(&path) {
                buf.push_str(&content);
                buf.push('\n');
            }
        }
    }
}

const BANNED_SYMBOLS: &[&str] = &[
    "TabAiOverlayState",
    "tab_ai_state",
    "render_tab_ai_overlay",
    "handle_tab_ai_key_down",
    "submit_tab_ai_overlay",
    "close_tab_ai_overlay",
    "open_tab_ai_overlay",
];

// ---------------------------------------------------------------------------
// No overlay-era symbols in production source
// ---------------------------------------------------------------------------

#[test]
fn no_overlay_symbols_in_production_source() {
    let src = collect_production_sources();
    for symbol in BANNED_SYMBOLS {
        assert!(
            !src.contains(symbol),
            "production source must not reference overlay-era symbol: {symbol}"
        );
    }
}

// ---------------------------------------------------------------------------
// Tab routing targets open_tab_ai_chat
// ---------------------------------------------------------------------------

#[test]
fn startup_routes_tab_into_full_view_chat() {
    let src = include_str!("../src/app_impl/startup_new_tab.rs");
    assert!(
        src.contains("this.open_tab_ai_chat(cx);"),
        "startup_new_tab.rs must call open_tab_ai_chat"
    );
    assert!(
        !src.contains("open_tab_ai_overlay"),
        "startup_new_tab.rs must not reference the removed overlay opener"
    );
}

// ---------------------------------------------------------------------------
// Storybook renamed from overlay to chat
// ---------------------------------------------------------------------------

#[test]
fn storybook_uses_chat_naming_not_overlay() {
    let stories_mod = include_str!("../src/stories/mod.rs");
    assert!(
        !stories_mod.contains("tab_ai_overlay_stories"),
        "stories/mod.rs must not reference the old overlay story module"
    );
    assert!(
        !stories_mod.contains("TabAiOverlayStory"),
        "stories/mod.rs must not reference the old TabAiOverlayStory type"
    );
    assert!(
        stories_mod.contains("tab_ai_chat_stories"),
        "stories/mod.rs must reference the new tab_ai_chat_stories module"
    );
    assert!(
        stories_mod.contains("TabAiChatStory"),
        "stories/mod.rs must reference the new TabAiChatStory type"
    );
}

#[test]
fn old_overlay_story_file_does_not_exist() {
    assert!(
        !Path::new("src/stories/tab_ai_overlay_stories.rs").exists(),
        "the old tab_ai_overlay_stories.rs file must be deleted"
    );
}

// ---------------------------------------------------------------------------
// TabAiChat entity has ChatPrompt-style state
// ---------------------------------------------------------------------------

#[test]
fn app_view_state_declares_tab_ai_chat_with_entity_state() {
    let src = include_str!("../src/main_sections/app_view_state.rs");
    assert!(
        src.contains("struct TabAiChat"),
        "app_view_state.rs must declare a TabAiChat struct"
    );
    assert!(
        src.contains("input: TextInputState"),
        "TabAiChat must own input via TextInputState"
    );
    assert!(
        src.contains("turns_list_state: ListState"),
        "TabAiChat must own a ListState for scrollable turns"
    );
    assert!(
        src.contains("focus_handle: FocusHandle"),
        "TabAiChat must own a FocusHandle"
    );
}

#[test]
fn tab_ai_chat_is_non_dismissable() {
    let src = include_str!("../src/app_impl/shortcuts_hud_grid.rs");
    assert!(
        src.contains("TabAiChat"),
        "is_dismissable_view() must include TabAiChat to prevent blur-close"
    );
}

#[test]
fn render_impl_dispatches_tab_ai_chat() {
    let src = include_str!("../src/main_sections/render_impl.rs");
    assert!(
        src.contains("AppView::TabAiChat"),
        "render_impl.rs must dispatch AppView::TabAiChat"
    );
}
