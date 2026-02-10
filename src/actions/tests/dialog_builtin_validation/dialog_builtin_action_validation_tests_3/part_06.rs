
#[test]
fn builtin_context_has_at_least_4_actions() {
    // Built-in: run, add_shortcut, add_alias, copy_deeplink
    let builtin = ScriptInfo::builtin("Test");
    let count = get_script_context_actions(&builtin).len();
    assert!(
        count >= 4,
        "Builtin context should have at least 4 actions, got {}",
        count
    );
}

#[test]
fn clipboard_text_has_at_least_10_actions() {
    let entry = ClipboardEntryInfo {
        id: "t".into(),
        content_type: ContentType::Text,
        pinned: false,
        preview: "text".into(),
        image_dimensions: None,
        frontmost_app_name: None,
    };
    let count = get_clipboard_history_context_actions(&entry).len();
    assert!(
        count >= 10,
        "Clipboard text context should have at least 10 actions, got {}",
        count
    );
}

#[test]
fn ai_command_bar_has_exactly_12_actions() {
    let count = get_ai_command_bar_actions().len();
    assert_eq!(
        count, 12,
        "AI command bar should have exactly 12 actions, got {}",
        count
    );
}

// =========================================================================
// 36. Score fuzzy match
// =========================================================================

#[test]
fn score_fuzzy_match_subsequence() {
    let action = Action::new(
        "reveal_in_finder",
        "Reveal in Finder",
        Some("Reveal in Finder".to_string()),
        ActionCategory::ScriptContext,
    );
    // "rif" is a subsequence of "reveal in finder"
    let score = ActionsDialog::score_action(&action, "rif");
    assert!(
        score > 0,
        "Fuzzy subsequence 'rif' should match 'reveal in finder', got score {}",
        score
    );
}

#[test]
fn score_fuzzy_no_match() {
    let action = Action::new(
        "run_script",
        "Run Script",
        None,
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}
