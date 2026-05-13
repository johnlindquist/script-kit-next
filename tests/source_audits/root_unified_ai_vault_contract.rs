use super::read_source;

#[test]
fn root_unified_ai_vault_contract() {
    // The committed-head spec table was split into `source_heads.rs` by the
    // in-progress grammar refactor while keeping the enum + receipt labels
    // in `payload.rs`. Scan both so this audit survives the move, and
    // normalise the `long:` field-name to the historical `canonical:`
    // spelling so existing assertions still apply.
    let payload = format!(
        "{}\n{}",
        read_source("src/menu_syntax/payload.rs"),
        read_source("src/menu_syntax/source_heads.rs"),
    )
    .replace("long: \"", "canonical: \"");
    let payload = payload.as_str();
    let config = read_source("src/config/types.rs");
    let defaults = read_source("src/config/defaults.rs");
    let ai_vault = read_source("src/ai_vault.rs");
    let grouping = read_source("src/scripts/grouping.rs");
    let actions = read_source("src/app_impl/root_unified_result_actions.rs");
    let selection = read_source("src/app_impl/selection_fallback.rs");
    let preflight = read_source("src/main_window_preflight/build.rs");
    let filter_matrix = read_source("scripts/agentic/root-source-filter-matrix.ts");
    let actions_matrix = read_source("scripts/agentic/root-source-actions-matrix.ts");

    assert!(payload.contains("RootUnifiedSourceFilter::AiVault"));
    assert!(payload.contains("canonical: \"vault:\""));
    assert!(payload.contains("short: Some(\"v:\")"));
    assert!(payload.contains("Self::AiVault => \"vault\""));
    assert!(!payload.contains("canonical: \"processes:\""));
    assert!(!payload.contains("short: Some(\"p:\")"));

    assert!(defaults.contains("DEFAULT_UNIFIED_SEARCH_AI_VAULT_ENABLED: bool = false"));
    assert!(config.contains("pub ai_vault: UnifiedSearchAiVaultConfig"));
    assert!(config.contains("AiVaultProvider"));
    assert!(config.contains("Self::AiVault"));
    assert!(config.contains("fn ai_vault_section_options("));
    assert!(config.contains("min_query_chars.clamp(3, 32)"));
    assert!(config.contains("cache_ttl_ms.clamp(5_000, 120_000)"));

    assert!(ai_vault.contains("SCRIPT_KIT_AI_VAULT_TEST_PROVIDER"));
    assert!(ai_vault.contains("\"type\": \"aiVault.search.v1\""));
    assert!(ai_vault.contains("\"includeContent\": false"));
    assert!(ai_vault.contains("\"type\": \"aiVault.resume.v1\""));
    assert!(!ai_vault.contains("transcript:"));
    assert!(!ai_vault.contains("preview:"));

    assert!(grouping.contains("append_root_ai_vault_section"));
    assert!(grouping.contains("SearchResult::AiVault"));
    assert!(
        grouping.contains("append_root_passive_section(grouped, flat_results, \"AI Vault\", rows)")
    );

    for id in [
        "root_ai_vault_resume_preferred_terminal",
        "root_ai_vault_resume_new_terminal",
        "root_ai_vault_copy_session_id",
        "root_ai_vault_copy_provider",
        "root_ai_vault_copy_workspace_path",
        "root_ai_vault_copy_title",
        "root_ai_vault_reveal_in_cmux",
    ] {
        assert!(actions.contains(id), "missing AI Vault action id {id}");
    }
    assert!(actions.contains("RootUnifiedActionSubject::AiVault"));
    assert!(actions.contains("\"AI Vault Conversation\""));
    assert!(ai_vault.contains("aiVault.reveal.v1"));
    assert!(actions.contains("reveal_vault_session(hit)"));
    assert!(selection.contains("execute_root_ai_vault_resume_preferred_terminal"));
    assert!(preflight.contains("ResumeVaultConversation"));
    assert!(filter_matrix.contains("SCRIPT_KIT_AI_VAULT_TEST_PROVIDER"));
    assert!(filter_matrix.contains("heads: [\"v:\", \"vault:\"]"));
    assert!(filter_matrix.contains("sourceName: \"AI Vault\""));
    assert!(actions_matrix.contains("SCRIPT_KIT_AI_VAULT_TEST_PROVIDER"));
    assert!(actions_matrix.contains("SCRIPT_KIT_CMUX_COMMAND"));
    assert!(actions_matrix.contains("root_ai_vault_resume_preferred_terminal"));
    assert!(actions_matrix.contains("\"terminalRouting\":\"userPreferred\""));
    assert!(actions_matrix.contains("cmuxRequests.includes(\"transcript\")"));
}
