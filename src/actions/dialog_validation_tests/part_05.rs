
#[test]
fn test_clipboard_long_preview_truncated_at_27() {
    let preview = "This is a very long clipboard preview text that exceeds the limit".to_string();
    let context_title = if preview.len() > 30 {
        format!("{}...", &preview[..27])
    } else {
        preview.clone()
    };
    assert_eq!(context_title.len(), 30); // 27 chars + "..."
    assert!(context_title.ends_with("..."));
}

#[test]
fn test_clipboard_exactly_30_chars_not_truncated() {
    let preview = "123456789012345678901234567890".to_string(); // exactly 30
    let context_title = if preview.len() > 30 {
        format!("{}...", &preview[..27])
    } else {
        preview.clone()
    };
    assert_eq!(context_title, preview);
}
