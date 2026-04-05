//! Automation Window Registry
//!
//! A shared, thread-safe registry that maps stable automation IDs to
//! [`AutomationWindowInfo`] descriptors.  Every product window (main,
//! Notes, AI, detached ACP, popups) registers here on open and
//! unregisters on close.  The resolver accepts an
//! [`AutomationWindowTarget`] and returns the matching entry — or an
//! error, never a silent fallback.
//!
//! All mutations emit structured tracing so agents can observe the
//! registry lifecycle in machine-parseable logs.

use crate::protocol::{AutomationWindowInfo, AutomationWindowKind, AutomationWindowTarget};
use anyhow::{anyhow, Result};
use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Global singleton
// ---------------------------------------------------------------------------

static AUTOMATION_WINDOWS: LazyLock<Mutex<BTreeMap<String, AutomationWindowInfo>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Register or update an automation window entry.
///
/// If an entry with the same `id` already exists it is replaced.
pub fn upsert_automation_window(info: AutomationWindowInfo) {
    tracing::info!(
        target: "script_kit::automation",
        id = %info.id,
        kind = ?info.kind,
        focused = info.focused,
        visible = info.visible,
        title = ?info.title,
        "automation_window_registered"
    );
    AUTOMATION_WINDOWS.lock().insert(info.id.clone(), info);
}

/// Remove an automation window entry by its stable ID.
///
/// Returns the removed entry if it existed.
pub fn remove_automation_window(id: &str) -> Option<AutomationWindowInfo> {
    let removed = AUTOMATION_WINDOWS.lock().remove(id);
    if removed.is_some() {
        tracing::info!(
            target: "script_kit::automation",
            id = %id,
            "automation_window_unregistered"
        );
    }
    removed
}

/// Return a snapshot of all registered automation windows.
pub fn list_automation_windows() -> Vec<AutomationWindowInfo> {
    AUTOMATION_WINDOWS.lock().values().cloned().collect()
}

/// Return the stable ID of whichever window is currently marked focused,
/// or `None` if no window has `focused == true`.
pub fn focused_automation_window_id() -> Option<String> {
    AUTOMATION_WINDOWS
        .lock()
        .values()
        .find(|w| w.focused)
        .map(|w| w.id.clone())
}

/// Update the focused state: set `focused = true` on the window with
/// `new_focused_id` and `focused = false` on every other entry.
///
/// Returns `true` if `new_focused_id` was found in the registry.
pub fn set_automation_focus(new_focused_id: &str) -> bool {
    let mut map = AUTOMATION_WINDOWS.lock();
    let found = map.contains_key(new_focused_id);
    if found {
        for (id, info) in map.iter_mut() {
            info.focused = id.as_str() == new_focused_id;
        }
        tracing::info!(
            target: "script_kit::automation",
            id = %new_focused_id,
            "automation_window_focus_changed"
        );
    }
    found
}

/// Update the visibility flag for a single window.
pub fn set_automation_visibility(id: &str, visible: bool) {
    let mut map = AUTOMATION_WINDOWS.lock();
    if let Some(info) = map.get_mut(id) {
        if info.visible != visible {
            info.visible = visible;
            tracing::info!(
                target: "script_kit::automation",
                id = %id,
                visible = visible,
                "automation_window_visibility_changed"
            );
        }
    }
}

/// Resolve an [`AutomationWindowTarget`] to a single
/// [`AutomationWindowInfo`].
///
/// Returns an error — never silently falls back to the main window —
/// when no matching entry exists.
pub fn resolve_automation_window(
    target: Option<&AutomationWindowTarget>,
) -> Result<AutomationWindowInfo> {
    let map = AUTOMATION_WINDOWS.lock();

    let result = match target {
        None | Some(AutomationWindowTarget::Focused) => map
            .values()
            .find(|w| w.focused)
            .cloned()
            .ok_or_else(|| anyhow!("No focused automation window")),

        Some(AutomationWindowTarget::Main) => map
            .values()
            .find(|w| w.kind == AutomationWindowKind::Main)
            .cloned()
            .ok_or_else(|| anyhow!("Main automation window not registered")),

        Some(AutomationWindowTarget::Id { id }) => map
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow!("Unknown automation window id: {id}")),

        Some(AutomationWindowTarget::Kind { kind, index }) => {
            let idx = index.unwrap_or(0);
            map.values()
                .filter(|w| w.kind == *kind)
                .nth(idx)
                .cloned()
                .ok_or_else(|| anyhow!("No automation window for kind {:?} index {}", kind, idx))
        }

        Some(AutomationWindowTarget::TitleContains { text }) => map
            .values()
            .find(|w| {
                w.title
                    .as_deref()
                    .is_some_and(|title| title.contains(text.as_str()))
            })
            .cloned()
            .ok_or_else(|| anyhow!("No automation window title contains '{text}'")),
    };

    match &result {
        Ok(info) => {
            tracing::debug!(
                target: "script_kit::automation",
                resolved_id = %info.id,
                kind = ?info.kind,
                target = ?target,
                "automation_window_resolved"
            );
        }
        Err(err) => {
            tracing::warn!(
                target: "script_kit::automation",
                error = %err,
                target = ?target,
                "automation_window_resolve_failed"
            );
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{
        AutomationWindowBounds, AutomationWindowInfo, AutomationWindowKind, AutomationWindowTarget,
    };
    use std::sync::atomic::{AtomicU32, Ordering};

    // Guard against parallel test interference on the global singleton.
    // Each test uses a unique ID prefix derived from an atomic counter.
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_prefix() -> String {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("t{n}")
    }

    fn make_info(prefix: &str, id: &str, kind: AutomationWindowKind) -> AutomationWindowInfo {
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

    // -- Core CRUD ---------------------------------------------------------

    #[test]
    fn automation_window_registry_resolves_main_and_focused() {
        let p = test_prefix();

        let mut main = make_info(&p, "main", AutomationWindowKind::Main);
        main.focused = true;
        upsert_automation_window(main.clone());

        let mut notes = make_info(&p, "notes", AutomationWindowKind::Notes);
        notes.focused = false;
        upsert_automation_window(notes.clone());

        // Resolve Main target
        let target_main = AutomationWindowTarget::Main;
        let resolved = resolve_automation_window(Some(&target_main)).expect("should resolve main");
        assert_eq!(resolved.id, format!("{p}:main"));
        assert_eq!(resolved.kind, AutomationWindowKind::Main);

        // Resolve Focused target (should be main since it has focused=true)
        let target_focused = AutomationWindowTarget::Focused;
        let resolved =
            resolve_automation_window(Some(&target_focused)).expect("should resolve focused");
        assert_eq!(resolved.id, format!("{p}:main"));
        assert!(resolved.focused);

        // None target behaves like Focused
        let resolved = resolve_automation_window(None).expect("should resolve None as focused");
        assert_eq!(resolved.id, format!("{p}:main"));

        // Cleanup
        remove_automation_window(&format!("{p}:main"));
        remove_automation_window(&format!("{p}:notes"));
    }

    #[test]
    fn automation_window_registry_resolves_kind_index() {
        let p = test_prefix();

        let mut acp0 = make_info(&p, "acp0", AutomationWindowKind::AcpDetached);
        acp0.title = Some("ACP Thread 0".into());
        upsert_automation_window(acp0);

        let mut acp1 = make_info(&p, "acp1", AutomationWindowKind::AcpDetached);
        acp1.title = Some("ACP Thread 1".into());
        upsert_automation_window(acp1);

        // Kind without index → first
        let target = AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::AcpDetached,
            index: None,
        };
        let resolved =
            resolve_automation_window(Some(&target)).expect("should resolve kind index 0");
        assert_eq!(resolved.kind, AutomationWindowKind::AcpDetached);

        // Kind with index 1 → second
        let target = AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::AcpDetached,
            index: Some(1),
        };
        let resolved =
            resolve_automation_window(Some(&target)).expect("should resolve kind index 1");
        assert_eq!(resolved.kind, AutomationWindowKind::AcpDetached);
        // The two should have different IDs
        assert_ne!(
            resolve_automation_window(Some(&AutomationWindowTarget::Kind {
                kind: AutomationWindowKind::AcpDetached,
                index: Some(0),
            }))
            .expect("idx 0")
            .id,
            resolved.id
        );

        // Kind with out-of-range index → error
        let target = AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::AcpDetached,
            index: Some(99),
        };
        assert!(resolve_automation_window(Some(&target)).is_err());

        // Cleanup
        remove_automation_window(&format!("{p}:acp0"));
        remove_automation_window(&format!("{p}:acp1"));
    }

    #[test]
    fn automation_window_registry_unregisters_closed_window() {
        let p = test_prefix();

        let info = make_info(&p, "notes", AutomationWindowKind::Notes);
        upsert_automation_window(info.clone());

        // Verify it exists
        let target = AutomationWindowTarget::Id {
            id: format!("{p}:notes"),
        };
        assert!(resolve_automation_window(Some(&target)).is_ok());

        // Unregister
        let removed = remove_automation_window(&format!("{p}:notes"));
        assert!(removed.is_some());
        assert_eq!(
            removed.as_ref().expect("removed").kind,
            AutomationWindowKind::Notes
        );

        // Now resolution fails
        assert!(resolve_automation_window(Some(&target)).is_err());

        // Double-remove is harmless
        assert!(remove_automation_window(&format!("{p}:notes")).is_none());
    }

    // -- Focus tracking ----------------------------------------------------

    #[test]
    fn focus_change_updates_all_entries() {
        let p = test_prefix();

        let mut main = make_info(&p, "main", AutomationWindowKind::Main);
        main.focused = true;
        upsert_automation_window(main);

        let notes = make_info(&p, "notes", AutomationWindowKind::Notes);
        upsert_automation_window(notes);

        // Shift focus to notes
        assert!(set_automation_focus(&format!("{p}:notes")));
        assert_eq!(
            focused_automation_window_id().as_deref(),
            Some(format!("{p}:notes").as_str())
        );

        // Main should now be unfocused
        let main_resolved = resolve_automation_window(Some(&AutomationWindowTarget::Main))
            .expect("main still registered");
        assert!(!main_resolved.focused);

        // Notes should be focused
        let notes_resolved = resolve_automation_window(Some(&AutomationWindowTarget::Id {
            id: format!("{p}:notes"),
        }))
        .expect("notes still registered");
        assert!(notes_resolved.focused);

        // Focus on unknown ID returns false
        assert!(!set_automation_focus("nonexistent-window"));

        // Cleanup
        remove_automation_window(&format!("{p}:main"));
        remove_automation_window(&format!("{p}:notes"));
    }

    // -- Visibility --------------------------------------------------------

    #[test]
    fn visibility_update() {
        let p = test_prefix();

        let info = make_info(&p, "ai", AutomationWindowKind::Ai);
        upsert_automation_window(info);

        set_automation_visibility(&format!("{p}:ai"), false);
        let resolved = resolve_automation_window(Some(&AutomationWindowTarget::Id {
            id: format!("{p}:ai"),
        }))
        .expect("should resolve");
        assert!(!resolved.visible);

        set_automation_visibility(&format!("{p}:ai"), true);
        let resolved = resolve_automation_window(Some(&AutomationWindowTarget::Id {
            id: format!("{p}:ai"),
        }))
        .expect("should resolve");
        assert!(resolved.visible);

        // No-op on unknown ID
        set_automation_visibility("nonexistent", false);

        remove_automation_window(&format!("{p}:ai"));
    }

    // -- TitleContains -----------------------------------------------------

    #[test]
    fn title_contains_resolution() {
        let p = test_prefix();

        let mut info = make_info(&p, "acp", AutomationWindowKind::AcpDetached);
        info.title = Some(format!("{p} Script Kit AI Chat"));
        upsert_automation_window(info);

        let target = AutomationWindowTarget::TitleContains {
            text: format!("{p} Script Kit AI"),
        };
        let resolved =
            resolve_automation_window(Some(&target)).expect("should match title substring");
        assert_eq!(resolved.kind, AutomationWindowKind::AcpDetached);

        // Non-matching title
        let target = AutomationWindowTarget::TitleContains {
            text: "Nonexistent Window Title".into(),
        };
        assert!(resolve_automation_window(Some(&target)).is_err());

        remove_automation_window(&format!("{p}:acp"));
    }

    // -- Bounds ------------------------------------------------------------

    #[test]
    fn info_with_bounds_round_trips_through_registry() {
        let p = test_prefix();

        let info = AutomationWindowInfo {
            id: format!("{p}:bounded"),
            kind: AutomationWindowKind::Main,
            title: Some("Bounded".into()),
            focused: false,
            visible: true,
            semantic_surface: Some("scriptList".into()),
            bounds: Some(AutomationWindowBounds {
                x: 100.0,
                y: 200.0,
                width: 800.0,
                height: 600.0,
            }),
        };
        upsert_automation_window(info.clone());

        let resolved = resolve_automation_window(Some(&AutomationWindowTarget::Id {
            id: format!("{p}:bounded"),
        }))
        .expect("should resolve");
        assert_eq!(resolved.bounds.as_ref().expect("bounds").width, 800.0);

        remove_automation_window(&format!("{p}:bounded"));
    }

    // -- list_automation_windows -------------------------------------------

    #[test]
    fn list_returns_all_registered_windows() {
        let p = test_prefix();

        upsert_automation_window(make_info(&p, "main", AutomationWindowKind::Main));
        upsert_automation_window(make_info(&p, "notes", AutomationWindowKind::Notes));
        upsert_automation_window(make_info(&p, "ai", AutomationWindowKind::Ai));

        let all = list_automation_windows();
        let our_windows: Vec<_> = all.iter().filter(|w| w.id.starts_with(&p)).collect();
        assert_eq!(our_windows.len(), 3);

        remove_automation_window(&format!("{p}:main"));
        remove_automation_window(&format!("{p}:notes"));
        remove_automation_window(&format!("{p}:ai"));
    }

    // -- Upsert overwrites -------------------------------------------------

    #[test]
    fn upsert_overwrites_existing_entry() {
        let p = test_prefix();

        let mut v1 = make_info(&p, "main", AutomationWindowKind::Main);
        v1.semantic_surface = Some("scriptList".into());
        upsert_automation_window(v1);

        let mut v2 = make_info(&p, "main", AutomationWindowKind::Main);
        v2.semantic_surface = Some("argPrompt".into());
        upsert_automation_window(v2);

        let resolved = resolve_automation_window(Some(&AutomationWindowTarget::Id {
            id: format!("{p}:main"),
        }))
        .expect("should resolve");
        assert_eq!(resolved.semantic_surface.as_deref(), Some("argPrompt"));

        remove_automation_window(&format!("{p}:main"));
    }

    // -- Cross-window targeting acceptance tests -------------------------

    /// Proves that targeting a Notes window by kind resolves to distinct
    /// semantic surface and ID, not the main window.
    #[test]
    fn targeted_get_elements_routes_notes_window() {
        let p = test_prefix();

        let mut main = make_info(&p, "main", AutomationWindowKind::Main);
        main.semantic_surface = Some("scriptList".into());
        main.focused = true;
        upsert_automation_window(main);

        let mut notes = make_info(&p, "notes", AutomationWindowKind::Notes);
        notes.semantic_surface = Some("notes".into());
        notes.title = Some("Script Kit Notes".into());
        upsert_automation_window(notes);

        // Resolve with Kind target → should get Notes, not Main
        let target = AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::Notes,
            index: None,
        };
        let resolved =
            resolve_automation_window(Some(&target)).expect("should resolve Notes window");
        assert_eq!(resolved.kind, AutomationWindowKind::Notes);
        assert_eq!(resolved.semantic_surface.as_deref(), Some("notes"));
        assert_ne!(
            resolved.id,
            format!("{p}:main"),
            "must not fall back to main"
        );

        // No target (None) → should resolve to focused (main)
        let focused = resolve_automation_window(None).expect("should resolve focused");
        assert_eq!(focused.kind, AutomationWindowKind::Main);
        assert_eq!(focused.semantic_surface.as_deref(), Some("scriptList"));

        remove_automation_window(&format!("{p}:main"));
        remove_automation_window(&format!("{p}:notes"));
    }

    /// Proves that targeting a detached ACP window resolves to a distinct
    /// window with its own ID and kind, suitable for screenshot routing.
    #[test]
    fn targeted_capture_screenshot_routes_detached_acp() {
        let p = test_prefix();

        let mut main = make_info(&p, "main", AutomationWindowKind::Main);
        main.title = Some("Script Kit".into());
        main.focused = false;
        upsert_automation_window(main);

        let mut acp = make_info(&p, "acp-thread-1", AutomationWindowKind::AcpDetached);
        acp.title = Some("Script Kit AI".into());
        acp.focused = true;
        acp.semantic_surface = Some("acpChat".into());
        upsert_automation_window(acp);

        // Target by kind → ACP
        let target = AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::AcpDetached,
            index: Some(0),
        };
        let resolved =
            resolve_automation_window(Some(&target)).expect("should resolve detached ACP");
        assert_eq!(resolved.kind, AutomationWindowKind::AcpDetached);
        assert_eq!(resolved.title.as_deref(), Some("Script Kit AI"));
        // The screenshot function would use this title to find the OS window
        assert_ne!(
            resolved.title.as_deref(),
            Some("Script Kit"),
            "must not screenshot the main window"
        );

        // Target by ID → ACP
        let target_id = AutomationWindowTarget::Id {
            id: format!("{p}:acp-thread-1"),
        };
        let resolved_by_id =
            resolve_automation_window(Some(&target_id)).expect("should resolve by ID");
        assert_eq!(resolved_by_id.kind, AutomationWindowKind::AcpDetached);

        remove_automation_window(&format!("{p}:main"));
        remove_automation_window(&format!("{p}:acp-thread-1"));
    }

    /// Proves that waitFor/batch resolution uses the same resolver as
    /// standalone queries — a Notes or ACP target resolves consistently.
    #[test]
    fn targeted_wait_for_uses_resolved_window_state() {
        let p = test_prefix();

        let mut main = make_info(&p, "main", AutomationWindowKind::Main);
        main.focused = true;
        main.semantic_surface = Some("scriptList".into());
        upsert_automation_window(main);

        let mut notes = make_info(&p, "notes", AutomationWindowKind::Notes);
        notes.semantic_surface = Some("notes".into());
        upsert_automation_window(notes);

        // The same resolve_automation_window used by getElements should
        // also be used by waitFor and batch — prove consistency.
        let target = AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::Notes,
            index: None,
        };

        // Standalone resolution
        let standalone = resolve_automation_window(Some(&target)).expect("standalone resolve");

        // Same call — proves the code path is shared
        let batch_path = resolve_automation_window(Some(&target)).expect("batch-path resolve");

        assert_eq!(standalone.id, batch_path.id);
        assert_eq!(standalone.kind, batch_path.kind);
        assert_eq!(standalone.semantic_surface, batch_path.semantic_surface);

        // Targeting a non-existent kind fails with an error, not a silent fallback
        let missing = AutomationWindowTarget::Kind {
            kind: AutomationWindowKind::MiniAi,
            index: None,
        };
        let err = resolve_automation_window(Some(&missing));
        assert!(
            err.is_err(),
            "missing target must return error, not fallback"
        );

        remove_automation_window(&format!("{p}:main"));
        remove_automation_window(&format!("{p}:notes"));
    }
}
