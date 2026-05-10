#[test]
fn root_browser_tabs_foreground_search_is_cache_only() {
    let browser_tabs = include_str!("../../src/browser_tabs.rs");
    let search_fn = browser_tabs
        .split("pub(crate) fn search_root_browser_tabs_meta(")
        .nth(1)
        .and_then(|rest| rest.split("pub(crate) fn focus_root_browser_tab(").next())
        .expect("search_root_browser_tabs_meta should exist");

    assert!(search_fn.contains("ensure_root_browser_tabs_refresh("));
    assert!(search_fn.contains("cached_root_browser_tabs_snapshot("));
    assert!(
        !search_fn.contains("list_open_tabs("),
        "foreground browser-tab root search must not call AppleScript/JXA tab enumeration"
    );
}

#[test]
fn root_browser_history_foreground_search_is_cache_only() {
    let browser_history = include_str!("../../src/browser_history.rs");
    let search_fn = browser_history
        .split("pub(crate) fn search_root_browser_history_meta(")
        .nth(1)
        .and_then(|rest| rest.split("pub(crate) fn open_browser_history_url(").next())
        .expect("search_root_browser_history_meta should exist");

    assert!(search_fn.contains("ensure_root_browser_history_refresh("));
    assert!(search_fn.contains("cached_root_browser_history_snapshot("));
    assert!(
        !search_fn.contains("copy_sqlite_db_snapshot("),
        "foreground browser-history root search must not copy browser SQLite databases"
    );
    assert!(
        !search_fn.contains("Connection::open"),
        "foreground browser-history root search must not open SQLite databases"
    );
}

#[test]
fn passive_snapshot_refresh_does_not_publish_current_frame() {
    for (source_name, source) in [
        ("browser_tabs", include_str!("../../src/browser_tabs.rs")),
        (
            "browser_history",
            include_str!("../../src/browser_history.rs"),
        ),
    ] {
        assert!(
            !source.contains("invalidate_grouped_cache"),
            "{source_name} passive snapshot refresh must not invalidate grouped root results"
        );
        assert!(
            !source.contains("publish_active_results"),
            "{source_name} passive snapshot refresh must not publish into the active root frame"
        );
        assert!(
            !source.contains("cx.notify"),
            "{source_name} passive snapshot refresh must not notify the main UI frame"
        );
    }
}

#[test]
fn passive_root_sections_share_score_cap_and_order() {
    let grouping = include_str!("../../src/scripts/grouping.rs");

    assert!(grouping.contains("pub const ROOT_PASSIVE_RESULT_SCORE_BASE: i32 = 100_000"));
    for section_fn in [
        "append_root_browser_tabs_section",
        "append_root_notes_section",
        "append_root_clipboard_history_section",
        "append_root_dictation_history_section",
        "append_root_acp_history_section",
        "append_root_browser_history_section",
    ] {
        let section = grouping
            .split(&format!("fn {section_fn}("))
            .nth(1)
            .and_then(|rest| rest.split("\nfn ").next())
            .unwrap_or_else(|| panic!("{section_fn} should exist"));
        assert!(
            section.contains("root_passive_result_score(rank)"),
            "{section_fn} should use the shared passive score cap"
        );
    }

    assert!(
        grouping.find("append_recent_root_file_section(")
            < grouping.find("append_root_browser_tabs_section("),
        "Browser Tabs should remain after Files/Recent Files"
    );
    assert!(
        grouping.find("append_root_browser_tabs_section(")
            < grouping.find("append_root_notes_section("),
        "Browser Tabs should remain before Notes"
    );
    assert!(
        grouping.find("append_root_clipboard_history_section(")
            < grouping.find("append_root_dictation_history_section("),
        "Dictation History should remain after Clipboard History"
    );
    assert!(
        grouping.find("append_root_dictation_history_section(")
            < grouping.find("append_root_acp_history_section("),
        "Dictation History should remain before AI Conversations"
    );
    assert!(
        grouping.find("append_root_acp_history_section(")
            < grouping.find("append_root_browser_history_section("),
        "Browser History should remain after AI Conversations"
    );
}
