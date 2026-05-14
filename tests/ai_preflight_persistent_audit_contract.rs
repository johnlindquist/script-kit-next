use script_kit_gpui::ai::message_parts::{
    ContextResolutionReceipt, PreparedMessageDecision, PreparedMessageReceipt,
};
use script_kit_gpui::ai::preflight_audit::{
    append_preflight_audit, read_preflight_audits, AiPreflightAudit, AI_PREFLIGHT_AUDIT_MAX_BYTES,
    AI_PREFLIGHT_AUDIT_SCHEMA_VERSION,
};

#[test]
fn audit_reader_skips_malformed_and_unsupported_schema_lines() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("audits.jsonl");
    std::fs::write(
        &path,
        format!(
            "{{bad json}}\n{}\n{}\n",
            serde_json::json!({"schemaVersion": 999, "correlationId": "old"}),
            serde_json::to_string(&audit("ok")).unwrap()
        ),
    )
    .expect("write audit log");

    let audits = read_preflight_audits(Some(&path)).expect("read audits");
    assert_eq!(audits.len(), 1);
    assert_eq!(audits[0].correlation_id, "ok");
}

#[test]
fn audit_reader_dedupes_by_correlation_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("audits.jsonl");
    let first = audit("same");
    let mut second = audit("same");
    second.preflight_generation = 2;
    std::fs::write(
        &path,
        format!(
            "{}\n{}\n",
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        ),
    )
    .expect("write audit log");

    let audits = read_preflight_audits(Some(&path)).expect("read audits");
    assert_eq!(audits.len(), 1);
    assert_eq!(audits[0].preflight_generation, 1);
}

#[test]
fn append_compacts_when_log_exceeds_growth_bound() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("audits.jsonl");
    std::fs::write(
        &path,
        "x".repeat((AI_PREFLIGHT_AUDIT_MAX_BYTES as usize) + 1),
    )
    .expect("seed oversized log");

    append_preflight_audit(Some(&path), &audit("after-compact")).expect("append audit");
    let metadata = std::fs::metadata(&path).expect("metadata");
    assert!(
        metadata.len() < AI_PREFLIGHT_AUDIT_MAX_BYTES,
        "compaction should bring the log back below the growth bound"
    );
    let audits = read_preflight_audits(Some(&path)).expect("read audits");
    assert_eq!(audits.len(), 1);
    assert_eq!(audits[0].correlation_id, "after-compact");
}

fn audit(correlation_id: &str) -> AiPreflightAudit {
    AiPreflightAudit {
        schema_version: AI_PREFLIGHT_AUDIT_SCHEMA_VERSION,
        correlation_id: correlation_id.to_string(),
        preflight_generation: 1,
        draft_fingerprint: Some("draft:1".to_string()),
        chat_id: "chat-1".to_string(),
        message_id: None,
        decision: PreparedMessageDecision::Ready,
        raw_content: "hello".to_string(),
        authored_content: "hello".to_string(),
        has_pending_image: false,
        has_context_parts: false,
        receipt: PreparedMessageReceipt {
            schema_version: 1,
            decision: PreparedMessageDecision::Ready,
            raw_content: "hello".to_string(),
            final_user_content: "hello".to_string(),
            context: ContextResolutionReceipt {
                attempted: 0,
                resolved: 0,
                failures: Vec::new(),
                prompt_prefix: String::new(),
            },
            assembly: None,
            outcomes: Vec::new(),
            unresolved_parts: Vec::new(),
            user_error: None,
        },
        actionable_failures: Vec::new(),
        created_at: "2026-05-14T00:00:00Z".to_string(),
    }
}
