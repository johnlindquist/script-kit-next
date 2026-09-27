const BUS_SOURCE: &str = include_str!("../src/agentic_protocol_bus.rs");
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
fn devtools_submit_gate_scopes_receipt_history_route_and_copy() {
    for needle in [
        "args.submitIntent === \"receipt-history-route\"",
        "receipt-history-route requires --allow-submit-reason",
        "actionId !== \"agent_chat_show_receipt_history\"",
        "allowedBy: \"submitIntent:receipt-history-route\"",
        "args.submitIntent === \"receipt-history-copy\"",
        "actionId.startsWith(\"agent_chat_receipt_history:copy:\")",
        "allowedBy: \"submitIntent:receipt-history-copy\"",
    ] {
        assert!(
            ACT_TS.contains(needle),
            "act.ts must fail closed for receipt-history activation: {needle}"
        );
    }
}
