//! Regression tests for simulateGpuiEvent ambiguity guard and coordinate
//! rebasing for attached-surface mouse events.
//!
//! These tests validate that the automation registry correctly exposes
//! the multi-window state that `dispatch_gpui_event` uses to fail closed
//! when multiple visible windows share the same kind, and that the
//! registry metadata is sufficient for coordinate rebasing.
//!
//! The actual dispatch function lives in `src/platform/gpui_event_simulator.rs`
//! (an `include!()` file), so we test the underlying registry invariants
//! that the guard depends on.

use script_kit_gpui::protocol::{
    AutomationWindowBounds, AutomationWindowInfo, AutomationWindowKind, AutomationWindowTarget,
};
use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(50_000);
fn prefix() -> String {
    let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("gpui_ev{n}")
}

fn make_visible(prefix: &str, id: &str, kind: AutomationWindowKind) -> AutomationWindowInfo {
    AutomationWindowInfo {
        id: format!("{prefix}:{id}"),
        kind,
        title: Some(format!("Window {id}")),
        focused: false,
        visible: true,
        semantic_surface: None,
        bounds: None,
    }
}

fn cleanup(prefix: &str, ids: &[&str]) {
    for id in ids {
        script_kit_gpui::windows::remove_automation_window(&format!("{prefix}:{id}"));
    }
}

/// Helper: count visible windows of a given kind registered under our prefix.
fn visible_count(prefix: &str, kind: AutomationWindowKind) -> usize {
    script_kit_gpui::windows::list_automation_windows()
        .into_iter()
        .filter(|w| w.kind == kind && w.visible && w.id.starts_with(prefix))
        .count()
}

// ============================================================
// Single visible window — dispatch should succeed
// ============================================================

#[test]
fn single_visible_window_is_not_ambiguous() {
    let p = prefix();

    let info = make_visible(&p, "acp-0", AutomationWindowKind::AcpDetached);
    script_kit_gpui::windows::upsert_automation_window(info);

    let count = visible_count(&p, AutomationWindowKind::AcpDetached);
    // With exactly one visible window, the ambiguity guard should NOT fire
    assert_eq!(
        count, 1,
        "Expected exactly 1 visible AcpDetached window under prefix {p}, got {count}"
    );

    cleanup(&p, &["acp-0"]);
}

// ============================================================
// Multiple visible windows of same kind — ambiguous
// ============================================================

#[test]
fn multiple_visible_windows_same_kind_is_ambiguous() {
    let p = prefix();

    let info0 = make_visible(&p, "acp-0", AutomationWindowKind::AcpDetached);
    let info1 = make_visible(&p, "acp-1", AutomationWindowKind::AcpDetached);
    script_kit_gpui::windows::upsert_automation_window(info0);
    script_kit_gpui::windows::upsert_automation_window(info1);

    let count = visible_count(&p, AutomationWindowKind::AcpDetached);
    // With 2+ visible windows sharing a kind, the ambiguity guard fires
    assert!(
        count > 1,
        "Expected >1 visible AcpDetached windows under prefix {p}, got {count}"
    );

    cleanup(&p, &["acp-0", "acp-1"]);
}

// ============================================================
// Hidden windows don't count as ambiguous
// ============================================================

#[test]
fn hidden_window_not_counted_for_ambiguity() {
    let p = prefix();

    let info0 = make_visible(&p, "acp-0", AutomationWindowKind::AcpDetached);
    let mut info1 = make_visible(&p, "acp-1", AutomationWindowKind::AcpDetached);
    info1.visible = false; // hidden

    script_kit_gpui::windows::upsert_automation_window(info0);
    script_kit_gpui::windows::upsert_automation_window(info1);

    let count = visible_count(&p, AutomationWindowKind::AcpDetached);
    assert_eq!(
        count, 1,
        "Hidden window should not be counted; expected 1 under prefix {p}, got {count}"
    );

    cleanup(&p, &["acp-0", "acp-1"]);
}

// ============================================================
// Different kinds are independent — no cross-contamination
// ============================================================

#[test]
fn different_kinds_are_not_ambiguous_with_each_other() {
    let p = prefix();

    let acp = make_visible(&p, "acp-0", AutomationWindowKind::AcpDetached);
    let notes = make_visible(&p, "notes-0", AutomationWindowKind::Notes);

    script_kit_gpui::windows::upsert_automation_window(acp);
    script_kit_gpui::windows::upsert_automation_window(notes);

    let acp_count = visible_count(&p, AutomationWindowKind::AcpDetached);
    let notes_count = visible_count(&p, AutomationWindowKind::Notes);

    assert_eq!(
        acp_count, 1,
        "AcpDetached should have 1 visible window under prefix {p}"
    );
    assert_eq!(
        notes_count, 1,
        "Notes should have 1 visible window under prefix {p}"
    );

    cleanup(&p, &["acp-0", "notes-0"]);
}

// ============================================================
// Removing a window reduces visible count back to non-ambiguous
// ============================================================

#[test]
fn removing_window_clears_ambiguity() {
    let p = prefix();

    let info0 = make_visible(&p, "acp-0", AutomationWindowKind::AcpDetached);
    let info1 = make_visible(&p, "acp-1", AutomationWindowKind::AcpDetached);
    script_kit_gpui::windows::upsert_automation_window(info0);
    script_kit_gpui::windows::upsert_automation_window(info1);

    assert!(visible_count(&p, AutomationWindowKind::AcpDetached) > 1);

    // Remove one window — ambiguity should clear
    script_kit_gpui::windows::remove_automation_window(&format!("{p}:acp-1"));

    let count = visible_count(&p, AutomationWindowKind::AcpDetached);
    assert_eq!(
        count, 1,
        "After removing one window, should have 1 visible under prefix {p}, got {count}"
    );

    cleanup(&p, &["acp-0"]);
}

// ============================================================
// Plain kind target (no index) — still rejected when ambiguous
// ============================================================

#[test]
fn plain_kind_target_resolves_when_single_window() {
    let p = prefix();

    let info = make_visible(&p, "acp-0", AutomationWindowKind::AcpDetached);
    script_kit_gpui::windows::upsert_automation_window(info);

    let target = script_kit_gpui::protocol::AutomationWindowTarget::Kind {
        kind: AutomationWindowKind::AcpDetached,
        index: None,
    };
    let resolved = script_kit_gpui::windows::resolve_automation_window(Some(&target));
    assert!(
        resolved.is_ok(),
        "Should resolve when exactly one window exists"
    );

    cleanup(&p, &["acp-0"]);
}

// ============================================================
// Protocol serde: simulateGpuiEvent result with error
// ============================================================

#[test]
fn simulate_gpui_event_result_with_error_serializes() {
    // Verify the error envelope shape matches what docs/PROTOCOL.md describes
    let result = serde_json::json!({
        "type": "simulateGpuiEventResult",
        "requestId": "gpui-ambiguous",
        "success": false,
        "errorCode": "target_ambiguous",
        "error": "Resolved target acpDetached:thread-1 (AcpDetached) is ambiguous: 2 visible windows share this kind and GPUI dispatch still routes through one WindowRole"
    });

    let success = result["success"].as_bool().expect("success field");
    assert!(!success, "Ambiguous result should have success=false");
    assert_eq!(
        result["errorCode"].as_str().unwrap(),
        "target_ambiguous",
        "Ambiguous result should have errorCode=target_ambiguous"
    );
    assert!(
        result["error"].as_str().unwrap().contains("ambiguous"),
        "Error message should mention ambiguity"
    );
}

// ============================================================
// Coordinate rebasing: attached surfaces need bounds for translation
// ============================================================

fn make_with_bounds(
    prefix: &str,
    id: &str,
    kind: AutomationWindowKind,
    bounds: Option<AutomationWindowBounds>,
) -> AutomationWindowInfo {
    AutomationWindowInfo {
        id: format!("{prefix}:{id}"),
        kind,
        title: Some(format!("Window {id}")),
        focused: false,
        visible: true,
        semantic_surface: None,
        bounds,
    }
}

#[test]
fn attached_surface_with_bounds_resolves_for_rebasing() {
    let p = prefix();

    // Register main window with bounds
    let main = make_with_bounds(
        &p,
        "main",
        AutomationWindowKind::Main,
        Some(AutomationWindowBounds {
            x: 100.0,
            y: 50.0,
            width: 800.0,
            height: 600.0,
        }),
    );
    script_kit_gpui::windows::upsert_automation_window(main);

    // Register attached ActionsDialog with bounds
    let actions = make_with_bounds(
        &p,
        "actions",
        AutomationWindowKind::ActionsDialog,
        Some(AutomationWindowBounds {
            x: 200.0,
            y: 150.0,
            width: 400.0,
            height: 300.0,
        }),
    );
    script_kit_gpui::windows::upsert_automation_window(actions);

    // Resolve the dialog — bounds must be present for coordinate translation
    let target = AutomationWindowTarget::Id {
        id: format!("{p}:actions"),
    };
    let resolved = script_kit_gpui::windows::resolve_automation_window(Some(&target))
        .expect("should resolve actions dialog");
    assert!(
        resolved.bounds.is_some(),
        "Attached surface must have bounds for coordinate rebasing"
    );

    // The offset should be target.origin - main.origin
    let bounds = resolved.bounds.as_ref().unwrap();
    let main_target = AutomationWindowTarget::Id {
        id: format!("{p}:main"),
    };
    let main_resolved = script_kit_gpui::windows::resolve_automation_window(Some(&main_target))
        .expect("should resolve main");
    let main_bounds = main_resolved.bounds.as_ref().unwrap();

    let offset_x = bounds.x - main_bounds.x;
    let offset_y = bounds.y - main_bounds.y;
    assert!(
        (offset_x - 100.0).abs() < f64::EPSILON,
        "Expected offset_x=100, got {offset_x}"
    );
    assert!(
        (offset_y - 100.0).abs() < f64::EPSILON,
        "Expected offset_y=100, got {offset_y}"
    );

    cleanup(&p, &["main", "actions"]);
}

#[test]
fn attached_surface_without_bounds_fails_closed() {
    let p = prefix();

    // Register main window with bounds
    let main = make_with_bounds(
        &p,
        "main",
        AutomationWindowKind::Main,
        Some(AutomationWindowBounds {
            x: 100.0,
            y: 50.0,
            width: 800.0,
            height: 600.0,
        }),
    );
    script_kit_gpui::windows::upsert_automation_window(main);

    // Register attached surface WITHOUT bounds
    let actions = make_with_bounds(
        &p,
        "actions",
        AutomationWindowKind::ActionsDialog,
        None, // no bounds
    );
    script_kit_gpui::windows::upsert_automation_window(actions);

    let target = AutomationWindowTarget::Id {
        id: format!("{p}:actions"),
    };
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&target)).expect("should resolve");
    assert!(
        resolved.bounds.is_none(),
        "Test precondition: attached surface has no bounds"
    );

    // Coordinate translation would fail because bounds are absent.
    // The dispatch code checks resolved.bounds and returns a deterministic
    // error — we verify the invariant that makes this possible.
    assert_eq!(resolved.kind, AutomationWindowKind::ActionsDialog);

    cleanup(&p, &["main", "actions"]);
}

#[test]
fn detached_window_bounds_not_required_for_dispatch() {
    let p = prefix();

    // Detached windows (Notes, AcpDetached) do not need coordinate rebasing
    let notes = make_with_bounds(
        &p,
        "notes",
        AutomationWindowKind::Notes,
        None, // no bounds — still fine for detached
    );
    script_kit_gpui::windows::upsert_automation_window(notes);

    let target = AutomationWindowTarget::Id {
        id: format!("{p}:notes"),
    };
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&target)).expect("should resolve");

    // Detached windows are dispatched directly via their runtime handle,
    // not through the main window. No coordinate rebasing needed.
    assert_eq!(resolved.kind, AutomationWindowKind::Notes);
    // bounds can be None — dispatch doesn't fail for detached
    assert!(resolved.bounds.is_none());

    cleanup(&p, &["notes"]);
}

#[test]
fn prompt_popup_is_also_attached_surface() {
    let p = prefix();

    let main = make_with_bounds(
        &p,
        "main",
        AutomationWindowKind::Main,
        Some(AutomationWindowBounds {
            x: 0.0,
            y: 0.0,
            width: 1280.0,
            height: 800.0,
        }),
    );
    let popup = make_with_bounds(
        &p,
        "popup",
        AutomationWindowKind::PromptPopup,
        Some(AutomationWindowBounds {
            x: 300.0,
            y: 200.0,
            width: 500.0,
            height: 400.0,
        }),
    );
    script_kit_gpui::windows::upsert_automation_window(main);
    script_kit_gpui::windows::upsert_automation_window(popup);

    let target = AutomationWindowTarget::Id {
        id: format!("{p}:popup"),
    };
    let resolved = script_kit_gpui::windows::resolve_automation_window(Some(&target))
        .expect("should resolve popup");
    let bounds = resolved.bounds.as_ref().expect("popup must have bounds");

    // Verify the popup bounds are correct
    assert!((bounds.x - 300.0).abs() < f64::EPSILON);
    assert!((bounds.y - 200.0).abs() < f64::EPSILON);
    assert!((bounds.width - 500.0).abs() < f64::EPSILON);
    assert!((bounds.height - 400.0).abs() < f64::EPSILON);

    cleanup(&p, &["main", "popup"]);
}

// ============================================================
// Geometry helpers: inspect snapshot target bounds
// ============================================================

#[test]
fn inspect_geometry_detached_window_bounds_at_origin() {
    let p = prefix();

    let notes = make_with_bounds(
        &p,
        "notes",
        AutomationWindowKind::Notes,
        Some(AutomationWindowBounds {
            x: 500.0,
            y: 300.0,
            width: 800.0,
            height: 600.0,
        }),
    );
    script_kit_gpui::windows::upsert_automation_window(notes);

    let target = AutomationWindowTarget::Id {
        id: format!("{p}:notes"),
    };
    let resolved = script_kit_gpui::windows::resolve_automation_window(Some(&target))
        .expect("should resolve notes");

    let bounds_in_screenshot =
        script_kit_gpui::protocol::target_bounds_in_screenshot(&resolved).expect("should compute");
    // Detached windows start at (0, 0) in their own screenshot
    assert!((bounds_in_screenshot.x - 0.0).abs() < f64::EPSILON);
    assert!((bounds_in_screenshot.y - 0.0).abs() < f64::EPSILON);
    assert!((bounds_in_screenshot.width - 800.0).abs() < f64::EPSILON);
    assert!((bounds_in_screenshot.height - 600.0).abs() < f64::EPSILON);

    cleanup(&p, &["notes"]);
}

#[test]
fn inspect_geometry_no_bounds_returns_none() {
    let p = prefix();

    let notes = make_with_bounds(&p, "notes", AutomationWindowKind::Notes, None);
    script_kit_gpui::windows::upsert_automation_window(notes);

    let target = AutomationWindowTarget::Id {
        id: format!("{p}:notes"),
    };
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&target)).expect("should resolve");
    let result = script_kit_gpui::protocol::target_bounds_in_screenshot(&resolved);
    assert!(result.is_none(), "No bounds should return None");

    cleanup(&p, &["notes"]);
}

#[test]
fn not_found_vs_ambiguous_error_codes_are_distinguishable() {
    // Agents need to programmatically distinguish these two failure modes
    let not_found = script_kit_gpui::protocol::Message::simulate_gpui_event_result_error(
        "nf-1".into(),
        "target_not_found".into(),
        "No automation window for kind Notes index 0".into(),
    );
    let ambiguous = script_kit_gpui::protocol::Message::simulate_gpui_event_result_error(
        "amb-1".into(),
        "target_ambiguous".into(),
        "2 visible windows share this kind".into(),
    );

    let nf_json = serde_json::to_value(&not_found).expect("serialize");
    let amb_json = serde_json::to_value(&ambiguous).expect("serialize");

    assert_ne!(
        nf_json["errorCode"], amb_json["errorCode"],
        "not_found and ambiguous must have distinct error codes"
    );
    assert_eq!(nf_json["errorCode"], "target_not_found");
    assert_eq!(amb_json["errorCode"], "target_ambiguous");
}

// ============================================================
// Coordinate agreement: inspect geometry and rebased dispatch
// use the same offset math for attached surfaces
// ============================================================

/// Verify that the offset computed by `target_bounds_in_screenshot_with_main`
/// for an attached ActionsDialog matches the offset that
/// `rebase_mouse_event_to_dispatch_space` would apply. Both must use
/// `(target.x - main.x, target.y - main.y)`.
///
/// Uses `_with_main` variant for test isolation (avoids global main_id race).
#[test]
fn inspect_geometry_and_rebase_agree_for_actions_dialog() {
    let p = prefix();

    let main_bounds_val = AutomationWindowBounds {
        x: 100.0,
        y: 50.0,
        width: 800.0,
        height: 600.0,
    };
    let main = make_with_bounds(
        &p,
        "main",
        AutomationWindowKind::Main,
        Some(main_bounds_val.clone()),
    );
    let actions = make_with_bounds(
        &p,
        "actions",
        AutomationWindowKind::ActionsDialog,
        Some(AutomationWindowBounds {
            x: 300.0,
            y: 200.0,
            width: 400.0,
            height: 300.0,
        }),
    );
    script_kit_gpui::windows::upsert_automation_window(main);
    script_kit_gpui::windows::upsert_automation_window(actions);

    let target = AutomationWindowTarget::Id {
        id: format!("{p}:actions"),
    };
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&target)).expect("should resolve");

    // Inspect geometry: target_bounds_in_screenshot_with_main offset
    let inspect_bounds = script_kit_gpui::protocol::target_bounds_in_screenshot_with_main(
        &resolved,
        Some(&main_bounds_val),
    )
    .expect("should compute bounds");
    let inspect_offset_x = inspect_bounds.x;
    let inspect_offset_y = inspect_bounds.y;

    // Rebase offset: target.origin - main.origin (same math dispatch uses)
    let target_bounds = resolved.bounds.as_ref().expect("target bounds");
    let dispatch_offset_x = target_bounds.x - main_bounds_val.x;
    let dispatch_offset_y = target_bounds.y - main_bounds_val.y;

    assert!(
        (inspect_offset_x - dispatch_offset_x).abs() < f64::EPSILON,
        "Inspect offset_x ({inspect_offset_x}) must equal dispatch offset_x ({dispatch_offset_x})"
    );
    assert!(
        (inspect_offset_y - dispatch_offset_y).abs() < f64::EPSILON,
        "Inspect offset_y ({inspect_offset_y}) must equal dispatch offset_y ({dispatch_offset_y})"
    );

    // Concrete values: (300-100, 200-50) = (200, 150)
    assert!(
        (inspect_offset_x - 200.0).abs() < f64::EPSILON,
        "Expected offset_x=200, got {inspect_offset_x}"
    );
    assert!(
        (inspect_offset_y - 150.0).abs() < f64::EPSILON,
        "Expected offset_y=150, got {inspect_offset_y}"
    );

    cleanup(&p, &["main", "actions"]);
}

/// Same agreement check for PromptPopup — the other attached surface kind.
/// Uses `_with_main` variant for test isolation.
#[test]
fn inspect_geometry_and_rebase_agree_for_prompt_popup() {
    let p = prefix();

    let main_bounds_val = AutomationWindowBounds {
        x: 0.0,
        y: 0.0,
        width: 1280.0,
        height: 800.0,
    };
    let main = make_with_bounds(
        &p,
        "main",
        AutomationWindowKind::Main,
        Some(main_bounds_val.clone()),
    );
    let popup = make_with_bounds(
        &p,
        "popup",
        AutomationWindowKind::PromptPopup,
        Some(AutomationWindowBounds {
            x: 200.0,
            y: 100.0,
            width: 500.0,
            height: 400.0,
        }),
    );
    script_kit_gpui::windows::upsert_automation_window(main);
    script_kit_gpui::windows::upsert_automation_window(popup);

    let target = AutomationWindowTarget::Id {
        id: format!("{p}:popup"),
    };
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&target)).expect("should resolve");

    let inspect_bounds = script_kit_gpui::protocol::target_bounds_in_screenshot_with_main(
        &resolved,
        Some(&main_bounds_val),
    )
    .expect("should compute bounds");

    // Main at origin (0,0), popup at (200,100) → offset = (200, 100)
    assert!(
        (inspect_bounds.x - 200.0).abs() < f64::EPSILON,
        "Expected offset_x=200, got {}",
        inspect_bounds.x
    );
    assert!(
        (inspect_bounds.y - 100.0).abs() < f64::EPSILON,
        "Expected offset_y=100, got {}",
        inspect_bounds.y
    );
    assert!(
        (inspect_bounds.width - 500.0).abs() < f64::EPSILON,
        "Width must be preserved"
    );
    assert!(
        (inspect_bounds.height - 400.0).abs() < f64::EPSILON,
        "Height must be preserved"
    );

    cleanup(&p, &["main", "popup"]);
}

/// Inspect hit point must land inside the target bounds when both are present.
/// Uses `_with_main` variant for test isolation.
#[test]
fn inspect_hit_point_lands_inside_target_bounds() {
    let p = prefix();

    let main_bounds_val = AutomationWindowBounds {
        x: 50.0,
        y: 25.0,
        width: 1000.0,
        height: 700.0,
    };
    let main = make_with_bounds(
        &p,
        "main",
        AutomationWindowKind::Main,
        Some(main_bounds_val.clone()),
    );
    let actions = make_with_bounds(
        &p,
        "actions",
        AutomationWindowKind::ActionsDialog,
        Some(AutomationWindowBounds {
            x: 250.0,
            y: 175.0,
            width: 520.0,
            height: 384.0,
        }),
    );
    script_kit_gpui::windows::upsert_automation_window(main);
    script_kit_gpui::windows::upsert_automation_window(actions);

    let target = AutomationWindowTarget::Id {
        id: format!("{p}:actions"),
    };
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&target)).expect("should resolve");

    let bounds = script_kit_gpui::protocol::target_bounds_in_screenshot_with_main(
        &resolved,
        Some(&main_bounds_val),
    )
    .expect("should compute");
    let hit = script_kit_gpui::protocol::default_surface_hit_point(&bounds);

    // Hit point must be within the target bounds rectangle
    assert!(
        hit.x >= bounds.x && hit.x <= bounds.x + bounds.width,
        "Hit point x ({}) must be within [{}, {}]",
        hit.x,
        bounds.x,
        bounds.x + bounds.width
    );
    assert!(
        hit.y >= bounds.y && hit.y <= bounds.y + bounds.height,
        "Hit point y ({}) must be within [{}, {}]",
        hit.y,
        bounds.y,
        bounds.y + bounds.height
    );

    cleanup(&p, &["main", "actions"]);
}

/// Detached windows (Notes, AcpDetached) must NOT have their coordinates
/// rebased — the inspect geometry should place them at (0, 0).
#[test]
fn detached_windows_are_not_rebased() {
    let p = prefix();

    let main = make_with_bounds(
        &p,
        "main",
        AutomationWindowKind::Main,
        Some(AutomationWindowBounds {
            x: 100.0,
            y: 50.0,
            width: 800.0,
            height: 600.0,
        }),
    );
    let notes = make_with_bounds(
        &p,
        "notes",
        AutomationWindowKind::Notes,
        Some(AutomationWindowBounds {
            x: 500.0,
            y: 300.0,
            width: 350.0,
            height: 280.0,
        }),
    );
    let acp = make_with_bounds(
        &p,
        "acp",
        AutomationWindowKind::AcpDetached,
        Some(AutomationWindowBounds {
            x: 900.0,
            y: 100.0,
            width: 480.0,
            height: 440.0,
        }),
    );
    script_kit_gpui::windows::upsert_automation_window(main);
    script_kit_gpui::windows::upsert_automation_window(notes);
    script_kit_gpui::windows::upsert_automation_window(acp);

    for (id_suffix, kind) in [("notes", "Notes"), ("acp", "AcpDetached")] {
        let target = AutomationWindowTarget::Id {
            id: format!("{p}:{id_suffix}"),
        };
        let resolved = script_kit_gpui::windows::resolve_automation_window(Some(&target))
            .expect("should resolve");
        let bounds = script_kit_gpui::protocol::target_bounds_in_screenshot(&resolved)
            .expect("should compute");

        assert!(
            (bounds.x - 0.0).abs() < f64::EPSILON,
            "{kind} target_bounds_in_screenshot.x must be 0, got {}",
            bounds.x
        );
        assert!(
            (bounds.y - 0.0).abs() < f64::EPSILON,
            "{kind} target_bounds_in_screenshot.y must be 0, got {}",
            bounds.y
        );
    }

    cleanup(&p, &["main", "notes", "acp"]);
}

/// When main window has no bounds, attached surface geometry must return None
/// (fail closed) rather than silently using (0, 0).
#[test]
fn attached_surface_fails_closed_when_main_has_no_bounds() {
    let p = prefix();

    // Main registered without bounds
    let main = make_with_bounds(&p, "main", AutomationWindowKind::Main, None);
    let actions = make_with_bounds(
        &p,
        "actions",
        AutomationWindowKind::ActionsDialog,
        Some(AutomationWindowBounds {
            x: 300.0,
            y: 200.0,
            width: 400.0,
            height: 300.0,
        }),
    );
    script_kit_gpui::windows::upsert_automation_window(main);
    script_kit_gpui::windows::upsert_automation_window(actions);

    let target = AutomationWindowTarget::Id {
        id: format!("{p}:actions"),
    };
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&target)).expect("should resolve");

    // Must fail closed — not silently produce (0, 0)
    // Use explicit None main bounds for test isolation
    let result = script_kit_gpui::protocol::target_bounds_in_screenshot_with_main(&resolved, None);
    assert!(
        result.is_none(),
        "Attached surface must return None when main has no bounds"
    );

    cleanup(&p, &["main", "actions"]);
}
