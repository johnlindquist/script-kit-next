//! The one window as a reducer: events in, effects out, time passed in.
//!
//! The host owns pixels, processes and Jev; this owns every rule about what the line above the
//! input says, which header leads the list, what Enter runs and when it is ignored, how the
//! window's height moves, and what waits before it goes. Nothing here reads a clock.

mod finishing;
mod keys;
mod show;
mod staging;
#[cfg(test)]
mod tests;
mod text;
mod time;
mod views;

use crate::Ms;
use crate::contract::{
    DISMISS_SETTLE_MS, ENTER_GUARD_MS, JEV_DEBOUNCE_MS, MAX_MATCHES, ROW_ARM_MS, words,
};
use crate::enter::{EnterGuard, EnterTarget};
use crate::height::{ContentCause, HeightDecision, HeightGovernor};
use crate::hold::{Held, HeldChange, HeldState, Holds};
use crate::ledger::{Entry, EntryState, Ledger, NewEntry, Risk, risk_of};
use crate::list::{Item, ListState};
use crate::row::{Row, RowAct};
use crate::slot::{SlotKind, SlotLine, SlotStyle};
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// The input's words changed because the user typed.
    Text(String),
    Key(Key),
    /// The user put the highlight on a row (a click or hover), by its place among the rows.
    Select(usize),
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
    /// A view under the top one changed its Enter; ↵ says it when that view is on top again.
    ViewEnterBelow {
        view: String,
        target: EnterTarget,
    },
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
}

impl Event {
    /// Whether handling it draws the list again at the height it had.
    fn redraws_list(&self) -> bool {
        matches!(
            self,
            Event::Key(Key::Up | Key::Down | Key::Enter | Key::CmdDigit(_))
                | Event::Select(_)
                | Event::Jev { .. }
                | Event::JevDown { .. }
        )
    }

    fn is_user(&self) -> bool {
        matches!(
            self,
            Event::Text(_) | Event::Key(_) | Event::Select(_) | Event::Summon
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
    /// The line was set by a view, and goes when no view is open.
    line_from_view: bool,
    focused: Subject,
    ledger: Ledger,
    holds: Holds,
    height: HeightGovernor,
    guard: EnterGuard,
    user_at: Option<Ms>,
    /// Why the next natural height changed, and when it was set.
    cause: Option<(ContentCause, Ms)>,
    jev_due: Option<(Ms, String)>,
    /// When rows Jev just gave stop arming and the list draws again.
    redraws: Vec<Ms>,
    dismiss: Option<PendingDismiss>,
    away: bool,
    steps: Option<(String, Vec<String>)>,
    picked: Option<(Row, RunHow)>,
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
            line_from_view: false,
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
            redraws: Vec::new(),
            dismiss: None,
            away: false,
            steps: None,
            picked: None,
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
        let redraws = ev.redraws_list();
        let consumed = self.apply(now, ev, &mut fx);
        if consumed && redraws {
            self.list_redrawn(now, &mut fx);
        }
        self.sync_pause(now, &mut fx);
        self.sync_staged(now);
        if self.line_from_view && !self.in_view() {
            self.line = None;
            self.line_from_view = false;
        }
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
            Event::Select(pos) => self.on_select(pos),
            Event::Names(rows) => {
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
                if text != self.text {
                    return false;
                }
                let rows = crate::list::fresh_readings(rows, &self.list.names);
                self.list.jev.assign(now, &text, rows);
                self.redraws.push(now + ROW_ARM_MS);
                self.list.move_sel(0);
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
                if text != self.text {
                    return false;
                }
                self.list.jev.down(now, &text);
                self.list.held = None;
                self.list.asking = false;
                self.refresh_rows();
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
            Event::ViewEnterBelow { view, target } => match self.views.below_mut(&view) {
                Some(frame) => {
                    frame.enter = Some(target);
                    true
                }
                None => false,
            },
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
        }
    }
}
