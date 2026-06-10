const MOD_SOURCE: &str = include_str!("../src/dev_style_tool/mod.rs");
const CATALOG_SOURCE: &str = include_str!("../src/dev_style_tool/agent_chat_catalog.rs");
const RUNTIME_SOURCE: &str = include_str!("../src/dev_style_tool/runtime_overrides.rs");
const RENDER_SOURCE: &str = include_str!("../src/dev_style_tool/render.rs");
const TARGETS_SOURCE: &str = include_str!("../src/dev_style_tool/kitchen_sink_targets.rs");
const EXPORT_SOURCE: &str = include_str!("../src/dev_style_tool/export.rs");
const TRANSCRIPT_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/components/transcript.rs");
const PROMPT_HANDLER_SOURCE: &str = include_str!("../src/prompt_handler/mod.rs");
const COLLECTOR_SOURCE: &str = include_str!("../src/windows/automation_surface_collector.rs");
const TEXT_VIEW_STYLE_SOURCE: &str =
    include_str!("../vendor/gpui-component/crates/ui/src/text/style.rs");
const TEXT_VIEW_NODE_SOURCE: &str =
    include_str!("../vendor/gpui-component/crates/ui/src/text/node.rs");

#[test]
fn agent_chat_style_catalog_is_registered_and_exported() {
    assert!(MOD_SOURCE.contains("pub mod agent_chat_catalog;"));
    assert!(MOD_SOURCE.contains("pub use agent_chat_catalog::*;"));
    assert!(CATALOG_SOURCE.contains("pub struct AgentChatStyleDef"));
    assert!(CATALOG_SOURCE.contains("pub const AGENT_CHAT_KNOBS"));
    assert!(CATALOG_SOURCE.contains("agentChat.markdown.bodyFontSize"));
    assert!(CATALOG_SOURCE.contains("agentChat.markdown.blockquotePaddingX"));
    assert!(CATALOG_SOURCE.contains("agentChat.markdown.blockquoteBgAlpha"));
    assert!(CATALOG_SOURCE.contains("agentChat.markdown.blockquoteBorderAlpha"));
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
    assert!(RENDER_SOURCE.contains("render_sidebar_groups"));
    assert!(RENDER_SOURCE.contains("tabs:dev-style-tool-agent-chat-groups"));
    assert!(RENDER_SOURCE.contains("active_agent_chat_group"));
    assert!(RENDER_SOURCE.contains("agent_chat_group_slug"));
    assert!(RENDER_SOURCE.contains("agent-chat-style-section:{}"));
    assert!(RENDER_SOURCE.contains("control:dev-style-tool-agent-chat"));
    assert!(RENDER_SOURCE.contains("refresh_agent_chat"));
    assert!(RENDER_SOURCE.contains("open_agent_chat_kitchen_sink"));
    assert!(RENDER_SOURCE.contains("open_agent_chat_kitchen_sink_fixture"));
    assert!(TARGETS_SOURCE.contains("button:dev-style-tool-open-agent-chat-kitchen-sink"));
    // The launcher renders contextually from the targets catalog
    // (target.semantic_id()/label()) inside the content header.
    assert!(RENDER_SOURCE.contains("DevStyleKitchenSinkTarget::AgentChat"));
    assert!(RENDER_SOURCE.contains("render_kitchen_sink_controls"));
    assert!(RENDER_SOURCE.contains("open_main_window_kitchen_sink"));
    assert!(RENDER_SOURCE.contains("open_actions_popup_kitchen_sink"));
    assert!(PROMPT_HANDLER_SOURCE.contains("set_agent_chat_number_from_devtools"));
    assert!(PROMPT_HANDLER_SOURCE.contains("agentChat."));
    assert!(PROMPT_HANDLER_SOURCE.contains("\"selectBySemanticId\""));
    assert!(PROMPT_HANDLER_SOURCE.contains("OPEN_AGENT_CHAT_KITCHEN_SINK_BUTTON"));
    assert!(PROMPT_HANDLER_SOURCE.contains("open_agent_chat_kitchen_sink_fixture(cx)"));
    assert!(COLLECTOR_SOURCE.contains("crate::dev_style_tool::AGENT_CHAT_KNOBS"));
    assert!(COLLECTOR_SOURCE.contains("DevStyleKitchenSinkTarget::ALL"));
    assert!(COLLECTOR_SOURCE.contains("tab:dev-style-tool-agent-chat:{}"));
    assert!(COLLECTOR_SOURCE.contains("agent-chat-style-section:{}"));
    assert!(COLLECTOR_SOURCE.contains("slider:dev-style-tool-agent-chat:{}"));
    assert!(COLLECTOR_SOURCE.contains("input:dev-style-tool-agent-chat:{}"));
    assert!(COLLECTOR_SOURCE.contains("button:dev-style-tool-agent-chat-reset:{}"));
}

#[test]
fn dev_style_tool_navigation_is_a_sidebar_tree() {
    // Navigation is a Storybook-style rail: filter on top, surfaces as rows,
    // the active surface's groups nested beneath it. The legacy `tabs:*` ids
    // stay on the sidebar containers so automation keeps working.
    assert!(RENDER_SOURCE.contains("fn render_sidebar"));
    assert!(RENDER_SOURCE.contains("sidebar:dev-style-tool"));
    assert!(RENDER_SOURCE.contains("tabs:dev-style-tool-primary"));
    assert!(RENDER_SOURCE.contains("tabs:dev-style-tool-groups"));
    assert!(RENDER_SOURCE.contains("tabs:dev-style-tool-actions-groups"));
    assert!(RENDER_SOURCE.contains("tabs:dev-style-tool-agent-chat-groups"));
    assert!(RENDER_SOURCE.contains("tab:dev-style-tool-group:{}"));
    assert!(RENDER_SOURCE.contains("summary:dev-style-tool-active-scope"));
    assert!(RENDER_SOURCE.contains("render_content_header"));
    assert!(RENDER_SOURCE.contains("active_surface_description"));
    assert!(RENDER_SOURCE.contains("chrome.input_surface_rgba"));
    assert!(RENDER_SOURCE.contains("chrome.border_rgba"));
}

#[test]
fn agent_chat_transcript_uses_live_agent_chat_style() {
    assert!(TRANSCRIPT_SOURCE.contains("effective_agent_chat_style()"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.markdown.body_font_size"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.transcript.row_padding_x"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.user_message"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.assistant_message"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.collapsible"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.error"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.system"));
    assert!(TRANSCRIPT_SOURCE.contains(".blockquote("));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.markdown.blockquote_bg_alpha"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.markdown.blockquote_border_alpha"));
    assert!(TRANSCRIPT_SOURCE.contains("style_def.markdown.blockquote_padding_x"));
    assert!(TEXT_VIEW_STYLE_SOURCE.contains("pub blockquote: StyleRefinement"));
    assert!(TEXT_VIEW_STYLE_SOURCE.contains("pub fn blockquote"));
    assert!(TEXT_VIEW_STYLE_SOURCE.contains("self.code_block == other.code_block"));
    assert!(TEXT_VIEW_STYLE_SOURCE.contains("self.blockquote == other.blockquote"));
    assert!(TEXT_VIEW_NODE_SOURCE.contains(".refine_style(&node_cx.style.blockquote)"));
}
