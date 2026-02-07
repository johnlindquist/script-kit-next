//! Tests for notification types and notification service behavior.

use super::*;
use std::time::Duration;

#[test]
fn test_notification_id_generation() {
    let id1 = next_notification_id();
    let id2 = next_notification_id();
    let id3 = next_notification_id();

    assert!(id2 > id1, "IDs should be strictly increasing");
    assert!(id3 > id2, "IDs should be strictly increasing");
    assert_ne!(id1, id2, "IDs should be unique");
}

#[test]
fn test_notification_creation() {
    let notif = Notification::new();

    assert!(notif.id > 0);
    assert_eq!(notif.channels, vec![NotificationChannel::InAppToast]);
    assert!(notif.behavior.duration.is_some());
    assert!(notif.behavior.dismissable);
    assert!(notif.actions.is_empty());
}

#[test]
fn test_notification_success() {
    let notif = Notification::success("Task completed!");

    match &notif.content {
        NotificationContent::Rich { title, .. } => {
            assert_eq!(title, "Task completed!");
        }
        _ => panic!("Expected Rich content"),
    }
    assert_eq!(notif.behavior.duration, Some(Duration::from_secs(3)));
}

#[test]
fn test_notification_error() {
    let notif = Notification::error("Something went wrong");

    match &notif.content {
        NotificationContent::Rich { title, message, .. } => {
            assert_eq!(title, "Error");
            assert_eq!(message.as_deref(), Some("Something went wrong"));
        }
        _ => panic!("Expected Rich content"),
    }
    assert_eq!(notif.behavior.priority, NotificationPriority::High);
}

#[test]
fn test_notification_hud() {
    let notif = Notification::hud("Copied!");

    assert_eq!(notif.channels, vec![NotificationChannel::InAppHud]);
    match &notif.content {
        NotificationContent::Text(msg) => assert_eq!(msg, "Copied!"),
        _ => panic!("Expected Text content"),
    }
}

#[test]
fn test_notification_progress() {
    let notif = Notification::progress("Downloading...", 0.5);

    match &notif.content {
        NotificationContent::Progress {
            title, progress, ..
        } => {
            assert_eq!(title, "Downloading...");
            assert!((progress - 0.5).abs() < f32::EPSILON);
        }
        _ => panic!("Expected Progress content"),
    }
    assert!(
        notif.behavior.duration.is_none(),
        "Progress should be persistent"
    );
}

#[test]
fn test_notification_builder() {
    let notif = Notification::new()
        .content(NotificationContent::Text("Test".to_string()))
        .channel(NotificationChannel::InAppBanner)
        .duration(Duration::from_secs(10))
        .priority(NotificationPriority::High)
        .action("Retry", "retry")
        .primary_action("Open", "open")
        .from_script("/path/to/script.ts");

    assert_eq!(notif.channels, vec![NotificationChannel::InAppBanner]);
    assert_eq!(notif.behavior.duration, Some(Duration::from_secs(10)));
    assert_eq!(notif.behavior.priority, NotificationPriority::High);
    assert_eq!(notif.actions.len(), 2);
    assert_eq!(notif.actions[0].label, "Retry");
    assert_eq!(notif.actions[1].style, ActionStyle::Primary);
    match &notif.source {
        NotificationSource::Script { path } => assert_eq!(path, "/path/to/script.ts"),
        _ => panic!("Expected Script source"),
    }
}

#[test]
fn test_notification_add_channel() {
    let notif = Notification::new()
        .add_channel(NotificationChannel::System)
        .add_channel(NotificationChannel::System);

    assert_eq!(notif.channels.len(), 2);
    assert!(notif.channels.contains(&NotificationChannel::InAppToast));
    assert!(notif.channels.contains(&NotificationChannel::System));
}

#[test]
fn test_notification_persistent() {
    let notif = Notification::new().persistent();

    assert!(notif.behavior.duration.is_none());
    assert!(!notif.should_auto_dismiss());
}

#[test]
fn test_notification_expiry() {
    let notif = Notification::new().duration(Duration::from_millis(1));

    assert!(notif.remaining_ms().is_some());

    std::thread::sleep(Duration::from_millis(5));
    assert!(notif.is_expired());
}

#[test]
fn test_active_notification_pause_resume() {
    let notif = Notification::new().duration(Duration::from_millis(100));
    let mut active = ActiveNotification::new(notif);

    assert!(!active.timer_paused);

    active.pause_timer();
    assert!(active.timer_paused);
    assert!(active.paused_at.is_some());

    std::thread::sleep(Duration::from_millis(150));
    assert!(!active.is_expired(), "Should not expire while paused");

    active.resume_timer();
    assert!(!active.timer_paused);
    assert!(active.paused_duration >= Duration::from_millis(100));
}

#[test]
fn test_notification_source_key() {
    let system_notif = Notification::new();
    assert_eq!(system_notif.source_key(), "system");

    let script_notif = Notification::new().from_script("/path/to/script.ts");
    assert_eq!(script_notif.source_key(), "/path/to/script.ts");

    let builtin_notif = Notification::new().source(NotificationSource::BuiltIn {
        id: "clipboard".to_string(),
    });
    assert_eq!(builtin_notif.source_key(), "builtin:clipboard");
}

#[test]
fn test_notification_dedupe() {
    let notif = Notification::new().dedupe("same-notification");
    assert_eq!(notif.dedupe_key.as_deref(), Some("same-notification"));
}

#[test]
fn test_notification_group() {
    let notif = Notification::new().group("build-progress");
    assert_eq!(notif.group_key.as_deref(), Some("build-progress"));
}

#[test]
fn test_notification_action_styles() {
    let default_action = NotificationAction::new("Label", "id");
    assert_eq!(default_action.style, ActionStyle::Default);

    let primary_action = NotificationAction::primary("Label", "id");
    assert_eq!(primary_action.style, ActionStyle::Primary);

    let destructive_action = NotificationAction::destructive("Label", "id");
    assert_eq!(destructive_action.style, ActionStyle::Destructive);
}

#[test]
fn test_notification_priority_ordering() {
    assert!(NotificationPriority::Low < NotificationPriority::Normal);
    assert!(NotificationPriority::Normal < NotificationPriority::High);
    assert!(NotificationPriority::High < NotificationPriority::Urgent);
}

#[test]
fn test_notification_channel_equality() {
    assert_eq!(
        NotificationChannel::InAppToast,
        NotificationChannel::InAppToast
    );
    assert_ne!(
        NotificationChannel::InAppToast,
        NotificationChannel::InAppHud
    );
}

#[test]
fn test_notification_behavior_defaults() {
    let behavior = NotificationBehavior::default();

    assert_eq!(behavior.duration, Some(Duration::from_secs(3)));
    assert!(behavior.dismissable);
    assert!(behavior.replace_key.is_none());
    assert_eq!(behavior.sound, NotificationSound::None);
    assert_eq!(behavior.priority, NotificationPriority::Normal);
}

#[test]
fn test_notification_with_replace_key() {
    let notif = Notification::new().with_replace_key("build-status");
    assert_eq!(notif.behavior.replace_key.as_deref(), Some("build-status"));
}

#[test]
fn test_notification_history_entry() {
    let notif = Notification::success("Done");
    let entry = NotificationHistoryEntry {
        notification: notif,
        dismissed_at: std::time::Instant::now(),
        dismiss_reason: DismissReason::Timeout,
    };

    assert_eq!(entry.dismiss_reason, DismissReason::Timeout);
}

#[test]
fn test_active_notification_dedupe_count() {
    let notif = Notification::new();
    let mut active = ActiveNotification::new(notif);

    assert_eq!(active.dedupe_count, 0);

    active.increment_dedupe();
    assert_eq!(active.dedupe_count, 1);

    active.increment_dedupe();
    active.increment_dedupe();
    assert_eq!(active.dedupe_count, 3);
}

#[test]
fn test_dismiss_reason_variants() {
    let reasons = [
        DismissReason::Timeout,
        DismissReason::UserDismissed,
        DismissReason::Replaced,
        DismissReason::ActionTaken,
        DismissReason::Cleared,
        DismissReason::Programmatic,
    ];

    for (i, r1) in reasons.iter().enumerate() {
        for (j, r2) in reasons.iter().enumerate() {
            if i == j {
                assert_eq!(r1, r2);
            } else {
                assert_ne!(r1, r2);
            }
        }
    }
}

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

    for i in 0..5 {
        service.add_notification_for_test(
            Notification::new()
                .content(NotificationContent::Text(format!("Toast {}", i)))
                .channel(NotificationChannel::InAppToast),
        );
    }

    let visible = service.visible_toasts();
    assert_eq!(visible.len(), 3);
    assert_eq!(service.overflow_toast_count(), 2);
}

#[test]
fn test_visible_toasts_mixed_channels() {
    let mut service = make_service();

    service.add_notification_for_test(Notification::new().channel(NotificationChannel::InAppToast));
    service.add_notification_for_test(Notification::new().channel(NotificationChannel::InAppHud));
    service.add_notification_for_test(Notification::new().channel(NotificationChannel::InAppToast));

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

    for i in 0..150 {
        let notif = Notification::new().content(NotificationContent::Text(format!("Notif {}", i)));
        service.add_to_history_for_test(notif, DismissReason::Timeout);
    }

    assert_eq!(service.history().len(), 100);

    if let NotificationContent::Text(msg) = &service.history().front().unwrap().notification.content
    {
        assert_eq!(msg, "Notif 149");
    }
}

#[test]
fn test_dismiss_by_replace_key() {
    let mut service = make_service();

    for _ in 0..3 {
        service.add_notification_for_test(Notification::new().with_replace_key("build-status"));
    }
    service.add_notification_for_test(Notification::new().with_replace_key("other-key"));

    assert_eq!(service.active_count(), 4);

    service.dismiss_by_replace_key_for_test("build-status", DismissReason::Replaced);

    assert_eq!(service.active_count(), 1);
    assert_eq!(service.history().len(), 3);
    for entry in service.history() {
        assert_eq!(entry.dismiss_reason, DismissReason::Replaced);
    }
}

#[test]
fn test_rate_limiting() {
    let mut service = make_service();

    assert!(!service.is_rate_limited_for_test("test-source"));
    service.set_rate_limit_for_test("test-source");
    assert!(service.is_rate_limited_for_test("test-source"));
    assert!(!service.is_rate_limited_for_test("other-source"));
}

#[test]
fn test_active_notification_expiry_check() {
    let notif = Notification::new().duration(Duration::from_millis(1));
    let mut active = ActiveNotification::new(notif);

    std::thread::sleep(Duration::from_millis(5));
    assert!(active.is_expired());

    active.pause_timer();
    assert!(!active.is_expired());
}

#[test]
fn test_dismiss_all() {
    let mut service = make_service();

    for _ in 0..5 {
        service.add_notification_for_test(Notification::new());
    }

    assert_eq!(service.active_count(), 5);
    service.dismiss_all_for_test();
    assert_eq!(service.active_count(), 0);
    assert_eq!(service.history().len(), 5);
}

#[test]
fn test_update_progress() {
    let mut service = make_service();

    let notif = Notification::progress("Downloading...", 0.0);
    let id = notif.id;
    service.add_notification_for_test(notif);

    service.update_progress(id, 0.5, Some("Halfway there".to_string()));

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

    service.add_notification_for_test(Notification::new().dedupe("same-event"));
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
