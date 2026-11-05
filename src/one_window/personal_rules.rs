//! The disk service never runs while the input is painted or parsed. Rules are adopted by the
//! reducer as one snapshot. Each submitted row pins the workspace, revision and typed operation.
use crate::ScriptListApp;
use gpui::Context;
use sk_one_window::{personal::LauncherRules, window::subject_tabs::TabsEvent, Event, Row};
use sk_personal::{
    speech::{self, Reading},
    Author, Change, Error, Request, State, Store,
};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Default)]
pub(super) struct PersonalHost {
    pub state: Option<State>,
    error: Option<Error>,
    generation: u64,
    busy: bool,
    refresh_after: bool,
}
const PREFIX: &str = "personal/launcher/";
pub(super) fn is_rule_row(row: &Row) -> bool {
    row.key.starts_with(PREFIX)
}
fn explanation(e: Error) -> &'static str {
    match e {
        Error::StaleRevision | Error::RequestConflict => {
            "Launcher settings changed; review the current choice"
        }
        Error::NothingToUndo => "There is no matching launcher change to undo",
        Error::WrongWorkspace => "That Kit workspace is no longer available",
        Error::SafeMode => "Personal launcher rules are off in safe mode",
        Error::WriterBusy => "Another launcher change is being saved",
        Error::HistoryLimit => "That request is outside the available history",
        Error::InvalidDocument | Error::UnsafePath => {
            "Personal launcher rules could not be verified"
        }
        Error::StorageUnavailable | Error::Unsupported => {
            "Personal launcher settings are unavailable"
        }
        Error::InvalidRequest => "That launcher change is not supported",
        Error::InvalidView => "That view definition is not supported",
        Error::ViewLimit => "Remove a personal view before adding another",
        Error::ViewNotFound => "That personal view no longer exists",
    }
}
impl ScriptListApp {
    pub(super) fn personal_root(&self) -> Result<PathBuf, Error> {
        if sk_personal::api::safe_mode() {
            return Err(Error::SafeMode);
        }
        let path = crate::setup::get_kit_path();
        if let Some(policy) = crate::runtime_policy::owned_evaluation() {
            policy
                .require_owned_path(&path)
                .map_err(|_| Error::WrongWorkspace)?;
        } else if !self.main_services.is_production() {
            return Err(Error::StorageUnavailable);
        }
        Ok(path)
    }
    fn adopt_personal(&mut self, result: Result<State, Error>, cx: &mut Context<Self>) {
        let (state, error, rules) = match result {
            Ok(s) => {
                let rules = s.effective;
                (Some(s), None, rules)
            }
            Err(e) => (None, Some(e), LauncherRules::default()),
        };
        let host = &mut self.one_window_host.personal;
        if host.state == state && host.error == error {
            return;
        }
        host.state = state;
        host.error = error;
        self.refresh_open_personal_view(cx);
        if self.one_window.personal_rules() != rules {
            self.one_window_handle(Event::Subjects(TabsEvent::Personal(rules)), cx);
        }
        self.one_window_republish("personal_rules", cx);
        cx.notify();
    }
    pub(crate) fn reload_personal_rules(&mut self, cx: &mut Context<Self>) {
        let root = match self.personal_root() {
            Ok(p) => p,
            Err(e) => {
                self.adopt_personal(Err(e), cx);
                return;
            }
        };
        let h = &mut self.one_window_host.personal;
        if h.busy {
            h.refresh_after = true;
            return;
        }
        h.generation = h.generation.wrapping_add(1);
        let generation = h.generation;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { Store::open(&root)?.inspect() })
                .await;
            let _ = this.update(cx, |app, cx| {
                if app.one_window_host.personal.generation == generation {
                    app.adopt_personal(result, cx);
                }
            });
        })
        .detach();
    }
    pub(crate) fn personal_adoption(&self) -> sk_personal::api::Adoption {
        let h = &self.one_window_host.personal;
        sk_personal::api::Adoption {
            workspace: h.state.as_ref().map(|s| s.workspace.clone()),
            applied_revision: h.state.as_ref().map(|s| s.revision.clone()),
            effective: self.one_window.personal_rules(),
            active_view_ids: h
                .state
                .as_ref()
                .filter(|s| s.enabled)
                .into_iter()
                .flat_map(|s| s.views.iter().map(|v| v.id.clone()))
                .collect(),
            error: h.error,
            pending_write: h.busy,
        }
    }
    pub(super) fn personal_observation(&self) -> serde_json::Value {
        let mut adoption = serde_json::json!(self.personal_adoption());
        adoption["scope"] = "runningWindow".into();
        adoption
    }
    pub(super) fn personal_rule_row(&self, text: &str) -> Option<Row> {
        let read = speech::read(text)?;
        let state = self.one_window_host.personal.state.as_ref();
        let rev = state.map_or("unavailable", |s| s.revision.as_str());
        let label = match &read {
            Reading::Why(_) => "Explain my launcher setting",
            Reading::Change(c) => speech::label(c),
            Reading::UndoLast => "Undo my last launcher change",
        };
        let sub = match state {
            Some(s) if !s.enabled => "Personal rules are disabled",
            Some(_) => "Only this Kit workspace",
            None => self
                .one_window_host
                .personal
                .error
                .map_or("Reading launcher settings", explanation),
        };
        Some(
            Row::new(format!("{PREFIX}{rev}/{read:?}"), label)
                .with_head("Customize Kit")
                .with_icon("settings")
                .with_sub(sub)
                .with_enter_label(match read {
                    Reading::Why(_) => "Explain",
                    Reading::UndoLast => "Undo",
                    _ => "Apply",
                }),
        )
    }
    pub(super) fn run_personal_rule(
        &mut self,
        row: &Row,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let text = self.filter_text.clone();
        let read =
            speech::read(&text).ok_or_else(|| anyhow::anyhow!("The launcher request changed"))?;
        anyhow::ensure!(
            self.personal_rule_row(&text)
                .is_some_and(|shown| shown.key == row.key),
            "The launcher setting changed"
        );
        let state = self
            .one_window_host
            .personal
            .state
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Personal launcher settings are unavailable"))?;
        let change = match read {
            Reading::Change(c) => c,
            Reading::UndoLast => Change::Undo {
                of: state
                    .undo
                    .ok_or_else(|| anyhow::anyhow!("No launcher change to undo"))?,
            },
            Reading::Why(target) => {
                let root = self
                    .personal_root()
                    .map_err(|e| anyhow::anyhow!(explanation(e)))?;
                cx.spawn(async move |this, cx| {
                    let result = cx
                        .background_executor()
                        .spawn(async move {
                            let value = Store::open(&root)?.why(target)?;
                            if value["revision"] != state.revision {
                                return Err(Error::StaleRevision);
                            }
                            Ok(value)
                        })
                        .await;
                    let _ = this.update(cx, |app, cx| {
                        let (icon, words) = match result {
                            Ok(v) => {
                                let why = if v["enabled"] == false {
                                    "Personal rules are disabled"
                                } else {
                                    match v["origin"].as_str() {
                                        Some("launcher") => "You set this in the launcher",
                                        Some("localAgent") => "Your local settings tool set this",
                                        Some("configuredAgent") => "Your configured agent set this",
                                        Some("connectedAgent") => "A connected agent set this",
                                        Some("personalFile") => {
                                            "This comes from your personal launcher file"
                                        }
                                        Some("stock") => "This is Kit’s default",
                                        _ => "The source of this preference is unavailable",
                                    }
                                };
                                (crate::line_notice::INFO, why)
                            }
                            Err(e) => (crate::line_notice::FAILED, explanation(e)),
                        };
                        app.one_window_notice(icon, words, cx);
                    });
                })
                .detach();
                return Ok(());
            }
        };
        self.submit_personal(
            Request {
                request_id: Uuid::new_v4(),
                workspace: state.workspace,
                expected_revision: state.revision,
                change,
            },
            Some(text),
            cx,
        );
        Ok(())
    }
    fn submit_personal(
        &mut self,
        request: Request,
        submitted: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let root = match self.personal_root() {
            Ok(p) => p,
            Err(e) => {
                self.one_window_notice(crate::line_notice::FAILED, explanation(e), cx);
                return;
            }
        };
        let h = &mut self.one_window_host.personal;
        if h.busy {
            return;
        }
        h.busy = true;
        h.generation = h.generation.wrapping_add(1);
        let change = request.change.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { Store::open(&root)?.apply(request, Author::Launcher) })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.one_window_host.personal.busy = false;
                let refresh = std::mem::take(&mut app.one_window_host.personal.refresh_after);
                match result {
                    Err(e) => {
                        app.one_window_notice(crate::line_notice::FAILED, explanation(e), cx);
                        app.reload_personal_rules(cx);
                    }
                    Ok(r) => {
                        let undo = Request {
                            workspace: r.state.workspace.clone(),
                            expected_revision: r.state.revision.clone(),
                            request_id: Uuid::new_v4(),
                            change: Change::Undo { of: r.request_id },
                        };
                        app.adopt_personal(Ok(r.state), cx);
                        if submitted.as_deref() == Some(app.filter_text.as_str())
                            && app.one_window_owns_list()
                        {
                            app.one_window_set_input(String::new(), cx);
                        }
                        if matches!(change, Change::Undo { .. }) {
                            app.one_window_notice(
                                crate::line_notice::INFO,
                                "Launcher change undone",
                                cx,
                            );
                        } else {
                            let label = match change {
                                Change::Edit { .. } => "Launcher preference saved",
                                Change::PutView { .. } => "Personal view saved",
                                Change::RemoveView { .. } => "Personal view removed",
                                Change::Enable => "Personal launcher rules enabled",
                                Change::Disable => "Using the stock launcher",
                                Change::Undo { .. } => unreachable!(),
                            };
                            app.one_window_done_undoable(
                                crate::line_notice::INFO,
                                label,
                                Box::new(move |app, cx| app.submit_personal(undo, None, cx)),
                                cx,
                            );
                        }
                        if refresh {
                            app.reload_personal_rules(cx);
                        }
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}
