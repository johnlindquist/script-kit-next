//! The intent row: Jev reads the typed sentence against the screen and the launcher's
//! commands, and a ready plan for exactly the current text is pinned first so Enter
//! does what the sentence asked for. Typing never waits on Jev; when Jev is off or
//! fails, the list is the ordinary launcher list.

use super::*;

use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::intent_live as live;
use crate::intent::launcher::{DEBOUNCE, HELD_ENTER_LIMIT};
use crate::intent::{
    self as intent_engine, CommandKind, EngineOptions, Finish, HeldOutcome, JevClient, JevConfig,
    JevError, LastAction, LastStep, Oracle, OracleSlot, Plan, ScriptOptions, Step, TextChange,
    Ticket,
};

/// Window and menu steps wait for apps opened earlier in the same plan.
const WINDOW_STEP_ATTEMPTS: u32 = 8;
const WINDOW_STEP_RETRY: Duration = Duration::from_millis(250);

/// `SCRIPT_KIT_JEV_INTENT=0` turns the intent row off.
const INTENT_ENV_TOGGLE: &str = "SCRIPT_KIT_JEV_INTENT";

impl ScriptListApp {
    fn intent_reading_eligible(&self, text: &str) -> bool {
        matches!(self.current_view, AppView::ScriptList)
            && intent_engine::wants_reading(text)
            && self.inline_calculator.is_none()
            && self.active_script_list_attachment_portal_kind().is_none()
            && !crate::menu_syntax::active_filter_head_owns_main_list(text)
            && self.menu_syntax_mode.capture_for(text).is_none()
            && self.menu_syntax_mode.command_for(text).is_none()
            && !self.menu_syntax_mode.capture_composer_owns_input_for(text)
            && !self.menu_syntax_mode.command_owns_input_for(text)
    }

    fn settle_intent_oracle_slot(&mut self) {
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

    /// Called with each committed query, before the list is rebuilt, so a stale plan
    /// never reaches the new list.
    pub(crate) fn maybe_start_intent_reading(&mut self, text: &str, cx: &mut Context<Self>) {
        self.settle_intent_oracle_slot();
        let eligible = self.intent_reading_eligible(text);
        let screen_key = if self.main_services.is_production() {
            live::screen_key()
        } else {
            0
        };
        match self
            .intent_launcher
            .text_changed(text, eligible, screen_key)
        {
            TextChange::Skip | TextChange::Recalled => {}
            TextChange::Read(ticket) => self.spawn_intent_reading(ticket, cx),
        }
    }

    fn spawn_intent_reading(&mut self, ticket: Ticket, cx: &mut Context<Self>) {
        let catalog = self.intent_catalog();
        let slot = self.intent_launcher.oracle_slot().clone();
        let last_action = self.intent_launcher.last_action().cloned();
        let live_screen = self.main_services.is_production();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(DEBOUNCE).await;
            let still_current = cx.update(|cx| {
                this.update(cx, |app, _| app.intent_launcher.is_current(&ticket))
                    .unwrap_or(false)
            });
            if !still_current {
                return;
            }
            let request = ticket.request.clone();
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
                    app.finish_intent_reading(&ticket, loaded, result, cx);
                })
            });
        })
        .detach();
    }

    fn finish_intent_reading(
        &mut self,
        ticket: &Ticket,
        loaded: Option<Arc<dyn Oracle>>,
        result: Result<Plan, JevError>,
        cx: &mut Context<Self>,
    ) {
        if let Some(oracle) = loaded {
            if matches!(self.intent_launcher.oracle_slot(), OracleSlot::Unloaded) {
                self.intent_launcher
                    .set_oracle_slot(OracleSlot::Ready(oracle));
            }
        }
        if let Err(error) = &result {
            tracing::info!(
                target: "script_kit::intent",
                event = "intent_reading_failed",
                error = %error,
            );
        }
        let current = self.filter_text.clone();
        let finish = self.intent_launcher.finish(ticket, &current, result);
        if let Some(plan) = self.intent_launcher.plan_for(&current) {
            tracing::info!(
                target: "script_kit::intent",
                event = "intent_plan_shown",
                pathway = plan.pathway.key(),
                steps = plan.steps.len(),
                clauses = plan.clauses.len(),
            );
        }
        if finish == Finish::Stale {
            return;
        }
        if finish == Finish::Shown {
            // A plan is the answer to the sentence, so it takes the highlight from the
            // automatic top row, but never from a row the user picked.
            if !matches!(
                self.main_menu_selection_intent(),
                MainMenuSelectionIntent::ExplicitAnchor { .. }
            ) {
                self.reset_main_menu_selection_intent();
            }
            self.commit_main_menu_results_refresh("intent_plan_ready", None, cx, |_, _| true);
        }
        if self.intent_launcher.held_enter().is_some() {
            self.release_held_intent_enter(cx);
        }
    }

    fn intent_selected_row_key(&self) -> Option<String> {
        match self.resolved_main_menu_selected_subject()? {
            ResolvedMainMenuSelection::SearchResult { row, .. }
            | ResolvedMainMenuSelection::Calculator { row, .. } => Some(row.stable_key.clone()),
        }
    }

    /// Enter during a reading of the same text waits for it, unless the user picked a
    /// row. Returns true when Enter is held.
    pub(crate) fn hold_enter_for_intent_reading(&mut self, cx: &mut Context<Self>) -> bool {
        let text = self.computed_filter_text.clone();
        let anchored = matches!(
            self.main_menu_selection_intent(),
            MainMenuSelectionIntent::ExplicitAnchor { .. }
        );
        let shown = self.intent_selected_row_key();
        if !self
            .intent_launcher
            .hold_enter(&text, anchored, shown, Instant::now())
        {
            return false;
        }
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(HELD_ENTER_LIMIT).await;
            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    if app.intent_launcher.held_enter().is_some() {
                        app.intent_launcher.abandon_reading();
                        app.release_held_intent_enter(cx);
                    }
                })
            });
        })
        .detach();
        true
    }

    fn release_held_intent_enter(&mut self, cx: &mut Context<Self>) {
        let text = self.computed_filter_text.clone();
        let selected = self.intent_selected_row_key();
        if self
            .intent_launcher
            .release_enter(&text, selected.as_deref())
            == Some(HeldOutcome::Run)
        {
            let _dispatch = self.execute_selected(cx);
        }
    }

    /// Pins the plan for exactly this text above everything else, unless the top row
    /// already does the same single command.
    pub(crate) fn pin_intent_row(
        &self,
        mut grouped: Vec<crate::list_item::GroupedListItem>,
        mut flat: Vec<scripts::SearchResult>,
        text: &str,
        ranking: &mut scripts::command_contract::MainMenuRankingEvidenceMap,
    ) -> (
        Vec<crate::list_item::GroupedListItem>,
        Vec<scripts::SearchResult>,
    ) {
        if self.inline_calculator.is_some() {
            return (grouped, flat);
        }
        let Some(plan) = self.intent_launcher.plan_for(text).cloned() else {
            return (grouped, flat);
        };
        if let [Step::Command { command }] = plan.steps.as_slice() {
            let top = grouped.iter().find_map(|item| match item {
                crate::list_item::GroupedListItem::Item(idx) => flat.get(*idx),
                _ => None,
            });
            if top.and_then(intent_command_id_for_result).as_deref() == Some(command.id.as_str()) {
                return (grouped, flat);
            }
        }
        flat.insert(
            0,
            scripts::SearchResult::Intent(scripts::IntentMatch::new(plan)),
        );
        if let Some(key) = flat[0].stable_selection_key() {
            ranking.entry(key).or_default().pin_reason = Some("intent-plan");
        }
        for item in grouped.iter_mut() {
            if let crate::list_item::GroupedListItem::Item(idx) = item {
                *idx += 1;
            }
        }
        grouped.insert(0, crate::list_item::GroupedListItem::Item(0));
        (grouped, flat)
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
                Step::Ask { question, subject } => {
                    let variant = if subject.is_some() {
                        crate::ai::agent_chat::ui::ui_variant::AgentChatUiVariant::Standard
                    } else {
                        crate::ai::agent_chat::ui::ui_variant::AgentChatUiVariant::QuickAi
                    };
                    self.open_tab_ai_agent_chat_with_entry_intent_variant(
                        Some(question.clone()),
                        variant,
                        cx,
                    );
                    return Ok(());
                }
                Step::Delegate { task, project, .. } => {
                    let prompt = match project {
                        Some(project) => format!("In {project}: {task}"),
                        None => task.clone(),
                    };
                    self.open_tab_ai_agent_chat_with_entry_intent_variant(
                        Some(prompt),
                        crate::ai::agent_chat::ui::ui_variant::AgentChatUiVariant::Standard,
                        cx,
                    );
                    return Ok(());
                }
                Step::Transform {
                    tool: None,
                    instruction,
                    ..
                } => {
                    self.open_tab_ai_agent_chat_with_entry_intent_variant(
                        instruction.clone(),
                        crate::ai::agent_chat::ui::ui_variant::AgentChatUiVariant::Standard,
                        cx,
                    );
                    return Ok(());
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
                self.execute_app(&app, cx);
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
        self.show_hud("Remembered".to_string(), Some(HUD_MEDIUM_MS), cx);
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
        self.show_hud(
            format!("Saved script: {}", draft.name),
            Some(HUD_MEDIUM_MS),
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
        if let Err(reason) = self.save_intent_plan_as_script(&intent_match.plan, cx) {
            tracing::warn!(target: "script_kit::intent", reason, "intent script not saved");
            self.show_hud(
                "Couldn't save the script".to_string(),
                Some(HUD_MEDIUM_MS),
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

fn append_capture_to_day(target: &str, body: &str, when: Option<&str>) -> Result<(), &'static str> {
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
