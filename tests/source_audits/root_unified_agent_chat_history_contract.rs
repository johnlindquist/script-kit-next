#[test]
fn root_unified_agent_chat_history_config_is_real_and_scoped() {
    let config_types = include_str!("../../src/config/types.rs");
    let config_schema = include_str!("../../scripts/config-schema.ts");

    assert!(config_types.contains("pub struct UnifiedSearchAgentChatHistoryConfig"));
    assert!(config_types.contains("fn agent_chat_history_section_options("));
    assert!(config_schema.contains("agent_chatHistory?: UnifiedSearchAgentChatHistoryConfig"));
    assert!(
        config_schema.contains("clipboardHistory?: UnifiedSearchClipboardHistoryConfig")
            && config_schema.contains("notes?: UnifiedSearchNotesConfig"),
        "config.ts schema should expose implemented unified-search sources"
    );
}

#[test]
fn root_unified_agent_chat_history_uses_passive_grouping_contract() {
    let grouping = include_str!("../../src/scripts/grouping.rs");

    assert!(grouping.contains("fn append_root_agent_chat_history_section("));
    assert!(grouping.contains("append_root_passive_section("));
    assert!(grouping.contains("\"Agent Chat Conversations\""));
    assert!(grouping.contains("root_agent_chat_history_query_is_eligible("));
    assert!(
        grouping.contains("label.starts_with(\"Use \\\"\") && label.ends_with(\"\\\" with...\")"),
        "passive insertion should target the fallback section header, not the first fallback row"
    );
}

#[test]
fn root_unified_agent_chat_history_result_is_stable_and_non_bindable() {
    let types = include_str!("../../src/scripts/types.rs");

    assert!(types.contains("pub struct AgentChatHistoryMatch"));
    assert!(types.contains("AgentChatHistory(AgentChatHistoryMatch)"));
    assert!(types.contains("\"agent_chat-history/{}\""));
    assert!(types.contains("SearchResult::AgentChatHistory(_) => None"));
    assert!(types.contains("SearchResult::AgentChatHistory(_) => \"Resume Conversation\""));
}

#[test]
fn root_unified_agent_chat_history_enter_resumes_existing_helper() {
    let selection = include_str!("../../src/app_impl/selection_fallback.rs");
    let agent_chat_history = include_str!("../../src/render_builtins/agent_chat_history.rs");

    assert!(selection.contains("SearchResult::AgentChatHistory(agent_chat_history_match)"));
    assert!(selection.contains("self.resume_agent_chat_conversation_from_history("));
    assert!(
        agent_chat_history.contains("pub(crate) fn resume_agent_chat_conversation_from_history(")
    );
}
