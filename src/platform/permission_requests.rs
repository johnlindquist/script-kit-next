//! Explicit, user-initiated macOS permission prompts for the Permissions
//! wizard's Grant buttons. Kept apart from `permiso_detect`, which is strictly
//! passive: detection and the Permission Assistant overlay never prompt or
//! mutate permissions.

use super::permiso_detect::PermissionStatus;

/// Trigger the one-time native macOS prompt for Screen Recording.
/// Returns the post-request status; once denied, macOS will not re-prompt
/// and the user must grant via System Settings.
#[cfg(target_os = "macos")]
pub fn request_screen_capture_access() -> PermissionStatus {
    if !super::native_effect_allowed(crate::runtime_policy::ExternalEffect::NativeVisibility) {
        return PermissionStatus::Unknown;
    }
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGRequestScreenCaptureAccess() -> bool;
    }

    if unsafe { CGRequestScreenCaptureAccess() } {
        PermissionStatus::Authorized
    } else {
        PermissionStatus::Denied
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_screen_capture_access() -> PermissionStatus {
    PermissionStatus::Unknown
}

/// Trigger the one-time native macOS prompt for Input Monitoring.
#[cfg(target_os = "macos")]
pub fn request_input_monitoring_access() -> PermissionStatus {
    if !super::native_effect_allowed(crate::runtime_policy::ExternalEffect::NativeVisibility) {
        return PermissionStatus::Unknown;
    }
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGRequestListenEventAccess() -> bool;
    }

    if unsafe { CGRequestListenEventAccess() } {
        PermissionStatus::Authorized
    } else {
        PermissionStatus::Denied
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_input_monitoring_access() -> PermissionStatus {
    PermissionStatus::Unknown
}

/// Trigger the one-time native macOS prompt for posting synthetic events
/// (the Accessibility-adjacent "control this computer" grant).
#[cfg(target_os = "macos")]
pub fn request_event_synthesizing_access() -> PermissionStatus {
    if !super::native_effect_allowed(crate::runtime_policy::ExternalEffect::NativeVisibility) {
        return PermissionStatus::Unknown;
    }
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGRequestPostEventAccess() -> bool;
    }

    if unsafe { CGRequestPostEventAccess() } {
        PermissionStatus::Authorized
    } else {
        PermissionStatus::Denied
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_event_synthesizing_access() -> PermissionStatus {
    PermissionStatus::Unknown
}
