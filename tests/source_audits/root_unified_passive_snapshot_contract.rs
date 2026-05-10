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
        search_fn.find("root_browser_tabs_query_is_eligible(")
            < search_fn.find("ensure_root_browser_tabs_refresh("),
        "disabled or ineligible browser-tab root search must not start a refresh"
    );
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
        search_fn.find("root_browser_history_query_is_eligible(")
            < search_fn.find("ensure_root_browser_history_refresh("),
        "disabled or ineligible browser-history root search must not start a refresh"
    );
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
fn browser_snapshot_sources_expose_status_for_state_first_proof() {
    let browser_tabs = include_str!("../../src/browser_tabs.rs");
    let browser_history = include_str!("../../src/browser_history.rs");

    for (source_name, source, fn_name) in [
        (
            "browser_tabs",
            browser_tabs,
            "root_browser_tabs_snapshot_status",
        ),
        (
            "browser_history",
            browser_history,
            "root_browser_history_snapshot_status",
        ),
    ] {
        assert!(
            source.contains("pub(crate) struct RootPassiveSnapshotStatus")
                && source.contains("pub generation: u64")
                && source.contains("pub refreshing: bool")
                && source.contains("pub cached_count: usize")
                && source.contains(&format!("pub(crate) fn {fn_name}(")),
            "{source_name} must expose content-free snapshot status for runtime proof"
        );
    }
}

#[test]
fn filtering_cache_freezes_browser_snapshot_hits_per_query_frame() {
    let app_state = include_str!("../../src/main_sections/app_state.rs");
    let filtering_cache = include_str!("../../src/app_impl/filtering_cache.rs");
    let preflight_types = include_str!("../../src/main_window_preflight/types.rs");
    let preflight_build = include_str!("../../src/main_window_preflight/build.rs");

    assert!(app_state.contains("pub(crate) struct RootPassiveFrameKey"));
    assert!(app_state.contains("pub(crate) struct RootPassiveFrame"));
    assert!(app_state.contains("root_passive_frame: Option<RootPassiveFrame>"));
    assert!(filtering_cache.contains("fn root_passive_frame_for_current_query("));
    assert!(
        filtering_cache.contains("if frame.key == key")
            && filtering_cache.contains("return frame.clone()"),
        "same-query passive frames should reuse the frozen hit vectors"
    );
    assert!(
        filtering_cache.contains("search_root_browser_tabs_meta(")
            && filtering_cache.contains("search_root_browser_history_meta(")
            && filtering_cache.contains("&root_passive_frame.browser_tab_hits")
            && filtering_cache.contains("&root_passive_frame.browser_history_hits"),
        "browser tab/history hits should flow through the frozen passive frame"
    );
    assert!(
        preflight_types.contains("RootPassiveFrameReceipt")
            && preflight_types.contains("RootPassiveSourceReceipt")
            && preflight_types.contains("root_passive_frame: Option<RootPassiveFrameReceipt>"),
        "preflight state receipts should expose content-free passive frame status"
    );
    assert!(
        preflight_build.contains("build_root_passive_frame_receipt(")
            && preflight_build.contains("root_browser_tabs_snapshot_status()")
            && preflight_build.contains("root_browser_history_snapshot_status()"),
        "runtime proof should be able to wait for browser passive refresh completion"
    );
}

#[test]
fn agentic_root_passive_frame_stability_proof_uses_state_receipts() {
    let proof = include_str!("../../scripts/agentic/root-passive-frame-stability.ts");

    for required in [
        "setFilter",
        "waitFor",
        "stateMatch",
        "getState",
        "mainWindowPreflight",
        "rootPassiveFrame",
        "selectedResultKey",
        "visibleResultKeyFingerprint",
        "enterAction",
        "browserTabs",
        "browserHistory",
        "refreshing",
    ] {
        assert!(
            proof.contains(required),
            "passive-frame runtime proof should contain `{required}`"
        );
    }

    assert!(
        !proof.contains("captureScreenshot") && !proof.contains("simulateClick"),
        "passive-frame proof should stay state-first, not screenshot or mouse based"
    );
}

#[test]
fn jsonl_history_sources_use_mtime_backed_foreground_indexes() {
    for (source_name, source) in [
        ("acp_history", include_str!("../../src/ai/acp/history.rs")),
        (
            "dictation_history",
            include_str!("../../src/dictation/history.rs"),
        ),
    ] {
        assert!(
            source.contains("OnceLock<Mutex<Option<")
                && source.contains("IndexCache")
                && source.contains("history_file_signature(&path)")
                && source.contains("parse_history_entries(&content)")
                && source.contains("return cache.entries.clone()"),
            "{source_name} root-search history reads should reuse an mtime-backed index cache"
        );
        assert!(
            source.contains("invalidate_history_cache();"),
            "{source_name} history writes/deletes must invalidate the root-search index cache"
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
