use script_kit_gpui::ai::{
    derive_acp_result_cards_from_assistant_message, AcpResultArtifactKind,
    RESULT_CARD_MAX_ARTIFACTS, RESULT_CARD_MAX_FOLLOW_UPS,
};

#[test]
fn result_cards_extract_safe_existing_files_and_http_links() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let artifact_path = dir.path().join("report.md");
    std::fs::write(&artifact_path, "# Report\n").expect("write artifact");

    let message = format!(
        "Created [Report]({}) and [Docs](https://example.com/docs).\n\
         NEXT_ACTIONS:\n\
         - Summarize the report\n\
         - Open the generated docs",
        artifact_path.display()
    );

    let cards = derive_acp_result_cards_from_assistant_message(&message);

    assert_eq!(cards.artifacts.len(), 2);
    assert_eq!(cards.artifacts[0].kind, AcpResultArtifactKind::File);
    assert_eq!(cards.artifacts[0].title, "Report");
    assert_eq!(
        cards.artifacts[0].target,
        artifact_path
            .canonicalize()
            .expect("canonical artifact")
            .to_string_lossy()
    );
    assert_eq!(cards.artifacts[1].kind, AcpResultArtifactKind::Link);
    assert_eq!(cards.artifacts[1].target, "https://example.com/docs");
    assert_eq!(cards.follow_ups.len(), 2);
    assert_eq!(cards.follow_ups[0].prompt, "Summarize the report");
}

#[test]
fn result_cards_reject_unsafe_or_missing_artifacts() {
    let message = "\
        [Javascript](javascript:alert(1))\n\
        [FTP](ftp://example.com/file)\n\
        [Relative](../secret.txt)\n\
        [Missing](/tmp/script-kit-gpui-result-card-missing-file)\n\
        [Good](http://example.com/ok)";

    let cards = derive_acp_result_cards_from_assistant_message(message);

    assert_eq!(cards.artifacts.len(), 1);
    assert_eq!(cards.artifacts[0].kind, AcpResultArtifactKind::Link);
    assert_eq!(cards.artifacts[0].target, "http://example.com/ok");
}

#[test]
fn result_cards_dedupe_and_cap_artifacts() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let first = dir.path().join("one.txt");
    let second = dir.path().join("two.txt");
    let third = dir.path().join("three.txt");
    let fourth = dir.path().join("four.txt");
    for path in [&first, &second, &third, &fourth] {
        std::fs::write(path, "artifact").expect("write artifact");
    }

    let message = format!(
        "[One]({}) [One Duplicate]({}) [Two]({}) [Three]({}) [Four]({})",
        first.display(),
        first.display(),
        second.display(),
        third.display(),
        fourth.display()
    );

    let cards = derive_acp_result_cards_from_assistant_message(&message);

    assert_eq!(cards.artifacts.len(), RESULT_CARD_MAX_ARTIFACTS);
    assert_eq!(cards.artifacts[0].title, "One");
    assert_eq!(cards.artifacts[1].title, "Two");
    assert_eq!(cards.artifacts[2].title, "Three");
}

#[test]
fn result_cards_cap_follow_ups_and_refuse_reserved_action_ids() {
    let message = "\
        NEXT_ACTIONS:\n\
        - action:delete-everything\n\
        - Review the diff\n\
        - sdk:dangerousBuiltin\n\
        - Run the focused tests\n\
        - Ship it";

    let cards = derive_acp_result_cards_from_assistant_message(message);

    assert_eq!(cards.follow_ups.len(), RESULT_CARD_MAX_FOLLOW_UPS);
    assert_eq!(cards.follow_ups[0].prompt, "Review the diff");
    assert_eq!(cards.follow_ups[1].prompt, "Run the focused tests");
}

#[test]
fn result_cards_sanitize_display_text() {
    let message = "\
        [A     label     with     extra     whitespace](https://example.com)\n\
        NEXT_ACTIONS:\n\
        -   Summarize     this      clearly";

    let cards = derive_acp_result_cards_from_assistant_message(message);

    assert_eq!(cards.artifacts[0].title, "A label with extra whitespace");
    assert_eq!(cards.follow_ups[0].prompt, "Summarize this clearly");
}
