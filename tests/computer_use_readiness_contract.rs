use script_kit_gpui::computer_use::{
    build_computer_use_readiness_receipt, ComputerUseReadinessInput,
    COMPUTER_USE_READINESS_RESOURCE_URI, COMPUTER_USE_READINESS_SCHEMA_VERSION,
};
use script_kit_gpui::platform::permiso_detect::PermissionStatus;

#[test]
fn readiness_receipt_is_ready_only_when_all_required_inputs_are_available() {
    let receipt = build_computer_use_readiness_receipt(ComputerUseReadinessInput {
        enabled: true,
        accessibility: PermissionStatus::Authorized,
        screen_recording: PermissionStatus::Authorized,
        keyboard_backend_available: true,
        visible_window_count: Some(2),
        focused_target: Some("TextEdit".to_string()),
        last_error: None,
    });

    assert_eq!(
        receipt.schema_version,
        COMPUTER_USE_READINESS_SCHEMA_VERSION
    );
    assert!(receipt.ready);
    assert!(receipt.attention.is_empty());
    assert!(receipt.redacted);
}

#[test]
fn readiness_receipt_fails_closed_for_missing_permissions_and_windows() {
    let receipt = build_computer_use_readiness_receipt(ComputerUseReadinessInput {
        enabled: false,
        accessibility: PermissionStatus::Denied,
        screen_recording: PermissionStatus::Denied,
        keyboard_backend_available: false,
        visible_window_count: Some(0),
        focused_target: None,
        last_error: Some("previous failure".to_string()),
    });

    let codes: Vec<&str> = receipt
        .attention
        .iter()
        .map(|attention| attention.code)
        .collect();

    assert!(!receipt.ready);
    assert!(codes.contains(&"computer_use_disabled"));
    assert!(codes.contains(&"accessibility_missing"));
    assert!(codes.contains(&"screen_recording_missing"));
    assert!(codes.contains(&"keyboard_backend_unavailable"));
    assert!(codes.contains(&"no_visible_windows"));
    assert!(codes.contains(&"last_error_present"));
}

#[test]
fn readiness_receipt_treats_unknown_window_inventory_as_not_ready() {
    let receipt = build_computer_use_readiness_receipt(ComputerUseReadinessInput {
        enabled: true,
        accessibility: PermissionStatus::Authorized,
        screen_recording: PermissionStatus::Authorized,
        keyboard_backend_available: true,
        visible_window_count: None,
        focused_target: None,
        last_error: None,
    });

    assert!(!receipt.ready);
    assert!(receipt.attention.is_empty());
}

#[test]
fn mcp_resources_expose_computer_use_readiness_resource() {
    let definitions = script_kit_gpui::mcp_resources::get_resource_definitions();

    assert!(definitions
        .iter()
        .any(|resource| resource.uri == COMPUTER_USE_READINESS_RESOURCE_URI));

    let resource = script_kit_gpui::mcp_resources::read_resource(
        COMPUTER_USE_READINESS_RESOURCE_URI,
        &[],
        &[],
        None,
    )
    .expect("read readiness resource");
    let json: serde_json::Value = serde_json::from_str(&resource.text).expect("parse readiness");

    assert_eq!(resource.uri, COMPUTER_USE_READINESS_RESOURCE_URI);
    assert_eq!(json["schemaVersion"], COMPUTER_USE_READINESS_SCHEMA_VERSION);
    assert_eq!(json["mode"], "readOnlyPreflight");
    assert_eq!(json["redacted"], true);
}
