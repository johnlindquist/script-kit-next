const APP_RUN_SETUP: &str = include_str!("../../src/main_entry/app_run_setup.rs");
const RUNTIME_TRAY_HOTKEYS: &str = include_str!("../../src/main_entry/runtime_tray_hotkeys.rs");
const FOCUSED_TEXT_ENTRY: &str =
    include_str!("../../src/app_impl/tab_ai_mode/focused_text_entry.rs");
const ACP_VIEW: &str = include_str!("../../src/ai/acp/view.rs");
const ACP_UI_VARIANT: &str = include_str!("../../src/ai/acp/ui_variant.rs");

#[test]
fn inline_ai_hotkeys_capture_before_opening_focused_text_agent_chat() {
    for source in [APP_RUN_SETUP, RUNTIME_TRAY_HOTKEYS] {
        let channel = source
            .find("inline_ai_hotkey_channel().1.recv().await")
            .expect("inline AI listener must exist");
        let capture = source[channel..]
            .find("capture_focused_text_field")
            .expect("inline AI listener must capture focused text")
            + channel;
        let open = source[channel..]
            .find("open_focused_text_agent_chat_from_snapshot")
            .expect("inline AI listener must open focused-text Agent Chat")
            + channel;

        assert!(
            capture < open,
            "focused text must be captured before opening Agent Chat"
        );
        assert!(!source[channel..].contains("launch_inline_agent_from_focused_text"));
    }
}

#[test]
fn focused_text_entry_forces_embedded_mini_agent_chat_surface() {
    for required in [
        "open_focused_text_agent_chat_from_snapshot",
        "AcpChatUiVariant::FocusedTextMini",
        "begin_tab_ai_harness_entry_from_source_view",
        "force_acp_surface",
        "stage_focused_text_from_host",
        "focused_text_agent_chat_open",
    ] {
        assert!(
            FOCUSED_TEXT_ENTRY.contains(required),
            "missing focused-text Agent Chat entry contract: {required}"
        );
    }
}

#[test]
fn acp_view_can_stage_focused_text_as_host_owned_context() {
    for required in [
        "stage_focused_text_from_host",
        "focused-text://",
        "Focused Text ·",
        "replace_pending_context_parts(vec![part], source, cx)",
        "focused_text_context_staged",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing focused-text ACP staging contract: {required}"
        );
    }
}

#[test]
fn focused_text_mini_variant_is_protocol_visible() {
    for required in [
        "FocusedTextMini",
        "\"focused-text-mini\"",
        "AcpTranscriptPresentation::FocusedTextPreview",
        "AcpComposerPlacement::FocusedTextSingleLine",
        "AcpChromeDensity::Mini",
    ] {
        assert!(
            ACP_UI_VARIANT.contains(required),
            "missing focused-text mini variant contract: {required}"
        );
    }
}
