use std::fs;

const COVERAGE: &str = include_str!("../scripts/devtools/coverage.ts");
const DEVTOOLS_SKILL: &str = include_str!("../.agents/skills/script-kit-devtools/SKILL.md");
const COVERAGE_MAP: &str =
    include_str!("../.agents/skills/script-kit-devtools/references/devtools-api-coverage-map.md");
const COVERAGE_AUDIT: &str =
    include_str!("../.agents/skills/script-kit-devtools/references/devtools-coverage-audit.md");

#[test]
fn coverage_cli_is_a_devtools_primitive_not_a_recipe() {
    for needle in [
        "script-kit-devtools.coverage",
        "primitiveFamilies",
        "devtools.inspect",
        "devtools.measure",
        "devtools.act",
        "devtools.compare",
        "devtools.investigate",
        "--surface",
        "--domain",
        "--markdown",
    ] {
        assert!(
            COVERAGE.contains(needle),
            "coverage CLI must expose DevTools primitive contract: {needle}"
        );
    }

    assert!(
        !COVERAGE.contains("scripts/agentic/index.ts"),
        "coverage CLI must not route through the agentic recipe catalog"
    );
}

#[test]
fn coverage_domains_match_chrome_devtools_breadth() {
    for domain in [
        "Targets and Windows",
        "Elements and Semantics",
        "Layout and Box Model",
        "Styles, Theme, and Text Fit",
        "Console, Logs, and Events",
        "Sources, Scripts, and Owners",
        "Performance and Timeline",
        "Storage, Resources, and Privacy",
        "Accessibility",
        "Input, Focus, and Actions",
        "Media, Sensors, and Permissions",
        "Screenshots and Visual Proof",
        "Investigation Records",
    ] {
        assert!(
            COVERAGE.contains(domain),
            "coverage CLI must include Chrome-style domain: {domain}"
        );
        assert!(
            COVERAGE_MAP.contains(domain),
            "coverage map docs must explain Chrome-style domain: {domain}"
        );
    }
}

#[test]
fn coverage_pins_notes_features_shortcuts_and_missing_primitives() {
    for needle in [
        "floating notes host",
        "editor mode",
        "browse/list mode",
        "trash mode",
        "markdown editor",
        "markdown preview",
        "editor find",
        "global search",
        "format toolbar",
        "focus mode",
        "pinning",
        "sort cycling",
        "command bar",
        "actions panel",
        "recent note switcher",
        "note cart",
        "clipboard-backed note creation",
        "embedded ACP mode",
        "ACP actions popup",
        "ACP history portal",
        "attachment/context chips",
        "draft snapshots",
        "auto-resize",
        "autosave and dirty state",
        "history back/forward",
        "scroll collapse after deleting trailing lines",
        "independent app-hide behavior",
        "src/notes/window.rs",
        "src/notes/window/keyboard.rs",
        "src/notes/window/acp_host.rs",
        "src/notes/actions_panel.rs",
        "src/notes/storage.rs",
        "Cmd+N",
        "Cmd+Shift+N",
        "Cmd+P",
        "Cmd+K",
        "Cmd+W",
        "Cmd+Shift+P",
        "Cmd+F",
        "Cmd+Shift+F",
        "Cmd+Shift+T",
        "Cmd+.",
        "Cmd+Shift+.",
        "Cmd+Shift+S",
        "Cmd+Z",
        "Cmd+D",
        "Cmd+Shift+D",
        "Cmd+Shift+X",
        "Cmd+Shift+L",
        "Cmd+Enter",
        "Cmd+Shift+A",
        "Cmd+Shift+O",
        "Cmd+1..Cmd+9",
        "Shift+Tab",
        "Alt+Shift+Up",
        "Ctrl+Shift+K",
        "Escape",
        "Tab",
        "Enter",
        "Notes target-scoped layout info",
        "editor and preview scroll anchors",
        "ACP embedded generation and origin receipts",
    ] {
        assert!(
            COVERAGE.contains(needle),
            "coverage CLI must keep Notes DevTools coverage explicit: {needle}"
        );
    }
}

#[test]
fn coverage_pins_dictation_states_media_and_privacy_boundaries() {
    for needle in [
        "idle/hidden",
        "recording",
        "quiet recording",
        "active speech",
        "confirming",
        "Script Kit target delivery",
        "Notes editor target delivery",
        "ACP target delivery",
        "Tab AI target delivery",
        "external app target delivery",
        "frontmost app paste delivery",
        "stop confirmation",
        "transcribing",
        "delivering",
        "finished",
        "failed/error",
        "Idle -> Recording",
        "Recording -> Confirming",
        "Transcribing -> Delivering",
        "Delivering -> Finished",
        "waveform/audio level bars",
        "microphone permission",
        "microphone device",
        "preferred device fallback",
        "model readiness",
        "model download/extract/failure status",
        "hotkey readiness",
        "hotkey registration",
        "hotkey conflict detection",
        "target identity",
        "transcript generation",
        "cursor insertion range",
        "wrong-target rejection",
        "cleanup without TCC/System Settings mutation",
        "dictation hotkey",
        "target badge click",
        "devtools.media.inspect",
        "passive microphone permission status",
        "hotkey binding snapshot",
        "src/dictation/window.rs",
        "src/dictation/runtime.rs",
        "src/dictation/types.rs",
        "src/dictation/setup.rs",
        "src/main_entry/runtime_tray_hotkeys.rs",
    ] {
        assert!(
            COVERAGE.contains(needle),
            "coverage CLI must keep Dictation DevTools coverage explicit: {needle}"
        );
    }
}

#[test]
fn coverage_source_files_exist_for_notes_and_dictation() {
    for path in [
        "src/notes/window.rs",
        "src/notes/window/keyboard.rs",
        "src/notes/window/acp_host.rs",
        "src/notes/window/window_ops.rs",
        "src/notes/actions_panel.rs",
        "src/notes/browse_panel.rs",
        "src/notes/storage.rs",
        "src/notes/model.rs",
        "src/dictation/window.rs",
        "src/dictation/runtime.rs",
        "src/dictation/types.rs",
        "src/dictation/setup.rs",
        "src/dictation/capture.rs",
        "src/dictation/device.rs",
        "src/dictation/transcription.rs",
        "src/main_entry/runtime_tray_hotkeys.rs",
    ] {
        assert!(
            fs::metadata(path).is_ok(),
            "coverage source reference should exist: {path}"
        );
        assert!(
            COVERAGE.contains(path),
            "coverage CLI should expose source reference: {path}"
        );
    }
}

#[test]
fn docs_route_agents_to_coverage_before_more_scripts() {
    for needle in [
        "bun scripts/devtools/coverage.ts",
        "Chrome-DevTools-level breadth",
        "Notes coverage",
        "Dictation coverage",
        "coverage command",
    ] {
        assert!(
            DEVTOOLS_SKILL.contains(needle)
                || COVERAGE_MAP.contains(needle)
                || COVERAGE_AUDIT.contains(needle),
            "DevTools docs must route agents through coverage map: {needle}"
        );
    }

    assert!(
        COVERAGE_MAP.contains("Recipes should only wrap these primitives"),
        "coverage docs must keep recipes bounded to regression wrappers"
    );
}

#[test]
fn devtools_coverage_artifacts_are_checked_in() {
    for path in [
        "scripts/devtools/coverage.ts",
        ".agents/skills/script-kit-devtools/references/devtools-api-coverage-map.md",
    ] {
        assert!(
            fs::metadata(path).is_ok(),
            "expected checked-in DevTools coverage artifact at {path}"
        );
    }
}
