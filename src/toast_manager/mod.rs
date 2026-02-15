//! Toast Manager for coordinating toast notifications.
//!
//! This module provides a `ToastManager` staging queue. Toasts are pushed via
//! `push()` from anywhere in the code (even without window access). In the
//! render loop, call `drain_pending()` to convert queued toasts into
//! gpui-component notifications.

use crate::components::{Toast, ToastVariant};
use uuid::Uuid;

/// A wrapper around Toast with a generated identifier.
pub struct ToastNotification {
    /// Unique identifier for this toast.
    pub id: String,
    /// The underlying toast component.
    pub toast: Toast,
}

impl std::fmt::Debug for ToastNotification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToastNotification")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl ToastNotification {
    /// Create a new toast notification wrapping a Toast.
    pub fn new(toast: Toast) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            toast,
        }
    }
}

/// Manager for handling the toast queue.
pub struct ToastManager {
    notifications: Vec<ToastNotification>,
}

impl std::fmt::Debug for ToastManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToastManager")
            .field("notification_count", &self.notifications.len())
            .finish()
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastManager {
    /// Create a new ToastManager with default settings.
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
        }
    }

    /// Push a new toast onto the queue.
    ///
    /// Returns the unique ID assigned to this toast.
    pub fn push(&mut self, toast: Toast) -> String {
        let notification = ToastNotification::new(toast);
        let id = notification.id.clone();

        tracing::debug!(
            toast_id = %id,
            queue_len_before = self.notifications.len(),
            "Toast pushed to queue"
        );

        self.notifications.push(notification);
        id
    }

    /// Drain all pending toasts from the queue.
    ///
    /// This is used to flush queued toasts to gpui-component's notification
    /// system. After draining, the internal queue is cleared.
    pub fn drain_pending(&mut self) -> Vec<PendingToast> {
        let pending: Vec<PendingToast> = self
            .notifications
            .drain(..)
            .map(|n| PendingToast {
                message: n.toast.get_message().to_string(),
                variant: n.toast.get_variant(),
                duration_ms: n.toast.get_duration_ms(),
            })
            .collect();

        if !pending.is_empty() {
            tracing::debug!(count = pending.len(), "Drained pending toasts");
        }

        pending
    }
}

/// A pending toast ready to be converted to a gpui-component Notification.
#[derive(Debug, Clone)]
pub struct PendingToast {
    pub message: String,
    pub variant: ToastVariant,
    pub duration_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ToastColors;

    fn make_test_toast(message: &'static str, duration_ms: Option<u64>) -> Toast {
        let colors = ToastColors::default();
        Toast::new(message, colors).duration_ms(duration_ms)
    }

    #[test]
    fn test_push_returns_generated_toast_id() {
        let mut manager = ToastManager::new();

        let id = manager.push(make_test_toast("Test message", Some(5000)));

        assert!(!id.is_empty());
    }

    #[test]
    fn test_drain_pending_returns_enqueued_toasts_in_fifo_order() {
        let mut manager = ToastManager::new();

        manager.push(make_test_toast("First", Some(4000)));
        manager.push(make_test_toast("Second", None));

        let pending = manager.drain_pending();
        assert_eq!(pending.len(), 2);

        assert_eq!(pending[0].message, "First");
        assert_eq!(pending[0].variant, ToastVariant::Info);
        assert_eq!(pending[0].duration_ms, Some(4000));

        assert_eq!(pending[1].message, "Second");
        assert_eq!(pending[1].variant, ToastVariant::Info);
        assert_eq!(pending[1].duration_ms, None);
    }

    #[test]
    fn test_drain_pending_clears_queue_after_first_drain() {
        let mut manager = ToastManager::new();
        manager.push(make_test_toast("Only toast", Some(2000)));

        let first_drain = manager.drain_pending();
        let second_drain = manager.drain_pending();

        assert_eq!(first_drain.len(), 1);
        assert!(second_drain.is_empty());
    }
}
