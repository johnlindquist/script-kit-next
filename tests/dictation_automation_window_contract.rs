//! Source-level contracts for Dictation as a first-class automation window.

const AUTOMATION_WINDOW: &str = include_str!("../src/protocol/types/automation_window.rs");
const AUTOMATION_REGISTRY: &str = include_str!("../src/windows/automation_registry.rs");
const SURFACE_COLLECTOR: &str = include_str!("../src/windows/automation_surface_collector.rs");
const DICTATION_WINDOW: &str = include_str!("../src/dictation/window.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const SCREENSHOT_ROUTING: &str = include_str!("../src/platform/screenshots_window_open.rs");
const STDIN_COMMANDS: &str = include_str!("../src/stdin_commands/mod.rs");
const DEVTOOLS_TARGETS: &str = include_str!("../scripts/devtools/targets.ts");

#[test]
fn dictation_is_a_first_class_automation_window_kind() {
    assert!(
        AUTOMATION_WINDOW.contains("Dictation,"),
        "AutomationWindowKind must include Dictation"
    );
    assert!(
        AUTOMATION_WINDOW.contains("AutomationWindowKind::Dictation => \"dictation\""),
        "Dictation automation kind must round-trip as camelCase dictation"
    );
    assert!(
        DEVTOOLS_TARGETS.contains("if (value === \"dictation\") return \"Dictation\";"),
        "DevTools target normalization must render dictation windows clearly"
    );
}

#[test]
fn dictation_overlay_has_visual_only_stdin_fixture() {
    for needle in [
        "OpenDictationOverlayFixture",
        "\"openDictationOverlayFixture\"",
        "Dictation overlay fixture opened without media capture",
    ] {
        assert!(
            STDIN_COMMANDS.contains(needle)
                || DICTATION_WINDOW.contains(needle)
                || include_str!("../src/main_entry/runtime_stdin.rs").contains(needle),
            "dictation overlay visual fixture must keep no-media proof path: {needle}"
        );
    }
}

#[test]
fn dictation_overlay_registers_and_removes_runtime_target() {
    for needle in [
        "DICTATION_OVERLAY_AUTOMATION_ID",
        "upsert_runtime_window_handle(DICTATION_OVERLAY_AUTOMATION_ID",
        "AutomationWindowKind::Dictation",
        "semantic_surface: Some(\"dictation\".to_string())",
        "remove_runtime_window_handle(DICTATION_OVERLAY_AUTOMATION_ID)",
        "remove_automation_window(DICTATION_OVERLAY_AUTOMATION_ID)",
    ] {
        assert!(
            DICTATION_WINDOW.contains(needle),
            "dictation overlay must manage automation lifecycle: {needle}"
        );
    }
}

#[test]
fn dictation_target_supports_elements_layout_and_screenshot_capture() {
    assert!(
        SURFACE_COLLECTOR.contains("AutomationWindowKind::Dictation => collect_dictation_snapshot"),
        "getElements/inspectAutomationWindow must expose dictation semantic nodes"
    );
    assert!(
        PROMPT_HANDLER.contains("AutomationWindowKind::Dictation")
            && PROMPT_HANDLER.contains("crate::dictation::automation_layout_info(&resolved)"),
        "getLayoutInfo must route dictation targets to dictation layout metrics"
    );
    assert!(
        SCREENSHOT_ROUTING.contains("AutomationWindowKind::Dictation"),
        "targeted screenshots must treat Dictation as a detached OS window"
    );
    assert!(
        AUTOMATION_REGISTRY.contains("AutomationWindowKind::Dictation =>"),
        "automation registry must give Dictation deterministic kind ordering"
    );
}

#[test]
fn dictation_overlay_radius_uses_shared_liquid_glass_token() {
    assert!(
        DICTATION_WINDOW.contains(
            "pub(crate) const OVERLAY_RADIUS_PX: f32 = crate::ui::chrome::LIQUID_GLASS_PANEL_RADIUS_PX;"
        ),
        "dictation overlay radius must use the shared Liquid Glass panel token"
    );
    assert!(
        DICTATION_WINDOW.contains("Some(OVERLAY_RADIUS_PX)"),
        "dictation layout info must report the overlay radius for visual audit"
    );
}
