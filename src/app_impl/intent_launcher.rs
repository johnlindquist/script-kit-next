//! Jev reads the typed words against the screen and the launcher's commands. The one-window
//! reducer decides when it is asked and which of its readings the list shows; this asks, and
//! runs a reading's plan when one of Jev's rows wins Enter.

use super::*;

use std::hash::{Hash, Hasher};
use std::sync::Arc;
use crate::intent::window_target::{WindowScope, WindowTarget};

use super::intent_live as live;
use crate::intent::{
    self as intent_engine, CommandKind, EngineOptions, JevClient, JevConfig, JevError, LastAction,
    LastStep, Oracle, OracleSlot, Plan, ScriptOptions, Step, WindowAction,
};

/// `SCRIPT_KIT_JEV_INTENT=0` keeps Jev from being asked.
const INTENT_ENV_TOGGLE: &str = "SCRIPT_KIT_JEV_INTENT";

/// Under owned evaluation the app never reaches Jev; it reads against what Jev was recorded
/// answering instead: the same screens, catalog and answers.
fn intent_fixture_world() -> Option<Arc<intent_engine::FixtureWorld>> {
    crate::runtime_policy::is_owned_evaluation()
        .then(intent_engine::FixtureWorld::get)
        .flatten()
}

impl ScriptListApp {
    pub(crate) fn settle_intent_oracle_slot(&mut self) {
        let setting = std::env::var(INTENT_ENV_TOGGLE).ok();
        let local_only = self.main_services.is_production() && !self.config.jev_intent_enabled && setting.as_deref() != Some("1");
        if setting.as_deref() == Some("0") || local_only {
            self.intent_launcher
                .set_oracle_slot(OracleSlot::Off("disabled"));
            return;
        }
        if matches!(self.intent_launcher.oracle_slot(), OracleSlot::Off("disabled")) {
            self.intent_launcher.set_oracle_slot(OracleSlot::Unloaded);
        }
        if !matches!(self.intent_launcher.oracle_slot(), OracleSlot::Unloaded) { return; }
        if !self.main_services.is_production() {
            match intent_fixture_world() {
                Some(world) => self
                    .intent_launcher
                    .set_oracle_slot(OracleSlot::Ready(world)),
                None => self
                    .intent_launcher
                    .set_oracle_slot(OracleSlot::Off("not production")),
            }
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
        for entry in &self.builtin_entries {
            entry.id.hash(&mut hasher);
            entry.name.hash(&mut hasher);
            entry.description.hash(&mut hasher);
        }
        for app in &self.apps {
            app.path.hash(&mut hasher);
            app.name.hash(&mut hasher);
        }
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

    /// A staged window's chip names its app; a staged non-app must never silently target
    /// whichever browser was last active. Without a chip use the cached real front app only.
    pub(crate) fn intent_tab_browser(&self) -> Option<&'static str> {
        if let Some(staged) = self.one_window.staged() {
            return crate::one_window::staging::subject_app(&staged.subject)
                .and_then(crate::browser_tabs::named_tab_browser);
        }
        if let Some(page)=self.one_window.subject_page() {return crate::one_window::staging::subject_app(&page.source.subject).and_then(crate::browser_tabs::named_tab_browser);}
        if self.main_services.is_production() {
            crate::frontmost_app_tracker::get_last_real_app()
                .and_then(|app| crate::browser_tabs::named_tab_browser(&app.bundle_id))
        } else {
            None
        }
    }

    fn intent_window_scope(&self) -> WindowScope {
        let page = self.one_window.page_for_action();
        if let Some(staged) = self.one_window.staged() {
            let subject = page.as_ref().map_or(&staged.subject, |p| &p.source.subject);
            WindowScope::Staged(self.target_for_window_subject(subject))
        } else if let Some(page) =
            page.filter(|p| p.source.subject.kind == sk_one_window::SubjectKind::Window)
        {
            WindowScope::WindowTab(self.target_for_window_subject(&page.source.subject))
        } else {
            WindowScope::Screen
        }
    }

    /// Fingerprints the screen Jev reads against, so a remembered reading is for this screen.
    pub(crate) fn intent_screen_key(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        if self.main_services.is_production() {
            live::screen_key().hash(&mut hasher);
        }
        self.one_window.page_for_action().map(|p|p.id).hash(&mut hasher);
        self.intent_window_scope().hash(&mut hasher);
        self.one_window_host.cursor_voice.dictated_input.as_ref().is_some_and(|s| s==&self.filter_text).hash(&mut hasher);
        self.intent_tab_browser().hash(&mut hasher);
        self.intent_catalog_revision().hash(&mut hasher);
        self.intent_launcher.context_revision().hash(&mut hasher);
        hasher.finish()
    }

    /// Asks Jev to read `text`; the answer goes back to the one-window host as request `id`.
    pub(crate) fn ask_jev(&mut self, id: u64, text: String, screen: u64, cx: &mut Context<Self>) {
        self.settle_intent_oracle_slot();
        if self.main_services.is_production() && (self.intent_launcher.is_off() || crate::intent::sensitive_words(&text) || !crate::intent::wants_reading(&text)) {
            self.one_window_jev_answered(id, text, screen, Err(JevError::Refused("local request")), cx);
            return;
        }
        // An owned service may supply a delayed plan rather than a remote request.
        // Its result still crosses the real request-id, text and context checks.
        if let Some(plan) = self.main_services.owned_sources().and_then(|sources|
            sources.delayed_test_readings.iter().find(|p| p.request == text)).cloned() {
            cx.spawn(async move |this, cx| {
                // Exercise both sides of the five-second fallback deadline. The
                // second, deliberately different response has the same request id.
                let duplicate=(text=="superwhisper").then(|| {
                    let mut duplicate=plan.clone(); duplicate.reading.title="Obsolete second answer".into(); duplicate.alternatives.clear(); duplicate
                });
                cx.background_executor().timer(std::time::Duration::from_millis(if duplicate.is_some() { 6500 } else { 1500 })).await;
                let _ = this.update(cx, |app, cx| app.one_window_jev_answered(id, text.clone(), screen, Ok(plan), cx));
                if let Some(plan)=duplicate {
                    cx.background_executor().timer(std::time::Duration::from_millis(1600)).await;
                    let _ = this.update(cx, |app,cx| app.one_window_jev_answered(id,text,screen,Ok(plan),cx));
                }
            }).detach();
            return;
        }
        let world = if self.main_services.is_production() {
            None
        } else {
            intent_fixture_world()
        };
        let catalog = match &world {
            Some(world) => world.catalog(),
            None => self.intent_catalog(),
        };
        let slot = self.intent_launcher.oracle_slot().clone();
        let last_action = self.intent_launcher.last_action().cloned();
        let live_screen = self.main_services.is_production();
        let window_scope = self.intent_window_scope();
        let dictated=self.one_window_host.cursor_voice.dictated_input.as_ref()==Some(&text);
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
                        let mut screen = live::live_screen(last_action);
                        screen.window_snapshot.scope = window_scope;
                        screen
                    } else if let Some(world) = &world {
                        intent_engine::ScreenContext {
                            last_action,
                            ..world.screen_for(&request)
                        }
                    } else {
                        intent_engine::ScreenContext {
                            id: "fixture".into(),
                            last_action,
                            subject_read_at_run: true,
                            ..intent_engine::ScreenContext::default()
                        }
                    };
                    let speech=crate::intent::jev::DictatedOracle(oracle.as_ref());
                    let reader:&dyn Oracle=if dictated{&speech}else{oracle.as_ref()};
                    let plan = intent_engine::understand(
                        &request,
                        &screen,
                        &catalog,
                        reader,
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
        let screen = self.intent_screen_key();
        let plan = self.intent_launcher.plan_for_row(key, screen);
        if let Some(plan) = plan {
            if let Some(why) = intent_engine::launcher::unavailable(&plan) {
                self.one_window_notice(crate::line_notice::INFO, format!("Can't do this yet · needs {why}"), cx);
                return Some(Err("intent_plan_unavailable"));
            }
            if self.one_window_open_plan(&plan, cx) {
                return Some(Ok(()));
            }
            if !plan.ready {
                self.explain_unready_plan(&plan, cx);
                return Some(Err("intent_plan_not_ready"));
            }
            return Some(self.execute_intent_plan(&plan, cx));
        }
        match key {
            "down:note" => {
                self.one_window_open_note(text.to_string(), cx);
                Some(Ok(()))
            }
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
        if !plan.ready || intent_engine::launcher::unavailable(plan).is_some() {
            return Err("intent_plan_not_ready");
        }
        if plan.pathway != intent_engine::Pathway::Automate && plan.steps.iter().any(intent_engine::launcher::quits_application) {
            let held = plan.clone();
            self.one_window_hold("power", &plan.title, Box::new(move |app,cx| {
                if let Err(error) = app.execute_admitted_intent(&held,cx) {
                    app.one_window_notice(crate::line_notice::FAILED,format!("Couldn't run the requested quit: {error}"),cx);
                }
            }),cx);
            return Ok(());
        }
        self.execute_admitted_intent(plan,cx)
    }

    fn execute_admitted_intent(&mut self, plan: &Arc<Plan>, cx: &mut Context<Self>) -> Result<(), &'static str> {
        tracing::info!(
            target: "script_kit::intent",
            event = "intent_plan_run",
            pathway = plan.pathway.key(),
            steps = plan.steps.len(),
            clauses = plan.clauses.len(),
        );
        // Inspection and approval belong to one live window. Reject a mixed plan
        // before any sibling step can run, even when the inspection is last.
        if plan.steps.iter().any(|s| matches!(s, Step::InspectFilename { .. }))
            && (plan.steps.len() != 1 || plan.pathway == intent_engine::Pathway::Automate)
        {
            return Err("intent_step_not_combinable");
        }
        // A stale window or a refused bounds change isn't something the user just did.
        let window_action = plan.pathway != intent_engine::Pathway::Automate
            && plan.steps.iter().any(|step| matches!(step, Step::Window { .. }));
        let mut record = last_action_for(plan);
        if !window_action {
            if let Some(action) = record.take() { self.intent_launcher.record(action); }
        }
        if plan.pathway == intent_engine::Pathway::Automate {
            return self.save_intent_plan_as_script(plan, cx).map(|_| ());
        }
        if let [only] = plan.steps.as_slice() {
            match only {
                Step::InspectFilename { app, pid, window_id, title, filename } => {
                    return self.inspect_desktop_filename(app.clone(), *pid, *window_id, title.clone(), filename.clone(), cx);
                }
                Step::FocusTab { app, query } if !self.main_services.is_production() => {
                    // Explicit transport receipt: owned evaluation must never activate a real app.
                    crate::runtime_policy::record_completed_fixture_effect();
                    self.one_window_notice(crate::line_notice::INFO, format!("Requested “{query}” tab in {app}"), cx);
                    return Ok(());
                }
                Step::Capture { .. } | Step::Recall { .. } => {
                    return self.pivot_intent_to_typed_line(plan, cx);
                }
                Step::Remember { text, subject } => {
                    return self.run_intent_remember(text, subject.as_deref(), cx);
                }
                _ => {}
            }
        }

        if window_action && (!self.main_services.is_production()
            || crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeInput).is_err()) {
            return Err("native_effects_forbidden");
        }
        if plan.steps.iter().any(needs_accessibility)
            && !crate::platform::accessibility::permissions::has_accessibility_permission()
        {
            self.ask_for_accessibility(&plan.steps, cx);
            return Err("intent_needs_accessibility");
        }
        if plan.steps.iter().any(needs_window_choice) {
            self.one_window_notice(crate::line_notice::INFO, "Choose one window and try again", cx);
            return Err("intent_window_not_identified");
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
                Step::FocusTab { .. } | Step::Menu { .. } | Step::Window { .. } | Step::OpenUrl { .. } => {
                    later.push(step.clone())
                }
                Step::InspectFilename { .. }
                | Step::Recall { .. }
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
        self.run_desktop_steps(later, record, cx);
        Ok(())
    }

    /// The staged row brings its observation; hiding Script Kit grants no new authority.
    pub(crate) fn run_window_op(
        &mut self,
        op: intent_engine::WindowOp,
        app: String,
        target: Option<WindowTarget>,
        cx: &mut Context<Self>,
    ) {
        if !self.main_services.is_production()
            || crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeInput)
                .is_err()
        {
            self.one_window_notice(
                crate::line_notice::INFO,
                "Desktop access is disabled in this session",
                cx,
            );
            return;
        }
        let step = Step::Window { op, app, target };
        if needs_accessibility(&step)
            && !crate::platform::accessibility::permissions::has_accessibility_permission()
        {
            self.ask_for_accessibility(std::slice::from_ref(&step), cx);
            return;
        }
        if needs_window_choice(&step) {
            self.one_window_notice(
                crate::line_notice::INFO,
                "Choose one window and try again",
                cx,
            );
            return;
        }
        self.close_and_reset_window(cx);
        self.run_desktop_steps(vec![step], None, cx);
    }

    /// Menu, window and URL steps drive other apps, off the main thread; a step that fails says
    /// why in the line and brings the window back to show it.
    fn run_desktop_steps(&mut self, steps: Vec<Step>, record: Option<LastAction>, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let failed = cx
                .background_executor()
                .spawn(async move {
                    steps.iter().find_map(|step| {
                        let error = run_desktop_step(step).err()?;
                        tracing::warn!(
                            target: "script_kit::intent",
                            event = "intent_step_failed",
                            error = %error,
                        );
                        Some(desktop_step_failure(step, &error))
                    })
                })
                .await;
            let Some(words) = failed else {
                if let Some(action) = record {
                    let _ = this.update(cx, |app, _| app.intent_launcher.record(action));
                }
                return;
            };
            let _ = this.update(cx, |app, cx| {
                app.one_window_notice(crate::line_notice::FAILED, words, cx);
                if !script_kit_gpui::is_main_window_visible() {
                    script_kit_gpui::request_show_main_window();
                }
            });
        })
        .detach();
    }

    /// Menu and window steps drive other apps through Accessibility; without it they can't run,
    /// so the window stays and says how to turn it on.
    pub(crate) fn ask_for_accessibility(&mut self, steps: &[Step], cx: &mut Context<Self>) {
        let title = match steps.iter().find_map(|step| match step {
            Step::Menu { app, .. } => Some(app.as_str()),
            _ => None,
        }) {
            Some(app) => format!("Script Kit needs Accessibility access to use {app}'s menus"),
            None => "Script Kit needs Accessibility access to move windows".to_string(),
        };
        let open: crate::one_window::ledger_view::LedgerRun = Box::new(|app, cx| {
            if let Err(error) = crate::platform::accessibility::permissions::open_accessibility_settings()
            {
                app.one_window_notice(
                    crate::line_notice::FAILED,
                    format!("Couldn't open System Settings: {error}"),
                    cx,
                );
            }
        });
        self.one_window_ask(
            crate::line_notice::FAILED,
            title,
            "System Settings › Privacy & Security › Accessibility",
            vec![("Open Accessibility Settings".to_string(), open)],
            None,
            cx,
        );
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
            self.execute_selected(cx).map_err(|_| "intent_capture_not_dispatched")?;
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
        self.one_window_notice(crate::line_notice::INFO, "Saved to today's note".to_string(), cx);
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
            Step::Window { op, app, .. } => Some(LastStep::Window {
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
            Step::OpenUrl { url, app } => Some(LastStep::OpenUrl {
                url: url.clone(),
                app: app.clone(),
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
    if text.trim().is_empty() { return Err("intent_memory_empty"); }
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

fn needs_window_choice(step: &Step) -> bool {
    matches!(step, Step::Window { op, target: None, .. } if op.action() != Some(WindowAction::Quit))
}

/// Whether a step drives another app through Accessibility. Quitting goes through AppleScript.
fn needs_accessibility(step: &Step) -> bool {
    match step {
        Step::Menu { .. } => true,
        Step::Window { op, .. } => op.action() != Some(WindowAction::Quit),
        _ => false,
    }
}

/// What the line says when a menu or window step didn't happen.
fn desktop_step_failure(step: &Step, error: &anyhow::Error) -> String {
    match step {
        Step::Menu { app, path, .. } => {
            let item = path.join(" › ");
            #[cfg(target_os = "macos")]
            {
                use script_kit_gpui::menu_executor::MenuExecutorError;
                match error.downcast_ref::<MenuExecutorError>() {
                    Some(MenuExecutorError::AccessibilityPermissionDenied) => {
                        return format!(
                            "Script Kit needs Accessibility access to use {app}'s menus"
                        );
                    }
                    Some(MenuExecutorError::AppNotFrontmost { .. }) => {
                        return format!("{app} wasn't in front, so “{item}” didn't run");
                    }
                    Some(MenuExecutorError::MenuItemDisabled { .. }) => {
                        return format!("“{item}” is turned off in {app} right now");
                    }
                    Some(
                        MenuExecutorError::MenuItemNotFound { .. }
                        | MenuExecutorError::MenuStructureChanged { .. },
                    ) => {
                        return format!("{app} has no “{item}” menu item");
                    }
                    Some(MenuExecutorError::ActionFailed(_)) => {
                        return format!("{app} didn't respond to “{item}”");
                    }
                    None => {}
                }
            }
            format!("Couldn't use “{item}” in {app}: {error}")
        }
        Step::Window { op, app, .. } => format!("{} didn't work on {app}: {error}", op.label),
        Step::OpenUrl { url, app: Some(app) } => format!("Couldn't open {url} in {app}: {error}"),
        Step::OpenUrl { url, app: None } => format!("Couldn't open {url}: {error}"),
        _ => error.to_string(),
    }
}

fn run_desktop_step(step: &Step) -> anyhow::Result<()> {
    if let Step::FocusTab { app, query } = step {
        return crate::browser_tabs::focus_named_tab(app, query);
    }
    // A second attempt would open a second tab.
    if let Step::OpenUrl { url, app } = step {
        return live::open_url(url, app.as_deref());
    }
    match step {
        Step::Window { op, app, target } => live::run_window_step(op, app, target.as_ref()),
        #[cfg(target_os = "macos")]
        Step::Menu { bundle_id, path, .. } => script_kit_gpui::menu_executor::execute_menu_action(bundle_id, path),
        _ => anyhow::bail!("unsupported desktop step"),
    }
}
