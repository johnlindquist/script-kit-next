//! ACP permission broker.
//!
//! Bridges the ACP agent's permission requests to the GPUI UI thread.
//! Instead of a boolean allow/deny, the broker preserves the full set of
//! ACP permission options and returns the exact user-selected `option_id`.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use anyhow::Context;

/// A single permission option presented to the user.
#[derive(Debug, Clone)]
pub(crate) struct AcpApprovalOption {
    /// ACP option ID (e.g., "allow-once", "allow-always", "deny").
    pub option_id: String,
    /// Human-readable name for the option.
    pub name: String,
    /// The kind of option (e.g., "AllowOnce", "AllowAlways", "RejectOnce").
    pub kind: String,
}

impl AcpApprovalOption {
    /// Canonical display label: `"Name (Kind)"`.
    ///
    /// Used in both the approval modal option list and the post-approval
    /// system message so they always match.
    pub(crate) fn summary_label(&self) -> String {
        format!("{} ({})", self.name, self.kind)
    }
}

/// Build summary labels for a slice of approval options.
pub(crate) fn summarize_approval_options(options: &[AcpApprovalOption]) -> Vec<String> {
    options.iter().map(AcpApprovalOption::summary_label).collect()
}

/// Structured preview data for an approval request.
///
/// Carries enough information for the UI to render a rich tool-call
/// preview instead of parsing a plain-text body blob.
#[derive(Debug, Clone, Default)]
pub(crate) struct AcpApprovalPreview {
    /// Human-readable tool name (e.g., "Write file", "terminal/create").
    pub tool_title: String,
    /// ACP tool call ID for traceability.
    pub tool_call_id: String,
    /// Primary subject of the tool call (e.g., a file path or command).
    pub subject: Option<String>,
    /// Short summary of what the tool call does.
    pub summary: Option<String>,
    /// Truncated preview of the tool call input payload.
    pub input_preview: Option<String>,
    /// Truncated preview of the tool call output payload.
    pub output_preview: Option<String>,
    /// Human-readable labels for each option (e.g., "Allow (AllowOnce)").
    pub option_summary: Vec<String>,
}

impl AcpApprovalPreview {
    /// Start building a preview with the required fields.
    pub(crate) fn new(
        tool_title: impl Into<String>,
        tool_call_id: impl Into<String>,
    ) -> Self {
        Self {
            tool_title: tool_title.into(),
            tool_call_id: tool_call_id.into(),
            subject: None,
            summary: None,
            input_preview: None,
            output_preview: None,
            option_summary: Vec::new(),
        }
    }

    /// Set the primary subject (e.g., file path or command). Blank values are ignored.
    pub(crate) fn with_subject(mut self, subject: Option<String>) -> Self {
        self.subject = subject.filter(|v| !v.trim().is_empty());
        self
    }

    /// Set a short summary of the tool call action. Blank values are ignored.
    pub(crate) fn with_summary(mut self, summary: Option<String>) -> Self {
        self.summary = summary.filter(|v| !v.trim().is_empty());
        self
    }

    /// Set a truncated preview of the tool call input payload. Blank values are ignored.
    pub(crate) fn with_input_preview(mut self, input_preview: Option<String>) -> Self {
        self.input_preview = input_preview.filter(|v| !v.trim().is_empty());
        self
    }

    /// Set a truncated preview of the tool call output payload. Blank values are ignored.
    pub(crate) fn with_output_preview(mut self, output_preview: Option<String>) -> Self {
        self.output_preview = output_preview.filter(|v| !v.trim().is_empty());
        self
    }

    /// Set option summary labels from approval options via `summarize_approval_options`.
    pub(crate) fn with_options(mut self, options: &[AcpApprovalOption]) -> Self {
        self.option_summary = summarize_approval_options(options);
        self
    }
}

/// Input for constructing an approval request (before assigning an ID).
#[derive(Debug, Clone)]
pub(crate) struct AcpApprovalRequestInput {
    /// Title for the approval dialog.
    pub title: String,
    /// Body text describing the action requiring approval.
    pub body: String,
    /// Structured preview for rich UI rendering. `None` falls back to body text.
    pub preview: Option<AcpApprovalPreview>,
    /// Available permission options from the ACP agent.
    pub options: Vec<AcpApprovalOption>,
}

/// A fully-formed approval request ready for the UI.
#[derive(Debug, Clone)]
pub(crate) struct AcpApprovalRequest {
    /// Unique request identifier.
    pub id: u64,
    /// Title for the approval dialog.
    pub title: String,
    /// Body text describing the action requiring approval.
    pub body: String,
    /// Structured preview for rich UI rendering. `None` falls back to body text.
    pub preview: Option<AcpApprovalPreview>,
    /// Available permission options from the ACP agent.
    pub options: Vec<AcpApprovalOption>,
    /// Channel to send the user's selected option ID (or `None` for cancel).
    pub reply_tx: async_channel::Sender<Option<String>>,
}

/// Broker that manages permission request flow between the ACP worker
/// thread and the GPUI UI thread.
///
/// The broker lives on the ACP worker thread and sends requests to the
/// UI via a bounded channel. The UI sends back the selected option ID
/// (or `None` for cancellation) through a per-request reply channel.
#[derive(Clone)]
pub(crate) struct AcpPermissionBroker {
    tx: async_channel::Sender<AcpApprovalRequest>,
    next_id: Arc<AtomicU64>,
}

impl AcpPermissionBroker {
    /// Create a new broker and its corresponding receiver.
    ///
    /// The receiver should be consumed by the UI thread to present
    /// approval dialogs.
    pub(crate) fn new() -> (Self, async_channel::Receiver<AcpApprovalRequest>) {
        let (tx, rx) = async_channel::bounded(32);
        (
            Self {
                tx,
                next_id: Arc::new(AtomicU64::new(1)),
            },
            rx,
        )
    }

    /// Submit a permission request and block until the UI responds.
    ///
    /// Returns `Some(option_id)` if the user selected an option, or
    /// `None` if they cancelled.
    pub(crate) fn request(
        &self,
        input: AcpApprovalRequestInput,
    ) -> anyhow::Result<Option<String>> {
        let (reply_tx, reply_rx) = async_channel::bounded(1);
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        self.tx
            .send_blocking(AcpApprovalRequest {
                id,
                title: input.title,
                body: input.body,
                preview: input.preview,
                options: input.options,
                reply_tx,
            })
            .context("failed to send ACP approval request to UI")?;

        reply_rx
            .recv_blocking()
            .context("ACP approval reply channel closed")
    }

    /// Create an `ApprovalFn` that routes through this broker.
    ///
    /// This is the bridge between the old `ApprovalFn` signature and
    /// the new broker-based flow. The returned function captures the
    /// broker and forwards all permission options.
    pub(crate) fn approval_fn(&self) -> super::handlers::ApprovalFn {
        let broker = self.clone();
        Arc::new(move |input: AcpApprovalRequestInput| broker.request(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broker_assigns_sequential_ids() {
        let (broker, _rx) = AcpPermissionBroker::new();
        assert_eq!(broker.next_id.load(Ordering::SeqCst), 1);
        // Simulate two increments
        broker.next_id.fetch_add(1, Ordering::SeqCst);
        broker.next_id.fetch_add(1, Ordering::SeqCst);
        assert_eq!(broker.next_id.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn broker_request_completes_when_reply_sent() {
        let (broker, rx) = AcpPermissionBroker::new();

        // Spawn a thread to answer the request
        let handle = std::thread::spawn(move || {
            let request = rx.recv_blocking().expect("should receive request");
            assert_eq!(request.id, 1);
            assert_eq!(request.options.len(), 2);
            request
                .reply_tx
                .send_blocking(Some("allow-once".to_string()))
                .expect("reply should send");
        });

        let result = broker
            .request(AcpApprovalRequestInput {
                title: "Test".to_string(),
                body: "Test body".to_string(),
                preview: None,
                options: vec![
                    AcpApprovalOption {
                        option_id: "allow-once".to_string(),
                        name: "Allow once".to_string(),
                        kind: "AllowOnce".to_string(),
                    },
                    AcpApprovalOption {
                        option_id: "deny".to_string(),
                        name: "Deny".to_string(),
                        kind: "RejectOnce".to_string(),
                    },
                ],
            })
            .expect("request should succeed");

        assert_eq!(result, Some("allow-once".to_string()));
        handle.join().expect("responder thread should finish");
    }

    #[test]
    fn broker_request_returns_none_on_cancel() {
        let (broker, rx) = AcpPermissionBroker::new();

        let handle = std::thread::spawn(move || {
            let request = rx.recv_blocking().expect("should receive request");
            request
                .reply_tx
                .send_blocking(None)
                .expect("reply should send");
        });

        let result = broker
            .request(AcpApprovalRequestInput {
                title: "Test".to_string(),
                body: "Cancel test".to_string(),
                preview: None,
                options: vec![AcpApprovalOption {
                    option_id: "allow".to_string(),
                    name: "Allow".to_string(),
                    kind: "AllowOnce".to_string(),
                }],
            })
            .expect("request should succeed");

        assert_eq!(result, None);
        handle.join().expect("responder thread should finish");
    }

    #[test]
    fn summary_label_formats_name_and_kind() {
        let option = AcpApprovalOption {
            option_id: "allow".to_string(),
            name: "Allow".to_string(),
            kind: "AllowOnce".to_string(),
        };
        assert_eq!(option.summary_label(), "Allow (AllowOnce)");
    }

    #[test]
    fn summarize_approval_options_maps_all() {
        let options = vec![
            AcpApprovalOption {
                option_id: "allow".to_string(),
                name: "Allow".to_string(),
                kind: "AllowOnce".to_string(),
            },
            AcpApprovalOption {
                option_id: "deny".to_string(),
                name: "Deny".to_string(),
                kind: "RejectOnce".to_string(),
            },
        ];
        assert_eq!(
            summarize_approval_options(&options),
            vec!["Allow (AllowOnce)", "Deny (RejectOnce)"],
        );
    }

    #[test]
    fn preview_builder_sets_all_fields() {
        let options = vec![
            AcpApprovalOption {
                option_id: "allow".to_string(),
                name: "Allow".to_string(),
                kind: "AllowOnce".to_string(),
            },
            AcpApprovalOption {
                option_id: "deny".to_string(),
                name: "Deny".to_string(),
                kind: "RejectOnce".to_string(),
            },
        ];
        let preview = AcpApprovalPreview::new("terminal/create", "client-terminal-create")
            .with_subject(Some("bun run dev".to_string()))
            .with_summary(Some("Spawn a subprocess owned by the ACP client".to_string()))
            .with_input_preview(Some("{ \"command\": \"bun\" }".to_string()))
            .with_output_preview(Some("ok".to_string()))
            .with_options(&options);

        assert_eq!(preview.tool_title, "terminal/create");
        assert_eq!(preview.tool_call_id, "client-terminal-create");
        assert_eq!(preview.subject.as_deref(), Some("bun run dev"));
        assert_eq!(
            preview.summary.as_deref(),
            Some("Spawn a subprocess owned by the ACP client"),
        );
        assert_eq!(
            preview.input_preview.as_deref(),
            Some("{ \"command\": \"bun\" }"),
        );
        assert_eq!(preview.output_preview.as_deref(), Some("ok"));
        assert_eq!(
            preview.option_summary,
            vec!["Allow (AllowOnce)", "Deny (RejectOnce)"],
        );
    }

    #[test]
    fn preview_builder_filters_blank_strings() {
        let preview = AcpApprovalPreview::new("test", "id")
            .with_subject(Some("  ".to_string()))
            .with_summary(Some(String::new()))
            .with_input_preview(None);

        assert!(preview.subject.is_none());
        assert!(preview.summary.is_none());
        assert!(preview.input_preview.is_none());
    }
}
