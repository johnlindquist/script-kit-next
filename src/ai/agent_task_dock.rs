//! Pure Agent Chat task-dock state.
//!
//! The UI layer should stay compact and hidden unless this model reports
//! attention-worthy work. This module does not own persistence or rendering.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTaskDockState {
    pub tasks: Vec<AgentTaskDockTask>,
    pub selected_task_id: Option<String>,
}

impl AgentTaskDockState {
    pub fn new(tasks: Vec<AgentTaskDockTask>) -> Self {
        let selected_task_id = tasks.first().map(|task| task.id.clone());
        Self {
            tasks,
            selected_task_id,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.tasks
            .iter()
            .any(|task| task.status.is_attention_worthy())
    }

    pub fn select_task(&mut self, task_id: &str) -> AgentTaskDockSelection {
        if self.tasks.iter().any(|task| task.id == task_id) {
            self.selected_task_id = Some(task_id.to_string());
            AgentTaskDockSelection::Selected
        } else {
            AgentTaskDockSelection::MissingTask
        }
    }

    pub fn queue_or_submit_follow_up(
        &mut self,
        task_id: &str,
        prompt: &str,
    ) -> AgentTaskDockSubmitDecision {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return AgentTaskDockSubmitDecision::RejectedEmptyPrompt;
        }

        let Some(task) = self.tasks.iter_mut().find(|task| task.id == task_id) else {
            return AgentTaskDockSubmitDecision::RejectedMissingTask;
        };

        match task.status {
            AgentTaskDockStatus::Running => {
                task.queued_follow_ups.push(prompt.to_string());
                task.status = AgentTaskDockStatus::Running;
                AgentTaskDockSubmitDecision::Queued
            }
            AgentTaskDockStatus::Archived => AgentTaskDockSubmitDecision::RejectedArchived,
            AgentTaskDockStatus::Queued => AgentTaskDockSubmitDecision::Queued,
            AgentTaskDockStatus::Ready
            | AgentTaskDockStatus::Failed
            | AgentTaskDockStatus::Resumable
            | AgentTaskDockStatus::Completed => AgentTaskDockSubmitDecision::SubmitNow,
        }
    }

    pub fn archive_task(&mut self, task_id: &str) -> AgentTaskDockArchiveDecision {
        let Some(task) = self.tasks.iter_mut().find(|task| task.id == task_id) else {
            return AgentTaskDockArchiveDecision::MissingTask;
        };

        if task.status.is_running_like() {
            return AgentTaskDockArchiveDecision::RejectedRunning;
        }

        task.status = AgentTaskDockStatus::Archived;
        AgentTaskDockArchiveDecision::Archived
    }

    pub fn resume_task(&mut self, task_id: &str) -> AgentTaskDockResumeDecision {
        let Some(task) = self.tasks.iter_mut().find(|task| task.id == task_id) else {
            return AgentTaskDockResumeDecision::MissingTask;
        };

        if task.status != AgentTaskDockStatus::Resumable {
            return AgentTaskDockResumeDecision::RejectedNotResumable;
        }

        task.status = AgentTaskDockStatus::Ready;
        AgentTaskDockResumeDecision::ReadyToResume
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTaskDockTask {
    pub id: String,
    pub title: String,
    pub surface: AgentTaskDockSurface,
    pub status: AgentTaskDockStatus,
    pub queued_follow_ups: Vec<String>,
}

impl AgentTaskDockTask {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        surface: AgentTaskDockSurface,
        status: AgentTaskDockStatus,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            surface,
            status,
            queued_follow_ups: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentTaskDockSurface {
    Embedded { semantic_id: String },
    Detached { semantic_id: String },
}

impl AgentTaskDockSurface {
    pub fn semantic_id(&self) -> &str {
        match self {
            Self::Embedded { semantic_id } | Self::Detached { semantic_id } => semantic_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskDockStatus {
    Ready,
    Running,
    Queued,
    Failed,
    Resumable,
    Completed,
    Archived,
}

impl AgentTaskDockStatus {
    fn is_running_like(self) -> bool {
        matches!(self, Self::Running | Self::Queued)
    }

    fn is_attention_worthy(self) -> bool {
        matches!(
            self,
            Self::Running | Self::Queued | Self::Failed | Self::Resumable | Self::Archived
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskDockSelection {
    Selected,
    MissingTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskDockSubmitDecision {
    SubmitNow,
    Queued,
    RejectedArchived,
    RejectedEmptyPrompt,
    RejectedMissingTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskDockArchiveDecision {
    Archived,
    RejectedRunning,
    MissingTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskDockResumeDecision {
    ReadyToResume,
    RejectedNotResumable,
    MissingTask,
}
