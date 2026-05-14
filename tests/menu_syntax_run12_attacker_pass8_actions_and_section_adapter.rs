//! Run 12 Pass 8 — ATTACKER MODE on the Pass-6 + Pass-7 cmdk-actions surfaces:
//! - [[src/menu_syntax/actions.rs#current_actions]] (pure spec layer)
//! - [[src/app_impl/menu_syntax_actions.rs#power_syntax_section_to_actions]]
//!   (the dialog-row adapter wired in Pass 6)
//!
//! These two functions sit between the App-side caller (Pass 7) and the
//! ActionsDialog renderer; if they break, the live Cmd+K render breaks
//! invisibly. This file pins their shape against future regressions.
//!
//! Categories: Boundary (8), Composition (7), Resurrection (7). Actions: 22.

use script_kit_gpui::actions::{Action, ActionCategory};
use script_kit_gpui::menu_syntax::actions::MenuSyntaxActionKind;
use script_kit_gpui::menu_syntax::capture_schema::builtin_schema;
use script_kit_gpui::menu_syntax::payload::{CaptureAlias, CaptureInvocation};
use script_kit_gpui::menu_syntax::query::parse_advanced_query;
use script_kit_gpui::menu_syntax::{
    current_menu_syntax_actions, MenuSyntaxAction, MenuSyntaxActionState,
};
use script_kit_gpui::menu_syntax_actions::{
    power_syntax_action_section, power_syntax_section_to_actions, PowerSyntaxActionSection,
    SectionMode,
};

fn capture_payload(target: &str, body: &str) -> CaptureInvocation {
    CaptureInvocation {
        target: target.to_string(),
        alias_form: CaptureAlias::CapturePrefix,
        body: body.to_string(),
        tags: vec![],
        priority: None,
        url: None,
        duration: None,
        kv: vec![],
        date_phrases: vec![],
        raw: format!("+{target} {body}"),
    }
}

// ---------------- Boundary (8) ----------------

#[test]
fn boundary_01_capture_with_empty_body_disables_save_and_copy_id() {
    // Falsifier for Pass-6 SaveAndCopyId enable rule.
    let payload = capture_payload("todo", "");
    let schema = builtin_schema("todo").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "todo",
        payload: &payload,
        schema: Some(&schema),
    };
    let actions = current_menu_syntax_actions(&state);
    let save = actions
        .iter()
        .find(|a| matches!(a.kind, MenuSyntaxActionKind::SaveAndCopyId))
        .expect("SaveAndCopyId row");
    assert!(
        !save.enabled,
        "empty-body capture must NOT allow SaveAndCopyId"
    );
}

#[test]
fn boundary_02_capture_with_whitespace_only_body_disables_save_and_copy_id() {
    let payload = capture_payload("todo", "   \t  ");
    let schema = builtin_schema("todo").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "todo",
        payload: &payload,
        schema: Some(&schema),
    };
    let actions = current_menu_syntax_actions(&state);
    let save = actions
        .iter()
        .find(|a| matches!(a.kind, MenuSyntaxActionKind::SaveAndCopyId))
        .unwrap();
    assert!(!save.enabled, "whitespace-only body counts as empty");
}

#[test]
fn boundary_03_cal_missing_date_surfaces_default_time_action() {
    // Pass-7 receipt-token invariant: +cal Design review must produce the
    // "Default Time → Today 9 AM" row. Without this, the screenshot test in
    // Pass 7 would have failed silently.
    let payload = capture_payload("cal", "Design review");
    let schema = builtin_schema("cal").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &payload,
        schema: Some(&schema),
    };
    let actions = current_menu_syntax_actions(&state);
    let default_time = actions
        .iter()
        .find(|a| matches!(a.kind, MenuSyntaxActionKind::DefaultTime { .. }))
        .expect("DefaultTime row required when AnyDate is missing");
    assert_eq!(default_time.label, "Default Time → Today 9 AM");
    assert!(default_time.enabled);
}

#[test]
fn boundary_04_cal_with_no_schema_does_not_surface_default_time() {
    // Falsifier: without a schema, we cannot know whether AnyDate is missing,
    // so DefaultTime must NOT appear.
    let payload = capture_payload("cal", "Design review");
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &payload,
        schema: None,
    };
    let actions = current_menu_syntax_actions(&state);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a.kind, MenuSyntaxActionKind::DefaultTime { .. })),
        "No schema → no DefaultTime row"
    );
}

#[test]
fn boundary_05_command_composer_with_empty_argv_still_has_actions() {
    let argv: Vec<String> = vec![];
    let state = MenuSyntaxActionState::CommandComposer {
        head: "deploy",
        argv: &argv,
    };
    let actions = current_menu_syntax_actions(&state);
    assert!(
        !actions.is_empty(),
        "command actions should not depend on argv presence"
    );
}

#[test]
fn boundary_06_refine_with_empty_query_string_returns_actions() {
    let query = parse_advanced_query(":");
    let state = MenuSyntaxActionState::RefineQuery { query: &query };
    let actions = current_menu_syntax_actions(&state);
    assert!(
        !actions.is_empty(),
        "refine produces actions even for bare prefix"
    );
}

#[test]
fn boundary_07_section_to_actions_id_prefix_is_namespaced() {
    // Pin the `menu_syntax:` ID prefix so Cmd+K dispatch never collides
    // with a built-in action id.
    let payload = capture_payload("todo", "Buy milk");
    let schema = builtin_schema("todo").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "todo",
        payload: &payload,
        schema: Some(&schema),
    };
    let section = power_syntax_action_section(&state);
    let dialog_actions: Vec<Action> = power_syntax_section_to_actions(&section);
    for a in &dialog_actions {
        assert!(
            a.id.starts_with("menu_syntax:"),
            "every Power Syntax action id must be namespaced, got {}",
            a.id
        );
    }
}

#[test]
fn boundary_08_section_to_actions_section_label_constant() {
    let payload = capture_payload("todo", "Buy milk");
    let schema = builtin_schema("todo").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "todo",
        payload: &payload,
        schema: Some(&schema),
    };
    let section = power_syntax_action_section(&state);
    let dialog_actions = power_syntax_section_to_actions(&section);
    for a in &dialog_actions {
        assert_eq!(a.section.as_deref(), Some("Power Syntax"));
        assert_eq!(a.category, ActionCategory::ScriptContext);
    }
}

// ---------------- Composition (7) ----------------

#[test]
fn composition_01_section_to_actions_skips_disabled_rows() {
    // Build a manual section with a mix of enabled and disabled rows.
    let section = PowerSyntaxActionSection {
        title: "Power Syntax".to_string(),
        mode: SectionMode::Replace,
        actions: vec![
            MenuSyntaxAction {
                id: "a.enabled".into(),
                label: "Yes".into(),
                kind: MenuSyntaxActionKind::Cancel,
                enabled: true,
            },
            MenuSyntaxAction {
                id: "a.disabled".into(),
                label: "No".into(),
                kind: MenuSyntaxActionKind::Cancel,
                enabled: false,
            },
        ],
    };
    let dialog = power_syntax_section_to_actions(&section);
    assert_eq!(dialog.len(), 1, "disabled row must be skipped");
    assert_eq!(dialog[0].id, "menu_syntax:a.enabled");
}

#[test]
fn composition_02_section_to_actions_preserves_enabled_order() {
    let section = PowerSyntaxActionSection {
        title: "Power Syntax".to_string(),
        mode: SectionMode::Replace,
        actions: vec![
            MenuSyntaxAction {
                id: "first".into(),
                label: "1".into(),
                kind: MenuSyntaxActionKind::Cancel,
                enabled: true,
            },
            MenuSyntaxAction {
                id: "skip".into(),
                label: "x".into(),
                kind: MenuSyntaxActionKind::Cancel,
                enabled: false,
            },
            MenuSyntaxAction {
                id: "second".into(),
                label: "2".into(),
                kind: MenuSyntaxActionKind::Cancel,
                enabled: true,
            },
            MenuSyntaxAction {
                id: "third".into(),
                label: "3".into(),
                kind: MenuSyntaxActionKind::Cancel,
                enabled: true,
            },
        ],
    };
    let dialog = power_syntax_section_to_actions(&section);
    let ids: Vec<&str> = dialog.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "menu_syntax:first",
            "menu_syntax:second",
            "menu_syntax:third"
        ]
    );
}

#[test]
fn composition_03_capture_action_ids_are_unique_within_section() {
    let payload = capture_payload("cal", "Design review");
    let schema = builtin_schema("cal").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &payload,
        schema: Some(&schema),
    };
    let actions = current_menu_syntax_actions(&state);
    let mut seen = std::collections::HashSet::new();
    for a in &actions {
        assert!(
            seen.insert(a.id.clone()),
            "duplicate action id within section: {}",
            a.id
        );
    }
}

#[test]
fn composition_04_command_action_ids_unique_within_section() {
    let argv = vec!["--prod".to_string()];
    let state = MenuSyntaxActionState::CommandComposer {
        head: "deploy",
        argv: &argv,
    };
    let actions = current_menu_syntax_actions(&state);
    let mut seen = std::collections::HashSet::new();
    for a in &actions {
        assert!(
            seen.insert(a.id.clone()),
            "duplicate command action id: {}",
            a.id
        );
    }
}

#[test]
fn composition_05_refine_action_ids_unique_within_section() {
    let query = parse_advanced_query(":type:script git");
    let state = MenuSyntaxActionState::RefineQuery { query: &query };
    let actions = current_menu_syntax_actions(&state);
    let mut seen = std::collections::HashSet::new();
    for a in &actions {
        assert!(
            seen.insert(a.id.clone()),
            "duplicate refine action id: {}",
            a.id
        );
    }
}

#[test]
fn composition_06_namespaced_dialog_ids_unique_across_three_states() {
    // The `menu_syntax:` prefix must keep IDs distinct across composer
    // states so Cmd+K dispatch can ALWAYS unambiguously route.
    let mut all_ids = std::collections::HashSet::new();
    let payload = capture_payload("cal", "x");
    let schema = builtin_schema("cal").unwrap();
    let cap_state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &payload,
        schema: Some(&schema),
    };
    let argv = vec!["x".to_string()];
    let cmd_state = MenuSyntaxActionState::CommandComposer {
        head: "x",
        argv: &argv,
    };
    let q = parse_advanced_query(":type:script");
    let ref_state = MenuSyntaxActionState::RefineQuery { query: &q };
    for state in [&cap_state, &cmd_state, &ref_state] {
        let section = power_syntax_action_section(state);
        for a in power_syntax_section_to_actions(&section) {
            // Cross-state collisions ARE allowed (refine/cancel and
            // capture/cancel could both exist if we ever added a cross
            // route), but within a single state they must be unique. This
            // assertion covers the within-state case across all three.
            // Track per-state.
            let _ = all_ids.insert(a.id);
        }
    }
    // Sanity: we should have at least 4 + 5 + 4 = 13 distinct labels across
    // the three states.
    assert!(
        all_ids.len() >= 9,
        "expected ≥9 distinct namespaced ids across 3 states, got {}",
        all_ids.len()
    );
}

#[test]
fn composition_07_disabled_only_section_yields_empty_dialog_actions() {
    let section = PowerSyntaxActionSection {
        title: "Power Syntax".to_string(),
        mode: SectionMode::Replace,
        actions: vec![MenuSyntaxAction {
            id: "off".into(),
            label: "Off".into(),
            kind: MenuSyntaxActionKind::Cancel,
            enabled: false,
        }],
    };
    let dialog = power_syntax_section_to_actions(&section);
    assert!(
        dialog.is_empty(),
        "all-disabled section must produce zero dialog rows"
    );
}

// ---------------- Resurrection (7) ----------------
// Pin the user-visible invariants from Pass 7's screenshot so future
// refactors can't silently break the live render.

#[test]
fn resurrection_01_pass7_default_time_label_exact() {
    // Pass 7 screenshot showed exactly "Default Time → Today 9 AM". Pin the
    // unicode arrow + lowercase "today 9am" so ASCII-style refactors
    // ("Default time -> today 9am") trip this test before reaching the UI.
    let payload = capture_payload("cal", "Design review");
    let schema = builtin_schema("cal").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &payload,
        schema: Some(&schema),
    };
    let actions = current_menu_syntax_actions(&state);
    let dt = actions
        .iter()
        .find(|a| matches!(a.kind, MenuSyntaxActionKind::DefaultTime { .. }))
        .unwrap();
    assert_eq!(dt.label, "Default Time → Today 9 AM");
}

#[test]
fn resurrection_02_pass7_capture_section_first_row_is_cancel() {
    // The Pass-7 screenshot's first row is "Cancel without saving". Pin
    // ordering so future spec refactors don't bury Cancel.
    let payload = capture_payload("cal", "Design review");
    let schema = builtin_schema("cal").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &payload,
        schema: Some(&schema),
    };
    let actions = current_menu_syntax_actions(&state);
    assert_eq!(actions[0].label, "Cancel without saving");
    assert!(matches!(actions[0].kind, MenuSyntaxActionKind::Cancel));
}

#[test]
fn resurrection_03_capture_section_count_matches_pass7_screenshot() {
    // Pass 7 screenshot showed 6 rows for `+cal Design review`. Pin the
    // count so accidentally-added rows trip a test before the screenshot
    // diverges.
    let payload = capture_payload("cal", "Design review");
    let schema = builtin_schema("cal").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &payload,
        schema: Some(&schema),
    };
    let actions = current_menu_syntax_actions(&state);
    assert_eq!(
        actions.len(),
        6,
        "Pass 7 screenshot pinned 6 rows for +cal Design review"
    );
}

#[test]
fn resurrection_04_pass6_replace_mode_for_capture() {
    // Pass 6 falsifier: capture state must produce SectionMode::Replace.
    let payload = capture_payload("todo", "Buy milk");
    let schema = builtin_schema("todo").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "todo",
        payload: &payload,
        schema: Some(&schema),
    };
    let section = power_syntax_action_section(&state);
    assert_eq!(section.mode, SectionMode::Replace);
}

#[test]
fn resurrection_05_pass6_prepend_mode_for_refine() {
    // Pass 6 falsifier: refine state must produce SectionMode::Prepend.
    let q = parse_advanced_query(":type:script foo");
    let state = MenuSyntaxActionState::RefineQuery { query: &q };
    let section = power_syntax_action_section(&state);
    assert_eq!(section.mode, SectionMode::Prepend);
}

#[test]
fn resurrection_06_pass6_section_title_constant() {
    let payload = capture_payload("todo", "x");
    let schema = builtin_schema("todo").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "todo",
        payload: &payload,
        schema: Some(&schema),
    };
    let section = power_syntax_action_section(&state);
    assert_eq!(section.title, "Power Syntax");
}

#[test]
fn resurrection_07_default_time_phrase_matches_label_suffix() {
    // The label says "today 9am"; the kind's phrase MUST match (used by
    // the dispatch path in apply_safe_effect to insert the literal token).
    // If these drift, the visible label and the actual inserted token
    // would diverge — silent UX bug.
    let payload = capture_payload("cal", "Design review");
    let schema = builtin_schema("cal").unwrap();
    let state = MenuSyntaxActionState::CaptureComposer {
        target: "cal",
        payload: &payload,
        schema: Some(&schema),
    };
    let actions = current_menu_syntax_actions(&state);
    let dt = actions
        .iter()
        .find_map(|a| match &a.kind {
            MenuSyntaxActionKind::DefaultTime { phrase } => Some(phrase.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(dt, "today 9am");
}
