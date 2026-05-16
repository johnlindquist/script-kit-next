const APP_STATE: &str = include_str!("../src/main_sections/app_state.rs");
const PROMPT_AI: &str = include_str!("../src/app_impl/prompt_ai.rs");
const QUERY_VARIANTS: &str = include_str!("../src/protocol/message/variants/query_ops.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const UI_WINDOW: &str = include_str!("../src/app_impl/ui_window.rs");

// doc-anchor-removed: [[removed-docs AI snapshot and close telemetry]]
#[test]
fn mini_ai_close_snapshot_carries_required_fields() {
    for field in [
        "prompt_id",
        "main_window_mode",
        "source",
        "draft_len",
        "pending_submit",
        "handoff_source",
        "return_origin",
    ] {
        assert!(
            APP_STATE.contains(field),
            "MiniAiCloseSnapshot must carry {field}"
        );
    }
    assert!(
        APP_STATE.contains("enum MiniAiCloseSource"),
        "MiniAiCloseSource must be typed"
    );
}

// doc-anchor-removed: [[removed-docs AI close telemetry]]
#[test]
fn mini_ai_close_path_emits_snapshot_before_close() {
    assert!(
        PROMPT_AI.contains("event = \"mini_ai_window_close_requested\"")
            && PROMPT_AI.contains("event = \"mini_ai_close_snapshot\""),
        "Mini AI close path must emit request and snapshot telemetry"
    );
    assert!(
        PROMPT_AI.contains("capture_mini_ai_close_snapshot"),
        "Mini AI receiver must collect the live draft snapshot before closing"
    );
    assert!(
        APP_RUN_SETUP.contains("SimulateKey: Escape handled by ChatPrompt")
            && APP_RUN_SETUP.contains("chat.handle_escape(cx);"),
        "simulateKey Escape must route through the ChatPrompt escape callback"
    );
    assert!(
        PROMPT_HANDLER.contains(".with_escape_callback(escape_callback)")
            && PROMPT_HANDLER.contains("inline_chat_escape_sender"),
        "SDK/protocol ChatPrompt must wire the Mini AI close callback"
    );
    assert!(
        UI_WINDOW.contains("AppView::ChatPrompt { entity, .. }")
            && UI_WINDOW.contains("prompt.set_input(text, cx)"),
        "setInput must update ChatPrompt drafts before close snapshots"
    );
    assert!(
        PROMPT_HANDLER.contains("prompt.set_input(text.to_string(), cx)"),
        "batch setInput must update ChatPrompt drafts before close snapshots"
    );
}

// doc-anchor-removed: [[removed-docs AI getState snapshot]]
#[test]
fn get_state_exposes_mini_ai_snapshot_shape() {
    assert!(
        QUERY_VARIANTS.contains("mini_ai: Option<serde_json::Value>")
            && QUERY_VARIANTS.contains("rename = \"miniAi\""),
        "stateResult must expose optional miniAi snapshot"
    );
    for key in [
        "\"visible\"",
        "\"promptId\"",
        "\"mainWindowMode\"",
        "\"draftLen\"",
        "\"pendingSubmit\"",
        "\"handoffSource\"",
        "\"returnOrigin\"",
        "\"lastCloseSource\"",
    ] {
        assert!(
            PROMPT_AI.contains(key),
            "mini_ai_state_snapshot must include {key}"
        );
    }
    assert!(
        PROMPT_HANDLER.contains("Some(self.mini_ai_state_snapshot(cx))"),
        "getState must attach miniAi snapshot"
    );
}
