//! Source-level contract for the broad agentic user-story audit runner.

const USER_STORY_AUDIT: &str = include_str!("../scripts/agentic/user-story-audit.ts");

#[test]
fn user_story_audit_selects_one_hundred_new_stress_recipes() {
    for token in [
        "agentic-100-user-story-ux-audit",
        "let limit = 100",
        "ALREADY_EXERCISED_THIS_THREAD",
        "skippedAlreadyExercisedThisThread",
        "extractStressRecipes",
        ".filter((recipe) => includeKnown || !ALREADY_EXERCISED_THIS_THREAD.has(recipe))",
    ] {
        assert!(
            USER_STORY_AUDIT.contains(token),
            "100-story audit runner must pin selection behavior with {token}"
        );
    }
}

#[test]
fn user_story_audit_records_fail_closed_blocked_and_runtime_statuses() {
    for token in [
        "\"pass\" | \"fail_closed\" | \"blocked_precondition\" | \"runtime_failure\" | \"timeout\"",
        "failureCode(parsed) === \"insufficient_target_count\"",
        "failureMode === \"fail_closed\"",
        "code.startsWith(\"missing_\")",
        "warnings.some((warning: string) => warning.startsWith(\"file_linear:\"))",
        "SCRIPT_KIT_AGENTIC_AUDIT",
    ] {
        assert!(
            USER_STORY_AUDIT.contains(token),
            "100-story audit runner must classify non-green outcomes with {token}"
        );
    }
}

#[test]
fn user_story_audit_writes_replayable_test_output_artifact() {
    for token in [
        ".test-output",
        "agentic-100-user-story-audit-",
        "command",
        "outputPreview",
        "\"--reclassify\"",
        "reclassifiedAt",
    ] {
        assert!(
            USER_STORY_AUDIT.contains(token),
            "100-story audit runner must leave replayable artifacts with {token}"
        );
    }
}
