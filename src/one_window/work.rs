//! Hosts the work the reducer opens under the input: the model's chat and agents run on Agent
//! Chat threads, steps are read and run, and services connect in place.
//!
//! The reducer decides what each view shows and what Enter does there; this runs what it asks
//! for (a turn for the model, a step, a sign-in) and feeds back what happened.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use gpui::{ClipboardItem, Context, Entity, Subscription};
use sk_one_window::steps::StepSpec;
use sk_one_window::strip::TaskState;
use sk_one_window::work::{WorkEffect, WorkEvent};
use sk_one_window::{chat, connect, past_chats, steps, strip, Done, Event, Risk, Row};

use crate::ai::agent_chat::ui::mock_fixture::AgentChatFixtureControl;
use crate::ai::agent_chat::ui::thread::{
    AgentChatThread, AgentChatThreadMessageRole as Role, AgentChatThreadStatus as Status,
};
use crate::{AppView, FocusTarget, MainServices, ScriptListApp};

/// The views whose keys the reducer hands back to this host.
pub(crate) const WORK_VIEWS: [&str; 5] = [
    chat::VIEW,
    steps::VIEW,
    connect::VIEW,
    strip::AGENT_VIEW,
    past_chats::VIEW,
];
const CHAT_THREAD: &str = "one-window-chat";
/// Each fixture chat is its own conversation in history.
static FIXTURE_CHATS: AtomicU64 = AtomicU64::new(1);
/// What the owned-evaluation fixture model says; it never calls a provider.
const FIXTURE_REPLY: &str = "Fixture Agent Chat response.";

/// A conversation the window runs on Agent Chat: the thread, and what the reducer last heard
/// about it.
pub(crate) struct ModelThread {
    pub(crate) thread: Entity<AgentChatThread>,
    fixture: Option<AgentChatFixtureControl>,
    heard: Option<(String, Status)>,
    news: super::chat_host::Heard,
    _observer: Subscription,
}

#[derive(Default)]
pub(crate) struct WorkHost {
    pub(crate) chat: Option<ModelThread>,
    pub(crate) agents: HashMap<String, ModelThread>,
    /// What waits in the line for the chat's next turn, by the key its subject carries.
    pub(crate) parts: HashMap<String, crate::ai::message_parts::AiContextPart>,
    pub(crate) next_part: u64,
    /// The view that stepped aside for the chat, and where its focus goes back.
    pub(crate) return_to: Option<(AppView, FocusTarget)>,
    /// What ⌘K staged, for "Ask the model about it".
    pub(crate) about: Option<crate::ai::message_parts::AiContextPart>,
}

/// The words of the model's reply after the latest turn you wrote.
fn reply_of(thread: &AgentChatThread) -> String {
    let after_you = thread
        .messages
        .iter()
        .rposition(|m| m.role == Role::User)
        .map_or(0, |i| i + 1);
    thread.messages[after_you..]
        .iter()
        .filter(|m| m.role == Role::Assistant)
        .map(|m| m.body.to_string())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// A reading that asks the model or hands the job to an agent opens that work in place.
fn work_of_plan(plan: &crate::intent::Plan) -> Option<WorkEvent> {
    use crate::intent::engine::Step;
    match plan.steps.as_slice() {
        [Step::Ask { question, .. }] => Some(WorkEvent::AskModel {
            sentence: question.clone(),
        }),
        [Step::Transform {
            tool: None,
            instruction: Some(instruction),
            ..
        }] => Some(WorkEvent::AskModel {
            sentence: instruction.clone(),
        }),
        [Step::Recall { source, query }] if source == "history" => Some(WorkEvent::OpenPastChats {
            query: query.clone().unwrap_or_default(),
        }),
        [Step::Delegate { task, project, .. }] => Some(WorkEvent::Agent {
            task: match project {
                Some(project) => format!("In {project}: {task}"),
                None => task.clone(),
            },
        }),
        _ => None,
    }
}

/// "tell mo i'm running late" → ("Mo", "I'm running late").
fn message_step(text: &str) -> Option<(String, String)> {
    let rest = text.strip_prefix("tell ")?;
    let (who, what) = rest.split_once(' ')?;
    let cap = |s: &str| {
        let mut c = s.chars();
        c.next()
            .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
            .unwrap_or_default()
    };
    Some((cap(who), cap(what.trim_start_matches("that "))))
}

/// "add to today's note: ship beta notes" → "ship beta notes".
fn note_step(text: &str) -> Option<&str> {
    let (head, body) = text.split_once(':')?;
    head.contains("note")
        .then(|| body.trim())
        .filter(|b| !b.is_empty())
}

/// The service a reading needs: one it recalls from that Script Kit can't read yet.
pub(crate) fn needs_of(plan: &crate::intent::Plan) -> Option<String> {
    plan.steps.iter().find_map(|step| match step {
        crate::intent::engine::Step::Recall { source, .. } => connect::integration(source)
            .filter(|_| {
                crate::intent::engine::recall_source(source).is_none_or(|s| s.builtin.is_none())
            })
            .map(|i| i.key.to_string()),
        _ => None,
    })
}

/// "Needs Google Calendar · <what it does> · ↵ connects it" for a row that waits on a service.
pub(crate) fn needs_subtitle(row: &Row, work: &sk_one_window::work::Work) -> Option<String> {
    let label = connect::needs_label(row, work.connections())?;
    Some(match &row.sub {
        Some(sub) => format!("{label} · {sub}"),
        None => label,
    })
}

/// A sentence with several jobs gets the local "Do these N things" row first.
pub(crate) fn steps_row(
    text: &str,
    rows: &mut Vec<Row>,
    results: &mut HashMap<String, crate::scripts::SearchResult>,
) {
    let Some(row) = steps::steps_row(text) else {
        return;
    };
    results.insert(
        row.key.clone(),
        crate::scripts::SearchResult::Intent(crate::scripts::IntentMatch {
            slot: crate::intent::READINGS,
            request: text.to_string(),
            title: row.title.clone(),
            subtitle: row.sub.clone().unwrap_or_default(),
            verb: "Run",
            reading: false,
            stale: false,
            plan: None,
            score: 100,
        }),
    );
    rows.push(row);
}

impl ScriptListApp {
    /// Whether one of the work views covers the list.
    pub(crate) fn one_window_work_view_open(&self) -> bool {
        self.one_window
            .views()
            .top()
            .is_some_and(|top| WORK_VIEWS.contains(&top.name.as_str()))
    }

    /// What an agent said last, for its view.
    pub(crate) fn one_window_agent_reply(&self, id: &str) -> Option<&str> {
        self.one_window_host
            .work
            .agents
            .get(id)
            .and_then(|m| m.heard.as_ref())
            .map(|(reply, _)| reply.as_str())
            .filter(|reply| !reply.is_empty())
    }

    /// Rows that open work in place instead of running: the steps row, asking the model, and a
    /// reading that hands the job to an agent.
    pub(crate) fn one_window_route_work(&mut self, row: &Row, cx: &mut Context<Self>) -> bool {
        let said = self.one_window.said();
        let plan = self
            .intent_launcher
            .plan(self.one_window.text(), &row.key)
            .filter(|plan| plan.ready);
        if row.key == super::chat_entry::ABOUT_KEY {
            let entry = match self.one_window_host.work.about.take() {
                Some(part) => super::chat_entry::ChatEntry::quick().with(part),
                None => super::chat_entry::ChatEntry::quick()
                    .capturing(crate::ai::TabAiCaptureKind::DefaultContext),
            };
            self.one_window_open_chat(entry, cx);
            return true;
        }
        let event = if row.key == steps::ROW_KEY {
            WorkEvent::OpenSteps { sentence: said }
        } else if row.key.starts_with("down:model") || row.key.starts_with("fallback-ai:") {
            WorkEvent::AskModel { sentence: said }
        } else if let Some(event) = plan.as_deref().and_then(work_of_plan) {
            event
        } else {
            return false;
        };
        self.one_window_handle(Event::Work(event), cx);
        true
    }

    /// Does what the reducer asked of the work views.
    pub(crate) fn one_window_apply_work(&mut self, effect: WorkEffect, cx: &mut Context<Self>) {
        tracing::info!(target: "script_kit::one_window", event = "work", effect = ?effect);
        match effect {
            WorkEffect::AskModel { text } => self.one_window_ask_model(text, Vec::new(), cx),
            WorkEffect::AskModelWith { text, with } => self.one_window_ask_with(text, with, cx),
            WorkEffect::AnswerModel { id, choice } => {
                let allowed = choice.as_deref().is_some_and(|c| !c.starts_with("reject"));
                self.one_window_answer_model(id, choice, cx);
                if let Some(control) = self
                    .one_window_host
                    .work
                    .chat
                    .as_ref()
                    .and_then(|m| m.fixture.as_ref())
                {
                    super::chat_host::fixture_answered(control, allowed);
                }
            }
            WorkEffect::ModelSignIn { approve } => self.one_window_model_sign_in(approve, cx),
            WorkEffect::UseModel { id } => self.one_window_use_model(id, cx),
            WorkEffect::ListPastChats => self.one_window_list_past_chats(cx),
            WorkEffect::ReopenChat { id } => self.one_window_reopen_chat(id, cx),
            WorkEffect::PastChat { id, act } => self.one_window_past_chat(id, act, cx),
            WorkEffect::ChatStep { row, reply } => {
                let done = match row.key.as_str() {
                    chat::EXPORT_KEY => match self.one_window_export_chat(cx) {
                        Some(done) => done,
                        None => return,
                    },
                    chat::COPY_KEY => {
                        cx.write_to_clipboard(ClipboardItem::new_string(reply));
                        Done {
                            icon: "copy".into(),
                            title: "Copied the model's reply".into(),
                            ..Done::default()
                        }
                    }
                    "chat:note" => {
                        if let Err(error) =
                            crate::intent_launcher::append_capture_to_day("note", &reply, None)
                        {
                            tracing::warn!(target: "script_kit::one_window", event = "chat_note_failed", error);
                            return;
                        }
                        Done {
                            icon: "append".into(),
                            title: "Added the model's reply to today's note".into(),
                            risk: Some(Risk::Undo),
                            ..Done::default()
                        }
                    }
                    _ => Done {
                        icon: row.icon.clone().unwrap_or_else(|| "check".into()),
                        title: row.title.clone(),
                        ..Done::default()
                    },
                };
                self.one_window_host.work.chat = None;
                self.one_window_handle(Event::Done(done), cx);
            }
            WorkEffect::ReadSteps { sentence } => {
                let steps = steps::split(&sentence)
                    .into_iter()
                    .map(|text| StepSpec {
                        needs: connect::named_in(&text).map(str::to_string),
                        title: Some(text.clone()),
                        text,
                    })
                    .collect();
                self.one_window_work_later(WorkEvent::StepsRead { sentence, steps }, cx);
            }
            WorkEffect::RunSteps { steps } => {
                for (index, text) in steps {
                    self.one_window_run_step(&text, cx);
                    self.one_window_handle(Event::Work(WorkEvent::StepDone { index }), cx);
                }
            }
            WorkEffect::SignIn { integration } => {
                super::connections::sign_in(integration, cx);
            }
            WorkEffect::CopyInstead { integration, text } => {
                tracing::info!(target: "script_kit::one_window", event = "copy_instead", integration);
                cx.write_to_clipboard(ClipboardItem::new_string(text));
            }
            WorkEffect::StartAgent { id, task } => {
                match self.one_window_model_thread(id.clone(), cx) {
                    Ok(model) => {
                        submit(&model, &task, cx);
                        self.one_window_host.work.agents.insert(id, model);
                    }
                    Err(error) => {
                        tracing::warn!(target: "script_kit::one_window", event = "agent_unavailable", %error);
                        self.one_window_work_later(
                            WorkEvent::TaskChanged {
                                id,
                                status: "Couldn't start".into(),
                                progress: 0.0,
                                state: TaskState::Failed,
                                approve: None,
                            },
                            cx,
                        );
                    }
                }
            }
            WorkEffect::AgentSay { id, text } => {
                if let Some(model) = self.one_window_host.work.agents.get(&id) {
                    submit(model, &text, cx);
                }
            }
            WorkEffect::AgentApprove { id } => {
                let title = self
                    .one_window
                    .work()
                    .strip()
                    .get(&id)
                    .map(|t| match &t.approve {
                        Some(approve) => format!("{approve} · {}", t.label),
                        None => t.label.clone(),
                    })
                    .unwrap_or_default();
                self.one_window_host.work.agents.remove(&id);
                self.one_window_handle(Event::Work(WorkEvent::TaskEnded { id }), cx);
                self.one_window_handle(
                    Event::Done(Done {
                        icon: "delegate".into(),
                        title,
                        risk: Some(Risk::Undo),
                        ..Done::default()
                    }),
                    cx,
                );
            }
            WorkEffect::Parked { .. } | WorkEffect::Reopened { .. } => {}
        }
    }

    /// A step of "Do all": a message waits in the line before it goes, a note line lands in
    /// today's note, and anything else is handed to an agent that works in the strip.
    fn one_window_run_step(&mut self, text: &str, cx: &mut Context<Self>) {
        if let Some((who, what)) = message_step(text) {
            self.one_window_handle(
                Event::Hold {
                    label: format!("To {who}: “{what}”"),
                    request: text.to_string(),
                },
                cx,
            );
        } else if let Some(body) = note_step(text) {
            if let Err(error) = crate::intent_launcher::append_capture_to_day("note", body, None) {
                tracing::warn!(target: "script_kit::one_window", event = "step_note_failed", error);
            }
        } else {
            self.one_window_handle(
                Event::Work(WorkEvent::Background {
                    task: text.to_string(),
                }),
                cx,
            );
        }
    }

    /// Feeds a fact on the next turn, the way an answer arrives after the key that asked for it.
    pub(crate) fn one_window_work_later(&mut self, event: WorkEvent, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let _ = this.update(cx, |app, cx| {
                app.one_window_handle(Event::Work(event), cx);
            });
        })
        .detach();
    }

    /// A turn for the model, with what went with it; the chat's thread starts on the first one.
    pub(crate) fn one_window_ask_model(
        &mut self,
        text: String,
        parts: Vec<crate::ai::message_parts::AiContextPart>,
        cx: &mut Context<Self>,
    ) {
        if self.one_window_host.work.chat.is_none() {
            match self.one_window_fresh_chat_thread(cx) {
                Ok(model) => self.one_window_host.work.chat = Some(model),
                Err(error) => {
                    tracing::warn!(target: "script_kit::one_window", event = "model_unavailable", %error);
                    self.one_window_work_later(
                        WorkEvent::ModelFailed {
                            message: "The model couldn't start".into(),
                        },
                        cx,
                    );
                    return;
                }
            }
        }
        if let Some(model) = self.one_window_host.work.chat.as_ref() {
            if !parts.is_empty() {
                model.thread.update(cx, |thread, cx| {
                    for part in parts {
                        thread.add_context_part(part, cx);
                    }
                });
            }
            submit(model, &text, cx);
        }
    }

    /// A new thread for the model's chat.
    pub(crate) fn one_window_fresh_chat_thread(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Result<ModelThread, String> {
        self.one_window_model_thread(CHAT_THREAD.to_string(), cx)
    }

    /// A thread for the model: the provider-free fixture in owned evaluation, Agent Chat's warm
    /// session otherwise.
    fn one_window_model_thread(
        &mut self,
        id: String,
        cx: &mut Context<Self>,
    ) -> Result<ModelThread, String> {
        let (thread, fixture) = if matches!(self.main_services, MainServices::OwnedFixtures(_)) {
            let thread_id = if id == CHAT_THREAD {
                format!(
                    "{CHAT_THREAD}-{}",
                    FIXTURE_CHATS.fetch_add(1, Ordering::Relaxed)
                )
            } else {
                id.clone()
            };
            let (thread, control) =
                crate::ai::agent_chat::ui::mock_fixture::create_mock_fixture_thread(
                    &thread_id,
                    crate::ai::agent_chat::ui::capabilities::AgentChatSessionPolicy::Full,
                    cx,
                );
            thread.update(cx, |thread, cx| thread.mark_context_bootstrap_ready(cx));
            (thread, Some(control))
        } else {
            let thread = crate::ai::agent_chat::ui::hosted::spawn_hosted_thread(
                None,
                crate::ai::agent_chat::ui::AgentChatLaunchRequirements::default(),
                cx,
            )?;
            (thread, None)
        };
        let observer = cx.observe(&thread, move |app, thread, cx| {
            app.one_window_model_changed(&id, &thread, cx);
        });
        Ok(ModelThread {
            thread,
            fixture,
            heard: None,
            news: super::chat_host::Heard::default(),
            _observer: observer,
        })
    }

    fn one_window_model_changed(
        &mut self,
        id: &str,
        thread: &Entity<AgentChatThread>,
        cx: &mut Context<Self>,
    ) {
        let (status, reply) = {
            let t = thread.read(cx);
            (t.status, reply_of(t))
        };
        let work = &mut self.one_window_host.work;
        let model = if id == CHAT_THREAD {
            work.chat.as_mut()
        } else {
            work.agents.get_mut(id)
        };
        let Some(model) = model else {
            return;
        };
        let news = if id == CHAT_THREAD {
            model.news.news(thread.read(cx))
        } else {
            Vec::new()
        };
        let heard = Some((reply.clone(), status));
        if model.heard == heard {
            for event in news {
                self.one_window_handle(Event::Work(event), cx);
            }
            return;
        }
        model.heard = heard;
        let written = !reply.is_empty() && status == Status::Idle;
        let events = if id == CHAT_THREAD {
            match status {
                Status::Error => vec![WorkEvent::ModelFailed {
                    message: "The model couldn't answer".into(),
                }],
                _ if reply.is_empty() => Vec::new(),
                _ if written => vec![
                    WorkEvent::ModelWrote {
                        text: reply,
                        done: true,
                    },
                    WorkEvent::ModelNext(Vec::new()),
                ],
                _ => vec![WorkEvent::ModelWrote {
                    text: reply,
                    done: false,
                }],
            }
        } else {
            let (state, status_text, progress) = match status {
                Status::Error => (TaskState::Failed, "Couldn't finish".to_string(), 0.0),
                Status::WaitingForPermission => {
                    (TaskState::NeedsYou, "Needs you · allow a step".into(), 0.5)
                }
                _ if written => (TaskState::Ready, "Ready for you".into(), 1.0),
                _ => (TaskState::Working, "Working".into(), 0.3),
            };
            vec![WorkEvent::TaskChanged {
                id: id.to_string(),
                status: status_text,
                progress,
                state,
                approve: written.then(|| "Looks good".to_string()),
            }]
        };
        for event in news.into_iter().chain(events) {
            self.one_window_handle(Event::Work(event), cx);
        }
    }
}

/// Sends a turn: the fixture answers from its queue, a live session from the provider.
fn submit(model: &ModelThread, text: &str, cx: &mut Context<ScriptListApp>) {
    let asks = model
        .fixture
        .as_ref()
        .and_then(|control| super::chat_host::fixture_turn(control, text, FIXTURE_REPLY));
    let words = text.to_string();
    model.thread.update(cx, |thread, cx| {
        thread.set_input(words, cx);
        if let Err(error) = thread.submit_input(cx) {
            tracing::warn!(target: "script_kit::one_window", event = "model_submit_failed", %error);
        }
    });
    if let (Some(control), Some(asks)) = (model.fixture.as_ref(), asks) {
        if let Err(error) = control.request_permission(asks) {
            tracing::warn!(target: "script_kit::one_window", event = "fixture_permission_failed", %error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_read_messages_and_note_lines() {
        assert_eq!(
            message_step("tell mo i'm running late"),
            Some(("Mo".into(), "I'm running late".into()))
        );
        assert_eq!(
            note_step("add to today's note: ship beta notes"),
            Some("ship beta notes")
        );
        assert_eq!(note_step("move standup to 10"), None);
    }
}
