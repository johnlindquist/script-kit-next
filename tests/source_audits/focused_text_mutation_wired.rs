const MUTATION: &str = include_str!("../../src/platform/accessibility/mutation.rs");
const CLIPBOARD: &str = include_str!("../../src/platform/accessibility/clipboard.rs");

#[test]
fn mutation_api_exposes_replace_append_copy_boundaries() {
    assert!(MUTATION.contains("replace_focused_text"));
    assert!(MUTATION.contains("append_focused_text"));
    assert!(MUTATION.contains("copy_text_output"));
    assert!(MUTATION.contains("TextMutationOptions"));
}

#[test]
fn clipboard_helper_exposes_snapshot_and_change_count() {
    assert!(CLIPBOARD.contains("PasteboardSnapshot"));
    assert!(CLIPBOARD.contains("write_plain_text_to_pasteboard"));
    assert!(CLIPBOARD.contains("general_pasteboard_change_count"));
}
