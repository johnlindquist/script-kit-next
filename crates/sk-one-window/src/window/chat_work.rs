//! The model's chat as the window runs it past its first turn: every way in, what the model is
//! doing and asking, what ↵ and ⌘2 answer, and the chats from before.

use super::*;
use crate::chat::{self, Chat, ChatKey};
use crate::past_chats::{self, PastChats, PastKey};
use crate::work::{WorkEffect, WorkEvent};

fn ask_effect(text: String, with: Vec<Subject>) -> Effect {
    Effect::Work(if with.is_empty() {
        WorkEffect::AskModel { text }
    } else {
        WorkEffect::AskModelWith { text, with }
    })
}

impl OneWindow {
    /// The chats from before, while their view is on top.
    pub fn past_chats(&self) -> Option<&PastChats> {
        self.work
            .past
            .as_ref()
            .filter(|_| self.views.top().is_some_and(|f| f.name == past_chats::VIEW))
    }

    /// The line above the input for the chat or past chats on top, when they have one.
    pub(super) fn work_line(&self) -> Option<SlotLine> {
        match self.views.top()?.name.as_str() {
            chat::VIEW => self.work.chat.as_ref().and_then(crate::chat_with::line),
            past_chats::VIEW => self.work.past.as_ref().and_then(PastChats::line),
            _ => None,
        }
    }

    /// Esc lets go of what ⌘K put in the line before it goes back.
    pub(super) fn let_go_in_view(&mut self) -> bool {
        match self.views.top().map(|f| f.name.as_str()) {
            Some(chat::VIEW) => self.work.chat.as_mut().is_some_and(Chat::let_go),
            Some(past_chats::VIEW) => self.work.past.as_mut().is_some_and(PastChats::let_go),
            _ => false,
        }
    }

    pub(super) fn on_chat_work(&mut self, now: Ms, ev: WorkEvent, fx: &mut Vec<Effect>) -> bool {
        match ev {
            WorkEvent::OpenChat { text, send, with } => {
                self.open_chat_about(now, text.trim(), send, with, fx);
                true
            }
            WorkEvent::ModelTools(tools) => self.with_chat(|c| c.tools(&tools)),
            WorkEvent::ModelAsks(asks) => self.with_chat(|c| c.asked(asks)),
            WorkEvent::ModelSignIn(provider) => self.with_chat(|c| c.needs_sign_in(provider)),
            WorkEvent::Models(models) => self.with_chat(|c| c.models = models),
            WorkEvent::OpenPastChats { query } => {
                self.open_past_chats(now, &query, fx);
                true
            }
            WorkEvent::PastChats(chats) => match self.work.past.as_mut() {
                Some(p) => {
                    p.listed(chats);
                    true
                }
                None => false,
            },
            WorkEvent::ChatReopened { id, turns } => self.chat_reopened(now, &id, turns, fx),
            WorkEvent::PastChatGone { id } => match self.work.past.as_mut() {
                Some(p) => {
                    p.removed(&id);
                    true
                }
                None => false,
            },
            _ => false,
        }
    }

    /// The chat already on top takes what comes in; otherwise a new chat opens about it. Words
    /// that don't go now wait in the input.
    fn open_chat_about(
        &mut self,
        now: Ms,
        text: &str,
        send: bool,
        with: Vec<Subject>,
        fx: &mut Vec<Effect>,
    ) {
        let on_top = self.views.top().is_some_and(|f| f.name == chat::VIEW);
        match self.work.chat.as_mut().filter(|_| on_top) {
            Some(c) => {
                c.with.extend(with);
                if !send && !text.is_empty() {
                    fx.push(Effect::SetInput(text.to_string()));
                    self.text = text.to_string();
                }
            }
            None => {
                let chat = Chat::open_with(with);
                let mut spec = chat.spec("");
                if !send {
                    spec.text = Some(text.to_string());
                }
                self.work.chat = Some(chat);
                self.on_view(now, ViewOp::Open(spec), fx);
            }
        }
        if send && !text.is_empty() {
            let with = self
                .work
                .chat
                .as_mut()
                .map(|c| c.ask(text))
                .unwrap_or_default();
            fx.push(ask_effect(text.to_string(), with));
        }
    }

    /// What ↵ or ⌘2 did in the chat, carried out.
    pub(super) fn chat_pressed(&mut self, now: Ms, pressed: ChatKey, fx: &mut Vec<Effect>) -> bool {
        match pressed {
            ChatKey::Nothing | ChatKey::Moved => false,
            ChatKey::Ask { text, with } => {
                fx.push(ask_effect(text, with));
                self.clear_view_input(now, fx);
                true
            }
            ChatKey::Step(row) => {
                let reply = self
                    .work
                    .chat
                    .as_ref()
                    .and_then(Chat::reply)
                    .unwrap_or_default()
                    .to_string();
                fx.push(Effect::Work(WorkEffect::ChatStep { row, reply }));
                true
            }
            ChatKey::Answer { id, choice } => {
                self.with_chat(|c| c.asked(None));
                fx.push(Effect::Work(WorkEffect::AnswerModel { id, choice }));
                true
            }
            ChatKey::SignIn(approve) => {
                self.with_chat(|c| c.needs_sign_in(None));
                fx.push(Effect::Work(WorkEffect::ModelSignIn { approve }));
                true
            }
            ChatKey::UseModel(id) => {
                fx.push(Effect::Work(WorkEffect::UseModel { id }));
                self.clear_view_input(now, fx);
                true
            }
            ChatKey::PastChats => {
                self.open_past_chats(now, "", fx);
                true
            }
        }
    }

    fn open_past_chats(&mut self, now: Ms, query: &str, fx: &mut Vec<Effect>) {
        let past = PastChats::open(query);
        let spec = past.spec();
        self.work.past = Some(past);
        self.on_view(now, ViewOp::Open(spec), fx);
        fx.push(Effect::Work(WorkEffect::ListPastChats));
    }

    pub(super) fn past_chats_key(&mut self, key: Key, fx: &mut Vec<Effect>) -> bool {
        let Some(p) = self.work.past.as_mut() else {
            return false;
        };
        match key {
            Key::Up | Key::Down => p.move_sel(key == Key::Down),
            Key::CmdK => p.stage(),
            Key::Enter => match p.press_enter() {
                PastKey::Nothing => false,
                PastKey::Reopen(id) => {
                    fx.push(Effect::Work(WorkEffect::ReopenChat { id }));
                    true
                }
                PastKey::Act { id, act } => {
                    fx.push(Effect::Work(WorkEffect::PastChat { id, act }));
                    true
                }
            },
            _ => false,
        }
    }

    /// The chat Enter asked for is open again: it takes the place of the past chats, or of the
    /// chat they were opened from.
    fn chat_reopened(
        &mut self,
        now: Ms,
        id: &str,
        turns: Vec<chat::Turn>,
        fx: &mut Vec<Effect>,
    ) -> bool {
        let Some(p) = self.work.past.as_mut() else {
            return false;
        };
        if p.opening.as_deref() != Some(id) {
            return false;
        }
        self.work.past = None;
        let chat = Chat::reopened(turns);
        let spec = chat.spec("");
        self.work.chat = Some(chat);
        if self.views.below_named(chat::VIEW) {
            self.on_view(now, ViewOp::Back, fx);
        } else {
            self.on_view(now, ViewOp::Replace(spec), fx);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asks::{Asks, Choice};
    use crate::chat::{Turn, Who};
    use crate::past_chats::PastChat;

    fn work(win: &mut OneWindow, now: Ms, ev: WorkEvent) -> Vec<Effect> {
        win.handle(now, Event::Work(ev)).effects
    }

    fn key(win: &mut OneWindow, now: Ms, key: Key) -> Vec<Effect> {
        let mut fx = win.handle(now, Event::Key(key)).effects;
        let handed: Vec<Key> = fx
            .iter()
            .filter_map(|e| match e {
                Effect::ViewKey(k) => Some(k.clone()),
                _ => None,
            })
            .collect();
        for k in handed {
            fx.extend(work(win, now, WorkEvent::ViewKey(k)));
        }
        fx
    }

    fn work_fx(fx: &[Effect]) -> Vec<WorkEffect> {
        fx.iter()
            .filter_map(|e| match e {
                Effect::Work(w) => Some(w.clone()),
                _ => None,
            })
            .collect()
    }

    fn error() -> Subject {
        Subject {
            kind: SubjectKind::Thing,
            label: crate::chat_with::ASKING_ABOUT.into(),
            title: "deploy.ts: token missing".into(),
            sub: "Script error".into(),
            icon: "warning".into(),
            row_key: Some("ctx:0".into()),
            ..Subject::default()
        }
    }

    #[test]
    fn a_script_error_waits_in_the_line_and_goes_with_the_first_turn() {
        let mut win = OneWindow::default();
        work(
            &mut win,
            0,
            WorkEvent::OpenChat {
                text: String::new(),
                send: false,
                with: vec![error()],
            },
        );
        assert_eq!(win.views().top().map(|f| f.name.as_str()), Some(chat::VIEW));
        let line = win.slot_line(0);
        assert_eq!(
            (line.label.as_str(), line.title.as_str()),
            (crate::chat_with::ASKING_ABOUT, "deploy.ts: token missing")
        );
        win.handle(450, Event::Text("why did it fail".into()));
        let fx = key(&mut win, 500, Key::Enter);
        assert_eq!(
            work_fx(&fx),
            [WorkEffect::AskModelWith {
                text: "why did it fail".into(),
                with: vec![error()]
            }]
        );
        assert_ne!(
            win.slot_line(500).label,
            crate::chat_with::ASKING_ABOUT,
            "it went with the turn"
        );
    }

    #[test]
    fn a_handoff_that_sends_asks_at_once_and_one_that_doesnt_waits_in_the_input() {
        let mut win = OneWindow::default();
        let fx = work(
            &mut win,
            0,
            WorkEvent::OpenChat {
                text: "summarize this".into(),
                send: true,
                with: vec![error()],
            },
        );
        assert!(matches!(
            work_fx(&fx).as_slice(),
            [WorkEffect::AskModelWith { text, with }] if text == "summarize this" && with.len() == 1
        ));
        assert_eq!(win.text(), "");

        let mut win = OneWindow::default();
        let fx = work(
            &mut win,
            0,
            WorkEvent::OpenChat {
                text: "draft a reply".into(),
                send: false,
                with: Vec::new(),
            },
        );
        assert!(work_fx(&fx).is_empty());
        assert_eq!(win.text(), "draft a reply");
        assert_eq!(win.enter_target(0).label, "Ask the model: “draft a reply”");
    }

    #[test]
    fn a_permission_request_is_answered_with_enter_after_the_guard() {
        let mut win = OneWindow::default();
        work(
            &mut win,
            0,
            WorkEvent::AskModel {
                sentence: "clean the build folder".into(),
            },
        );
        work(
            &mut win,
            100,
            WorkEvent::ModelAsks(Some(Asks {
                id: 3,
                title: "Run rm -rf build".into(),
                detail: None,
                choices: vec![
                    Choice::new("once", "Allow once", true),
                    Choice::new("no", "Reject", false),
                ],
            })),
        );
        assert_eq!(win.enter_target(100).label, "Allow: Run rm -rf build");
        assert!(win.enter_armed(100), "a request arriving guards Enter");
        assert!(work_fx(&key(&mut win, 200, Key::Enter)).is_empty());
        let fx = key(&mut win, 600, Key::Enter);
        assert_eq!(
            work_fx(&fx),
            [WorkEffect::AnswerModel {
                id: 3,
                choice: Some("once".into())
            }]
        );
        assert_eq!(win.enter_target(600).label, "The model is writing…");
    }

    #[test]
    fn cmd_2_denies_what_the_model_asks() {
        let mut win = OneWindow::default();
        work(
            &mut win,
            0,
            WorkEvent::AskModel {
                sentence: "x".into(),
            },
        );
        work(
            &mut win,
            10,
            WorkEvent::ModelAsks(Some(Asks {
                id: 4,
                title: "Edit a file".into(),
                detail: None,
                choices: vec![Choice::new("no", "Reject", false)],
            })),
        );
        let fx = key(&mut win, 500, Key::CmdDigit(2));
        assert_eq!(
            work_fx(&fx),
            [WorkEffect::AnswerModel {
                id: 4,
                choice: Some("no".into())
            }]
        );
    }

    #[test]
    fn past_chats_open_filter_stage_and_reopen_in_place_of_the_chat() {
        let mut win = OneWindow::default();
        work(
            &mut win,
            0,
            WorkEvent::AskModel {
                sentence: "hi".into(),
            },
        );
        work(
            &mut win,
            10,
            WorkEvent::ModelWrote {
                text: "Hello".into(),
                done: true,
            },
        );
        work(&mut win, 20, WorkEvent::ModelNext(Vec::new()));
        for _ in 0..3 {
            key(&mut win, 500, Key::Down);
        }
        assert_eq!(win.enter_target(500).label, "See past chats");
        let fx = key(&mut win, 600, Key::Enter);
        assert_eq!(work_fx(&fx), [WorkEffect::ListPastChats]);
        assert_eq!(win.views().crumb(), "The model › Past chats");
        work(
            &mut win,
            700,
            WorkEvent::PastChats(vec![
                PastChat {
                    id: "b".into(),
                    title: "Launch party".into(),
                    when: "Today".into(),
                    preview: String::new(),
                    search: String::new(),
                },
                PastChat {
                    id: "a".into(),
                    title: "Deploy fix".into(),
                    when: "Yesterday".into(),
                    preview: String::new(),
                    search: String::new(),
                },
            ]),
        );
        win.handle(800, Event::Text("deploy".into()));
        assert_eq!(win.enter_target(800).label, "Open “Deploy fix”");
        key(&mut win, 900, Key::CmdK);
        assert_eq!(win.slot_line(900).title, "Deploy fix");
        key(&mut win, 950, Key::Escape);
        assert_eq!(
            win.views().crumb(),
            "The model › Past chats",
            "esc lets go first"
        );
        assert!(
            work_fx(&key(&mut win, 1_000, Key::Enter)).is_empty(),
            "the list arrived on its own"
        );
        let fx = key(&mut win, 1_200, Key::Enter);
        assert_eq!(work_fx(&fx), [WorkEffect::ReopenChat { id: "a".into() }]);
        work(
            &mut win,
            1_300,
            WorkEvent::ChatReopened {
                id: "a".into(),
                turns: vec![Turn::new(Who::You, "fix it"), Turn::new(Who::Model, "Done")],
            },
        );
        assert_eq!(win.views().crumb(), "The model");
        assert_eq!(win.chat().map(|c| c.turns.len()), Some(2));
        assert_eq!(win.enter_target(1_300).label, "Copy it");
    }

    #[test]
    fn asking_for_past_chats_opens_them_with_the_words_asked_about() {
        let mut win = OneWindow::default();
        let fx = work(
            &mut win,
            0,
            WorkEvent::OpenPastChats {
                query: "launch".into(),
            },
        );
        assert_eq!(work_fx(&fx), [WorkEffect::ListPastChats]);
        assert_eq!(win.text(), "launch");
        assert_eq!(win.views().crumb(), past_chats::LABEL);
        assert_eq!(win.enter_target(0).label, "Reading past chats…");
    }
}
