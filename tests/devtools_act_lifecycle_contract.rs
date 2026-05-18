const ACT_TS: &str = include_str!("../scripts/devtools/act.ts");

#[test]
fn act_models_submit_lifecycle_as_named_states() {
    for needle in [
        "type SubmitLifecycleState",
        "\"not-submit\"",
        "\"blocked-before-dispatch\"",
        "\"dispatched\"",
        "\"source-live\"",
        "\"source-closed-parent-live\"",
        "\"failed\"",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts must include submit lifecycle state marker {needle}"
        );
    }
}
#[test]
fn actions_dialog_submit_requires_selected_choice_row() {
    for needle in [
        "submit requires selected ActionsDialog choice:* row",
        "selectedActionIdFromSemanticId",
        "isActionsDialogTargetReceipt",
        "submitPreflight",
        "submitPreflightSelectedSemanticId",
        "submitAttempted",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts submit preflight must include {needle}"
        );
    }
}

#[test]
fn act_models_post_action_lifecycle_as_named_states() {
    for needle in [
        "type PostActionLifecycleState",
        "\"not-lifecycle-sensitive\"",
        "\"dismissed\"",
        "\"source-closed-parent-live\"",
        "postActionLifecycle",
        "dismissLifecycle",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts must include post-action lifecycle marker {needle}"
        );
    }
}

#[test]
fn submit_after_state_inspects_parent_when_source_closes() {
    for needle in [
        "inspectParentAfterSubmit",
        "\"--main\", \"--surface\", \"ScriptList\"",
        "resolveSubmitLifecycleAfterAction",
        "source-closed-parent-live",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts must resolve post-submit parent lifecycle with {needle}"
        );
    }
}

#[test]
fn escape_and_cmd_keys_are_dismiss_like_without_submit_preflight() {
    for needle in [
        "function isDismissLike",
        "normalizedKey === \"escape\"",
        "normalizedKey === \"esc\"",
        "normalizedKey === \"k\" && args.modifiers.includes(\"cmd\")",
        "normalizedKey === \"w\" && args.modifiers.includes(\"cmd\")",
        "resolvePostActionLifecycle",
        "inspectParentAfterAction",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts dismiss lifecycle must include {needle}"
        );
    }
}

#[test]
fn closed_source_parent_live_is_ok_for_dismiss_not_target_ambiguity() {
    for needle in [
        "postActionLifecycle.state === \"source-live\"",
        "postActionLifecycle.state === \"source-closed-parent-live\"",
        "return \"ok\";",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts classify must accept dismiss source close with parent live: {needle}"
        );
    }
}

#[test]
fn parse_timeouts_and_errors_are_failed_actions() {
    for needle in [
        "actionReceipt.parseOutcome === \"timeout\"",
        "actionReceipt.parseOutcome === \"parseError\"",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts actionFailed must treat parse failure as failed: {needle}"
        );
    }
}
