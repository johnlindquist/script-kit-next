#[test]
fn root_unified_dictation_history_config_is_opt_in_and_bounded() {
    let config_types = include_str!("../../src/config/types.rs");
    let config_schema = include_str!("../../scripts/config-schema.ts");
    let defaults = include_str!("../../src/config/defaults.rs");

    assert!(config_types.contains("pub struct UnifiedSearchDictationHistoryConfig"));
    assert!(config_types.contains("fn dictation_history_section_options("));
    assert!(config_schema.contains("dictationHistory?: UnifiedSearchDictationHistoryConfig"));
    assert!(config_schema.contains("export interface UnifiedSearchDictationHistoryConfig"));
    assert!(defaults.contains("DEFAULT_UNIFIED_SEARCH_DICTATION_HISTORY_ENABLED: bool = false"));
    assert!(defaults.contains("DEFAULT_UNIFIED_SEARCH_DICTATION_HISTORY_MAX_RESULTS: usize = 3"));
    assert!(
        defaults.contains("DEFAULT_UNIFIED_SEARCH_DICTATION_HISTORY_MIN_QUERY_CHARS: usize = 4")
    );
    assert!(defaults.contains("DEFAULT_UNIFIED_SEARCH_DICTATION_HISTORY_SCAN_LIMIT: usize = 200"));
}

#[test]
fn root_unified_dictation_history_search_is_bounded_and_local_only() {
    let history = include_str!("../../src/dictation/history.rs");
    let root_search_fn = history
        .split("pub fn search_root_dictation_history(")
        .nth(1)
        .and_then(|rest| rest.split("fn resource_payload(").next())
        .expect("search_root_dictation_history should exist");

    assert!(history.contains("pub struct RootDictationHistorySectionOptions"));
    assert!(history.contains("pub struct RootDictationHistorySearchHit"));
    assert!(history.contains("root_dictation_history_query_is_eligible("));
    assert!(root_search_fn.contains("load_history()"));
    assert!(root_search_fn.contains(".take(options.scan_limit)"));
    assert!(root_search_fn.contains("rank_history_entries(entries, query, options.max_results)"));
    assert!(root_search_fn.contains("query_len = query.trim().chars().count()"));
    assert!(
        !root_search_fn.contains("query = %query"),
        "root dictation history search should not log raw transcript queries"
    );
    assert!(
        !root_search_fn.contains("transcript:"),
        "root dictation history hits should carry metadata only"
    );
    assert!(
        !root_search_fn.contains("std::fs::read("),
        "root dictation history search should use the compact JSONL loader only"
    );
    assert!(
        !root_search_fn.contains("transcribe"),
        "root dictation history search must not touch audio or transcription paths"
    );
}

#[test]
fn root_unified_dictation_history_uses_passive_grouping_contract() {
    let grouping = include_str!("../../src/scripts/grouping.rs");

    assert!(grouping.contains("fn append_root_dictation_history_section("));
    assert!(grouping.contains(
        "append_root_passive_section(grouped, flat_results, \"Dictation History\", rows"
    ));
    assert!(grouping.contains("root_dictation_history_query_is_eligible("));
    assert!(
        grouping.find("append_root_clipboard_history_section(")
            < grouping.find("append_root_dictation_history_section("),
        "Dictation History rows should be appended after Clipboard History"
    );
    assert!(
        grouping.find("append_root_dictation_history_section(")
            < grouping.find("append_root_acp_history_section("),
        "Dictation History rows should be appended before Agent Chat Conversations"
    );
}

#[test]
fn root_unified_dictation_history_result_is_stable_and_non_bindable() {
    let types = include_str!("../../src/scripts/types.rs");
    let unified = include_str!("../../src/scripts/search/unified.rs");

    assert!(types.contains("pub struct DictationHistoryMatch"));
    assert!(types.contains("DictationHistory(DictationHistoryMatch)"));
    assert!(types.contains("pub(crate) id: String"));
    assert!(
        !types
            .split("pub struct DictationHistoryMatch")
            .nth(1)
            .and_then(|rest| rest.split("/// Represents a passive root-search match for local browser history metadata.").next())
            .unwrap_or_default()
            .contains("transcript"),
        "root DictationHistoryMatch must not carry raw transcript text"
    );
    assert!(types.contains("SearchResult::DictationHistory(_) => None"));
    assert!(types.contains("\"dictation-history/{}\""));
    assert!(types.contains("SearchResult::DictationHistory(_) => \"Paste Dictation\""));
    assert!(types.contains("SearchResult::DictationHistory(_) => (\"Dictation\", 0xFB7185)"));
    assert!(unified.contains("SearchResult::DictationHistory(_) => 10"));
}

#[test]
fn root_unified_dictation_history_enter_reuses_existing_paste_contract() {
    let selection = include_str!("../../src/app_impl/selection_fallback.rs");

    assert!(selection.contains("SearchResult::DictationHistory(dictation_match)"));
    assert!(selection.contains("execute_root_dictation_history_paste("));
    assert!(selection.contains("crate::dictation::get_history_entry(entry_id)"));
    assert!(selection.contains("crate::text_injector::TextInjector::new()"));
    assert!(selection.contains("injector.paste_text(&transcript)"));
}
