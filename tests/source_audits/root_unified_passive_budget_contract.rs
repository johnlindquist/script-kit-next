#[test]
fn every_passive_append_function_consumes_shared_budget() {
    let grouping = include_str!("../../src/scripts/grouping.rs");
    for fn_name in [
        "append_root_browser_tabs_section",
        "append_root_notes_section",
        "append_root_clipboard_history_section",
        "append_root_dictation_history_section",
        "append_root_agent_chat_history_section",
        "append_root_browser_history_section",
    ] {
        let section = grouping
            .split(&format!("fn {fn_name}("))
            .nth(1)
            .and_then(|rest| rest.split("\nfn ").next())
            .unwrap_or_else(|| panic!("{fn_name} should exist"));
        assert!(
            section.contains("budget.limit_for_source("),
            "{fn_name} must compute its source limit from the shared passive budget"
        );
        assert!(
            section.contains("budget.consume(rows.len())"),
            "{fn_name} must consume the shared passive budget after constructing rows"
        );
        assert!(
            section.contains("root_passive_result_score(rank)"),
            "{fn_name} must keep passive scoring capped"
        );
    }
}
