//! Source-level contract for twelfth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_twelve_recipes() {
    for name in [
        "menu-syntax-ambiguity-diagnostics-stress",
        "ime-composition-input-boundary-stress",
        "accessibility-selected-text-fallback-stress",
    ] {
        assert!(
            INDEX.contains(&format!("name: \"{name}\"")),
            "help --json must advertise {name}"
        );
        assert!(
            INDEX.contains(&format!("case \"{name}\"")),
            "index.ts must route {name}"
        );
    }
    for function_name in [
        "runMenuSyntaxAmbiguityDiagnosticsStressScenario",
        "runImeCompositionInputBoundaryStressScenario",
        "runAccessibilitySelectedTextFallbackStressScenario",
    ] {
        assert!(
            INDEX.contains(function_name) || SCENARIO.contains(function_name),
            "loop-twelve function {function_name} must be wired"
        );
    }
}

#[test]
fn menu_syntax_ambiguity_pins_parse_diagnostics_selection_and_no_execute_guard() {
    for token in [
        "menu-syntax-ambiguity-diagnostics-stress",
        "menuSyntaxAmbiguity",
        "missing_menu_syntax_ambiguity_diagnostics_receipt",
        "menuSyntaxDiagnostics",
        "powerSyntaxMode",
        "parsedFragments",
        "skippedMalformedFragments",
        "ambiguityReasons",
        "tolerantDiagnosticsVisible",
        "selectedCommandId",
        "selectedSemanticId",
        "selectedSourceSurface",
        "fallbackRowUsed",
        "ambiguousParseBlockedExecution",
        "accidentalActionExecuted",
        "submittedCommandId",
        "selected_text_only",
        "implicit_submit",
        "file_linear:menu_syntax_ambiguity_diagnostics_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "menu syntax ambiguity stress must pin {token}"
        );
    }
}

#[test]
fn ime_composition_pins_lifecycle_premature_action_guards_and_committed_semantics() {
    for token in [
        "ime-composition-input-boundary-stress",
        "imeCompositionBoundary",
        "missing_ime_composition_input_boundary_receipt",
        "input.compositionBoundary",
        "filterInput",
        "promptInput",
        "acpComposer",
        "compositionStart",
        "compositionUpdateEvents",
        "compositionCommit",
        "committedText",
        "preeditTextPreserved",
        "enterDuringCompositionSubmitted",
        "actionsOpenedDuringComposition",
        "filterCommittedBeforeCompositionEnd",
        "acpMessageSentBeforeCompositionEnd",
        "finalFilterText",
        "finalPromptValue",
        "finalComposerText",
        "cursorRangeAfterCommit",
        "native_input_without_composition_receipt",
        "file_linear:ime_composition_input_boundary_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "IME composition stress must pin {token}"
        );
    }
}

#[test]
fn accessibility_selected_text_fallback_pins_permission_stale_context_redaction_and_safe_disable() {
    for token in [
        "accessibility-selected-text-fallback-stress",
        "accessibilitySelectedTextFallback",
        "missing_accessibility_selected_text_fallback_receipt",
        "platform.selectedTextFallback",
        "permissionMatrix",
        "accessibilityGranted",
        "screenRecordingGranted",
        "selectedTextProvider",
        "providerDeniedReason",
        "staleContextGuard",
        "frontmostAppBefore",
        "frontmostAppAfter",
        "selectedTextGeneration",
        "staleSelectedTextRejected",
        "staleFrontmostContextRejected",
        "privateTextRedacted",
        "rawSelectedTextLogged",
        "actionPayloadContainsRawText",
        "fallbackSource",
        "actionDisabledWhenUnsafe",
        "permission_prompt_side_effect",
        "file_linear:accessibility_selected_text_fallback_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "selected-text fallback stress must pin {token}"
        );
    }
}

#[test]
fn canonical_skill_and_verification_docs_teach_loop_twelve_boundaries() {
    for token in [
        "menu syntax ambiguity",
        "IME composition",
        "selected-text fallback",
        "no accidental execution",
        "no premature submit",
        "agentic_loop_twelve_contract",
    ] {
        assert!(
            SKILL.contains(token) || AUTOMATION.contains(token) || VERIFICATION.contains(token),
            "loop-twelve docs and skill must teach {token}"
        );
    }
}
