//! An authored list opens under the existing input. The author may change its declaration;
//! only a person opening it asks for data. Nothing is executed from the personal document.
use super::views::HostedView;
use crate::{
    notes::{self, NoteId, NoteTarget},
    ScriptListApp,
};
use gpui::Context;
use sk_one_window::{
    personal_list::{Item, PersonalList},
    EnterTarget, Event, Key, Row,
};
use sk_personal::{
    views::{ListSpec, NoteDetail, NoteFilter, NoteSort, PaneAction, ViewRecipe},
    Store,
};
use uuid::Uuid;

pub(super) const NAME: &str = "personal-list";
const OPEN: &str = "personal/view/";
const ROW: &str = "personal-item:";
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct List {
    mount: Uuid,
    workspace: String,
    revision: String,
    pub(super) recipe: ViewRecipe,
    pub(super) data: PersonalList,
    pub(super) message: Option<&'static str>,
    opening: Option<Uuid>,
    more: bool,
}
impl List {
    pub(super) fn label(&self) -> String {
        if self.more {
            format!("{} · first {}", self.recipe.title, self.recipe.limit)
        } else {
            self.recipe.title.clone()
        }
    }
    pub(super) fn read_only(&self) -> bool {
        matches!(self.recipe.list, ListSpec::Panes { .. })
    }
    pub(super) fn row_id(&self, item: &str) -> String {
        format!("{ROW}{}:{item}", self.mount)
    }
    pub(super) fn enter(&self) -> EnterTarget {
        match self.data.selected() {
            Some(item) if self.opening.is_none() && !self.read_only() => {
                EnterTarget::new(self.row_id(&item.id), "Open note")
            }
            _ => EnterTarget::new(
                format!("personal-idle:{}", self.mount),
                if self.opening.is_some() {
                    "Opening…"
                } else {
                    "No action"
                },
            ),
        }
    }
    pub(super) fn typed(&mut self, text: String) {
        self.opening = None;
        self.data.type_query(text);
    }
    fn invalidate(&mut self) {
        self.data.replace(Vec::new());
        self.more = false;
        self.message = Some("This view changed. Reopen it from the launcher.");
        self.opening = None;
    }
}

pub(super) struct SourceSnapshot {
    pub(super) items: Vec<Item>,
    pub(super) pair_id: Option<String>,
    pub(super) more: bool,
    pub(super) message: Option<&'static str>,
}
type SourceRows = Result<SourceSnapshot, &'static str>;
fn load_items(
    root: &std::path::Path,
    workspace: &str,
    revision: &str,
    recipe: &ViewRecipe,
    pair_id: Option<&str>,
) -> SourceRows {
    let current = || {
        let stored = Store::open(root)
            .and_then(|s| s.inspect())
            .map_err(|_| "Personal views are unavailable")?;
        if stored.enabled
            && stored.workspace == workspace
            && stored.revision == revision
            && stored.views.contains(recipe)
        {
            Ok(())
        } else {
            Err("This view changed. Reopen it from the launcher.")
        }
    };
    current()?;
    match recipe.list {
        ListSpec::Notes {
            filter,
            sort,
            detail,
            ..
        } => {
            let notes = notes::personal_note_titles(
                filter == NoteFilter::Pinned,
                sort == NoteSort::Title,
                recipe.limit,
            )
            .map_err(|_| "The note index is unavailable")?;
            Ok(SourceSnapshot {
                items: notes
                    .into_iter()
                    .map(|note| Item {
                        id: note.id.to_string(),
                        title: if note.title.trim().is_empty() {
                            "Untitled note".into()
                        } else {
                            note.title
                        },
                        detail: match detail {
                            NoteDetail::None => None,
                            NoteDetail::Pinned => note.pinned.then(|| "Pinned".into()),
                            NoteDetail::LastChanged => {
                                Some(format!("Changed {}", note.updated.format("%b %-d")))
                            }
                        },
                        icon: if note.pinned { "starfilled" } else { "file" },
                    })
                    .collect(),
                pair_id: None,
                more: false,
                message: None,
            })
        }
        ListSpec::Panes {
            filter,
            sort,
            detail,
            enter: PaneAction::None,
        } => {
            let snapshot = super::herdr_panes::read(
                root,
                workspace,
                pair_id,
                filter,
                sort,
                detail,
                recipe.limit,
            )?;
            current()?; // A late reply cannot revive a changed or disabled recipe.
            Ok(snapshot)
        }
        // Keep control-demanding recipes projectable, but never silently reinterpret a metadata grant.
        ListSpec::Panes { .. } => {
            Err("This view asks to control Herdr. Pane sharing is read only.")
        }
    }
}
impl ScriptListApp {
    fn receive_personal_items(&mut self, mount: Uuid, rows: SourceRows, cx: &mut Context<Self>) {
        if !matches!(self.one_window_hosted_view(),Some(HostedView::Personal(list)) if list.mount==mount)
        {
            return;
        }
        let state = self.one_window_host.personal.state.as_ref();
        let Some(HostedView::Personal(list)) = self.one_window_host.view.as_mut() else {
            return;
        };
        let before = (list.data.clone(), list.message, list.more);
        if !state.is_some_and(|s| {
            s.enabled && s.workspace == list.workspace && s.revision == list.revision
        }) {
            list.invalidate();
        } else {
            match rows {
                Ok(source) => {
                    list.data.replace(source.items);
                    list.more = source.more;
                    list.message = source.message;
                }
                Err(msg) => {
                    list.data.replace(Vec::new());
                    list.message = Some(msg);
                    list.more = false;
                }
            }
        }
        if before.0 == list.data && before.1 == list.message && before.2 == list.more {
            return;
        }
        let enter = list.enter();
        self.one_window_handle(Event::ViewEnter(enter), cx);
        cx.notify();
    }
    pub(super) fn personal_view_rows(&self, text: &str) -> Vec<Row> {
        let query = text.trim().to_lowercase();
        let needle = if query == "my views" || query == "view" {
            ""
        } else if let Some(q) = query.strip_prefix("view ") {
            q.trim()
        } else {
            return Vec::new();
        };
        let Some(state) = self
            .one_window_host
            .personal
            .state
            .as_ref()
            .filter(|s| s.enabled)
        else {
            return Vec::new();
        };
        state
            .views
            .iter()
            .filter(|v| v.id.contains(needle) || v.title.to_lowercase().contains(needle))
            .map(|v| {
                Row::new(
                    format!("{OPEN}{}/{}", state.revision, v.id),
                    v.title.clone(),
                )
                .with_authored_label()
                .with_head("My views")
                .with_icon("list")
                .with_enter_label("Open view")
            })
            .collect()
    }
    pub(super) fn open_personal_view(
        &mut self,
        row: &Row,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.personal_view_rows(&self.filter_text)
                .iter()
                .any(|r| r.key == row.key),
            "personal_view_no_longer_visible"
        );
        let state = self
            .one_window_host
            .personal
            .state
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("personal_view_unavailable"))?;
        let recipe = state
            .views
            .iter()
            .find(|v| row.key == format!("{OPEN}{}/{}", state.revision, v.id))
            .ok_or_else(|| anyhow::anyhow!("personal_view_changed"))?
            .clone();
        let workspace = state.workspace.clone();
        let revision = state.revision.clone();
        let root = self.personal_root()?;
        let mount = Uuid::new_v4();
        self.one_window_open_hosted(
            HostedView::Personal(List {
                mount,
                workspace: workspace.clone(),
                revision: revision.clone(),
                recipe: recipe.clone(),
                data: PersonalList::default(),
                message: Some("Loading…"),
                opening: None,
                more: false,
            }),
            Some(String::new()),
            cx,
        );
        cx.spawn(async move |this, cx| {
            let mut expected_pair = None;
            loop {
                let (root, ws, rev, recipe, pair) = (
                    root.clone(), workspace.clone(), revision.clone(), recipe.clone(), expected_pair.take(),
                );
                let rows = cx.background_executor().spawn(async move {
                    load_items(&root, &ws, &rev, &recipe, pair.as_deref())
                }).await;
                // Only a fresh ready pane list gets another read, always for the same pairing. A
                // refusal, uncertain reply, closed view or new mount is terminal for this loop.
                expected_pair = rows.as_ref().ok().and_then(|s| s.pair_id.clone());
                if this.update(cx, |app, cx| app.receive_personal_items(mount, rows, cx)).is_err()
                    || expected_pair.is_none()
                {
                    break;
                }
                cx.background_executor().timer(std::time::Duration::from_secs(4)).await;
                if !this.update(cx, |app, ctx| {
                    if !crate::is_main_window_visible() {
                        app.receive_personal_items(mount, Err("Pane sharing paused. Reopen the view for a fresh list."), ctx);
                        return false;
                    }
                    matches!(app.one_window_hosted_view(),Some(HostedView::Personal(l)) if l.mount == mount)
                    && app.one_window_host.personal.state.as_ref().is_some_and(|s| s.enabled && s.workspace == workspace && s.revision == revision)
                }).unwrap_or(false) { break; }
            }
        })
        .detach();
        Ok(())
    }
    pub(super) fn refresh_open_personal_view(&mut self, cx: &mut Context<Self>) {
        let current = self.one_window_host.personal.state.as_ref();
        let Some(HostedView::Personal(list)) = self.one_window_host.view.as_mut() else {
            return;
        };
        if !current.is_some_and(|s| {
            s.enabled && s.workspace == list.workspace && s.revision == list.revision
        }) {
            list.invalidate();
            let enter = list.enter();
            if self
                .one_window_hosted_view()
                .is_some_and(|v| v.name() == NAME)
            {
                self.one_window_handle(Event::ViewEnter(enter), cx);
            }
        }
    }
    pub(crate) fn personal_submit_block_reason(&self) -> Option<&'static str> {
        match self.one_window_hosted_view() {
            Some(HostedView::Personal(list)) if list.read_only() => {
                Some("Pane sharing is read only")
            }
            _ => None,
        }
    }
    pub(crate) fn pick_personal_view_item(
        &mut self,
        id: &str,
        submit: bool,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            matches!(self.one_window_hosted_view(), Some(HostedView::Personal(_))),
            "personal_view_not_current"
        );
        let Some(HostedView::Personal(list)) = self.one_window_host.view.as_mut() else {
            anyhow::bail!("personal_view_not_current")
        };
        anyhow::ensure!(!submit || !list.read_only(), "personal_row_read_only");
        let prefix = format!("{ROW}{}:", list.mount);
        anyhow::ensure!(
            list.opening.is_none()
                && id
                    .strip_prefix(&prefix)
                    .is_some_and(|key| list.data.pick(key)),
            "personal_row_not_current"
        );
        let enter = list.enter();
        self.one_window_handle(Event::ViewEnter(enter), cx);
        cx.notify();
        if submit {
            anyhow::ensure!(
                !self.one_window_enter_arming(),
                "personal_row_selected_wait_for_return"
            );
            self.one_window_key(Key::Enter, cx);
            anyhow::ensure!(
                matches!(self.one_window_hosted_view(),Some(HostedView::Personal(list)) if list.opening.is_some()),
                "personal_open_not_scheduled"
            );
        }
        Ok(())
    }
    pub(crate) fn return_to_personal_list(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.show_script_list_with_main_filter_focus();
        if let Some(HostedView::Personal(list)) = self.one_window_hosted_view() {
            self.one_window_handle(Event::ViewEnter(list.enter()), cx);
        }
        self.focus_main_filter(window, cx);
        self.sync_main_footer_popup(window, cx);
        cx.notify();
    }
    pub(super) fn personal_view_key(&mut self, key: Key, cx: &mut Context<Self>) -> Vec<Event> {
        let Some(HostedView::Personal(list)) = self.one_window_host.view.as_mut() else {
            return Vec::new();
        };
        match key {
            Key::Enter if list.opening.is_some() => Vec::new(),
            Key::Up | Key::Down => {
                list.opening = None;
                list.data.arrow(key == Key::Down);
                cx.notify();
                vec![Event::ViewEnter(list.enter())]
            }
            Key::Enter => {
                let Some(item) = list.data.selected() else {
                    return Vec::new();
                };
                if !matches!(list.recipe.list, ListSpec::Notes { .. }) {
                    return Vec::new();
                }
                let Some(id) = NoteId::parse(&item.id) else {
                    return Vec::new();
                };
                let shown = item.clone();
                let recipe = list.recipe.clone();
                let (mount, workspace, revision) =
                    (list.mount, list.workspace.clone(), list.revision.clone());
                let Some(window) =
                    crate::get_main_window_handle().or_else(|| match cx.windows().as_slice() {
                        [only] => Some(*only),
                        _ => None,
                    })
                else {
                    return Vec::new();
                };
                let opening = Uuid::new_v4();
                list.opening = Some(opening);
                let enter = list.enter();
                let root = crate::setup::get_kit_path();
                cx.spawn(async move |this, cx| {
                    // Re-run the same bounded, title-only query. An existing note can have
                    // been unpinned, renamed or pushed out of the recipe's result limit.
                    let refreshed = cx.background_executor().spawn(async move {
                        load_items(&root, &workspace, &revision, &recipe, None)
                    }).await;
                    // Re-check the original window, view, user choice and operation on the event
                    // that actually opens the editor. Never hand a stale target to a later defer.
                    let _ = window.update(cx, |_, window, cx| this.update(cx, |app, cx| {
                        let Some(HostedView::Personal(list)) = app.one_window_hosted_view() else { return; };
                        if !matches!(app.current_view, crate::AppView::ScriptList) || list.mount != mount
                            || list.opening != Some(opening) || list.data.selected() != Some(&shown) { return; }
                        let Some(HostedView::Personal(list)) = app.one_window_host.view.as_mut() else { return; };
                        list.opening = None;
                        let refusal = match refreshed {
                            Ok(source) => (!list.data.confirm_refreshed(&shown, source.items))
                                .then_some("That note changed or no longer matches. Choose a result again."),
                            Err(reason) => {
                                list.data.replace(Vec::new());
                                list.message = Some(reason);
                                Some(reason)
                            }
                        };
                        if let Some(reason) = refusal {
                            let enter = list.enter();
                            app.one_window_handle(Event::ViewEnter(enter), cx);
                            app.one_window_notice(crate::line_notice::INFO, reason, cx);
                        } else {
                            let opened = app.open_note_in_editor(NoteTarget::Note(id), crate::DayPageBack::PersonalView, window, cx);
                            if opened.is_err() || !matches!(&app.current_view, crate::AppView::DayPage { entity } if entity.read(cx).current_note_id()==Some(id)) {
                                app.return_to_personal_list(window, cx);
                                app.one_window_notice(crate::line_notice::FAILED, "Couldn't open that note", cx);
                            }
                        }
                        cx.notify();
                    }));
                }).detach();
                vec![Event::ViewEnter(enter)]
            }
            _ => Vec::new(),
        }
    }
}
