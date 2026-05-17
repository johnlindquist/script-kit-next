//! Source-level contract test for the `pushDictationResult` stdin RPC.
//!
//! The RPC is an agentic-testing hook: it injects a synthetic transcript into
//! the real dictation delivery pipeline so ACP reveal/focus behavior can be
//! verified without microphone audio or a local transcription model.

const STDIN_COMMANDS: &str = include_str!("../src/stdin_commands/mod.rs");
const BUILTIN_EXECUTION: &str = include_str!("../src/app_execute/builtin_execution.rs");
const DICTATION_RUNTIME: &str = include_str!("../src/dictation/runtime.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const RUNTIME_STDIN_MATCH_TAIL: &str =
    include_str!("../src/main_entry/runtime_stdin_match_tail.rs");

fn push_arm<'a>(source: &'a str, name: &str) -> &'a str {
    let arm_start = source
        .find("ExternalCommand::PushDictationResult {")
        .unwrap_or_else(|| panic!("{name} must define PushDictationResult arm"));
    let arm = &source[arm_start..];
    let next_arm = arm["ExternalCommand::PushDictationResult {".len()..]
        .find("\n                            ExternalCommand::")
        .map(|idx| idx + "ExternalCommand::PushDictationResult {".len())
        .unwrap_or(arm.len());
    &arm[..next_arm]
}

#[test]
fn push_dictation_result_variant_is_defined_with_expected_fields() {
    assert!(
        STDIN_COMMANDS.contains(
            "PushDictationResult {\n        transcript: String,\n        #[serde(default)]\n        target: Option<String>,\n        #[serde(default, rename = \"requestId\")]\n        request_id: Option<ExternalCommandRequestId>,\n    },"
        ),
        "stdin protocol must keep the loose String target shape so aliases can be accepted without coupling serde to DictationTarget"
    );
}

#[test]
fn push_dictation_result_is_wired_into_request_id_and_command_type() {
    assert!(
        STDIN_COMMANDS.contains("| Self::PushDictationResult { request_id, .. }"),
        "request_id() must include PushDictationResult for correlation"
    );
    assert!(
        STDIN_COMMANDS.contains("Self::PushDictationResult { .. } => \"pushDictationResult\","),
        "command_type() must preserve the pushDictationResult verb"
    );
}

// doc-anchor-removed: [[acp-chat#ACP Chat#Detached window behavior#Dictation delivery to the composer#pushDictationResult stdin RPC]]
#[test]
fn push_dictation_result_routes_through_real_delivery_helper() {
    for (name, source) in [
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP),
        ("src/main_entry/runtime_stdin.rs", RUNTIME_STDIN),
        (
            "src/main_entry/runtime_stdin_match_tail.rs",
            RUNTIME_STDIN_MATCH_TAIL,
        ),
    ] {
        let arm = push_arm(source, name);
        assert!(
            arm.contains("view.deliver_stdin_dictation_result(")
                && arm.contains("event = \"push_dictation_result_delivered\"")
                && arm.contains("event = \"push_dictation_result_failed\""),
            "{name} must deliver pushDictationResult through the real dictation pipeline and log success/failure receipts"
        );
        assert!(
            !arm.contains("push_dictation_result_stub")
                && !arm.contains("stub — no delivery pipeline wired"),
            "{name} must not leave pushDictationResult as a stub"
        );
    }
}

#[test]
fn delivery_helper_preserves_active_session_target_and_accepts_acp_alias() {
    assert!(
        BUILTIN_EXECUTION.contains("pub(crate) fn deliver_stdin_dictation_result(")
            && BUILTIN_EXECUTION.contains(".or_else(crate::dictation::get_dictation_target)")
            && BUILTIN_EXECUTION.contains("self.handle_dictation_transcript("),
        "delivery helper must resolve target through explicit label, active session target, then UI-derived fallback before calling handle_dictation_transcript"
    );
    assert!(
        BUILTIN_EXECUTION.contains("\"tabaiharness\" | \"acp\" | \"acpchat\" | \"ai\"")
            && BUILTIN_EXECUTION.contains("DictationTarget::TabAiHarness"),
        "pushDictationResult must accept acp/acpChat aliases for ACP-targeted verification"
    );
    assert!(
        BUILTIN_EXECUTION.contains("crate::dictation::abort_dictation()"),
        "synthetic delivery must stop any active capture session before injecting the transcript"
    );
}

#[test]
fn push_dictation_result_does_not_log_transcript_contents() {
    for (name, source) in [
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP),
        ("src/main_entry/runtime_stdin.rs", RUNTIME_STDIN),
        (
            "src/main_entry/runtime_stdin_match_tail.rs",
            RUNTIME_STDIN_MATCH_TAIL,
        ),
    ] {
        let arm = push_arm(source, name);
        assert!(
            arm.contains("transcript_len = transcript.len()"),
            "{name} must log only transcript length"
        );
        assert!(
            !arm.contains("transcript = %transcript")
                && !arm.contains("transcript = ?transcript")
                && !arm.contains("?transcript,"),
            "{name} must not log transcript content; cloning for helper calls should stay outside tracing fields"
        );
    }
}

#[test]
fn dictation_delivery_records_redacted_receipt_for_devtools() {
    assert!(
        DICTATION_RUNTIME.contains("static LAST_DELIVERY_RECEIPT")
            && DICTATION_RUNTIME.contains("pub fn record_delivery_receipt(")
            && DICTATION_RUNTIME.contains("pub fn last_delivery_receipt()")
            && DICTATION_RUNTIME.contains("pub fn redacted_transcript_fingerprint("),
        "dictation runtime must expose a redacted last-delivery receipt for agent-facing DevTools"
    );
    assert!(
        DICTATION_RUNTIME.contains("\"transcriptLen\"")
            && DICTATION_RUNTIME.contains("\"transcriptFingerprint\"")
            && DICTATION_RUNTIME.contains("\"insertionRange\"")
            && DICTATION_RUNTIME.contains("\"redacted\": true")
            && !DICTATION_RUNTIME.contains("\"transcript\": transcript")
            && !DICTATION_RUNTIME.contains("\"transcriptText\""),
        "delivery receipt must expose only length/fingerprint metadata, never raw transcript text"
    );
    assert!(
        BUILTIN_EXECUTION.contains("crate::dictation::record_delivery_receipt(")
            && BUILTIN_EXECUTION.contains("DictationDestination::FrontmostApp")
            && BUILTIN_EXECUTION.contains("\"operation\": \"replaceInput\"")
            && BUILTIN_EXECUTION.contains("\"unit\": \"utf8Bytes\""),
        "both internal and frontmost-app delivery paths must write the receipt at the delivery boundary"
    );
}
