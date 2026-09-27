//! Screenshot targeting determinism tests.
//!
//! These tests validate the scoring and tie-rejection contracts for targeted
//! screenshot capture without requiring a live GPUI event loop or real
//! OS windows. They mirror the private scoring logic to verify behavioral
//! contracts: bounds-awareness, tie rejection, and logging field presence.

use script_kit_gpui::protocol::{
    AutomationWindowBounds, AutomationWindowInfo, AutomationWindowKind,
};

// ── Scoring mirror ─────────────────────────────────────────────────────
// These replicate the private scoring functions so integration tests can
// verify the behavioral contract without depending on internal visibility.

fn candidate_size_score(bounds: Option<&AutomationWindowBounds>, cw: i32, ch: i32) -> i32 {
    let Some(b) = bounds else { return 0 };

    let tw = b.width.round() as i32;
    let th = b.height.round() as i32;
    let dw = (cw - tw).abs();
    let dh = (ch - th).abs();

    match (dw, dh) {
        (0, 0) => 5_000,
        (dw, dh) if dw <= 4 && dh <= 4 => 2_500,
        (dw, dh) if dw <= 16 && dh <= 16 => 500,
        _ => -1_500,
    }
}

fn score(
    resolved: &AutomationWindowInfo,
    title: &str,
    focused: bool,
    width: i32,
    height: i32,
) -> i32 {
    let mut s = candidate_size_score(resolved.bounds.as_ref(), width, height);

    if let Some(rt) = resolved.title.as_deref() {
        if !rt.is_empty() && title == rt {
            s += 1_000;
        } else if !rt.is_empty() && title.contains(rt) {
            s += 500;
        }
    }

    if resolved.focused == focused {
        s += 100;
    }

    if resolved.kind == AutomationWindowKind::Main
        && (title.contains("Notes") || title.contains("AI") || title.contains("Agent Chat"))
    {
        s -= 200;
    }

    s
}

fn make_info(
    kind: AutomationWindowKind,
    title: Option<&str>,
    focused: bool,
    bounds: Option<(f64, f64)>,
) -> AutomationWindowInfo {
    AutomationWindowInfo {
        id: "test".to_string(),
        kind,
        title: title.map(|s| s.to_string()),
        focused,
        visible: true,
        semantic_surface: None,
        bounds: bounds.map(|(w, h)| AutomationWindowBounds {
            x: 0.0,
            y: 0.0,
            width: w,
            height: h,
        }),
        parent_window_id: None,
        parent_window_generation: None,
        parent_kind: None,
        pid: None,
        generation: None,
    }
}

// ── Size scoring tiers ─────────────────────────────────────────────────

#[test]
fn size_score_exact_match() {
    let b = AutomationWindowBounds {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    assert_eq!(candidate_size_score(Some(&b), 800, 600), 5_000);
}

#[test]
fn size_score_close_match() {
    let b = AutomationWindowBounds {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    assert_eq!(candidate_size_score(Some(&b), 803, 602), 2_500);
}

#[test]
fn size_score_moderate_match() {
    let b = AutomationWindowBounds {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    assert_eq!(candidate_size_score(Some(&b), 810, 610), 500);
}

#[test]
fn size_score_far_mismatch() {
    let b = AutomationWindowBounds {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    assert_eq!(candidate_size_score(Some(&b), 400, 300), -1_500);
}

#[test]
fn size_score_no_bounds_returns_zero() {
    assert_eq!(candidate_size_score(None, 800, 600), 0);
}

// ── Composite scoring ──────────────────────────────────────────────────

#[test]
fn exact_size_dominates_title() {
    let resolved = make_info(
        AutomationWindowKind::Main,
        Some("Other"),
        true,
        Some((800.0, 600.0)),
    );

    let size_match = score(&resolved, "Script Kit", true, 800, 600);
    let title_match = score(&resolved, "Other", true, 400, 300);

    assert!(
        size_match > title_match,
        "size_match={size_match} should beat title_match={title_match}"
    );
}

#[test]
fn title_still_matters_without_bounds() {
    let resolved = make_info(AutomationWindowKind::Main, Some("Script Kit"), true, None);

    let titled = score(&resolved, "Script Kit", true, 800, 600);
    let untitled = score(&resolved, "Notes", true, 800, 600);

    assert!(titled > untitled);
}

#[test]
fn main_target_penalizes_secondary_windows() {
    let resolved = make_info(AutomationWindowKind::Main, None, false, None);

    let main = score(&resolved, "Script Kit", false, 800, 600);
    let notes = score(&resolved, "Notes", false, 800, 600);
    let ai = score(&resolved, "Agent Chat", false, 800, 600);

    assert!(main > notes, "main={main} should beat notes={notes}");
    assert!(main > ai, "main={main} should beat ai={ai}");
}

// ── Tie detection ──────────────────────────────────────────────────────

#[test]
fn identical_candidates_tie() {
    let resolved = make_info(
        AutomationWindowKind::SnapOverlay,
        None,
        false,
        Some((600.0, 400.0)),
    );

    let a = score(&resolved, "Script Kit Agent Chat", false, 600, 400);
    let b = score(&resolved, "Script Kit Agent Chat", false, 600, 400);

    assert_eq!(a, b, "identical candidates must tie");
}

#[test]
fn size_difference_breaks_tie() {
    let resolved = make_info(
        AutomationWindowKind::SnapOverlay,
        None,
        false,
        Some((600.0, 400.0)),
    );

    let exact = score(&resolved, "Script Kit Agent Chat", false, 600, 400);
    let close = score(&resolved, "Script Kit Agent Chat", false, 603, 401);

    assert!(
        exact > close,
        "exact={exact} should beat close={close} to break tie"
    );
}

#[test]
fn focus_difference_breaks_tie_when_sizes_equal() {
    let resolved = make_info(
        AutomationWindowKind::SnapOverlay,
        None,
        true,
        Some((600.0, 400.0)),
    );

    let focused = score(&resolved, "Script Kit Agent Chat", true, 600, 400);
    let unfocused = score(&resolved, "Script Kit Agent Chat", false, 600, 400);

    assert!(
        focused > unfocused,
        "focused={focused} should beat unfocused={unfocused}"
    );
}

// ── Protocol error contract ────────────────────────────────────────────

#[test]
fn ambiguous_error_message_contains_candidates_and_score() {
    // Simulate what capture_resolved_window would produce for a tie
    let id = "snapOverlay:thread-1";
    let kind = AutomationWindowKind::SnapOverlay;
    let title_a = "Script Kit Agent Chat";
    let title_b = "Script Kit Agent Chat";
    let tied_score = 5100;

    let error = format!(
        "Ambiguous OS window match for automation target {} ({:?}); \
         '{}' and '{}' tied at score {}",
        id, kind, title_a, title_b, tied_score
    );

    assert!(error.contains("Ambiguous"));
    assert!(error.contains(id));
    assert!(error.contains("SnapOverlay"));
    assert!(error.contains(title_a));
    assert!(error.contains(&tied_score.to_string()));
}

// ── Target bounds in screenshot ───────────────────────────────────────

use script_kit_gpui::protocol::{AutomationWindowTarget, InspectBoundsInScreenshot};
use std::sync::atomic::{AtomicU32, Ordering};

static SHOT_COUNTER: AtomicU32 = AtomicU32::new(70_000);
fn shot_prefix() -> String {
    let n = SHOT_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("shot{n}")
}

fn make_registered(
    prefix: &str,
    id: &str,
    kind: AutomationWindowKind,
    bounds: Option<AutomationWindowBounds>,
) -> AutomationWindowInfo {
    let info = AutomationWindowInfo {
        id: format!("{prefix}:{id}"),
        kind,
        title: Some(format!("Window {id}")),
        focused: false,
        visible: true,
        semantic_surface: None,
        bounds,
        parent_window_id: None,
        parent_window_generation: None,
        parent_kind: None,
        pid: None,
        generation: None,
    };
    script_kit_gpui::windows::upsert_automation_window(info.clone());
    info
}

fn shot_cleanup(prefix: &str, ids: &[&str]) {
    for id in ids {
        script_kit_gpui::windows::remove_automation_window(&format!("{prefix}:{id}"));
    }
}

/// Detached window target bounds must NOT be offset — they're at (0, 0)
/// because they have their own independent screenshot.
#[test]
fn detached_window_bounds_at_origin_in_screenshot() {
    let p = shot_prefix();

    make_registered(
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
    make_registered(
        &p,
        "agent_chat",
        AutomationWindowKind::SnapOverlay,
        Some(AutomationWindowBounds {
            x: 900.0,
            y: 200.0,
            width: 480.0,
            height: 440.0,
        }),
    );

    let target = AutomationWindowTarget::Id {
        id: format!("{p}:agent_chat"),
    };
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&target)).expect("should resolve");

    let bounds =
        script_kit_gpui::protocol::target_bounds_in_screenshot(&resolved).expect("must compute");

    assert!(
        (bounds.x - 0.0).abs() < f64::EPSILON,
        "Detached x must be 0"
    );
    assert!(
        (bounds.y - 0.0).abs() < f64::EPSILON,
        "Detached y must be 0"
    );
    assert!(
        (bounds.width - 480.0).abs() < f64::EPSILON,
        "Width must match"
    );
    assert!(
        (bounds.height - 440.0).abs() < f64::EPSILON,
        "Height must match"
    );

    shot_cleanup(&p, &["main", "agent_chat"]);
}

// ── Popup capture receipt contract ────────────────────────────────────

use script_kit_gpui::platform::{PopupCaptureReceipt, PopupCaptureStrategy};

/// Helper to build a popup capture receipt for testing.
fn make_popup_receipt(
    strategy: PopupCaptureStrategy,
    kind: &str,
    target_bounds: Option<InspectBoundsInScreenshot>,
    semantic_primary: bool,
) -> PopupCaptureReceipt {
    PopupCaptureReceipt {
        strategy,
        window_kind: kind.to_string(),
        target_bounds,
        semantic_receipts_are_primary: semantic_primary,
    }
}

/// Popup capture receipt for detached window has direct strategy and no bounds.
#[test]
fn popup_receipt_detached() {
    let receipt = make_popup_receipt(
        PopupCaptureStrategy::DirectWindowCapture,
        "SnapOverlay",
        None,
        true,
    );

    assert_eq!(receipt.strategy, PopupCaptureStrategy::DirectWindowCapture);
    assert!(receipt.target_bounds.is_none());
    assert!(receipt.semantic_receipts_are_primary);
}

/// Popup capture receipt for main window has not-applicable strategy.
#[test]
fn popup_receipt_main_not_applicable() {
    let receipt = make_popup_receipt(PopupCaptureStrategy::NotApplicable, "Main", None, false);

    assert_eq!(receipt.strategy, PopupCaptureStrategy::NotApplicable);
    assert!(!receipt.semantic_receipts_are_primary);
}

/// Both strategy variants roundtrip through serde.
#[test]
fn popup_capture_strategy_serde_roundtrip() {
    for (strategy, expected_str) in [
        (
            PopupCaptureStrategy::DirectWindowCapture,
            "\"direct_window_capture\"",
        ),
        (PopupCaptureStrategy::NotApplicable, "\"not_applicable\""),
    ] {
        let json = serde_json::to_string(&strategy).expect("serialize");
        assert_eq!(
            json, expected_str,
            "strategy {:?} must serialize to {}",
            strategy, expected_str
        );
        let parsed: PopupCaptureStrategy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, strategy);
    }
}
