#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum PermissionStatus {
    Authorized,
    Denied,
    NotDetermined,
    Unknown,
}

#[cfg(target_os = "macos")]
pub fn ax_is_trusted() -> PermissionStatus {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }

    if unsafe { AXIsProcessTrusted() } {
        PermissionStatus::Authorized
    } else {
        PermissionStatus::Denied
    }
}

#[cfg(not(target_os = "macos"))]
pub fn ax_is_trusted() -> PermissionStatus {
    PermissionStatus::Unknown
}

#[cfg(target_os = "macos")]
pub fn screen_capture_authorized() -> PermissionStatus {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
    }

    if unsafe { CGPreflightScreenCaptureAccess() } {
        PermissionStatus::Authorized
    } else {
        PermissionStatus::Denied
    }
}

#[cfg(not(target_os = "macos"))]
pub fn screen_capture_authorized() -> PermissionStatus {
    PermissionStatus::Unknown
}

#[cfg(target_os = "macos")]
pub fn microphone_authorized() -> PermissionStatus {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use objc::{class, msg_send, sel, sel_impl};

    const AV_AUTHORIZED: i64 = 3;
    const AV_DENIED: i64 = 2;
    const AV_RESTRICTED: i64 = 1;
    const AV_NOT_DETERMINED: i64 = 0;

    #[link(name = "AVFoundation", kind = "framework")]
    extern "C" {}

    unsafe {
        let media_type = NSString::alloc(nil).init_str("soun");
        if media_type == nil {
            return PermissionStatus::Unknown;
        }

        let status: i64 = msg_send![
            class!(AVCaptureDevice),
            authorizationStatusForMediaType: media_type as id
        ];

        match status {
            AV_AUTHORIZED => PermissionStatus::Authorized,
            AV_DENIED | AV_RESTRICTED => PermissionStatus::Denied,
            AV_NOT_DETERMINED => PermissionStatus::NotDetermined,
            _ => PermissionStatus::Unknown,
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn microphone_authorized() -> PermissionStatus {
    PermissionStatus::Unknown
}

#[cfg(target_os = "macos")]
pub fn input_monitoring_authorized() -> PermissionStatus {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGPreflightListenEventAccess() -> bool;
    }

    if unsafe { CGPreflightListenEventAccess() } {
        PermissionStatus::Authorized
    } else {
        PermissionStatus::Denied
    }
}

#[cfg(not(target_os = "macos"))]
pub fn input_monitoring_authorized() -> PermissionStatus {
    PermissionStatus::Unknown
}

#[cfg(target_os = "macos")]
pub fn event_synthesizing_authorized() -> PermissionStatus {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGPreflightPostEventAccess() -> bool;
    }

    if unsafe { CGPreflightPostEventAccess() } {
        PermissionStatus::Authorized
    } else {
        PermissionStatus::Denied
    }
}

#[cfg(not(target_os = "macos"))]
pub fn event_synthesizing_authorized() -> PermissionStatus {
    PermissionStatus::Unknown
}
