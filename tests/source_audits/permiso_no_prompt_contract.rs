use std::fs;

// @lat: [[tests/permission-assistant#Permission Assistant#Passive detection does not prompt]]
#[test]
fn permiso_no_prompt_contract_detection_and_overlay_are_passive() {
    let detect = fs::read_to_string("src/platform/permiso_detect.rs").expect("read permiso detect");
    let overlay =
        fs::read_to_string("src/platform/permiso/overlay_window.rs").expect("read overlay");
    let drag = fs::read_to_string("src/platform/permiso/drag_source.rs").expect("read drag");
    let host = fs::read_to_string("src/platform/permiso/host_app.rs").expect("read host app");
    let locator = fs::read_to_string("src/platform/permiso/locator.rs").expect("read locator");

    for required in [
        "pub enum PermissionStatus",
        "AXIsProcessTrusted()",
        "CGPreflightScreenCaptureAccess()",
        "authorizationStatusForMediaType",
        "PermissionStatus::NotDetermined",
    ] {
        assert!(
            detect.contains(required),
            "passive detect missing {required}"
        );
    }

    for forbidden in [
        "AXIsProcessTrustedWithOptions",
        "CGRequestScreenCaptureAccess",
        "requestAccessForMediaType",
        "TCC.db",
        "tccutil",
    ] {
        assert!(
            !detect.contains(forbidden)
                && !overlay.contains(forbidden)
                && !drag.contains(forbidden)
                && !host.contains(forbidden)
                && !locator.contains(forbidden),
            "Permission Assistant must not prompt or mutate permissions; found {forbidden}"
        );
    }

    for required in [
        "PassiveOverlayPanel",
        "canBecomeKey = false",
        "canBecomeMain = false",
        "orderFrontRegardless",
        "never calls setActivationPolicy",
        "NSDraggingSource + NSPasteboardItemDataProvider",
        "host_app_bundle_url()",
        "NSDragOperationCopy",
        "CGWindowListCopyWindowInfo",
        "com.apple.systempreferences",
        "layer == 0",
    ] {
        assert!(
            overlay.contains(required) || drag.contains(required) || locator.contains(required),
            "native Permission Assistant contract missing {required}"
        );
    }
}
