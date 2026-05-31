use script_kit_gpui::ai::{
    AgentTaskDockArchiveDecision, AgentTaskDockResumeDecision, AgentTaskDockSelection,
    AgentTaskDockState, AgentTaskDockStatus, AgentTaskDockSubmitDecision, AgentTaskDockSurface,
    AgentTaskDockTask,
};

fn embedded_task(id: &str, status: AgentTaskDockStatus) -> AgentTaskDockTask {
    AgentTaskDockTask::new(
        id,
        format!("Task {id}"),
        AgentTaskDockSurface::Embedded {
            semantic_id: format!("acp:embedded:{id}"),
        },
        status,
    )
}

#[test]
fn task_dock_hidden_when_only_completed_or_ready_work_exists() {
    let state = AgentTaskDockState::new(vec![
        embedded_task("ready", AgentTaskDockStatus::Ready),
        embedded_task("done", AgentTaskDockStatus::Completed),
    ]);

    assert!(!state.is_visible());
}

#[test]
fn task_dock_visible_for_active_attention_and_archived_work() {
    for status in [
        AgentTaskDockStatus::Running,
        AgentTaskDockStatus::Queued,
        AgentTaskDockStatus::Failed,
        AgentTaskDockStatus::Resumable,
        AgentTaskDockStatus::Archived,
    ] {
        let state = AgentTaskDockState::new(vec![embedded_task("task", status)]);
        assert!(state.is_visible(), "status {status:?} should show the dock");
    }
}

#[test]
fn task_dock_selects_only_existing_tasks() {
    let mut state = AgentTaskDockState::new(vec![embedded_task("one", AgentTaskDockStatus::Ready)]);

    assert_eq!(state.select_task("one"), AgentTaskDockSelection::Selected);
    assert_eq!(state.selected_task_id.as_deref(), Some("one"));
    assert_eq!(
        state.select_task("missing"),
        AgentTaskDockSelection::MissingTask
    );
    assert_eq!(state.selected_task_id.as_deref(), Some("one"));
}

#[test]
fn task_dock_queues_follow_up_when_selected_task_is_running() {
    let mut state =
        AgentTaskDockState::new(vec![embedded_task("one", AgentTaskDockStatus::Running)]);

    assert_eq!(
        state.queue_or_submit_follow_up("one", "   continue with tests   "),
        AgentTaskDockSubmitDecision::Queued
    );
    assert_eq!(
        state.tasks[0].queued_follow_ups,
        vec!["continue with tests"]
    );
}

#[test]
fn task_dock_submits_immediately_when_task_is_not_running() {
    let mut state = AgentTaskDockState::new(vec![embedded_task("one", AgentTaskDockStatus::Ready)]);

    assert_eq!(
        state.queue_or_submit_follow_up("one", "continue"),
        AgentTaskDockSubmitDecision::SubmitNow
    );
    assert!(state.tasks[0].queued_follow_ups.is_empty());
}

#[test]
fn task_dock_archive_guard_rejects_running_or_queued_tasks() {
    let mut state = AgentTaskDockState::new(vec![
        embedded_task("running", AgentTaskDockStatus::Running),
        embedded_task("queued", AgentTaskDockStatus::Queued),
        embedded_task("failed", AgentTaskDockStatus::Failed),
    ]);

    assert_eq!(
        state.archive_task("running"),
        AgentTaskDockArchiveDecision::RejectedRunning
    );
    assert_eq!(
        state.archive_task("queued"),
        AgentTaskDockArchiveDecision::RejectedRunning
    );
    assert_eq!(
        state.archive_task("failed"),
        AgentTaskDockArchiveDecision::Archived
    );
}

#[test]
fn task_dock_resume_guard_only_allows_resumable_tasks() {
    let mut state = AgentTaskDockState::new(vec![
        embedded_task("ready", AgentTaskDockStatus::Ready),
        embedded_task("resume", AgentTaskDockStatus::Resumable),
    ]);

    assert_eq!(
        state.resume_task("ready"),
        AgentTaskDockResumeDecision::RejectedNotResumable
    );
    assert_eq!(
        state.resume_task("resume"),
        AgentTaskDockResumeDecision::ReadyToResume
    );
    assert_eq!(state.tasks[1].status, AgentTaskDockStatus::Ready);
}

#[test]
fn task_dock_preserves_embedded_and_detached_semantic_ids() {
    let embedded = AgentTaskDockSurface::Embedded {
        semantic_id: "acp:embedded:main".to_string(),
    };
    let detached = AgentTaskDockSurface::Detached {
        semantic_id: "acp:detached:window-1".to_string(),
    };

    assert_eq!(embedded.semantic_id(), "acp:embedded:main");
    assert_eq!(detached.semantic_id(), "acp:detached:window-1");
}
