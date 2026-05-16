//! Source-audit contract for `unifiedSearch.sourceFilterBrowse`.
//!
//! Scope: prove the static structure of the source-filter-only browse
//! override is in place — config defaults, detector, per-source cap
//! override, passive-budget raise, lazy-source hint snapshot, no
//! window-resize calls — so the runtime behaviour cannot silently
//! regress past `source checks`.

const DEFAULTS: &str = include_str!("../../src/config/defaults.rs");
const TYPES: &str = include_str!("../../src/config/types.rs");
const SCHEMA: &str = include_str!("../../scripts/config-schema.ts");
const FILTERING_CACHE: &str = include_str!("../../src/app_impl/filtering_cache.rs");
const BROWSE_MOD: &str = include_str!("../../src/menu_syntax/source_filter_browse.rs");
const PAYLOAD: &str = include_str!("../../src/menu_syntax/payload.rs");

#[test]
fn defaults_declare_source_filter_browse_constants() {
    for needle in [
        "DEFAULT_UNIFIED_SEARCH_SOURCE_FILTER_BROWSE_ENABLED",
        "DEFAULT_UNIFIED_SEARCH_SOURCE_FILTER_BROWSE_TARGET_VISIBLE_ROWS",
        "MAX_UNIFIED_SEARCH_SOURCE_FILTER_BROWSE_TARGET_VISIBLE_ROWS",
    ] {
        assert!(
            DEFAULTS.contains(needle),
            "src/config/defaults.rs must declare {needle}"
        );
    }
    assert!(
        DEFAULTS.contains(": usize = 14;"),
        "default target visible rows must be 14"
    );
    assert!(
        DEFAULTS.contains(": usize = 32;"),
        "max target visible rows must be 32"
    );
}

#[test]
fn config_types_declare_browse_structs_and_accessor() {
    for needle in [
        "pub struct UnifiedSearchSourceFilterBrowseConfig",
        "pub struct UnifiedSearchSourceFilterBrowseSourcesConfig",
        "pub struct UnifiedSearchSourceFilterBrowseSourceOverride",
        "pub struct UnifiedSearchSourceFilterBrowseResolvedConfig",
        "fn source_filter_browse(",
    ] {
        assert!(
            TYPES.contains(needle),
            "src/config/types.rs must contain `{needle}`"
        );
    }
    assert!(
        TYPES.contains(".clamp(1, max_rows)"),
        "source_filter_browse accessor must clamp targets to 1..=32"
    );
}

#[test]
fn config_schema_ts_exposes_source_filter_browse() {
    for needle in [
        "sourceFilterBrowse?: UnifiedSearchSourceFilterBrowseConfig",
        "export interface UnifiedSearchSourceFilterBrowseConfig",
        "export interface UnifiedSearchSourceFilterBrowseSourcesConfig",
        "export interface UnifiedSearchSourceFilterBrowseSourceOverride",
        "targetVisibleRows?: number",
    ] {
        assert!(
            SCHEMA.contains(needle),
            "scripts/config-schema.ts must contain `{needle}`"
        );
    }
}

#[test]
fn payload_exposes_positive_includes_helpers() {
    for needle in ["fn has_positive_includes", "fn positive_includes"] {
        assert!(
            PAYLOAD.contains(needle),
            "RootUnifiedSourceFilterSet must expose `{needle}`"
        );
    }
}

#[test]
fn browse_mode_detector_enforces_positive_includes_only() {
    for needle in [
        "pub struct SourceFilterBrowseMode",
        "fn from_query",
        "search_text.trim().is_empty()",
        "has_positive_includes()",
        "active_passive_source_count",
        "if advanced_predicate_active",
    ] {
        assert!(
            BROWSE_MOD.contains(needle),
            "source_filter_browse.rs must contain `{needle}`"
        );
    }
}

#[test]
fn filtering_cache_overrides_per_source_caps_in_browse_mode() {
    assert!(
        FILTERING_CACHE.contains("SourceFilterBrowseMode::from_query"),
        "filtering_cache.rs must detect browse mode via SourceFilterBrowseMode::from_query"
    );
    for source in [
        "Src::Notes",
        "Src::ClipboardHistory",
        "Src::Dictation",
        "Src::Conversations",
        "Src::AiVault",
        "Src::BrowserTabs",
        "Src::BrowserHistory",
    ] {
        let needle = format!("browse.target_for({source})");
        assert!(
            FILTERING_CACHE.contains(&needle),
            "filtering_cache.rs must override max_results for {source} via `{needle}`"
        );
    }
    assert!(
        FILTERING_CACHE.contains("root_passive_result_limits.max_total_results = raised_total"),
        "filtering_cache.rs must raise the passive total to target * active_source_count"
    );
}

#[test]
fn filtering_cache_publishes_lazy_hint_snapshot() {
    assert!(
        FILTERING_CACHE.contains("compute_source_filter_browse_hint"),
        "filtering_cache.rs must expose a hint computation helper"
    );
    assert!(
        FILTERING_CACHE.contains("self.source_filter_browse_hint ="),
        "browse mode must store its hint snapshot on the app state"
    );
    for needle in [
        "root_browser_tabs_snapshot_status",
        "root_browser_history_snapshot_status",
        "SourceFilterBrowseHintSnapshot::warming",
        "SourceFilterBrowseHintSnapshot::type_to_load_more",
        "SourceFilterBrowseHintSnapshot::showing_recent",
    ] {
        assert!(
            FILTERING_CACHE.contains(needle),
            "hint helper must consult `{needle}`"
        );
    }
}

#[test]
fn source_filter_browse_path_never_calls_window_resize_apis() {
    for forbidden in [
        "update_window_size_deferred",
        "update_window_size(",
        "resize_to_view_sync(",
    ] {
        assert!(
            !BROWSE_MOD.contains(forbidden),
            "source_filter_browse.rs must not call `{forbidden}`"
        );
    }
    // filtering_cache.rs has unrelated callers — guard only the browse helper.
    let helper = FILTERING_CACHE
        .split("fn compute_source_filter_browse_hint(")
        .nth(1)
        .and_then(|rest| rest.split("\nfn ").next())
        .expect("compute_source_filter_browse_hint must exist");
    for forbidden in [
        "update_window_size_deferred",
        "update_window_size(",
        "resize_to_view_sync(",
    ] {
        assert!(
            !helper.contains(forbidden),
            "compute_source_filter_browse_hint must not call `{forbidden}`"
        );
    }
}
