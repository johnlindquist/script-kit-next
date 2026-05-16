//! Source-audit tests for selected-text clipboard restoration.
//!
//! `setSelectedText` temporarily owns the system clipboard so it can paste into
//! the frontmost app. These tests pin the non-text clipboard boundary without
//! running a live Cmd+V in CI.

use super::read_source as read;

const SELECTED_TEXT_PATH: &str = "src/selected_text.rs";

fn selected_text_source() -> String {
    read(SELECTED_TEXT_PATH)
}

#[test]
fn selected_text_snapshots_full_pasteboard_before_mutating_clipboard() {
    let source = selected_text_source();

    let snapshot_idx = source
        .find("let snapshot = PasteboardSnapshot::capture()")
        .expect("set_via_clipboard_fallback must capture a full pasteboard snapshot");
    let write_idx = source
        .find("write_plain_text_to_pasteboard(text)")
        .expect("set_via_clipboard_fallback must write replacement text after snapshot");
    let paste_idx = source
        .find("let paste_result = simulate_paste_with_cg();")
        .expect("set_via_clipboard_fallback must still simulate Cmd+V");

    assert!(
        snapshot_idx < write_idx && write_idx < paste_idx,
        "selected-text replacement must snapshot before mutating clipboard, then paste"
    );
    assert!(
        !source.contains("let original = clipboard.get_text().ok()"),
        "selected-text replacement must not regress to text-only clipboard restore"
    );
}

#[test]
fn pasteboard_snapshot_preserves_every_item_type_data_representation() {
    let source = selected_text_source();
    let snapshot_impl = source
        .split("impl PasteboardSnapshot {")
        .nth(1)
        .expect("PasteboardSnapshot implementation must exist");

    for required in [
        "pasteboardItems",
        "dataForType",
        "std::slice::from_raw_parts(bytes_ptr, byte_len).to_vec()",
        "setData: data forType: ns_type",
        "writeObjects",
        "changeCount",
    ] {
        assert!(
            snapshot_impl.contains(required),
            "PasteboardSnapshot must preserve and restore item/type/data representation: {required}"
        );
    }
}

#[test]
fn pasteboard_restore_failure_is_returned_after_paste_attempt() {
    let source = selected_text_source();
    let fallback_body = source
        .split("fn set_via_clipboard_fallback(text: &str) -> Result<()> {")
        .nth(1)
        .and_then(|rest| rest.split("struct PasteboardSnapshot").next())
        .expect("selected-text fallback body must exist");

    let paste_idx = fallback_body
        .find("paste_result?;")
        .expect("paste result must remain explicit");
    let restore_idx = fallback_body
        .find("restore_result")
        .expect("restore result must be captured");
    let restore_return_idx = fallback_body
        .find(".context(\"Failed to restore original clipboard after selected-text replacement\")")
        .expect("restore failure must be returned explicitly");

    assert!(
        restore_idx < paste_idx && paste_idx < restore_return_idx,
        "restore must be attempted before returning paste errors, and restore failures must remain explicit"
    );
    assert!(
        fallback_body.contains("temporary_change_count"),
        "selected-text restore must record the temporary clipboard change count"
    );
    assert!(
        fallback_body.contains("skipped restore to avoid overwriting external clipboard update"),
        "selected-text restore must make external clipboard changes explicit"
    );
}

#[test]
fn selected_text_clipboard_restore_logs_only_content_light_summary() {
    let source = selected_text_source();
    let fallback_body = source
        .split("fn set_via_clipboard_fallback(text: &str) -> Result<()> {")
        .nth(1)
        .and_then(|rest| rest.split("struct PasteboardSnapshot").next())
        .expect("selected-text fallback body must exist");

    for required in [
        "item_count",
        "type_count",
        "total_bytes",
        "has_text",
        "has_rich_text",
        "has_image",
        "has_file_url",
        "has_other",
    ] {
        assert!(
            fallback_body.contains(required),
            "selected-text restore logs must include content-light summary field: {required}"
        );
    }

    for forbidden in [
        "text = %text",
        "text = ?text",
        "original_text",
        "type_name =",
        "data =",
    ] {
        assert!(
            !fallback_body.contains(forbidden),
            "selected-text restore logs must not expose clipboard or replacement contents: {forbidden}"
        );
    }
}
