const INFO_PLIST_EXT: &str = include_str!("../assets/Info.plist.ext");
const ENTITLEMENTS: &str = include_str!("../entitlements.plist");

#[test]
fn release_bundle_declares_usage_reasons_for_signed_protected_resources() {
    for (entitlement, usage_key, copy_marker) in [
        (
            "com.apple.security.device.audio-input",
            "NSMicrophoneUsageDescription",
            "local dictation audio",
        ),
        (
            "com.apple.security.device.camera",
            "NSCameraUsageDescription",
            "webcam frames",
        ),
        (
            "com.apple.security.automation.apple-events",
            "NSAppleEventsUsageDescription",
            "automate other apps",
        ),
    ] {
        assert!(
            ENTITLEMENTS.contains(entitlement),
            "expected signed entitlement {entitlement} to remain declared"
        );
        assert!(
            INFO_PLIST_EXT.contains(usage_key) && INFO_PLIST_EXT.contains(copy_marker),
            "Info.plist extension must explain {usage_key} because {entitlement} is signed"
        );
    }
}
