//! The model's chat, under the same input: the input is the chat's input, each reply is tagged as
//! written by the model, and under the latest reply are the next steps Jev ranks for it. Esc goes
//! back one step and restores the sentence the chat came from.
//!
//! What the chat was opened about (an error, the selection, a file) waits in the line above the
//! input and goes with the next turn. The model's tools show as cards in its reply; when it asks
//! before doing something, or needs you to sign in, those are rows under the reply: ↵ says yes,
//! ⌘2 says no. ⌘K puts the chat in the line and lists what you can do to it.

use crate::asks::Asks;
use crate::enter::EnterTarget;
use crate::models::Models;
use crate::row::Row;
use crate::stage::Subject;
use crate::window::ViewSpec;

pub const VIEW: &str = "chat";
pub const LABEL: &str = "The model";
/// The list's head under the latest reply.
pub const NEXT_HEAD: &str = "What to do with it · Jev ranks, you pick";
pub const WRITTEN_BY: &str = "Written by the model";
/// The key of the next step that asks the model for a shorter reply.
pub const SHORTER_KEY: &str = "chat:shorter";
pub const COPY_KEY: &str = "chat:copy";
pub const EXPORT_KEY: &str = "chat:export";
/// The list's head while ⌘K acts on the chat.
pub const ACTING_HEAD: &str = "What to do with this chat";
/// The list's head while the model waits for you.
pub const WAITING_HEAD: &str = "The model is waiting for you";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Who {
    You,
    Model,
    /// A tool the model ran while it answered.
    Tool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Turn {
    pub who: Who,
    pub text: String,
    /// Under the words: what went with one of your turns, or how a tool is doing.
    pub sub: Option<String>,
}

impl Turn {
    pub fn new(who: Who, text: impl Into<String>) -> Self {
        Self {
            who,
            text: text.into(),
            sub: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolState {
    Running,
    Done,
    Failed,
}

/// A tool the model ran, as its card says it.
#[derive(Clone, Debug, PartialEq)]
pub struct Tool {
    pub title: String,
    pub subject: Option<String>,
    pub state: ToolState,
}

impl Tool {
    fn turn(&self) -> Turn {
        let state = match self.state {
            ToolState::Running => "Running",
            ToolState::Done => "Done",
            ToolState::Failed => "Didn't work",
        };
        Turn {
            who: Who::Tool,
            text: self.title.clone(),
            sub: Some(match &self.subject {
                Some(subject) => format!("{state} · {subject}"),
                None => state.to_string(),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    /// Nothing asked yet: a quick question.
    Idle,
    Writing,
    /// The reply is written; Jev is ranking what to do with it.
    Ranking,
    Ready,
    /// The model couldn't answer; what the screen says about it.
    Failed(String),
}

/// What Enter did in the chat.
#[derive(Clone, Debug, PartialEq)]
pub enum ChatKey {
    Nothing,
    Moved,
    /// A new turn for the model, with what waited in the line.
    Ask {
        text: String,
        with: Vec<Subject>,
    },
    /// A next step for the host to do with the latest reply.
    Step(Box<Row>),
    /// The model's question, answered with one of its choices; `None` says no.
    Answer {
        id: u64,
        choice: Option<String>,
    },
    SignIn(bool),
    UseModel(String),
    PastChats,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chat {
    pub turns: Vec<Turn>,
    pub phase: Phase,
    pub next: Vec<Row>,
    pub sel: usize,
    /// What goes with the next turn; it waits in the line above the input.
    pub with: Vec<Subject>,
    /// The model asks before it does something.
    pub asks: Option<Asks>,
    /// The account the model needs you to sign in to before it can answer.
    pub sign_in: Option<String>,
    pub models: Models,
    /// ⌘K put the chat in the line; these are what you can do to it.
    pub acting: Option<Vec<Row>>,
}

/// Next steps for a reply when Jev hasn't ranked any.
pub fn default_next() -> Vec<Row> {
    vec![
        Row::new(COPY_KEY, "Copy it").with_icon("copy"),
        Row::new("chat:note", "Add it to today's note").with_icon("append"),
        Row::new(SHORTER_KEY, "Make it shorter").with_icon("edit"),
        Row::new(crate::past_chats::ROW_KEY, "See past chats").with_icon("history"),
    ]
}

impl Chat {
    /// Opens on the words it was asked, or empty for a quick question.
    pub fn open(sentence: &str) -> Self {
        let mut chat = Self {
            turns: Vec::new(),
            phase: Phase::Idle,
            next: Vec::new(),
            sel: 0,
            with: Vec::new(),
            asks: None,
            sign_in: None,
            models: Models::default(),
            acting: None,
        };
        let sentence = sentence.trim();
        if !sentence.is_empty() {
            chat.ask(sentence);
        }
        chat
    }

    /// Opens about `with`, which waits in the line and goes with the first turn.
    pub fn open_with(with: Vec<Subject>) -> Self {
        Self {
            with,
            ..Self::open("")
        }
    }

    /// A chat from before, opened again: its turns, ready for the next one.
    pub fn reopened(turns: Vec<Turn>) -> Self {
        let mut chat = Self::open("");
        let answered = turns.last().is_some_and(|t| t.who == Who::Model);
        chat.turns = turns;
        if answered {
            chat.phase = Phase::Ready;
            chat.next = default_next();
        }
        chat
    }

    pub fn spec(&self, typed: &str) -> ViewSpec {
        ViewSpec {
            name: VIEW.into(),
            label: LABEL.into(),
            enter: Some(self.enter(typed)),
            takes_input: true,
            text: Some(String::new()),
            leave_text: None,
        }
    }

    /// The first turn goes with what waited in the line.
    pub fn ask(&mut self, text: &str) -> Vec<Subject> {
        let with = std::mem::take(&mut self.with);
        self.turns.push(Turn {
            who: Who::You,
            text: text.to_string(),
            sub: crate::chat_with::sent_with(&with),
        });
        self.phase = Phase::Writing;
        self.next.clear();
        self.asks = None;
        self.acting = None;
        self.sel = 0;
        with
    }

    /// The reply so far; `done` once it's all written.
    pub fn wrote(&mut self, text: &str, done: bool) {
        if !matches!(self.phase, Phase::Writing | Phase::Ranking) {
            return;
        }
        match self.turns.last_mut() {
            Some(t) if t.who == Who::Model => t.text = text.to_string(),
            _ => self.turns.push(Turn::new(Who::Model, text)),
        }
        if done {
            self.phase = Phase::Ranking;
        }
    }

    /// The tools the model ran for the latest turn, as cards before its words.
    pub fn tools(&mut self, tools: &[Tool]) {
        let from = self
            .turns
            .iter()
            .rposition(|t| t.who == Who::You)
            .map_or(0, |i| i + 1);
        let mut rest: Vec<Turn> = self
            .turns
            .drain(from..)
            .filter(|t| t.who != Who::Tool)
            .collect();
        self.turns.extend(tools.iter().map(Tool::turn));
        self.turns.append(&mut rest);
    }

    /// Jev ranked what to do with the reply; none ranked leaves the defaults.
    pub fn ranked(&mut self, rows: Vec<Row>) {
        if self.phase != Phase::Ranking {
            return;
        }
        self.next = if rows.is_empty() {
            default_next()
        } else {
            rows
        };
        self.sel = 0;
        self.phase = Phase::Ready;
    }

    pub fn failed(&mut self, message: &str) {
        self.phase = Phase::Failed(message.to_string());
        self.next.clear();
        self.asks = None;
    }

    /// The model asks before it does something, or stopped asking.
    pub fn asked(&mut self, asks: Option<Asks>) {
        if self.asks.as_ref().map(|a| a.id) != asks.as_ref().map(|a| a.id) {
            self.sel = 0;
        }
        self.asks = asks;
    }

    pub fn needs_sign_in(&mut self, provider: Option<String>) {
        if self.sign_in != provider {
            self.sel = 0;
        }
        self.sign_in = provider;
    }

    pub fn reply(&self) -> Option<&str> {
        self.turns
            .iter()
            .rev()
            .find(|t| t.who == Who::Model)
            .map(|t| t.text.as_str())
    }

    /// The rows under the turns and the head over them: what ⌘K offers, the sign-in, the model's
    /// question, or the next steps for its reply.
    pub fn rows(&self) -> (Option<&'static str>, Vec<Row>) {
        if let Some(rows) = &self.acting {
            return (Some(ACTING_HEAD), rows.clone());
        }
        if let Some(provider) = &self.sign_in {
            return (Some(WAITING_HEAD), sign_in_rows(provider));
        }
        if let Some(asks) = &self.asks {
            return (Some(WAITING_HEAD), asks.rows());
        }
        match self.phase {
            Phase::Ready => (Some(NEXT_HEAD), self.next.clone()),
            _ => (None, Vec::new()),
        }
    }

    /// What ⌘K offers for the chat: its reply, the whole chat, another model, the chats before.
    fn actions(&self) -> Vec<Row> {
        let mut rows = Vec::new();
        if self.reply().is_some() {
            rows.push(Row::new(COPY_KEY, "Copy the reply").with_icon("copy"));
        }
        if !self.turns.is_empty() {
            rows.push(Row::new(EXPORT_KEY, "Export the chat as Markdown").with_icon("export"));
        }
        rows.extend(self.models.others().map(|m| {
            Row::new(
                crate::models::row_key(&m.key),
                format!("Use {} for this chat", m.title),
            )
            .with_sub(format!("or say “use {} for this”", m.title.to_lowercase()))
            .with_icon("sparkles")
        }));
        rows.push(Row::new(crate::past_chats::ROW_KEY, "See past chats").with_icon("history"));
        rows
    }

    /// ⌘K: the chat goes in the line, or out of it again.
    pub fn toggle_acting(&mut self) {
        self.acting = match self.acting {
            Some(_) => None,
            None => Some(self.actions()),
        };
        self.sel = 0;
    }

    pub fn let_go(&mut self) -> bool {
        self.sel = 0;
        self.acting.take().is_some()
    }

    /// What ↵ says: the words typed go to the model (or pick a model); otherwise the row under
    /// the reply or the chat's state.
    pub fn enter(&self, typed: &str) -> EnterTarget {
        let typed = typed.trim();
        if !typed.is_empty() {
            if let Some(model) = self.models.said(typed) {
                return EnterTarget::new(
                    crate::models::row_key(&model.key),
                    format!("Use {} for this chat", model.title),
                );
            }
            return EnterTarget::new("chat:say", format!("Ask the model: “{typed}”"));
        }
        let turn = self.turns.iter().filter(|t| t.who == Who::You).count();
        let (_, rows) = self.rows();
        if let Some(row) = rows.get(self.sel) {
            return EnterTarget::new(
                format!("chat:{}:{}", turn.saturating_sub(1), row.key),
                row.enter_words(),
            );
        }
        match &self.phase {
            Phase::Idle if !self.with.is_empty() => {
                EnterTarget::new("chat:idle", "Say what to ask about it")
            }
            Phase::Idle => EnterTarget::new("chat:idle", "Say what to ask the model"),
            Phase::Writing => EnterTarget::new("chat:writing", "The model is writing…"),
            Phase::Ranking => EnterTarget::new("chat:ranking", "Jev is reading the draft…"),
            Phase::Ready => EnterTarget::new("chat:idle", "Say what to change"),
            Phase::Failed(_) => EnterTarget::new("chat:failed", "Say it again to retry"),
        }
    }

    pub fn move_sel(&mut self, down: bool) -> bool {
        let n = self.rows().1.len();
        if n == 0 {
            return false;
        }
        self.sel = if down {
            (self.sel + 1).min(n - 1)
        } else {
            self.sel.saturating_sub(1)
        };
        true
    }

    /// Enter: typed words are a new turn (or pick a model); otherwise the highlighted row.
    pub fn press_enter(&mut self, typed: &str) -> ChatKey {
        let typed = typed.trim();
        if !typed.is_empty() {
            if let Some(model) = self.models.said(typed) {
                let id = model.key.clone();
                self.models.current = Some(id.clone());
                return ChatKey::UseModel(id);
            }
            let with = self.ask(typed);
            return ChatKey::Ask {
                text: typed.to_string(),
                with,
            };
        }
        let (_, rows) = self.rows();
        let Some(row) = rows.get(self.sel).cloned() else {
            return ChatKey::Nothing;
        };
        if self.acting.take().is_some() {
            self.sel = 0;
            if let Some(id) = crate::models::id_of(&row.key) {
                self.models.current = Some(id.to_string());
                return ChatKey::UseModel(id.to_string());
            }
        } else if self.sign_in.is_some() {
            return ChatKey::SignIn(row.key == SIGN_IN_KEY);
        } else if let Some(asks) = &self.asks {
            return ChatKey::Answer {
                id: asks.id,
                choice: asks.choice_of(&row.key),
            };
        }
        match row.key.as_str() {
            crate::past_chats::ROW_KEY => ChatKey::PastChats,
            SHORTER_KEY => {
                self.ask("shorter");
                ChatKey::Ask {
                    text: "shorter".into(),
                    with: Vec::new(),
                }
            }
            _ => ChatKey::Step(Box::new(row)),
        }
    }

    /// ⌘2: no to what the model asked, or not now to signing in.
    pub fn say_no(&self) -> ChatKey {
        if self.sign_in.is_some() {
            return ChatKey::SignIn(false);
        }
        match &self.asks {
            Some(asks) => ChatKey::Answer {
                id: asks.id,
                choice: asks.deny(),
            },
            None => ChatKey::Nothing,
        }
    }
}

const SIGN_IN_KEY: &str = "chat:sign-in";

fn sign_in_rows(provider: &str) -> Vec<Row> {
    vec![
        Row::new(SIGN_IN_KEY, format!("Sign in to {provider}"))
            .with_sub("Opens the sign-in page once; it stays signed in")
            .with_icon("key"),
        Row::new("chat:not-now", "Not now")
            .with_sub("⌘2 · the chat waits for your next question")
            .with_icon("close"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asks::Choice;
    use crate::stage::SubjectKind;

    #[test]
    fn a_sentence_opens_the_chat_writing_its_first_turn() {
        let chat = Chat::open("brainstorm names for the launch party");
        assert_eq!(chat.turns.len(), 1);
        assert_eq!(chat.enter("").label, "The model is writing…");
        let quick = Chat::open("  ");
        assert!(quick.turns.is_empty());
        assert_eq!(quick.enter("").id, "chat:idle");
    }

    #[test]
    fn enter_follows_the_reply_then_the_next_step_then_what_you_type() {
        let mut chat = Chat::open("write it for me");
        chat.wrote("Hi Sarah,", false);
        assert_eq!(chat.enter("").label, "The model is writing…");
        chat.wrote("Hi Sarah, beta Oct 14.", true);
        assert_eq!(chat.enter("").label, "Jev is reading the draft…");
        chat.ranked(vec![Row::new("use", "Use it as the reply to Sarah")]);
        assert_eq!(
            chat.enter(""),
            EnterTarget::new("chat:0:use", "Use it as the reply to Sarah")
        );
        assert_eq!(chat.enter("shorter").label, "Ask the model: “shorter”");
        assert_eq!(
            chat.press_enter("shorter"),
            ChatKey::Ask {
                text: "shorter".into(),
                with: Vec::new()
            }
        );
        assert_eq!(chat.enter("").label, "The model is writing…");
        assert_eq!(chat.reply(), Some("Hi Sarah, beta Oct 14."));
    }

    #[test]
    fn with_nothing_ranked_the_reply_offers_copy_note_shorter_and_past_chats() {
        let mut chat = Chat::open("brainstorm");
        chat.wrote("Names", true);
        chat.ranked(Vec::new());
        assert_eq!(chat.enter("").label, "Copy it");
        assert!(chat.move_sel(true));
        assert!(chat.move_sel(true));
        assert_eq!(chat.enter("").label, "Make it shorter");
        assert!(chat.move_sel(true));
        assert_eq!(chat.press_enter(""), ChatKey::PastChats);
    }

    #[test]
    fn a_failure_is_said_and_typing_retries() {
        let mut chat = Chat::open("hello");
        chat.failed("The model couldn't start");
        assert_eq!(chat.enter("").id, "chat:failed");
        assert_eq!(chat.press_enter(""), ChatKey::Nothing);
        assert!(matches!(chat.press_enter("hello"), ChatKey::Ask { text, .. } if text == "hello"));
    }

    #[test]
    fn what_the_chat_is_about_goes_with_the_first_turn_only() {
        let error = Subject {
            kind: SubjectKind::Thing,
            title: "Script error in deploy.ts".into(),
            row_key: Some("ctx:0".into()),
            ..Subject::default()
        };
        let mut chat = Chat::open_with(vec![error.clone()]);
        assert_eq!(chat.enter("").label, "Say what to ask about it");
        let ChatKey::Ask { with, .. } = chat.press_enter("why did it fail") else {
            panic!("a turn");
        };
        assert_eq!(with, vec![error]);
        assert_eq!(
            chat.turns[0].sub.as_deref(),
            Some("With Script error in deploy.ts")
        );
        assert!(chat.with.is_empty());
        chat.wrote("A missing token.", true);
        chat.ranked(Vec::new());
        let ChatKey::Ask { with, .. } = chat.press_enter("fix it") else {
            panic!("a turn");
        };
        assert!(with.is_empty(), "context goes once");
    }

    #[test]
    fn tool_cards_sit_before_the_models_words_in_its_reply() {
        let mut chat = Chat::open("what's in the readme");
        chat.wrote("It says", false);
        chat.tools(&[Tool {
            title: "Read file".into(),
            subject: Some("README.md".into()),
            state: ToolState::Running,
        }]);
        chat.tools(&[Tool {
            title: "Read file".into(),
            subject: Some("README.md".into()),
            state: ToolState::Done,
        }]);
        let who: Vec<Who> = chat.turns.iter().map(|t| t.who).collect();
        assert_eq!(who, [Who::You, Who::Tool, Who::Model]);
        assert_eq!(chat.turns[1].sub.as_deref(), Some("Done · README.md"));
    }

    #[test]
    fn the_models_question_is_rows_enter_allows_and_cmd_2_denies() {
        let mut chat = Chat::open("clean up the build folder");
        chat.asked(Some(Asks {
            id: 7,
            title: "Run rm -rf build".into(),
            detail: None,
            choices: vec![
                Choice::new("deny", "Reject", false),
                Choice::new("once", "Allow once", true),
            ],
        }));
        assert_eq!(chat.enter("").label, "Allow: Run rm -rf build");
        assert_eq!(
            chat.press_enter(""),
            ChatKey::Answer {
                id: 7,
                choice: Some("once".into())
            }
        );
        assert_eq!(
            chat.say_no(),
            ChatKey::Answer {
                id: 7,
                choice: Some("deny".into())
            }
        );
    }

    #[test]
    fn signing_in_is_a_row_that_waits_for_you() {
        let mut chat = Chat::open("hello");
        chat.needs_sign_in(Some("ChatGPT".into()));
        assert_eq!(chat.enter("").label, "Sign in to ChatGPT");
        assert_eq!(chat.press_enter(""), ChatKey::SignIn(true));
        assert_eq!(chat.say_no(), ChatKey::SignIn(false));
    }

    #[test]
    fn cmd_k_lists_what_to_do_with_the_chat_and_a_model_you_can_say() {
        let mut chat = Chat::open("hello");
        chat.models = Models {
            list: vec![
                Row::new("anthropic/sonnet", "Claude Sonnet"),
                Row::new("anthropic/opus", "Claude Opus"),
            ],
            current: Some("anthropic/sonnet".into()),
        };
        chat.wrote("Hi", true);
        chat.ranked(Vec::new());
        assert_eq!(
            chat.enter("use opus for this").label,
            "Use Claude Opus for this chat"
        );
        chat.toggle_acting();
        let (head, rows) = chat.rows();
        assert_eq!(head, Some(ACTING_HEAD));
        let titles: Vec<&str> = rows.iter().map(|r| r.title.as_str()).collect();
        assert_eq!(
            titles,
            [
                "Copy the reply",
                "Export the chat as Markdown",
                "Use Claude Opus for this chat",
                "See past chats"
            ]
        );
        chat.move_sel(true);
        chat.move_sel(true);
        assert_eq!(
            chat.press_enter(""),
            ChatKey::UseModel("anthropic/opus".into())
        );
        assert!(chat.acting.is_none());
        assert_eq!(chat.models.current.as_deref(), Some("anthropic/opus"));
    }
}
