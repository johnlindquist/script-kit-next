//! Work the window does in place: the model's chat, steps, connecting a service, and agents in the
//! strip. The host feeds what happened (a reply streamed, a step was read, a service connected, an
//! agent moved along) and hands back the keys it got for these views as [`Effect::ViewKey`]; the
//! rules are in [`crate::chat`], [`crate::steps`], [`crate::connect`] and [`crate::strip`].
//!
//! [`Effect::ViewKey`]: crate::window::Effect::ViewKey

use crate::asks::Asks;
use crate::chat::{Chat, Tool, Turn};
use crate::connect::{Connecting, Connections};
use crate::models::Models;
use crate::past_chats::{PastChat, PastChats};
use crate::row::Row;
use crate::stage::Subject;
use crate::steps::{StepSpec, Steps};
use crate::strip::{Strip, TaskState};
use crate::window::Key;

#[derive(Clone, Debug, PartialEq)]
pub enum WorkEvent {
    /// A key the host got back as `Effect::ViewKey` while one of these views was on top.
    ViewKey(Key),
    /// Ask the model: its chat opens under the input with these words as the first turn.
    AskModel {
        sentence: String,
    },
    /// The model's reply so far; `done` once it's all written.
    ModelWrote {
        text: String,
        done: bool,
    },
    /// What Jev ranks to do with the reply.
    ModelNext(Vec<Row>),
    /// The model couldn't answer; what the screen says about it.
    ModelFailed {
        message: String,
    },
    /// A sentence with several jobs opens as steps.
    OpenSteps {
        sentence: String,
    },
    /// Each step of `sentence`, read.
    StepsRead {
        sentence: String,
        steps: Vec<StepSpec>,
    },
    /// A step finished while the steps ran.
    StepDone {
        index: usize,
    },
    /// A step ran and didn't do what it said; the host said why in the line.
    StepFailed {
        index: usize,
    },
    /// What the user had connected before this session.
    Connections(Vec<String>),
    Connected {
        integration: String,
    },
    ConnectFailed {
        integration: String,
        message: String,
    },
    /// Hand a job to an agent: its view opens and it gets a place in the strip.
    Agent {
        task: String,
    },
    TaskChanged {
        id: String,
        status: String,
        progress: f32,
        state: TaskState,
        approve: Option<String>,
    },
    TaskEnded {
        id: String,
    },
    /// Hand a job to an agent without opening its view: it works in the strip.
    Background {
        task: String,
    },
    /// Every way into the model's chat: words that go now (`send`) or wait in the input, and what
    /// the chat is about, which waits in the line until the next turn takes it.
    OpenChat {
        text: String,
        send: bool,
        with: Vec<Subject>,
    },
    /// The tools the model ran for the latest turn.
    ModelTools(Vec<Tool>),
    /// The model asks before it does something, or stopped asking.
    ModelAsks(Option<Asks>),
    /// The account the model needs you to sign in to, or none.
    ModelSignIn(Option<String>),
    Models(Models),
    /// What else the host can do to the chat, offered under ⌘K.
    ChatMore(Vec<Row>),
    /// The chat went back to before one of your messages. `text` was that
    /// message: it goes again when `resend`, otherwise it waits in the input.
    ChatRewound {
        turns: Vec<Turn>,
        text: String,
        resend: bool,
    },
    OpenPastChats {
        query: String,
    },
    /// The chats from before, newest first.
    PastChats(Vec<PastChat>),
    /// A chat from before is open again, with its turns.
    ChatReopened {
        id: String,
        turns: Vec<Turn>,
    },
    PastChatGone {
        id: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorkEffect {
    /// Send this turn to the model.
    AskModel {
        text: String,
    },
    /// Do a next step with the model's latest reply (copy it, add it to a note, use it).
    ChatStep {
        row: Box<Row>,
        reply: String,
    },
    /// Read each step of the sentence.
    ReadSteps {
        sentence: String,
    },
    /// Run these steps, by their place among the steps.
    RunSteps {
        steps: Vec<(usize, String)>,
    },
    /// Sign in to the service, in the connect view.
    SignIn {
        integration: String,
    },
    /// Go without the service: copy a message the user can paste into it.
    CopyInstead {
        integration: String,
        text: String,
    },
    StartAgent {
        id: String,
        task: String,
    },
    AgentSay {
        id: String,
        text: String,
    },
    AgentApprove {
        id: String,
    },
    /// The agent's view went into the strip; it keeps working.
    Parked {
        id: String,
    },
    Reopened {
        id: String,
    },
    /// Send this turn to the model with what waited in the line.
    AskModelWith {
        text: String,
        with: Vec<Subject>,
    },
    /// Answer the model's question; `None` cancels it.
    AnswerModel {
        id: u64,
        choice: Option<String>,
    },
    ModelSignIn {
        approve: bool,
    },
    UseModel {
        id: String,
    },
    ListPastChats,
    ReopenChat {
        id: String,
    },
    /// Copy, export or delete a chat from before.
    PastChat {
        id: String,
        act: &'static str,
    },
}

/// What the work views hold between events.
#[derive(Clone, Debug, Default)]
pub struct Work {
    pub(crate) chat: Option<Chat>,
    pub(crate) steps: Option<Steps>,
    pub(crate) connecting: Option<Connecting>,
    pub(crate) connections: Connections,
    pub(crate) strip: Strip,
    /// The task whose view is open.
    pub(crate) agent: Option<String>,
    pub(crate) past: Option<PastChats>,
}

impl Work {
    pub fn connections(&self) -> &Connections {
        &self.connections
    }

    pub fn strip(&self) -> &Strip {
        &self.strip
    }
}
