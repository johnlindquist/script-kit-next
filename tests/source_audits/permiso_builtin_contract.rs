use std::fs;

// @lat: [[tests/permission-assistant#Permission Assistant#Built-in assistant entry points]]
#[test]
fn permiso_builtin_contract_allow_commands_route_to_retained_assistant() {
    let builtins = fs::read_to_string("src/builtins/mod.rs").expect("read builtins");
    let execution =
        fs::read_to_string("src/app_execute/builtin_execution.rs").expect("read execution");

    for required in [
        "PermissionCommandType::AllowAccessibility",
        "PermissionCommandType::AllowScreenRecording",
        "\"builtin/allow-accessibility\"",
        "\"builtin/allow-screen-recording\"",
    ] {
        assert!(
            builtins.contains(required),
            "builtins must expose {required}"
        );
    }

    for required in [
        "PermisoAssistant::present_retained",
        "PermisoPanel::Accessibility",
        "PermisoPanel::ScreenRecording",
        "\"allow_accessibility\"",
        "\"allow_screen_recording\"",
    ] {
        assert!(
            execution.contains(required),
            "permission execution must route through retained assistant: {required}"
        );
    }

    let permission_arm = execution
        .split("builtins::BuiltInFeature::PermissionCommand(cmd_type) =>")
        .nth(1)
        .expect("permission command arm");
    assert!(
        !permission_arm.contains("prepare_for_submit_hide"),
        "Permission Assistant built-ins must not hide the main prompt through prepare_for_submit_hide"
    );
}
