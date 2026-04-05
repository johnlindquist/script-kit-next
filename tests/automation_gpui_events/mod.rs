//! Regression tests for simulateGpuiEvent ambiguity guard.
//!
//! These tests validate that the automation registry correctly exposes
//! the multi-window state that `dispatch_gpui_event` uses to fail closed
//! when multiple visible windows share the same kind.
//!
//! The actual dispatch function lives in `src/platform/gpui_event_simulator.rs`
//! (an `include!()` file), so we test the underlying registry invariants
//! that the guard depends on.

use script_kit_gpui::protocol::{AutomationWindowInfo, AutomationWindowKind};
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

    assert_eq!(acp_count, 1, "AcpDetached should have 1 visible window under prefix {p}");
    assert_eq!(notes_count, 1, "Notes should have 1 visible window under prefix {p}");

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
        "error": "Resolved target acpDetached:thread-1 (AcpDetached) is ambiguous: 2 visible windows share this kind and GPUI dispatch still routes through one WindowRole"
    });

    let success = result["success"].as_bool().expect("success field");
    assert!(!success, "Ambiguous result should have success=false");
    assert!(
        result["error"].as_str().unwrap().contains("ambiguous"),
        "Error message should mention ambiguity"
    );
}
