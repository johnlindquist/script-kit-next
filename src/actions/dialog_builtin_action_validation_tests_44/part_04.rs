
#[test]
fn note_switcher_no_preview_shows_char_count() {
    let notes = vec![NoteSwitcherNoteInfo {
        id: "abc".into(),
        title: "T".into(),
        char_count: 42,
        is_current: false,
        is_pinned: false,
        preview: "".into(),
        relative_time: "".into(),
    }];
    let actions = get_note_switcher_actions(&notes);
    assert_eq!(actions[0].description, Some("42 chars".to_string()));
}

// =========== 28. to_deeplink_name: various edge cases ===========

#[test]
fn to_deeplink_name_uppercase_to_lower() {
    assert_eq!(to_deeplink_name("HELLO"), "hello");
}

#[test]
fn to_deeplink_name_preserves_numbers() {
    assert_eq!(to_deeplink_name("test123"), "test123");
}

#[test]
fn to_deeplink_name_multiple_special_chars_collapse() {
    assert_eq!(to_deeplink_name("a!!!b"), "a-b");
}

#[test]
fn to_deeplink_name_leading_trailing_special_removed() {
    assert_eq!(to_deeplink_name("---hello---"), "hello");
}

// =========== 29. score_action: various match type scores ===========

#[test]
fn score_action_prefix_match_100() {
    let a = Action::new("id", "copy path", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&a, "copy");
    assert_eq!(score, 100);
}

#[test]
fn score_action_contains_match_50() {
    let a = Action::new("id", "my copy action", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&a, "copy");
    assert_eq!(score, 50);
}

#[test]
fn score_action_fuzzy_match_25() {
    let a = Action::new("id", "clipboard", None, ActionCategory::ScriptContext);
    // "cpd" is a subsequence of "clipboard" (c-l-i-p-b-o-a-r-d)
    // c..p..d - wait, let me verify: c(lipboar)d - not quite
    // "cbd" = c(lip)b(oar)d - that works
    let score = ActionsDialog::score_action(&a, "cbd");
    assert_eq!(score, 25);
}

#[test]
fn score_action_no_match_0() {
    let a = Action::new("id", "abc title", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&a, "xyz");
    assert_eq!(score, 0);
}

// =========== 30. fuzzy_match: various patterns ===========

#[test]
fn fuzzy_match_full_string() {
    assert!(ActionsDialog::fuzzy_match("hello", "hello"));
}

#[test]
fn fuzzy_match_subsequence() {
    assert!(ActionsDialog::fuzzy_match("hello world", "hwd"));
}

#[test]
fn fuzzy_match_empty_needle_matches() {
    assert!(ActionsDialog::fuzzy_match("anything", ""));
}

#[test]
fn fuzzy_match_reversed_fails() {
    assert!(!ActionsDialog::fuzzy_match("abc", "cba"));
}
