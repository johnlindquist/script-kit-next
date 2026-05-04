//! Source-level contract for the state-first surface navigator.
//!
//! The navigator is the agent-facing path for warm-session surface entry,
//! safe non-submitting interaction, and strict screenshot-library capture.

const NAVIGATOR: &str = include_str!("../scripts/agentic/surface-navigator.ts");

#[test]
fn surface_navigator_script_exists_and_uses_matrix() {
    assert!(
        NAVIGATOR.contains("FILTERABLE_SURFACE_MATRIX"),
        "surface navigator must reuse the stable matrix instead of forking surface data"
    );
    assert!(
        NAVIGATOR.contains("selectedCases("),
        "surface navigator must use the matrix case selector"
    );
    assert!(
        NAVIGATOR.contains("selectedAttachedPopupCases("),
        "surface navigator must reuse the attached-popup matrix selector"
    );
}

#[test]
fn surface_navigator_exposes_agent_facing_flags() {
    for flag in [
        "--list",
        "--case",
        "--capture",
        "--out-dir",
        "--manifest",
        "--fresh-per-case",
        "--interact",
        "--keep-session",
        "--json",
    ] {
        assert!(
            NAVIGATOR.contains(flag),
            "surface navigator must support flag {flag}"
        );
    }
}

#[test]
fn surface_navigator_can_isolate_screenshot_cases() {
    assert!(
        NAVIGATOR.contains("freshPerCase") && NAVIGATOR.contains("--fresh-per-case"),
        "screenshot-library sweeps need an isolated per-case mode for visual correctness"
    );
    assert!(
        NAVIGATOR.contains("--fresh-per-case cannot be combined with --keep-session"),
        "fresh per-case mode must not leak multiple retained sessions"
    );
}

#[test]
fn surface_navigator_supports_all_active_group_without_changing_default() {
    assert!(
        NAVIGATOR.contains("\"filterable-main\" | \"attached-popup\"")
            && NAVIGATOR.contains("| \"all-active\""),
        "surface navigator must expose all-active alongside existing groups"
    );
    assert!(
        NAVIGATOR.contains("argValue(\"--group\", \"filterable-main\")"),
        "filterable-main must remain the default group"
    );
}

#[test]
fn surface_navigator_exposes_manifest_flag() {
    assert!(
        NAVIGATOR.contains("--manifest"),
        "surface navigator must support a manifest path for durable image-library proof"
    );
}

#[test]
fn surface_navigator_uses_warm_session_protocol_path() {
    assert!(
        NAVIGATOR.contains("sessionStart(session)")
            || NAVIGATOR.contains("sessionStart(opts.session)"),
        "surface navigator must start or reuse sessions through session.sh helpers"
    );
    assert!(
        NAVIGATOR.contains("enterFilterableSurface("),
        "surface entry must use parse-receipted real entry commands"
    );
    assert!(
        NAVIGATOR.contains("waitForPromptType("),
        "surface readiness must poll getState for the expected promptType"
    );
    assert!(
        NAVIGATOR.contains("{ type: \"show\" }") && NAVIGATOR.contains("Bun.sleep(300)"),
        "navigator must reveal and settle the app window before collecting visible-state receipts"
    );
    assert!(
        NAVIGATOR.contains("getStateAndElements("),
        "navigator must collect state and elements receipts before interaction"
    );
    assert!(
        !NAVIGATOR.contains("mkfifo") && !NAVIGATOR.contains("macos-input.ts"),
        "stable matrix navigation must not use manual FIFO code or native input"
    );
}

#[test]
fn surface_navigator_safe_interaction_is_non_submitting_batch() {
    assert!(
        NAVIGATOR.contains("type: \"batch\""),
        "safe interaction must use protocol batch"
    );
    assert!(
        NAVIGATOR.contains("type: \"selectBySemanticId\""),
        "safe interaction must select by semantic id"
    );
    assert!(
        NAVIGATOR.contains("submit: false"),
        "safe interaction must not submit a selected row"
    );
    assert!(
        NAVIGATOR.contains("selectionReceipt.success !== true"),
        "safe interaction must fail closed when semantic selection fails"
    );
}

#[test]
fn surface_navigator_captures_strict_image_library_outputs() {
    assert!(
        NAVIGATOR.contains("DEFAULT_OUT_DIR = \".notes/image-library\""),
        "image-library root must default to .notes/image-library"
    );
    assert!(
        NAVIGATOR.contains("scripts/agentic/verify-shot.ts"),
        "navigator must reuse verify-shot screenshot auditing"
    );
    assert!(
        NAVIGATOR.contains("\"--strict-window\""),
        "navigator captures must require strict window identity"
    );
    assert!(
        NAVIGATOR.contains("\"--target-json\""),
        "strict capture must pass an exact automation target"
    );
}
