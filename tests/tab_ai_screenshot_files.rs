//! Tests for the Tab AI screenshot-to-file bridge.
//!
//! Validates screenshot temp file cleanup and retention cap.

use script_kit_gpui::ai::{
    cleanup_old_tab_ai_screenshot_files_in_dir, tab_ai_screenshot_prefix,
    TAB_AI_SCREENSHOT_MAX_KEEP,
};

#[test]
fn screenshot_file_bridge_writes_png_and_caps_retention_at_ten() {
    let tmp = tempfile::tempdir().expect("must create temp dir");
    let dir = tmp.path();
    let prefix = tab_ai_screenshot_prefix();

    // Create 15 fake screenshot files with staggered modification times
    for i in 0..15 {
        let name = format!("{prefix}2026{i:04}T000000Z-99999.png");
        let path = dir.join(&name);
        std::fs::write(&path, format!("fake-png-{i}")).expect("must write");

        // Set modification time to ensure ordering: newer files have later times
        let mtime = filetime::FileTime::from_unix_time(1700000000 + i * 60, 0);
        filetime::set_file_mtime(&path, mtime).expect("must set mtime");
    }

    // Verify we have 15 files
    let count_before = count_screenshot_files(dir, prefix);
    assert_eq!(
        count_before, 15,
        "should have 15 screenshot files before cleanup"
    );

    // Run cleanup
    cleanup_old_tab_ai_screenshot_files_in_dir(dir, TAB_AI_SCREENSHOT_MAX_KEEP)
        .expect("cleanup must succeed");

    // Verify we now have at most TAB_AI_SCREENSHOT_MAX_KEEP files
    let count_after = count_screenshot_files(dir, prefix);
    assert_eq!(
        count_after, TAB_AI_SCREENSHOT_MAX_KEEP,
        "cleanup must cap files at TAB_AI_SCREENSHOT_MAX_KEEP"
    );

    // The newest files should be the ones retained
    let mut remaining: Vec<String> = std::fs::read_dir(dir)
        .expect("must read dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(prefix) && name.ends_with(".png"))
        .collect();
    remaining.sort();

    // Files 5-14 should be retained (the newest 10)
    for i in 5..15 {
        let expected_name = format!("{prefix}2026{i:04}T000000Z-99999.png");
        assert!(
            remaining.contains(&expected_name),
            "newest file {expected_name} should be retained"
        );
    }
}

#[test]
fn cleanup_is_noop_when_under_limit() {
    let tmp = tempfile::tempdir().expect("must create temp dir");
    let dir = tmp.path();
    let prefix = tab_ai_screenshot_prefix();

    // Create only 3 files
    for i in 0..3 {
        let name = format!("{prefix}2026{i:04}T000000Z-99999.png");
        std::fs::write(dir.join(&name), "fake").expect("must write");
    }

    cleanup_old_tab_ai_screenshot_files_in_dir(dir, TAB_AI_SCREENSHOT_MAX_KEEP)
        .expect("cleanup must succeed");

    let count = count_screenshot_files(dir, prefix);
    assert_eq!(count, 3, "no files should be removed when under limit");
}

#[test]
fn cleanup_ignores_non_screenshot_files() {
    let tmp = tempfile::tempdir().expect("must create temp dir");
    let dir = tmp.path();
    let prefix = tab_ai_screenshot_prefix();

    // Create 12 screenshot files
    for i in 0..12 {
        let name = format!("{prefix}2026{i:04}T000000Z-99999.png");
        let path = dir.join(&name);
        std::fs::write(&path, "fake").expect("must write");
        let mtime = filetime::FileTime::from_unix_time(1700000000 + i * 60, 0);
        filetime::set_file_mtime(&path, mtime).expect("must set mtime");
    }

    // Also create a non-screenshot file
    std::fs::write(dir.join("other-file.txt"), "not a screenshot").expect("must write");

    cleanup_old_tab_ai_screenshot_files_in_dir(dir, TAB_AI_SCREENSHOT_MAX_KEEP)
        .expect("cleanup must succeed");

    // Only screenshot files should have been cleaned up
    assert_eq!(
        count_screenshot_files(dir, prefix),
        TAB_AI_SCREENSHOT_MAX_KEEP
    );
    // Non-screenshot file should still exist
    assert!(
        dir.join("other-file.txt").exists(),
        "non-screenshot file must not be removed"
    );
}

#[test]
fn cleanup_handles_empty_dir() {
    let tmp = tempfile::tempdir().expect("must create temp dir");
    cleanup_old_tab_ai_screenshot_files_in_dir(tmp.path(), TAB_AI_SCREENSHOT_MAX_KEEP)
        .expect("cleanup of empty dir must succeed");
}

#[test]
fn cleanup_handles_nonexistent_dir() {
    let nonexistent = std::path::Path::new("/tmp/nonexistent-tab-ai-test-dir-12345");
    cleanup_old_tab_ai_screenshot_files_in_dir(nonexistent, TAB_AI_SCREENSHOT_MAX_KEEP)
        .expect("cleanup of nonexistent dir must succeed");
}

fn count_screenshot_files(dir: &std::path::Path, prefix: &str) -> usize {
    std::fs::read_dir(dir)
        .expect("must read dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.starts_with(prefix) && name.ends_with(".png")
        })
        .count()
}
