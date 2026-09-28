//! Source-level contract for targeted automation batch capabilities.
//!
//! Target-specific batch command support should live behind named capability
//! helpers before any future extraction of the async execution loops.

const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");

fn source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end: {end_pat}"));
    &source[start..start + end_rel]
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

#[test]
fn prompt_handler_declares_batch_target_capability_owner() {
    for required in [
        "enum AutomationBatchTargetKind",
        "struct BatchTargetCapabilities",
        "fn batch_target_kind_for_resolved_target(",
        "fn supported_batch_commands_for_target(",
        "fn unsupported_batch_command_error(",
        "fn is_agent_chat_wait_condition(",
    ] {
        assert!(
            PROMPT_HANDLER.contains(required),
            "prompt_handler must declare {required}"
        );
    }
}

#[test]
fn supported_command_lists_are_named_by_target_kind() {
    let body = source_between(
        PROMPT_HANDLER,
        "impl BatchTargetCapabilities",
        "\nfn batch_target_kind_for_resolved_target(",
    );

    let arm = source_between(
        body,
        "AutomationBatchTargetKind::Main",
        "AutomationBatchTargetKind::SnapOverlay =>",
    );
    for command in [
        "\"setInput\"",
        "\"forceSubmit\"",
        "\"waitFor\"",
        "\"openActions\"",
        "\"selectByValue\"",
        "\"selectBySemanticId\"",
        "\"filterAndSelect\"",
        "\"typeAndSubmit\"",
    ] {
        assert!(
            arm.contains(command),
            "AutomationBatchTargetKind::Main capability list must include {command}"
        );
    }
}

#[test]
fn main_batch_open_actions_uses_shared_actions_dispatcher() {
    let main_batch_body = source_between(
        PROMPT_HANDLER,
        "// ── Main-window batch path (existing)",
        "let success = !failed;",
    );

    for required in [
        "protocol::BatchCommand::OpenActions =>",
        "crate::get_main_window_handle()",
        "this.dispatch_actions_toggle_for_current_view(",
        "\"devtools_batch_open_actions\"",
        "command: \"openActions\".to_string()",
        "Current main view does not expose actions",
    ] {
        assert!(
            main_batch_body.contains(required),
            "Main batch openActions must route through the shared actions dispatcher: {required}"
        );
    }

    let open_actions_arm = source_between(
        main_batch_body,
        "protocol::BatchCommand::OpenActions =>",
        "protocol::BatchCommand::TogglePreview =>",
    );

    assert!(
        !open_actions_arm.contains("simulateKey"),
        "Main batch openActions must not synthesize Cmd+K"
    );
}

#[test]
fn unsupported_command_errors_are_generated_by_the_helper() {
    let helper_body = source_between(
        PROMPT_HANDLER,
        "fn unsupported_batch_command_error(",
        "\nfn is_agent_chat_wait_condition(",
    );

    assert!(helper_body.contains("protocol::TransactionErrorCode::UnsupportedCommand"));
    assert!(helper_body.contains("supported_batch_commands_for_target(kind).join(\", \")"));
    assert_eq!(
        count_occurrences(PROMPT_HANDLER, "TransactionErrorCode::UnsupportedCommand"),
        1,
        "target batch branches must not copy unsupported-command construction"
    );

    let main_batch_body = source_between(
        PROMPT_HANDLER,
        "// ── Main-window batch path (existing)",
        "let success = !failed;",
    );
    assert!(
        main_batch_body.contains("unsupported_batch_command_error("),
        "main batch branch must report unsupported commands through the shared helper"
    );
}

#[test]
fn batch_routing_uses_the_resolved_target_kind() {
    let body = source_between(
        PROMPT_HANDLER,
        "PromptMessage::Batch {",
        "\n            PromptMessage::ForceSubmit",
    );

    assert!(body
        .contains("let batch_target_kind = batch_target_kind_for_resolved_target(&batch_target);"));
}
