//! The work views as the window runs them: the model's chat, steps, the connect view and agents,
//! their keys, and the strip.

use super::*;
use crate::chat::{self, Chat};
use crate::connect::{self, Connecting, Opener};
use crate::past_chats::{self, PastChats};
use crate::steps::{self, Steps, StepsKey};
use crate::strip::{self, Task};
use crate::work::{Work, WorkEffect, WorkEvent};

const WORK_VIEWS: [&str; 5] = [
    chat::VIEW,
    steps::VIEW,
    connect::VIEW,
    strip::AGENT_VIEW,
    past_chats::VIEW,
];

impl OneWindow {
    pub fn work(&self) -> &Work {
        &self.work
    }

    /// The model's chat, while its view is open.
    pub fn chat(&self) -> Option<&Chat> {
        self.work
            .chat
            .as_ref()
            .filter(|_| self.has_view(chat::VIEW))
    }

    /// The steps, while their view is open.
    pub fn steps_view(&self) -> Option<&Steps> {
        self.work
            .steps
            .as_ref()
            .filter(|_| self.has_view(steps::VIEW))
    }

    /// The service being connected, while the connect view is open.
    pub fn connecting(&self) -> Option<&Connecting> {
        self.work
            .connecting
            .as_ref()
            .filter(|_| self.has_view(connect::VIEW))
    }

    /// The task whose view is open.
    pub fn open_task(&self) -> Option<&Task> {
        self.work
            .agent
            .as_deref()
            .filter(|_| self.has_view(strip::AGENT_VIEW))
            .and_then(|id| self.work.strip.get(id))
    }

    fn has_view(&self, name: &str) -> bool {
        self.views.top().is_some_and(|f| f.name == name) || self.views.below_named(name)
    }

    /// Rows as the list shows them: a row that needs a service says so.
    pub(super) fn with_needs(&self, rows: Vec<Row>) -> Vec<Row> {
        rows.into_iter()
            .map(|r| connect::decorate(r, &self.work.connections))
            .collect()
    }

    /// A step of "Do all" ran. Once every step has, the run leaves its line: only the steps that
    /// did what they said are counted, and a run with a failed step says so with its icon.
    fn step_settled(&mut self, now: Ms, index: usize, failed: bool, fx: &mut Vec<Effect>) -> bool {
        let Some(s) = self.work.steps.as_mut() else {
            return false;
        };
        if failed {
            s.step_failed(index);
        } else {
            s.step_done(index);
        }
        if s.all_done() {
            let title = s.done_title();
            let failed = s.any_failed();
            self.steps = Some((s.sentence.clone(), s.texts()));
            self.work.steps = None;
            self.done(
                now,
                Done {
                    icon: if failed {
                        crate::contract::FAILED_ICON.into()
                    } else {
                        "steps".into()
                    },
                    title,
                    ..Done::default()
                },
                fx,
            );
        }
        true
    }

    pub(super) fn on_work(&mut self, now: Ms, ev: WorkEvent, fx: &mut Vec<Effect>) -> bool {
        let user = self.user_turn(now);
        let taken = match ev {
            WorkEvent::ViewKey(key) => self.work_view_key(now, key, fx),
            WorkEvent::AskModel { sentence } => {
                self.open_chat(now, &sentence, fx);
                true
            }
            WorkEvent::ModelWrote { text, done } => self.with_chat(|c| c.wrote(&text, done)),
            WorkEvent::ModelNext(rows) => self.with_chat(|c| c.ranked(rows)),
            WorkEvent::ModelEndedEmpty { stopped } => self.with_chat(|c| { c.phase = chat::Phase::EndedEmpty { stopped }; }),
            WorkEvent::ModelFailed { message } => self.with_chat(|c| c.failed(&message)),
            WorkEvent::OpenSteps { sentence } => {
                let open = Steps::open(&sentence);
                let spec = ViewSpec {
                    name: steps::VIEW.into(),
                    label: steps::LABEL.into(),
                    enter: Some(open.enter(&self.work.connections)),
                    takes_input: true,
                    text: None,
                    leave_text: None,
                };
                self.work.steps = Some(open);
                self.on_view(now, ViewOp::Open(spec), fx);
                fx.push(Effect::Work(WorkEffect::ReadSteps { sentence }));
                true
            }
            WorkEvent::StepsRead { sentence, steps } => {
                let Some(s) = self.work.steps.as_mut() else {
                    return false;
                };
                if s.read(&sentence, steps) {
                    self.steps_enter(now, fx);
                }
                true
            }
            WorkEvent::StepDone { index } => self.step_settled(now, index, false, fx),
            WorkEvent::StepFailed { index } => self.step_settled(now, index, true, fx),
            WorkEvent::Connections(keys) => {
                for k in keys {
                    self.work.connections.add(&k);
                }
                true
            }
            WorkEvent::Connected { integration } => self.connected(now, &integration, fx),
            WorkEvent::ConnectFailed {
                integration,
                message,
            } => match self.work.connecting.as_mut() {
                Some(c) if c.integration == integration => {
                    c.phase = connect::Phase::Ask;
                    c.error = Some(message);
                    true
                }
                _ => false,
            },
            WorkEvent::Agent { task } => {
                let id = self.work.strip.next_id();
                let t = Task::agent(&id, &task);
                let spec = t.spec("");
                self.work.strip.start(t);
                self.work.agent = Some(id.clone());
                self.on_view(now, ViewOp::Open(spec), fx);
                fx.push(Effect::Work(WorkEffect::StartAgent { id, task }));
                true
            }
            WorkEvent::TaskChanged {
                id,
                status,
                progress,
                state,
                approve,
            } => self
                .work
                .strip
                .update(&id, status, progress, state, approve),
            WorkEvent::Background { task } => {
                let id = self.work.strip.next_id();
                self.work.strip.start(Task::agent(&id, &task));
                fx.push(Effect::Work(WorkEffect::StartAgent { id, task }));
                true
            }
            WorkEvent::TaskEnded { id } => {
                if self.work.agent.as_deref() == Some(id.as_str()) {
                    self.work.agent = None;
                }
                self.work.strip.end(&id).is_some()
            }
            ev => self.on_chat_work(now, ev, fx),
        };
        self.sync_work_enter(now, user);
        taken
    }

    pub(super) fn with_chat(&mut self, f: impl FnOnce(&mut Chat)) -> bool {
        match self.work.chat.as_mut() {
            Some(c) => {
                f(c);
                true
            }
            None => false,
        }
    }

    /// Keys the window itself handles for work, before anything else sees them: Esc parks an
    /// agent's view into the strip, ⌘J opens what needs you, and ⌘↵ with nothing typed asks the
    /// model.
    pub(super) fn work_key(&mut self, now: Ms, key: &Key, fx: &mut Vec<Effect>) -> Option<bool> {
        if *key == Key::Escape && self.let_go_in_view() {
            self.sync_work_enter(now, true);
            return Some(true);
        }
        if self.in_view() {
            let parks = *key == Key::Escape
                && self.views.depth() == 1
                && self
                    .views
                    .top()
                    .is_some_and(|f| f.name == strip::AGENT_VIEW);
            let id = self
                .work
                .agent
                .clone()
                .filter(|id| parks && self.work.strip.get(id).is_some())?;
            self.work.agent = None;
            self.on_view(now, ViewOp::Park, fx);
            fx.push(Effect::Work(WorkEffect::Parked { id }));
            return Some(true);
        }
        if self.away {
            return None;
        }
        match key {
            Key::CmdJ => {
                let task = self.work.strip.target()?.clone();
                self.work.agent = Some(task.id.clone());
                self.on_view(now, ViewOp::Reopen(task.spec(&self.text)), fx);
                fx.push(Effect::Work(WorkEffect::Reopened { id: task.id }));
                Some(true)
            }
            Key::CmdEnter if self.text.trim().is_empty() && self.staged.is_none() => {
                self.open_chat(now, "", fx);
                Some(true)
            }
            _ => None,
        }
    }

    /// Enter on a row that needs a service opens the connect view instead of running it.
    pub(super) fn work_run(&mut self, now: Ms, row: &Row, fx: &mut Vec<Effect>) -> bool {
        if row.key == strip::OWNER_ROW {
            let task = self.text.trim().to_string();
            return self.on_work(now, WorkEvent::Agent { task }, fx);
        }
        let Some(key) = connect::waits_on(row, &self.work.connections).map(str::to_string) else {
            return false;
        };
        let does = row.fill.clone().unwrap_or_else(|| row.title.clone());
        let said = self.text.clone();
        self.open_connect(
            now,
            Connecting::new(&key, &said, &does, Opener::Row(Box::new(row.clone()))),
            fx,
        );
        true
    }

    /// Typing in a work view: new words for the steps read them again; the chat and agent say
    /// what Enter will send.
    pub(super) fn work_text(&mut self, now: Ms, fx: &mut Vec<Effect>) {
        if self.views.top().is_some_and(|f| f.name == chat::VIEW) {
            if let Some(c) = self.work.chat.as_mut() {
                c.filter(&self.text);
            }
        }
        if self.views.top().is_some_and(|f| f.name == past_chats::VIEW) {
            if let Some(p) = self.work.past.as_mut() {
                p.filter(&self.text);
            }
        }
        if self.views.top().is_some_and(|f| f.name == steps::VIEW) {
            if let Some(s) = self.work.steps.as_mut() {
                if s.edit(&self.text) {
                    fx.push(Effect::Work(WorkEffect::ReadSteps {
                        sentence: self.text.clone(),
                    }));
                }
            }
        }
        self.sync_work_enter(now, true);
    }

    fn open_chat(&mut self, now: Ms, sentence: &str, fx: &mut Vec<Effect>) {
        let chat = Chat::open(sentence);
        let spec = chat.spec("");
        self.work.chat = Some(chat);
        self.on_view(now, ViewOp::Open(spec), fx);
        if !sentence.trim().is_empty() {
            fx.push(Effect::Work(WorkEffect::AskModel {
                text: sentence.trim().to_string(),
            }));
        }
    }

    fn open_connect(&mut self, now: Ms, c: Connecting, fx: &mut Vec<Effect>) {
        let spec = c.spec();
        self.work.connecting = Some(c);
        self.on_view(now, ViewOp::Open(spec), fx);
    }

    fn work_view_key(&mut self, now: Ms, key: Key, fx: &mut Vec<Effect>) -> bool {
        let Some(top) = self.views.top().map(|f| f.name.clone()) else {
            return false;
        };
        match top.as_str() {
            chat::VIEW => self.chat_key(now, key, fx),
            steps::VIEW => match key {
                Key::Up | Key::Down => self
                    .work
                    .steps
                    .as_mut()
                    .is_some_and(|s| s.move_sel(key == Key::Down)),
                Key::Enter => {
                    if let Some(s) = self.work.steps.as_mut() {
                        s.held = false;
                    }
                    self.steps_enter(now, fx)
                }
                _ => false,
            },
            connect::VIEW => self.connect_key(now, key, fx),
            strip::AGENT_VIEW => self.agent_key(key, fx),
            past_chats::VIEW => self.past_chats_key(key, fx),
            _ => false,
        }
    }

    fn chat_key(&mut self, now: Ms, key: Key, fx: &mut Vec<Effect>) -> bool {
        let Some(c) = self.work.chat.as_mut() else {
            return false;
        };
        let pressed = match key {
            Key::Up | Key::Down => return c.move_sel(key == Key::Down),
            Key::CmdK => {
                c.filter(&self.text);
                c.toggle_acting();
                return true;
            }
            Key::CmdDigit(2) => c.say_no(),
            Key::Enter => {
                let acting = c.acting.is_some();
                let pressed = c.press_enter(&self.text);
                if acting && !matches!(pressed, chat::ChatKey::Ask { .. }) {
                    self.clear_view_input(now, fx);
                }
                pressed
            }
            _ => return false,
        };
        self.chat_pressed(now, pressed, fx)
    }

    /// Enter in the steps view, or the Enter that waited for the steps to be read.
    fn steps_enter(&mut self, now: Ms, fx: &mut Vec<Effect>) -> bool {
        let conns = self.work.connections.clone();
        let Some(s) = self.work.steps.as_mut() else {
            return false;
        };
        match s.press_enter(&conns) {
            StepsKey::Nothing | StepsKey::Moved => false,
            StepsKey::Held => true,
            StepsKey::Connect { step, integration } => {
                let text = s.steps[step].text.clone();
                let does = s.steps[step].title.clone().unwrap_or_else(|| text.clone());
                let c = Connecting::new(&integration, &text, &does, Opener::Step(step));
                self.open_connect(now, c, fx);
                true
            }
            StepsKey::Run => {
                let steps = s.live().map(|(i, st)| (i, st.text.clone())).collect();
                fx.push(Effect::Work(WorkEffect::RunSteps { steps }));
                true
            }
        }
    }

    fn connect_key(&mut self, now: Ms, key: Key, fx: &mut Vec<Effect>) -> bool {
        let Some(c) = self.work.connecting.as_mut() else {
            return false;
        };
        match key {
            Key::Enter if c.phase == connect::Phase::Ask => {
                c.phase = connect::Phase::Connecting;
                c.error = None;
                fx.push(Effect::Work(WorkEffect::SignIn {
                    integration: c.integration.clone(),
                }));
                true
            }
            Key::CmdDigit(3) => {
                let c = c.clone();
                self.work.connecting = None;
                fx.push(Effect::Work(WorkEffect::CopyInstead {
                    integration: c.integration.clone(),
                    text: c.waiting.clone(),
                }));
                match c.opener {
                    Opener::Step(step) => {
                        if let Some(s) = self.work.steps.as_mut() {
                            s.go_without(step);
                        }
                        self.on_view(now, ViewOp::Back, fx);
                    }
                    Opener::Row(_) => self.done(
                        now,
                        Done {
                            icon: "message".into(),
                            title: c.without_title(),
                            ..Done::default()
                        },
                        fx,
                    ),
                }
                true
            }
            _ => false,
        }
    }

    /// The service connected: the window says so above the input and hands back to whoever
    /// opened the connect view, with Enter guarded against the change.
    fn connected(&mut self, now: Ms, integration: &str, fx: &mut Vec<Effect>) -> bool {
        self.work.connections.add(integration);
        let Some(c) = self
            .work
            .connecting
            .take_if(|c| c.integration == integration)
        else {
            return true;
        };
        self.ledger.record(
            now,
            NewEntry {
                icon: "connect".into(),
                title: c.done_title(),
                ..NewEntry::default()
            },
        );
        if self.views.top().is_some_and(|f| f.name == connect::VIEW) {
            self.on_view(now, ViewOp::Back, fx);
        }
        match c.opener {
            Opener::Step(_) => {
                if let Some(s) = self.work.steps.as_mut() {
                    s.reread();
                    fx.push(Effect::Work(WorkEffect::ReadSteps {
                        sentence: s.sentence.clone(),
                    }));
                }
            }
            Opener::Row(row) => self.run(now, *row, RunHow::Slot, fx),
        }
        self.guard.arm(now);
        true
    }

    fn agent_key(&mut self, key: Key, fx: &mut Vec<Effect>) -> bool {
        let Some(task) = self.open_task().cloned() else {
            return false;
        };
        if key != Key::Enter {
            return false;
        }
        let typed = self.text.trim().to_string();
        if !typed.is_empty() {
            fx.push(Effect::Work(WorkEffect::AgentSay {
                id: task.id,
                text: typed,
            }));
            fx.push(Effect::SetInput(String::new()));
            self.text.clear();
            return true;
        }
        if task.state == strip::TaskState::Ready {
            fx.push(Effect::Work(WorkEffect::AgentApprove { id: task.id }));
            return true;
        }
        false
    }

    pub(super) fn clear_view_input(&mut self, _now: Ms, fx: &mut Vec<Effect>) {
        if !self.text.is_empty() {
            fx.push(Effect::SetInput(String::new()));
            self.text.clear();
        }
    }

    /// Keeps each work view's ↵ words in step with its state. A change on top that the user's own
    /// key didn't cause guards Enter, like any view's.
    fn sync_work_enter(&mut self, now: Ms, user: bool) {
        let depth = self.views.depth();
        for i in 0..depth {
            let Some(name) = self.views.frame(i).map(|f| f.name.clone()) else {
                continue;
            };
            if !WORK_VIEWS.contains(&name.as_str()) {
                continue;
            }
            let typed = if i + 1 == depth {
                self.text.as_str()
            } else {
                ""
            };
            let target = match name.as_str() {
                chat::VIEW => self.work.chat.as_ref().map(|c| c.enter(typed)),
                steps::VIEW => self
                    .work
                    .steps
                    .as_ref()
                    .map(|s| s.enter(&self.work.connections)),
                connect::VIEW => self.work.connecting.as_ref().map(Connecting::enter),
                past_chats::VIEW => self.work.past.as_ref().map(PastChats::enter),
                _ => self
                    .work
                    .agent
                    .as_deref()
                    .and_then(|id| self.work.strip.get(id))
                    .map(|t| t.enter(typed)),
            };
            let Some(target) = target else {
                continue;
            };
            let top = i + 1 == depth;
            if let Some(frame) = self.views.frame_mut(i) {
                if frame.enter.as_ref() != Some(&target) {
                    frame.enter = Some(target.clone());
                    if top {
                        self.guard.observe_view(now, &target.id, user);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steps::StepSpec;
    use crate::strip::TaskState;

    const S10: &str = "move standup to tomorrow at 9, tell mo i'm running late and add to today's note: ship beta notes";

    fn work(win: &mut OneWindow, now: Ms, ev: WorkEvent) -> Vec<Effect> {
        win.handle(now, Event::Work(ev)).effects
    }

    /// A key, and the `ViewKey` the host hands back in the same turn.
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

    fn read_s10(win: &mut OneWindow, now: Ms, calendar: bool) {
        let texts = crate::steps::split(S10);
        let steps = texts
            .iter()
            .enumerate()
            .map(|(i, t)| StepSpec {
                text: t.clone(),
                title: Some(t.clone()),
                needs: (i == 0 && calendar).then(|| "calendar".to_string()),
            })
            .collect();
        work(
            win,
            now,
            WorkEvent::StepsRead {
                sentence: S10.into(),
                steps,
            },
        );
    }

    #[test]
    fn cmd_enter_with_nothing_typed_asks_the_model_under_the_same_input() {
        let mut win = OneWindow::default();
        let fx = key(&mut win, 0, Key::CmdEnter);
        assert!(!fx.iter().any(|e| matches!(e, Effect::PassKey(_))));
        assert_eq!(win.views().top().map(|f| f.name.as_str()), Some(chat::VIEW));
        assert_eq!(win.enter_target(0).id, "chat:idle");
        win.handle(10, Event::Text("what's a good name".into()));
        assert_eq!(
            win.enter_target(10).label,
            "Ask the model: “what's a good name”"
        );
        let fx = key(&mut win, 20, Key::Enter);
        assert_eq!(
            work_fx(&fx),
            [WorkEffect::AskModel {
                text: "what's a good name".into()
            }]
        );
        assert_eq!(win.text(), "", "the chat's input clears for the next turn");
        assert_eq!(win.enter_target(20).label, "Waiting for the model…");
    }

    #[test]
    fn asking_the_model_opens_its_chat_and_esc_restores_the_sentence() {
        let mut win = OneWindow::default();
        win.handle(
            0,
            Event::Text("brainstorm names for the launch party".into()),
        );
        let fx = work(
            &mut win,
            10,
            WorkEvent::AskModel {
                sentence: "brainstorm names for the launch party".into(),
            },
        );
        assert!(work_fx(&fx).contains(&WorkEffect::AskModel {
            text: "brainstorm names for the launch party".into()
        }));
        assert!(fx.contains(&Effect::SetInput(String::new())));
        work(
            &mut win,
            100,
            WorkEvent::ModelWrote {
                text: "1. Ship It".into(),
                done: true,
            },
        );
        work(&mut win, 200, WorkEvent::ModelNext(Vec::new()));
        assert_eq!(win.enter_target(200).label, "Copy it");
        assert!(
            win.enter_armed(200),
            "the reply settling on its own guards Enter"
        );
        let fx = key(&mut win, 700, Key::Enter);
        assert!(matches!(
            work_fx(&fx).as_slice(),
            [WorkEffect::ChatStep { row, reply }] if row.key == "chat:copy" && reply == "1. Ship It"
        ));
        let fx = key(&mut win, 800, Key::Escape);
        assert!(fx.contains(&Effect::SetInput(
            "brainstorm names for the launch party".into()
        )));
        assert!(!win.in_view());
        assert!(win.chat().is_none());
    }

    #[test]
    fn steps_ask_for_calendar_connect_in_place_and_carry_on() {
        let mut win = OneWindow::default();
        win.handle(0, Event::Text(S10.into()));
        let fx = work(
            &mut win,
            10,
            WorkEvent::OpenSteps {
                sentence: S10.into(),
            },
        );
        assert!(work_fx(&fx).contains(&WorkEffect::ReadSteps {
            sentence: S10.into()
        }));
        assert_eq!(win.enter_target(10).label, "Reading the steps…");
        assert_eq!(win.text(), S10, "the sentence stays in the input");

        read_s10(&mut win, 200, true);
        assert_eq!(
            win.enter_target(200),
            EnterTarget::new(
                "steps:connect:calendar",
                "Connect Google Calendar for step 1, then do all 3"
            )
        );
        assert!(win.enter_armed(200));
        assert!(
            work_fx(&key(&mut win, 300, Key::Enter)).is_empty(),
            "guarded"
        );

        key(&mut win, 1_000, Key::Enter);
        assert_eq!(win.views().crumb(), "Steps › Connect Google Calendar");
        assert_eq!(win.enter_target(1_000).label, "Connect Google Calendar");
        let c = win.connecting().expect("connect view");
        assert_eq!(c.waiting, "move standup to tomorrow at 9");

        let fx = key(&mut win, 1_500, Key::Enter);
        assert_eq!(
            work_fx(&fx),
            [WorkEffect::SignIn {
                integration: "calendar".into()
            }]
        );
        assert_eq!(win.enter_target(1_500).label, "Connecting Google Calendar…");

        let fx = work(
            &mut win,
            2_000,
            WorkEvent::Connected {
                integration: "calendar".into(),
            },
        );
        assert!(work_fx(&fx).contains(&WorkEffect::ReadSteps {
            sentence: S10.into()
        }));
        assert_eq!(win.views().crumb(), "Steps");
        assert_eq!(win.text(), S10);
        let line = win.recent_line(2_000).unwrap();
        assert_eq!(
            (line.label.as_str(), line.title.as_str()),
            ("Last", "Google Calendar connected")
        );
        assert!(win.enter_armed(2_000), "the hand-back guards Enter");
        assert_eq!(win.enter_target(2_000).label, "Reading the steps…");

        read_s10(&mut win, 2_200, true);
        assert_eq!(win.enter_target(2_200).label, "Do all 3");
        let fx = key(&mut win, 3_000, Key::Enter);
        assert!(matches!(
            work_fx(&fx).as_slice(),
            [WorkEffect::RunSteps { steps }] if steps.len() == 3
        ));
        assert_eq!(win.enter_target(3_000).label, "Working through them…");
        for i in 0..3 {
            work(&mut win, 3_100 + i as Ms, WorkEvent::StepDone { index: i });
        }
        assert!(!win.in_view());
        let line = win.recent_line(3_200).unwrap();
        assert!(line.title.starts_with("Did 3 steps: move standup"));
        assert!(
            line.sub.contains("saved as"),
            "the run is saved as a script"
        );
    }

    #[test]
    fn a_row_that_needs_a_service_connects_then_runs_what_was_said() {
        let mut win = OneWindow::default();
        win.handle(0, Event::Text("move standup to 10".into()));
        let row = Row {
            needs: Some("calendar".into()),
            ..Row::new("jev:calendar: move", "Move standup to 10")
        };
        win.handle(
            500,
            Event::Jev {
                text: "move standup to 10".into(),
                rows: vec![row],
            },
        );
        assert_eq!(win.enter_target(500).label, "Connect Google Calendar");
        let fx = key(&mut win, 1_000, Key::Enter);
        assert!(!fx.iter().any(|e| matches!(e, Effect::Run { .. })));
        assert_eq!(
            win.views().top().map(|f| f.name.as_str()),
            Some(connect::VIEW)
        );
        key(&mut win, 1_100, Key::Enter);
        let fx = work(
            &mut win,
            1_500,
            WorkEvent::Connected {
                integration: "calendar".into(),
            },
        );
        assert!(
            fx.iter()
                .any(|e| matches!(e, Effect::Run { row, .. } if row.key == "jev:calendar: move")),
            "the waiting sentence runs once connected"
        );
        assert!(!win.in_view());
        assert!(win.work().connections().has("calendar"));
    }

    #[test]
    fn going_without_the_service_copies_a_message_instead() {
        let mut win = OneWindow::default();
        win.handle(0, Event::Text("move standup to 10".into()));
        let row = Row {
            needs: Some("calendar".into()),
            ..Row::new("jev:calendar: move", "Move standup to 10")
        };
        win.handle(
            500,
            Event::Jev {
                text: "move standup to 10".into(),
                rows: vec![row],
            },
        );
        key(&mut win, 1_000, Key::Enter);
        let fx = key(&mut win, 1_100, Key::CmdDigit(3));
        assert!(matches!(
            work_fx(&fx).as_slice(),
            [WorkEffect::CopyInstead { .. }]
        ));
        assert!(!win.in_view());
        assert_eq!(
            win.recent_line(1_100).unwrap().title,
            "Copied a message you can paste into Google Calendar"
        );
    }

    #[test]
    fn an_agent_parks_in_the_strip_and_cmd_j_opens_what_needs_you() {
        let mut win = OneWindow::default();
        win.handle(0, Event::Text("fix SK-431".into()));
        let fx = work(
            &mut win,
            10,
            WorkEvent::Agent {
                task: "fix SK-431".into(),
            },
        );
        let id = match work_fx(&fx).as_slice() {
            [WorkEffect::StartAgent { id, task }] if task == "fix SK-431" => id.clone(),
            other => panic!("{other:?}"),
        };
        assert_eq!(win.enter_target(10).label, "Nothing to do — it's working");
        let fx = key(&mut win, 500, Key::Escape);
        assert_eq!(work_fx(&fx), [WorkEffect::Parked { id: id.clone() }]);
        assert!(!win.in_view());
        assert_eq!(win.text(), "");
        assert_eq!(
            win.work().strip().tasks().len(),
            1,
            "it keeps working in the strip"
        );

        work(
            &mut win,
            900,
            WorkEvent::TaskChanged {
                id: id.clone(),
                status: "Needs you · which list?".into(),
                progress: 0.5,
                state: TaskState::NeedsYou,
                approve: None,
            },
        );
        let fx = key(&mut win, 1_000, Key::CmdJ);
        assert_eq!(work_fx(&fx), [WorkEffect::Reopened { id: id.clone() }]);
        assert_eq!(win.enter_target(1_000).label, "Pick an answer, or say it");
        win.handle(1_100, Event::Text("both".into()));
        assert_eq!(win.enter_target(1_100).label, "Send to the agent");
        let fx = key(&mut win, 1_200, Key::Enter);
        assert_eq!(
            work_fx(&fx),
            [WorkEffect::AgentSay {
                id,
                text: "both".into()
            }]
        );
    }

    #[test]
    fn tab_makes_the_agent_the_owner_and_only_enter_starts_it() {
        use crate::contract::words;
        let mut win = OneWindow::default();
        win.handle(0, Event::Text("fix this".into()));
        let before = win.list().layout();
        let fx = key(&mut win, 10, Key::Tab);
        assert!(work_fx(&fx).is_empty(), "⇥ starts nothing");
        assert!(win.agent_owns());
        assert!(!win.in_view());
        assert_eq!(win.list().header(), words::AGENT);
        assert_eq!(
            win.list().visible().len(),
            1,
            "the one row that hands it over"
        );
        let enter = win.enter_target(10);
        assert_eq!(
            (enter.id.as_str(), enter.label.as_str()),
            (strip::OWNER_ROW, words::START_AGENT)
        );
        assert!(
            !win.enter_armed(10),
            "the user's own key never guards Enter"
        );

        win.handle(20, Event::Text("fix this now".into()));
        assert!(win.agent_owns(), "typing on keeps the Owner");
        assert_eq!(win.enter_target(20).id, strip::OWNER_ROW);

        key(&mut win, 30, Key::Escape);
        assert!(!win.agent_owns(), "Esc lets go of the Owner first");
        assert_eq!(win.text(), "fix this now");
        assert_ne!(win.list().header(), words::AGENT);

        key(&mut win, 40, Key::Tab);
        key(&mut win, 50, Key::Tab);
        assert!(!win.agent_owns(), "⇥ again takes it back");
        win.handle(60, Event::Text("fix this".into()));
        assert_eq!(win.list().layout(), before);

        key(&mut win, 70, Key::Tab);
        win.handle(80, Event::Text(String::new()));
        assert!(!win.agent_owns(), "an empty input has no Owner");
        let fx = key(&mut win, 90, Key::Tab);
        assert!(
            fx.contains(&Effect::PassKey(Key::Tab)),
            "⇥ on nothing is the host's"
        );

        win.handle(100, Event::Text("fix this".into()));
        key(&mut win, 110, Key::Tab);
        let fx = key(&mut win, 120, Key::Enter);
        assert!(
            matches!(work_fx(&fx).as_slice(), [WorkEffect::StartAgent { task, .. }] if task == "fix this")
        );
        assert_eq!(
            win.views().top().map(|f| f.name.as_str()),
            Some(strip::AGENT_VIEW)
        );
        assert!(!win.agent_owns(), "the agent took the sentence");
    }

    #[test]
    fn the_owner_stays_while_a_view_reads_the_sentence_and_is_there_after_esc() {
        let mut win = OneWindow::default();
        win.handle(0, Event::Text("fix this".into()));
        key(&mut win, 10, Key::Tab);
        win.handle(
            20,
            Event::View(ViewOp::Open(ViewSpec {
                name: "reading".into(),
                label: "How Jev reads it".into(),
                ..ViewSpec::default()
            })),
        );
        assert!(win.agent_owns());
        key(&mut win, 30, Key::Escape);
        assert!(!win.in_view());
        assert!(win.agent_owns(), "Esc closed the reading, not the Owner");
        assert_eq!(win.enter_target(30).id, strip::OWNER_ROW);
    }

    #[test]
    fn a_job_handed_off_in_the_background_starts_in_the_strip_without_moving_anything() {
        let mut win = OneWindow::default();
        win.handle(0, Event::Text("fix".into()));
        let before = win.list().layout();
        let fx = work(
            &mut win,
            10,
            WorkEvent::Background {
                task: "fix SK-431".into(),
            },
        );
        assert!(
            matches!(work_fx(&fx).as_slice(), [WorkEffect::StartAgent { task, .. }] if task == "fix SK-431")
        );
        assert!(!win.in_view());
        assert_eq!(win.list().layout(), before);
        assert_eq!(win.text(), "fix");
        assert_eq!(win.work().strip().tasks().len(), 1);
    }

    #[test]
    fn cmd_j_with_nothing_in_the_strip_is_the_hosts() {
        let mut win = OneWindow::default();
        let fx = key(&mut win, 0, Key::CmdJ);
        assert!(fx.contains(&Effect::PassKey(Key::CmdJ)));
    }
}
