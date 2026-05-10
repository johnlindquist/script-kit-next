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
