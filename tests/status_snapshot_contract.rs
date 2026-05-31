use script_kit_gpui::status_snapshot::{
    build_status_snapshot_at, write_status_snapshot_to_path, StatusSnapshotAgentTaskInput,
    StatusSnapshotInput, STATUS_SNAPSHOT_SCHEMA_VERSION,
};

fn input(expose_names: bool) -> StatusSnapshotInput {
    StatusSnapshotInput {
        agent_tasks: vec![StatusSnapshotAgentTaskInput {
            id: "task-1".to_string(),
            name: "Sensitive customer migration".to_string(),
            status: "running".to_string(),
        }],
        expose_agent_task_names: expose_names,
        voice_count_today: 2,
        agent_turn_count_today: 7,
        missing_permissions: vec!["Accessibility".to_string()],
        failed_session_count: 1,
        flagged_log_review_count: 3,
    }
}

#[test]
fn status_snapshot_defaults_redact_agent_task_names_and_context() {
    let snapshot = build_status_snapshot_at(input(false), 1234);

    assert_eq!(snapshot.schema_version, STATUS_SNAPSHOT_SCHEMA_VERSION);
    assert_eq!(snapshot.generated_at_ms, 1234);
    assert!(snapshot.redacted);
    assert!(!snapshot.privacy.agent_task_names_exposed);
    assert!(!snapshot.privacy.focused_app_context_exposed);
    assert!(!snapshot.privacy.memory_snippets_exposed);
    assert_eq!(snapshot.agent_tasks[0].name, None);
    assert!(snapshot.agent_tasks[0].name_redacted);
}

#[test]
fn status_snapshot_explicit_privacy_toggle_exposes_agent_task_names_only() {
    let snapshot = build_status_snapshot_at(input(true), 1234);

    assert!(!snapshot.redacted);
    assert!(snapshot.privacy.agent_task_names_exposed);
    assert!(!snapshot.privacy.focused_app_context_exposed);
    assert!(!snapshot.privacy.memory_snippets_exposed);
    assert_eq!(
        snapshot.agent_tasks[0].name.as_deref(),
        Some("Sensitive customer migration")
    );
    assert!(!snapshot.agent_tasks[0].name_redacted);
}

#[test]
fn status_snapshot_missing_permissions_and_review_items_emit_attention() {
    let snapshot = build_status_snapshot_at(input(false), 1234);
    let codes: Vec<&str> = snapshot
        .attention
        .iter()
        .map(|attention| attention.code.as_str())
        .collect();

    assert!(codes.contains(&"missing_permission"));
    assert!(codes.contains(&"failed_sessions"));
    assert!(codes.contains(&"flagged_log_reviews"));
}

#[test]
fn status_snapshot_writes_json_file_contract() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("status-snapshot.json");
    let snapshot = build_status_snapshot_at(input(false), 1234);

    write_status_snapshot_to_path(&snapshot, &path).expect("write snapshot");

    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read snapshot"))
            .expect("parse snapshot");
    assert_eq!(json["schemaVersion"], STATUS_SNAPSHOT_SCHEMA_VERSION);
    assert_eq!(json["privacy"]["agentTaskNamesExposed"], false);
    assert_eq!(json["agentTasks"][0]["name"], serde_json::Value::Null);
    assert_eq!(json["counters"]["voiceCountToday"], 2);
}
