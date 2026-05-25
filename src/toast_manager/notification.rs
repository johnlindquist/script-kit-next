use gpui::{prelude::*, rgba};
use gpui_component::notification::{Notification, NotificationType};

use crate::components::ToastVariant;
use crate::theme::{get_cached_theme, AppChromeColors};

use super::PendingToast;

/// Convert Script Kit's stripped-down queued toast into the active
/// gpui-component notification runtime.
fn toast_variant_to_notification_type(variant: ToastVariant) -> NotificationType {
    match variant {
        ToastVariant::Success => NotificationType::Success,
        ToastVariant::Warning => NotificationType::Warning,
        ToastVariant::Error => NotificationType::Error,
        ToastVariant::Info => NotificationType::Info,
    }
}

/// Convert a PendingToast to a gpui-component Notification.
///
/// The active notification runtime supports a binary autohide contract:
/// non-persistent notifications use gpui-component's default duration,
/// while persistent notifications disable autohide.
pub fn pending_toast_to_notification(toast: &PendingToast) -> Notification {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);
    let notification_type = toast_variant_to_notification_type(toast.variant);

    let mut notification = Notification::new()
        .message(&toast.message)
        .with_type(notification_type)
        .bg(rgba(chrome.popup_surface_rgba))
        .border_color(rgba(chrome.border_rgba));

    if theme.is_vibrancy_enabled() {
        notification = notification.shadow_none();
    }

    match toast.variant {
        ToastVariant::Error => {
            notification = notification.title("Couldn't complete action");
        }
        ToastVariant::Warning => {
            notification = notification.title("Warning");
        }
        _ => {}
    }

    if toast.persistent {
        notification = notification.autohide(false);
    }

    notification
}
