use super::*;

#[test]
fn test_jsonl_reader_skips_empty_lines() {
    use std::io::Cursor;

    let jsonl = "\n{\"type\":\"beep\"}\n\n{\"type\":\"show\"}\n";
    let cursor = Cursor::new(jsonl);
    let mut reader = JsonlReader::new(cursor);

    // First message should be beep (skipping initial empty line)
    let msg1 = reader.next_message().unwrap();
    assert!(matches!(msg1, Some(Message::Beep {})));

    // Second message should be show (skipping intermediate empty lines)
    let msg2 = reader.next_message().unwrap();
    assert!(matches!(msg2, Some(Message::Show {})));

    // Should be EOF
    let msg3 = reader.next_message().unwrap();
    assert!(msg3.is_none());
}

#[test]
fn test_jsonl_reader_graceful_skips_unknown() {
    use std::io::Cursor;

    let jsonl = r#"{"type":"unknownType","id":"1"}
{"type":"beep"}
{"type":"anotherUnknown","data":"test"}
{"type":"show"}
"#;
    let cursor = Cursor::new(jsonl);
    let mut reader = JsonlReader::new(cursor);

    // Should skip unknownType and return beep
    let msg1 = reader.next_message_graceful().unwrap();
    assert!(matches!(msg1, Some(Message::Beep {})));

    // Should skip anotherUnknown and return show
    let msg2 = reader.next_message_graceful().unwrap();
    assert!(matches!(msg2, Some(Message::Show {})));

    // Should be EOF
    let msg3 = reader.next_message_graceful().unwrap();
    assert!(msg3.is_none());
}

#[test]
fn test_jsonl_reader_reports_invalid_payload() {
    use std::io::Cursor;

    let jsonl = r#"{"type":"arg","id":"1"}
{"type":"beep"}
"#;
    let cursor = Cursor::new(jsonl);
    let mut reader = JsonlReader::new(cursor);
    let mut issues: Vec<ParseIssue> = Vec::new();

    let msg = reader
        .next_message_graceful_with_handler(|issue| issues.push(issue))
        .unwrap();

    assert!(matches!(msg, Some(Message::Beep {})));
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].kind, ParseIssueKind::InvalidPayload);
    assert_eq!(issues[0].message_type.as_deref(), Some("arg"));
    assert!(issues[0]
        .error
        .as_deref()
        .unwrap_or("")
        .contains("placeholder"));
}

#[test]
fn test_next_message_returns_error_when_line_exceeds_limit() {
    use std::io::Cursor;

    let huge_html = "x".repeat(70_000);
    let jsonl = format!(
        r#"{{"type":"div","id":"1","html":"{}"}}
"#,
        huge_html
    );
    let cursor = Cursor::new(jsonl);
    let mut reader = JsonlReader::new(cursor);

    let result = reader.next_message();
    assert!(result.is_err(), "Expected oversized JSONL line to fail");
}

#[test]
fn test_next_message_graceful_skips_oversized_line_and_recovers() {
    use std::io::Cursor;

    let huge_html = "x".repeat(70_000);
    let jsonl = format!(
        r#"{{"type":"div","id":"1","html":"{}"}}
{{"type":"beep"}}
"#,
        huge_html
    );
    let cursor = Cursor::new(jsonl);
    let mut reader = JsonlReader::new(cursor);
    let mut issues = Vec::new();

    let msg = reader
        .next_message_graceful_with_handler(|issue| issues.push(issue))
        .expect("Reader should recover after oversized line");

    assert!(
        matches!(msg, Some(Message::Beep {})),
        "Expected oversized line to be skipped"
    );
    assert_eq!(
        issues.len(),
        1,
        "Expected one parse issue for oversized line"
    );
    assert_eq!(issues[0].kind, ParseIssueKind::LineTooLong);
    assert!(issues[0].raw_len > MAX_PROTOCOL_LINE_BYTES);
}
