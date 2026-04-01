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

/// Input for constructing an approval request (before assigning an ID).
#[derive(Debug, Clone)]
pub(crate) struct AcpApprovalRequestInput {
    /// Title for the approval dialog.
    pub title: String,
    /// Body text describing the action requiring approval.
    pub body: String,
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
}
