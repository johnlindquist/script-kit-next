//! The model's chat, under the same input: the input is the chat's input, each reply is tagged as
//! written by the model, and under the latest reply are the next steps Jev ranks for it. Esc goes
//! back one step and restores the sentence the chat came from.

use crate::enter::EnterTarget;
use crate::row::Row;
use crate::window::ViewSpec;

pub const VIEW: &str = "chat";
pub const LABEL: &str = "The model";
/// The list's head under the latest reply.
pub const NEXT_HEAD: &str = "What to do with it · Jev ranks, you pick";
pub const WRITTEN_BY: &str = "Written by the model";
/// The key of the next step that asks the model for a shorter reply.
pub const SHORTER_KEY: &str = "chat:shorter";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Who {
    You,
    Model,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Turn {
    pub who: Who,
    pub text: String,
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
    /// A new turn for the model.
    Ask(String),
    /// A next step for the host to do with the latest reply.
    Step(Box<Row>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chat {
    pub turns: Vec<Turn>,
    pub phase: Phase,
    pub next: Vec<Row>,
    pub sel: usize,
}

/// Next steps for a reply when Jev hasn't ranked any.
pub fn default_next() -> Vec<Row> {
    vec![
        Row::new("chat:copy", "Copy it").with_icon("copy"),
        Row::new("chat:note", "Add it to today's note").with_icon("append"),
        Row::new(SHORTER_KEY, "Make it shorter").with_icon("edit"),
    ]
}

impl Chat {
    /// Opens on the words it was asked, or empty for a quick question.
    pub fn open(sentence: &str) -> Self {
        let sentence = sentence.trim();
        let mut chat = Self {
            turns: Vec::new(),
            phase: Phase::Idle,
            next: Vec::new(),
            sel: 0,
        };
        if !sentence.is_empty() {
            chat.ask(sentence);
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

    fn ask(&mut self, text: &str) {
        self.turns.push(Turn {
            who: Who::You,
            text: text.to_string(),
        });
        self.phase = Phase::Writing;
        self.next.clear();
        self.sel = 0;
    }

    /// The reply so far; `done` once it's all written.
    pub fn wrote(&mut self, text: &str, done: bool) {
        if !matches!(self.phase, Phase::Writing | Phase::Ranking) {
            return;
        }
        match self.turns.last_mut() {
            Some(t) if t.who == Who::Model => t.text = text.to_string(),
            _ => self.turns.push(Turn {
                who: Who::Model,
                text: text.to_string(),
            }),
        }
        if done {
            self.phase = Phase::Ranking;
        }
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
    }

    pub fn reply(&self) -> Option<&str> {
        self.turns
            .iter()
            .rev()
            .find(|t| t.who == Who::Model)
            .map(|t| t.text.as_str())
    }

    /// What ↵ says: the words typed go to the model; otherwise the chat's state or the next step.
    pub fn enter(&self, typed: &str) -> EnterTarget {
        let typed = typed.trim();
        if !typed.is_empty() {
            return EnterTarget::new("chat:say", format!("Ask the model: “{typed}”"));
        }
        let turn = self.turns.iter().filter(|t| t.who == Who::You).count();
        match &self.phase {
            Phase::Idle => EnterTarget::new("chat:idle", "Say what to ask the model"),
            Phase::Writing => EnterTarget::new("chat:writing", "The model is writing…"),
            Phase::Ranking => EnterTarget::new("chat:ranking", "Jev is reading the draft…"),
            Phase::Ready => match self.next.get(self.sel) {
                Some(row) => EnterTarget::new(
                    format!("chat:{}:{}", turn.saturating_sub(1), row.key),
                    row.enter_words(),
                ),
                None => EnterTarget::new("chat:idle", "Say what to change"),
            },
            Phase::Failed(_) => EnterTarget::new("chat:failed", "Say it again to retry"),
        }
    }

    pub fn move_sel(&mut self, down: bool) -> bool {
        if self.phase != Phase::Ready || self.next.is_empty() {
            return false;
        }
        let n = self.next.len();
        self.sel = if down {
            (self.sel + 1).min(n - 1)
        } else {
            self.sel.saturating_sub(1)
        };
        true
    }

    /// Enter: typed words are a new turn; otherwise the highlighted next step.
    pub fn press_enter(&mut self, typed: &str) -> ChatKey {
        let typed = typed.trim();
        if !typed.is_empty() {
            self.ask(typed);
            return ChatKey::Ask(typed.to_string());
        }
        if self.phase != Phase::Ready {
            return ChatKey::Nothing;
        }
        match self.next.get(self.sel).cloned() {
            Some(row) if row.key == SHORTER_KEY => {
                self.ask("shorter");
                ChatKey::Ask("shorter".into())
            }
            Some(row) => ChatKey::Step(Box::new(row)),
            None => ChatKey::Nothing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(chat.press_enter("shorter"), ChatKey::Ask("shorter".into()));
        assert_eq!(chat.enter("").label, "The model is writing…");
        assert_eq!(chat.reply(), Some("Hi Sarah, beta Oct 14."));
    }

    #[test]
    fn with_nothing_ranked_the_reply_offers_copy_note_and_shorter() {
        let mut chat = Chat::open("brainstorm");
        chat.wrote("Names", true);
        chat.ranked(Vec::new());
        assert_eq!(chat.enter("").label, "Copy it");
        assert!(chat.move_sel(true));
        assert!(chat.move_sel(true));
        assert_eq!(chat.enter("").label, "Make it shorter");
        assert_eq!(chat.press_enter(""), ChatKey::Ask("shorter".into()));
    }

    #[test]
    fn a_failure_is_said_and_typing_retries() {
        let mut chat = Chat::open("hello");
        chat.failed("The model couldn't start");
        assert_eq!(chat.enter("").id, "chat:failed");
        assert_eq!(chat.press_enter(""), ChatKey::Nothing);
        assert_eq!(chat.press_enter("hello"), ChatKey::Ask("hello".into()));
    }
}
