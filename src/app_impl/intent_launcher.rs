//! The intent row: Jev reads the typed sentence against the screen and the launcher's
//! commands. The row sits second, under the closest match, from the sentence's first
//! frame; it reads until Jev answers, then shows the plan in place. ⌘↵ runs it from any
//! row. Typing never waits on Jev, and Jev's answer never moves a row or the highlight.

use super::*;

use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::intent_live as live;
use crate::intent::launcher::{DEBOUNCE, HELD_ENTER_LIMIT};
use crate::intent::{
    self as intent_engine, CommandKind, EngineOptions, Finish, HeldOutcome, JevClient, JevConfig,
    JevError, LastAction, LastStep, Oracle, OracleSlot, Plan, ScriptOptions, Step, TextChange,
    RowState, Ticket,
};

/// Window and menu steps wait for apps opened earlier in the same plan.
const WINDOW_STEP_ATTEMPTS: u32 = 8;
const WINDOW_STEP_RETRY: Duration = Duration::from_millis(250);

/// `SCRIPT_KIT_JEV_INTENT=0` turns the intent row off.
const INTENT_ENV_TOGGLE: &str = "SCRIPT_KIT_JEV_INTENT";

impl ScriptListApp {
    fn intent_reading_eligible(&self, text: &str) -> bool {
        intent_engine::wants_reading(text) && self.intent_view_eligible(text)
    }

    /// Launcher syntax, the calculator, and attachment portals own the list.
    fn intent_view_eligible(&self, text: &str) -> bool {
        matches!(self.current_view, AppView::ScriptList)
            && !text.trim().is_empty()
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
        let eligible = self.intent_reading_eligible(text);
        self.start_intent_reading(text, eligible, cx);
    }

    fn start_intent_reading(&mut self, text: &str, eligible: bool, cx: &mut Context<Self>) {
        self.settle_intent_oracle_slot();
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
        let current = self.computed_filter_text.clone();
        let finish = self.intent_launcher.finish(ticket, &current, result);
        if finish == Finish::Stale {
            return;
        }
        if let Some(RowState::Read(plan)) = self
            .intent_launcher
            .row_for(&current)
            .map(|row| &row.state)
        {
            tracing::info!(
                target: "script_kit::intent",
                event = "intent_plan_shown",
                pathway = plan.pathway.key(),
                ready = plan.ready,
                missing = plan.missing.as_deref().unwrap_or(""),
                steps = plan.steps.len(),
                clauses = plan.clauses.len(),
            );
        }
        self.commit_main_menu_results_refresh("intent_row_read", None, cx, |_, _| true);
        if self.intent_launcher.held_enter().is_some() {
            self.release_held_intent_enter(cx);
        }
    }

    fn intent_row_selected(&self) -> bool {
        match self.resolved_main_menu_selected_subject() {
            Some(ResolvedMainMenuSelection::SearchResult { row, .. }) => {
                row.stable_key == scripts::INTENT_ROW_KEY
            }
            _ => false,
        }
    }

    /// Enter on the intent row while Jev reads waits for the reading. Returns true when
    /// Enter is held.
    pub(crate) fn hold_enter_for_intent_reading(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.intent_row_selected() {
            return false;
        }
        let text = self.computed_filter_text.clone();
        let pressed_at = Instant::now();
        if !self.intent_launcher.hold_enter(&text, pressed_at) {
            if self.intent_launcher.plan_for(&text).is_none() {
                self.explain_intent_row(&text, cx);
            }
            return false;
        }
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(HELD_ENTER_LIMIT).await;
            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    if app
                        .intent_launcher
                        .held_enter()
                        .is_some_and(|held| held.pressed_at == pressed_at)
                    {
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
        let on_row = self.intent_row_selected();
        match self.intent_launcher.release_enter(&text, on_row) {
            Some(HeldOutcome::Run) => {
                let _dispatch = self.execute_selected(cx);
            }
            Some(HeldOutcome::Wait) => self.explain_intent_row(&text, cx),
            Some(HeldOutcome::Moved) | None => {}
        }
    }

    /// Why the intent row can't run yet.
    fn explain_intent_row(&mut self, text: &str, cx: &mut Context<Self>) {
        let Some(row) = self.intent_launcher.row_for(text) else {
            return;
        };
        let message = match &row.state {
            RowState::Reading => "Still understanding…".to_string(),
            RowState::Read(plan) => match plan.missing.as_deref() {
                Some(missing) => format!("Needs {missing}"),
                None => "Needs more to go on".to_string(),
            },
            RowState::Failed => "Couldn't understand that".to_string(),
        };
        self.show_hud(message, Some(HUD_MEDIUM_MS), cx);
    }

    /// ⌘↵ runs what the sentence means from any row: the highlight moves to the intent
    /// row, so the user sees what runs, and Enter waits there while Jev reads. A single
    /// word gets a reading on demand. Returns false when Jev is off.
    pub(crate) fn run_intent_for_current_input(&mut self, cx: &mut Context<Self>) -> bool {
        if !matches!(self.current_view, AppView::ScriptList) || self.filter_text.trim().is_empty()
        {
            return false;
        }
        self.flush_pending_main_menu_query(cx);
        let text = self.computed_filter_text.clone();
        if self.intent_launcher.row_for(&text).is_none() {
            if !self.intent_view_eligible(&text) {
                return false;
            }
            self.start_intent_reading(&text, true, cx);
            if self.intent_launcher.row_for(&text).is_none() {
                return false;
            }
            self.commit_main_menu_results_refresh("intent_row_requested", None, cx, |_, _| true);
        }
        let Some(index) = self
            .main_menu_committed_rows()
            .iter()
            .find(|row| row.stable_key == scripts::INTENT_ROW_KEY)
            .map(|row| row.grouped_index)
        else {
            return false;
        };
        if self.select_main_menu_row(index, crate::MainMenuSelectionOrigin::Keyboard, cx) {
            self.reveal_main_list_selection_above_footer("intent_cmd_enter");
            cx.notify();
        }
        let reading = self.intent_launcher.is_reading(&text);
        let runnable = self.intent_launcher.plan_for(&text).is_some();
        tracing::info!(
            target: "script_kit::intent",
            event = "intent_cmd_enter",
            reading,
            runnable,
        );
        if reading || runnable {
            let _dispatch = self.execute_selected(cx);
        } else {
            self.explain_intent_row(&text, cx);
        }
        true
    }

    /// ⌘↵ in the main window: what the sentence means, or Agent Chat when there's no
    /// sentence to read or Jev is off.
    pub(crate) fn route_main_window_cmd_enter(&mut self, cx: &mut Context<Self>) -> bool {
        self.run_intent_for_current_input(cx)
            || self.try_route_global_cmd_enter_to_agent_chat_context_capture(cx)
    }

    /// Whether ⌘↵ runs the sentence rather than opening Agent Chat, for the footer.
    pub(crate) fn cmd_enter_runs_intent(&self) -> bool {
        !self.intent_launcher.is_off()
            && !std::env::var(INTENT_ENV_TOGGLE).is_ok_and(|value| value.trim() == "0")
            && self.intent_view_eligible(&self.filter_text)
    }

    /// The intent row goes second, under the closest match, or first when nothing else
    /// matches. Its key never changes, so Jev's answer rewrites it in place.
    pub(crate) fn place_intent_row(
        &self,
        mut grouped: Vec<crate::list_item::GroupedListItem>,
        mut flat: Vec<scripts::SearchResult>,
        text: &str,
        ranking: &mut scripts::command_contract::MainMenuRankingEvidenceMap,
    ) -> (
        Vec<crate::list_item::GroupedListItem>,
        Vec<scripts::SearchResult>,
    ) {
        use crate::list_item::GroupedListItem;
        if self.inline_calculator.is_some() {
            return (grouped, flat);
        }
        let Some(row) = self.intent_launcher.row_for(text) else {
            return (grouped, flat);
        };
        let (grouped_at, flat_at) = grouped
            .iter()
            .enumerate()
            .find_map(|(at, item)| match item {
                GroupedListItem::Item(idx) => Some((at + 1, idx + 1)),
                _ => None,
            })
            .unwrap_or((grouped.len(), flat.len()));
        flat.insert(
            flat_at,
            scripts::SearchResult::Intent(scripts::IntentMatch::from_row(row)),
        );
        ranking
            .entry(scripts::INTENT_ROW_KEY.to_string())
            .or_default()
            .pin_reason = Some("intent-row");
        for item in grouped.iter_mut() {
            if let GroupedListItem::Item(idx) = item {
                if *idx >= flat_at {
                    *idx += 1;
                }
            }
        }
        grouped.insert(grouped_at, GroupedListItem::Item(flat_at));
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
        let Some(plan) = intent_match.plan else {
            let request = intent_match.request;
            self.explain_intent_row(&request, cx);
            return true;
        };
        if let Err(reason) = self.save_intent_plan_as_script(&plan, cx) {
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
