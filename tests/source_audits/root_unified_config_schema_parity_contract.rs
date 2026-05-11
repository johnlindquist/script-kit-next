struct RootUnifiedSourceSpec {
    rust_field: &'static str,
    schema_field: &'static str,
    config_struct: &'static str,
    default_prefix: &'static str,
    section_options_fn: &'static str,
    grouping_fn: &'static str,
    audit_module: &'static str,
    verification_heading: &'static str,
}

const ROOT_UNIFIED_SOURCES: &[RootUnifiedSourceSpec] = &[
    RootUnifiedSourceSpec {
        rust_field: "files",
        schema_field: "files",
        config_struct: "UnifiedSearchFilesConfig",
        default_prefix: "DEFAULT_UNIFIED_SEARCH_FILES_",
        section_options_fn: "root_file_section_options",
        grouping_fn: "append_root_file_section",
        audit_module: "root_file_search_contract",
        verification_heading: "Root Unified Search Safety Controls",
    },
    RootUnifiedSourceSpec {
        rust_field: "notes",
        schema_field: "notes",
        config_struct: "UnifiedSearchNotesConfig",
        default_prefix: "DEFAULT_UNIFIED_SEARCH_NOTES_",
        section_options_fn: "notes_section_options",
        grouping_fn: "append_root_notes_section",
        audit_module: "root_unified_notes_contract",
        verification_heading: "Root Unified Search Notes",
    },
    RootUnifiedSourceSpec {
        rust_field: "acp_history",
        schema_field: "acpHistory",
        config_struct: "UnifiedSearchAcpHistoryConfig",
        default_prefix: "DEFAULT_UNIFIED_SEARCH_ACP_HISTORY_",
        section_options_fn: "acp_history_section_options",
        grouping_fn: "append_root_acp_history_section",
        audit_module: "root_unified_acp_history_contract",
        verification_heading: "Root Unified Search ACP History",
    },
    RootUnifiedSourceSpec {
        rust_field: "clipboard_history",
        schema_field: "clipboardHistory",
        config_struct: "UnifiedSearchClipboardHistoryConfig",
        default_prefix: "DEFAULT_UNIFIED_SEARCH_CLIPBOARD_HISTORY_",
        section_options_fn: "root_clipboard_history_section_options",
        grouping_fn: "append_root_clipboard_history_section",
        audit_module: "root_unified_clipboard_history_contract",
        verification_heading: "Root Unified Search Clipboard History",
    },
    RootUnifiedSourceSpec {
        rust_field: "dictation_history",
        schema_field: "dictationHistory",
        config_struct: "UnifiedSearchDictationHistoryConfig",
        default_prefix: "DEFAULT_UNIFIED_SEARCH_DICTATION_HISTORY_",
        section_options_fn: "dictation_history_section_options",
        grouping_fn: "append_root_dictation_history_section",
        audit_module: "root_unified_dictation_history_contract",
        verification_heading: "Root Unified Search Dictation History",
    },
    RootUnifiedSourceSpec {
        rust_field: "browser_tabs",
        schema_field: "browserTabs",
        config_struct: "UnifiedSearchBrowserTabsConfig",
        default_prefix: "DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_",
        section_options_fn: "browser_tabs_section_options",
        grouping_fn: "append_root_browser_tabs_section",
        audit_module: "root_unified_browser_tabs_contract",
        verification_heading: "Root Unified Search Browser Tabs",
    },
    RootUnifiedSourceSpec {
        rust_field: "browser_history",
        schema_field: "browserHistory",
        config_struct: "UnifiedSearchBrowserHistoryConfig",
        default_prefix: "DEFAULT_UNIFIED_SEARCH_BROWSER_HISTORY_",
        section_options_fn: "browser_history_section_options",
        grouping_fn: "append_root_browser_history_section",
        audit_module: "root_unified_browser_history_contract",
        verification_heading: "Root Unified Search Browser History",
    },
];

#[test]
fn root_unified_sources_have_rust_schema_default_and_audit_parity() {
    let config_types = include_str!("../../src/config/types.rs");
    let config_defaults = include_str!("../../src/config/defaults.rs");
    let config_schema = include_str!("../../scripts/config-schema.ts");
    let source_audits = include_str!("../source_audits.rs");
    let grouping = include_str!("../../src/scripts/grouping.rs");
    let verification = include_str!("../../lat.md/verification.md");

    assert!(config_types.contains("pub struct UnifiedSearchConfig"));
    assert!(config_schema.contains("export interface UnifiedSearchConfig"));
    assert!(config_defaults.contains("DEFAULT_UNIFIED_SEARCH_ENABLED"));
    assert!(verification.contains("## Root Unified Search Config Parity"));

    for source in ROOT_UNIFIED_SOURCES {
        assert!(
            config_types.contains(&format!(
                "pub {}: {}",
                source.rust_field, source.config_struct
            )),
            "UnifiedSearchConfig missing Rust field `{}` with struct `{}`",
            source.rust_field,
            source.config_struct
        );
        assert!(
            config_types.contains(&format!("pub struct {}", source.config_struct)),
            "src/config/types.rs missing `{}`",
            source.config_struct
        );
        assert!(
            config_types.contains(&format!(
                "{}: {}::default()",
                source.rust_field, source.config_struct
            )),
            "UnifiedSearchConfig::default must initialize `{}` through its source default",
            source.rust_field
        );
        assert!(
            config_types.contains(&format!("fn {}(", source.section_options_fn))
                || config_types.contains(&format!("pub fn {}(", source.section_options_fn)),
            "src/config/types.rs missing section-options accessor `{}`",
            source.section_options_fn
        );
        assert!(
            config_defaults.contains(source.default_prefix),
            "src/config/defaults.rs missing constants with prefix `{}`",
            source.default_prefix
        );
        assert!(
            config_schema.contains(&format!(
                "{}?: {}",
                source.schema_field, source.config_struct
            )),
            "scripts/config-schema.ts missing unifiedSearch.{} schema field for `{}`",
            source.schema_field,
            source.config_struct
        );
        assert!(
            config_schema.contains(&format!("export interface {}", source.config_struct)),
            "scripts/config-schema.ts missing interface `{}`",
            source.config_struct
        );
        assert!(
            source_audits.contains(&format!("mod {};", source.audit_module)),
            "tests/source_audits.rs missing module `{}`",
            source.audit_module
        );
        assert!(
            grouping.contains(&format!("fn {}(", source.grouping_fn)),
            "src/scripts/grouping.rs missing grouping append function `{}`",
            source.grouping_fn
        );
        assert!(
            verification.contains(&format!("## {}", source.verification_heading)),
            "lat.md/verification.md missing verification heading `{}`",
            source.verification_heading
        );
    }
}

#[test]
fn root_unified_user_controls_are_clamped_or_explicitly_policy_gated() {
    let config_types = include_str!("../../src/config/types.rs");
    let config_schema = include_str!("../../scripts/config-schema.ts");

    let section = config_types
        .split("impl UnifiedSearchConfig {")
        .nth(1)
        .and_then(|rest| {
            rest.split("// ============================================")
                .next()
        })
        .expect("UnifiedSearchConfig impl should contain source options accessors");

    assert!(
        section.contains("promotion_policy: self.files.promotion.into()"),
        "file promotion must remain an explicit user policy rather than implicit score tuning"
    );
    assert!(
        config_schema.contains("promotion?: \"never\" | \"exactFilenameOnly\""),
        "config schema must expose the file-promotion policy"
    );

    for required_clamp in [
        "max_results.clamp(1, 5)",
        "min_query_chars.clamp(2, 32)",
        "min_query_chars.clamp(4, 32)",
        "scan_limit.clamp(25, 200)",
        "scan_limit.clamp(25, 2_000)",
        "scan_limit.clamp(10, 250)",
        "cache_ttl_ms.clamp(1_000, 60_000)",
        "cache_ttl_ms.clamp(5_000, 120_000)",
    ] {
        assert!(
            section.contains(required_clamp),
            "UnifiedSearchConfig source options must clamp user control `{required_clamp}`"
        );
    }
    assert!(
        config_types.contains("scan_limit: unified.clipboard_history.scan_limit.clamp(25, 1000)"),
        "clipboard-history options must clamp scan_limit while applying built-in gating"
    );

    for schema_control in [
        "enabled?: boolean",
        "maxResults?: number",
        "minQueryChars?: number",
        "scanLimit?: number",
        "cacheTtlMs?: number",
        "providers?: BrowserTabProvider[]",
        "providers?: BrowserHistoryProvider[]",
    ] {
        assert!(
            config_schema.contains(schema_control),
            "config schema missing user-facing control `{schema_control}`"
        );
    }
}

#[test]
fn root_unified_passive_source_order_is_schema_backed_and_total() {
    let config_types = include_str!("../../src/config/types.rs");
    let config_schema = include_str!("../../scripts/config-schema.ts");
    let filtering_cache = include_str!("../../src/app_impl/filtering_cache.rs");
    let grouping = include_str!("../../src/scripts/grouping.rs");

    assert!(
        config_types.contains("pub enum UnifiedSearchPassiveSource"),
        "Rust config must expose a typed passive source order enum"
    );
    assert!(
        config_types.contains("pub passive_source_order: Vec<UnifiedSearchPassiveSource>"),
        "UnifiedSearchConfig must carry the passive source order"
    );
    assert!(
        config_types.contains(
            "pub(crate) fn passive_source_order(&self) -> Vec<UnifiedSearchPassiveSource>"
        ),
        "UnifiedSearchConfig must normalize passive source order"
    );
    assert!(
        config_types.contains("if seen.insert(*source)")
            && config_types.contains("if seen.insert(source)"),
        "passive_source_order must dedupe configured entries and append missing defaults"
    );
    assert!(
        config_schema.contains("export type UnifiedSearchPassiveSource"),
        "TypeScript config schema must expose the passive source order type"
    );
    assert!(
        config_schema.contains("passiveSourceOrder?: UnifiedSearchPassiveSource[]"),
        "TypeScript config schema must expose unifiedSearch.passiveSourceOrder"
    );
    assert!(
        config_types.contains("pub passive_result_limits: UnifiedSearchPassiveResultLimitsConfig"),
        "UnifiedSearchConfig must carry passive result limits"
    );
    assert!(
        config_types.contains("pub struct UnifiedSearchPassiveResultLimitsConfig"),
        "Rust config must expose passive result limits"
    );
    assert!(
        config_types
            .contains("passive_result_limits: UnifiedSearchPassiveResultLimitsConfig::default()"),
        "UnifiedSearchConfig::default must initialize passive result limits"
    );
    assert!(
        config_types.contains(
            "pub(crate) fn passive_result_limits(&self) -> UnifiedSearchPassiveResultLimitsConfig"
        ),
        "UnifiedSearchConfig must expose normalized passive result limits"
    );
    for required_clamp in [
        "max_total_results.clamp(0, 24)",
        "max_total_results_when_primary_visible\n                .clamp(0, 12)",
        "max_results_per_source_when_primary_visible\n                .clamp(0, 5)",
    ] {
        assert!(
            config_types.contains(required_clamp),
            "passive result limits must clamp `{required_clamp}`"
        );
    }
    assert!(
        config_schema.contains("passiveResultLimits?: UnifiedSearchPassiveResultLimitsConfig"),
        "TypeScript config schema must expose unifiedSearch.passiveResultLimits"
    );
    assert!(
        config_schema.contains("export interface UnifiedSearchPassiveResultLimitsConfig"),
        "TypeScript config schema must expose passive result limits interface"
    );
    assert!(
        config_schema.contains(
            "Reorders passive local source sections only; it does not enable or disable sources."
        ),
        "schema docs must make clear that order does not toggle source enablement"
    );
    assert!(
        filtering_cache
            .contains("let root_passive_source_order = unified_search.passive_source_order()"),
        "filtering cache must read the normalized config order"
    );
    assert!(
        grouping
            .contains("root_passive_source_order: &[crate::config::UnifiedSearchPassiveSource]"),
        "grouping must accept the normalized passive source order"
    );
    assert!(
        grouping.contains("for source in root_passive_source_order"),
        "grouping must iterate the configured passive source order instead of a fixed sequence"
    );

    for (variant, schema_value, grouping_fn) in [
        (
            "BrowserTabs",
            "\"browserTabs\"",
            "append_root_browser_tabs_section",
        ),
        ("Notes", "\"notes\"", "append_root_notes_section"),
        (
            "ClipboardHistory",
            "\"clipboardHistory\"",
            "append_root_clipboard_history_section",
        ),
        (
            "DictationHistory",
            "\"dictationHistory\"",
            "append_root_dictation_history_section",
        ),
        (
            "AcpHistory",
            "\"acpHistory\"",
            "append_root_acp_history_section",
        ),
        (
            "BrowserHistory",
            "\"browserHistory\"",
            "append_root_browser_history_section",
        ),
    ] {
        assert!(
            config_types.contains(&format!("Self::{variant}")),
            "default order must include `{variant}`"
        );
        assert!(
            grouping.contains(&format!(
                "crate::config::UnifiedSearchPassiveSource::{variant}"
            )),
            "grouping match must handle `{variant}`"
        );
        assert!(
            config_schema.contains(schema_value),
            "schema source union must include `{schema_value}`"
        );
        assert!(
            grouping.contains(grouping_fn),
            "grouping must still route `{variant}` through `{grouping_fn}`"
        );
    }
}
