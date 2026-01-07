//! NotificationService - GPUI Global for centralized notification management
//!
//! Uses GPUI's Global trait pattern for thread-safe, UI-thread-owned state.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use gpui::{App, BorrowAppContext, Global, Timer};

use super::types::*;

/// Maximum number of notifications in history
const MAX_HISTORY_SIZE: usize = 100;

/// Maximum visible toasts before collapsing
const MAX_VISIBLE_TOASTS: usize = 3;

/// Rate limit window for per-source notifications (ms)
const RATE_LIMIT_WINDOW_MS: u64 = 250;

/// Notification service - registered as a GPUI Global
pub struct NotificationService {
    /// Active notifications (not yet dismissed)
    active: Vec<ActiveNotification>,
    /// Notification history (dismissed notifications)
    history: VecDeque<NotificationHistoryEntry>,
    /// Last notification time per source (for rate limiting)
    last_notification_time: std::collections::HashMap<String, Instant>,
    /// Whether timers should be paused (window hidden)
    timers_paused: bool,
    /// Do Not Disturb mode
    dnd_enabled: bool,
}

impl Global for NotificationService {}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationService {
    /// Create a new notification service
    pub fn new() -> Self {
        Self {
            active: Vec::new(),
            history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            last_notification_time: std::collections::HashMap::new(),
            timers_paused: false,
            dnd_enabled: false,
        }
    }

    /// Initialize the notification service as a GPUI global
    pub fn init(cx: &mut App) {
        cx.set_global(Self::new());
    }

    // =========================================================================
    // Core notification methods
    // =========================================================================

    /// Show a notification
    ///
    /// Routes to appropriate renderers based on channels, handles deduplication,
    /// rate limiting, and replacement.
    pub fn notify(&mut self, notification: Notification, cx: &mut App) {
        let source_key = notification.source_key();

        // Rate limiting check
        if self.is_rate_limited(&source_key) {
            tracing::debug!(
                source = %source_key,
                "Notification rate limited, skipping"
            );
            return;
        }

        // DND check (still store in history but don't show)
        if self.dnd_enabled && notification.behavior.priority < NotificationPriority::Urgent {
            self.add_to_history_silent(notification, DismissReason::Cleared);
            return;
        }

        // Deduplication check
        if let Some(dedupe_key) = &notification.dedupe_key {
            if let Some(existing) = self
                .active
                .iter_mut()
                .find(|n| n.notification.dedupe_key.as_ref() == Some(dedupe_key))
            {
                existing.increment_dedupe();
                tracing::debug!(
                    dedupe_key = %dedupe_key,
                    count = existing.dedupe_count,
                    "Notification deduplicated"
                );
                return;
            }
        }

        // Replacement check
        if let Some(replace_key) = &notification.behavior.replace_key {
            self.dismiss_by_replace_key(replace_key, DismissReason::Replaced);
        }

        // Update rate limit timestamp
        self.last_notification_time
            .insert(source_key.clone(), Instant::now());

        let id = notification.id;
        let channels = notification.channels.clone();
        let duration = notification.behavior.duration;

        // Create active notification
        let mut active = ActiveNotification::new(notification);
        if self.timers_paused {
            active.pause_timer();
        }

        // Store in active list
        self.active.push(active);

        // Route to renderers
        for channel in &channels {
            self.route_to_channel(*channel, id, cx);
        }

        // Schedule auto-dismiss if needed
        if let Some(dur) = duration {
            self.schedule_auto_dismiss(id, dur, cx);
        }

        tracing::debug!(
            notification_id = id,
            channels = ?channels,
            "Notification shown"
        );
    }

    /// Route notification to a specific channel renderer
    fn route_to_channel(&self, channel: NotificationChannel, _id: NotificationId, _cx: &mut App) {
        // For now, just log the routing decision
        // Actual rendering integration will be added in the next task
        match channel {
            NotificationChannel::InAppToast => {
                tracing::debug!("Routing to InAppToast renderer");
                // Will integrate with gpui-component Notification
            }
            NotificationChannel::InAppHud => {
                tracing::debug!("Routing to InAppHud renderer");
                // Will integrate with hud_manager
            }
            NotificationChannel::InAppBanner => {
                tracing::debug!("Routing to InAppBanner renderer");
                // Future: banner implementation
            }
            NotificationChannel::InAppInline => {
                tracing::debug!("Routing to InAppInline renderer");
                // Future: inline implementation
            }
            NotificationChannel::System => {
                tracing::debug!("Routing to System notification");
                // Future: macOS notification integration
            }
            NotificationChannel::Dialog => {
                tracing::debug!("Routing to Dialog renderer");
                // Future: blocking dialog implementation
            }
        }
    }

    /// Schedule auto-dismiss for a notification
    fn schedule_auto_dismiss(&self, id: NotificationId, duration: Duration, cx: &mut App) {
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            Timer::after(duration).await;

            let _ = cx.update(|cx| {
                if cx.has_global::<NotificationService>() {
                    cx.update_global::<NotificationService, _>(
                        |service: &mut NotificationService, cx| {
                            service.dismiss(id, DismissReason::Timeout, cx);
                        },
                    );
                }
            });
        })
        .detach();
    }

    // =========================================================================
    // Convenience notification methods
    // =========================================================================

    /// Show a success notification
    pub fn success(&mut self, message: impl Into<String>, cx: &mut App) {
        self.notify(Notification::success(message), cx);
    }

    /// Show an error notification
    pub fn error(&mut self, message: impl Into<String>, cx: &mut App) {
        self.notify(Notification::error(message), cx);
    }

    /// Show a warning notification
    pub fn warning(&mut self, message: impl Into<String>, cx: &mut App) {
        self.notify(Notification::warning(message), cx);
    }

    /// Show an info notification
    pub fn info(&mut self, message: impl Into<String>, cx: &mut App) {
        self.notify(Notification::info(message), cx);
    }

    /// Show a HUD notification
    pub fn hud(&mut self, message: impl Into<String>, cx: &mut App) {
        self.notify(Notification::hud(message), cx);
    }

    /// Show a progress notification and return its ID for updates
    pub fn progress(
        &mut self,
        title: impl Into<String>,
        progress: f32,
        cx: &mut App,
    ) -> NotificationId {
        let notification = Notification::progress(title, progress);
        let id = notification.id;
        self.notify(notification, cx);
        id
    }

    /// Update a progress notification
    pub fn update_progress(&mut self, id: NotificationId, progress: f32, message: Option<String>) {
        if let Some(active) = self.active.iter_mut().find(|n| n.notification.id == id) {
            if let NotificationContent::Progress {
                progress: p,
                message: m,
                ..
            } = &mut active.notification.content
            {
                *p = progress;
                if let Some(msg) = message {
                    *m = Some(msg);
                }
            }
        }
    }

    // =========================================================================
    // Dismissal methods
    // =========================================================================

    /// Dismiss a notification by ID
    pub fn dismiss(&mut self, id: NotificationId, reason: DismissReason, _cx: &mut App) {
        if let Some(pos) = self.active.iter().position(|n| n.notification.id == id) {
            let active = self.active.remove(pos);
            self.add_to_history(active.notification, reason);

            tracing::debug!(
                notification_id = id,
                reason = ?reason,
                "Notification dismissed"
            );
        }
    }

    /// Dismiss notifications by replace key
    fn dismiss_by_replace_key(&mut self, replace_key: &str, reason: DismissReason) {
        let to_dismiss: Vec<usize> = self
            .active
            .iter()
            .enumerate()
            .filter(|(_, n)| n.notification.behavior.replace_key.as_deref() == Some(replace_key))
            .map(|(i, _)| i)
            .collect();

        // Remove in reverse order to preserve indices
        for idx in to_dismiss.into_iter().rev() {
            let active = self.active.remove(idx);
            self.add_to_history(active.notification, reason);
        }
    }

    /// Dismiss all notifications
    pub fn dismiss_all(&mut self, _cx: &mut App) {
        let notifications: Vec<_> = self.active.drain(..).collect();
        for active in notifications {
            self.add_to_history(active.notification, DismissReason::Cleared);
        }
        tracing::debug!("All notifications dismissed");
    }

    // =========================================================================
    // Timer control (for window hide/show)
    // =========================================================================

    /// Pause all auto-dismiss timers (called when window is hidden)
    pub fn pause_timers(&mut self) {
        if !self.timers_paused {
            self.timers_paused = true;
            for active in &mut self.active {
                active.pause_timer();
            }
            tracing::debug!("Notification timers paused");
        }
    }

    /// Resume all auto-dismiss timers (called when window is shown)
    pub fn resume_timers(&mut self) {
        if self.timers_paused {
            self.timers_paused = false;
            for active in &mut self.active {
                active.resume_timer();
            }
            tracing::debug!("Notification timers resumed");
        }
    }

    // =========================================================================
    // Do Not Disturb
    // =========================================================================

    /// Enable Do Not Disturb mode
    pub fn enable_dnd(&mut self) {
        self.dnd_enabled = true;
        tracing::info!("Do Not Disturb enabled");
    }

    /// Disable Do Not Disturb mode
    pub fn disable_dnd(&mut self) {
        self.dnd_enabled = false;
        tracing::info!("Do Not Disturb disabled");
    }

    /// Toggle Do Not Disturb mode
    pub fn toggle_dnd(&mut self) {
        self.dnd_enabled = !self.dnd_enabled;
        tracing::info!(enabled = self.dnd_enabled, "Do Not Disturb toggled");
    }

    /// Check if DND is enabled
    pub fn is_dnd_enabled(&self) -> bool {
        self.dnd_enabled
    }

    // =========================================================================
    // Query methods
    // =========================================================================

    /// Get active notifications (for rendering)
    pub fn active_notifications(&self) -> &[ActiveNotification] {
        &self.active
    }

    /// Get visible toast notifications (limited to MAX_VISIBLE_TOASTS)
    pub fn visible_toasts(&self) -> Vec<&ActiveNotification> {
        self.active
            .iter()
            .filter(|n| {
                n.notification
                    .channels
                    .contains(&NotificationChannel::InAppToast)
            })
            .take(MAX_VISIBLE_TOASTS)
            .collect()
    }

    /// Get count of overflow toasts (beyond max visible)
    pub fn overflow_toast_count(&self) -> usize {
        let toast_count = self
            .active
            .iter()
            .filter(|n| {
                n.notification
                    .channels
                    .contains(&NotificationChannel::InAppToast)
            })
            .count();
        toast_count.saturating_sub(MAX_VISIBLE_TOASTS)
    }

    /// Get notification history
    pub fn history(&self) -> &VecDeque<NotificationHistoryEntry> {
        &self.history
    }

    /// Get a notification by ID
    pub fn get(&self, id: NotificationId) -> Option<&ActiveNotification> {
        self.active.iter().find(|n| n.notification.id == id)
    }

    /// Check if any notifications are active
    pub fn has_active(&self) -> bool {
        !self.active.is_empty()
    }

    /// Get count of active notifications
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    /// Check if a source is rate limited
    fn is_rate_limited(&self, source_key: &str) -> bool {
        if let Some(last_time) = self.last_notification_time.get(source_key) {
            last_time.elapsed() < Duration::from_millis(RATE_LIMIT_WINDOW_MS)
        } else {
            false
        }
    }

    /// Add notification to history
    fn add_to_history(&mut self, notification: Notification, reason: DismissReason) {
        let entry = NotificationHistoryEntry {
            notification,
            dismissed_at: Instant::now(),
            dismiss_reason: reason,
        };

        self.history.push_front(entry);

        // Trim history if needed
        while self.history.len() > MAX_HISTORY_SIZE {
            self.history.pop_back();
        }
    }

    /// Add notification to history without showing (for DND)
    fn add_to_history_silent(&mut self, notification: Notification, reason: DismissReason) {
        self.add_to_history(notification, reason);
    }

    /// Tick expired notifications (called periodically)
    pub fn tick(&mut self, cx: &mut App) {
        let expired: Vec<NotificationId> = self
            .active
            .iter()
            .filter(|n| n.is_expired())
            .map(|n| n.notification.id)
            .collect();

        for id in expired {
            self.dismiss(id, DismissReason::Timeout, cx);
        }
    }

    /// Check if timers are paused
    pub fn are_timers_paused(&self) -> bool {
        self.timers_paused
    }

    // =========================================================================
    // Test helpers (only compiled for tests)
    // =========================================================================

    #[cfg(test)]
    pub(crate) fn add_notification_for_test(&mut self, notification: Notification) {
        self.active.push(ActiveNotification::new(notification));
    }

    #[cfg(test)]
    pub(crate) fn add_to_history_for_test(
        &mut self,
        notification: Notification,
        reason: DismissReason,
    ) {
        self.add_to_history(notification, reason);
    }

    #[cfg(test)]
    pub(crate) fn dismiss_by_replace_key_for_test(
        &mut self,
        replace_key: &str,
        reason: DismissReason,
    ) {
        self.dismiss_by_replace_key(replace_key, reason);
    }

    #[cfg(test)]
    pub(crate) fn is_rate_limited_for_test(&self, source_key: &str) -> bool {
        self.is_rate_limited(source_key)
    }

    #[cfg(test)]
    pub(crate) fn set_rate_limit_for_test(&mut self, source_key: &str) {
        self.last_notification_time
            .insert(source_key.to_string(), Instant::now());
    }

    #[cfg(test)]
    pub(crate) fn dismiss_all_for_test(&mut self) {
        let notifications: Vec<_> = self.active.drain(..).collect();
        for active in notifications {
            self.add_to_history(active.notification, DismissReason::Cleared);
        }
    }

    #[cfg(test)]
    pub(crate) fn increment_dedupe_for_test(&mut self, index: usize) {
        if let Some(active) = self.active.get_mut(index) {
            active.increment_dedupe();
        }
    }
}
