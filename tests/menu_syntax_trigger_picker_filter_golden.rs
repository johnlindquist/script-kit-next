use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use script_kit_gpui::menu_syntax::trigger_picker::{
    build_trigger_picker_snapshot, TriggerPickerAction, TriggerPickerContext, TriggerPickerMode,
    TriggerPickerRow, TriggerPickerRowKind, TriggerPickerSnapshot,
};
use script_kit_gpui::metadata_parser::TypedMetadata;
use script_kit_gpui::scripts::Script;

fn snapshot(input: &str, ctx: &TriggerPickerContext) -> TriggerPickerSnapshot {
    build_trigger_picker_snapshot(input, ctx).expect("trigger picker snapshot")
}

fn target_rows(snapshot: &TriggerPickerSnapshot) -> Vec<&TriggerPickerRow> {
    snapshot
        .rows
        .iter()
        .filter(|row| row.kind == TriggerPickerRowKind::CaptureTarget)
        .collect()
}

fn target_tokens(snapshot: &TriggerPickerSnapshot) -> Vec<&str> {
    target_rows(snapshot)
        .into_iter()
        .filter_map(|row| row.token.as_deref())
        .collect()
}

fn target_titles(snapshot: &TriggerPickerSnapshot) -> Vec<&str> {
    target_rows(snapshot)
        .into_iter()
        .map(|row| row.title.as_str())
        .collect()
}

fn create_handler_footer(snapshot: &TriggerPickerSnapshot) -> &TriggerPickerRow {
    snapshot
        .rows
        .iter()
        .find(|row| row.id == "footer:create-handler")
        .expect("create handler footer")
}

fn script_with_menu_syntax(name: &str, menu_syntax_json: &str) -> Arc<Script> {
    let mut extra: HashMap<String, serde_json::Value> = HashMap::new();
    extra.insert(
        "menuSyntax".to_string(),
        serde_json::from_str(menu_syntax_json).expect("valid menuSyntax JSON"),
    );
    let meta = TypedMetadata {
        extra,
        ..Default::default()
    };
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(format!("/tmp/{}.ts", name.to_lowercase().replace(' ', "-"))),
        extension: "ts".to_string(),
        description: Some(format!("{name} description")),
        typed_metadata: Some(meta),
        plugin_id: "custom".to_string(),
        ..Default::default()
    })
}

#[test]
fn bare_semicolon_lists_all_targets_original_order() {
    let ctx = TriggerPickerContext::default();
    let snap = snapshot(";", &ctx);

    assert_eq!(snap.mode, TriggerPickerMode::Capture);
    assert!(snap.target.is_none());
    assert_eq!(
        target_tokens(&snap),
        vec![";todo", ";cal", ";note", ";social", ";link"]
    );
    assert_eq!(
        target_titles(&snap),
        vec![
            "Todo inbox",
            "Calendar event",
            "Daily note",
            "Social draft",
            "Tagged link",
        ]
    );
    assert_eq!(
        create_handler_footer(&snap).title,
        "Create capture handler…"
    );
}

#[test]
fn exact_todo_focuses_target() {
    let ctx = TriggerPickerContext::default();
    let snap = snapshot(";todo", &ctx);
    let rows = target_rows(&snap);

    assert_eq!(snap.target.as_deref(), Some("todo"));
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].token.as_deref(), Some(";todo"));
    assert_eq!(rows[0].title, "Todo inbox");
}

#[test]
fn partial_dai_ranks_daily_note_first() {
    let ctx = TriggerPickerContext::default();
    let snap = snapshot(";dai", &ctx);
    let rows = target_rows(&snap);

    assert_eq!(snap.target, None);
    assert_eq!(rows[0].title, "Daily note");
    assert_eq!(rows[0].token.as_deref(), Some(";note"));
}

#[test]
fn partial_daily_ranks_daily_note_first() {
    let ctx = TriggerPickerContext::default();
    let snap = snapshot(";daily", &ctx);
    let rows = target_rows(&snap);

    assert_eq!(rows[0].title, "Daily note");
    assert_eq!(rows[0].token.as_deref(), Some(";note"));
}

#[test]
fn partial_cal_ranks_calendar_event_first() {
    let ctx = TriggerPickerContext::default();
    let snap = snapshot(";cal", &ctx);
    let rows = target_rows(&snap);

    assert_eq!(rows[0].title, "Calendar event");
    assert_eq!(rows[0].token.as_deref(), Some(";cal"));
}

#[test]
fn unknown_xyz_shows_empty_list_with_create_handler_footer() {
    let ctx = TriggerPickerContext::default();
    let snap = snapshot(";xyz", &ctx);
    let footer = create_handler_footer(&snap);

    assert_eq!(snap.target, None);
    assert!(
        target_rows(&snap).is_empty(),
        "unknown slug should not show unrelated capture targets"
    );
    assert!(footer.title.contains("Create capture handler for ;xyz…"));
    assert_eq!(
        footer.action,
        TriggerPickerAction::CreateHandler {
            target: Some("xyz".to_string())
        }
    );
}

#[test]
fn daily_does_not_leave_todo_first() {
    let ctx = TriggerPickerContext::default();
    let snap = snapshot(";daily", &ctx);
    let rows = target_rows(&snap);

    assert_ne!(rows[0].token.as_deref(), Some(";todo"));
    assert_eq!(rows[0].title, "Daily note");
}

#[test]
fn dynamic_target_label_participates_in_filter() {
    let github = script_with_menu_syntax(
        "Capture GitHub Issue",
        r#"[{ "family": "capture.v1", "targets": ["github"], "label": "GitHub issue" }]"#,
    );
    let ctx = TriggerPickerContext {
        scripts: vec![github],
        ..Default::default()
    };
    let snap = snapshot(";issue", &ctx);
    let rows = target_rows(&snap);

    assert_eq!(rows[0].title, "GitHub issue");
    assert_eq!(rows[0].token.as_deref(), Some(";github"));
}

fn mcal_context() -> TriggerPickerContext {
    let mcal = script_with_menu_syntax(
        "Add event to macOS Calendar",
        r#"[{ "family": "capture.v1", "targets": ["mcal"], "label": "Add event to macOS Calendar" }]"#,
    );
    TriggerPickerContext {
        scripts: vec![mcal],
        ..Default::default()
    }
}

#[test]
fn exact_dynamic_slug_uses_metadata_label() {
    let ctx = mcal_context();
    let snap = snapshot(";mcal", &ctx);
    let rows = target_rows(&snap);

    assert_eq!(snap.mode, TriggerPickerMode::Capture);
    assert_eq!(snap.target.as_deref(), Some("mcal"));
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].token.as_deref(), Some(";mcal"));
    assert_eq!(rows[0].title, "Add event to macOS Calendar");
    assert_eq!(rows[0].detail.as_deref(), Some("Registered capture target"));
}

#[test]
fn exact_dynamic_slug_focused_path_matches_filter_path_label() {
    let ctx = mcal_context();
    let filtered = snapshot(";mca", &ctx);
    let focused = snapshot(";mcal", &ctx);

    let filtered_row = target_rows(&filtered)
        .into_iter()
        .find(|row| row.token.as_deref() == Some(";mcal"))
        .expect("filtered mcal row");
    let focused_rows = target_rows(&focused);

    assert_eq!(focused_rows.len(), 1);
    assert_eq!(filtered_row.title.as_str(), focused_rows[0].title.as_str());
    assert_eq!(filtered_row.title, "Add event to macOS Calendar");
}

#[test]
fn exact_unknown_slug_still_offers_create_handler_footer() {
    let ctx = TriggerPickerContext::default();
    let snap = snapshot(";xyz123", &ctx);
    let rows = target_rows(&snap);
    let footer = create_handler_footer(&snap);

    assert_eq!(snap.mode, TriggerPickerMode::Capture);
    assert!(snap.target.is_none());
    assert!(
        !rows.iter().any(|row| {
            row.title == "Capture target" && row.detail.as_deref() == Some("Unknown target")
        }),
        "unknown filter text should not become a focused Unknown target row"
    );
    assert!(footer.title.contains("Create capture handler for ;xyz123…"));
    assert_eq!(
        footer.action,
        TriggerPickerAction::CreateHandler {
            target: Some("xyz123".to_string())
        }
    );
}

#[test]
fn unknown_gcal_with_trailing_space_does_not_relist_all_targets() {
    let ctx = mcal_context();
    let snap = snapshot(";gcal ", &ctx);
    let footer = create_handler_footer(&snap);

    assert_eq!(snap.target, None);
    assert!(
        target_rows(&snap).is_empty(),
        "trailing space after unknown ;gcal must not fall back to the full target list"
    );
    assert!(footer.title.contains("Create capture handler for ;gcal…"));
}

#[test]
fn unknown_gcal_with_body_text_does_not_relist_all_targets() {
    let ctx = mcal_context();
    let snap = snapshot(";gcal Lunch w/ Mindy", &ctx);
    let footer = create_handler_footer(&snap);

    assert_eq!(snap.target, None);
    assert!(target_rows(&snap).is_empty());
    assert!(footer.title.contains("Create capture handler for ;gcal…"));
}

#[test]
fn unknown_gcal_with_dynamic_targets_still_shows_empty_list() {
    let ctx = mcal_context();
    let snap = snapshot(";gcal", &ctx);
    let footer = create_handler_footer(&snap);

    assert_eq!(snap.target, None);
    assert!(
        target_rows(&snap).is_empty(),
        "no fuzzy match for ';gcal' should leave the picker empty rather than show unrelated targets"
    );
    assert!(footer.title.contains("Create capture handler for ;gcal…"));
    assert_eq!(
        footer.action,
        TriggerPickerAction::CreateHandler {
            target: Some("gcal".to_string())
        }
    );
}

#[test]
fn filtered_match_keeps_generic_footer_until_exact_target_is_locked() {
    let ctx = mcal_context();

    let filtered = snapshot(";mca", &ctx);
    let filtered_footer = create_handler_footer(&filtered);
    assert_eq!(filtered.target, None);
    assert_eq!(filtered_footer.title, "Create capture handler…");
    assert_eq!(
        filtered_footer.action,
        TriggerPickerAction::CreateHandler { target: None }
    );

    let focused = snapshot(";mcal", &ctx);
    let focused_footer = create_handler_footer(&focused);
    assert_eq!(focused.target.as_deref(), Some("mcal"));
    assert!(focused_footer
        .title
        .contains("Create capture handler for ;mcal…"));
    assert_eq!(
        focused_footer.action,
        TriggerPickerAction::CreateHandler {
            target: Some("mcal".to_string())
        }
    );
}
