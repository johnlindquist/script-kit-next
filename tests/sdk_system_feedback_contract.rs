const SDK: &str = include_str!("../scripts/kit-sdk.ts");
const EXECUTE_SCRIPT: &str = include_str!("../src/execute_script/mod.rs");
const MCP_RESOURCES: &str = include_str!("../src/mcp_resources/mod.rs");

fn function_body(source: &str, marker: &str) -> String {
    let start = source.find(marker).expect("missing function marker");
    let rest = &source[start..];
    let end = rest.find("\n};").expect("missing function terminator");
    rest[..end].to_string()
}

// @lat: [[lat.md/scripting#Scripting]]
#[test]
fn feedback_dispatch_helpers_do_not_claim_unsupported() {
    for marker in [
        "globalThis.beep = function beep(): Promise<SystemFeedbackResult> {",
        "globalThis.say = function say(text: string, voice?: string): Promise<SystemFeedbackResult> {",
        "globalThis.notify = function notify(options: string | NotifyOptions): Promise<SystemFeedbackResult> {",
    ] {
        let body = function_body(SDK, marker);
        assert!(
            !body.contains("not yet implemented") && !body.contains("rejectUnsupportedSdkFeature"),
            "{marker} must stay a supported dispatch wrapper"
        );
        assert!(
            body.contains("requestSystemFeedback"),
            "{marker} must wait for a runtime dispatch receipt"
        );
    }
}

// @lat: [[lat.md/scripting#Scripting]]
#[test]
fn setstatus_and_menu_reject_before_send() {
    for (marker, feature) in [
        (
            "globalThis.setStatus = function setStatus(_options: StatusOptions): Promise<SystemFeedbackResult> {",
            "setStatus",
        ),
        (
            "globalThis.menu = function menu(_icon: string, _scripts?: string[]): Promise<SystemFeedbackResult> {",
            "menu",
        ),
    ] {
        let body = function_body(SDK, marker);
        assert!(
            body.contains(&format!("unsupportedSystemFeedbackResult(\n    '{feature}'")),
            "{feature} must return the shared unsupported SDK result"
        );
        assert!(
            !body.contains("send("),
            "{feature} must not send a misleading fire-and-forget message"
        );
    }
}

// @lat: [[lat.md/protocol#Protocol#Prompt and control messages]]
#[test]
fn feedback_runtime_dispatch_paths_are_source_audited() {
    for needle in [
        "Message::Notify {",
        "execute_script_dispatch_notify_command(",
        "Message::SystemFeedbackResult { request_id, result }",
        "Message::Beep { request_id }",
        "execute_script_dispatch_beep_command()",
        "Message::Say {",
        "execute_script_dispatch_say_command(",
    ] {
        assert!(
            EXECUTE_SCRIPT.contains(needle),
            "execute_script dispatch must contain {needle}"
        );
    }
}

// @lat: [[lat.md/scripting#Scripting]]
#[test]
fn unsupported_sdk_inventory_matches_feedback_contract() {
    assert!(
        !MCP_RESOURCES.contains("\"beep\",\n    \"say\","),
        "beep and say have platform dispatch paths and must not stay in the unsupported inventory"
    );
    for needle in [
        "\"setStatus\"",
        "\"menu\"",
        "setStatus(...) currently has no visible GPUI status surface or receipt",
        "menu(...) currently has no GPUI tray/menu mutation handler",
    ] {
        assert!(
            MCP_RESOURCES.contains(needle),
            "unsupported SDK reference inventory/note must contain {needle}"
        );
    }
}
