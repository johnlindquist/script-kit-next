//! Jev reads the typed words against the screen and the launcher's commands. The one-window
//! reducer decides when it is asked and which of its readings the list shows; this asks, and
//! runs a reading's plan when one of Jev's rows wins Enter.

use super::*;

use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use super::intent_live as live;
use crate::intent::{
    self as intent_engine, CommandKind, EngineOptions, JevClient, JevConfig, JevError, LastAction,
    LastStep, Oracle, OracleSlot, Plan, ScriptOptions, Step,
};

/// Window and menu steps wait for apps opened earlier in the same plan.
const WINDOW_STEP_ATTEMPTS: u32 = 8;
const WINDOW_STEP_RETRY: Duration = Duration::from_millis(250);

/// `SCRIPT_KIT_JEV_INTENT=0` keeps Jev from being asked.
const INTENT_ENV_TOGGLE: &str = "SCRIPT_KIT_JEV_INTENT";

impl ScriptListApp {
    pub(crate) fn settle_intent_oracle_slot(&mut self) {
        if !matches!(self.intent_launcher.oracle_slot(), OracleSlot::Unloaded) {
            return;
        }
        if std::env::var(INTENT_ENV_TOGGLE).is_ok_and(|value| value.trim() == "0") {
            self.intent_launcher
                .set_oracle_slot(OracleSlot::Off("disabled"));
        } else if !self.main_services.is_production() {
            self.intent_launcher
                .set_oracle_slot(OracleSlot::Off("not production"));
        }
    }

    fn intent_catalog_revision(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for script in &self.scripts {
            (Arc::as_ptr(script) as usize).hash(&mut hasher);
        }
        for scriptlet in &self.scriptlets {
            (Arc::as_ptr(scriptlet) as usize).hash(&mut hasher);
        }
        self.builtin_entries.len().hash(&mut hasher);
        self.apps.len().hash(&mut hasher);
        hasher.finish()
    }

    fn intent_catalog(&mut self) -> Arc<intent_engine::Catalog> {
        let revision = self.intent_catalog_revision();
        if let Some(catalog) = self.intent_launcher.catalog_for(revision) {
            return catalog;
        }
        let catalog = Arc::new(live::catalog_from_launcher(
            &self.scripts,
            &self.scriptlets,
            &self.builtin_entries,
            &self.apps,
        ));
        self.intent_launcher
            .store_catalog(revision, catalog.clone());
        catalog
    }

    /// Fingerprints the screen Jev reads against, so a remembered reading is for this screen.
    pub(crate) fn intent_screen_key(&self) -> u64 {
        if self.main_services.is_production() {
            live::screen_key()
        } else {
            0
        }
    }

    /// Asks Jev to read `text`; the answer goes back to the one-window host as request `id`.
    pub(crate) fn ask_jev(&mut self, id: u64, text: String, screen: u64, cx: &mut Context<Self>) {
        self.settle_intent_oracle_slot();
        let catalog = self.intent_catalog();
        let slot = self.intent_launcher.oracle_slot().clone();
        let last_action = self.intent_launcher.last_action().cloned();
        let live_screen = self.main_services.is_production();
        let request = text.clone();
        cx.spawn(async move |this, cx| {
            let (loaded, result) = cx
                .background_executor()
                .spawn(async move {
                    let (oracle, loaded): (Arc<dyn Oracle>, Option<Arc<dyn Oracle>>) = match slot {
                        OracleSlot::Ready(oracle) => (oracle, None),
                        OracleSlot::Off(reason) => return (None, Err(JevError::Refused(reason))),
                        OracleSlot::Unloaded => match JevConfig::load() {
                            Ok(config) => {
                                let oracle: Arc<dyn Oracle> = Arc::new(JevClient::new(config));
                                (oracle.clone(), Some(oracle))
                            }
                            Err(error) => return (None, Err(error)),
                        },
                    };
                    let screen = if live_screen {
                        live::live_screen(last_action)
                    } else {
                        intent_engine::ScreenContext {
                            id: "fixture".into(),
                            last_action,
                            subject_read_at_run: true,
                            ..intent_engine::ScreenContext::default()
                        }
                    };
                    let plan = intent_engine::understand(
                        &request,
                        &screen,
                        &catalog,
                        oracle.as_ref(),
                        &EngineOptions::default(),
                    );
                    (loaded, plan)
                })
                .await;
            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    if let Some(oracle) = loaded {
                        if matches!(app.intent_launcher.oracle_slot(), OracleSlot::Unloaded) {
                            app.intent_launcher
                                .set_oracle_slot(OracleSlot::Ready(oracle));
                        }
                    }
                    app.one_window_jev_answered(id, text, screen, result, cx);
                })
            });
        })
        .detach();
    }

    /// Runs one of Jev's rows for `text`, or one of the rows that stand in for Jev when it's
    /// down. `None` when the key is neither.
    pub(crate) fn run_jev_row(
        &mut self,
        key: &str,
        text: &str,
        cx: &mut Context<Self>,
    ) -> Option<Result<(), &'static str>> {
        if let Some(plan) = self.intent_launcher.plan(text, key) {
            if !plan.ready {
                self.explain_unready_plan(&plan, cx);
                return Some(Err("intent_plan_not_ready"));
            }
            return Some(self.execute_intent_plan(&plan, cx));
        }
        match key {
            "down:note" => Some(append_capture_to_day("note", text, None).map(|()| {
                self.one_window_ran_here(cx);
            })),
            _ => None,
        }
    }

    /// Why one of Jev's readings can't run yet.
    fn explain_unready_plan(&mut self, plan: &Plan, cx: &mut Context<Self>) {
        let message = match plan.missing.as_deref() {
            Some(missing) => format!("Needs {missing}"),
            None => "Needs more to go on".to_string(),
        };
        self.one_window_notice(crate::line_notice::INFO, message, cx);
    }

    /// ⌘↵ in the main window: Jev's first row for the words, or the model's chat under the input
    /// when nothing is typed. Agent Chat keeps ⌘↵ when the list belongs to launcher syntax.
    pub(crate) fn route_main_window_cmd_enter(&mut self, cx: &mut Context<Self>) -> bool {
        if matches!(self.current_view, AppView::ScriptList) {
            self.flush_pending_main_menu_query(cx);
        }
        (self.one_window_owns_list()
            && self
                .one_window_key(sk_one_window::Key::CmdEnter, cx)
                .taken())
            || self.try_route_global_cmd_enter_to_agent_chat_context_capture(cx)
    }

    /// Whether ⌘↵ runs the words rather than asking the model, for the footer.
    pub(crate) fn cmd_enter_runs_intent(&self) -> bool {
        self.one_window_owns_list() && !self.filter_text.trim().is_empty()
    }

    /// Runs a plan's steps in order. Launcher lines (captures, searches) run through the
    /// launcher itself so they behave exactly as if typed.
    pub(crate) fn execute_intent_plan(
        &mut self,
        plan: &Arc<Plan>,
        cx: &mut Context<Self>,
    ) -> Result<(), &'static str> {
        if !plan.ready {
            return Err("intent_plan_not_ready");
        }
        tracing::info!(
            target: "script_kit::intent",
            event = "intent_plan_run",
            pathway = plan.pathway.key(),
            steps = plan.steps.len(),
            clauses = plan.clauses.len(),
        );
        if let Some(action) = last_action_for(plan) {
            self.intent_launcher.record(action);
        }
        if plan.pathway == intent_engine::Pathway::Automate {
            return self.save_intent_plan_as_script(plan, cx).map(|_| ());
        }
        if let [only] = plan.steps.as_slice() {
            match only {
                Step::Capture { .. } | Step::Recall { .. } => {
                    return self.pivot_intent_to_typed_line(plan, cx);
                }
                Step::Remember { text, subject } => {
                    return self.run_intent_remember(text, subject.as_deref(), cx);
                }
                _ => {}
            }
        }

        let mut later = Vec::new();
        for step in &plan.steps {
            match step {
                Step::Command { command } => self.run_intent_command(command, cx)?,
                Step::Transform {
                    tool: Some(tool), ..
                } => self.run_intent_command(tool, cx)?,
                Step::Capture {
                    target, body, when, ..
                } => append_capture_to_day(target, body, when.as_deref())?,
                Step::Remember { text, .. } => append_memory_to_day(text)?,
                Step::Menu { .. } | Step::Window { .. } => later.push(step.clone()),
                Step::Recall { .. }
                | Step::Ask { .. }
                | Step::Delegate { .. }
                | Step::Transform { .. } => return Err("intent_step_not_combinable"),
            }
        }
        if later.is_empty() {
            return Ok(());
        }
        if !plan
            .steps
            .iter()
            .any(|step| matches!(step, Step::Command { .. }))
        {
            self.close_and_reset_window(cx);
        }
        cx.background_executor()
            .spawn(async move {
                for step in later {
                    if let Err(error) = run_desktop_step_with_retries(&step) {
                        tracing::warn!(
                            target: "script_kit::intent",
                            event = "intent_step_failed",
                            error = %error,
                        );
                        break;
                    }
                }
            })
            .detach();
        Ok(())
    }

    fn run_intent_command(
        &mut self,
        command: &intent_engine::Command,
        cx: &mut Context<Self>,
    ) -> Result<(), &'static str> {
        match command.kind {
            CommandKind::App => {
                let app = self
                    .apps
                    .iter()
                    .find(|app| live::app_command_id(app) == command.id)
                    .cloned()
                    .ok_or("intent_app_missing")?;
                if self.execute_app(&app, cx) {
                    self.one_window_app_launched(&app.name, cx);
                }
            }
            CommandKind::Builtin => {
                let entry = self
                    .builtin_entries
                    .iter()
                    .find(|entry| live::builtin_command_id(entry) == command.id)
                    .cloned()
                    .ok_or("intent_builtin_missing")?;
                let _outcome = self.execute_builtin(&entry, cx);
            }
            CommandKind::Script => {
                let script = self
                    .scripts
                    .iter()
                    .find(|script| live::script_command_id(script) == command.id)
                    .cloned()
                    .ok_or("intent_script_missing")?;
                self.execute_interactive(&script, cx);
            }
            CommandKind::Scriptlet => {
                let scriptlet = self
                    .scriptlets
                    .iter()
                    .find(|scriptlet| live::scriptlet_command_id(scriptlet) == command.id)
                    .cloned()
                    .ok_or("intent_scriptlet_missing")?;
                self.execute_scriptlet(&scriptlet, cx);
            }
        }
        Ok(())
    }

    /// Replace the input with the launcher line the plan compiles to and run it.
    fn pivot_intent_to_typed_line(
        &mut self,
        plan: &Plan,
        cx: &mut Context<Self>,
    ) -> Result<(), &'static str> {
        let typed = plan
            .typed
            .clone()
            .filter(|_| !plan.typed_is_proposed)
            .ok_or("intent_plan_has_no_launcher_line")?;
        self.filter_text = typed;
        self.pending_filter_sync = true;
        self.flush_pending_main_menu_query(cx);
        cx.notify();
        if matches!(plan.pathway, intent_engine::Pathway::Capture) {
            let _dispatch = self.execute_selected(cx);
        }
        Ok(())
    }

    fn run_intent_remember(
        &mut self,
        text: &str,
        subject: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Result<(), &'static str> {
        let text = match subject {
            Some("clipboard") => cx
                .read_from_clipboard()
                .and_then(|item| item.text())
                .filter(|clip| !clip.trim().is_empty())
                .ok_or("intent_clipboard_empty")?,
            _ => text.to_string(),
        };
        append_memory_to_day(&text)?;
        self.one_window_notice(crate::line_notice::INFO, "Remembered".to_string(), cx);
        self.close_and_reset_window(cx);
        Ok(())
    }

    /// Writes the plan as a script in the main plugin, where the launcher picks it up.
    pub(crate) fn save_intent_plan_as_script(
        &mut self,
        plan: &Plan,
        cx: &mut Context<Self>,
    ) -> Result<std::path::PathBuf, &'static str> {
        let draft = intent_engine::plan_to_script(
            plan,
            &ScriptOptions {
                shortcut: plan.slots.shortcut.clone(),
                ..ScriptOptions::default()
            },
        );
        let dir = crate::setup::get_kit_path()
            .join("plugins")
            .join("main")
            .join("scripts");
        std::fs::create_dir_all(&dir).map_err(|_| "intent_scripts_dir_unavailable")?;
        let stem = draft.filename.trim_end_matches(".ts").to_string();
        let path = (1..100)
            .map(|n| {
                if n == 1 {
                    dir.join(&draft.filename)
                } else {
                    dir.join(format!("{stem}-{n}.ts"))
                }
            })
            .find(|path| !path.exists())
            .ok_or("intent_script_name_taken")?;
        std::fs::write(&path, &draft.source).map_err(|_| "intent_script_write_failed")?;
        tracing::info!(
            target: "script_kit::intent",
            event = "intent_script_saved",
            path = %path.display(),
        );
        self.one_window_notice(
            crate::line_notice::INFO,
            format!("Saved script: {}", draft.name),
            cx,
        );
        self.close_and_reset_window(cx);
        Ok(path)
    }

    /// Save the selected intent row as a script. Returns false when the selection
    /// isn't an intent row.
    pub(crate) fn save_selected_intent_as_script(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(scripts::SearchResult::Intent(intent_match)) =
            self.selected_main_list_search_result_owned()
        else {
            return false;
        };
        let Some(plan) = intent_match.plan else {
            self.one_window_notice(
                crate::line_notice::FAILED,
                "Nothing to save until Jev reads it".to_string(),
                cx,
            );
            return true;
        };
        if let Err(reason) = self.save_intent_plan_as_script(&plan, cx) {
            tracing::warn!(target: "script_kit::intent", reason, "intent script not saved");
            self.one_window_notice(
                crate::line_notice::FAILED,
                "Couldn't save the script".to_string(),
                cx,
            );
        }
        true
    }

    /// Commands run from ordinary rows are also something "save that as a script" can mean.
    pub(crate) fn record_intent_last_action_for_result(&mut self, result: &scripts::SearchResult) {
        let Some(id) = intent_command_id_for_result(result) else {
            return;
        };
        self.intent_launcher.record(LastAction {
            summary: format!("Ran {}", result.name()),
            steps: vec![LastStep::Command { id }],
        });
    }
}

/// The catalog id of the command a launcher row runs.
pub(crate) fn intent_command_id_for_result(result: &scripts::SearchResult) -> Option<String> {
    match result {
        scripts::SearchResult::Script(m) => Some(live::script_command_id(&m.script)),
        scripts::SearchResult::Scriptlet(m) => Some(live::scriptlet_command_id(&m.scriptlet)),
        scripts::SearchResult::BuiltIn(m) => Some(live::builtin_command_id(&m.entry)),
        scripts::SearchResult::App(m) => Some(live::app_command_id(&m.app)),
        _ => None,
    }
}

fn last_action_for(plan: &Plan) -> Option<LastAction> {
    let steps: Vec<LastStep> = plan
        .steps
        .iter()
        .filter_map(|step| match step {
            Step::Command { command } => Some(LastStep::Command {
                id: command.id.clone(),
            }),
            Step::Window { op, app } => Some(LastStep::Window {
                op_label: op.label.clone(),
                app: app.clone(),
            }),
            Step::Menu {
                app,
                bundle_id,
                path,
                ..
            } => Some(LastStep::Menu {
                app: app.clone(),
                bundle_id: bundle_id.clone(),
                path: path.clone(),
            }),
            _ => None,
        })
        .collect();
    (!steps.is_empty()).then(|| LastAction {
        summary: plan.title.clone(),
        steps,
    })
}

fn append_memory_to_day(text: &str) -> Result<(), &'static str> {
    crate::brain::substrate::BrainSubstrate::default_kit()
        .append_to_day(
            chrono::Utc::now(),
            crate::brain::substrate::DayEntry::Capture {
                text: text.to_string(),
            },
        )
        .map_err(|_| "intent_brain_write_failed")
}

pub(crate) fn append_capture_to_day(
    target: &str,
    body: &str,
    when: Option<&str>,
) -> Result<(), &'static str> {
    let entry = match target {
        "todo" | "reminder" => crate::brain::substrate::DayEntry::Task {
            body: body.to_string(),
            tags: Vec::new(),
            due: when.map(str::to_string),
        },
        _ => crate::brain::substrate::DayEntry::Capture {
            text: body.to_string(),
        },
    };
    crate::brain::substrate::BrainSubstrate::default_kit()
        .append_to_day(chrono::Utc::now(), entry)
        .map_err(|_| "intent_brain_write_failed")
}

fn run_desktop_step_with_retries(step: &Step) -> anyhow::Result<()> {
    let mut last_error = None;
    for _ in 0..WINDOW_STEP_ATTEMPTS {
        let attempt = match step {
            Step::Window { op, app } => live::run_window_step(op, app),
            #[cfg(target_os = "macos")]
            Step::Menu {
                bundle_id, path, ..
            } => script_kit_gpui::menu_executor::execute_menu_action(bundle_id, path),
            _ => return Ok(()),
        };
        match attempt {
            Ok(()) => return Ok(()),
            Err(error) => last_error = Some(error),
        }
        std::thread::sleep(WINDOW_STEP_RETRY);
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("step did not run")))
}
