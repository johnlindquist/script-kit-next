#[test]
fn root_unified_acp_history_config_is_real_and_scoped() {
    let config_types = include_str!("../../src/config/types.rs");
    let config_schema = include_str!("../../scripts/config-schema.ts");

    assert!(config_types.contains("pub struct UnifiedSearchAcpHistoryConfig"));
    assert!(config_types.contains("fn acp_history_section_options("));
    assert!(config_schema.contains("acpHistory?: UnifiedSearchAcpHistoryConfig"));
    assert!(
        config_schema.contains("clipboardHistory?: UnifiedSearchClipboardHistoryConfig")
            && !config_schema.contains("notes?: UnifiedSearch"),
        "config.ts schema should only expose implemented unified-search sources"
    );
}

#[test]
fn root_unified_acp_history_uses_passive_grouping_contract() {
    let grouping = include_str!("../../src/scripts/grouping.rs");

    assert!(grouping.contains("fn append_root_acp_history_section("));
    assert!(grouping.contains("append_root_passive_section("));
    assert!(grouping.contains("\"AI Conversations\""));
    assert!(grouping.contains("root_acp_history_query_is_eligible("));
    assert!(
        grouping.contains("label.starts_with(\"Use \\\"\") && label.ends_with(\"\\\" with...\")"),
        "passive insertion should target the fallback section header, not the first fallback row"
    );
}

#[test]
fn root_unified_acp_history_result_is_stable_and_non_bindable() {
    let types = include_str!("../../src/scripts/types.rs");

    assert!(types.contains("pub struct AcpHistoryMatch"));
    assert!(types.contains("AcpHistory(AcpHistoryMatch)"));
    assert!(types.contains("\"acp-history/{}\""));
    assert!(types.contains("SearchResult::AcpHistory(_) => None"));
    assert!(types.contains("SearchResult::AcpHistory(_) => \"Resume Conversation\""));
}

#[test]
fn root_unified_acp_history_enter_resumes_existing_helper() {
    let selection = include_str!("../../src/app_impl/selection_fallback.rs");
    let acp_history = include_str!("../../src/render_builtins/acp_history.rs");

    assert!(selection.contains("SearchResult::AcpHistory(acp_history_match)"));
    assert!(selection.contains("self.resume_acp_conversation_from_history("));
    assert!(acp_history.contains("pub(crate) fn resume_acp_conversation_from_history("));
}
