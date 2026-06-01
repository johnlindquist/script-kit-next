use std::fs;

const MEASURE: &str = include_str!("../scripts/devtools/measure.ts");
const MEDIA: &str = include_str!("../scripts/devtools/media.ts");
const DICTATION: &str = include_str!("../scripts/devtools/dictation.ts");
const DEVTOOLS_SKILL: &str = include_str!("../.agents/skills/script-kit-devtools/SKILL.md");
const COVERAGE_AUDIT: &str =
    include_str!("../.agents/skills/script-kit-devtools/references/devtools-coverage-audit.md");
const COVERAGE_MAP: &str =
    include_str!("../.agents/skills/script-kit-devtools/references/devtools-api-coverage-map.md");

#[test]
fn measure_cli_reports_devtools_measure_schema_and_gaps() {
    for needle in [
        "script-kit-devtools.measure",
        "--inspect",
        "--coverage",
        "--surface",
        "availableMeasurements",
        "plannedMeasurements",
        "targetSurfaceMatch",
        "targetMatchesSurface",
        "inspect receipt target does not match requested surface",
        "missingRuntimePrimitives",
        "failClosed",
        "target-scoped layout info",
        "text and selection bounds",
        "scroll anchor and viewport receipts",
        "focus owner and shortcut registry receipts",
    ] {
        assert!(
            MEASURE.contains(needle),
            "devtools.measure must expose measurable schema/gap field: {needle}"
        );
    }

    assert!(
        !MEASURE.contains("scripts/agentic/index.ts"),
        "devtools.measure must not call the agentic recipe catalog"
    );
}

#[test]
fn media_cli_pins_passive_dictation_inspection_contract() {
    for needle in [
        "script-kit-devtools.media.inspect",
        "blocked-by-missing-primitive",
        "passive microphone permission status",
        "microphone device snapshot",
        "model readiness generation",
        "recording state generation",
        "audio level metrics",
        "target delivery generation",
        "transcript fingerprint",
        "cursor insertion range",
        "wrong-target refusal receipt",
        "wrongTargetRefusal",
        "noDeliveryAttempted",
        "requestedTargetLabelFingerprint",
        "--expect-refusal",
        "hotkey binding snapshot",
        "media cleanup receipt",
        "noMicrophoneCaptureRequired",
        "noSystemSettingsRequired",
        "noTccMutationRequired",
    ] {
        assert!(
            MEDIA.contains(needle) || DICTATION.contains(needle),
            "devtools.media.inspect must preserve passive Dictation field: {needle}"
        );
    }

    assert!(
        !MEDIA.contains("scripts/agentic/index.ts"),
        "devtools.media.inspect must not call the agentic recipe catalog"
    );
}

#[test]
fn docs_promote_measure_and_media_as_next_primitives() {
    for needle in [
        "devtools.measure",
        "devtools.media.inspect",
        "layout, text fit, scroll, overlap, contrast",
        "passive microphone permission",
        "before claiming live Dictation bugs are verifiable",
    ] {
        assert!(
            DEVTOOLS_SKILL.contains(needle)
                || COVERAGE_AUDIT.contains(needle)
                || COVERAGE_MAP.contains(needle),
            "DevTools docs must route next API work through {needle}"
        );
    }
}

#[test]
fn devtools_measure_media_artifacts_are_checked_in() {
    for path in ["scripts/devtools/measure.ts", "scripts/devtools/media.ts"] {
        assert!(
            fs::metadata(path).is_ok(),
            "expected checked-in DevTools primitive at {path}"
        );
    }
}
