const BUS_SOURCE: &str = include_str!("../src/agentic_protocol_bus.rs");
const SCRIPT_CONTEXT_SOURCE: &str = include_str!("../src/actions/builders/script_context.rs");
const ACTIONS_BUILDERS_SOURCE: &str = include_str!("../src/actions/builders.rs");
const ACTIONS_MOD_SOURCE: &str = include_str!("../src/actions/mod.rs");
const ACTIONS_DIALOG_SOURCE: &str = include_str!("../src/actions/dialog.rs");
const ACTION_HANDLER_SOURCE: &str = include_str!("../src/app_actions/handle_action/mod.rs");
const DETACHED_CHAT_WINDOW_SOURCE: &str = include_str!("../src/ai/acp/chat_window.rs");
const ACT_TS: &str = include_str!("../scripts/devtools/act.ts");

#[test]
fn protocol_response_bus_exposes_read_only_history_helpers() {
    for needle in [
        "pub struct ProtocolResponseHistoryEntry",
        "pub struct ProtocolResponseHistorySummary",
        "pub fn protocol_response_history_path()",
        "pub fn load_recent_protocol_response_history(",
        "pub fn load_recent_protocol_response_summaries(",
        "pub fn find_protocol_response_by_request_id(",
        "fn summarize_protocol_response(",
        "response_surface_kind",
        "response_automation_id",
    ] {
        assert!(
            BUS_SOURCE.contains(needle),
            "protocol response bus must expose receipt-history helper: {needle}"
        );
    }
}

#[test]
fn acp_actions_route_exposes_receipt_history_without_new_ui_island() {
    for needle in [
        "pub const ACP_SHOW_RECEIPT_HISTORY_ACTION_ID: &str = \"acp_show_receipt_history\";",
        "pub const ACP_RECEIPT_HISTORY_ROUTE_ID: &str = \"acp:receipt_history\";",
        "pub const ACP_RECEIPT_HISTORY_COPY_ACTION_PREFIX: &str = \"acp_receipt_history:copy:\";",
        "pub const ACP_RECEIPT_HISTORY_ROUTE_LIMIT: usize = 20;",
        "pub(crate) fn acp_receipt_history_copy_action_id(",
        "pub(crate) fn acp_receipt_history_request_id_from_action(",
        "pub(crate) fn get_acp_receipt_history_route()",
        "load_recent_protocol_response_summaries(\n            ACP_RECEIPT_HISTORY_ROUTE_LIMIT,\n        )",
        ".with_section(\"Proof\")",
        ".with_section(\"Receipts\")",
        "No receipt history",
    ] {
        assert!(
            SCRIPT_CONTEXT_SOURCE.contains(needle),
            "Agent Chat Actions must expose receipt history through shared route machinery: {needle}"
        );
    }

    assert!(
        !SCRIPT_CONTEXT_SOURCE.contains("AppView::ReceiptHistory"),
        "receipt history must not introduce a new local AppView island in this slice"
    );
}

#[test]
fn receipt_history_route_is_registered_and_exported_for_action_hosts() {
    for needle in [
        "get_acp_receipt_history_route",
        "acp_receipt_history_request_id_from_action",
        "ACP_RECEIPT_HISTORY_COPY_ACTION_PREFIX",
        "ACP_RECEIPT_HISTORY_ROUTE_ID",
        "ACP_SHOW_RECEIPT_HISTORY_ACTION_ID",
    ] {
        assert!(
            ACTIONS_BUILDERS_SOURCE.contains(needle) && ACTIONS_MOD_SOURCE.contains(needle),
            "receipt-history action API must be re-exported for host dispatch: {needle}"
        );
    }

    assert!(
        ACTIONS_DIALOG_SOURCE.contains("ACP_SHOW_RECEIPT_HISTORY_ACTION_ID")
            && ACTIONS_DIALOG_SOURCE.contains("get_acp_receipt_history_route()")
            && ACTIONS_DIALOG_SOURCE.contains("register_drill_down_route"),
        "ActionsDialog must register receipt history as a drill-down route"
    );
}

#[test]
fn shared_and_detached_acp_hosts_copy_selected_receipt_json() {
    for (label, source, event) in [
        (
            "shared",
            ACTION_HANDLER_SOURCE,
            "event = \"acp_receipt_history_copied\"",
        ),
        (
            "detached",
            DETACHED_CHAT_WINDOW_SOURCE,
            "event = \"detached_action_receipt_history_copied\"",
        ),
    ] {
        assert!(
            source.contains("acp_receipt_history_request_id_from_action(action_id)")
                && source.contains("find_protocol_response_by_request_id(request_id)")
                && source.contains("serde_json::to_string_pretty(&entry)")
                && source.contains("write_to_clipboard")
                && source.contains(event),
            "{label} ACP host must copy selected receipt JSON through the stable action prefix"
        );
    }
}

#[test]
fn devtools_submit_gate_scopes_receipt_history_route_and_copy() {
    for needle in [
        "args.submitIntent === \"receipt-history-route\"",
        "receipt-history-route requires --allow-submit-reason",
        "actionId !== \"acp_show_receipt_history\"",
        "allowedBy: \"submitIntent:receipt-history-route\"",
        "args.submitIntent === \"receipt-history-copy\"",
        "actionId.startsWith(\"acp_receipt_history:copy:\")",
        "allowedBy: \"submitIntent:receipt-history-copy\"",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts must fail closed for receipt-history activation: {needle}"
        );
    }
}
