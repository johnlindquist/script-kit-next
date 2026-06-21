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

#[test]
fn printable_text_keys_use_input_ladder_not_simulatekey_allowlist_block() {
    for needle in [
        "function isPrintableTextKey",
        "setFilterTextInput",
        "type: \"setFilter\"",
        "currentInput",
        "function postActionReceiptArgs",
        "return isPrintableTextKey(args) ? withoutExpectedSurface(args) : args",
        "const afterArgs = postActionReceiptArgs(args);",
        "isPrintableTextKey(args) && after.target && after.classification === \"blocked-by-missing-primitive\"",
        "if (args.actionKind === \"key\" && !allowedKeys.has(normalizedKey) && !isPrintableTextKey(args))",
        "if (isPrintableTextKey(args)) return \"stdin_command_parsed\";",
        "nativeEscalation: false",
        "noNativeEscalation: !guardWithPreflight.nativeEscalation",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts must route printable text keys like @ through the input ladder: {needle}"
        );
    }
}

#[test]
fn act_exposes_theme_designer_control_setter() {
    for needle in [
        "\"set-theme-control\"",
        "arg === \"--text\" || arg === \"--value\"",
        "set-theme-control requires --control",
        "commands: [{ type: \"setThemeControl\", control: args.control, value: args.value }]",
        "control: args.actionKind === \"set-theme-control\" ? args.control : null",
        "value: args.actionKind === \"set-theme-control\" ? args.value : null",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts must expose Theme Designer control setter marker {needle}"
        );
    }
}

#[test]
fn act_exposes_scoped_submit_intent_and_preflight_only() {
    for needle in [
        "--submit-intent",
        "--allow-submit-reason",
        "--preflight-only",
        "submitIntent",
        "allowSubmitReason",
        "preflightOnly",
        "submit.intent.required",
        "requiredFlags",
        "nextSafeCommand",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts must expose scoped submit proof field {needle}"
        );
    }
}

#[test]
fn act_allows_cmd_enter_agent_chat_only_by_named_intent() {
    for needle in [
        "function isCmdEnter",
        "function isScopedAgentChatRoute",
        "agent-chat-route",
        "cmd-enter-agent-chat-route",
        "allowedBy: \"submitIntent:agent-chat-route\"",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "Cmd+Enter Agent Chat proof must be named and scoped: {needle}"
        );
    }
}

#[test]
fn act_allows_profile_search_enter_only_with_named_intent_reason_and_main_target() {
    for needle in [
        "function isProfileSearchTargetReceipt",
        "function isPlainEnter",
        "function isScopedProfileSearchSelect",
        "\"profile-search-select\"",
        "allowedBy: \"submitIntent:profile-search-select\"",
        "profile-search-row:",
        "submit.reason.required",
        "profile-search-select requires plain Enter on main ProfileSearch with a selected profile row",
        "resolved?.automationId === \"main\"",
        "resolved?.targetKind === \"Main\"",
        "resolved?.surfaceKind === \"ProfileSearch\"",
        "resolved?.semanticSurface === \"profileSearch\"",
        "args.modifiers.length === 0",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts ProfileSearch Enter allowlist must include {needle}"
        );
    }
}

#[test]
fn act_profile_search_selection_uses_scriptlist_post_intent_proof() {
    for needle in [
        "requiresPostIntentTargetProof",
        "profile-search-select",
        "targetArgs: [\"--main\", \"--strict\", \"--surface\", \"ScriptList\"]",
        "expectedSurfaceKind: \"ScriptList\"",
        "expectedAutomationId: \"main\"",
        "!args.preflightOnly && preflight.state === \"dispatched\"",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "ProfileSearch submit must prove return to ScriptList: {needle}"
        );
    }
}

#[test]
fn act_profile_search_submit_is_non_destructive_only_after_scoped_preflight() {
    for needle in [
        "function isNonDestructiveProfileSearchSubmit",
        "preflight.allowedBy === \"submitIntent:profile-search-select\"",
        "isNonDestructiveProfileSearchSubmit(preflight)",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "ProfileSearch submit must be classified non-destructive only through scoped preflight: {needle}"
        );
    }
}

#[test]
fn act_reports_native_footer_activation_gap_without_native_escalation() {
    for needle in [
        "profile-picker-route",
        "native-footer.activation.missing",
        "nativeFooterActivationReceipt",
        "blocked-by-native-escalation-required",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "native footer model-picker proof must fail closed with {needle}"
        );
    }
}

#[test]
fn act_allows_menu_syntax_trigger_accept_by_enter_or_select_semantic_id() {
    for needle in [
        "function isMenuSyntaxTriggerPickerSelected",
        "function isSelectSemanticActivation",
        "return (isPlainEnter(args) || isSelectSemanticActivation(args))",
        "args.actionKind === \"select\"",
        "args.semanticId.length > 0",
        "menu-syntax-trigger-accept requires plain Enter or selectBySemanticId on main ScriptList with a selected menuSyntaxTriggerPicker row",
        "allowedBy: \"submitIntent:menu-syntax-trigger-accept\"",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts menu syntax trigger accept allowlist must include {needle}"
        );
    }
}
