use script_kit_gpui::permissions_wizard::{
    build_permission_guide_receipt, permission_guide_action, PermissionGuideActionKind,
    PermissionGuideInput, PermissionKind, PermissionRequirement,
    PERMISSION_GUIDE_RECEIPT_SCHEMA_VERSION,
};

use script_kit_gpui::platform::permiso_detect::PermissionStatus;

#[test]
fn permission_guide_action_chooses_prompt_before_settings_fallback() {
    assert_eq!(
        permission_guide_action(
            PermissionKind::Accessibility,
            PermissionStatus::Authorized,
            false
        ),
        PermissionGuideActionKind::AlreadyGranted
    );
    assert_eq!(
        permission_guide_action(
            PermissionKind::Accessibility,
            PermissionStatus::Denied,
            false
        ),
        PermissionGuideActionKind::SystemPrompt
    );
    assert_eq!(
        permission_guide_action(
            PermissionKind::Accessibility,
            PermissionStatus::Denied,
            true
        ),
        PermissionGuideActionKind::SystemSettings
    );
    assert_eq!(
        permission_guide_action(
            PermissionKind::ScreenRecording,
            PermissionStatus::NotDetermined,
            false
        ),
        PermissionGuideActionKind::SystemPrompt
    );
    assert_eq!(
        permission_guide_action(
            PermissionKind::ScreenRecording,
            PermissionStatus::Denied,
            true
        ),
        PermissionGuideActionKind::SystemSettings
    );
}

#[test]
fn permission_guide_keeps_microphone_read_only_in_first_slice() {
    assert_eq!(
        permission_guide_action(PermissionKind::Microphone, PermissionStatus::Denied, false),
        PermissionGuideActionKind::ReadOnlyStatus
    );
}

#[test]
fn permission_guide_receipt_identifies_primary_required_missing_permission() {
    let receipt = build_permission_guide_receipt(&[
        PermissionGuideInput {
            kind: PermissionKind::Accessibility,
            status: PermissionStatus::Denied,
            prompt_attempted: false,
        },
        PermissionGuideInput {
            kind: PermissionKind::ScreenRecording,
            status: PermissionStatus::Denied,
            prompt_attempted: true,
        },
        PermissionGuideInput {
            kind: PermissionKind::Microphone,
            status: PermissionStatus::Denied,
            prompt_attempted: false,
        },
    ]);

    assert_eq!(
        receipt.schema_version,
        PERMISSION_GUIDE_RECEIPT_SCHEMA_VERSION
    );
    assert_eq!(receipt.primary_missing, Some(PermissionKind::Accessibility));
    assert!(receipt.redacted);
    assert_eq!(
        receipt.steps[0].action,
        PermissionGuideActionKind::SystemPrompt
    );
    assert_eq!(
        receipt.steps[1].action,
        PermissionGuideActionKind::SystemSettings
    );
    assert_eq!(
        receipt.steps[2].requirement,
        PermissionRequirement::Optional
    );
}

#[test]
fn permission_guide_receipt_serializes_camel_case() {
    let receipt = build_permission_guide_receipt(&[PermissionGuideInput {
        kind: PermissionKind::ScreenRecording,
        status: PermissionStatus::Denied,
        prompt_attempted: true,
    }]);

    let json = serde_json::to_value(receipt).expect("serialize receipt");

    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["primaryMissing"], "ScreenRecording");
    assert_eq!(json["redacted"], true);
    assert_eq!(json["steps"][0]["action"], "systemSettings");
    assert_eq!(
        json["steps"][0]["settingsUrl"],
        PermissionKind::ScreenRecording.settings_url()
    );
}
