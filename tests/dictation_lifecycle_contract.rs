use script_kit_gpui::dictation::resolve_final_or_partial_transcript;

#[test]
fn final_transcript_wins_over_partial_transcript() {
    let resolution = resolve_final_or_partial_transcript("final text", Some("partial text"));

    assert_eq!(resolution.transcript.as_deref(), Some("final text"));
    assert!(!resolution.used_partial_fallback);
    assert_eq!(resolution.final_len, "final text".len());
    assert_eq!(resolution.partial_len, Some("partial text".len()));
}

#[test]
fn partial_transcript_is_used_when_final_is_empty() {
    let resolution = resolve_final_or_partial_transcript("   ", Some("partial text"));

    assert_eq!(resolution.transcript.as_deref(), Some("partial text"));
    assert!(resolution.used_partial_fallback);
    assert_eq!(resolution.final_len, 3);
    assert_eq!(resolution.partial_len, Some("partial text".len()));
}

#[test]
fn no_transcript_is_returned_when_final_and_partial_are_empty() {
    let resolution = resolve_final_or_partial_transcript("", Some("  "));

    assert_eq!(resolution.transcript, None);
    assert!(!resolution.used_partial_fallback);
    assert_eq!(resolution.final_len, 0);
    assert_eq!(resolution.partial_len, Some(2));
}
