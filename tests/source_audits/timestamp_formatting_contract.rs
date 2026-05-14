//! Source-level contract for central timestamp display formatting.

const FORMATTING: &str = include_str!("../../src/formatting.rs");
const ACP_HISTORY_POPUP: &str = include_str!("../../src/ai/acp/history_popup.rs");
const PROCESS_MANAGER: &str = include_str!("../../src/render_builtins/process_manager.rs");
const KIT_STORE: &str = include_str!("../../src/render_builtins/kit_store.rs");

#[test]
fn central_formatting_exposes_rfc3339_date_and_running_duration_helpers() {
    assert!(FORMATTING.contains("pub fn format_rfc3339_date_for_display"));
    assert!(FORMATTING.contains("pub fn format_running_duration"));
}

#[test]
fn acp_and_builtin_display_code_do_not_split_rfc3339_timestamps() {
    assert!(
        !ACP_HISTORY_POPUP.contains(".split('T')") && !ACP_HISTORY_POPUP.contains(".split(\"T\")"),
        "ACP history popup display must use the central RFC3339 date formatter"
    );
    assert!(
        ACP_HISTORY_POPUP.contains("format_rfc3339_date_for_display(&entry.timestamp)"),
        "ACP history popup must call the central RFC3339 display formatter"
    );
}

#[test]
fn process_manager_duration_display_uses_central_helper() {
    assert!(
        PROCESS_MANAGER.contains("format_running_duration("),
        "Process Manager display must use the central duration formatter"
    );
    assert!(
        !PROCESS_MANAGER.contains(".signed_duration_since(process_info.started_at)"),
        "Process Manager renderer must not inline duration math"
    );
}

#[test]
fn raw_rfc3339_storage_paths_must_be_whitelisted() {
    assert!(
        KIT_STORE.contains("LAT_WHITELIST_RFC3339_STORAGE")
            && KIT_STORE.contains("installed_at: chrono::Utc::now().to_rfc3339()"),
        "Kit Store may keep raw RFC3339 only for registry storage with an explicit whitelist"
    );
}
