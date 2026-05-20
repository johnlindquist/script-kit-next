#[test]
fn root_unified_clipboard_history_config_is_default_enabled_and_scoped() {
    let config_types = include_str!("../../src/config/types.rs");
    let config_schema = include_str!("../../scripts/config-schema.ts");
    let defaults = include_str!("../../src/config/defaults.rs");

    assert!(config_types.contains("pub struct UnifiedSearchClipboardHistoryConfig"));
    assert!(config_types.contains("root_clipboard_history_section_options("));
    assert!(config_types.contains("builtins.clipboard_history"));
    assert!(config_schema.contains("clipboardHistory?: UnifiedSearchClipboardHistoryConfig"));
    assert!(defaults.contains("DEFAULT_UNIFIED_SEARCH_CLIPBOARD_HISTORY_ENABLED: bool = true"));
    assert!(config_schema.contains("notes?: UnifiedSearchNotesConfig"));
}

#[test]
fn root_unified_clipboard_history_search_is_metadata_only_and_bounded() {
    let database = include_str!("../../src/clipboard_history/database.rs");
    let types = include_str!("../../src/clipboard_history/types.rs");
    let search_fn = database
        .split("pub fn search_root_clipboard_history_meta(")
        .nth(1)
        .and_then(|rest| rest.split("/// Get just the content for an entry").next())
        .expect("search_root_clipboard_history_meta should exist");

    assert!(types.contains("pub struct RootClipboardHistorySectionOptions"));
    assert!(types.contains("root_clipboard_history_query_is_eligible("));
    assert!(types.contains("root_clipboard_entry_is_eligible("));
    assert!(types.contains(
        "ContentType::Text | ContentType::Link | ContentType::File | ContentType::Color"
    ));
    assert!(search_fn.contains("get_clipboard_history_meta(options.scan_limit, 0)"));
    assert!(search_fn.contains(".filter(root_clipboard_entry_is_eligible)"));
    assert!(search_fn.contains(".take(options.max_results)"));
    assert!(
        !search_fn.contains("get_entry_content("),
        "root clipboard search must not load raw clipboard content during grouping"
    );
}

#[test]
fn root_unified_clipboard_history_uses_passive_grouping_contract() {
    let grouping = include_str!("../../src/scripts/grouping.rs");

    assert!(grouping.contains("fn append_root_clipboard_history_section("));
    assert!(grouping.contains(
        "append_root_passive_section(grouped, flat_results, \"Clipboard History\", rows"
    ));
    assert!(grouping.contains("root_clipboard_history_query_is_eligible("));
    assert!(
        grouping.find("append_root_clipboard_history_section(")
            < grouping.find("append_root_acp_history_section("),
        "clipboard rows should be appended before Agent Chat Conversations"
    );
    assert!(
        grouping.contains("label.starts_with(\"Use \\\"\") && label.ends_with(\"\\\" with...\")"),
        "passive insertion should target the fallback section header, not the first fallback row"
    );
}

#[test]
fn root_unified_clipboard_history_result_is_stable_and_non_bindable() {
    let types = include_str!("../../src/scripts/types.rs");

    assert!(types.contains("pub struct ClipboardHistoryMatch"));
    assert!(types.contains("ClipboardHistory(ClipboardHistoryMatch)"));
    assert!(types.contains("\"clipboard-history/{}\""));
    assert!(types.contains("SearchResult::ClipboardHistory(_) => None"));
    assert!(types.contains("SearchResult::ClipboardHistory(_) => \"Paste Clipboard\""));
}

#[test]
fn root_unified_clipboard_attach_action_is_limited_to_text_submit_content() {
    let actions = include_str!("../../src/app_impl/root_unified_result_actions.rs");
    let clipboard_arm = actions
        .split("RootUnifiedActionSubject::Clipboard(entry) => {")
        .nth(1)
        .and_then(|rest| rest.split("RootUnifiedActionSubject::BrowserTab(_)").next())
        .expect("root unified clipboard actions arm should exist");

    assert!(clipboard_arm.contains("RootUnifiedResultAction::ClipboardAttachToAi"));
    assert!(clipboard_arm.contains("Attach to Agent Chat"));
    assert!(clipboard_arm.contains("entry.content_type"));
    assert!(clipboard_arm.contains("ContentType::Text"));
    assert!(clipboard_arm.contains("ContentType::Link"));
    assert!(clipboard_arm.contains("ContentType::Color"));
    assert!(
        !clipboard_arm.contains("ContentType::File"),
        "root unified attach only submits text; file rows should not advertise attachment"
    );
    assert!(
        !clipboard_arm.contains("ContentType::Image"),
        "root unified attach only submits text; image rows should not advertise attachment"
    );
}

#[test]
fn root_unified_clipboard_history_enter_reuses_existing_paste_contract() {
    let selection = include_str!("../../src/app_impl/selection_fallback.rs");

    assert!(selection.contains("SearchResult::ClipboardHistory(clipboard_match)"));
    assert!(selection.contains("execute_root_clipboard_history_paste("));
    assert!(selection.contains("crate::clipboard_history::copy_entry_to_clipboard(entry_id)"));
    assert!(selection.contains("crate::selected_text::simulate_paste_with_cg()"));
    assert!(selection.contains("self.hide_main_and_reset(cx);"));
}
