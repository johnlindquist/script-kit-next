//! What the model's thread is doing beyond its words, told to the reducer: the tools it runs, what
//! it asks before doing something, an account it needs, the models it can use. And what the
//! reducer answers: a turn with what waited in the line, a yes or a no, a sign-in, another model.

use gpui::{ClipboardItem, Context};
use sk_one_window::asks::{Asks, Choice};
use sk_one_window::chat::{Tool, ToolState};
use sk_one_window::models::Models;
use sk_one_window::work::WorkEvent;
use sk_one_window::{Done, Event, Row, Subject};

use crate::ai::agent_chat::ui::thread::{
    AgentChatSignInPromptKind, AgentChatThread, AgentChatThreadMessageRole as Role,
};
use crate::ai::agent_chat::ui::tool_card::AgentChatToolStatus;
use crate::ScriptListApp;

/// What the reducer last heard about the chat's thread beyond its words.
#[derive(Default, PartialEq)]
pub(crate) struct Heard {
    tools: Vec<Tool>,
    asks: Option<u64>,
    sign_in: Option<String>,
    models: Models,
}

/// The tools the model ran since your latest turn.
fn tools_of(thread: &AgentChatThread) -> Vec<Tool> {
    let after = thread
        .messages
        .iter()
        .rev()
        .find(|m| m.role == Role::User)
        .map_or(0, |m| m.id);
    thread
        .active_tool_calls()
        .iter()
        .filter(|t| t.message_id > after)
        .map(|t| Tool {
            title: t.title.clone(),
            subject: t.subject.clone(),
            state: match AgentChatToolStatus::from_status_str(&t.status) {
                _ if t.is_error => ToolState::Failed,
                AgentChatToolStatus::Complete => ToolState::Done,
                AgentChatToolStatus::Failed => ToolState::Failed,
                AgentChatToolStatus::Pending | AgentChatToolStatus::Running => ToolState::Running,
            },
        })
        .collect()
}

/// What the model asks before it goes on.
fn asks_of(thread: &AgentChatThread) -> Option<Asks> {
    let request = thread.pending_permission.as_ref()?;
    let detail = request
        .preview
        .as_ref()
        .and_then(|p| p.subject.clone())
        .or_else(|| {
            request
                .body
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .map(str::to_string)
        })
        .filter(|detail| *detail != request.title);
    Some(Asks {
        id: request.id,
        title: request.title.clone(),
        detail,
        choices: request
            .options
            .iter()
            .map(|o| {
                let choice = Choice::new(o.option_id.clone(), o.name.clone(), !o.is_reject());
                if o.is_persistent_allow() {
                    choice.lasting()
                } else {
                    choice
                }
            })
            .collect(),
    })
}

fn sign_in_of(thread: &AgentChatThread) -> Option<String> {
    thread
        .pending_sign_in_prompt()
        .filter(|prompt| prompt.kind == AgentChatSignInPromptKind::Consent)
        .map(|prompt| prompt.provider.to_string())
}

fn models_of(thread: &AgentChatThread) -> Models {
    Models {
        list: thread
            .available_models()
            .iter()
            .map(|m| Row::new(m.id.clone(), m.display_name.clone().unwrap_or(m.id.clone())))
            .collect(),
        current: thread.selected_model_id().map(str::to_string),
    }
}

impl Heard {
    /// What changed since the reducer last heard, as events for it.
    pub(crate) fn news(&mut self, thread: &AgentChatThread) -> Vec<WorkEvent> {
        let mut events = Vec::new();
        let tools = tools_of(thread);
        if tools != self.tools {
            events.push(WorkEvent::ModelTools(tools.clone()));
            self.tools = tools;
        }
        let asks = asks_of(thread);
        if asks.as_ref().map(|a| a.id) != self.asks {
            self.asks = asks.as_ref().map(|a| a.id);
            events.push(WorkEvent::ModelAsks(asks));
        }
        let sign_in = sign_in_of(thread);
        if sign_in != self.sign_in {
            events.push(WorkEvent::ModelSignIn(sign_in.clone()));
            self.sign_in = sign_in;
        }
        let models = models_of(thread);
        if models != self.models {
            events.push(WorkEvent::Models(models.clone()));
            self.models = models;
        }
        events
    }
}

impl ScriptListApp {
    /// A turn for the model with the parts that waited in the line for it.
    pub(crate) fn one_window_ask_with(
        &mut self,
        text: String,
        with: Vec<Subject>,
        cx: &mut Context<Self>,
    ) {
        let parts: Vec<_> = with
            .iter()
            .filter_map(|s| s.row_key.as_ref())
            .filter_map(|key| self.one_window_host.work.parts.remove(key))
            .collect();
        self.one_window_ask_model(text, parts, cx);
    }

    pub(crate) fn one_window_chat_thread(
        &self,
    ) -> Option<gpui::Entity<crate::ai::agent_chat::ui::thread::AgentChatThread>> {
        self.one_window_host
            .work
            .chat
            .as_ref()
            .map(|m| m.thread.clone())
    }

    /// Yes or no to what the model asked; the thread goes on or stops that step.
    pub(crate) fn one_window_answer_model(
        &mut self,
        id: u64,
        choice: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(thread) = self.one_window_chat_thread() else {
            return;
        };
        thread.update(cx, |thread, cx| {
            if thread.pending_permission.as_ref().map(|r| r.id) == Some(id) {
                thread.approve_pending_permission(choice, cx);
            }
        });
    }

    pub(crate) fn one_window_model_sign_in(&mut self, approve: bool, cx: &mut Context<Self>) {
        if let Some(thread) = self.one_window_chat_thread() {
            thread.update(cx, |thread, cx| thread.answer_sign_in_consent(approve, cx));
        }
    }

    /// The chat goes on with another model; the line says which.
    pub(crate) fn one_window_use_model(&mut self, id: String, cx: &mut Context<Self>) {
        let Some(thread) = self.one_window_chat_thread() else {
            return;
        };
        let name = thread.update(cx, |thread, cx| {
            thread.select_model(&id, cx);
            thread
                .available_models()
                .iter()
                .find(|m| m.id == id)
                .map(|m| m.display_name.clone().unwrap_or(m.id.clone()))
        });
        self.one_window_handle(
            Event::Done(Done {
                icon: "sparkles".into(),
                title: format!("Using {} for this chat", name.unwrap_or(id)),
                stay: true,
                ..Done::default()
            }),
            cx,
        );
    }

    /// The whole chat as Markdown, in Downloads.
    pub(crate) fn one_window_export_chat(&mut self, cx: &mut Context<Self>) -> Option<Done> {
        let thread = self.one_window_chat_thread()?;
        let (markdown, id) = {
            let thread = thread.read(cx);
            (
                crate::ai::agent_chat::ui::export::build_agent_chat_conversation_markdown_from_thread(
                    thread,
                )?,
                thread.ui_thread_id().to_string(),
            )
        };
        let dir = dirs::download_dir().unwrap_or_else(std::env::temp_dir);
        match crate::ai::agent_chat::ui::export::persist_private_agent_chat_export(
            &dir, &id, &markdown,
        ) {
            Ok(path) => Some(Done {
                icon: "export".into(),
                title: format!(
                    "Exported the chat to {}",
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Downloads")
                ),
                ..Done::default()
            }),
            Err(error) => {
                tracing::warn!(target: "script_kit::one_window", event = "chat_export_failed", kind = ?error.kind());
                cx.write_to_clipboard(ClipboardItem::new_string(markdown));
                Some(Done {
                    icon: "copy".into(),
                    title: "Couldn't save the chat, so it's on the clipboard".into(),
                    ..Done::default()
                })
            }
        }
    }
}

/// What the provider-free fixture model does with a turn, so owned evaluation can show every
/// part of the chat: it reads a file when asked about one, asks before it cleans anything up,
/// and otherwise answers at once. Returns the question it asks, once the turn has started.
pub(crate) fn fixture_turn(
    control: &crate::ai::agent_chat::ui::mock_fixture::AgentChatFixtureControl,
    text: &str,
    reply: &str,
) -> Option<crate::ai::agent_chat::ui::AgentChatApprovalRequestInput> {
    use crate::ai::agent_chat::events::AgentChatEvent;
    let lower = text.to_lowercase();
    let asks = ["clean", "delete", "remove"]
        .iter()
        .any(|w| lower.contains(w));
    let reads = ["read", "file"].iter().any(|w| lower.contains(w));
    let (title, tool, input) = if asks {
        (
            "Run rm -rf build",
            "bash",
            serde_json::json!({ "command": "rm -rf build" }),
        )
    } else {
        (
            "Read README.md",
            "read",
            serde_json::json!({ "path": "README.md" }),
        )
    };
    let mut events = Vec::new();
    if asks || reads {
        FIXTURE_TOOLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        events.push(AgentChatEvent::ToolCallStarted {
            tool_call_id: fixture_tool_id(),
            title: title.into(),
            status: "running".into(),
            tool_name: Some(tool.into()),
            raw_input: Some(input),
        });
    }
    if !asks {
        if reads {
            events.push(fixture_tool_done(true));
        }
        events.push(AgentChatEvent::AgentMessageDelta(reply.to_string()));
        events.push(AgentChatEvent::completed("fixture"));
    }
    if let Err(error) = control.queue_turn(events) {
        tracing::warn!(target: "script_kit::one_window", event = "fixture_turn_failed", %error);
    }
    asks.then(
        || crate::ai::agent_chat::ui::AgentChatApprovalRequestInput {
            title: title.into(),
            body: "rm -rf build".into(),
            preview: None,
            options: [
                ("allow-once", "Allow once", "AllowOnce"),
                ("allow-always", "Always allow", "AllowAlways"),
                ("reject-once", "Don't allow", "RejectOnce"),
            ]
            .into_iter()
            .map(
                |(id, name, kind)| crate::ai::agent_chat::ui::AgentChatApprovalOption {
                    option_id: id.into(),
                    name: name.into(),
                    kind: kind.into(),
                },
            )
            .collect(),
        },
    )
}

/// Each fixture tool call has its own id, so a later call never rewrites an earlier card.
static FIXTURE_TOOLS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn fixture_tool_id() -> String {
    format!(
        "fixture-tool-{}",
        FIXTURE_TOOLS.load(std::sync::atomic::Ordering::Relaxed)
    )
}

fn fixture_tool_done(ok: bool) -> crate::ai::agent_chat::events::AgentChatEvent {
    crate::ai::agent_chat::events::AgentChatEvent::ToolCallUpdated {
        tool_call_id: fixture_tool_id(),
        title: None,
        status: Some(if ok { "completed" } else { "failed" }.into()),
        body: None,
        raw_input: None,
        diff: None,
        is_error: !ok,
    }
}

/// The fixture finishes the turn it paused to ask about.
pub(crate) fn fixture_answered(
    control: &crate::ai::agent_chat::ui::mock_fixture::AgentChatFixtureControl,
    allowed: bool,
) {
    use crate::ai::agent_chat::events::AgentChatEvent;
    let Ok(generation) = control.latest_generation() else {
        return;
    };
    let reply = if allowed {
        "Cleaned up the build folder."
    } else {
        "Left the build folder as it was."
    };
    for event in [
        fixture_tool_done(allowed),
        AgentChatEvent::AgentMessageDelta(reply.into()),
        AgentChatEvent::completed("fixture"),
    ] {
        if let Err(error) = control.emit(generation, event) {
            tracing::warn!(target: "script_kit::one_window", event = "fixture_emit_failed", %error);
        }
    }
}
