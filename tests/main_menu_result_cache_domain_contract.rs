//! Source-level contract for AURP-12/AURP-13 main-menu result-cache ownership.
//!
//! Main-menu search and grouped-result caches should stay behind one named
//! owner with behavior-named accessors so agents can identify cache
//! invalidation and read paths without reconstructing loose `ScriptListApp`
//! fields.

const APP_STATE: &str = include_str!("../src/main_sections/app_state.rs");
const STARTUP: &str = include_str!("../src/app_impl/startup.rs");
const STARTUP_NEW_PRELUDE: &str = include_str!("../src/app_impl/startup_new_prelude.rs");
const STARTUP_NEW_STATE: &str = include_str!("../src/app_impl/startup_new_state.rs");
const FILTERING_CACHE: &str = include_str!("../src/app_impl/filtering_cache.rs");
const UI_WINDOW: &str = include_str!("../src/app_impl/ui_window.rs");
const MAIN_WINDOW_PREFLIGHT: &str = include_str!("../src/main_window_preflight/build.rs");
const TAB_AI_MODE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const SELECTION_FALLBACK: &str = include_str!("../src/app_impl/selection_fallback.rs");
const PREVIEW_PANEL: &str = include_str!("../src/app_render/preview_panel.rs");
const IMPL_MOVEMENT: &str = include_str!("../src/app_navigation/impl_movement.rs");
const IMPL_SCROLL: &str = include_str!("../src/app_navigation/impl_scroll.rs");
const BUILTIN_EXECUTION: &str = include_str!("../src/app_execute/builtin_execution.rs");

fn source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end: {end_pat}"));
    &source[start..start + end_rel]
}

fn source_contains_ignoring_whitespace(source: &str, needle: &str) -> bool {
    let compact_source: String = source.chars().filter(|c| !c.is_whitespace()).collect();
    let compact_needle: String = needle.chars().filter(|c| !c.is_whitespace()).collect();
    compact_source.contains(&compact_needle)
}

#[test]
fn result_cache_owner_names_behavior_accessors() {
    let cache_impl = source_between(
        APP_STATE,
        "impl MainMenuResultCacheState {",
        "\n}\n\nstruct ScriptListApp",
    );

    for accessor in [
        "fn has_filtered_results_for(&self, filter_text: &str) -> bool",
        "fn filtered_cache_key(&self) -> &str",
        "fn clone_filtered_results(&self) -> Vec<scripts::SearchResult>",
        "fn filtered_results(&self) -> &Vec<scripts::SearchResult>",
        "fn store_filtered_results(",
        "fn has_grouped_results_for(&self, computed_filter_text: &str) -> bool",
        "fn grouped_cache_key(&self) -> &str",
        "fn clone_grouped_results(",
        "fn grouped_items(&self) -> &[GroupedListItem]",
        "fn grouped_flat_results(&self) -> &[scripts::SearchResult]",
        "fn grouped_flat_result_count(&self) -> usize",
        "fn flat_result_index_for_grouped_item(&self, grouped_index: usize) -> Option<usize>",
        "fn flat_result_index_for_coerced_grouped_selection(",
        "fn search_result_for_flat_index(",
        "fn cloned_search_result_for_flat_index(",
        "fn search_result_for_grouped_item(",
        "fn cloned_search_result_for_grouped_item(",
        "fn first_search_result_at_or_after_grouped_item(",
        "fn cloned_first_search_result_at_or_after_grouped_item(",
        "fn grouped_search_results(&self) -> impl Iterator<Item = &scripts::SearchResult>",
        "fn selectable_bounds(&self) -> (Option<usize>, Option<usize>)",
        "fn first_selectable_index(&self) -> Option<usize>",
        "fn last_selectable_index(&self) -> Option<usize>",
        "fn has_selectable_grouped_item(&self) -> bool",
        "fn store_grouped_results(",
        "fn mark_apps_loaded(&mut self)",
        "fn invalidate_filtered_results(&mut self)",
        "fn invalidate_grouped_results(&mut self)",
    ] {
        assert!(
            cache_impl.contains(accessor),
            "MainMenuResultCacheState must expose behavior accessor: {accessor}"
        );
    }
}

// doc-anchor-removed: [[removed-docs State Domains]]
#[test]
fn result_caches_have_a_named_domain_owner() {
    let cache_body = source_between(
        APP_STATE,
        "struct MainMenuResultCacheState {",
        "\n}\n\nimpl Default for MainMenuResultCacheState",
    );
    for field in [
        "cached_filtered_results: Vec<scripts::SearchResult>,",
        "filter_cache_key: String,",
        "cached_grouped_items: Arc<[GroupedListItem]>,",
        "cached_grouped_flat_results: Arc<[scripts::SearchResult]>,",
        "cached_grouped_first_selectable_index: Option<usize>,",
        "cached_grouped_last_selectable_index: Option<usize>,",
        "grouped_cache_key: String,",
    ] {
        assert!(
            cache_body.contains(field),
            "MainMenuResultCacheState must own {field}"
        );
    }

    let app_body = source_between(
        APP_STATE,
        "struct ScriptListApp {",
        "\n}\n\n#[derive(Clone, Debug)]\nstruct AttachmentPortalHostSnapshot",
    );
    assert!(
        app_body.contains("main_menu_result_caches: MainMenuResultCacheState,"),
        "ScriptListApp must expose one named result-cache owner field."
    );
    for loose_field in [
        "cached_filtered_results: Vec<scripts::SearchResult>,",
        "filter_cache_key: String,",
        "cached_grouped_items: Arc<[GroupedListItem]>,",
        "cached_grouped_flat_results: Arc<[scripts::SearchResult]>,",
        "cached_grouped_first_selectable_index: Option<usize>,",
        "cached_grouped_last_selectable_index: Option<usize>,",
        "grouped_cache_key: String,",
    ] {
        assert!(
            !app_body.contains(loose_field),
            "ScriptListApp must not keep {loose_field} as loose state."
        );
    }
}

#[test]
fn startup_and_background_app_loading_route_through_cache_owner() {
    assert!(
        STARTUP.contains("main_menu_result_caches: MainMenuResultCacheState::default(),"),
        "Live startup must initialize result caches through the named default."
    );
    assert!(
        STARTUP_NEW_STATE.contains("main_menu_result_caches: MainMenuResultCacheState::default(),"),
        "Legacy startup state must preserve source-audit parity with the named default."
    );
    assert!(
        STARTUP.contains("app.main_menu_result_caches.mark_apps_loaded();")
            && STARTUP_NEW_PRELUDE.contains("app.main_menu_result_caches.mark_apps_loaded();"),
        "Background app loading must invalidate both cache keys through the owner."
    );
}

#[test]
fn filtering_cache_mutation_routes_through_cache_owner() {
    for required in [
        "self.main_menu_result_caches.has_filtered_results_for(",
        "self.main_menu_result_caches.clone_filtered_results()",
        "self.main_menu_result_caches.filtered_cache_key()",
        ".store_filtered_results(",
        "self.main_menu_result_caches.filtered_results()",
        "self.main_menu_result_caches.has_grouped_results_for(",
        "self.main_menu_result_caches.clone_grouped_results()",
        ".store_grouped_results(",
        "self.main_menu_result_caches.grouped_items().len()",
        "self.main_menu_result_caches.grouped_flat_result_count()",
        "self.main_menu_result_caches.invalidate_filtered_results();",
        "self.main_menu_result_caches.invalidate_grouped_results();",
    ] {
        assert!(
            source_contains_ignoring_whitespace(FILTERING_CACHE, required),
            "filtering_cache.rs must route through {required}"
        );
    }
}

#[test]
fn grouped_cache_readers_use_behavior_named_accessors() {
    for (name, source, required_accessors) in [
        (
            "ui_window",
            UI_WINDOW,
            &[
                ".grouped_items()",
                ".flat_result_index_for_grouped_item(",
                ".search_result_for_flat_index(",
            ][..],
        ),
        (
            "main_window_preflight",
            MAIN_WINDOW_PREFLIGHT,
            &[".cloned_first_search_result_at_or_after_grouped_item("][..],
        ),
        (
            "tab_ai_mode",
            TAB_AI_MODE,
            &[
                ".search_result_for_grouped_item(",
                ".grouped_search_results()",
            ][..],
        ),
        (
            "selection_fallback",
            SELECTION_FALLBACK,
            &[
                ".flat_result_index_for_coerced_grouped_selection(",
                ".cloned_search_result_for_flat_index(",
            ][..],
        ),
        (
            "preview_panel",
            PREVIEW_PANEL,
            &[
                ".flat_result_index_for_grouped_item(",
                ".cloned_search_result_for_flat_index(",
            ][..],
        ),
        (
            "impl_movement",
            IMPL_MOVEMENT,
            &[".first_selectable_index()", ".last_selectable_index()"][..],
        ),
        (
            "impl_scroll",
            IMPL_SCROLL,
            &[
                ".first_selectable_index()",
                ".last_selectable_index()",
                ".has_selectable_grouped_item()",
            ][..],
        ),
        (
            "builtin_execution",
            BUILTIN_EXECUTION,
            &[".grouped_cache_key()"][..],
        ),
    ] {
        for required in required_accessors {
            assert!(
                source.contains(required),
                "{name} must read grouped cache state through behavior-named accessor {required}."
            );
        }
    }
}

#[test]
fn submit_selection_reads_grouped_cache_through_domain_guard() {
    assert!(
        SELECTION_FALLBACK.contains("self.filter_text != self.computed_filter_text")
            && SELECTION_FALLBACK.contains("has_grouped_results_for(&self.computed_filter_text)")
            && SELECTION_FALLBACK
                .contains("flat_result_index_for_coerced_grouped_selection(self.selected_index)"),
        "ScriptList submit must prove live filter, computed filter, grouped cache key, and selected grouped row before dispatch"
    );
}

#[test]
fn production_cache_consumers_do_not_read_storage_fields_directly() {
    for (name, source) in [
        ("filtering_cache", FILTERING_CACHE),
        ("ui_window", UI_WINDOW),
        ("main_window_preflight", MAIN_WINDOW_PREFLIGHT),
        ("tab_ai_mode", TAB_AI_MODE),
        ("selection_fallback", SELECTION_FALLBACK),
        ("preview_panel", PREVIEW_PANEL),
        ("impl_movement", IMPL_MOVEMENT),
        ("impl_scroll", IMPL_SCROLL),
        ("builtin_execution", BUILTIN_EXECUTION),
    ] {
        assert!(
            !source.contains("main_menu_result_caches.cached_"),
            "{name} must not read MainMenuResultCacheState storage fields directly."
        );
        assert!(
            !source.contains("main_menu_result_caches.filter_cache_key"),
            "{name} must use filtered_cache_key() instead of direct key storage."
        );
        for line in source.lines() {
            assert!(
                !line.contains("main_menu_result_caches.grouped_cache_key")
                    || line.contains("main_menu_result_caches.grouped_cache_key()"),
                "{name} must use grouped_cache_key() instead of direct key storage: {line}"
            );
        }
    }
}
