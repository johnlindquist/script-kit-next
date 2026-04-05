//! Notes window targeting regression tests.
//!
//! Proves that the automation registry resolves Notes as a distinct
//! window, not the main window, and that Notes-specific metadata
//! is preserved across register/resolve/unregister cycles.

use script_kit_gpui::protocol::{
    AutomationWindowInfo, AutomationWindowKind, AutomationWindowTarget,
};
use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(20_000);
fn prefix() -> String {
    let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("nt{n}")
}

fn cleanup(prefix: &str, ids: &[&str]) {
    for id in ids {
        script_kit_gpui::windows::remove_automation_window(&format!("{prefix}:{id}"));
    }
}

#[test]
fn notes_window_targeting_flow() {
    let p = prefix();

    // Register main window as focused
    let main = AutomationWindowInfo {
        id: format!("{p}:main"),
        kind: AutomationWindowKind::Main,
        title: Some("Script Kit".into()),
        focused: true,
        visible: true,
        semantic_surface: Some("scriptList".into()),
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(main);

    // Register Notes window
    let notes = AutomationWindowInfo {
        id: format!("{p}:notes"),
        kind: AutomationWindowKind::Notes,
        title: Some("Script Kit Notes".into()),
        focused: false,
        visible: true,
        semantic_surface: Some("notes".into()),
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(notes);

    // Target by kind → Notes, not Main
    let resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::Notes,
            index: None,
        }))
        .expect("should resolve Notes");
    assert_eq!(resolved.kind, AutomationWindowKind::Notes);
    assert_eq!(resolved.semantic_surface.as_deref(), Some("notes"));
    assert_eq!(resolved.title.as_deref(), Some("Script Kit Notes"));
    assert_ne!(
        resolved.id,
        format!("{p}:main"),
        "must not fall back to main"
    );

    // Target by title → Notes
    let resolved_title = script_kit_gpui::windows::resolve_automation_window(Some(
        &AutomationWindowTarget::TitleContains {
            text: "Notes".into(),
        },
    ))
    .expect("should resolve by title");
    assert_eq!(resolved_title.kind, AutomationWindowKind::Notes);

    // No target (None) → focused window (Main)
    let focused =
        script_kit_gpui::windows::resolve_automation_window(None).expect("should resolve focused");
    assert_eq!(focused.kind, AutomationWindowKind::Main);

    // Close Notes → should disappear from registry
    let removed = script_kit_gpui::windows::remove_automation_window(&format!("{p}:notes"));
    assert!(removed.is_some());

    // Notes targeting now fails
    let err =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::Notes,
            index: None,
        }));
    assert!(err.is_err(), "Notes should no longer be resolvable");

    cleanup(&p, &["main"]);
}

#[test]
fn notes_window_info_serde_round_trip() {
    let info = AutomationWindowInfo {
        id: "notes:primary".into(),
        kind: AutomationWindowKind::Notes,
        title: Some("Script Kit Notes".into()),
        focused: true,
        visible: true,
        semantic_surface: Some("notes".into()),
        bounds: None,
    };
    let json = serde_json::to_string(&info).expect("serialize");
    let back: AutomationWindowInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, info);
    assert!(json.contains(r#""kind":"notes"#));
}

#[test]
fn notes_focus_transfer_from_main() {
    let p = prefix();

    let main = AutomationWindowInfo {
        id: format!("{p}:main"),
        kind: AutomationWindowKind::Main,
        title: Some("Script Kit".into()),
        focused: true,
        visible: true,
        semantic_surface: Some("scriptList".into()),
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(main);

    let notes = AutomationWindowInfo {
        id: format!("{p}:notes"),
        kind: AutomationWindowKind::Notes,
        title: Some("Script Kit Notes".into()),
        focused: false,
        visible: true,
        semantic_surface: Some("notes".into()),
        bounds: None,
    };
    script_kit_gpui::windows::upsert_automation_window(notes);

    // Transfer focus to Notes
    assert!(script_kit_gpui::windows::set_automation_focus(&format!(
        "{p}:notes"
    )));

    // Focused resolution now returns Notes
    let focused =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Focused))
            .expect("resolve focused");
    assert_eq!(focused.kind, AutomationWindowKind::Notes);

    // Main should be unfocused
    let main_resolved =
        script_kit_gpui::windows::resolve_automation_window(Some(&AutomationWindowTarget::Main))
            .expect("resolve main");
    assert!(!main_resolved.focused);

    cleanup(&p, &["main", "notes"]);
}
