const TARGETS_SOURCE: &str = include_str!("../src/dev_style_tool/kitchen_sink_targets.rs");
const RENDER_SOURCE: &str = include_str!("../src/dev_style_tool/render.rs");
const COLLECTOR_SOURCE: &str = include_str!("../src/windows/automation_surface_collector.rs");
const PROMPT_HANDLER_SOURCE: &str = include_str!("../src/prompt_handler/mod.rs");

#[test]
fn dev_style_tool_kitchen_sink_targets_have_stable_unique_semantics() {
    let ids = [
        "button:dev-style-tool-open-main-window-kitchen-sink",
        "button:dev-style-tool-open-main-window-no-match-kitchen-sink",
        "button:dev-style-tool-open-actions-popup-kitchen-sink",
        "button:dev-style-tool-open-actions-popup-no-match-kitchen-sink",
        "button:dev-style-tool-open-agent-chat-kitchen-sink",
    ];

    for id in ids {
        assert!(
            TARGETS_SOURCE.contains(id),
            "target catalog must define semantic id {id}"
        );
        assert!(
            RENDER_SOURCE.contains(id) || RENDER_SOURCE.contains("target.semantic_id()"),
            "render source must expose semantic id {id}"
        );
        assert!(
            COLLECTOR_SOURCE.contains("DevStyleKitchenSinkTarget::ALL"),
            "collector should emit target catalog semantics"
        );
    }

    assert!(PROMPT_HANDLER_SOURCE.contains("OPEN_MAIN_WINDOW_KITCHEN_SINK_BUTTON"));
    assert!(PROMPT_HANDLER_SOURCE.contains("OPEN_MAIN_WINDOW_NO_MATCH_KITCHEN_SINK_BUTTON"));
    assert!(PROMPT_HANDLER_SOURCE.contains("OPEN_ACTIONS_POPUP_KITCHEN_SINK_BUTTON"));
    assert!(PROMPT_HANDLER_SOURCE.contains("OPEN_ACTIONS_POPUP_NO_MATCH_KITCHEN_SINK_BUTTON"));
    assert!(PROMPT_HANDLER_SOURCE.contains("OPEN_AGENT_CHAT_KITCHEN_SINK_BUTTON"));
    assert!(TARGETS_SOURCE.contains("MainWindowPopulated"));
    assert!(TARGETS_SOURCE.contains("MainWindowNoMatch"));
    assert!(TARGETS_SOURCE.contains("ActionsPopupPopulated"));
    assert!(TARGETS_SOURCE.contains("ActionsPopupNoMatch"));
    assert!(TARGETS_SOURCE.contains("AgentChat"));
}

#[test]
fn dev_style_tool_pairs_kitchen_sinks_with_style_tabs() {
    assert!(RENDER_SOURCE.contains("render_kitchen_sink_controls"));
    assert!(RENDER_SOURCE.contains("DevStyleToolTab::MainWindowStyling"));
    assert!(RENDER_SOURCE.contains("DevStyleKitchenSinkTarget::MainWindowPopulated"));
    assert!(RENDER_SOURCE.contains("DevStyleKitchenSinkTarget::MainWindowNoMatch"));
    assert!(RENDER_SOURCE.contains("DevStyleToolTab::ActionsPopupStyling"));
    assert!(RENDER_SOURCE.contains("DevStyleKitchenSinkTarget::ActionsPopupPopulated"));
    assert!(RENDER_SOURCE.contains("DevStyleKitchenSinkTarget::ActionsPopupNoMatch"));
    assert!(RENDER_SOURCE.contains("DevStyleToolTab::AgentChatStyling"));
    assert!(RENDER_SOURCE.contains("DevStyleKitchenSinkTarget::AgentChat"));
}

#[test]
fn devtools_semantic_actions_dispatch_all_kitchen_sinks() {
    assert!(PROMPT_HANDLER_SOURCE.contains("run_dev_style_tool_semantic_action_for_batch"));
    assert!(PROMPT_HANDLER_SOURCE.contains("open_main_window_kitchen_sink_fixture"));
    assert!(PROMPT_HANDLER_SOURCE.contains("open_main_window_no_match_kitchen_sink_fixture"));
    assert!(PROMPT_HANDLER_SOURCE.contains("open_actions_popup_kitchen_sink_fixture"));
    assert!(PROMPT_HANDLER_SOURCE.contains("open_actions_popup_no_match_kitchen_sink_fixture"));
    assert!(PROMPT_HANDLER_SOURCE.contains("open_agent_chat_kitchen_sink_fixture"));
}
