use super::read_source;

#[test]
fn root_unified_ai_vault_contract() {
    // `payload.rs` owns the committed-head spec table (`SOURCE_HEAD_SPECS`);
    // the abandoned `source_heads.rs` split was deleted as an uncompiled
    // orphan.
    let payload = read_source("src/menu_syntax/payload.rs");
    let payload = payload.as_str();
    let config = read_source("src/config/types.rs");
    let defaults = read_source("src/config/defaults.rs");
    let ai_vault = read_source("src/ai_vault.rs");
    let grouping = read_source("src/scripts/grouping.rs");
    let app_state = read_source("src/main_sections/app_state.rs");
    let filtering_cache = read_source("src/app_impl/filtering_cache.rs");
    let search_mode = read_source("src/scripts/grouping/search_mode.rs");
    let scripts_tests = read_source("src/scripts/tests/chunk_12.rs");
    let actions = read_source("src/app_impl/root_unified_result_actions.rs");
    let selection = read_source("src/app_impl/selection_fallback.rs");
    let preflight = read_source("src/main_window_preflight/build.rs");
    let trigger_registry = read_source("src/builtins/trigger_registry.rs");
    let builtin_execution = read_source("src/app_execute/builtin_execution.rs");
    let filter_matrix = read_source("scripts/agentic/root-source-filter-matrix.ts");
    let actions_matrix = read_source("scripts/agentic/root-source-actions-matrix.ts");

    assert!(payload.contains("RootUnifiedSourceFilter::AiVault"));
    assert!(payload.contains("canonical: \"vault:\""));
    assert!(payload.contains("short: Some(\"v:\")"));
    assert!(payload.contains("Self::AiVault => \"vault\""));

    assert!(defaults.contains("DEFAULT_UNIFIED_SEARCH_AI_VAULT_ENABLED: bool = true"));
    assert!(config.contains("pub ai_vault: UnifiedSearchAiVaultConfig"));
    assert!(config.contains("AiVaultProvider"));
    assert!(config.contains("AiVaultResumeTerminal"));
    assert!(config.contains("pub exclude_patterns: Vec<String>"));
    assert!(config.contains("Self::Claude"));
    assert!(config.contains("Self::Codex"));
    assert!(config.contains("Self::AiVault"));
    assert!(config.contains("fn ai_vault_section_options("));
    assert!(config.contains("min_query_chars.clamp(3, 32)"));
    assert!(config.contains("cache_ttl_ms.clamp(5_000, 120_000)"));
    assert!(config.contains("exclude_patterns: self.ai_vault.exclude_patterns.clone()"));

    assert!(ai_vault.contains("SCRIPT_KIT_AI_VAULT_TEST_PROVIDER"));
    assert!(ai_vault.contains("exclude_patterns: Vec<String>"));
    assert!(ai_vault.contains("fn apply_ai_vault_exclusions("));
    assert!(ai_vault.contains("fn ai_vault_hit_excluded("));
    assert!(ai_vault.contains("fn ai_vault_pattern_matches_hit("));
    assert!(ai_vault.contains("fn ai_vault_pattern_matches_metadata("));
    assert!(ai_vault.contains("fn wildcard_match("));
    assert!(ai_vault.contains("apply_ai_vault_exclusions(&mut hits, &options);"));
    assert!(ai_vault.contains("fn search_local_vault("));
    assert!(ai_vault.contains("fn local_vault_index("));
    assert!(ai_vault.contains("fn ai_vault_index_cache_key("));
    assert!(ai_vault.contains("fn ai_vault_exclude_fingerprint("));
    assert!(ai_vault.contains("exclude={}\""));
    assert!(ai_vault.contains("cmux_search_limit(&options)"));
    assert!(ai_vault.contains("local_vault_index(options.clone())"));
    assert!(ai_vault.contains("provider_enabled(options, \"claude\")"));
    assert!(ai_vault.contains("provider_enabled(options, \"codex\")"));
    assert!(ai_vault.contains("hits.extend(read_claude_vault_hits()?);"));
    assert!(
        ai_vault.contains("hits.extend(read_codex_vault_hits(options, codex_row_limit(mode))?);")
    );
    assert!(ai_vault.contains("spawn_warm_ai_vault_index"));
    assert!(ai_vault.contains("FAST_CODEX_ROW_LIMIT"));
    assert!(ai_vault.contains("SYNC_CONTENT_SCAN_LIMIT"));
    assert!(ai_vault.contains("append_bounded_content_matches"));
    let search_local_vault_body = ai_vault
        .split("fn search_local_vault(")
        .nth(1)
        .and_then(|tail| tail.split("fn local_vault_index(").next())
        .expect("search_local_vault body should be present");
    assert!(!search_local_vault_body.contains("append_bounded_content_matches"));
    assert!(!search_local_vault_body.contains("hydrate_rollout_search_terms"));
    assert!(search_local_vault_body.contains("apply_ai_vault_exclusions(&mut hits, &options);"));
    assert!(search_local_vault_body.contains("apply_local_vault_query_match"));
    assert!(ai_vault.contains("search_haystack"));
    assert!(ai_vault.contains("root_ai_vault_snapshot_status"));
    assert!(ai_vault.contains("ai_vault_cache_generation"));
    assert!(ai_vault.contains("fn read_claude_vault_hits("));
    assert!(ai_vault.contains("fn read_codex_vault_hits("));
    assert!(ai_vault.contains("fn read_codex_vault_hits_via_state_db("));
    assert!(ai_vault.contains("fn read_codex_vault_hits_from_session_index("));
    assert!(ai_vault.contains("state_5.sqlite"));
    assert!(ai_vault.contains("FROM threads"));
    assert!(ai_vault.contains("rollout_path"));
    assert!(ai_vault.contains("copy_sqlite_db_snapshot"));
    assert!(ai_vault.contains("hydrate_rollout_search_terms"));
    assert!(ai_vault.contains("ai_vault_codex_state_db_unavailable"));
    assert!(ai_vault.contains("ai_vault_codex_state_db_unsupported"));
    assert!(ai_vault.contains("fn search_cmux_vault("));
    assert_eq!(
        ai_vault.matches("search_cmux_vault(").count(),
        1,
        "search_cmux_vault must remain defined exactly once with no in-file callers"
    );
    assert!(ai_vault.contains("fn resume_local_vault_session("));
    assert!(ai_vault.contains("fn local_resume_command("));
    assert!(ai_vault.contains(".arg(\"new-workspace\")"));
    assert!(ai_vault.contains("claude --resume"));
    assert!(ai_vault.contains("codex resume"));
    assert!(ai_vault.contains("\"type\": \"aiVault.search.v1\""));
    assert!(ai_vault.contains("\"includeContent\": false"));
    assert!(ai_vault.contains("\"type\": \"aiVault.resume.v1\""));
    assert!(ai_vault.contains("#[serde(default)]"));
    assert!(ai_vault.contains("#[default]"));
    assert!(ai_vault.contains("ai_vault_cache_get"));
    assert!(ai_vault.contains("output_with_timeout"));
    assert!(ai_vault.contains("cmux_failure_message"));
    assert!(ai_vault.contains("ai_vault_local_search_unavailable"));
    assert!(ai_vault.contains("ai_vault_cmux_response_parse_failed"));
    assert!(!ai_vault.contains("transcript:"));
    assert!(!ai_vault.contains("preview:"));
    let config_schema = read_source("scripts/config-schema.ts");
    assert!(config_schema.contains("excludePatterns?: string[]"));
    assert!(config_schema.contains("Metadata-only patterns"));
    assert!(config_schema.contains("title:private*"));
    assert!(config_schema.contains("workspace:~/dev/secret*"));
    assert!(read_source("scripts/config-cli.ts").contains("excludePatterns?: string[]"));
    assert!(read_source("src/designs/core/render.rs").contains("ai_vault_provider_svg_icon"));
    assert!(read_source("src/designs/core/render.rs").contains("ai_provider_openai.svg"));
    assert!(read_source("src/designs/core/render.rs").contains("ai_provider_claude.svg"));
    assert!(read_source("src/designs/core/render.rs").contains("ai_provider_atlassian.svg"));

    assert!(grouping.contains("append_root_ai_vault_section"));
    assert!(grouping.contains(
        "root_source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::AiVault)"
    ));
    assert!(grouping.contains("includes(crate::menu_syntax::RootUnifiedSourceFilter::AiVault)"));
    assert!(grouping.contains("SearchResult::AiVault"));
    assert!(grouping.contains("RootUnifiedSourceFilter::AiVault"));
    assert!(app_state.contains("ai_vault_snapshot_generation"));
    assert!(filtering_cache.contains("root_ai_vault_snapshot_status"));
    assert!(filtering_cache.contains("ai-vault-gen="));

    for id in [
        "root_ai_vault_paste_resume_command",
        "root_ai_vault_copy_resume_command",
        "root_ai_vault_resume_configured_terminal",
        "root_ai_vault_configure_terminal",
        "root_ai_vault_resume_quick_terminal",
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
    assert!(selection.contains("execute_root_ai_vault_paste_resume_command"));
    assert!(preflight.contains("PasteResumeCommand"));
    assert!(trigger_registry.contains("TriggerBuiltin::AiVault => \"builtin/vault\""));
    assert!(trigger_registry
        .contains("TriggerBuiltin::AiVault => &[\"vault\", \"ai-vault\", \"aivault\"]"));
    assert!(search_mode.contains("reserved_exact_builtin_preferred_result_key"));
    assert!(search_mode.contains("\"vault\" | \"ai-vault\" | \"aivault\""));
    assert!(search_mode.contains("BuiltInFeature::AiVault"));
    assert!(scripts_tests
        .contains("test_get_grouped_results_ai_vault_builtin_beats_stale_script_history"));
    assert!(scripts_tests.contains("script/main:Vault"));
    assert!(scripts_tests.contains("builtin/vault"));
    assert!(builtin_execution.contains("fn open_ai_vault_source_filter("));
    assert!(builtin_execution.contains("let filter_text = \"vault: \".to_string();"));
    assert!(builtin_execution.contains("Search AI Vault sessions..."));
    assert!(filter_matrix.contains("SCRIPT_KIT_AI_VAULT_TEST_PROVIDER"));
    assert!(filter_matrix.contains("heads: [\"v:\", \"vault:\"]"));
    assert!(filter_matrix.contains("sourceName: \"AI Vault\""));
    assert!(filter_matrix.contains("codex-sql-title-match"));
    assert!(filter_matrix.contains("Claude SQL source filter"));
    assert!(filter_matrix.contains("POISON_TRANSCRIPT"));
    assert!(filter_matrix.contains("AI Vault receipt leaked poison metadata"));
    assert!(actions_matrix.contains("SCRIPT_KIT_AI_VAULT_TEST_PROVIDER"));
    assert!(actions_matrix.contains("codex-sql-title-match"));
    assert!(actions_matrix.contains("claude-source-actions"));
    assert!(actions_matrix.contains("SCRIPT_KIT_CMUX_COMMAND"));
    assert!(actions_matrix.contains("root_ai_vault_paste_resume_command"));
    assert!(actions_matrix.contains("root_ai_vault_copy_resume_command"));
    assert!(actions_matrix.contains("root_ai_vault_resume_configured_terminal"));
    assert!(actions_matrix.contains("root_ai_vault_resume_quick_terminal"));
    assert!(actions_matrix.contains("\"terminalRouting\":\"userPreferred\""));
    assert!(actions_matrix.contains("cmuxRequests.includes(\"transcript\")"));
    assert!(actions_matrix.contains("containsResumeCommand"));
    assert!(actions_matrix.contains("AI Vault cmux request leaked sensitive fields"));
    assert!(
        read_source("scripts/agentic/root-ai-vault-codex-perf.ts").contains("aiVault.selection.v1")
    );
    assert!(read_source("scripts/agentic/root-ai-vault-perf-matrix.ts").contains("aiVault.perf.v1"));
}
