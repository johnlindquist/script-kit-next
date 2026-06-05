use std::fs;

#[test]
fn permiso_teardown_contract_retains_one_active_handle_and_drops_controller() {
    let permiso = fs::read_to_string("src/platform/permiso/mod.rs").expect("read permiso mod");
    let overlay =
        fs::read_to_string("src/platform/permiso/overlay_window.rs").expect("read overlay");

    for required in [
        "ACTIVE_PERMISO_HANDLE",
        "parking_lot::Mutex<Option<PermisoHandle>>",
        "present_retained",
        "*active_permiso_handle().lock() = Some(handle)",
        "dismiss_active",
    ] {
        assert!(
            permiso.contains(required),
            "permission assistant handle retention missing {required}"
        );
    }

    for required in [
        "impl Drop for PermisoHandle",
        "controller.dismiss()",
        "impl Drop for OverlayController",
        "self.overlay = None",
    ] {
        assert!(
            permiso.contains(required) || overlay.contains(required),
            "permission assistant teardown missing {required}"
        );
    }
}
