//! Tests for NotificationService
//!
//! Note: These tests exercise the service's logic without GPUI context.

use super::*;
use std::time::Duration;

fn make_service() -> NotificationService {
    NotificationService::new()
}

#[test]
fn test_service_creation() {
    let service = make_service();
    assert!(!service.has_active());
    assert_eq!(service.active_count(), 0);
    assert!(service.history().is_empty());
    assert!(!service.is_dnd_enabled());
}

#[test]
fn test_dnd_toggle() {
    let mut service = make_service();

    assert!(!service.is_dnd_enabled());

    service.enable_dnd();
    assert!(service.is_dnd_enabled());

    service.disable_dnd();
    assert!(!service.is_dnd_enabled());

    service.toggle_dnd();
    assert!(service.is_dnd_enabled());

    service.toggle_dnd();
    assert!(!service.is_dnd_enabled());
}

#[test]
fn test_timer_pause_resume() {
    let mut service = make_service();

    service.add_notification_for_test(Notification::new().duration(Duration::from_secs(5)));

    let active = service.active_notifications();
    assert!(!active[0].timer_paused);

    service.pause_timers();
    let active = service.active_notifications();
    assert!(active[0].timer_paused);

    service.resume_timers();
    let active = service.active_notifications();
    assert!(!active[0].timer_paused);
}

#[test]
fn test_visible_toasts_limit() {
    let mut service = make_service();

    // Add 5 toast notifications
    for i in 0..5 {
        service.add_notification_for_test(
            Notification::new()
                .content(NotificationContent::Text(format!("Toast {}", i)))
                .channel(NotificationChannel::InAppToast),
        );
    }

    // Should only return MAX_VISIBLE_TOASTS (3)
    let visible = service.visible_toasts();
    assert_eq!(visible.len(), 3);

    // Overflow should be 2
    assert_eq!(service.overflow_toast_count(), 2);
}

#[test]
fn test_visible_toasts_mixed_channels() {
    let mut service = make_service();

    // Add toast notification
    service.add_notification_for_test(Notification::new().channel(NotificationChannel::InAppToast));

    // Add HUD notification
    service.add_notification_for_test(Notification::new().channel(NotificationChannel::InAppHud));

    // Add another toast
    service.add_notification_for_test(Notification::new().channel(NotificationChannel::InAppToast));

    // Should only count toasts
    let visible = service.visible_toasts();
    assert_eq!(visible.len(), 2);
    assert_eq!(service.overflow_toast_count(), 0);
}

#[test]
fn test_get_notification_by_id() {
    let mut service = make_service();

    let notif = Notification::new();
    let id = notif.id;
    service.add_notification_for_test(notif);

    assert!(service.get(id).is_some());
    assert!(service.get(999999).is_none());
}

#[test]
fn test_history_max_size() {
    let mut service = make_service();

    // Add more than MAX_HISTORY_SIZE (100) notifications to history
    for i in 0..150 {
        let notif = Notification::new().content(NotificationContent::Text(format!("Notif {}", i)));
        service.add_to_history_for_test(notif, DismissReason::Timeout);
    }

    // History should be capped at MAX_HISTORY_SIZE
    assert_eq!(service.history().len(), 100);

    // Most recent should be first
    if let NotificationContent::Text(msg) = &service.history().front().unwrap().notification.content
    {
        assert_eq!(msg, "Notif 149");
    }
}

#[test]
fn test_dismiss_by_replace_key() {
    let mut service = make_service();

    // Add notifications with same replace key
    for _ in 0..3 {
        service.add_notification_for_test(Notification::new().with_replace_key("build-status"));
    }

    // Add one with different key
    service.add_notification_for_test(Notification::new().with_replace_key("other-key"));

    assert_eq!(service.active_count(), 4);

    // Dismiss by replace key
    service.dismiss_by_replace_key_for_test("build-status", DismissReason::Replaced);

    // Should have 1 active, 3 in history
    assert_eq!(service.active_count(), 1);
    assert_eq!(service.history().len(), 3);

    // All history entries should have Replaced reason
    for entry in service.history() {
        assert_eq!(entry.dismiss_reason, DismissReason::Replaced);
    }
}

#[test]
fn test_rate_limiting() {
    let mut service = make_service();

    // First check should not be rate limited
    assert!(!service.is_rate_limited_for_test("test-source"));

    // After adding timestamp, should be rate limited
    service.set_rate_limit_for_test("test-source");
    assert!(service.is_rate_limited_for_test("test-source"));

    // Different source should not be rate limited
    assert!(!service.is_rate_limited_for_test("other-source"));
}

#[test]
fn test_active_notification_expiry_check() {
    // Add a notification that's already "expired"
    let notif = Notification::new().duration(Duration::from_millis(1));
    let mut active = ActiveNotification::new(notif);

    // Wait for it to expire
    std::thread::sleep(Duration::from_millis(5));

    // Manually check expiry (tick would normally do this)
    assert!(active.is_expired());

    // Paused notification should not expire
    active.pause_timer();
    assert!(!active.is_expired());
}

#[test]
fn test_dismiss_all() {
    let mut service = make_service();

    // Add several notifications
    for _ in 0..5 {
        service.add_notification_for_test(Notification::new());
    }

    assert_eq!(service.active_count(), 5);

    // Dismiss all using test helper
    service.dismiss_all_for_test();

    assert_eq!(service.active_count(), 0);
    assert_eq!(service.history().len(), 5);
}

#[test]
fn test_update_progress() {
    let mut service = make_service();

    // Add a progress notification
    let notif = Notification::progress("Downloading...", 0.0);
    let id = notif.id;
    service.add_notification_for_test(notif);

    // Update progress
    service.update_progress(id, 0.5, Some("Halfway there".to_string()));

    // Verify update
    let active = service.get(id).unwrap();
    if let NotificationContent::Progress {
        progress, message, ..
    } = &active.notification.content
    {
        assert!((*progress - 0.5).abs() < f32::EPSILON);
        assert_eq!(message.as_deref(), Some("Halfway there"));
    } else {
        panic!("Expected Progress content");
    }
}

#[test]
fn test_dedupe_increment() {
    let mut service = make_service();

    // Add notification with dedupe key
    service.add_notification_for_test(Notification::new().dedupe("same-event"));

    // The dedupe logic would happen in notify(), but we can test increment
    service.increment_dedupe_for_test(0);
    service.increment_dedupe_for_test(0);

    let active = &service.active_notifications()[0];
    assert_eq!(active.dedupe_count, 2);
}

#[test]
fn test_service_defaults() {
    let service = NotificationService::default();
    assert!(!service.has_active());
    assert!(!service.is_dnd_enabled());
    assert!(!service.are_timers_paused());
}
