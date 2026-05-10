use std::fs;

#[test]
fn global_root_file_search_does_not_stream_into_active_frame() {
    let source = fs::read_to_string("src/app_impl/root_file_search.rs")
        .expect("read src/app_impl/root_file_search.rs");
    let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(
        normalized.contains("fn cache_root_file_search_results_for_generation("),
        "global provider completion should have a cache-only path"
    );
    assert!(
        normalized.contains("let publish_active_results =")
            && normalized
                .contains("matches!(&request, RootFileSearchRequest::DirectoryBrowse { .. })"),
        "only explicit directory browse should be allowed to publish into the active frame"
    );
    assert!(
        normalized.contains("app.cache_root_file_search_results_for_generation( generation, request_cache_key, batch, true, );"),
        "global provider completion should warm cache instead of applying visible rows"
    );
    assert!(
        !normalized.contains("publish_partial_results"),
        "root global file search must not publish partial result batches"
    );
}

#[test]
fn selection_snapshots_use_stable_selection_keys_not_history_memory() {
    let app_state =
        fs::read_to_string("src/main_sections/app_state.rs").expect("read app_state.rs");
    let types = fs::read_to_string("src/scripts/types.rs").expect("read src/scripts/types.rs");

    assert!(types.contains("pub fn stable_selection_key(&self) -> Option<String>"));
    assert!(
        app_state.contains("grouped_index_for_stable_selection_key")
            && app_state.contains("result.stable_selection_key()")
            && !app_state.contains("grouped_index_for_history_result_key"),
        "selection restoration should use selection identity, not input-history identity"
    );
    assert!(
        types.contains(
            "SearchResult::Fallback(fm) => Some(format!(\"fallback/{}\", fm.fallback.name()))"
        ) && types.contains("SearchResult::Fallback(_) | SearchResult::Agent(_) => None"),
        "fallback rows need selection keys without becoming input-history promotion keys"
    );
}

#[test]
fn grouped_cache_read_is_pure_before_recent_file_refresh() {
    let filtering = fs::read_to_string("src/app_impl/filtering_cache.rs")
        .expect("read src/app_impl/filtering_cache.rs");

    let cache_check = filtering
        .find(".has_grouped_results_for(&self.computed_filter_text)")
        .expect("grouped cache check should exist");
    let recent_refresh = filtering
        .find("self.refresh_root_recent_file_results();")
        .expect("recent file refresh should exist");

    assert!(
        cache_check < recent_refresh,
        "grouped-result cache hits should return before refreshing recent files"
    );
}

#[test]
fn main_window_preflight_exposes_selection_key_and_frame_fingerprint() {
    let types =
        fs::read_to_string("src/main_window_preflight/types.rs").expect("read preflight types");
    let build =
        fs::read_to_string("src/main_window_preflight/build.rs").expect("read preflight builder");
    let protocol =
        fs::read_to_string("src/protocol/message/variants/query_ops.rs").expect("read protocol");
    let prompt_handler =
        fs::read_to_string("src/prompt_handler/mod.rs").expect("read prompt handler");

    assert!(types.contains("pub selected_result_key: Option<String>"));
    assert!(types.contains("pub selected_result_role: MainWindowPreflightResultRole"));
    assert!(types.contains("pub visible_results: Vec<MainWindowPreflightVisibleResult>"));
    assert!(types.contains("pub visible_result_key_fingerprint: String"));
    assert!(types.contains("pub visible_result_count: usize"));
    assert!(build.contains("result.stable_selection_key()"));
    assert!(build.contains("visible_result_receipts(app)"));
    assert!(build.contains("visible_result_keys(app).join(\"|\")"));
    assert!(build.contains("selected_result_key = ?receipt.selected_result_key"));
    assert!(protocol.contains("mainWindowPreflight"));
    assert!(protocol.contains("rootFileSearch"));
    assert!(prompt_handler.contains("serde_json::to_value(receipt).ok()"));
    assert!(prompt_handler.contains("\"loading\": self.root_file_search_loading"));
}

#[test]
fn preflight_visible_results_expose_search_safety_roles() {
    let types =
        fs::read_to_string("src/main_window_preflight/types.rs").expect("read preflight types");
    let build =
        fs::read_to_string("src/main_window_preflight/build.rs").expect("read preflight builder");

    assert!(types.contains("pub(crate) enum MainWindowPreflightResultRole"));
    assert!(types.contains("pub(crate) struct MainWindowPreflightVisibleResult"));
    for required in [
        "Primary",
        "RootFile",
        "RootPassive",
        "Fallback",
        "ScriptIssue",
        "Agent",
        "pub visible_rank: usize",
        "pub grouped_index: usize",
        "pub stable_key: Option<String>",
        "pub action_kind: MainWindowPreflightActionKind",
        "pub type_label: String",
        "pub source_name: Option<String>",
    ] {
        assert!(
            types.contains(required),
            "preflight types should expose `{required}`"
        );
    }

    let classifier = build
        .split("fn result_role(")
        .nth(1)
        .and_then(|rest| rest.split("fn enter_action_kind(").next())
        .expect("result_role classifier should precede enter_action_kind");
    assert!(
        !classifier.contains("_ =>"),
        "result_role must classify every SearchResult variant explicitly"
    );
    for required in [
        "SearchResult::Script(_)",
        "SearchResult::Scriptlet(_)",
        "SearchResult::Skill(_)",
        "SearchResult::BuiltIn(_)",
        "SearchResult::App(_)",
        "SearchResult::Window(_)",
        "MainWindowPreflightResultRole::Primary",
        "SearchResult::File(_)",
        "MainWindowPreflightResultRole::RootFile",
        "SearchResult::Note(_)",
        "SearchResult::AcpHistory(_)",
        "SearchResult::ClipboardHistory(_)",
        "SearchResult::DictationHistory(_)",
        "SearchResult::BrowserTab(_)",
        "SearchResult::BrowserHistory(_)",
        "MainWindowPreflightResultRole::RootPassive",
        "SearchResult::Fallback(_)",
        "MainWindowPreflightResultRole::Fallback",
        "SearchResult::ScriptIssue(_)",
        "MainWindowPreflightResultRole::ScriptIssue",
        "SearchResult::Agent(_)",
        "MainWindowPreflightResultRole::Agent",
    ] {
        assert!(
            classifier.contains(required),
            "result_role should contain `{required}`"
        );
    }

    let receipt_builder = build
        .split("fn visible_result_receipts(")
        .nth(1)
        .and_then(|rest| rest.split("fn build_tab_action(").next())
        .expect("visible_result_receipts should precede build_tab_action");
    for required in [
        "grouped_items()",
        "search_result_for_flat_index",
        "GroupedListItem::Item(flat_index)",
        "visible_rank",
        "grouped_index",
        "result.stable_selection_key()",
        "result_role(result)",
        "enter_action_kind(result)",
        "result.type_label().to_string()",
        "result.source_name().map(ToString::to_string)",
    ] {
        assert!(
            receipt_builder.contains(required),
            "visible_result_receipts should contain `{required}`"
        );
    }
}

#[test]
fn script_list_typing_does_not_notify_before_computed_query_catches_up() {
    let source = fs::read_to_string("src/app_impl/filter_input_change.rs")
        .expect("read filter_input_change.rs");
    let body_start = source
        .find("pub(crate) fn handle_filter_input_change(")
        .expect("handle_filter_input_change should exist");
    let body_end = source[body_start..]
        .find("/// Describes the source of a file search stream.")
        .map(|offset| body_start + offset)
        .expect("file search stream marker should follow handler");
    let body = &source[body_start..body_end];

    let script_list_tail_start = body
        .find("let previous_text = std::mem::replace(&mut self.filter_text, new_text.clone());")
        .expect("ScriptList free-text tail should update canonical filter");
    let script_list_tail = &body[script_list_tail_start..];
    let queue_index = script_list_tail
        .find("self.queue_filter_compute(new_text.clone(), cx);")
        .expect("ScriptList typing should queue computed filter update");

    assert!(
        !script_list_tail[..queue_index].contains("cx.notify();"),
        "ScriptList typing must not render after filter_text changes but before computed_filter_text/grouped rows catch up"
    );
}

#[test]
fn agentic_root_search_frame_stability_proof_compares_preflight_receipts() {
    let proof = fs::read_to_string("scripts/agentic/root-search-frame-stability.ts")
        .expect("read root-search-frame-stability.ts");

    for required in [
        "setFilter",
        "waitFor",
        "stateMatch",
        "getState",
        "mainWindowPreflight",
        "selectedResultKey",
        "visibleResultKeyFingerprint",
        "enterAction",
        "rootFileSearch",
        "GlobalQuery",
        "loading === false",
    ] {
        assert!(
            proof.contains(required),
            "runtime proof script should contain `{required}`"
        );
    }

    assert!(
        !proof.contains("captureScreenshot") && !proof.contains("simulateClick"),
        "root frame stability proof should stay state-first, not screenshot or mouse based"
    );
}
