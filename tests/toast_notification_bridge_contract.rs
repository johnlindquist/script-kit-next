//! Source-level contracts for the active toast-to-notification bridge.

const TOAST_MANAGER_SOURCE: &str = include_str!("../src/toast_manager/mod.rs");
const TOAST_BRIDGE_SOURCE: &str = include_str!("../src/toast_manager/notification.rs");
const LIFECYCLE_SOURCE: &str = include_str!("../src/app_impl/lifecycle_reset.rs");
const DEEPLINK_SOURCE: &str = include_str!("../src/main_sections/deeplink.rs");
const TOAST_MODEL_SOURCE: &str = include_str!("../src/components/toast/model.rs");

#[test]
fn pending_toast_runtime_contract_is_message_variant_and_persistence() {
    for needle in [
        "pub struct PendingToast",
        "pub message: String",
        "pub variant: ToastVariant",
        "pub persistent: bool",
    ] {
        assert!(
            TOAST_MANAGER_SOURCE.contains(needle),
            "PendingToast should expose only fields that survive notification conversion: {needle}"
        );
    }
    assert!(
        !TOAST_MANAGER_SOURCE.contains("pub duration_ms: Option<u64>"),
        "PendingToast should not advertise custom duration support"
    );
    assert!(
        TOAST_MANAGER_SOURCE.contains("notification.toast.get_duration_ms().is_none()"),
        "ToastManager should map None duration to persistent notification state"
    );
}

#[test]
fn notification_bridge_owns_conversion_and_script_kit_chrome() {
    for needle in [
        "toast_variant_to_notification_type",
        "pending_toast_to_notification",
        "get_cached_theme()",
        "AppChromeColors::from_theme(&theme)",
        ".bg(rgba(chrome.popup_surface_rgba))",
        ".border_color(rgba(chrome.border_rgba))",
        "theme.is_vibrancy_enabled()",
        "notification.shadow_none()",
        "toast.persistent",
        "notification.autohide(false)",
    ] {
        assert!(
            TOAST_BRIDGE_SOURCE.contains(needle),
            "Notification bridge should preserve behavior while applying Script Kit chrome: {needle}"
        );
    }
}

#[test]
fn deeplink_no_longer_owns_toast_notification_conversion() {
    assert!(
        LIFECYCLE_SOURCE
            .contains("crate::toast_manager::notification::pending_toast_to_notification"),
        "flush_pending_toasts should call the toast_manager bridge"
    );
    for needle in [
        "fn pending_toast_to_notification",
        "fn toast_variant_to_notification_type",
        "NotificationType",
    ] {
        assert!(
            !DEEPLINK_SOURCE.contains(needle),
            "deeplink code should not own toast notification conversion: {needle}"
        );
    }
}

#[test]
fn toast_model_documents_binary_runtime_autohide_contract() {
    for needle in ["`Some(_)` as", "vendor default duration", "`None` disables"] {
        assert!(
            TOAST_MODEL_SOURCE.contains(needle),
            "Toast duration docs should state the runtime bridge contract: {needle}"
        );
    }
}
