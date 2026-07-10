// Flow Desk (Conversation Desk → Threadline, 2026-07-09).
//
// Every flow is an agent identity. One main-window surface over the shared
// flow substrate (`crate::flows`):
//
// - Enter on a flow = CONVERSE: open a Threadline session — Script Kit's
//   own `ChatPrompt` transcript + composer. No engine TUI is ever wrapped.
//   Codex-engine flows talk to a persistent `codex app-server` thread
//   (`crate::flows::codex_client`); other engines run one
//   `md <flow> --_task … --events` registry run per turn (second-class).
// - Enter on an Active session row = reattach the SAME transcript entity.
// - ⇧↵ = run once in the background via `md <flow> --events` (registry).
// - Esc in a session = background (never kills); ⌘⇧D does the same. Esc in
//   the desk clears the filter / goes back. Stop is an explicit ⌘K verb
//   that cancels only the in-flight turn.
//
// The detached Flow Manager and the Flash/Dispatch/Lens/Mission-Control
// variants are dead; `FlowUxVariant` survives only as builtin-entry plumbing.

/// One selectable row in the desk list.
#[derive(Clone)]
pub(crate) enum FlowDeskRow {
    /// Live or recently-ended conversation (index into `flow_sessions`).
    Session(u64),
    /// A flow identity from the combined roster+package corpus.
    Flow(crate::flows::model::FlowDescriptor),
    /// The plain-English creation affordance (always last).
    CreateFlow,
}

/// How a settled turn presents in the transcript: normal completion, a quiet
/// user-initiated stop (never the red error treatment), or a real failure.
#[derive(Clone)]
pub(crate) enum FlowTurnOutcome {
    Ok,
    Stopped,
    Failed(String),
}

/// What the Flow Desk ⌘K dialog acts on. Derived fresh from view state at
/// toggle/execute time so the popup never captures a stale row.
#[derive(Clone)]
pub(crate) enum FlowDeskSubject {
    /// A flow identity row (or the desk's flow list generally).
    Flow(crate::flows::model::FlowDescriptor),
    /// A conversation session by id — selected row or the open session view.
    Session(u64),
    /// The Create Flow affordance.
    Create,
}

impl ScriptListApp {
    /// Effective cwd for flow discovery: the spine cwd chip when set,
    /// otherwise $HOME. mdflow resolves project vs global flows from here.
    pub(crate) fn flow_ux_cwd(&self) -> String {
        crate::flows::resolve_flow_cwd(
            self.spine_cwd
                .as_ref()
                .map(|cwd| cwd.to_string_lossy().to_string()),
        )
    }

    /// Short human form of the flow cwd for chips/empty states: `~`-relative
    /// when under $HOME, and never more than the last two components.
    fn flow_ux_cwd_display(cwd: &str) -> String {
        let home = std::env::var("HOME").unwrap_or_default();
        if !home.is_empty() && cwd == home {
            return "~".to_string();
        }
        let tail: Vec<&str> = cwd.rsplit('/').filter(|s| !s.is_empty()).take(2).collect();
        match tail.as_slice() {
            [last, parent] => format!("{parent}/{last}"),
            [last] => (*last).to_string(),
            _ => cwd.to_string(),
        }
    }

    /// Spawn the repaint tick that keeps flow surfaces live. Single
    /// instance; exits when nothing is active. The tick is the ONLY seam
    /// where transport events reach GPUI entities: codex app-server events
    /// and mdflow run tails apply here on the main thread every 120ms.
    /// (ChatPrompt callback requests drain in the render pass instead —
    /// they need window access.)
    pub(crate) fn start_flow_ux_tick(&mut self, cx: &mut Context<Self>) {
        if self.flow_ux_tick_running {
            return;
        }
        self.flow_ux_tick_running = true;
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(120))
                    .await;
                let keep_going = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        let registry = crate::flows::run_registry::flow_run_registry();
                        let generation = registry.generation();
                        let mut dirty = generation != app.flow_ux_seen_generation;
                        app.flow_ux_seen_generation = generation;

                        // ChatPrompt callback requests are drained in the
                        // app render pass (render_impl — needs window
                        // access); this tick owns transport events only.
                        // While a session view is open, repaint every tick
                        // so that drain is guaranteed to run within 120ms
                        // of a callback posting a request.
                        if matches!(app.current_view, AppView::FlowSessionView { .. }) {
                            dirty = true;
                        }

                        // 1. Codex app-server events (native transport).
                        for event in
                            crate::flows::codex_client::codex_app_server().drain_events()
                        {
                            dirty = true;
                            app.apply_flow_thread_event(event, cx);
                        }

                        // 2. mdflow-turn runs: stream stdout, settle turns.
                        if app.sync_mdflow_turns(cx) {
                            dirty = true;
                        }

                        if dirty {
                            cx.notify();
                        }
                        let view_active = matches!(
                            app.current_view,
                            AppView::FlowUxView { .. } | AppView::FlowSessionView { .. }
                        );
                        let keep = view_active
                            || registry.active_count() > 0
                            || !app.flow_sessions.is_empty();
                        if !keep {
                            app.flow_ux_tick_running = false;
                        }
                        keep
                    })
                });
                match keep_going {
                    Ok(true) => continue,
                    _ => break,
                }
            }
        })
        .detach();
    }

    // ------------------------------------------------------------------
    // Desk corpus + rows
    // ------------------------------------------------------------------

    /// The combined flow corpus for the desk (roster for the effective cwd
    /// plus the installed flows package).
    pub(crate) fn flow_desk_corpus(&self) -> Vec<crate::flows::model::FlowDescriptor> {
        let cwd = self.flow_ux_cwd();
        let roster = crate::flows::catalog::flow_catalog().roster_for(&cwd);
        crate::flows::catalog::desk_flows(&roster)
    }

    /// Build the selectable desk rows for a filter: Active/recent sessions
    /// first (newest first), then matching flows, then Create Flow.
    pub(crate) fn flow_desk_rows(&self, filter: &str) -> Vec<FlowDeskRow> {
        let mut rows: Vec<FlowDeskRow> = Vec::new();
        let query = filter.trim().to_lowercase();

        let mut sessions: Vec<&crate::flows::session::FlowSessionMeta> =
            self.flow_sessions.iter().map(|(meta, _)| meta).collect();
        sessions.sort_by(|a, b| b.id.cmp(&a.id));
        for meta in sessions {
            let matches = query.is_empty()
                || meta.friendly_name.to_lowercase().contains(&query)
                || meta.flow_name.to_lowercase().contains(&query);
            if matches {
                rows.push(FlowDeskRow::Session(meta.id));
            }
        }

        let corpus = self.flow_desk_corpus();
        for flow in crate::flows::catalog::filter_flows(&corpus, filter) {
            rows.push(FlowDeskRow::Flow(flow.clone()));
        }

        rows.push(FlowDeskRow::CreateFlow);
        rows
    }

    fn flow_session_index(&self, session_id: u64) -> Option<usize> {
        self.flow_sessions
            .iter()
            .position(|(meta, _)| meta.id == session_id)
    }

    /// Activate the desk's selected row — Enter (`run_once: false`) and ⇧↵
    /// (`run_once: true`) share this with the native footer buttons so
    /// keyboard and footer can never diverge.
    pub(crate) fn flow_desk_activate_selected(&mut self, run_once: bool, cx: &mut Context<Self>) {
        let AppView::FlowUxView {
            filter,
            selected_index,
            ..
        } = &self.current_view
        else {
            return;
        };
        let filter = filter.clone();
        let selected = *selected_index;
        let rows = self.flow_desk_rows(&filter);
        let Some(row) = rows.get(selected).cloned() else {
            return;
        };
        match row {
            FlowDeskRow::Session(session_id) => {
                self.open_flow_session(session_id, cx);
            }
            FlowDeskRow::Flow(flow) => {
                if run_once || flow.is_workflow {
                    // Workflows (DAGs) are run-once by nature; ⇧↵ is
                    // explicit run-once for anything.
                    self.flow_desk_run_once(&flow, cx);
                } else {
                    // Typed text rides along as the first message only when
                    // it reads like a request (multi-word, not the flow's
                    // own name) — fuzzy lookups must never become the task.
                    let trimmed = filter.trim();
                    let lowered = trimmed.to_lowercase();
                    let is_name_lookup = trimmed.is_empty()
                        || !trimmed.contains(' ')
                        || flow.name.to_lowercase() == lowered
                        || flow.friendly_name().to_lowercase() == lowered;
                    let first_message = (!is_name_lookup).then(|| trimmed.to_string());
                    self.start_flow_session(&flow, first_message, cx);
                }
            }
            FlowDeskRow::CreateFlow => {
                self.start_flow_create_session(cx);
            }
        }
        cx.notify();
    }

    // ------------------------------------------------------------------
    // Conversation lifecycle
    // ------------------------------------------------------------------

    /// Start a conversation with a flow: create its Threadline (ChatPrompt)
    /// session and show it. `initial_message` (Tab router / typed text)
    /// becomes the first submitted turn.
    pub(crate) fn start_flow_session(
        &mut self,
        flow: &crate::flows::model::FlowDescriptor,
        initial_message: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let cwd = self.flow_ux_cwd();
        crate::flows::manager_window::remember_flow_cwd(&cwd);
        self.flow_session_counter += 1;
        let session_id = self.flow_session_counter;
        let transport = crate::flows::session::SessionTransport::for_engine(&flow.engine);

        // Log length only — message text can carry anything.
        tracing::info!(
            target: "script_kit::flows",
            event = "flow_session_start",
            session_id,
            flow_id = %flow.id,
            transport = ?transport,
            message_len = initial_message.as_deref().map(str::len).unwrap_or(0),
            "Starting flow conversation"
        );

        let friendly = flow.friendly_name();
        let submit_sender = self.flow_chat_sender.clone();
        let submit_callback: crate::prompts::ChatSubmitCallback =
            std::sync::Arc::new(move |_id: String, text: String| {
                let _ = submit_sender.try_send(
                    crate::flows::session::FlowChatRequest::Submit { session_id, text },
                );
            });
        let escape_sender = self.flow_chat_sender.clone();
        let escape_callback: crate::prompts::ChatEscapeCallback =
            std::sync::Arc::new(move |_id: String| {
                let _ = escape_sender.try_send(
                    crate::flows::session::FlowChatRequest::Background { session_id },
                );
            });

        let mut chat = crate::prompts::ChatPrompt::new(
            format!("flow-session-{session_id}"),
            Some(format!("Message {friendly}…")),
            vec![],
            None,
            None,
            self.focus_handle.clone(),
            submit_callback,
            std::sync::Arc::clone(&self.theme),
        )
        .with_title(friendly.clone())
        .with_save_history(false)
        .with_escape_callback(escape_callback)
        .with_escape_over_stop(true)
        .with_external_footer(true)
        .with_external_header(true)
        .with_external_input(true)
        .with_empty_state_note(
            flow.description
                .clone()
                .unwrap_or_else(|| format!("Converse with {friendly}.")),
        )
        .with_top_aligned_turns();
        let actions_sender = self.flow_chat_sender.clone();
        chat.set_on_show_actions(std::sync::Arc::new(move |_id: String| {
            let _ = actions_sender.try_send(
                crate::flows::session::FlowChatRequest::ShowActions { session_id },
            );
        }));
        let entity = cx.new(|_| chat);

        let meta = crate::flows::session::FlowSessionMeta {
            id: session_id,
            flow_id: flow.id.clone(),
            flow_name: flow.name.clone(),
            friendly_name: friendly,
            origin: flow.origin_label().to_string(),
            engine: flow.engine.clone(),
            flow_path: flow.path.clone(),
            cwd,
            transport,
            state: crate::flows::session::SessionState::NeedsYou,
            started_at: std::time::Instant::now(),
            turns: Vec::new(),
            active_turn: None,
        };
        self.flow_sessions.push((meta, entity));
        self.open_flow_session(session_id, cx);
        self.start_flow_ux_tick(cx);
        if let Some(message) = initial_message {
            self.submit_flow_chat_message(session_id, message, cx);
        }
    }

    /// Submit one user message on a session: echo it into the transcript,
    /// open a streaming assistant bubble, and dispatch the turn on the
    /// session's transport. One turn in flight per session.
    pub(crate) fn submit_flow_chat_message(
        &mut self,
        session_id: u64,
        text: String,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self.flow_session_index(session_id) else {
            return;
        };
        let text = text.trim().to_string();
        if text.is_empty() {
            return;
        }
        if self.flow_sessions[index].0.active_turn.is_some() {
            self.toast_manager.push(
                crate::components::toast::Toast::error(
                    "Still working — stop the current turn first (⌘K)".to_string(),
                    &self.theme,
                )
                .duration_ms(Some(2500)),
            );
            cx.notify();
            return;
        }

        let (transport, prompt) = {
            let meta = &self.flow_sessions[index].0;
            let prompt = match meta.transport {
                crate::flows::session::SessionTransport::CodexThread => {
                    if meta.turns.is_empty() {
                        // First turn carries the flow's resolved mission;
                        // the protocol thread holds context afterwards.
                        let markdown =
                            std::fs::read_to_string(&meta.flow_path).unwrap_or_default();
                        crate::flows::session::resolve_flow_mission(&markdown, &text)
                    } else {
                        text.clone()
                    }
                }
                crate::flows::session::SessionTransport::MdflowTurns => {
                    crate::flows::session::build_turn_task(&meta.turns, &text)
                }
            };
            (meta.transport, prompt)
        };

        let turn_index = self.flow_sessions[index].0.turns.len();
        let message_id = format!("flow-{session_id}-turn-{turn_index}");
        let entity = self.flow_sessions[index].1.clone();
        let user_text = text.clone();
        entity.update(cx, |chat, cx| {
            chat.add_message(crate::protocol::ChatPromptMessage::user(user_text), cx);
            chat.start_streaming(
                message_id.clone(),
                crate::protocol::ChatMessagePosition::Left,
                cx,
            );
        });

        tracing::info!(
            target: "script_kit::flows",
            event = "flow_turn_submit",
            session_id,
            transport = ?transport,
            prompt_len = prompt.len(),
            "Submitting flow turn"
        );

        let run_id = match transport {
            crate::flows::session::SessionTransport::CodexThread => {
                let meta = &self.flow_sessions[index].0;
                crate::flows::codex_client::codex_app_server().converse(
                    session_id,
                    &meta.cwd,
                    prompt,
                );
                None
            }
            crate::flows::session::SessionTransport::MdflowTurns => {
                let meta = &self.flow_sessions[index].0;
                Some(crate::flows::runner::launch_flow(
                    &meta.flow_id,
                    &meta.flow_name,
                    &meta.flow_path,
                    &meta.cwd,
                    crate::flows::model::FlowUxVariant::Flash,
                    crate::flows::model::EngagementMode::Background,
                    vec![("task".to_string(), prompt)],
                    std::time::Instant::now(),
                ))
            }
        };

        let meta = &mut self.flow_sessions[index].0;
        meta.active_turn = Some(crate::flows::session::ActiveTurn {
            run_id,
            message_id,
            assistant_acc: String::new(),
            user_text: text,
        });
        meta.state = crate::flows::session::SessionState::Working;
        self.start_flow_ux_tick(cx);
        cx.notify();
    }

    /// Append streamed assistant text to a session's open turn.
    fn append_flow_turn_text(&mut self, session_id: u64, delta: &str, cx: &mut Context<Self>) {
        let Some(index) = self.flow_session_index(session_id) else {
            return;
        };
        let entity = self.flow_sessions[index].1.clone();
        let Some(active) = self.flow_sessions[index].0.active_turn.as_mut() else {
            return;
        };
        active.assistant_acc.push_str(delta);
        let message_id = active.message_id.clone();
        let delta = delta.to_string();
        entity.update(cx, |chat, cx| {
            chat.append_chunk(&message_id, &delta, cx);
        });
    }

    /// Settle a session's open turn: close the streaming bubble, surface the
    /// outcome, commit the SessionTurn, set state. A user-initiated stop is
    /// NOT an error — it renders as a quiet italic caption, never the red
    /// error treatment.
    fn finish_flow_turn(
        &mut self,
        session_id: u64,
        state: crate::flows::session::SessionState,
        outcome: FlowTurnOutcome,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self.flow_session_index(session_id) else {
            return;
        };
        let Some(active) = self.flow_sessions[index].0.active_turn.take() else {
            return;
        };
        let entity = self.flow_sessions[index].1.clone();
        let message_id = active.message_id.clone();
        let had_error = matches!(outcome, FlowTurnOutcome::Failed(_));
        let stopped_caption = match &outcome {
            FlowTurnOutcome::Stopped if active.assistant_acc.is_empty() => {
                Some("*Stopped.*".to_string())
            }
            FlowTurnOutcome::Stopped => Some("\n\n*Stopped.*".to_string()),
            _ => None,
        };
        entity.update(cx, |chat, cx| {
            // append_chunk is gated on the live stream — captions go in
            // before the stream closes.
            if let Some(caption) = stopped_caption {
                chat.append_chunk(&message_id, &caption, cx);
            }
            chat.complete_streaming(&message_id, cx);
            if let FlowTurnOutcome::Failed(note) = outcome {
                chat.set_message_error(&message_id, note, cx);
            }
        });
        let meta = &mut self.flow_sessions[index].0;
        meta.turns.push(crate::flows::session::SessionTurn {
            user: active.user_text,
            assistant: active.assistant_acc,
        });
        meta.state = state;
        tracing::info!(
            target: "script_kit::flows",
            event = "flow_turn_settled",
            session_id,
            state = state.label(),
            had_error,
            "Flow turn settled"
        );
    }

    /// Apply one codex app-server event to its session.
    fn apply_flow_thread_event(
        &mut self,
        event: crate::flows::codex_client::FlowThreadEvent,
        cx: &mut Context<Self>,
    ) {
        use crate::flows::codex_client::FlowThreadEvent;
        use crate::flows::session::SessionState;
        match event {
            FlowThreadEvent::ThreadStarted { session_id, model } => {
                if let Some(index) = self.flow_session_index(session_id) {
                    if !model.is_empty() {
                        self.flow_sessions[index].0.engine = format!("codex · {model}");
                    }
                }
            }
            FlowThreadEvent::TurnStarted { session_id } => {
                if let Some(index) = self.flow_session_index(session_id) {
                    if self.flow_sessions[index].0.active_turn.is_some() {
                        self.flow_sessions[index].0.state = SessionState::Working;
                    }
                }
            }
            FlowThreadEvent::AgentDelta { session_id, delta } => {
                self.append_flow_turn_text(session_id, &delta, cx);
            }
            FlowThreadEvent::AgentMessageFinal { session_id, text } => {
                // Authoritative full text: append whatever the deltas
                // missed (deltas can lag or be skipped entirely).
                let Some(index) = self.flow_session_index(session_id) else {
                    return;
                };
                let acc = self.flow_sessions[index]
                    .0
                    .active_turn
                    .as_ref()
                    .map(|active| active.assistant_acc.clone())
                    .unwrap_or_default();
                if text.len() > acc.len() && text.starts_with(&acc) {
                    let suffix = text[acc.len()..].to_string();
                    self.append_flow_turn_text(session_id, &suffix, cx);
                } else if acc.is_empty() && !text.is_empty() {
                    self.append_flow_turn_text(session_id, &text, cx);
                }
            }
            FlowThreadEvent::TurnCompleted {
                session_id,
                status,
                error,
            } => {
                let (state, outcome) = match status.as_str() {
                    "completed" => (SessionState::NeedsYou, FlowTurnOutcome::Ok),
                    "interrupted" => (SessionState::NeedsYou, FlowTurnOutcome::Stopped),
                    _ => (
                        SessionState::Done(None),
                        FlowTurnOutcome::Failed(
                            error.unwrap_or_else(|| "Turn failed".to_string()),
                        ),
                    ),
                };
                self.finish_flow_turn(session_id, state, outcome, cx);
            }
            FlowThreadEvent::SessionFailed { session_id, error } => {
                let Some(index) = self.flow_session_index(session_id) else {
                    return;
                };
                if self.flow_sessions[index].0.active_turn.is_some() {
                    self.finish_flow_turn(
                        session_id,
                        crate::flows::session::SessionState::Done(None),
                        FlowTurnOutcome::Failed(error),
                        cx,
                    );
                } else {
                    self.flow_sessions[index].0.state =
                        crate::flows::session::SessionState::Done(None);
                }
            }
        }
    }

    /// Stream mdflow-turn run output into transcripts and settle finished
    /// turns. Returns true when anything changed.
    fn sync_mdflow_turns(&mut self, cx: &mut Context<Self>) -> bool {
        let registry = crate::flows::run_registry::flow_run_registry();
        let mut dirty = false;
        for index in 0..self.flow_sessions.len() {
            let (session_id, run_id, acc_len) = {
                let meta = &self.flow_sessions[index].0;
                let Some(active) = &meta.active_turn else {
                    continue;
                };
                let Some(run_id) = active.run_id else {
                    continue;
                };
                (meta.id, run_id, active.assistant_acc.len())
            };
            let Some(run) = registry.get(run_id) else {
                self.finish_flow_turn(
                    session_id,
                    crate::flows::session::SessionState::Done(None),
                    FlowTurnOutcome::Failed("run disappeared from the registry".to_string()),
                    cx,
                );
                dirty = true;
                continue;
            };
            let full: String = run
                .stdout_tail
                .lines()
                .collect::<Vec<&str>>()
                .join("\n");
            if full.len() > acc_len {
                if let Some(delta) = full.get(acc_len..) {
                    let delta = delta.to_string();
                    self.append_flow_turn_text(session_id, &delta, cx);
                    dirty = true;
                }
            }
            if run.phase.is_terminal() {
                use crate::flows::model::RunPhase;
                use crate::flows::session::SessionState;
                let (state, outcome) = match run.phase {
                    RunPhase::Succeeded => (SessionState::NeedsYou, FlowTurnOutcome::Ok),
                    RunPhase::Cancelled => (SessionState::NeedsYou, FlowTurnOutcome::Stopped),
                    _ => (
                        SessionState::Done(run.exit_code.map(|code| code as i32)),
                        FlowTurnOutcome::Failed(
                            run.error_message
                                .clone()
                                .unwrap_or_else(|| run.display_status()),
                        ),
                    ),
                };
                self.finish_flow_turn(session_id, state, outcome, cx);
                dirty = true;
            }
        }
        dirty
    }

    /// Show an existing session (same ChatPrompt entity — the reattach).
    pub(crate) fn open_flow_session(&mut self, session_id: u64, cx: &mut Context<Self>) {
        let Some(index) = self.flow_session_index(session_id) else {
            return;
        };
        let friendly = self.flow_sessions[index].0.friendly_name.clone();
        self.current_view = AppView::FlowSessionView { session_id };
        // The MAIN input is the composer (with all its context-attachment
        // features) — clear any desk query and retitle the placeholder.
        self.filter_text.clear();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some(format!("Message {friendly}…"));
        self.focused_input = FocusedInput::MainFilter;
        self.pending_focus = Some(FocusTarget::MainFilter);
        cx.spawn(async move |_this, _cx| {
            crate::window_resize::resize_to_view_sync(
                crate::window_resize::ViewType::MainWindow,
                0,
            );
        })
        .detach();
        cx.notify();
    }

    /// Leave the session view without touching the process. The session
    /// stays in `flow_sessions` and reappears under Active.
    pub(crate) fn background_flow_session(&mut self, cx: &mut Context<Self>) {
        let AppView::FlowSessionView { session_id } = self.current_view else {
            return;
        };
        tracing::info!(
            target: "script_kit::flows",
            event = "flow_session_backgrounded",
            session_id,
            "Backgrounding flow session (process stays alive)"
        );
        // Clear the shared filter INPUT too (pending_filter_sync flushes the
        // widget on next render) — the desk must never show a stale query
        // over an unfiltered list — and restore the desk placeholder.
        self.filter_text.clear();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some("Search flows...".to_string());
        self.current_view = AppView::FlowUxView {
            variant: crate::flows::model::FlowUxVariant::Flash,
            filter: String::new(),
            selected_index: 0,
            inline_run: None,
        };
        self.focused_input = FocusedInput::MainFilter;
        cx.notify();
    }

    /// Explicit stop (⌘K verb): cancel the in-flight turn only. The
    /// conversation survives and the composer stays usable.
    pub(crate) fn stop_flow_session(&mut self, session_id: u64, cx: &mut Context<Self>) {
        let Some(index) = self.flow_session_index(session_id) else {
            return;
        };
        let Some(active) = self.flow_sessions[index].0.active_turn.clone() else {
            return;
        };
        match self.flow_sessions[index].0.transport {
            crate::flows::session::SessionTransport::CodexThread => {
                crate::flows::codex_client::codex_app_server().interrupt(session_id);
                // turn/completed {status: interrupted} settles the turn.
            }
            crate::flows::session::SessionTransport::MdflowTurns => {
                if let Some(run_id) = active.run_id {
                    crate::flows::runner::cancel_run(run_id);
                    // The registry's Cancelled phase settles the turn.
                }
            }
        }
        cx.notify();
    }

    /// Remove a session row (⌘K "Dismiss"). Only idle sessions can be
    /// dismissed — stop the in-flight turn first.
    pub(crate) fn dismiss_flow_session(&mut self, session_id: u64, cx: &mut Context<Self>) {
        if let Some(index) = self.flow_session_index(session_id) {
            if self.flow_sessions[index].0.active_turn.is_none() {
                crate::flows::codex_client::codex_app_server().forget_session(session_id);
                self.flow_sessions.remove(index);
                if matches!(
                    self.current_view,
                    AppView::FlowSessionView { session_id: current } if current == session_id
                ) {
                    self.background_flow_session(cx);
                }
                cx.notify();
            }
        }
    }

    /// Run once in the background via the run registry (`--events`).
    pub(crate) fn flow_desk_run_once(
        &mut self,
        flow: &crate::flows::model::FlowDescriptor,
        cx: &mut Context<Self>,
    ) -> u64 {
        let cwd = self.flow_ux_cwd();
        crate::flows::manager_window::remember_flow_cwd(&cwd);
        let run_id = crate::flows::runner::launch_flow(
            &flow.id,
            &flow.name,
            &flow.path,
            &cwd,
            crate::flows::model::FlowUxVariant::Flash,
            crate::flows::model::EngagementMode::Background,
            Vec::new(),
            std::time::Instant::now(),
        );
        self.start_flow_ux_tick(cx);
        self.toast_manager.push(
            crate::components::toast::Toast::success(
                format!("{} running once in background", flow.friendly_name()),
                &self.theme,
            )
            .duration_ms(Some(1800)),
        );
        cx.notify();
        run_id
    }

    /// Start the plain-English creation path. `md create` is a genuinely
    /// interactive CLI wizard, so it runs in the shared Quick Terminal
    /// (honest transport) rather than being faked into a chat surface.
    pub(crate) fn start_flow_create_session(&mut self, cx: &mut Context<Self>) {
        let cwd = self.flow_ux_cwd();
        tracing::info!(
            target: "script_kit::flows",
            event = "flow_create_open",
            "Opening md create in Quick Terminal"
        );
        self.open_quick_terminal_with_command(
            Some(std::path::PathBuf::from(cwd)),
            "md create".to_string(),
            cx,
        );
    }

    // ------------------------------------------------------------------
    // Tab flow router entry (from the main menu input)
    // ------------------------------------------------------------------

    /// Route free text typed in the main menu to a flow (Tab). Confident →
    /// start the conversation; the text rides along as the first message
    /// ONLY when it reads like a request rather than the flow's own name
    /// (a lookup query like "githu" must never become the agent's task).
    /// Otherwise → open the desk with the text as the filter so the user
    /// picks (the Create Flow row is always present for the no-match case).
    pub(crate) fn route_text_to_flow(
        &mut self,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let corpus = self.flow_desk_corpus();
        let decision = crate::flows::router::route(&text, &corpus);
        match decision {
            crate::flows::router::RouteDecision::AutoStart { flow } => {
                let trimmed = text.trim();
                let lowered = trimmed.to_lowercase();
                let is_name_lookup = !trimmed.contains(' ')
                    || flow.name.to_lowercase() == lowered
                    || flow.friendly_name().to_lowercase() == lowered;
                let first_message =
                    (!is_name_lookup).then(|| trimmed.to_string());
                tracing::info!(
                    target: "script_kit::flows",
                    event = "flow_router_auto_start",
                    flow_id = %flow.id,
                    query_len = text.len(),
                    carries_message = first_message.is_some(),
                    "Tab router: confident match, starting conversation"
                );
                self.start_flow_session(&flow, first_message, cx);
            }
            crate::flows::router::RouteDecision::Candidates { .. }
            | crate::flows::router::RouteDecision::NoMatch => {
                tracing::info!(
                    target: "script_kit::flows",
                    event = "flow_router_candidates",
                    query_len = text.len(),
                    "Tab router: opening desk with candidates"
                );
                let cwd = self.flow_ux_cwd();
                crate::flows::catalog::flow_catalog().refresh(&cwd);
                self.open_builtin_filterable_view(
                    AppView::FlowUxView {
                        variant: crate::flows::model::FlowUxVariant::Flash,
                        filter: text.clone(),
                        selected_index: 0,
                        inline_run: None,
                    },
                    "Search flows...",
                    false,
                    cx,
                );
                // Seed the visible input with the routed text so the desk
                // filter and the header input agree (cwd-pick pattern).
                self.suppress_filter_events = true;
                self.gpui_input_state.update(cx, |state, cx| {
                    state.set_value(text.clone(), window, cx);
                    let len = text.len();
                    state.set_selection(len, len, window, cx);
                });
                self.suppress_filter_events = false;
                self.start_flow_ux_tick(cx);
            }
        }
    }

    // ------------------------------------------------------------------
    // Desk render
    // ------------------------------------------------------------------

    fn render_flow_ux(
        &mut self,
        _variant: crate::flows::model::FlowUxVariant,
        filter: String,
        selected_index: usize,
        _inline_run: Option<u64>,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let chrome = crate::theme::AppChromeColors::from_theme(&self.theme);
        let list_colors = crate::list_item::ListItemColors::from_theme(&self.theme);
        let cwd = self.flow_ux_cwd();
        let roster = crate::flows::catalog::flow_catalog().roster_for(&cwd);
        let rows = self.flow_desk_rows(&filter);
        let row_count = rows.len();
        let registry = crate::flows::run_registry::flow_run_registry();

        // ------------------------------------------------------------------
        // Key handler — the Conversation Desk grammar.
        // ------------------------------------------------------------------
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                this.hide_mouse_cursor(cx);
                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;
                let has_shift = event.keystroke.modifiers.shift;

                let view_state = if let AppView::FlowUxView {
                    filter,
                    selected_index,
                    ..
                } = &this.current_view
                {
                    Some((filter.clone(), *selected_index))
                } else {
                    None
                };
                let Some((current_filter, current_selected)) = view_state else {
                    return;
                };

                if is_key_escape(key) && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    cx.stop_propagation();
                    return;
                }

                if has_cmd && key.eq_ignore_ascii_case("w") {
                    this.close_and_reset_window(cx);
                    cx.stop_propagation();
                    return;
                }

                let rows = this.flow_desk_rows(&current_filter);
                let current_len = rows.len();

                if is_key_up(key) {
                    if current_selected > 0 {
                        if let AppView::FlowUxView { selected_index, .. } = &mut this.current_view
                        {
                            *selected_index = current_selected - 1;
                            this.flow_ux_scroll_handle
                                .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                    return;
                }
                if is_key_down(key) {
                    if current_selected < current_len.saturating_sub(1) {
                        if let AppView::FlowUxView { selected_index, .. } = &mut this.current_view
                        {
                            *selected_index = current_selected + 1;
                            this.flow_ux_scroll_handle
                                .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                    return;
                }

                if is_key_enter(key) {
                    this.flow_desk_activate_selected(has_shift, cx);
                    cx.stop_propagation();
                }
            },
        );

        // ------------------------------------------------------------------
        // List element
        // ------------------------------------------------------------------
        let cwd_display = Self::flow_ux_cwd_display(&cwd);
        let empty_message = match roster.status {
            crate::flows::catalog::RosterStatus::Loading => {
                format!("Loading flows in {cwd_display}…")
            }
            crate::flows::catalog::RosterStatus::Legacy => {
                "mdflow is pre-protocol — upgrade with: npm i -g mdflow".to_string()
            }
            crate::flows::catalog::RosterStatus::Error => match roster.warnings.first() {
                Some(warning) => format!("Flow roster unavailable — {warning}"),
                None => format!("Flow roster unavailable in {cwd_display}"),
            },
            crate::flows::catalog::RosterStatus::Ready => String::new(),
        };

        let list_element: gpui::AnyElement = {
            let display_rows = rows.clone();
            let hovered = self.hovered_index;
            let session_meta: Vec<crate::flows::session::FlowSessionMeta> = self
                .flow_sessions
                .iter()
                .map(|(meta, _)| meta.clone())
                .collect();
            uniform_list("flow-desk-list", row_count, move |visible_range, _window, _cx| {
                visible_range
                    .map(|ix| {
                        let is_selected = ix == selected_index;
                        let is_hovered = hovered == Some(ix);
                        let (title, description, icon) = match &display_rows[ix] {
                            FlowDeskRow::Session(session_id) => {
                                let meta =
                                    session_meta.iter().find(|m| m.id == *session_id);
                                match meta {
                                    Some(meta) => (
                                        meta.friendly_name.clone(),
                                        format!(
                                            "{} · {} · {} · conversation",
                                            meta.state.label(),
                                            meta.elapsed_label(),
                                            meta.engine,
                                        ),
                                        if meta.state.is_live() { "💬" } else { "◽" },
                                    ),
                                    None => (
                                        "Session".to_string(),
                                        "ended".to_string(),
                                        "◽",
                                    ),
                                }
                            }
                            FlowDeskRow::Flow(flow) => {
                                let purpose = flow
                                    .description
                                    .clone()
                                    .unwrap_or_else(|| flow.name.clone());
                                (
                                    flow.friendly_name(),
                                    format!(
                                        "{purpose} · {} · {}",
                                        flow.engine,
                                        flow.origin_label()
                                    ),
                                    if flow.is_workflow { "🧩" } else { "⚡" },
                                )
                            }
                            FlowDeskRow::CreateFlow => (
                                "Create a flow…".to_string(),
                                "Describe an agent in plain English (md create)"
                                    .to_string(),
                                "✚",
                            ),
                        };
                        div().id(ix).cursor_pointer().child(
                            ListItem::new(title, list_colors)
                                .description_opt(Some(description))
                                .icon(icon)
                                .selected(is_selected)
                                .hovered(is_hovered)
                                .with_accent_bar(true),
                        )
                    })
                    .collect()
            })
            .h_full()
            .track_scroll(&self.flow_ux_scroll_handle)
            .into_any_element()
        };

        let list_scrollbar =
            self.builtin_uniform_list_scrollbar(&self.flow_ux_scroll_handle, row_count, 8);
        let mut list_pane = div()
            .relative()
            .w_full()
            .h_full()
            .min_h(px(0.))
            .child(list_element)
            .child(list_scrollbar);
        if !empty_message.is_empty() && roster.flows.is_empty() {
            // Roster problems surface as a banner above the (package) rows
            // instead of replacing the whole list — package flows still work
            // when a repo has none of its own.
            list_pane = div()
                .relative()
                .w_full()
                .h_full()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .child(
                    div()
                        .px_3()
                        .py_1()
                        .text_xs()
                        .text_color(rgb(chrome.text_muted_hex))
                        .child(empty_message),
                )
                .child(div().flex_1().min_h(px(0.)).child(list_pane));
        }
        let main = list_pane.into_any_element();

        // ------------------------------------------------------------------
        // Footer + shell (Conversation Desk contract: primary verbs only).
        // ------------------------------------------------------------------
        let live_sessions = self
            .flow_sessions
            .iter()
            .filter(|(meta, _)| meta.state.is_live())
            .count();
        let active_runs = registry.active_count();
        let hints: Vec<gpui::SharedString> = vec![
            gpui::SharedString::from("↵ Converse"),
            gpui::SharedString::from("⇧↵ Run once"),
            gpui::SharedString::from("⌘K Actions"),
            gpui::SharedString::from("Esc Back"),
        ];
        let footer = self.main_window_footer_slot(crate::components::render_simple_hint_strip(
            hints, None,
        ));

        let mut count_parts: Vec<String> = Vec::new();
        if live_sessions > 0 {
            count_parts.push(format!("{live_sessions} active"));
        }
        if active_runs > 0 {
            count_parts.push(format!("{active_runs} running"));
        }
        count_parts.push(format!("{} flows", row_count.saturating_sub(1)));
        let count_label = count_parts.join(" · ");
        let cwd_chip = div()
            .flex_none()
            .whitespace_nowrap()
            .text_sm()
            .text_color(rgb(chrome.text_secondary_hex))
            .child(cwd_display.clone())
            .into_any_element();
        let trailing = vec![
            cwd_chip,
            self.render_builtin_main_input_count_label(count_label),
        ];

        let menu_def = self.current_main_menu_theme.def();
        let shell = menu_def.shell;
        crate::components::main_view_chrome::render_main_view_chrome_footer_flush(
            crate::components::main_view_chrome::render_main_view_shell()
                .text_color(rgb(chrome.text_primary_hex))
                .font_family(self.theme_font_family())
                .key_context("flow_ux")
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key),
            &self.theme,
            menu_def,
            crate::components::main_view_chrome::MainViewChrome {
                header: self.render_builtin_main_input_header(trailing, cx),
                divider: crate::components::main_view_chrome::MainViewDividerChrome {
                    margin_x: shell.divider_margin_x,
                    height: shell.divider_height,
                    visible: shell.divider_height > 0.0,
                },
                main,
                footer,
                overlays: Vec::new(),
            },
        )
    }

    // ------------------------------------------------------------------
    // Session render (FlowSessionView)
    // ------------------------------------------------------------------

    pub(crate) fn render_flow_session(
        &mut self,
        session_id: u64,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let chrome = crate::theme::AppChromeColors::from_theme(&self.theme);
        let Some(index) = self.flow_session_index(session_id) else {
            // Session vanished (dismissed elsewhere) — fall back to the desk.
            return self.render_flow_ux(
                crate::flows::model::FlowUxVariant::Flash,
                String::new(),
                0,
                None,
                cx,
            );
        };
        let (meta, entity) = {
            let (meta, entity) = &self.flow_sessions[index];
            (meta.clone(), entity.clone())
        };

        // The MAIN input is the composer — the same shared input every
        // surface uses (with its context-attachment features). Identity and
        // honest state ride as trailing chips on that one input row;
        // ChatPrompt renders transcript only.
        let state_color = if meta.state.is_live() {
            rgb(chrome.accent_hex)
        } else {
            rgb(chrome.text_muted_hex)
        };
        let identity_chip = div()
            .flex_none()
            .whitespace_nowrap()
            .text_sm()
            .text_color(rgb(chrome.text_secondary_hex))
            .child(format!(
                "{} · {} · {}",
                meta.friendly_name, meta.engine, meta.origin
            ))
            .into_any_element();
        let state_chip = div()
            .flex_none()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .pr(px(self.current_main_menu_theme.def().search.text_inset_x))
            .child(div().w(px(6.)).h(px(6.)).rounded_full().bg(state_color))
            .child(
                div()
                    .text_sm()
                    .whitespace_nowrap()
                    .text_color(state_color)
                    .child(format!("{} · {}", meta.state.label(), meta.elapsed_label())),
            )
            .into_any_element();
        let trailing = vec![identity_chip, state_chip];

        // Enter = send the main-input draft as the next turn; Esc =
        // background to the desk; ⌘K = session actions. Same shell-level
        // key routing the desk uses while the main input is focused.
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                let AppView::FlowSessionView { session_id } = this.current_view else {
                    return;
                };
                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;
                if is_key_escape(key) && !this.show_actions_popup {
                    this.background_flow_session(cx);
                    cx.stop_propagation();
                    return;
                }
                if has_cmd && key.eq_ignore_ascii_case("k") {
                    this.dispatch_actions_toggle_for_current_view(window, cx, "flow_session_chat");
                    cx.stop_propagation();
                    return;
                }
                if is_key_enter(key) {
                    let text = this.filter_text.trim().to_string();
                    if !text.is_empty() {
                        this.set_filter_text_immediate(String::new(), window, cx);
                        this.submit_flow_chat_message(session_id, text, cx);
                    }
                    cx.stop_propagation();
                }
            },
        );

        let hints: Vec<gpui::SharedString> = vec![
            gpui::SharedString::from("↵ Send"),
            gpui::SharedString::from("Esc Desk"),
            gpui::SharedString::from("⌘K Actions"),
        ];
        let footer = self.main_window_footer_slot(crate::components::render_simple_hint_strip(
            hints, None,
        ));

        let menu_def = self.current_main_menu_theme.def();
        let shell = menu_def.shell;
        crate::components::main_view_chrome::render_main_view_chrome_footer_flush(
            crate::components::main_view_chrome::render_main_view_shell()
                .text_color(rgb(chrome.text_primary_hex))
                .font_family(self.theme_font_family())
                .key_context("flow_session")
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key),
            &self.theme,
            menu_def,
            crate::components::main_view_chrome::MainViewChrome {
                header: self.render_builtin_main_input_header(trailing, cx),
                divider: crate::components::main_view_chrome::MainViewDividerChrome {
                    margin_x: shell.divider_margin_x,
                    height: shell.divider_height,
                    visible: shell.divider_height > 0.0,
                },
                main: div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.))
                    .w_full()
                    .child(entity)
                    .into_any_element(),
                footer,
                overlays: Vec::new(),
            },
        )
    }

    /// `flowUx` automation snapshot for getState (protocol §6).
    pub(crate) fn flow_ux_automation_snapshot(&self) -> serde_json::Value {
        let (desk_active, selected_flow_id) = match &self.current_view {
            AppView::FlowUxView {
                filter,
                selected_index,
                ..
            } => {
                let rows = self.flow_desk_rows(filter);
                let selected = rows.get(*selected_index).and_then(|row| match row {
                    FlowDeskRow::Flow(flow) => Some(flow.id.clone()),
                    FlowDeskRow::Session(id) => self
                        .flow_sessions
                        .iter()
                        .find(|(meta, _)| meta.id == *id)
                        .map(|(meta, _)| meta.flow_id.clone()),
                    FlowDeskRow::CreateFlow => Some("builtin:create-flow".to_string()),
                });
                (true, selected)
            }
            AppView::FlowSessionView { session_id } => (
                false,
                self.flow_sessions
                    .iter()
                    .find(|(meta, _)| meta.id == *session_id)
                    .map(|(meta, _)| meta.flow_id.clone()),
            ),
            _ => (false, None),
        };
        let cwd = self.flow_ux_cwd();
        let roster_entry = crate::flows::catalog::flow_catalog().roster_for(&cwd);
        let sessions: Vec<crate::flows::automation::SessionSnapshot> = self
            .flow_sessions
            .iter()
            .map(|(meta, _)| crate::flows::automation::SessionSnapshot {
                id: meta.id,
                flow_id: meta.flow_id.clone(),
                flow_name: meta.flow_name.clone(),
                state: meta.state.label(),
                live: meta.state.is_live(),
                elapsed_ms: meta.started_at.elapsed().as_millis() as u64,
                turns: meta.turns.len(),
                turn_in_flight: meta.active_turn.is_some(),
                transport: match meta.transport {
                    crate::flows::session::SessionTransport::CodexThread => "codexThread",
                    crate::flows::session::SessionTransport::MdflowTurns => "mdflowTurns",
                },
            })
            .collect();
        crate::flows::automation::flow_ux_state(crate::flows::automation::FlowUxSnapshotInputs {
            active_variant: desk_active.then_some(crate::flows::model::FlowUxVariant::Flash),
            selected_flow_id: selected_flow_id.as_deref(),
            roster: Some((&roster_entry, cwd.as_str())),
            preview: None,
            manager_visible: false,
            manager_focused_run_id: None,
            sessions,
        })
    }
}
