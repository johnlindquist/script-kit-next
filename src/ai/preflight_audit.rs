use super::message_parts::{
    ContextResolutionFailure, PreparedMessageDecision, PreparedMessageReceipt,
};
use super::model::ChatId;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub const AI_PREFLIGHT_AUDIT_SCHEMA_VERSION: u32 = 2;
pub const AI_PREFLIGHT_AUDIT_MAX_BYTES: u64 = 5 * 1024 * 1024;
const AI_PREFLIGHT_AUDIT_COMPACT_KEEP: usize = 2_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionableContextFailure {
    pub label: String,
    pub source: String,
    pub code: String,
    pub message: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AiPreflightAudit {
    pub schema_version: u32,
    pub correlation_id: String,
    #[serde(default)]
    pub preflight_generation: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft_fingerprint: Option<String>,
    pub chat_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    pub decision: PreparedMessageDecision,
    pub raw_content: String,
    pub authored_content: String,
    pub has_pending_image: bool,
    pub has_context_parts: bool,
    pub receipt: PreparedMessageReceipt,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actionable_failures: Vec<ActionableContextFailure>,
    pub created_at: String,
}

impl AiPreflightAudit {
    pub fn new(
        chat_id: &ChatId,
        raw_content: &str,
        authored_content: &str,
        has_pending_image: bool,
        has_context_parts: bool,
        receipt: PreparedMessageReceipt,
    ) -> Self {
        let created_at = Utc::now();
        let correlation_id = format!(
            "preflight-{}-{}",
            chat_id.as_str(),
            created_at.timestamp_micros()
        );

        let actionable_failures = receipt
            .context
            .failures
            .iter()
            .map(actionable_context_failure)
            .collect();

        Self {
            schema_version: AI_PREFLIGHT_AUDIT_SCHEMA_VERSION,
            correlation_id,
            preflight_generation: 0,
            draft_fingerprint: Some(stable_draft_fingerprint(raw_content, authored_content)),
            chat_id: chat_id.as_str(),
            message_id: None,
            decision: receipt.decision.clone(),
            raw_content: raw_content.to_string(),
            authored_content: authored_content.to_string(),
            has_pending_image,
            has_context_parts,
            receipt,
            actionable_failures,
            created_at: created_at.to_rfc3339(),
        }
    }

    pub fn attach_message_id(&mut self, message_id: &str) {
        self.message_id = Some(message_id.to_string());
    }
}

fn stable_draft_fingerprint(raw_content: &str, authored_content: &str) -> String {
    format!(
        "raw:{}:authored:{}",
        raw_content.len(),
        authored_content.len()
    )
}

pub fn default_preflight_audit_log_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".scriptkit")
        .join("logs")
        .join("ai-preflight-audits.jsonl")
}

pub fn append_preflight_audit(
    path: Option<&Path>,
    audit: &AiPreflightAudit,
) -> anyhow::Result<PathBuf> {
    if audit.schema_version != AI_PREFLIGHT_AUDIT_SCHEMA_VERSION {
        anyhow::bail!(
            "unsupported preflight audit schema version {}",
            audit.schema_version
        );
    }

    let path = path
        .map(PathBuf::from)
        .unwrap_or_else(default_preflight_audit_log_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    compact_preflight_audits_if_needed(&path)?;

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(&path)?;
    ensure_jsonl_append_boundary(&mut file)?;
    writeln!(file, "{}", serde_json::to_string(audit)?)?;
    Ok(path)
}

fn ensure_jsonl_append_boundary(file: &mut fs::File) -> anyhow::Result<()> {
    let len = file.metadata()?.len();
    if len == 0 {
        return Ok(());
    }
    file.seek(SeekFrom::End(-1))?;
    let mut last = [0u8; 1];
    file.read_exact(&mut last)?;
    file.seek(SeekFrom::End(0))?;
    if last[0] != b'\n' {
        writeln!(file)?;
    }
    Ok(())
}

pub fn read_preflight_audits(path: Option<&Path>) -> anyhow::Result<Vec<AiPreflightAudit>> {
    let path = path
        .map(PathBuf::from)
        .unwrap_or_else(default_preflight_audit_log_path);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = OpenOptions::new().read(true).open(&path)?;
    let reader = BufReader::new(file);
    let mut seen = BTreeSet::new();
    let mut audits = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let audit: AiPreflightAudit = match serde_json::from_str(&line) {
            Ok(audit) => audit,
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::ai_preflight",
                    %error,
                    "Skipping malformed preflight audit log entry"
                );
                continue;
            }
        };
        if audit.schema_version != AI_PREFLIGHT_AUDIT_SCHEMA_VERSION {
            tracing::warn!(
                target: "script_kit::ai_preflight",
                schema_version = audit.schema_version,
                "Skipping unsupported preflight audit schema version"
            );
            continue;
        }
        if seen.insert(audit.correlation_id.clone()) {
            audits.push(audit);
        }
    }

    Ok(audits)
}

pub fn compact_preflight_audits_if_needed(path: &Path) -> anyhow::Result<()> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if metadata.len() <= AI_PREFLIGHT_AUDIT_MAX_BYTES {
        return Ok(());
    }

    let audits = read_preflight_audits(Some(path))?;
    let start = audits.len().saturating_sub(AI_PREFLIGHT_AUDIT_COMPACT_KEEP);
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    for audit in audits.into_iter().skip(start) {
        writeln!(file, "{}", serde_json::to_string(&audit)?)?;
    }
    Ok(())
}

pub fn actionable_context_failure(failure: &ContextResolutionFailure) -> ActionableContextFailure {
    let source = failure.source.as_str();

    if source.contains("browserUrl=1") {
        return ActionableContextFailure {
            label: failure.label.clone(),
            source: failure.source.clone(),
            code: "browser_url_unavailable".to_string(),
            message: "Couldn't capture the focused browser tab URL.".to_string(),
            remediation:
                "Focus a supported browser tab and retry, or use /window if URL is not required."
                    .to_string(),
        };
    }

    if source.contains("selectedText=1") {
        return ActionableContextFailure {
            label: failure.label.clone(),
            source: failure.source.clone(),
            code: "selected_text_unavailable".to_string(),
            message: "Couldn't read a non-empty text selection.".to_string(),
            remediation:
                "Select text in the target app and retry, or switch to /context or /window."
                    .to_string(),
        };
    }

    if source.contains("focusedWindow=1") {
        return ActionableContextFailure {
            label: failure.label.clone(),
            source: failure.source.clone(),
            code: "focused_window_unavailable".to_string(),
            message: "Couldn't capture focused window metadata.".to_string(),
            remediation:
                "Bring the target window to the front and retry, then inspect kit://context?diagnostics=1 if it still fails."
                    .to_string(),
        };
    }

    if source.contains("kit://context") {
        return ActionableContextFailure {
            label: failure.label.clone(),
            source: failure.source.clone(),
            code: "context_resource_unavailable".to_string(),
            message: "A desktop context resource could not be resolved.".to_string(),
            remediation:
                "Retry after refocusing the target app, or inspect kit://context?diagnostics=1 for field-level status."
                    .to_string(),
        };
    }

    ActionableContextFailure {
        label: failure.label.clone(),
        source: failure.source.clone(),
        code: "attachment_unavailable".to_string(),
        message: "An attachment could not be resolved.".to_string(),
        remediation:
            "Verify the source still exists and retry, or remove the attachment and send the message without it."
                .to_string(),
    }
}

pub fn build_actionable_preflight_error(audit: &AiPreflightAudit) -> Option<String> {
    if audit.actionable_failures.is_empty() {
        return None;
    }

    Some(
        audit
            .actionable_failures
            .iter()
            .map(|failure| {
                format!(
                    "{}: {} {}",
                    failure.label, failure.message, failure.remediation
                )
            })
            .collect::<Vec<_>>()
            .join(" "),
    )
}

pub fn log_preflight_audit(audit: &AiPreflightAudit, stage: &str) {
    tracing::info!(
        target: "script_kit::ai_preflight",
        event = "ai_preflight_audit",
        stage = stage,
        correlation_id = %audit.correlation_id,
        chat_id = %audit.chat_id,
        message_id = ?audit.message_id,
        decision = ?audit.decision,
        attempted = audit.receipt.context.attempted,
        resolved = audit.receipt.context.resolved,
        failure_count = audit.receipt.context.failures.len(),
        has_pending_image = audit.has_pending_image,
        has_context_parts = audit.has_context_parts,
        final_user_content_len = audit.receipt.final_user_content.len(),
        "ai_preflight_audit"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::message_parts::{
        ContextResolutionFailure, ContextResolutionReceipt, PreparedMessageDecision,
        PreparedMessageReceipt,
    };

    #[test]
    fn test_build_actionable_preflight_error_for_browser_failure() {
        let receipt = PreparedMessageReceipt {
            schema_version: 1,
            decision: PreparedMessageDecision::Blocked,
            raw_content: "Summarize this page".to_string(),
            final_user_content: "Summarize this page".to_string(),
            context: ContextResolutionReceipt {
                attempted: 1,
                resolved: 0,
                failures: vec![ContextResolutionFailure {
                    label: "Browser URL".to_string(),
                    source: "kit://context?browserUrl=1".to_string(),
                    error: "No browser detected".to_string(),
                }],
                prompt_prefix: String::new(),
            },
            assembly: None,
            outcomes: Vec::new(),
            unresolved_parts: Vec::new(),
            user_error: None,
        };

        let audit = AiPreflightAudit {
            schema_version: AI_PREFLIGHT_AUDIT_SCHEMA_VERSION,
            correlation_id: "corr-1".to_string(),
            preflight_generation: 1,
            draft_fingerprint: Some("raw:19:authored:19".to_string()),
            chat_id: "chat-1".to_string(),
            message_id: None,
            decision: PreparedMessageDecision::Blocked,
            raw_content: "Summarize this page".to_string(),
            authored_content: "Summarize this page".to_string(),
            has_pending_image: false,
            has_context_parts: true,
            actionable_failures: vec![actionable_context_failure(&receipt.context.failures[0])],
            receipt,
            created_at: "2026-03-21T18:32:13Z".to_string(),
        };

        let error = build_actionable_preflight_error(&audit).expect("expected actionable error");
        assert!(
            error.contains("Focus a supported browser tab and retry"),
            "Expected remediation guidance in error, got: {error}"
        );
        assert!(
            error.contains("Couldn't capture the focused browser tab URL"),
            "Expected user-facing message in error, got: {error}"
        );
    }

    #[test]
    fn test_actionable_failure_codes_for_known_sources() {
        let cases = vec![
            ("kit://context?browserUrl=1", "browser_url_unavailable"),
            (
                "kit://context?selectedText=1&frontmostApp=0",
                "selected_text_unavailable",
            ),
            (
                "kit://context?focusedWindow=1&browserUrl=0",
                "focused_window_unavailable",
            ),
            (
                "kit://context?profile=minimal",
                "context_resource_unavailable",
            ),
            ("/tmp/missing.txt", "attachment_unavailable"),
        ];

        for (source, expected_code) in cases {
            let failure = ContextResolutionFailure {
                label: "Test".to_string(),
                source: source.to_string(),
                error: "test error".to_string(),
            };
            let actionable = actionable_context_failure(&failure);
            assert_eq!(
                actionable.code, expected_code,
                "source={source} should map to code={expected_code}"
            );
        }
    }

    #[test]
    fn test_no_actionable_error_when_no_failures() {
        let receipt = PreparedMessageReceipt {
            schema_version: 1,
            decision: PreparedMessageDecision::Ready,
            raw_content: "Hello".to_string(),
            final_user_content: "Hello".to_string(),
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
        };

        let audit = AiPreflightAudit {
            schema_version: AI_PREFLIGHT_AUDIT_SCHEMA_VERSION,
            correlation_id: "corr-2".to_string(),
            preflight_generation: 1,
            draft_fingerprint: Some("raw:5:authored:5".to_string()),
            chat_id: "chat-2".to_string(),
            message_id: None,
            decision: PreparedMessageDecision::Ready,
            raw_content: "Hello".to_string(),
            authored_content: "Hello".to_string(),
            has_pending_image: false,
            has_context_parts: false,
            actionable_failures: Vec::new(),
            receipt,
            created_at: "2026-03-21T18:32:13Z".to_string(),
        };

        assert!(build_actionable_preflight_error(&audit).is_none());
    }

    #[test]
    fn test_serde_roundtrip_camel_case() {
        let failure = ActionableContextFailure {
            label: "Browser URL".to_string(),
            source: "kit://context?browserUrl=1".to_string(),
            code: "browser_url_unavailable".to_string(),
            message: "Couldn't capture the focused browser tab URL.".to_string(),
            remediation: "Focus a supported browser tab and retry.".to_string(),
        };

        let json = serde_json::to_string(&failure).expect("serialize");
        assert!(json.contains("\"label\""), "fields should be camelCase");
        assert!(
            !json.contains("\"label_\""),
            "should not have snake_case fields"
        );

        let deserialized: ActionableContextFailure =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, failure);
    }
}
