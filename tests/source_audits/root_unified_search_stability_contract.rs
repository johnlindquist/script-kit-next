use std::fs;

#[test]
fn selection_snapshots_use_stable_selection_keys_not_history_memory() {
    let app_state =
        fs::read_to_string("src/main_sections/app_state.rs").expect("read app_state.rs");
    let types = fs::read_to_string("src/scripts/types.rs").expect("read src/scripts/types.rs");

    assert!(types.contains("pub fn stable_selection_key(&self) -> Option<String>"));
    let rows = fs::read_to_string("src/scripts/main_menu_rows.rs").expect("read main_menu_rows.rs");
    assert!(
        app_state.contains("grouped_index_for_stable_selection_key")
            && rows.contains(".stable_selection_key()")
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
        .find(".has_grouped_results_for(&grouped_cache_key)")
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
    assert!(types.contains("pub selected_result_role: Option<MainWindowPreflightResultRole>"));
    assert!(types.contains("pub visible_results: Vec<MainWindowPreflightVisibleResult>"));
    assert!(types.contains("pub visible_result_key_fingerprint: String"));
    assert!(types.contains("pub visible_row_fingerprint: String"));
    assert!(types.contains("pub visible_result_count: usize"));
    assert!(build.contains("result.stable_selection_key()"));
    assert!(build.contains("visible_result_receipts(app)"));
    assert!(build.contains("visible_result_keys(app).join(\"|\")"));
    assert!(build.contains("visible_row_fingerprint(app)"));
    assert!(build.contains("selected_result_key = ?receipt.selected_result_key"));
    assert!(protocol.contains("mainWindowPreflight"));
    assert!(protocol.contains("rootFileSearch"));
    assert!(prompt_handler.contains("serde_json::to_value(receipt).ok()"));
    assert!(prompt_handler.contains("\"loading\": self.root_search.root_file_provider_loading"));
}

#[test]
fn grouped_cache_store_preserves_selectable_bounds_contract() {
    let app_state =
        fs::read_to_string("src/main_sections/app_state.rs").expect("read app_state.rs");
    let body = app_state
        .split("fn store_grouped_results(")
        .nth(1)
        .and_then(|rest| rest.split("fn mark_apps_loaded").next())
        .expect("store_grouped_results body should exist");

    assert!(
        body.contains("rows.iter().filter(|row| row.eligibility.selectable)"),
        "store_grouped_results must compute displayed selectable bounds through the shared selectable-result predicate"
    );
    assert!(
        !body.contains("if matches!(grouped_item, GroupedListItem::Item(_))"),
        "store_grouped_results must not treat every item row as selectable"
    );
}

#[test]
fn preflight_visible_count_uses_selectable_result_count() {
    let build =
        fs::read_to_string("src/main_window_preflight/build.rs").expect("read preflight builder");

    let normalized = build.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        normalized.contains(
            "visible_result_count: app .main_menu_committed_rows() .iter() .filter(|row| row.eligibility.selectable) .count(),"
        ),
        "main-window preflight visible_result_count must use selectable count, not raw item count"
    );
    let receipts = build
        .split("fn visible_result_receipts(")
        .nth(1)
        .expect("visible_result_receipts should exist");
    assert!(
        receipts.contains(".filter(|row| row.eligibility.selectable)"),
        "preflight visible keys/results must exclude non-selectable placeholder rows"
    );
}

#[test]
fn root_file_frame_key_latches_visible_generation_and_loading() {
    let app_state =
        fs::read_to_string("src/main_sections/app_state.rs").expect("read app_state.rs");
    let filtering = fs::read_to_string("src/app_impl/filtering_cache.rs")
        .expect("read src/app_impl/filtering_cache.rs");

    for required in [
        "pub(crate) search_generation: u64",
        "pub(crate) recent_file_revision: u64",
        "pub(crate) visible_loading: bool",
    ] {
        assert!(
            app_state.contains(required),
            "RootFileFrameKey should include `{required}`"
        );
    }

    for required in [
        "search_generation: self.root_search.root_file_search_generation",
        "recent_file_revision: self.root_search.root_recent_file_revision",
        "visible_loading: self.root_search.root_file_search_loading",
    ] {
        assert!(
            filtering.contains(required),
            "root_file_frame_for_current_query should latch `{required}`"
        );
    }
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
        "SearchResult::AgentChatHistory(_)",
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
        "main_menu_committed_rows()",
        "row.eligibility.selectable",
        "MainMenuRowSubject::SearchResult { flat_index }",
        "visible_rank: row.selectable_ordinal?",
        "grouped_index: row.grouped_index",
        "stable_key: Some(row.stable_key.clone())",
        "role: result_role(result)",
        "action_kind: enter_action_kind(result)",
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
fn script_list_typing_echoes_before_deferred_computed_query() {
    let source = fs::read_to_string("src/app_impl/filter_input_change.rs")
        .expect("read filter_input_change.rs");
    let updates = fs::read_to_string("src/app_impl/filter_input_updates.rs")
        .expect("read filter_input_updates.rs");
    let body_start = source
        .find("pub(crate) fn handle_filter_input_change(")
        .expect("handle_filter_input_change should exist");
    let body_end = source[body_start..]
        .find("/// Describes the source of a file search stream.")
        .map(|offset| body_start + offset)
        .expect("file search stream marker should follow handler");
    let body = &source[body_start..body_end];

    let script_list_tail_start = body
        .find("self.filter_text = new_text.clone();\n        self.sync_menu_syntax_form_inputs_from_filter(window, cx);")
        .expect("ScriptList free-text tail should update canonical filter");
    let script_list_tail = &body[script_list_tail_start..];
    let queue_index = script_list_tail
        .find("self.queue_filter_compute(new_text.clone(), cx);")
        .expect("ScriptList typing should queue computed filter update");

    let notify_index = script_list_tail[..queue_index]
        .find("cx.notify();")
        .expect("ScriptList typing should notify after canonical filter_text changes so the typed glyph can paint before deferred search work");
    let history_index = script_list_tail[..queue_index]
        .find("self.input_history.reset_navigation();")
        .expect("ScriptList typing should reset history after the echo notify");
    assert!(
        notify_index < history_index,
        "ScriptList typing should schedule an immediate echo frame before slower per-query bookkeeping and coalesced compute"
    );

    assert!(
        updates.contains("const FILTER_COMPUTE_DEFER: std::time::Duration = std::time::Duration::from_millis(16);"),
        "ScriptList typing should give the input an echo-frame budget before running foreground filter compute"
    );

    let queue_body = updates
        .split("pub(crate) fn queue_filter_compute(")
        .nth(1)
        .and_then(|section| {
            section
                .split("/// Apply a filter text change synchronously")
                .next()
        })
        .expect("queue_filter_compute body should be present");
    assert!(
        queue_body.contains("self.filter_coalescer.queue(value)")
            && queue_body.contains("timer(FILTER_COMPUTE_DEFER)")
            && queue_body.contains("app.filter_coalescer.take_latest(ticket)")
            && queue_body.contains("app.apply_filter_compute_now(latest, cx);"),
        "ScriptList typing should coalesce rapid input and apply only the latest computed query after an echo-frame defer"
    );

    let apply_body = updates
        .split("fn apply_filter_compute_now(")
        .nth(1)
        .and_then(|section| section.split("pub(crate) fn queue_filter_compute(").next())
        .expect("apply_filter_compute_now body should be present");
    let computed_index = apply_body
        .find("self.computed_filter_text = value.clone();")
        .expect("apply_filter_compute_now should install computed_filter_text");
    let root_file_index = apply_body
        .find("self.maybe_start_root_file_search(&value, cx);")
        .expect("apply_filter_compute_now should start root file frame before reconcile");
    let reconcile_index = apply_body
        .find("self.reconcile_script_list_after_filter_change(\"query_commit\", cx);")
        .expect("apply_filter_compute_now should reconcile the list");

    assert!(
        computed_index < root_file_index && root_file_index < reconcile_index,
        "The coalesced apply step should install computed text and root async-loading state before the reconcile publishes grouped rows"
    );
    // The reconcile owns the single publication notify for the stable frame.
    let reconcile_body = updates
        .split("pub(crate) fn reconcile_script_list_after_filter_change(")
        .nth(1)
        .and_then(|rest| {
            rest.split("pub(crate) fn commit_main_menu_results_refresh(")
                .next()
        })
        .expect("reconcile_script_list_after_filter_change should exist");
    assert!(
        reconcile_body.contains("cx.notify_with_owned_cause(\"mainSearchPublication\", sequence);"),
        "the reconcile should publish the stable frame with one owned notify"
    );
}

#[test]
fn agentic_root_search_visual_stability_proof_captures_native_window_and_logs() {
    let proof = fs::read_to_string("scripts/agentic/root-search-visual-stability.ts")
        .expect("read root-search-visual-stability.ts");

    for required in [
        "Bun.spawn([binary]",
        "macos-input.ts",
        "screencapture",
        "contact-sheet.png",
        "SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER",
        "visibleRowFingerprint",
        "visibleResultKeyFingerprint",
        "observedProviderLoading",
        "observedProviderSettled",
        "cacheResultCount",
        "rootPassiveFrame",
        "visibleResults",
        "visibleRootFileCount",
        "warmProvider",
        "expectVisibleFileResults",
        "warmRootFileProvider",
        "waitForRootFileProviderSettlement",
        "assertInputFramesStable",
        "assertLatency",
        "maxGroupMs",
        "maxHandlerMs",
        "app.log",
        "responses.jsonl",
        "receipt.json",
        "assertStable",
    ] {
        assert!(
            proof.contains(required),
            "visual runtime proof script should contain `{required}`"
        );
    }

    assert!(
        !proof.contains("simulateKey"),
        "visual stability proof should use native macOS input, not protocol simulateKey"
    );
}

#[test]
fn global_root_file_grouping_uses_query_frame_latch() {
    let filtering = fs::read_to_string("src/app_impl/filtering_cache.rs")
        .expect("read src/app_impl/filtering_cache.rs");

    for required in [
        "fn root_file_frame_for_current_query(",
        "if frame.key == key",
        "return frame.clone()",
        "RootFileSectionMode::GlobalQuery",
        "frame.file_results.as_slice()",
        "frame.recent_file_results.as_slice()",
        "frame.visible_loading",
    ] {
        assert!(
            filtering.contains(required),
            "filtering cache should contain `{required}`"
        );
    }
}

#[test]
fn root_file_state_receipt_separates_provider_loading_from_visible_loading() {
    let prompt_handler =
        fs::read_to_string("src/prompt_handler/mod.rs").expect("read prompt handler");

    for required in [
        "\"loading\": self.root_search.root_file_provider_loading",
        "\"visibleLoading\": self.visible_root_file_search_loading()",
        "\"cacheResultCount\": self.active_root_file_cache_result_count()",
        "let main_list_scroll = if script_list_active",
        "Some(self.main_list_scroll_receipt())",
    ] {
        assert!(
            prompt_handler.contains(required),
            "rootFileSearch receipt should contain `{required}`"
        );
    }
}
