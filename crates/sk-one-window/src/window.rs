//! The one window as a reducer: events in, effects out, time passed in.
//!
//! The host owns pixels, processes and Jev; this owns every rule about what the line above the
//! input says, which header leads the list, what Enter runs and when it is ignored, how the
//! window's height moves, and what waits before it goes. Nothing here reads a clock.

use crate::Ms;
use crate::contract::{DISMISS_SETTLE_MS, ENTER_GUARD_MS, JEV_DEBOUNCE_MS, MAX_MATCHES, words};
use crate::enter::{EnterGuard, EnterTarget};
use crate::height::{ContentCause, HeightDecision, HeightGovernor};
use crate::hold::{HeldChange, HeldState, Holds};
use crate::ledger::{Entry, EntryState, Ledger, NewEntry, Risk, risk_of};
use crate::list::{Item, ListState};
use crate::row::{Row, RowAct};
use crate::slot::{SlotKind, SlotLine};
use crate::stage::{
    Staged, Subject, SubjectKind, default_actions, entry_rows, entry_subject, row_subject,
    script_name, share_rows,
};
use crate::view::{ViewFrame, ViewStack};

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    /// How long a message waits before it goes; 0 sends at once.
    pub hold_ms: u64,
    /// Jev's rows come first when no name is strong enough to own Enter.
    pub jev_first: bool,
    /// A row read from older words never owns Enter.
    pub stale_holds: bool,
    /// A sentence that did several things is saved as a script.
    pub auto_save: bool,
    /// Launching an app hides the window.
    pub dismiss_on_launch: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hold_ms: crate::contract::MESSAGE_HOLD_MS,
            jev_first: true,
            stale_holds: true,
            auto_save: true,
            dismiss_on_launch: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Key {
    Enter,
    CmdEnter,
    Escape,
    Up,
    Down,
    CmdK,
    CmdJ,
    CmdZ,
    Tab,
    CmdDigit(u8),
    Right,
    Other(String),
}

/// A view the host opened in place of the list.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ViewSpec {
    pub name: String,
    pub label: String,
    pub enter: Option<EnterTarget>,
    pub takes_input: bool,
    /// Words the view puts in the input when it opens.
    pub text: Option<String>,
    /// Words it leaves in the input when you go back; `None` restores what you'd typed.
    pub leave_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ViewOp {
    Open(ViewSpec),
    Replace(ViewSpec),
    Back,
    Home { text: String },
    Park,
    Reopen(ViewSpec),
}

/// Something finished.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Done {
    pub icon: String,
    pub title: String,
    pub risk: Option<Risk>,
    /// The form's kind id ("message", "event", ...) when a form finished.
    pub form_kind: Option<String>,
    /// Stay in the view instead of going back to the list.
    pub stay: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// The input's words changed because the user typed.
    Text(String),
    Key(Key),
    /// Name matches for the current words (fallback rows marked as such).
    Names(Vec<Row>),
    /// Rows the host puts above everything for the current words.
    Pinned(Vec<Row>),
    /// What the empty input lists when nothing else is pinned.
    Suggested(Vec<Row>),
    Jev {
        text: String,
        rows: Vec<Row>,
    },
    JevDown {
        text: String,
    },
    View(ViewOp),
    /// The open view's Enter changed.
    ViewEnter(EnterTarget),
    Done(Done),
    /// A view held a message on its own.
    Hold {
        label: String,
        request: String,
    },
    /// New words for a waiting message.
    Rewrite {
        target: String,
        text: String,
    },
    /// Stop waiting messages: one by label, or every one.
    Unsend {
        target: Option<String>,
    },
    /// The host ran a row that finished on the spot (an app launch, a script with no UI).
    Ran {
        row: Row,
        app: Option<String>,
    },
    /// The steps a sentence was split into, for naming the script it's saved as.
    Steps {
        sentence: String,
        steps: Vec<String>,
    },
    /// The window's natural height for what it shows now.
    Content {
        px: f32,
    },
    Summon,
    Focused(Subject),
    /// The staged subject's actions.
    Actions(Vec<Row>),
    /// Shows a line above the input until cleared.
    Line(Option<SlotLine>),
    Tick,
}

impl Event {
    fn is_user(&self) -> bool {
        matches!(self, Event::Text(_) | Event::Key(_) | Event::Summon)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunHow {
    Match,
    Slot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeldOp {
    Start,
    Sent,
    Unsent,
    Paused,
    Resumed,
    Rewritten,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    /// Run a row the host owns; it answers with a view, `Ran`, or `Done`.
    Run {
        row: Row,
        how: RunHow,
    },
    /// A key for the open view.
    ViewKey(Key),
    /// A key nothing here used; the host's default applies.
    PassKey(Key),
    AskJev {
        text: String,
    },
    Height(HeightDecision),
    WakeAt(Ms),
    /// A waiting message changed; on `Sent` the host sends it.
    Held {
        op: HeldOp,
        id: u64,
        label: String,
        request: String,
    },
    Undo {
        entry: u64,
    },
    Again {
        entry: u64,
    },
    OpenScript {
        name: String,
    },
    SaveScript {
        entry: u64,
        name: String,
        said: String,
    },
    ActionsFor(Subject),
    Hide {
        app: Option<String>,
    },
    Show,
    EnterIgnored,
    SetInput(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outcome {
    pub effects: Vec<Effect>,
    pub consumed: bool,
}

/// What the window shows, for the host to draw and traces to compare.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub slot: SlotLine,
    /// 1 is live; below 1 the line reads as background.
    pub slot_opacity: f32,
    pub header: String,
    pub view: Option<String>,
    pub crumb: String,
    pub enter: EnterTarget,
    pub armed: bool,
    pub items: Vec<Item>,
    pub sel: usize,
    pub text: String,
}

#[derive(Clone, Debug)]
struct PendingDismiss {
    at: Ms,
    row: Row,
    app: Option<String>,
    said: String,
}

#[derive(Clone, Debug)]
pub struct OneWindow {
    cfg: Config,
    text: String,
    list: ListState,
    host_names: Vec<Row>,
    host_pinned: Vec<Row>,
    suggested: Vec<Row>,
    views: ViewStack,
    staged: Option<Staged>,
    line: Option<SlotLine>,
    focused: Subject,
    ledger: Ledger,
    holds: Holds,
    height: HeightGovernor,
    guard: EnterGuard,
    user_at: Option<Ms>,
    /// Why the next natural height changed, and when it was set.
    cause: Option<(ContentCause, Ms)>,
    jev_due: Option<(Ms, String)>,
    dismiss: Option<PendingDismiss>,
    away: bool,
    steps: Option<(String, Vec<String>)>,
    wake: Option<Ms>,
    now: Ms,
}

impl Default for OneWindow {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl OneWindow {
    pub fn new(cfg: Config) -> Self {
        Self {
            list: ListState::new(cfg.jev_first, cfg.stale_holds),
            holds: Holds::new(cfg.hold_ms),
            cfg,
            text: String::new(),
            host_names: Vec::new(),
            host_pinned: Vec::new(),
            suggested: Vec::new(),
            views: ViewStack::default(),
            staged: None,
            line: None,
            focused: Subject {
                kind: SubjectKind::Window,
                label: "Front window".into(),
                icon: "window".into(),
                ..Subject::default()
            },
            ledger: Ledger::new(),
            height: HeightGovernor::new(),
            guard: EnterGuard::new(),
            user_at: None,
            cause: None,
            jev_due: None,
            dismiss: None,
            away: false,
            steps: None,
            wake: None,
            now: 0,
        }
    }

    /// Starts with what ran before this session.
    pub fn with_history(mut self, history: Vec<Entry>) -> Self {
        for e in history {
            self.ledger.seed(e);
        }
        self
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn list(&self) -> &ListState {
        &self.list
    }
    pub fn views(&self) -> &ViewStack {
        &self.views
    }
    pub fn staged(&self) -> Option<&Staged> {
        self.staged.as_ref()
    }
    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }
    pub fn holds(&self) -> &Holds {
        &self.holds
    }
    pub fn height(&self) -> &HeightGovernor {
        &self.height
    }
    pub fn is_away(&self) -> bool {
        self.away
    }
    pub fn in_view(&self) -> bool {
        !self.views.is_empty()
    }

    /// The sentence the open view came from, or what's typed when nothing is open.
    pub fn said(&self) -> String {
        self.views
            .first_saved_text()
            .map(str::to_string)
            .unwrap_or_else(|| self.text.clone())
    }

    fn user_turn(&self, now: Ms) -> bool {
        self.user_at == Some(now)
    }

    pub fn handle(&mut self, now: Ms, ev: Event) -> Outcome {
        let now = now.max(self.now);
        self.now = now;
        let mut fx = Vec::new();
        if ev.is_user() {
            self.user_at = Some(now);
        }
        self.tick(now, &mut fx);
        let consumed = self.apply(now, ev, &mut fx);
        self.sync_pause(now, &mut fx);
        self.sync_staged(now);
        self.schedule(now, &mut fx);
        Outcome {
            effects: fx,
            consumed,
        }
    }

    fn apply(&mut self, now: Ms, ev: Event, fx: &mut Vec<Effect>) -> bool {
        match ev {
            Event::Text(text) => {
                if self.cause.is_some_and(|(_, at)| at < now) {
                    self.cause = None;
                }
                if let Some(d) = self.height.typed(now) {
                    fx.push(Effect::Height(d));
                }
                self.on_text(now, text, true, fx);
                true
            }
            Event::Key(key) => self.on_key(now, key, fx),
            Event::Names(rows) => {
                self.host_names = rows;
                self.refresh_rows();
                true
            }
            Event::Pinned(rows) => {
                self.host_pinned = rows;
                self.refresh_rows();
                true
            }
            Event::Suggested(rows) => {
                self.suggested = rows;
                self.refresh_rows();
                true
            }
            Event::Jev { text, rows } => {
                if text != self.text {
                    return false;
                }
                let rows = crate::list::fresh_readings(rows, &self.list.names);
                self.list.jev.assign(now, &text, rows);
                self.list.move_sel(0);
                if let Some(i) = self.list.held.take() {
                    self.list.asking = false;
                    if let Some(row) = self.list.jev.get(i).and_then(|s| s.row.clone()) {
                        self.run(now, row, RunHow::Slot, fx);
                    }
                }
                true
            }
            Event::JevDown { text } => {
                if text != self.text {
                    return false;
                }
                self.list.jev.down(now, &text);
                self.list.held = None;
                self.list.asking = false;
                let user = self.user_turn(now);
                self.guard.arm_unless_user(now, user);
                true
            }
            Event::View(op) => {
                self.on_view(now, op, fx);
                true
            }
            Event::ViewEnter(target) => {
                let user = self.user_turn(now);
                if let Some(top) = self.views.top_mut() {
                    top.enter = Some(target.clone());
                    self.guard.observe_view(now, &target.id, user);
                }
                true
            }
            Event::Done(done) => {
                self.done(now, done, fx);
                true
            }
            Event::Hold { label, request } => {
                self.start_hold(now, label, request, None, fx);
                true
            }
            Event::Rewrite { target, text } => {
                let Some(held) = self.holds.find_live_by_label(&target).cloned() else {
                    return false;
                };
                let to = match held.label.find(": “") {
                    Some(ix) if held.label.ends_with('”') => held.label[..ix].to_string(),
                    _ => held.label.clone(),
                };
                let label = format!("{to}: “{text}”");
                let request = format!(
                    "{} {text}",
                    to.strip_prefix("To ")
                        .map(|n| format!("tell {n}"))
                        .unwrap_or(to.clone())
                );
                self.holds
                    .rewrite(now, held.id, label.clone(), request.clone());
                if let Some(entry) = held.entry {
                    self.ledger.set_title(entry, label.clone());
                }
                fx.push(Effect::Held {
                    op: HeldOp::Rewritten,
                    id: held.id,
                    label: label.clone(),
                    request,
                });
                self.done(
                    now,
                    Done {
                        icon: "change".into(),
                        title: format!("Changed: {label}"),
                        ..Done::default()
                    },
                    fx,
                );
                true
            }
            Event::Unsend { target } => {
                let ids: Vec<u64> = self
                    .holds
                    .live()
                    .filter(|h| target.as_deref().is_none_or(|t| t == h.label))
                    .map(|h| h.id)
                    .collect();
                if ids.is_empty() {
                    return false;
                }
                self.unsend(now, &ids, fx);
                true
            }
            Event::Ran { row, app } => {
                let said = self.said();
                let app = app.filter(|_| self.cfg.dismiss_on_launch);
                if app.is_some() {
                    self.away = true;
                    fx.push(Effect::Hide { app: app.clone() });
                    self.dismiss = Some(PendingDismiss {
                        at: now + DISMISS_SETTLE_MS,
                        row,
                        app,
                        said,
                    });
                } else {
                    self.record_ran(now, &row, None, said);
                    self.home(now, String::new(), fx);
                }
                true
            }
            Event::Steps { sentence, steps } => {
                self.steps = Some((sentence, steps));
                true
            }
            Event::Content { px } => {
                let cause = self
                    .cause
                    .take()
                    .map(|(c, _)| c)
                    .unwrap_or(if self.in_view() {
                        ContentCause::Other
                    } else {
                        ContentCause::List
                    });
                if let Some(d) = self.height.content(now, px, cause) {
                    fx.push(Effect::Height(d));
                }
                true
            }
            Event::Summon => {
                self.away = false;
                fx.push(Effect::Show);
                self.home(now, String::new(), fx);
                true
            }
            Event::Focused(subject) => {
                self.focused = subject;
                true
            }
            Event::Actions(rows) => {
                if let Some(staged) = self.staged.as_mut() {
                    staged.rows = rows;
                    staged.waiting_for_rows = false;
                    self.refresh_rows();
                }
                true
            }
            Event::Line(line) => {
                self.line = line;
                true
            }
            Event::Tick => true,
        }
    }

    // ------------------------------------------------------------------ words

    fn on_text(&mut self, now: Ms, text: String, typed: bool, fx: &mut Vec<Effect>) {
        if typed {
            if let Some(top) = self.views.top() {
                if top.takes_input {
                    self.text = text;
                    return;
                }
                self.views.home();
                self.guard.forget_view();
                self.cause = Some((ContentCause::Home, now));
            }
        }
        self.text = text;
        self.host_names.clear();
        self.list.on_text(&self.text);
        self.refresh_rows();
        self.jev_due =
            (!self.text.trim().is_empty()).then(|| (now + JEV_DEBOUNCE_MS, self.text.clone()));
        let _ = fx;
    }

    /// Sets the input's words the way the window does after a view closes: the list follows, but
    /// it isn't typing.
    fn set_input(&mut self, now: Ms, text: String, fx: &mut Vec<Effect>) {
        fx.push(Effect::SetInput(text.clone()));
        self.on_text(now, text, false, fx);
    }

    fn refresh_rows(&mut self) {
        let empty = self.text.trim().is_empty();
        let mut pins = self.host_pinned.clone();
        if let Some(staged) = &self.staged {
            if empty {
                pins.extend(staged.pinned());
            }
        }
        if empty && pins.is_empty() && self.staged.is_none() {
            pins = self
                .suggested
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    let mut r = r.clone();
                    if i == 0 && r.head.is_none() {
                        r.head = Some(words::SUGGESTED.into());
                    }
                    r
                })
                .collect();
        }
        self.list.pinned = pins;

        let mut names: Vec<Row> = self
            .staged
            .as_ref()
            .map(|s| s.matches(&self.text))
            .unwrap_or_default();
        let (fallbacks, real): (Vec<Row>, Vec<Row>) =
            self.host_names.iter().cloned().partition(|r| r.fallback);
        for r in real {
            if !names.iter().any(|n| n.title.eq_ignore_ascii_case(&r.title)) {
                names.push(r);
            }
        }
        names.truncate(MAX_MATCHES);
        if names.is_empty() && !empty {
            let down = self.list.jev.is_down_for(&self.text);
            names = fallbacks
                .into_iter()
                .filter(|r| !(down && r.key.starts_with("fallback-ai:")))
                .collect();
        }
        self.list.names = names;
        self.list.move_sel(0);
    }

    // ------------------------------------------------------------------ keys

    fn on_key(&mut self, now: Ms, key: Key, fx: &mut Vec<Effect>) -> bool {
        let armed = self.guard.armed(now);
        if self.in_view() {
            return match key {
                Key::Enter if armed => {
                    fx.push(Effect::EnterIgnored);
                    true
                }
                Key::Escape => {
                    self.on_view(now, ViewOp::Back, fx);
                    true
                }
                key => {
                    fx.push(Effect::ViewKey(key));
                    true
                }
            };
        }
        if self.away {
            return false;
        }
        if key == Key::Enter && armed {
            fx.push(Effect::EnterIgnored);
            return true;
        }
        let empty = self.text.is_empty();
        match &key {
            Key::CmdZ if empty => {
                if let Some(id) = self.holds.newest().map(|h| h.id) {
                    self.staged = None;
                    self.unsend(now, &[id], fx);
                    return true;
                }
            }
            Key::Up if empty && self.staged.is_none() && self.list.sel == 0 => {
                let target = self
                    .holds
                    .newest()
                    .and_then(|h| h.entry)
                    .or_else(|| self.ledger.last().map(|e| e.id));
                if let Some(id) = target {
                    let entry = self.ledger.get(id).expect("entry").clone();
                    let label = if entry.state == EntryState::Holding {
                        words::SENDING
                    } else {
                        words::LAST
                    };
                    self.stage(now, entry_subject(&entry, label), None, words::STAGED, fx);
                    return true;
                }
            }
            Key::CmdEnter if !self.text.trim().is_empty() => {
                return self.on_key(now, Key::CmdDigit(1), fx);
            }
            Key::CmdK => {
                let row = if self.text.trim().is_empty() {
                    None
                } else {
                    self.list.current_row(now)
                };
                let subject = row
                    .as_ref()
                    .map(row_subject)
                    .unwrap_or_else(|| self.focused.clone());
                self.stage(now, subject, None, words::STAGED, fx);
                return true;
            }
            Key::Escape if self.staged.is_some() => {
                self.staged = None;
                self.home(now, String::new(), fx);
                return true;
            }
            Key::CmdDigit(d) => {
                let i = (*d as usize).saturating_sub(1);
                if let Some(pos) = self.list.slot_position(i) {
                    self.list.sel = pos;
                    self.activate(now, fx);
                }
                return true;
            }
            _ => {}
        }
        let cancels = self.list.held.is_some() && matches!(key, Key::Up | Key::Down | Key::Escape);
        if cancels {
            self.list.held = None;
            self.list.asking = false;
        }
        match key {
            Key::Down => self.list.move_sel(1),
            Key::Up => self.list.move_sel(-1),
            Key::Enter => self.activate(now, fx),
            key if !cancels => {
                fx.push(Effect::PassKey(key));
                return false;
            }
            _ => {}
        }
        true
    }

    fn activate(&mut self, now: Ms, fx: &mut Vec<Effect>) {
        let Some(item) = self.list.current() else {
            return;
        };
        match item {
            Item::Slot(i) => match self.list.effective(i, now).cloned() {
                Some(row) => {
                    self.list.asking = false;
                    self.run(now, row, RunHow::Slot, fx);
                }
                None => self.list.held = Some(i),
            },
            item => {
                if let Some(row) = self.list.row(&item).cloned() {
                    self.run(now, row, RunHow::Match, fx);
                }
            }
        }
    }

    fn run(&mut self, now: Ms, row: Row, how: RunHow, fx: &mut Vec<Effect>) {
        let staged_entry = self.staged.as_ref().and_then(|s| s.subject.entry);
        let from_staged = self
            .staged
            .as_ref()
            .is_some_and(|s| s.rows.iter().any(|r| r.key == row.key));
        if from_staged && row.act != RowAct::Share {
            self.staged = None;
            self.refresh_rows();
        }
        match row.act {
            RowAct::Host => fx.push(Effect::Run { row, how }),
            RowAct::AskJevAgain => {
                self.list.jev.reset();
                self.list.sel = 0;
                self.jev_due = Some((now, self.text.clone()));
            }
            act => {
                let Some(entry) = staged_entry.and_then(|id| self.ledger.get(id).cloned()) else {
                    fx.push(Effect::Run { row, how });
                    return;
                };
                self.entry_act(now, act, &entry, fx);
            }
        }
    }

    fn entry_act(&mut self, now: Ms, act: RowAct, entry: &Entry, fx: &mut Vec<Effect>) {
        match act {
            RowAct::Unsend => {
                if let Some(id) = entry.held {
                    self.unsend(now, &[id], fx);
                }
            }
            RowAct::SendNow => {
                if let Some(h) = entry.held.and_then(|id| self.holds.send_now(now, id)) {
                    self.ledger.mark_sent(entry.id, now);
                    fx.push(Effect::Held {
                        op: HeldOp::Sent,
                        id: h.id,
                        label: h.label,
                        request: h.request,
                    });
                }
                self.home(now, String::new(), fx);
            }
            RowAct::Undo => {
                if self.ledger.undo(now, entry.id).is_some() {
                    fx.push(Effect::Undo { entry: entry.id });
                }
                self.home(now, String::new(), fx);
            }
            RowAct::Again => {
                if entry.held.is_some() || entry.risk == Risk::Hold {
                    self.start_hold(
                        now,
                        entry.title.clone(),
                        entry.said.clone(),
                        Some(entry.icon.clone()),
                        fx,
                    );
                    fx.push(Effect::Again { entry: entry.id });
                    self.home(now, String::new(), fx);
                    return;
                }
                fx.push(Effect::Again { entry: entry.id });
                let row = Row::new(format!("again:{}", entry.id), entry.title.clone())
                    .with_icon(entry.icon.clone());
                match entry.app.clone().filter(|_| self.cfg.dismiss_on_launch) {
                    Some(app) => {
                        self.away = true;
                        fx.push(Effect::Hide {
                            app: Some(app.clone()),
                        });
                        self.dismiss = Some(PendingDismiss {
                            at: now + DISMISS_SETTLE_MS,
                            row,
                            app: Some(app),
                            said: entry.said.clone(),
                        });
                    }
                    None => {
                        let id = self.ledger.record(
                            now,
                            NewEntry {
                                icon: entry.icon.clone(),
                                title: entry.title.clone(),
                                said: entry.said.clone(),
                                risk: entry.risk,
                                ..NewEntry::default()
                            },
                        );
                        if let Some(name) = entry.script.clone() {
                            self.ledger.set_script(id, name);
                        }
                        self.home(now, String::new(), fx);
                    }
                }
            }
            RowAct::OpenScript => {
                if let Some(name) = entry.script.clone() {
                    fx.push(Effect::OpenScript { name });
                }
            }
            RowAct::Share => {
                let subject = Subject {
                    label: words::SHARING.into(),
                    ..entry_subject(entry, words::SHARING)
                };
                self.stage(now, subject, Some(share_rows()), words::SHARE_HOW, fx);
            }
            RowAct::Host | RowAct::AskJevAgain => {}
        }
    }

    // ------------------------------------------------------------------ staging

    fn stage(
        &mut self,
        now: Ms,
        subject: Subject,
        rows: Option<Vec<Row>>,
        head: &str,
        fx: &mut Vec<Effect>,
    ) {
        let entry = subject.entry.and_then(|id| self.ledger.get(id)).cloned();
        let (rows, waiting) = match (rows, &entry) {
            (Some(rows), _) => (rows, false),
            (None, Some(e)) => (entry_rows(e), false),
            (None, None) => {
                fx.push(Effect::ActionsFor(subject.clone()));
                (default_actions(&subject), false)
            }
        };
        self.staged = Some(Staged {
            subject,
            rows,
            head: head.to_string(),
            waiting_for_rows: waiting,
            entry_state: entry.map(|e| e.state),
        });
        let user = self.user_turn(now);
        self.guard.arm_unless_user(now, user);
        self.home(now, String::new(), fx);
    }

    /// A staged entry whose state changed (a message went out) gets its new rows.
    fn sync_staged(&mut self, now: Ms) {
        let Some(staged) = self.staged.as_ref() else {
            return;
        };
        let Some(entry) = staged
            .subject
            .entry
            .and_then(|id| self.ledger.get(id))
            .cloned()
        else {
            return;
        };
        if staged.entry_state == Some(entry.state) {
            return;
        }
        let label = staged.subject.label.clone();
        let share = label == words::SHARING;
        let staged = self.staged.as_mut().expect("staged");
        staged.entry_state = Some(entry.state);
        if !share {
            staged.rows = entry_rows(&entry);
            staged.subject = entry_subject(&entry, &label);
        }
        let user = self.user_turn(now);
        self.guard.arm_unless_user(now, user);
        if !self.in_view() {
            self.refresh_rows();
        }
    }

    // ------------------------------------------------------------------ views

    fn on_view(&mut self, now: Ms, op: ViewOp, fx: &mut Vec<Effect>) {
        let user = self.user_turn(now);
        match op {
            ViewOp::Open(spec) | ViewOp::Reopen(spec) => {
                let frame = self.frame(&spec);
                self.views.open(frame);
                self.opened(now, spec, user, fx);
            }
            ViewOp::Replace(spec) => {
                let frame = self.frame(&spec);
                self.views.replace(frame);
                self.opened(now, spec, user, fx);
            }
            ViewOp::Back => {
                let Some(popped) = self.views.back() else {
                    return;
                };
                if let Some(top) = self.views.top() {
                    let target = top
                        .enter
                        .clone()
                        .unwrap_or_else(|| EnterTarget::new(top.name.clone(), ""));
                    self.cause = Some((ContentCause::ViewOpened, now));
                    self.guard.observe_view(now, &target.id, user);
                    let restore = popped.leave_text.unwrap_or(popped.saved_text);
                    fx.push(Effect::SetInput(restore.clone()));
                    self.text = restore;
                } else {
                    self.guard.forget_view();
                    self.guard.arm_unless_user(now, user);
                    self.cause = Some((ContentCause::Home, now));
                    self.set_input(now, popped.leave_text.unwrap_or(popped.saved_text), fx);
                }
            }
            ViewOp::Home { text } => self.home(now, text, fx),
            ViewOp::Park => {
                self.views.home();
                self.guard.forget_view();
                self.guard.arm_unless_user(now, user);
                self.cause = Some((ContentCause::Home, now));
                self.set_input(now, String::new(), fx);
            }
        }
    }

    fn frame(&self, spec: &ViewSpec) -> ViewFrame {
        ViewFrame {
            name: spec.name.clone(),
            label: spec.label.clone(),
            saved_text: self.text.clone(),
            leave_text: spec.leave_text.clone(),
            enter: spec.enter.clone(),
            takes_input: spec.takes_input,
        }
    }

    fn opened(&mut self, now: Ms, spec: ViewSpec, user: bool, fx: &mut Vec<Effect>) {
        self.cause = Some((ContentCause::ViewOpened, now));
        self.list.held = None;
        let target = spec
            .enter
            .unwrap_or_else(|| EnterTarget::new(spec.name.clone(), ""));
        self.guard.observe_view(now, &target.id, user);
        if let Some(text) = spec.text {
            if text != self.text {
                fx.push(Effect::SetInput(text.clone()));
            }
            self.text = text;
        }
    }

    fn home(&mut self, now: Ms, text: String, fx: &mut Vec<Effect>) {
        self.views.home();
        self.guard.forget_view();
        let user = self.user_turn(now);
        self.guard.arm_unless_user(now, user);
        self.cause = Some((ContentCause::Home, now));
        self.set_input(now, text, fx);
    }

    // ------------------------------------------------------------------ finishing

    fn done(&mut self, now: Ms, done: Done, fx: &mut Vec<Effect>) {
        let said = self.said();
        let risk = risk_of(
            done.risk,
            done.form_kind.as_deref(),
            Some(done.icon.as_str()),
        );
        if done.icon != "change" {
            let holds_it = risk == Risk::Hold
                && self.cfg.hold_ms > 0
                && done.form_kind.as_deref() == Some("message");
            if holds_it {
                self.start_hold(now, done.title.clone(), said, Some(done.icon.clone()), fx);
            } else {
                self.adopt_held(now);
                let id = self.ledger.record(
                    now,
                    NewEntry {
                        icon: done.icon.clone(),
                        title: done.title.clone(),
                        said: said.clone(),
                        risk,
                        ..NewEntry::default()
                    },
                );
                if self.cfg.auto_save
                    && matches!(done.icon.as_str(), "steps" | "delegate")
                    && !said.trim().is_empty()
                {
                    let steps = match &self.steps {
                        Some((sentence, steps)) if *sentence == said => steps.clone(),
                        _ => Vec::new(),
                    };
                    let name = script_name(&said, &steps);
                    self.ledger.set_script(id, name.clone());
                    fx.push(Effect::SaveScript {
                        entry: id,
                        name,
                        said,
                    });
                }
            }
        }
        if !done.stay {
            self.guard.arm(now);
            self.home(now, String::new(), fx);
        }
    }

    fn start_hold(
        &mut self,
        now: Ms,
        label: String,
        request: String,
        icon: Option<String>,
        fx: &mut Vec<Effect>,
    ) {
        if self.cfg.hold_ms == 0 {
            fx.push(Effect::Held {
                op: HeldOp::Sent,
                id: 0,
                label,
                request,
            });
            return;
        }
        let id = self.holds.start(now, label.clone(), request.clone());
        if let Some(icon) = icon {
            let entry = self.ledger.record(
                now,
                NewEntry {
                    icon,
                    title: label.clone(),
                    said: request.clone(),
                    risk: Risk::Hold,
                    held: Some(id),
                    ..NewEntry::default()
                },
            );
            self.holds.set_entry(id, entry);
        }
        fx.push(Effect::Held {
            op: HeldOp::Start,
            id,
            label,
            request,
        });
    }

    /// Messages a view held on its own get their own line in the ledger.
    fn adopt_held(&mut self, now: Ms) {
        let orphans: Vec<(u64, String, String)> = self
            .holds
            .live()
            .filter(|h| h.entry.is_none())
            .map(|h| (h.id, h.label.clone(), h.request.clone()))
            .collect();
        for (id, label, request) in orphans {
            let entry = self.ledger.record(
                now,
                NewEntry {
                    icon: "message".into(),
                    title: label,
                    said: request,
                    risk: Risk::Hold,
                    held: Some(id),
                    ..NewEntry::default()
                },
            );
            self.holds.set_entry(id, entry);
        }
    }

    fn unsend(&mut self, now: Ms, ids: &[u64], fx: &mut Vec<Effect>) {
        let mut labels = Vec::new();
        for id in ids {
            if let Some(h) = self.holds.unsend(now, *id) {
                if let Some(entry) = h.entry {
                    self.ledger.mark_unsent(entry);
                }
                fx.push(Effect::Held {
                    op: HeldOp::Unsent,
                    id: h.id,
                    label: h.label.clone(),
                    request: h.request,
                });
                labels.push(h.label);
            }
        }
        if labels.is_empty() {
            return;
        }
        let title = if labels.len() == 1 {
            format!("Didn't send: {}", labels[0])
        } else {
            format!("Didn't send {} messages", labels.len())
        };
        self.staged = None;
        self.done(
            now,
            Done {
                icon: "undo".into(),
                title,
                ..Done::default()
            },
            fx,
        );
    }

    fn record_ran(&mut self, now: Ms, row: &Row, app: Option<String>, said: String) {
        self.ledger.record(
            now,
            NewEntry {
                icon: row.icon.clone().unwrap_or_else(|| "check".into()),
                title: row.title.clone(),
                said,
                risk: Risk::None,
                app,
                ..NewEntry::default()
            },
        );
    }

    // ------------------------------------------------------------------ time

    fn tick(&mut self, now: Ms, fx: &mut Vec<Effect>) {
        for change in self.holds.advance(now) {
            if let HeldChange::Sent { id } = change {
                self.sent(now, id, fx);
            }
        }
        if let Some(d) = self.height.tick(now) {
            fx.push(Effect::Height(d));
        }
        if let Some((at, text)) = self.jev_due.clone() {
            if now >= at {
                self.jev_due = None;
                if text == self.text {
                    fx.push(Effect::AskJev { text });
                }
            }
        }
        if self.dismiss.as_ref().is_some_and(|d| now >= d.at) {
            let d = self.dismiss.take().expect("dismiss");
            self.staged = None;
            self.record_ran(now, &d.row, d.app, d.said);
            self.home(now, String::new(), fx);
        }
    }

    fn sent(&mut self, now: Ms, id: u64, fx: &mut Vec<Effect>) {
        let Some(h) = self.holds.get(id).cloned() else {
            return;
        };
        if let Some(entry) = h.entry {
            self.ledger.mark_sent(entry, now);
        }
        fx.push(Effect::Held {
            op: HeldOp::Sent,
            id,
            label: h.label,
            request: h.request,
        });
    }

    fn sync_pause(&mut self, now: Ms, fx: &mut Vec<Effect>) {
        let paused = !self.in_view() && !self.text.trim().is_empty();
        for change in self.holds.set_paused(now, paused) {
            let (op, id) = match change {
                HeldChange::Sent { id } => {
                    self.sent(now, id, fx);
                    continue;
                }
                HeldChange::Paused { id } => (HeldOp::Paused, id),
                HeldChange::Resumed { id } => (HeldOp::Resumed, id),
            };
            if let Some(h) = self.holds.get(id) {
                fx.push(Effect::Held {
                    op,
                    id,
                    label: h.label.clone(),
                    request: h.request.clone(),
                });
            }
        }
    }

    fn schedule(&mut self, now: Ms, fx: &mut Vec<Effect>) {
        let armed_end = self.guard.armed(now).then(|| self.guard.armed_until());
        let next = [
            self.holds.next_wake(now),
            self.height.next_wake(),
            self.list.jev.next_wake(now),
            armed_end,
            self.jev_due.as_ref().map(|(at, _)| *at),
            self.dismiss.as_ref().map(|d| d.at),
        ]
        .into_iter()
        .flatten()
        .filter(|t| *t > now)
        .min();
        if next != self.wake {
            self.wake = next;
            if let Some(t) = next {
                fx.push(Effect::WakeAt(t));
            }
        }
    }

    /// The next time the host must send `Tick`.
    pub fn next_wake(&self) -> Option<Ms> {
        self.wake
    }

    // ------------------------------------------------------------------ what it shows

    /// The line above the input: a set line, what's staged, a message about to go, the last thing
    /// that ran (dimmed), or what's focused (dimmed). It is always there and always one height.
    pub fn slot_line(&self, now: Ms) -> SlotLine {
        let _ = now;
        if let Some(line) = &self.line {
            return line.clone();
        }
        if let Some(staged) = &self.staged {
            let s = &staged.subject;
            return SlotLine::new(SlotKind::Staged, s.label.clone(), s.title.clone())
                .with_icon(if s.icon.is_empty() {
                    "builtin".to_string()
                } else {
                    s.icon.clone()
                })
                .with_sub(s.sub.clone())
                .with_hint("esc", "let go");
        }
        if let Some(m) = self.holds.newest() {
            let live = self.holds.live_count();
            let paused = self.holds.paused();
            let label = if paused {
                words::PAUSED.to_string()
            } else {
                words::sending_in(m.seconds_left())
            };
            let mut line = SlotLine::new(SlotKind::Holding, label, m.label.clone())
                .with_icon("message")
                .with_hint("⌘Z", "undo")
                .with_hint("↑", "more");
            if live > 1 {
                line = line.with_sub(format!("+ {} more", live - 1));
            }
            line.progress = Some(m.progress());
            return line;
        }
        if let Some(e) = self.ledger.last() {
            let mut parts: Vec<String> = Vec::new();
            match e.state {
                EntryState::Undone if e.held.is_some() => parts.push(words::NOT_SENT.into()),
                EntryState::Sent => parts.push(words::SENT.into()),
                _ => {}
            }
            if let Some(name) = &e.script {
                parts.push(format!("saved as “{name}”"));
            }
            let when = if e.seeded {
                e.when.clone().unwrap_or_default()
            } else {
                words::JUST_NOW.into()
            };
            if !when.is_empty() {
                parts.push(when);
            }
            return SlotLine::new(SlotKind::Recent, words::LAST, e.title.clone())
                .with_icon(e.icon.clone())
                .with_sub(parts.join(" · "))
                .with_hint("↑", "");
        }
        let f = &self.focused;
        SlotLine::new(SlotKind::Focused, words::FOCUSED, f.title.clone())
            .with_icon(if f.icon.is_empty() {
                "window".to_string()
            } else {
                f.icon.clone()
            })
            .with_sub(f.sub.clone())
            .with_hint("⌘K", "act on it")
    }

    pub fn enter_target(&self, now: Ms) -> EnterTarget {
        match self.views.top() {
            Some(top) => top
                .enter
                .clone()
                .unwrap_or_else(|| EnterTarget::new(top.name.clone(), "")),
            None => self.list.target(now),
        }
    }

    pub fn snapshot(&self, now: Ms) -> Snapshot {
        let view = self.views.top().map(|t| t.name.clone());
        let slot = self.slot_line(now);
        Snapshot {
            slot_opacity: self.slot_opacity(&slot),
            slot,
            header: if view.is_some() {
                String::new()
            } else {
                self.list.header()
            },
            crumb: self.views.crumb(),
            view,
            enter: self.enter_target(now),
            armed: self.guard.armed(now),
            items: self.list.layout(),
            sel: self.list.sel,
            text: self.text.clone(),
        }
    }

    /// Context (what's focused, what ran last) reads as background and fainter while you type; in a
    /// view everything steps back except a message waiting to go and words landing by voice.
    fn slot_opacity(&self, slot: &SlotLine) -> f32 {
        use crate::contract::slot_opacity as o;
        let typing = !self.text.trim().is_empty();
        match slot.kind {
            SlotKind::Holding => 1.0,
            SlotKind::Typing if !self.views.is_empty() => 1.0,
            _ if !self.views.is_empty() => o::IN_VIEW_NOT_HOLDING,
            SlotKind::Focused if typing => o::TYPING_FOCUSED,
            SlotKind::Focused => o::FOCUSED,
            SlotKind::Recent if typing => o::TYPING_RECENT,
            SlotKind::Recent => o::RECENT,
            SlotKind::Staged | SlotKind::Typing => 1.0,
        }
    }

    /// Whether Enter pressed now would be ignored.
    pub fn enter_armed(&self, now: Ms) -> bool {
        self.guard.armed(now)
    }

    /// How long the guard has left, for the ↵ button's fill.
    pub fn enter_armed_left(&self, now: Ms) -> Ms {
        self.guard
            .armed_until()
            .saturating_sub(now)
            .min(ENTER_GUARD_MS)
    }

    pub fn held_state(&self, id: u64) -> Option<HeldState> {
        self.holds.get(id).map(|h| h.state)
    }
}
