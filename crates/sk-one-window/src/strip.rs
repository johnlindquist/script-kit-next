//! Agents work in the strip above the capsules while you keep going: a task starting or moving
//! along never changes the list, Esc parks an agent's view into the strip, and ⌘J opens whatever
//! needs you first, then the newest task.

use crate::contract::words;
use crate::enter::EnterTarget;
use crate::row::Row;
use crate::window::ViewSpec;

pub const AGENT_VIEW: &str = "agent";
pub const AGENT_LABEL: &str = "Agent";
/// The one row the list shows once ⇥ made the agent the Owner; Enter starts the agent.
pub const OWNER_ROW: &str = "owner:agent";

pub fn owner_row(sentence: &str) -> Row {
    Row::new(OWNER_ROW, words::HAND_TO_AGENT)
        .with_sub(format!("{AGENT_LABEL} · {}", sentence.trim()))
        .with_icon("delegate")
        .with_head(words::AGENT)
        .with_enter_label(words::START_AGENT)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Working,
    /// It asked something; ⌘J opens the question.
    NeedsYou,
    /// Finished and waiting for you to look.
    Ready,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub id: String,
    /// "Agent · fix SK-431".
    pub label: String,
    /// "Reading the issue · 1 of 4", "Needs you · which list?", "Ready for you".
    pub status: String,
    pub progress: f32,
    pub state: TaskState,
    /// What Enter does once it's ready ("Open PR #815 for review").
    pub approve: Option<String>,
    pub view: ViewSpec,
}

impl Task {
    pub fn agent(id: &str, task: &str) -> Self {
        Self {
            id: id.to_string(),
            label: format!("{AGENT_LABEL} · {}", short(task)),
            status: "Starting".into(),
            progress: 0.0,
            state: TaskState::Working,
            approve: None,
            view: ViewSpec {
                name: AGENT_VIEW.into(),
                label: AGENT_LABEL.into(),
                enter: None,
                takes_input: true,
                text: Some(String::new()),
                leave_text: Some(String::new()),
            },
        }
    }

    /// What ↵ says in the task's view.
    pub fn enter(&self, typed: &str) -> EnterTarget {
        if !typed.trim().is_empty() {
            return EnterTarget::new("agent:send", "Send to the agent");
        }
        match self.state {
            TaskState::Working => EnterTarget::new("agent:running", "Nothing to do — it's working"),
            TaskState::NeedsYou => EnterTarget::new("agent:answer", "Pick an answer, or say it"),
            TaskState::Ready => EnterTarget::new(
                "agent:approve",
                self.approve.clone().unwrap_or_else(|| "Looks good".into()),
            ),
            TaskState::Failed => EnterTarget::new("agent:failed", "Say it again to retry"),
        }
    }

    pub fn spec(&self, typed: &str) -> ViewSpec {
        ViewSpec {
            enter: Some(self.enter(typed)),
            ..self.view.clone()
        }
    }
}

/// A task's words, short enough for the strip.
fn short(task: &str) -> String {
    let words: Vec<&str> = task.split_whitespace().collect();
    let mut out = words.iter().take(6).copied().collect::<Vec<_>>().join(" ");
    if words.len() > 6 {
        out.push('…');
    }
    out
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Strip {
    tasks: Vec<Task>,
    started: u64,
}

impl Strip {
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn get(&self, id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// A new id for a task starting now.
    pub fn next_id(&mut self) -> String {
        self.started += 1;
        format!("agent:{}", self.started)
    }

    pub fn start(&mut self, task: Task) {
        self.tasks.retain(|t| t.id != task.id);
        self.tasks.push(task);
    }

    /// Returns whether the task is in the strip.
    pub fn update(
        &mut self,
        id: &str,
        status: String,
        progress: f32,
        state: TaskState,
        approve: Option<String>,
    ) -> bool {
        let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) else {
            return false;
        };
        t.status = status;
        t.progress = progress.clamp(0.0, 1.0);
        t.state = state;
        if approve.is_some() {
            t.approve = approve;
        }
        true
    }

    pub fn end(&mut self, id: &str) -> Option<Task> {
        let ix = self.tasks.iter().position(|t| t.id == id)?;
        Some(self.tasks.remove(ix))
    }

    /// What ⌘J opens: whatever needs you first, then the newest task.
    pub fn target(&self) -> Option<&Task> {
        self.tasks
            .iter()
            .find(|t| t.state == TaskState::NeedsYou)
            .or_else(|| self.tasks.last())
    }

    /// The task whose view is `name`, for parking it.
    pub fn owner_of_view(&self, name: &str) -> Option<&Task> {
        self.tasks.iter().rev().find(|t| t.view.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_j_opens_what_needs_you_first_then_the_newest() {
        let mut strip = Strip::default();
        assert!(strip.target().is_none());
        let a = strip.next_id();
        strip.start(Task::agent(&a, "fix SK-431 in the launcher"));
        let b = strip.next_id();
        strip.start(Task::agent(&b, "review the beta notes"));
        assert_eq!(strip.target().map(|t| t.id.as_str()), Some(b.as_str()));
        strip.update(
            &a,
            "Needs you · which list?".into(),
            0.5,
            TaskState::NeedsYou,
            None,
        );
        assert_eq!(strip.target().map(|t| t.id.as_str()), Some(a.as_str()));
        strip.end(&a);
        assert_eq!(strip.target().map(|t| t.id.as_str()), Some(b.as_str()));
    }

    #[test]
    fn an_agent_task_says_what_enter_does_in_its_view() {
        let mut t = Task::agent(
            "agent:1",
            "have the agent fix SK-431 in the launcher flicker",
        );
        assert_eq!(t.label, "Agent · have the agent fix SK-431 in…");
        assert_eq!(t.enter("").label, "Nothing to do — it's working");
        assert_eq!(t.enter("both").label, "Send to the agent");
        t.state = TaskState::Ready;
        t.approve = Some("Open PR #815 for review".into());
        assert_eq!(
            t.enter(""),
            EnterTarget::new("agent:approve", "Open PR #815 for review")
        );
    }
}
