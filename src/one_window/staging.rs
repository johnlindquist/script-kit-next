//! ⌘K in place. The reducer puts the highlighted row, or what's focused, in the line above the
//! input and asks for what can be done to it; the actions come from `crate::actions` and list as
//! the reducer's rows under its header. Nothing opens: no popup, no second window.

use gpui::{AppContext as _, Context};
use sk_one_window::{Event, Key, Row, Subject, SubjectKind};

use crate::scripts::SearchResult;
use crate::{AppView, ScriptListApp};

/// The key prefix of a row that runs an action on what's staged.
pub(crate) const ACTION_KEY: &str = "action:";

/// The action id of a window arrangement: `window/<op>@<app>`, the op's catalog id and the app
/// whose front window it arranges.
const WINDOW_ACTION_APP: char = '@';

/// The arrangements a staged window lists first, in this order; the rest follow in the catalog's.
const WINDOW_ACTIONS_FIRST: &[&str] = &[
    "Tile Left Half",
    "Tile Right Half",
    "Maximize",
    "Almost Maximize",
    "Center",
    "Move to Next Display",
    "Minimize",
    "Close Window",
    "Quit App",
];

/// What a staged window can have done to it: asking the model about it, then every arrangement,
/// each named for the window's app.
fn window_action_rows(app: &str) -> Vec<Row> {
    let mut ops = crate::intent::native_window_ops();
    ops.sort_by_key(|op| {
        WINDOW_ACTIONS_FIRST
            .iter()
            .position(|label| *label == op.label)
            .unwrap_or(WINDOW_ACTIONS_FIRST.len())
    });
    std::iter::once(super::chat_entry::about_row())
        .chain(ops.into_iter().map(|op| {
            Row::new(
                format!("{ACTION_KEY}{}{WINDOW_ACTION_APP}{app}", op.id),
                op.title_for(app),
            )
            .with_enter_label(op.label)
        }))
        .collect()
}

/// The arrangement and app a window action id names.
fn window_action(id: &str) -> Option<(crate::intent::WindowOp, String)> {
    let (op_id, app) = id.split_once(WINDOW_ACTION_APP)?;
    let op = crate::intent::native_window_ops()
        .into_iter()
        .find(|op| op.id == op_id)?;
    Some((op, app.to_string()))
}

/// The app a front-window subject belongs to: the line names the window by its title and the
/// app under it, or by the app alone when the window has no title of its own.
pub(crate) fn subject_app(subject: &Subject) -> Option<&str> {
    match subject.kind {
        SubjectKind::Window if !subject.sub.trim().is_empty() => Some(subject.sub.trim()),
        SubjectKind::Window | SubjectKind::App => {
            Some(subject.title.trim()).filter(|app| !app.is_empty())
        }
        _ => None,
    }
}

/// What was highlighted when something was staged: the words and the row, so an action runs on
/// the row it was offered for.
pub(crate) struct StagedFrom {
    text: String,
    row: Option<String>,
}

pub(super) fn action_row(action: &crate::actions::Action) -> Row {
    let row = Row::new(format!("{ACTION_KEY}{}", action.id), action.title.clone())
        .with_enter_label(action.title.clone());
    match action.description.as_deref().filter(|d| !d.is_empty()) {
        Some(sub) => row.with_sub(sub),
        None => row,
    }
}

impl ScriptListApp {
    /// ⌘K on the main window, from the key or its capsule.
    pub(crate) fn one_window_stage(&mut self, cx: &mut Context<Self>) -> bool {
        let row = self
            .main_menu_committed_rows()
            .iter()
            .find(|row| row.grouped_index == self.selected_index)
            .map(|row| row.stable_key.clone());
        self.one_window_host.staged_from = Some(StagedFrom {
            text: self.filter_text.clone(),
            row,
        });
        if !self.one_window.in_view() && matches!(self.selected_main_list_search_result_owned(),Some(crate::scripts::SearchResult::DictationHistory(_))){self.one_window_history_stage(cx);return true;}
        self.one_window_key(Key::CmdK, cx).taken()
    }

    /// The staged subject's actions, as rows: asking the model about it first, then what
    /// `crate::actions` offers. A row the list still highlights is the subject.
    pub(super) fn one_window_actions_for(&mut self, subject: &Subject) -> Event {
        if let Some(actions) = self.one_window_history_actions(subject) {
            return actions;
        }
        if let Some(app) = subject
            .row_key
            .is_none()
            .then(|| subject_app(subject))
            .flatten()
        {
            self.pending_root_unified_actions_subject = None;
            self.pending_root_file_actions_file = None;
            self.one_window_host.work.about = None;
            return Event::Actions(window_action_rows(app));
        }
        self.pending_root_unified_actions_subject = None;
        self.pending_root_file_actions_file = None;
        self.one_window_host.work.about = subject
            .row_key
            .as_ref()
            .and_then(|_| self.one_window_highlighted_part());
        let result = subject
            .row_key
            .as_ref()
            .and_then(|_| self.selected_main_list_search_result_owned());
        let actions = match result
            .as_ref()
            .map(crate::root_unified_result_actions::root_unified_action_owner_for_result)
        {
            Some(
                crate::root_unified_result_actions::RootUnifiedResultActionOwner::RootSubject(
                    owned,
                ),
            ) => {
                let actions =
                    crate::root_unified_result_actions::root_unified_actions_for_subject(&owned);
                if let crate::root_unified_result_actions::RootUnifiedActionSubject::File(file) =
                    owned.as_ref()
                {
                    self.pending_root_file_actions_file = Some(file.clone());
                }
                self.pending_root_unified_actions_subject = Some(*owned);
                actions
            }
            Some(
                crate::root_unified_result_actions::RootUnifiedResultActionOwner::ExistingScriptActions
                | crate::root_unified_result_actions::RootUnifiedResultActionOwner::None,
            ) => match self.get_focused_script_info() {
                Some(info) if info.is_scriptlet => {
                    crate::actions::get_scriptlet_context_actions_with_custom(
                        &info,
                        self.get_focused_scriptlet_with_actions().as_ref(),
                    )
                }
                Some(info) => crate::actions::get_script_context_actions(&info),
                None => crate::actions::get_global_actions(),
            },
            None => crate::actions::get_global_actions(),
        };
        let mut rows = actions
            .iter()
            .filter(|action| action.is_enabled())
            .map(action_row);
        Event::Actions(
            rows.next()
                .into_iter()
                .chain(std::iter::once(super::chat_entry::about_row()))
                .chain(rows)
                .collect(),
        )
    }

    /// A row that acts on what's staged, as the list draws it.
    pub(super) fn one_window_action_result(row: &Row) -> Option<SearchResult> {
        let id = row
            .key
            .strip_prefix(ACTION_KEY)
            .or((row.key == super::chat_entry::ABOUT_KEY).then_some(row.key.as_str()))?;
        Some(SearchResult::BuiltIn(crate::scripts::BuiltInMatch {
            entry: crate::builtins::BuiltInEntry {
                id: row.key.clone(),
                name: row.title.clone(),
                description: row.sub.clone().unwrap_or_default(),
                keywords: Vec::new(),
                feature: crate::builtins::BuiltInFeature::StagedAction(id.to_string()),
                icon: row.icon.clone(),
                group: crate::builtins::BuiltInGroup::Core,
            },
            score: 0,
            match_evidence: None,
        }))
    }

    /// Runs an action on what was staged. The actions read the list's highlighted row, so the
    /// subject's words and row are the list again while the action runs; the input goes back
    /// to what the reducer shows if the action left the window as it was.
    pub(crate) fn one_window_run_action(&mut self, id: String, cx: &mut Context<Self>) {
        if id == super::chat_entry::ABOUT_KEY {
            self.one_window_route_work(&super::chat_entry::about_row(), cx);
            return;
        }
        if let Some((op, app)) = window_action(&id) {
            self.one_window_host.staged_from = None;
            self.one_window_arrange_window(op, app, cx);
            return;
        }
        let Some(handle) = self.one_window_host.window else {
            tracing::warn!(target: "script_kit::one_window", event = "action_without_window", id = %id);
            return;
        };
        let from = self.one_window_host.staged_from.take();
        let entity = cx.entity();
        cx.defer(move |cx| {
            let _ = cx.update_window(handle, |_, window, cx| {
                entity.update(cx, |app, cx| {
                    let shown = app.filter_text.clone();
                    if let Some(from) = from.as_ref().filter(|f| f.row.is_some()) {
                        app.one_window_show_subject(from, cx);
                    }
                    app.handle_action(id, window, cx);
                    if matches!(app.current_view, AppView::ScriptList) && app.filter_text != shown {
                        app.filter_text = shown;
                        app.flush_pending_main_menu_query(cx);
                    }
                });
            });
        });
    }

    /// A staged window's arrangement. Quitting the app can't be taken back, so it waits in the
    /// line first; everything else happens at once.
    fn one_window_arrange_window(
        &mut self,
        op: crate::intent::WindowOp,
        app: String,
        cx: &mut Context<Self>,
    ) {
        if op.action() != Some(crate::intent::WindowAction::Quit) {
            self.run_window_op(op, app, cx);
            return;
        }
        let title = op.title_for(&app);
        self.one_window_hold(
            "power",
            title,
            Box::new(move |view, cx| view.run_window_op(op, app, cx)),
            cx,
        );
    }

    fn one_window_show_subject(&mut self, from: &StagedFrom, cx: &mut Context<Self>) {
        if self.filter_text != from.text {
            self.filter_text = from.text.clone();
            self.flush_pending_main_menu_query(cx);
        }
        let Some(index) = self
            .main_menu_committed_rows()
            .iter()
            .find(|row| Some(&row.stable_key) == from.row.as_ref())
            .map(|row| row.grouped_index)
        else {
            return;
        };
        self.set_selected_index(index, "one_window_staged_subject", cx);
    }
}
