//! Source audits for the actions popup structured observability contract.
//!
//! Validates that the actions popup lifecycle (open, close, resize, route)
//! emits structured `tracing` events under the `ACTIONS_POPUP` target and
//! that the mini-mode position resolver branches on `MainWindowMode`.

use super::read_source as read;

#[test]
fn actions_popup_event_enum_defined_in_window() {
    let content = read("src/actions/window.rs");
    assert!(
        content.contains("pub(crate) enum ActionsPopupEvent"),
        "actions/window.rs must define ActionsPopupEvent enum"
    );
    for variant in ["OpenRequested", "OpenSucceeded", "OpenFailed", "RoutedKey", "Resized", "Closed"] {
        assert!(
            content.contains(variant),
            "ActionsPopupEvent must include variant {variant}"
        );
    }
}

#[test]
fn emit_actions_popup_event_uses_structured_tracing_target() {
    let content = read("src/actions/window.rs");
    assert!(
        content.contains("target: \"ACTIONS_POPUP\""),
        "emit_actions_popup_event must emit under the ACTIONS_POPUP tracing target"
    );
}

#[test]
fn open_actions_window_emits_open_succeeded_receipt() {
    let content = read("src/actions/window.rs");
    let open_fn_start = content
        .find("pub fn open_actions_window(")
        .expect("open_actions_window function not found");
    let open_fn = &content[open_fn_start..];

    assert!(
        open_fn.contains("ActionsPopupEvent::OpenSucceeded"),
        "open_actions_window must emit OpenSucceeded receipt"
    );
    assert!(
        open_fn.contains("Some(position)"),
        "OpenSucceeded receipt must include the window position"
    );
}

#[test]
fn close_actions_window_emits_closed_receipt() {
    let content = read("src/actions/window.rs");
    let close_fn_start = content
        .find("pub fn close_actions_window(")
        .expect("close_actions_window function not found");
    let close_fn = &content[close_fn_start..];

    assert!(
        close_fn.contains("ActionsPopupEvent::Closed"),
        "close_actions_window must emit Closed receipt"
    );
}

#[test]
fn resize_actions_window_direct_emits_resized_receipt() {
    let content = read("src/actions/window.rs");
    let resize_fn_start = content
        .find("pub fn resize_actions_window_direct(")
        .expect("resize_actions_window_direct function not found");
    let resize_fn = &content[resize_fn_start..];

    assert!(
        resize_fn.contains("ActionsPopupEvent::Resized"),
        "resize_actions_window_direct must emit Resized receipt"
    );
}

#[test]
fn toggle_actions_emits_open_requested_receipt() {
    let content = read("src/app_impl/actions_toggle.rs");
    let toggle_fn_start = content
        .find("pub(crate) fn toggle_actions(")
        .expect("toggle_actions function not found");
    let toggle_fn = &content[toggle_fn_start..];

    assert!(
        toggle_fn.contains("ActionsPopupEvent::OpenRequested"),
        "toggle_actions must emit OpenRequested receipt"
    );
}

#[test]
fn close_actions_popup_delegates_close_receipt_to_window_layer() {
    let content = read("src/app_impl/actions_dialog.rs");
    let close_fn_start = content
        .find("pub(crate) fn close_actions_popup(")
        .expect("close_actions_popup function not found");
    let close_fn = &content[close_fn_start..];

    assert!(
        close_fn.contains("close_actions_window(cx);"),
        "close_actions_popup should delegate the Closed receipt to close_actions_window()"
    );
    assert!(
        !close_fn.contains("ActionsPopupEvent::Closed"),
        "close_actions_popup must not emit a duplicate Closed receipt"
    );
}

#[test]
fn main_list_actions_window_position_branches_on_window_mode() {
    let content = read("src/app_impl/actions_toggle.rs");
    let pos_fn_start = content
        .find("fn main_list_actions_window_position(")
        .expect("main_list_actions_window_position function not found");
    let pos_fn = &content[pos_fn_start..pos_fn_start + 400];

    assert!(
        pos_fn.contains("MainWindowMode::Mini"),
        "main_list_actions_window_position must branch on Mini mode"
    );
    assert!(
        pos_fn.contains("MainWindowMode::Full"),
        "main_list_actions_window_position must branch on Full mode"
    );
    assert!(
        pos_fn.contains("TopCenter"),
        "Mini mode should resolve to TopCenter position"
    );
    assert!(
        pos_fn.contains("BottomRight"),
        "Full mode should resolve to BottomRight position"
    );
}

#[test]
fn toggle_actions_uses_resolved_position() {
    let content = read("src/app_impl/actions_toggle.rs");
    let toggle_fn_start = content
        .find("pub(crate) fn toggle_actions(")
        .expect("toggle_actions function not found");
    let toggle_fn = &content[toggle_fn_start..];

    assert!(
        toggle_fn.contains("self.main_list_actions_window_position()"),
        "toggle_actions must call main_list_actions_window_position()"
    );
}

#[test]
fn spawn_open_emits_open_failed_on_error() {
    let content = read("src/app_impl/actions_toggle.rs");
    let spawn_fn_start = content
        .find("fn spawn_open_actions_window(")
        .expect("spawn_open_actions_window function not found");
    let spawn_fn = &content[spawn_fn_start..];

    assert!(
        spawn_fn.contains("ActionsPopupEvent::OpenFailed"),
        "spawn_open_actions_window must emit OpenFailed on error"
    );
}
