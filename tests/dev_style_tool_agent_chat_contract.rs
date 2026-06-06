const MOD_SOURCE: &str = include_str!("../src/dev_style_tool/mod.rs");
const CATALOG_SOURCE: &str = include_str!("../src/dev_style_tool/agent_chat_catalog.rs");
const RUNTIME_SOURCE: &str = include_str!("../src/dev_style_tool/runtime_overrides.rs");
const RENDER_SOURCE: &str = include_str!("../src/dev_style_tool/render.rs");
const EXPORT_SOURCE: &str = include_str!("../src/dev_style_tool/export.rs");
const TRANSCRIPT_SOURCE: &str = include_str!("../src/ai/acp/components/transcript.rs");
const PROMPT_HANDLER_SOURCE: &str = include_str!("../src/prompt_handler/mod.rs");
const COLLECTOR_SOURCE: &str = include_str!("../src/windows/automation_surface_collector.rs");

#[test]
fn agent_chat_style_catalog_is_registered_and_exported() {
    assert!(MOD_SOURCE.contains("pub mod agent_chat_catalog;"));
    assert!(MOD_SOURCE.contains("pub use agent_chat_catalog::*;"));
    assert!(CATALOG_SOURCE.contains("pub struct AgentChatStyleDef"));
    assert!(CATALOG_SOURCE.contains("pub const AGENT_CHAT_KNOBS"));
    assert!(CATALOG_SOURCE.contains("agentChat.markdown.bodyFontSize"));
    assert!(CATALOG_SOURCE.contains("agentChat.user.paddingX"));
    assert!(CATALOG_SOURCE.contains("agentChat.collapsible.maxBodyHeight"));
    assert!(CATALOG_SOURCE.contains("agent_chat_knob_id_from_str"));
}

#[test]
fn agent_chat_runtime_overrides_participate_in_history_and_export() {
    assert!(RUNTIME_SOURCE.contains("agent_chat_values: BTreeMap<AgentChatKnobId, StyleValue>"));
    assert!(RUNTIME_SOURCE.contains("HistoryEntry::AgentChatSingle"));
    assert!(RUNTIME_SOURCE.contains("set_agent_chat_value"));
    assert!(RUNTIME_SOURCE.contains("reset_agent_chat_value"));
    assert!(RUNTIME_SOURCE.contains("effective_agent_chat_style"));
    assert!(RUNTIME_SOURCE.contains("set_agent_chat_number_from_devtools"));
    assert!(EXPORT_SOURCE.contains("\"agentChatStyle\""));
    assert!(EXPORT_SOURCE.contains("\"agentChat\""));
    assert!(EXPORT_SOURCE.contains("src/dev_style_tool/agent_chat_catalog.rs"));
}

#[test]
fn dev_style_tool_has_agent_chat_tab_and_controls() {
    assert!(RENDER_SOURCE.contains("AgentChatStyling"));
    assert!(RENDER_SOURCE.contains("\"Agent Chat Styling\""));
    assert!(RENDER_SOURCE.contains("tab:dev-style-tool:agent-chat-styling"));
    assert!(RENDER_SOURCE.contains("AgentChatControlState"));
    assert!(RENDER_SOURCE.contains("render_agent_chat_controls"));
    assert!(RENDER_SOURCE.contains("control:dev-style-tool-agent-chat"));
    assert!(RENDER_SOURCE.contains("refresh_agent_chat"));
    assert!(PROMPT_HANDLER_SOURCE.contains("set_agent_chat_number_from_devtools"));
    assert!(PROMPT_HANDLER_SOURCE.contains("agentChat."));
    assert!(COLLECTOR_SOURCE.contains("crate::dev_style_tool::AGENT_CHAT_KNOBS"));
    assert!(COLLECTOR_SOURCE.contains("slider:dev-style-tool-agent-chat:{}"));
    assert!(COLLECTOR_SOURCE.contains("input:dev-style-tool-agent-chat:{}"));
    assert!(COLLECTOR_SOURCE.contains("button:dev-style-tool-agent-chat-reset:{}"));
}

#[test]
fn acp_transcript_uses_live_agent_chat_style() {
    assert!(TRANSCRIPT_SOURCE.contains("effective_agent_chat_style()"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.markdown.body_font_size"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.transcript.row_padding_x"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.user_message"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.assistant_message"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.collapsible"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.error"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.system"));
}
