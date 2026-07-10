// Flow Desk (Conversation Desk, fusion-ultra 2026-07-09).
//
// Every flow is an agent identity. One main-window surface over the shared
// flow substrate (`crate::flows`):
//
// - Enter on a flow = CONVERSE: launch the real `flow-*` wrapper (or
//   `md <path> --_interactive`) in an embedded PTY session in this window.
// - Enter on an Active session row = reattach the SAME PTY entity.
// - ⇧↵ = run once in the background via `md <flow> --events` (registry).
// - ⌘⇧D (in a session) = background: leave the process alive, return here.
// - Esc in a session goes to the TUI (codex/claude own Escape); Esc in the
//   desk clears the filter / goes back. Stop is an explicit ⌘K verb only.
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

    /// Spawn the repaint tick that keeps flow surfaces live while runs
    /// stream or sessions are open. Single instance; exits when nothing is
    /// active. Also the seam that notices a session's PTY exited (the
    /// process is polled here — never scraped).
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

                        // Poll session liveness: a PTY that exited flips its
                        // meta to Done exactly once.
                        let mut any_live_session = false;
                        for index in 0..app.flow_sessions.len() {
                            let entity = app.flow_sessions[index].1.clone();
                            let running =
                                entity.update(cx, |term, _cx| term.terminal.is_running());
                            let meta = &mut app.flow_sessions[index].0;
                            if meta.state.is_live() && !running {
                                meta.state =
                                    crate::flows::session::SessionState::Done(None);
                                dirty = true;
                            }
                            any_live_session |= meta.state.is_live();
                        }

                        if dirty {
                            cx.notify();
                        }
                        let view_active = matches!(
                            app.current_view,
                            AppView::FlowUxView { .. } | AppView::FlowSessionView { .. }
                        );
                        let keep =
                            view_active || registry.active_count() > 0 || any_live_session;
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

    // ------------------------------------------------------------------
    // Conversation lifecycle
    // ------------------------------------------------------------------

    /// Start a conversation with a flow: spawn its wrapper interactively in
    /// a new PTY session and show it. `initial_task` (Tab router / typed
    /// text) rides along as the engine's first prompt.
    pub(crate) fn start_flow_session(
        &mut self,
        flow: &crate::flows::model::FlowDescriptor,
        initial_task: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let cwd = self.flow_ux_cwd();
        crate::flows::manager_window::remember_flow_cwd(&cwd);
        self.flow_session_counter += 1;
        let session_id = self.flow_session_counter;
        let command = crate::flows::session::build_conversation_command(
            session_id,
            &cwd,
            flow.wrapper_command.as_deref(),
            &flow.path,
            initial_task.as_deref(),
        );

        // Log length only — task text can carry anything.
        tracing::info!(
            target: "script_kit::flows",
            event = "flow_session_start",
            session_id,
            flow_id = %flow.id,
            wrapper = ?flow.wrapper_command,
            task_len = initial_task.as_deref().map(str::len).unwrap_or(0),
            "Starting flow conversation"
        );

        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(move |_id: String, _value: Option<String>| {
                // Exit is observed by the flow tick (PTY liveness poll).
            });
        let term_height = crate::window_resize::layout::MAX_HEIGHT
            - px(crate::window_resize::layout::FOOTER_HEIGHT);
        match crate::term_prompt::TermPrompt::with_height(
            format!("flow-session-{session_id}"),
            Some(command.clone()),
            self.focus_handle.clone(),
            submit_callback,
            std::sync::Arc::clone(&self.theme),
            std::sync::Arc::new(self.config.clone()),
            Some(term_height),
        ) {
            Ok(mut term) => {
                // Codex/Claude TUIs own Escape (interrupt); backgrounding is
                // ⌘⇧D. Never let Escape cancel the session.
                term.escape_cancels = false;
                let entity = cx.new(|_| term);
                let meta = crate::flows::session::FlowSessionMeta {
                    id: session_id,
                    flow_id: flow.id.clone(),
                    flow_name: flow.name.clone(),
                    friendly_name: flow.friendly_name(),
                    origin: flow.origin_label().to_string(),
                    engine: flow.engine.clone(),
                    command,
                    initial_task,
                    state: crate::flows::session::SessionState::Working,
                    started_at: std::time::Instant::now(),
                };
                self.flow_sessions.push((meta, entity));
                self.open_flow_session(session_id, cx);
                self.start_flow_ux_tick(cx);
            }
            Err(err) => {
                self.toast_manager.push(
                    crate::components::toast::Toast::error(
                        format!("Couldn't start {}: {err}", flow.friendly_name()),
                        &self.theme,
                    )
                    .duration_ms(Some(4000)),
                );
                cx.notify();
            }
        }
    }

    /// Show an existing session (same PTY entity — this is the reattach).
    pub(crate) fn open_flow_session(&mut self, session_id: u64, cx: &mut Context<Self>) {
        if self.flow_session_index(session_id).is_none() {
            return;
        }
        self.current_view = AppView::FlowSessionView { session_id };
        self.focused_input = FocusedInput::None;
        self.pending_focus = Some(FocusTarget::TermPrompt);
        cx.spawn(async move |_this, _cx| {
            crate::window_resize::resize_to_view_sync(crate::window_resize::ViewType::TermPrompt, 0);
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
        self.current_view = AppView::FlowUxView {
            variant: crate::flows::model::FlowUxVariant::Flash,
            filter: String::new(),
            selected_index: 0,
            inline_run: None,
        };
        self.focused_input = FocusedInput::MainFilter;
        cx.notify();
    }

    /// Explicit stop (⌘K verb): kill the PTY process group and mark Done.
    /// The row stays visible with its final state until dismissed.
    pub(crate) fn stop_flow_session(&mut self, session_id: u64, cx: &mut Context<Self>) {
        let Some(index) = self.flow_session_index(session_id) else {
            return;
        };
        let entity = self.flow_sessions[index].1.clone();
        entity.update(cx, |term, _cx| {
            let _ = term.terminal.kill();
        });
        self.flow_sessions[index].0.state = crate::flows::session::SessionState::Done(None);
        cx.notify();
    }

    /// Remove an ended session row (⌘K "Dismiss").
    pub(crate) fn dismiss_flow_session(&mut self, session_id: u64, cx: &mut Context<Self>) {
        if let Some(index) = self.flow_session_index(session_id) {
            if !self.flow_sessions[index].0.state.is_live() {
                self.flow_sessions.remove(index);
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

    /// Start the plain-English creation path in a conversation session.
    pub(crate) fn start_flow_create_session(&mut self, cx: &mut Context<Self>) {
        let create = crate::flows::model::FlowDescriptor {
            id: "builtin:create-flow".to_string(),
            path: String::new(),
            source: crate::flows::model::FlowSource::Global,
            name: "flow-create".to_string(),
            description: Some("Create a new Markdown flow".to_string()),
            engine: "md".to_string(),
            engine_source: None,
            inputs: Vec::new(),
            is_workflow: false,
            interactive: true,
            mtime_ms: 0,
            origin: Some("mdflow".to_string()),
            wrapper_command: None,
        };
        // `md create` is itself interactive; reuse the session plumbing with
        // a bespoke command (no --_interactive suffix needed).
        let cwd = self.flow_ux_cwd();
        self.flow_session_counter += 1;
        let session_id = self.flow_session_counter;
        let command = format!(
            "cd {} && SCRIPT_KIT_FLOW_SESSION_ID={session_id} md create",
            crate::flows::session::shell_quote(&cwd)
        );
        let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
            std::sync::Arc::new(move |_id, _value| {});
        let term_height = crate::window_resize::layout::MAX_HEIGHT
            - px(crate::window_resize::layout::FOOTER_HEIGHT);
        match crate::term_prompt::TermPrompt::with_height(
            format!("flow-session-{session_id}"),
            Some(command.clone()),
            self.focus_handle.clone(),
            submit_callback,
            std::sync::Arc::clone(&self.theme),
            std::sync::Arc::new(self.config.clone()),
            Some(term_height),
        ) {
            Ok(mut term) => {
                term.escape_cancels = false;
                let entity = cx.new(|_| term);
                let meta = crate::flows::session::FlowSessionMeta {
                    id: session_id,
                    flow_id: create.id.clone(),
                    flow_name: create.name.clone(),
                    friendly_name: "Create Flow".to_string(),
                    origin: "mdflow".to_string(),
                    engine: create.engine.clone(),
                    command,
                    initial_task: None,
                    state: crate::flows::session::SessionState::Working,
                    started_at: std::time::Instant::now(),
                };
                self.flow_sessions.push((meta, entity));
                self.open_flow_session(session_id, cx);
                self.start_flow_ux_tick(cx);
            }
            Err(err) => {
                self.toast_manager.push(
                    crate::components::toast::Toast::error(
                        format!("Couldn't start flow creation: {err}"),
                        &self.theme,
                    )
                    .duration_ms(Some(4000)),
                );
                cx.notify();
            }
        }
    }

    // ------------------------------------------------------------------
    // Tab flow router entry (from the main menu input)
    // ------------------------------------------------------------------

    /// Route free text typed in the main menu to a flow (Tab). Confident →
    /// start the conversation with the text as the first task. Otherwise →
    /// open the desk with the text as the filter so the user picks (the
    /// Create Flow row is always present for the no-match case).
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
                tracing::info!(
                    target: "script_kit::flows",
                    event = "flow_router_auto_start",
                    flow_id = %flow.id,
                    query_len = text.len(),
                    "Tab router: confident match, starting conversation"
                );
                self.start_flow_session(&flow, Some(text), cx);
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
                    let Some(row) = rows.get(current_selected).cloned() else {
                        cx.stop_propagation();
                        return;
                    };
                    match row {
                        FlowDeskRow::Session(session_id) => {
                            this.open_flow_session(session_id, cx);
                        }
                        FlowDeskRow::Flow(flow) => {
                            if has_shift || flow.is_workflow {
                                // Workflows (DAGs) are run-once by nature;
                                // ⇧↵ is explicit run-once for anything.
                                this.flow_desk_run_once(&flow, cx);
                            } else {
                                let task = if current_filter.trim().is_empty() {
                                    None
                                } else {
                                    // Typed text that found this flow rides
                                    // along as the first prompt only when it
                                    // is more than the flow's own name.
                                    let trimmed = current_filter.trim().to_lowercase();
                                    let is_just_name = flow.name.to_lowercase() == trimmed
                                        || flow.friendly_name().to_lowercase() == trimmed;
                                    (!is_just_name).then(|| current_filter.trim().to_string())
                                };
                                this.start_flow_session(&flow, task, cx);
                            }
                        }
                        FlowDeskRow::CreateFlow => {
                            this.start_flow_create_session(cx);
                        }
                    }
                    cx.notify();
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

        let header = div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(chrome.text_primary_hex))
                    .child(format!("💬 {}", meta.friendly_name)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(chrome.text_secondary_hex))
                    .child(format!("{} · {}", meta.engine, meta.origin)),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_xs()
                    .text_color(if meta.state.is_live() {
                        rgb(chrome.accent_hex)
                    } else {
                        rgb(chrome.text_muted_hex)
                    })
                    .child(format!("{} · {}", meta.state.label(), meta.elapsed_label())),
            );

        let hints: Vec<gpui::SharedString> = vec![
            gpui::SharedString::from("⌘⇧D Background"),
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
                .key_context("flow_session"),
            &self.theme,
            menu_def,
            crate::components::main_view_chrome::MainViewChrome {
                header: crate::components::main_view_chrome::MainViewHeaderChrome {
                    context: None,
                    input: header.into_any_element(),
                    padding_x: shell.header_padding_x,
                    padding_y: shell.header_padding_y,
                    gap: shell.header_gap,
                },
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
