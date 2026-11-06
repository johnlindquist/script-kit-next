//! The ledger's side of the main window: what waits before it runs, what ⌘Z takes back, what the
//! app said, what ran today, and what still needs setting up.
//!
//! The reducer decides all of it. This keeps the work each decision stands for (the run a wait
//! holds, the undo a change offers, the row a run came from) and does it when the reducer says.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use gpui::Context;
use sk_one_window::contract::ledger_words;
use sk_one_window::notice::{Irreversible, Notice};
use sk_one_window::setup::{needs_setup_rows, Missing, Need, Permission, BUN_KEY, CONTINUE_KEY};
use sk_one_window::stage::entry_subject;
use sk_one_window::{history, Done, Effect, Event, HeldOp, Key, Risk, Row, ViewOp};

use crate::list_item::GroupedListItem;
use crate::scripts::SearchResult;
use crate::ScriptListApp;

/// Work the reducer will say when to do: a run that waited, or an undo.
pub(crate) type LedgerRun = Box<dyn FnOnce(&mut ScriptListApp, &mut Context<ScriptListApp>)>;

/// Rows the reducer lists itself (history entries, setup, staged actions) are drawn as built-in
/// rows whose id carries this prefix and the row's key.
const OWN_ROW_PREFIX: &str = "one-window/";

/// A wait's work: what runs when it runs out, and what runs if it's stopped.
struct Held {
    run: LedgerRun,
    stopped: Option<LedgerRun>,
}

/// A question staged in the line: its choices are rows, and letting go of it answers `cancel`.
struct Ask {
    title: String,
    choices: HashMap<String, LedgerRun>,
    cancel: Option<LedgerRun>,
}

#[derive(Default)]
pub(crate) struct LedgerHost {
    /// A run handed to the reducer this turn; its `Held` `Start` names the wait it belongs to.
    pending: Option<Held>,
    held: HashMap<u64, Held>,
    undo: HashMap<u64, LedgerRun>,
    /// What each title ran, so "Run it again" can run it.
    reruns: HashMap<String, SearchResult>,
    ask: Option<Ask>,
    /// Bumped whenever the line says something, so a run that failed can tell whether it said why.
    pub(super) said: u64,
}

const ASK_PREFIX: &str = "ask:";

/// The reducer row a drawn built-in row stands for.
pub(crate) fn own_row_key(builtin_id: &str) -> Option<&str> {
    builtin_id.strip_prefix(OWN_ROW_PREFIX)
}

/// A reducer row drawn as a launcher row.
pub(crate) fn own_row_result(row: &Row) -> SearchResult {
    SearchResult::BuiltIn(crate::scripts::BuiltInMatch {
        authored_label: row.authored_label,
        entry: crate::builtins::BuiltInEntry {
            id: format!("{OWN_ROW_PREFIX}{}", row.key),
            name: row.title.clone(),
            description: row.sub.clone().unwrap_or_default(),
            keywords: Vec::new(),
            feature: crate::builtins::BuiltInFeature::RunHistory,
            icon: row.icon.clone(),
            group: crate::builtins::BuiltInGroup::Core,
        },
        score: 0,
        match_evidence: None,
    })
}

/// A permission as its "Needs setup" row shows it.
fn setup_permission(
    kind: crate::permissions_wizard::PermissionKind,
    status: crate::platform::permiso_detect::PermissionStatus,
) -> Permission {
    use crate::permissions_wizard::PermissionRequirement;
    use crate::platform::permiso_detect::PermissionStatus;
    let granted = match status {
        PermissionStatus::Authorized => Some(true),
        PermissionStatus::Unknown => None,
        PermissionStatus::Denied | PermissionStatus::NotDetermined => Some(false),
    };
    Permission {
        id: crate::permissions::kind_code(kind),
        name: kind.name(),
        icon: kind.icon(),
        need: match kind.requirement() {
            PermissionRequirement::Required => Need::Required,
            PermissionRequirement::Recommended => Need::Recommended,
            PermissionRequirement::Optional => Need::Optional,
        },
        granted,
        about: if granted == Some(false) && crate::permissions::grant_looks_stuck(kind) {
            "Already on in System Settings? Press ⌘R to restart Script Kit".to_string()
        } else if granted == Some(false) {
            kind.missing_consequence().to_string()
        } else {
            kind.subtitle().to_string()
        },
    }
}

/// Rows as the launcher's grouped list: each row's head becomes a section header.
fn grouped_rows(rows: &[Row]) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    let mut grouped = Vec::with_capacity(rows.len() + 1);
    let mut flat = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(head) = &row.head {
            grouped.push(GroupedListItem::SectionHeader(head.clone(), None));
        }
        grouped.push(GroupedListItem::Item(flat.len()));
        flat.push(own_row_result(row));
    }
    (grouped, flat)
}

impl ScriptListApp {
    /// The app has something to say. The line above the input says it now, or when the window
    /// comes back.
    ///
    /// A failure reads "<what didn't happen>: <the error>": the line says only the words before
    /// the colon, and the error goes to the log (`sk_one_window::notice::failure_words`).
    pub(crate) fn one_window_notice(
        &mut self,
        icon: &str,
        text: impl Into<String>,
        cx: &mut Context<Self>,
    ) {
        let text = text.into();
        if icon == crate::line_notice::FAILED {
            if let (sentence, Some(detail)) = sk_one_window::notice::failure_words(&text) {
                tracing::warn!(
                    category = "NOTICE",
                    notice = %sentence,
                    detail = %detail,
                    "Failure said in the line"
                );
            }
        }
        self.one_window_host.ledger.said += 1;
        self.one_window_handle(Event::Notice(Notice::new(icon, text)), cx);
    }

    /// Something that can't be taken back: it waits in the line, and `run` happens when the wait
    /// runs out unless ⌘Z or "Don't send it" stops it.
    pub(crate) fn one_window_hold(
        &mut self,
        icon: &str,
        title: impl Into<String>,
        run: LedgerRun,
        cx: &mut Context<Self>,
    ) {
        self.one_window_hold_or_stop(icon, title, run, None, cx);
    }

    /// Like [`Self::one_window_hold`], and `stopped` runs if the wait is stopped.
    pub(crate) fn one_window_hold_or_stop(
        &mut self,
        icon: &str,
        title: impl Into<String>,
        run: LedgerRun,
        stopped: Option<LedgerRun>,
        cx: &mut Context<Self>,
    ) {
        self.one_window_host.ledger.pending = Some(Held { run, stopped });
        self.one_window_handle(
            Event::Irreversible(Irreversible::new(icon, title.into())),
            cx,
        );
        self.one_window_host.ledger.pending = None;
    }

    /// A question with a few answers: it goes in the line and its answers become the rows.
    /// Letting go of it (Esc) runs `cancel`.
    pub(crate) fn one_window_ask(
        &mut self,
        icon: &str,
        title: impl Into<String>,
        detail: impl Into<String>,
        choices: Vec<(String, LedgerRun)>,
        cancel: Option<LedgerRun>,
        cx: &mut Context<Self>,
    ) {
        self.one_window_let_go_of_ask(cx);
        self.one_window_host.ledger.said += 1;
        let title = title.into();
        let mut rows = Vec::with_capacity(choices.len());
        let mut runs = HashMap::with_capacity(choices.len());
        let question = uuid::Uuid::new_v4();
        for (i, (label, run)) in choices.into_iter().enumerate() {
            let key = format!("{ASK_PREFIX}{question}:{i}");
            rows.push(Row::new(key.clone(), label.clone()).with_enter_label(label));
            runs.insert(key, run);
        }
        self.one_window_host.ledger.ask = Some(Ask {
            title: title.clone(),
            choices: runs,
            cancel,
        });
        self.one_window_handle(
            Event::Stage(sk_one_window::Subject {
                kind: sk_one_window::SubjectKind::Question,
                label: sk_one_window::contract::words::ACTING_ON.to_string(),
                title,
                sub: detail.into(),
                icon: icon.to_string(),
                ..sk_one_window::Subject::default()
            }),
            cx,
        );
        self.one_window_handle(Event::Actions(rows), cx);
        self.one_window_republish_ledger("one_window_ask", cx);
    }

    /// Takes back a question that no longer needs an answer, without answering it.
    pub(crate) fn one_window_withdraw_ask(&mut self, cx: &mut Context<Self>) {
        let Some(ask) = self.one_window_host.ledger.ask.take() else {
            return;
        };
        let staged = self
            .one_window
            .staged()
            .is_some_and(|staged| staged.subject.title == ask.title);
        if staged {
            self.one_window_handle(Event::Key(Key::Escape), cx);
            self.one_window_republish_ledger("one_window_ask_withdrawn", cx);
        }
    }

    /// A staged question the user let go of is answered with its cancel, and an alias editor
    /// whose view went away closes.
    pub(crate) fn one_window_let_go(&mut self, cx: &mut Context<Self>) {
        if self.alias_input_state.is_some() && !self.alias_view_on_top() {
            self.close_alias_input(cx);
        }
        self.one_window_let_go_of_ask(cx);
    }

    fn one_window_let_go_of_ask(&mut self, cx: &mut Context<Self>) {
        let still_staged = match &self.one_window_host.ledger.ask {
            None => return,
            Some(ask) => self
                .one_window
                .staged()
                .is_some_and(|staged| staged.subject.title == ask.title),
        };
        if still_staged {
            return;
        }
        if let Some(cancel) = self
            .one_window_host
            .ledger
            .ask
            .take()
            .and_then(|ask| ask.cancel)
        {
            cancel(self, cx);
        }
    }

    /// Something that happened at once and can be taken back: the line offers ⌘Z, and `undo`
    /// runs if the user takes it.
    /// A File Search change: the line offers ⌘Z from the main menu, and ⌘Z inside File Search
    /// takes back the latest one. Whichever runs first runs the undo; the other finds it gone.
    pub(crate) fn file_search_done_undoable(
        &mut self,
        icon: &str,
        title: impl Into<String>,
        undo: LedgerRun,
        cx: &mut Context<Self>,
    ) {
        self.file_search_last_undo = self.record_one_window_undo(icon, title, undo, cx);
    }

    /// ⌘Z in File Search consumes the same entry and callback as the line.
    pub(crate) fn file_search_undo_last(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(id) = self.file_search_last_undo.take() else {
            return false;
        };
        self.one_window_handle(Event::UndoEntry(id), cx).consumed
    }

    pub(crate) fn one_window_done_undoable(
        &mut self,
        icon: &str,
        title: impl Into<String>,
        undo: LedgerRun,
        cx: &mut Context<Self>,
    ) {
        self.record_one_window_undo(icon, title, undo, cx);
    }

    fn record_one_window_undo(
        &mut self,
        icon: &str,
        title: impl Into<String>,
        undo: LedgerRun,
        cx: &mut Context<Self>,
    ) -> Option<u64> {
        let title = title.into();
        self.one_window_handle(
            Event::Done(Done {
                icon: icon.to_string(),
                title: title.clone(),
                risk: Some(Risk::Undo),
                undo_kept: true,
                ..Done::default()
            }),
            cx,
        );
        let kept = self
            .one_window
            .ledger()
            .last()
            .filter(|e| e.title == title && e.can_undo())
            .map(|e| e.id);
        if let Some(id) = kept {
            // Keep the history, but not an unbounded collection of closures, open
            // file descriptors and potentially unlinked files in a resident app.
            let callbacks = &mut self.one_window_host.ledger.undo;
            callbacks.insert(id, undo);
            if callbacks.len() > super::lessons_host::RUNS_REMEMBERED {
                if let Some(oldest) = callbacks.keys().copied().min() {
                    callbacks.remove(&oldest);
                    self.one_window.release_undo(oldest);
                }
            }
        }
        kept
    }

    /// Effects that stand for work the ledger keeps: a wait running out, an undo, a rerun, and
    /// the keys of the history view.
    pub(crate) fn one_window_ledger_effect(&mut self, effect: Effect, cx: &mut Context<Self>) {
        match effect {
            Effect::Held { op, id, label, .. } => {
                let ledger = &mut self.one_window_host.ledger;
                let run = match op {
                    HeldOp::Start => {
                        if let Some(held) = ledger.pending.take() {
                            ledger.held.insert(id, held);
                        }
                        None
                    }
                    HeldOp::Sent if id == 0 => ledger.pending.take().map(|held| held.run),
                    HeldOp::Sent => ledger.held.remove(&id).map(|held| held.run),
                    HeldOp::Unsent => ledger.held.remove(&id).and_then(|held| held.stopped),
                    HeldOp::Paused | HeldOp::Resumed | HeldOp::Rewritten => None,
                };
                tracing::info!(
                    target: "script_kit::one_window",
                    event = "held",
                    op = ?op,
                    id,
                    label = %label,
                    runs = run.is_some(),
                );
                if let Some(run) = run {
                    run(self, cx);
                }
            }
            Effect::Undo { entry } => {
                if self.file_search_last_undo == Some(entry) {
                    self.file_search_last_undo = None;
                }
                match self.one_window_host.ledger.undo.remove(&entry) {
                    Some(undo) => undo(self, cx),
                    None => {
                        tracing::info!(target: "script_kit::one_window", event = "undo_not_kept", entry)
                    }
                }
            }
            Effect::Again { entry } => self.one_window_run_again(entry, cx),
            Effect::ViewKey(key) if self.one_window_in_history() => {
                self.one_window_history_key(key, cx)
            }
            Effect::ViewKey(key) if self.alias_view_on_top() => self.alias_view_key(key, cx),
            Effect::ViewKey(key) => self.one_window_view_key(key, cx),
            effect => tracing::info!(
                target: "script_kit::one_window",
                event = "effect_not_hosted",
                effect = ?effect,
            ),
        }
    }

    /// The launcher row a reducer row ran, kept by title for "Run it again".
    pub(crate) fn one_window_remember_rerun(&mut self, row: &Row) {
        let Some((_, idx)) = self.live_script_list_flat_selection_for_submit() else {
            return;
        };
        if let Some(result) = self
            .main_menu_result_caches
            .cloned_search_result_for_flat_index(idx)
        {
            self.one_window_host
                .ledger
                .reruns
                .insert(row.title.clone(), result);
        }
    }

    fn one_window_run_again(&mut self, entry: u64, cx: &mut Context<Self>) {
        let result = self
            .one_window
            .ledger()
            .get(entry)
            .and_then(|e| self.one_window_host.ledger.reruns.get(&e.title))
            .cloned();
        match result {
            Some(SearchResult::Script(m)) => self.execute_interactive(&m.script, cx),
            Some(SearchResult::Scriptlet(m)) => self.execute_scriptlet(&m.scriptlet, cx),
            Some(SearchResult::BuiltIn(m)) => {
                self.execute_builtin(&m.entry, cx);
            }
            Some(SearchResult::App(m)) => {
                self.execute_app(&m.app, cx);
            }
            _ => tracing::info!(target: "script_kit::one_window", event = "again_not_kept", entry),
        }
    }

    /// While the shortcut view records, every key is the shortcut, so no launcher binding
    /// acts on it first. Otherwise ⌘Z in the main window takes back what the line offers
    /// before the input's own undo sees it. Interceptors run ahead of the focused input's
    /// key bindings.
    pub(crate) fn install_key_owner_interceptor(&mut self, cx: &mut Context<Self>) {
        let app = cx.weak_entity();
        let interceptor = cx.intercept_keystrokes(move |event, window, cx| {
            let keys = &event.keystroke;
            let recorded = app
                .update(cx, |app, cx| app.one_window_shortcut_keystroke(keys, cx))
                .unwrap_or(false);
            if recorded {
                cx.stop_propagation();
                return;
            }
            let cmd_z = keys.key.eq_ignore_ascii_case("z")
                && keys.modifiers.platform
                && !keys.modifiers.shift
                && !keys.modifiers.alt
                && !keys.modifiers.control;
            if !cmd_z {
                return;
            }
            let handle = window.window_handle();
            let taken = app
                .update(cx, |app, cx| app.one_window_line_undo(handle, cx))
                .unwrap_or(false);
            if taken {
                cx.stop_propagation();
            }
        });
        self.gpui_input_subscriptions.push(interceptor);
    }

    fn one_window_line_undo(
        &mut self,
        window: gpui::AnyWindowHandle,
        cx: &mut Context<Self>,
    ) -> bool {
        self.one_window_host
            .window
            .is_none_or(|main| main == window)
            && self.one_window_owns_list()
            && self.filter_text.is_empty()
            && self.one_window.can_take_back()
            && self.one_window_key(Key::CmdZ, cx).taken()
    }

    // ------------------------------------------------------------------ history

    pub(crate) fn one_window_open_history(&mut self, cx: &mut Context<Self>) {
        self.one_window_handle(Event::View(ViewOp::Open(history::view_spec())), cx);
        self.one_window_republish_ledger("one_window_history", cx);
    }

    fn one_window_in_history(&self) -> bool {
        self.one_window
            .views()
            .top()
            .is_some_and(|top| top.name == history::VIEW)
    }

    fn one_window_entry_rows(&self, text: &str) -> Vec<Row> {
        history::history_rows(
            &self.one_window.ledger().search(text),
            self.one_window_clock(),
        )
    }

    /// ↑/↓ move the history view's highlight; Enter puts its entry in the line.
    fn one_window_history_key(&mut self, key: Key, cx: &mut Context<Self>) {
        let rows: Vec<(usize, usize)> = self
            .main_menu_committed_rows()
            .iter()
            .filter_map(|row| row.selectable_ordinal.map(|at| (at, row.grouped_index)))
            .collect();
        let at = rows
            .iter()
            .position(|(_, grouped)| *grouped == self.selected_index);
        match key {
            Key::Up | Key::Down => {
                let next = match (key, at) {
                    (Key::Up, Some(at)) => at.saturating_sub(1),
                    (Key::Down, Some(at)) => (at + 1).min(rows.len().saturating_sub(1)),
                    _ => 0,
                };
                if let Some((_, grouped)) = rows.get(next) {
                    self.set_selected_index(*grouped, "one_window_history", cx);
                }
            }
            Key::Enter => {
                let text = self.one_window.text().to_string();
                let picked = self
                    .one_window_entry_rows(&text)
                    .into_iter()
                    .nth(at.unwrap_or(0));
                if let Some(row) = picked {
                    self.one_window_stage_entry_row(&row, cx);
                }
            }
            _ => {}
        }
    }

    fn one_window_stage_entry_row(&mut self, row: &Row, cx: &mut Context<Self>) {
        let Some(entry) = history::entry_id(&row.key)
            .and_then(|id| self.one_window.ledger().get(id))
            .cloned()
        else {
            return;
        };
        self.one_window_handle(
            Event::Stage(entry_subject(
                &entry,
                sk_one_window::contract::words::ACTING_ON,
            )),
            cx,
        );
        self.one_window_republish_ledger("one_window_staged", cx);
    }

    // ------------------------------------------------------------------ the list

    /// The list when the ledger lists its own rows: the alias view or the history view.
    pub(crate) fn one_window_ledger_list(
        &self,
        text: &str,
    ) -> Option<(Vec<GroupedListItem>, Vec<SearchResult>)> {
        if self.alias_view_on_top() {
            return Some(grouped_rows(&self.alias_view_rows()));
        }
        if !self.one_window_in_history() {
            return None;
        }
        let mut rows = self.one_window_entry_rows(text);
        if rows.is_empty() {
            return Some((
                vec![GroupedListItem::SectionHeader(
                    ledger_words::HISTORY_HEAD.to_string(),
                    None,
                )],
                Vec::new(),
            ));
        }
        rows[0].head = Some(ledger_words::HISTORY_HEAD.to_string());
        Some(grouped_rows(&rows))
    }

    /// What still needs setting up, to lead the empty list.
    pub(crate) fn one_window_setup_rows(&self) -> Vec<Row> {
        let mut missing: Vec<Missing> = self
            .bun_missing
            .then_some(Missing::Bun)
            .into_iter()
            .collect();
        missing.extend(
            self.permission_setup_list()
                .into_iter()
                .map(|(kind, status)| Missing::Permission(setup_permission(kind, status))),
        );
        needs_setup_rows(&missing)
    }

    /// The permissions "Needs setup" lists now, with their state.
    pub(crate) fn permission_setup_list(
        &self,
    ) -> Vec<(
        crate::permissions_wizard::PermissionKind,
        crate::platform::permiso_detect::PermissionStatus,
    )> {
        // Read the published observations here. Building rows (and hashing their
        // cache key) is a paint/search path, not a reason to query macOS.
        let statuses = self.main_services.owned_sources().map_or_else(
            || {
                crate::permissions_wizard::PermissionKind::all()
                    .iter()
                    .map(|&kind| {
                        (
                            kind,
                            crate::permissions::cached(kind).unwrap_or(
                                crate::platform::permiso_detect::PermissionStatus::Unknown,
                            ),
                        )
                    })
                    .collect()
            },
            |sources| sources.permissions.clone(),
        );
        crate::permissions_wizard::setup_list(
            &statuses,
            &self.permission_setup_declined,
            self.permission_setup_every,
        )
    }

    /// "Set Up Permissions": every permission leads the empty launcher with its state, until
    /// the user goes on without the rest. Nothing is listed when every one is already on.
    pub(crate) fn one_window_set_up_permissions(&mut self, cx: &mut Context<Self>) {
        if self.main_services.is_production() {
            crate::permissions::refresh_all();
        }
        self.permission_setup_every = true;
        if self.permission_setup_list().is_empty() {
            self.permission_setup_every = false;
            self.one_window_notice(crate::line_notice::INFO, "Every permission is on", cx);
            return;
        }
        if !matches!(self.current_view, crate::AppView::ScriptList) || !self.filter_text.is_empty()
        {
            self.reset_to_script_list(cx);
        }
        if self.main_services.is_production() {
            script_kit_gpui::request_show_main_window();
        }
        self.one_window_republish_ledger("permission_setup", cx);
        cx.notify();
    }

    /// Runs a "Needs setup" permission row: asks macOS, or opens where it's switched on.
    fn one_window_run_permission_row(
        &mut self,
        row: &Row,
        cx: &mut Context<Self>,
    ) -> Option<anyhow::Result<()>> {
        if row.key == CONTINUE_KEY {
            return Some(self.one_window_continue_without_permissions(cx));
        }
        let id = sk_one_window::setup::permission_id(&row.key)?;
        let Some(&kind) = crate::permissions_wizard::PermissionKind::all()
            .iter()
            .find(|kind| crate::permissions::kind_code(**kind) == id)
        else {
            return Some(Err(anyhow::anyhow!("That permission is no longer listed")));
        };
        self.execute_permissions_wizard_grant(kind, cx);
        self.one_window_republish_ledger("permission_requested", cx);
        Some(Ok(()))
    }

    /// The user goes on without the permissions still off: they stop leading the list and stop
    /// coming up at startup. ⌘Z brings them back.
    fn one_window_continue_without_permissions(
        &mut self,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let before = self.permission_setup_declined.clone();
        let was_every = self.permission_setup_every;
        let mut declined = before.clone();
        for (kind, status) in self.permission_setup_list() {
            if crate::permissions_wizard::is_missing(status) && !declined.contains(&kind) {
                declined.push(kind);
            }
        }
        crate::permissions_wizard::save_declined(&declined)
            .map_err(|error| anyhow::anyhow!("Couldn't save the permission choice: {error}"))?;
        self.permission_setup_declined = declined;
        self.permission_setup_every = false;
        tracing::info!(
            category = "PERMISSIONS",
            event = "permission_setup_continued",
            declined = ?self.permission_setup_declined,
        );
        self.one_window_done_undoable(
            "arrow-right",
            "Continued without some permissions",
            Box::new(
                move |app: &mut ScriptListApp, cx: &mut Context<ScriptListApp>| {
                    if let Err(error) = crate::permissions_wizard::save_declined(&before) {
                        app.one_window_notice(
                            crate::line_notice::FAILED,
                            format!("Couldn't bring the permissions back: {error}"),
                            cx,
                        );
                        return;
                    }
                    app.permission_setup_declined = before;
                    app.permission_setup_every = was_every;
                    app.one_window_republish_ledger("permission_setup_undone", cx);
                    app.spawn_permission_setup_refresh(cx);
                },
            ),
            cx,
        );
        self.one_window_republish_ledger("permission_setup_continued", cx);
        Ok(())
    }

    /// Changes whenever the reducer's own rows would draw differently.
    pub(crate) fn one_window_ledger_token(&self, hasher: &mut impl Hasher) {
        self.one_window
            .staged()
            .map(|s| &s.subject.title)
            .hash(hasher);
        self.one_window.staged().map(|s| s.rows.len()).hash(hasher);
        self.one_window.views().top().map(|v| &v.name).hash(hasher);
        self.one_window.ledger().entries().len().hash(hasher);
        self.one_window
            .ledger()
            .last()
            .map(|e| e.state)
            .hash(hasher);
        self.one_window
            .staged()
            .map(|s| s.waiting_for_rows)
            .hash(hasher);
        self.bun_missing.hash(hasher);
        for (kind, status) in self.permission_setup_list() {
            kind.hash(hasher);
            crate::permissions::status_code(status).hash(hasher);
            crate::permissions::grant_looks_stuck(kind).hash(hasher);
        }
    }

    /// "Needs setup" changed: the empty list draws it again.
    pub(crate) fn one_window_republish_setup(
        &mut self,
        reason: &'static str,
        cx: &mut Context<Self>,
    ) {
        self.one_window_republish_ledger(reason, cx);
    }

    fn one_window_republish_ledger(&mut self, reason: &'static str, cx: &mut Context<Self>) {
        let text = self.computed_filter_text.clone();
        if self.one_window_owns(&text) {
            self.commit_main_menu_results_refresh(reason, None, cx, |_, _| true);
        }
    }

    // ------------------------------------------------------------------ running

    /// A reducer row the host runs itself. Returns `None` for rows that aren't its own.
    pub(crate) fn one_window_run_ledger_row(
        &mut self,
        row: &Row,
        cx: &mut Context<Self>,
    ) -> Option<anyhow::Result<()>> {
        if let Some(ran) = self.one_window_run_permission_row(row, cx) {
            return Some(ran);
        }
        if row.key == BUN_KEY {
            let opened = self.open_bun_website();
            if let Err(error) = &opened {
                self.one_window_notice(
                    crate::line_notice::FAILED,
                    format!("Browser was not opened: {error}"),
                    cx,
                );
            }
            return Some(opened);
        }
        if history::entry_id(&row.key).is_some() {
            self.one_window_stage_entry_row(row, cx);
            return Some(Ok(()));
        }
        if row.key.starts_with(ASK_PREFIX) {
            let run = self
                .one_window_host
                .ledger
                .ask
                .take()
                .and_then(|mut ask| ask.choices.remove(&row.key));
            return Some(match run {
                Some(run) => {
                    run(self, cx);
                    Ok(())
                }
                None => Err(anyhow::anyhow!("one_window_question_answered")),
            });
        }
        None
    }

    /// A drawn reducer row was clicked or run by the launcher: the reducer runs it as if Enter
    /// were pressed on it.
    pub(crate) fn one_window_activate_own_row(
        &mut self,
        key: &str,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let rows: Vec<Row> = if self.one_window_in_history() {
            let text = self.one_window.text().to_string();
            self.one_window_entry_rows(&text)
        } else {
            let list = self.one_window.list();
            list.visible()
                .iter()
                .filter_map(|item| list.row(item).cloned())
                .collect()
        };
        let row = rows
            .into_iter()
            .find(|row| row.key == key)
            .ok_or_else(|| anyhow::anyhow!("one_window_row_not_listed"))?;
        if let Some(ran) = self.one_window_run_ledger_row(&row, cx) {
            return ran;
        }
        let list = self.one_window.list();
        let position = list
            .visible()
            .iter()
            .position(|item| list.row(item).is_some_and(|shown| shown.key == key))
            .ok_or_else(|| anyhow::anyhow!("one_window_row_not_listed"))?;
        self.one_window_handle(Event::Select(position), cx);
        self.one_window_key(Key::Enter, cx);
        Ok(())
    }
}
