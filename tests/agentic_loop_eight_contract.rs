//! Source-level contract for eighth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");

#[test]
fn index_help_exposes_loop_eight_recipes() {
    for name in [
        "tray-global-hotkey-menu-mutation-stress",
        "multi-window-resize-monitor-restoration-stress",
        "agent_chat-targeted-dictation-delivery-stress",
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
        "runTrayGlobalHotkeyMenuMutationStressScenario",
        "runMultiWindowResizeMonitorRestorationStressScenario",
        "runAgentChatTargetedDictationDeliveryStressScenario",
    ] {
        assert!(
            INDEX.contains(function_name) || SCENARIO.contains(function_name),
            "loop-eight function {function_name} must be wired"
        );
    }
}

#[test]
fn tray_global_hotkey_menu_mutation_stress_pins_section_update_route_and_duplicate_receipts() {
    for token in [
        "tray-global-hotkey-menu-mutation-stress",
        "missing_tray_global_hotkey_menu_mutation_receipt",
        "trayMenuMutation",
        "sectionOrder",
        "duplicateItemIds",
        "duplicateLabels",
        "versionLabelBefore",
        "versionLabelAfter",
        "updateStateMutation",
        "refreshRanOnMainThread",
        "targetActionIds",
        "globalHotkeyRoute",
        "openedExternalUrl: false",
        "reloadedScripts: false",
        "quitApp: false",
        "mutatedUserConfig: false",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "Tray/global-hotkey mutation stress must pin {token}"
        );
    }
}

#[test]
fn multi_window_resize_monitor_restoration_stress_pins_identity_bounds_scale_and_clobber_receipts()
{
    for token in [
        "multi-window-resize-monitor-restoration-stress",
        "missing_multi_window_resize_monitor_restoration_receipt",
        "multiWindowRestore",
        "requestedSurfaces",
        "monitorSimulation",
        "scale-bounds-drift",
        "usedRealDisplayMutation: false",
        "windowIdsStable",
        "semanticSurfacesStable",
        "attachedPopupParentId",
        "detachedSurfaceId",
        "notesWindowId",
        "restoreOrder",
        "scaleRem",
        "remPxStable",
        "noPopupMainClobber",
        "mutatedDisplays: false",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "Multi-window resize/monitor restoration stress must pin {token}"
        );
    }
}

#[test]
fn agent_chat_targeted_dictation_delivery_stress_pins_target_generation_range_and_passive_setup_receipts(
) {
    for token in [
        "agent_chat-targeted-dictation-delivery-stress",
        "missing_agent_chat_targeted_dictation_delivery_receipt",
        "agent_chatDictationDelivery",
        "targetAgentChatWindowId",
        "targetSurfaceId",
        "targetGenerationId",
        "embeddedAgentChatWindowId",
        "wrongWindowUnchanged",
        "deliveryId",
        "transcriptGenerationId",
        "cursorInsertionRange",
        "deliveredToWrongWindow",
        "microphonePrompted: false",
        "modelDownloadStarted: false",
        "captureStarted: false",
        "startedAudioCapture: false",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "Agent Chat-targeted dictation delivery stress must pin {token}"
        );
    }
}
