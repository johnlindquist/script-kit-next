//! The one window as a reducer: events in, effects out, time passed in.
//!
//! The host owns pixels, processes and Jev; this owns every rule about what the line above the
//! input says, which header leads the list, what Enter runs and when it is ignored, and what
//! waits before it goes. The window's height is not one of them: it is fixed while the window
//! shows. Nothing here reads a clock.

mod chat_work;
mod chips;
mod clipboard;
mod finishing;
mod keys;
mod learning;
mod ledger_rules;
mod show;
mod staging;
pub mod subject_tabs;
#[cfg(test)]
mod subject_tabs_tests;
use subject_tabs::{Tabs, TabsEvent};
mod talking;
#[cfg(test)]
mod tests;
mod text;
mod time;
mod view_stage;
mod views;
mod work;

use crate::Ms;
use crate::contract::{
    DISMISS_SETTLE_MS, ENTER_GUARD_MS, JEV_DEBOUNCE_MS, MAX_FALLBACKS_WITH_MATCHES, MAX_MATCHES,
    words,
};
use crate::enter::{EnterGuard, EnterTarget};
use crate::hold::{Held, HeldChange, HeldState, Holds};
use crate::ledger::{Entry, EntryState, Ledger, NewEntry, Risk, risk_of};
use crate::list::{Item, ListState};
use crate::row::{Row, RowAct};
use crate::slot::{ContextChip, SlotKind, SlotLine, SlotStyle, SubjectChip};
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
    /// What you act on is a chip in the input: what's focused is offered at its end while it is
    /// empty, and Space or ⌘K attach it at its start; ↑ attaches the last run. The line above the
    /// input only reports what's waiting, news and the last run. In the model's chat the line
    /// names its latest reply. The prototype's traces were recorded without it: one line shows
    /// everything, including what's staged or focused.
    pub subject_chip: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hold_ms: crate::contract::MESSAGE_HOLD_MS,
            jev_first: true,
            stale_holds: true,
            auto_save: true,
            dismiss_on_launch: true,
            subject_chip: true,
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
    /// A space on an empty input.
    Space,
    /// The receipt of what just happened was clicked: it goes in play, or lets go if it is.
    Recent,
    CmdZ,
    Tab,
    BackTab,
    CmdDigit(u8),
    Right,
    /// ← with the caret at the input's start: the chip before it is highlighted.
    Left,
    /// ⌫ with the caret at the input's start: the highlighted chip, or the last one, goes.
    Backspace,
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
    Home {
        text: String,
    },
    Park,
    Reopen(ViewSpec),
    /// The open view's crumb changed words; nothing opened.
    Label(String),
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
    /// The host kept a way to take it back. Only then does ⌘Z offer it: a run that could be
    /// undone but that nobody kept a way to undo never says "Undid".
    pub undo_kept: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// The input's words changed because the user typed.
    Text(String),
    Key(Key),
    Subjects(TabsEvent),
    /// `@` finished naming something: it becomes a chip after the subject. The host takes its
    /// token out of the input's words and keeps it at the sentence's end.
    Attach(ContextChip),
    /// The input's words were replaced wholesale; what `@` attached goes with them.
    DropChips,
    /// The user put the highlight on a row (a click or hover), by its place among the rows.
    Select(usize),
    /// Name matches for the list's words (fallback rows marked as such). While a view that takes
    /// input is open the input holds the view's words, so names searched for them are ignored.
    Names(Vec<Row>),
    /// Rows the host puts above everything for the current words.
    Pinned(Vec<Row>),
    /// What the empty input lists when nothing else is pinned.
    Suggested(Vec<Row>),
    /// Jev's readings of `text`; they land only when `text` is the list's words.
    Jev {
        text: String,
        rows: Vec<Row>,
    },
    /// Jev couldn't read `text`; like [`Event::Jev`], only for the list's words.
    JevDown {
        text: String,
    },
    View(ViewOp),
    /// The open view's Enter changed.
    ViewEnter(EnterTarget),
    /// A view under the top one changed its Enter; ↵ says it when that view is on top again.
    ViewEnterBelow {
        view: String,
        target: EnterTarget,
    },
    Done(Done),
    /// A hosted view requested Undo for its own completed operation.
    UndoEntry(u64),
    StopHold(u64),
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
    /// The window went away without running anything (the user switched to another app); the
    /// host brings it home when it resets.
    Dismissed,
    Summon,
    Focused(Subject),
    /// The staged subject's actions, as the host lists them. A row with the key of one the window
    /// offered itself (undo, run again, share…) keeps doing what the window does for it.
    Actions(Vec<Row>),
    /// Shows a line above the input until cleared; one a view set also goes when no view is open.
    Line(Option<SlotLine>),
    Tick,
    /// Something happened to the model's chat, the steps, a service or an agent.
    Work(crate::work::WorkEvent),
    /// Holding the voice key.
    Talk(crate::talk::TalkEvent),
    /// Word from the app (a script's `toast()` or `hud()`, a failure): it lands in the ledger.
    Notice(crate::notice::Notice),
    /// The host is about to run something that can't be taken back: it waits in the line first.
    Irreversible(crate::notice::Irreversible),
    /// A view put something in the line (history's Enter stages the entry it's on).
    Stage(Subject),
    /// ⌘V with this text on the clipboard.
    Paste(String),
    /// ⌘K in a view: what it has focused, and the actions it lists for it.
    StageView {
        subject: Subject,
        rows: Vec<Row>,
    },
    /// The view stopped acting on what it staged.
    LetGo,
    /// Saving this entry's run as a script failed on disk: it no longer says "saved as".
    ScriptNotSaved(u64),
}

impl Event {
    fn is_user(&self) -> bool {
        matches!(
            self,
            Event::Text(_)
                | Event::Key(_)
                | Event::Select(_)
                | Event::Summon
                | Event::Paste(_)
                | Event::Subjects(TabsEvent::Pick(_) | TabsEvent::Paste(_))
        )
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
    /// Work for the host: ask the model, read or run steps, sign in, start or park an agent.
    Work(crate::work::WorkEffect),
    /// Ask Jev whether the heard words are writing or a request; `last` for the final words.
    TalkAsk {
        words: String,
        last: bool,
    },
    /// Paste the words at the front app's cursor.
    Paste {
        text: String,
        app: String,
    },
    /// The voice session is over: stop listening and let go of what was heard.
    TalkEnded,
    /// Something new was learned; the host keeps it for the next session.
    Learned,
    /// Add the words at the end of today's note.
    AddToNote {
        text: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outcome {
    pub effects: Vec<Effect>,
    pub consumed: bool,
    /// The row Enter or ⌘1–3 ran, whether the host runs it ([`Effect::Run`]) or the window does
    /// (undo, send now, share…).
    pub picked: Option<(Row, RunHow)>,
}

/// What the window shows, for the host to draw and traces to compare.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub slot: SlotLine,
    /// 1 is live; below 1 the line reads as background.
    pub slot_opacity: f32,
    /// What you act on, as a chip in the input.
    pub chip: Option<SubjectChip>,
    pub header: String,
    pub view: Option<String>,
    pub crumb: String,
    pub enter: EnterTarget,
    pub armed: bool,
    pub items: Vec<Item>,
    pub sel: usize,
    pub text: String,
}

/// The row the highlight was on before something arrived, and the list it was in.
#[derive(Clone, Debug)]
struct Focus {
    text: String,
    staged: bool,
    owned: bool,
    item: Item,
    row: Row,
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
    tabs: Tabs,
    text: String,
    list: ListState,
    host_names: Vec<Row>,
    host_pinned: Vec<Row>,
    suggested: Vec<Row>,
    views: ViewStack,
    staged: Option<Staged>,
    line: Option<SlotLine>,
    /// The line was set by a view, and goes when no view is open.
    line_from_view: bool,
    focused: Subject,
    ledger: Ledger,
    holds: Holds,
    guard: EnterGuard,
    user_at: Option<Ms>,
    jev_due: Option<(Ms, String)>,
    dismiss: Option<PendingDismiss>,
    away: bool,
    steps: Option<(String, Vec<String>)>,
    picked: Option<(Row, RunHow)>,
    wake: Option<Ms>,
    now: Ms,
    work: crate::work::Work,
    lessons: crate::lessons::Lessons,
    talk: Option<crate::talk::Talk>,
    /// Voice sessions so far, so each one's Enter targets are new.
    talks: u64,
    /// Waiting things that are runs rather than messages.
    runs_held: Vec<u64>,
    /// ⇥ made the agent the Owner of what's typed.
    agent_owns: bool,
    /// The user moved the highlight since the words last changed: ⌘K acts on that row rather
    /// than on what the input's chip offers.
    chose: bool,
    /// What `@` attached, in order, as chips after the subject.
    context: Vec<ContextChip>,
    /// The chip ← highlighted, counted from the attached subject; ⌫ removes it.
    chip_at: Option<usize>,
    /// A failure said while a waiting thing held the line; it gets the line once nothing waits.
    missed: Option<u64>,
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
            tabs: Tabs::default(),
            text: String::new(),
            host_names: Vec::new(),
            host_pinned: Vec::new(),
            suggested: Vec::new(),
            views: ViewStack::default(),
            staged: None,
            line: None,
            line_from_view: false,
            focused: Subject {
                kind: SubjectKind::Window,
                label: "Front window".into(),
                icon: "window".into(),
                ..Subject::default()
            },
            ledger: Ledger::new(),
            guard: EnterGuard::new(),
            user_at: None,
            jev_due: None,
            dismiss: None,
            away: false,
            steps: None,
            picked: None,
            wake: None,
            now: 0,
            work: crate::work::Work::default(),
            lessons: crate::lessons::Lessons::default(),
            talk: None,
            talks: 0,
            runs_held: Vec::new(),
            agent_owns: false,
            chose: false,
            context: Vec::new(),
            chip_at: None,
            missed: None,
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
    /// The host released the work (and any open file handle) for an older run.
    /// Keep its history row, but never offer an inverse that can no longer run.
    pub fn release_undo(&mut self, id: u64) {
        if let Some(entry) = self.ledger.get_mut(id) {
            entry.undo_kept = false;
        }
    }
    pub fn holds(&self) -> &Holds {
        &self.holds
    }
    pub fn is_away(&self) -> bool {
        self.away
    }
    pub fn in_view(&self) -> bool {
        !self.views.is_empty()
    }

    /// ⇥ made the agent the Owner of what's typed: the list is the one row that starts it.
    pub fn agent_owns(&self) -> bool {
        self.agent_owns
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

    /// What Enter says and the row it would run; the ↵ label can lag the row until typing settles.
    fn enter_runs(&self, now: Ms) -> (EnterTarget, Option<String>) {
        let row = if self.in_view() {
            None
        } else {
            self.list.current_row().map(|r| r.key)
        };
        (self.enter_target(now), row)
    }

    fn focus(&self) -> Option<Focus> {
        if self.in_view() || self.away {
            return None;
        }
        Some(Focus {
            text: self.list.text.clone(),
            staged: self.staged.is_some(),
            owned: self.list.owned,
            item: self.list.current()?,
            row: self.list.focused()?.clone(),
        })
    }

    /// Something arrived that the user didn't do, after the turn they last acted in: in the same
    /// list, the highlighted row is still there and still highlighted. Only a staged subject's
    /// own rows can go (its message went out); then Enter waits a moment so a press meant for the
    /// old row runs nothing.
    fn keep_focus(&mut self, now: Ms, was: Focus) {
        let same = !self.in_view()
            && self.picked.is_none()
            && self.list.text == was.text
            && self.staged.is_some() == was.staged
            && self.list.owned == was.owned;
        if same && !self.list.keep(was.item, &was.row) {
            self.guard.arm(now);
        }
    }

    pub fn handle(&mut self, now: Ms, ev: Event) -> Outcome {
        let before_page = self.subject_page().map(|p| p.id);
        let now = now.max(self.now);
        self.now = now;
        let mut fx = Vec::new();
        if ev.is_user() {
            self.user_at = Some(now);
        }
        // A saved preference may finish loading after the subject was first painted. Its
        // implicit top action follows the new order; a row the user chose keeps its identity,
        // including when the settings callback arrives in the same turn as that choice.
        let new_rules = matches!(&ev, Event::Subjects(TabsEvent::Personal(_)));
        let untouched_top = new_rules && !self.chose && self.list.sel == 0;
        let focus = if ev.is_user() || (self.user_turn(now) && !new_rules) {
            None
        } else {
            self.focus()
        };
        self.tick(now, &mut fx);
        let consumed = self.apply(now, ev, &mut fx);
        let changed_page = before_page != self.subject_page().map(|p| p.id);
        if changed_page
            || (before_page
                .as_deref()
                .is_some_and(|id| id.starts_with("run-"))
                && self.text.is_empty())
        {
            self.refresh_rows();
        }
        let changed_default = untouched_top
            && focus.as_ref().is_some_and(|old| {
                old.row.key.starts_with("subject/")
                    && self.list.pinned.first().is_some_and(|new| {
                        new.key.starts_with("subject/") && new.key != old.row.key
                    })
            });
        if (changed_page || changed_default)
            && !self.chose
            && self.tabs_visible()
            && self.text.is_empty()
        {
            self.list.sel = 0;
            if focus.is_some() {
                self.guard.arm(now);
            }
        } else if let Some(focus) = focus {
            self.keep_focus(now, focus);
        }
        self.sync_pause(now, &mut fx);
        self.give_missed_its_turn();
        self.sync_staged(now);
        if self.line_from_view && !self.in_view() {
            self.line = None;
            self.line_from_view = false;
        }
        self.settle_lessons(&mut fx);
        self.schedule(now, &mut fx);
        Outcome {
            effects: fx,
            consumed,
            picked: self.picked.take(),
        }
    }

    fn apply(&mut self, now: Ms, ev: Event, fx: &mut Vec<Effect>) -> bool {
        match ev {
            Event::Text(text) => {
                self.on_text(now, text, true, fx);
                true
            }
            Event::Key(key) => self.on_key(now, key, fx),
            Event::Subjects(event) => self.subject_event(now, event),
            Event::Attach(chip) => self.attach(chip),
            Event::DropChips => {
                let had = !self.context.is_empty();
                self.drop_chips();
                had
            }
            Event::Select(pos) => self.on_select(pos),
            Event::Names(rows) => {
                if self.views.top().is_some_and(|top| top.takes_input) {
                    return false;
                }
                self.host_names = rows;
                self.refresh_rows();
                self.list.names_came();
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
                if text != self.list.text {
                    return false;
                }
                let mut rows = crate::list::fresh_readings(self.with_needs(rows), &self.list.names);
                if rows.is_empty() {
                    rows = crate::list::stand_in_rows(&text);
                }
                let on = self.list.current();
                let keep = self.list.focused().map(|row| row.key.clone());
                self.list.jev.assign(now, &text, rows, keep.as_deref());
                self.list.reselect(on, keep.as_deref());
                if let Some(i) = self.list.held.take() {
                    self.list.asking = false;
                    if let Some(row) = self.list.jev.get(i).and_then(|s| s.row.clone()) {
                        let readings: Vec<Row> = (0..self.list.jev.len())
                            .filter_map(|j| self.list.jev.get(j).and_then(|s| s.row.clone()))
                            .collect();
                        if crate::list::unsure(&row, &readings) {
                            self.list.asking = true;
                        } else {
                            self.run(now, row, RunHow::Slot, fx);
                        }
                    }
                }
                true
            }
            Event::JevDown { text } => {
                if text != self.list.text || self.list.jev.read(&text) {
                    return false;
                }
                let before = self.enter_runs(now);
                let on = self.list.current();
                let focused = self.list.focused().cloned();
                self.list
                    .jev
                    .down(now, &text, focused.as_ref().map(|row| row.key.as_str()));
                self.list.held = None;
                self.list.asking = false;
                self.refresh_rows();
                match (on, focused) {
                    (Some(item), Some(row)) => {
                        self.list.keep(item, &row);
                    }
                    (on, _) => self.list.reselect(on, None),
                }
                if self.enter_runs(now) != before {
                    let user = self.user_turn(now);
                    self.guard.arm_unless_user(now, user);
                }
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
            Event::ViewEnterBelow { view, target } => match self.views.below_mut(&view) {
                Some(frame) => {
                    frame.enter = Some(target);
                    true
                }
                None => false,
            },
            Event::StopHold(id) => {
                if !self.holds.live().any(|h| h.id == id) {
                    return false;
                }
                self.unsend(now, &[id], fx);
                true
            }
            Event::UndoEntry(id) => {
                if self.ledger.undo(now, id).is_some() {
                    fx.push(Effect::Undo { entry: id });
                    true
                } else {
                    false
                }
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
            Event::Dismissed => {
                self.away = true;
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
                    staged.rows = rows
                        .into_iter()
                        .map(|mut row| {
                            if let Some(own) = staged.rows.iter().find(|o| o.key == row.key) {
                                row.act = own.act;
                            }
                            row
                        })
                        .collect();
                    staged.waiting_for_rows = false;
                    self.refresh_rows();
                }
                true
            }
            Event::Line(line) => {
                self.line_from_view = line.is_some() && self.in_view();
                self.line = line;
                true
            }
            Event::Tick => true,
            Event::Work(ev) => self.on_work(now, ev, fx),
            Event::Talk(ev) => self.on_talk(now, ev, fx),
            Event::Notice(notice) => {
                self.on_notice(now, notice);
                true
            }
            Event::Irreversible(run) => {
                self.on_irreversible(now, run, fx);
                true
            }
            Event::Stage(subject) => {
                self.stage(now, subject, None, words::STAGED, fx);
                true
            }
            Event::Paste(text) => self.on_paste(now, text, fx),
            Event::StageView { subject, rows } => {
                self.stage_from_view(subject, rows);
                true
            }
            Event::LetGo => self.let_go(),
            Event::ScriptNotSaved(entry) => {
                self.ledger.forget_script(entry);
                true
            }
        }
    }
}
