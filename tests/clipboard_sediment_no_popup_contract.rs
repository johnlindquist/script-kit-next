use std::path::Path;

fn read_source(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

/// Clipboard sediment may track copied content for brain storage, but that path
/// must not resurrect the removed post-copy popup. The tracker may bridge the
/// quiet "Kept" HUD only; annotate/reject popup behavior must not be
/// reintroduced by satisfying stale quick-menu tests.
#[test]
fn post_copy_tracker_does_not_own_popup_ui() {
    let post_copy = read_source("src/clipboard_history/post_copy.rs");

    for forbidden in [
        "open_window",
        "register_attached_popup",
        "inline_popup_window_options",
        "CGEventTap",
        "OpenQuickMenu",
        "PostCopyQuickMenuWindow",
        "WindowHandle",
    ] {
        assert!(
            !post_copy.contains(forbidden),
            "post-copy tracker must stay popup-free; found `{forbidden}`"
        );
    }

    assert!(
        post_copy.contains("pub fn install_post_copy_tracker("),
        "post-copy lane should install only the tracker/HUD bridge"
    );
    assert!(
        post_copy.contains("pub fn notify_text_copy_stored(_entry_id: &str) {}"),
        "copy-stored notification should remain a UI-free no-op; sediment owns brain writes"
    );
    assert!(
        !Path::new("src/clipboard_history/tap_window.rs").exists(),
        "the removed tap-window state machine must not come back for copy tracking"
    );
}
