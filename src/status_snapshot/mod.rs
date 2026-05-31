//! Privacy-gated local status snapshot.
//!
//! The snapshot is a compact JSON file contract for diagnostics and future
//! menu/tray surfaces. It redacts task names and captions by default.

use std::path::{Path, PathBuf};

pub const STATUS_SNAPSHOT_SCHEMA_VERSION: u32 = 1;
pub const STATUS_SNAPSHOT_FILE_NAME: &str = "status-snapshot.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSnapshotInput {
    pub agent_tasks: Vec<StatusSnapshotAgentTaskInput>,
    pub expose_agent_task_names: bool,
    pub voice_count_today: u64,
    pub agent_turn_count_today: u64,
    pub missing_permissions: Vec<String>,
    pub failed_session_count: u64,
    pub flagged_log_review_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSnapshotAgentTaskInput {
    pub id: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSnapshot {
    pub schema_version: u32,
    pub generated_at_ms: u128,
    pub privacy: StatusSnapshotPrivacy,
    pub agent_tasks: Vec<StatusSnapshotAgentTask>,
    pub counters: StatusSnapshotCounters,
    pub attention: Vec<StatusSnapshotAttention>,
    pub redacted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSnapshotPrivacy {
    pub agent_task_names_exposed: bool,
    pub focused_app_context_exposed: bool,
    pub memory_snippets_exposed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSnapshotAgentTask {
    pub id: String,
    pub name: Option<String>,
    pub name_redacted: bool,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSnapshotCounters {
    pub voice_count_today: u64,
    pub agent_turn_count_today: u64,
    pub failed_session_count: u64,
    pub flagged_log_review_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSnapshotAttention {
    pub code: String,
    pub message: String,
}

pub fn status_snapshot_path() -> PathBuf {
    crate::setup::get_kit_path().join(STATUS_SNAPSHOT_FILE_NAME)
}

pub fn build_status_snapshot(input: StatusSnapshotInput) -> StatusSnapshot {
    build_status_snapshot_at(input, now_epoch_ms())
}

pub fn build_status_snapshot_at(
    input: StatusSnapshotInput,
    generated_at_ms: u128,
) -> StatusSnapshot {
    let agent_task_names_exposed = input.expose_agent_task_names;
    let agent_tasks = input
        .agent_tasks
        .into_iter()
        .map(|task| StatusSnapshotAgentTask {
            id: task.id,
            name: agent_task_names_exposed.then_some(task.name),
            name_redacted: !agent_task_names_exposed,
            status: task.status,
        })
        .collect();

    let mut attention = Vec::new();
    for permission in input.missing_permissions {
        attention.push(StatusSnapshotAttention {
            code: "missing_permission".to_string(),
            message: format!("Missing permission: {permission}"),
        });
    }
    if input.failed_session_count > 0 {
        attention.push(StatusSnapshotAttention {
            code: "failed_sessions".to_string(),
            message: format!("{} failed session(s)", input.failed_session_count),
        });
    }
    if input.flagged_log_review_count > 0 {
        attention.push(StatusSnapshotAttention {
            code: "flagged_log_reviews".to_string(),
            message: format!(
                "{} flagged log review item(s)",
                input.flagged_log_review_count
            ),
        });
    }

    StatusSnapshot {
        schema_version: STATUS_SNAPSHOT_SCHEMA_VERSION,
        generated_at_ms,
        privacy: StatusSnapshotPrivacy {
            agent_task_names_exposed,
            focused_app_context_exposed: false,
            memory_snippets_exposed: false,
        },
        agent_tasks,
        counters: StatusSnapshotCounters {
            voice_count_today: input.voice_count_today,
            agent_turn_count_today: input.agent_turn_count_today,
            failed_session_count: input.failed_session_count,
            flagged_log_review_count: input.flagged_log_review_count,
        },
        attention,
        redacted: !agent_task_names_exposed,
    }
}

pub fn write_status_snapshot_to_path(
    snapshot: &StatusSnapshot,
    path: &Path,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    std::fs::write(path, json)
}

pub fn write_status_snapshot(snapshot: &StatusSnapshot) -> std::io::Result<PathBuf> {
    let path = status_snapshot_path();
    write_status_snapshot_to_path(snapshot, &path)?;
    Ok(path)
}

fn now_epoch_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
