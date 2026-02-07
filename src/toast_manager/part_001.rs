#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ToastColors;

    fn make_test_toast(duration_ms: Option<u64>) -> Toast {
        let colors = ToastColors::default();
        Toast::new("Test message", colors).duration_ms(duration_ms)
    }

    #[test]
    fn test_toast_notification_creation() {
        let toast = make_test_toast(Some(5000));
        let notification = ToastNotification::new(toast);

        assert!(!notification.id.is_empty());
        assert!(!notification.is_dismissed);
        assert_eq!(notification.duration_ms, Some(5000));
    }

    #[test]
    fn test_toast_notification_with_id() {
        let toast = make_test_toast(Some(5000));
        let notification = ToastNotification::with_id("custom-id", toast);

        assert_eq!(notification.id, "custom-id");
    }

    #[test]
    fn test_toast_manager_push() {
        let mut manager = ToastManager::new();
        let toast = make_test_toast(Some(5000));

        let id = manager.push(toast);

        assert!(!id.is_empty());
        assert_eq!(manager.total_count(), 1);
        assert_eq!(manager.visible_count(), 1);
        assert!(manager.take_needs_notify());
    }

    #[test]
    fn test_toast_manager_dismiss() {
        let mut manager = ToastManager::new();
        let toast = make_test_toast(Some(5000));
        let id = manager.push(toast);

        // Reset notify flag
        manager.take_needs_notify();

        // Dismiss the toast
        let dismissed = manager.dismiss(&id);

        assert!(dismissed);
        assert_eq!(manager.visible_count(), 0);
        assert_eq!(manager.total_count(), 1); // Still in queue until cleanup
        assert!(manager.take_needs_notify());
    }

    #[test]
    fn test_toast_manager_dismiss_nonexistent() {
        let mut manager = ToastManager::new();
        let dismissed = manager.dismiss("nonexistent-id");

        assert!(!dismissed);
    }

    #[test]
    fn test_toast_manager_dismiss_all() {
        let mut manager = ToastManager::new();
        manager.push(make_test_toast(Some(5000)));
        manager.push(make_test_toast(Some(5000)));
        manager.push(make_test_toast(Some(5000)));

        manager.take_needs_notify();
        manager.dismiss_all();

        assert_eq!(manager.visible_count(), 0);
        assert_eq!(manager.total_count(), 3);
        assert!(manager.take_needs_notify());
    }

    #[test]
    fn test_toast_manager_visible_toasts_limit() {
        let mut manager = ToastManager::with_max_visible(2);

        manager.push(make_test_toast(Some(5000)));
        manager.push(make_test_toast(Some(5000)));
        manager.push(make_test_toast(Some(5000)));

        let visible = manager.visible_toasts();

        assert_eq!(visible.len(), 2);
        assert_eq!(manager.total_count(), 3);
    }

    #[test]
    fn test_toast_manager_cleanup() {
        let mut manager = ToastManager::new();
        let id1 = manager.push(make_test_toast(Some(5000)));
        manager.push(make_test_toast(Some(5000)));

        manager.dismiss(&id1);
        let cleaned = manager.cleanup();

        assert_eq!(cleaned, 1);
        assert_eq!(manager.total_count(), 1);
    }

    #[test]
    fn test_toast_manager_clear() {
        let mut manager = ToastManager::new();
        manager.push(make_test_toast(Some(5000)));
        manager.push(make_test_toast(Some(5000)));

        manager.take_needs_notify();
        manager.clear();

        assert_eq!(manager.total_count(), 0);
        assert!(manager.take_needs_notify());
    }

    #[test]
    fn test_toast_notification_should_auto_dismiss() {
        // Create a toast with very short duration
        let colors = ToastColors::default();
        let toast = Toast::new("Test", colors).duration_ms(Some(1));
        let notification = ToastNotification::new(toast);

        // Wait a bit for the duration to pass
        std::thread::sleep(std::time::Duration::from_millis(5));

        assert!(notification.should_auto_dismiss());
    }

    #[test]
    fn test_toast_notification_persistent() {
        let colors = ToastColors::default();
        let toast = Toast::new("Test", colors).duration_ms(None);
        let notification = ToastNotification::new(toast);

        // Persistent toasts should never auto-dismiss
        assert!(!notification.should_auto_dismiss());
    }

    #[test]
    fn test_toast_manager_tick() {
        let mut manager = ToastManager::new();

        // Add a toast with very short duration
        let colors = ToastColors::default();
        let toast = Toast::new("Test", colors).duration_ms(Some(1));
        manager.push(toast);

        // Wait for duration to pass
        std::thread::sleep(std::time::Duration::from_millis(5));

        // Tick should auto-dismiss
        let dismissed = manager.tick();

        assert!(dismissed);
        assert_eq!(manager.visible_count(), 0);
    }

    #[test]
    fn test_toast_notification_remaining_ms() {
        let colors = ToastColors::default();
        let toast = Toast::new("Test", colors).duration_ms(Some(5000));
        let notification = ToastNotification::new(toast);

        let remaining = notification.remaining_ms();

        assert!(remaining.is_some());
        assert!(remaining.unwrap() <= 5000);
        assert!(remaining.unwrap() > 4900); // Should be close to 5000
    }

    #[test]
    fn test_toast_manager_has_visible() {
        let mut manager = ToastManager::new();

        assert!(!manager.has_visible());

        let id = manager.push(make_test_toast(Some(5000)));
        assert!(manager.has_visible());

        manager.dismiss(&id);
        assert!(!manager.has_visible());
    }
}
