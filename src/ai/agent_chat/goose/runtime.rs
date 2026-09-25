//! goose ACP sidecar runtime behind [`AgentChatConnection`].
//!
//! One runtime owns one `goose acp` child process and one goose session. The
//! worker thread runs a single-threaded tokio runtime: a command loop that
//! performs the ACP handshake lazily and sends prompts, a stdout reader that
//! routes responses, `session/update` notifications, and agent-to-client
//! requests, and a writer task that serializes every outbound line.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context as _, Result};
use parking_lot::Mutex;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, oneshot};

use crate::ai::agent_chat::events::{AgentChatEvent, AgentChatEventRx};
use crate::ai::agent_chat::reveal::REVEAL_CHUNK_DELAY_MS;
use crate::ai::agent_chat::runtime::{
    AgentChatConnection, AgentChatTurnRequest, IsolatedTurnHandle,
};
use crate::ai::agent_chat::ui::capabilities::AgentChatToolPolicy;
use crate::ai::agent_chat::ui::events::AgentChatEventTx;
use crate::ai::phase_trace::{AiTransport, PhaseTrace, TurnOutcome};
use sk_protocol::ai_reliability::ProtocolComponent;

use super::events::{
    failure_for_acp_error, map_session_update, models_available, prompt_outcome,
    provider_error_in_update, strip_citation_markers,
};
use super::protocol::{
    cancel_params, error_response, initialize_params, new_session_params, notification, parse_line,
    prompt_params, request, response, set_config_option_params, split_model_selection, AcpError,
    AcpLine, GooseLaunchSpec,
};
use super::recipe::recipe_deeplink;
use super::sign_in::{self, LeaderLease, SignInRole};

const HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const CONFIG_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);
const SEND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

enum GooseCommand {
    PrepareSession {
        event_tx: AgentChatEventTx,
    },
    StartTurn {
        request: AgentChatTurnRequest,
        event_tx: AgentChatEventTx,
    },
    CancelTurn {
        ui_thread_id: String,
    },
}

pub(crate) struct GooseRuntime {
    tx: async_channel::Sender<GooseCommand>,
    spec: Arc<GooseLaunchSpec>,
}

impl GooseRuntime {
    pub(crate) fn spawn(spec: GooseLaunchSpec) -> Result<Self> {
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Provider)?;
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Process)?;
        let spec = Arc::new(spec);
        let worker_spec = spec.clone();
        let (tx, rx) = async_channel::bounded::<GooseCommand>(8);
        std::thread::Builder::new()
            .name("goose-acp-agent-chat".to_string())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        tracing::error!(%error, "goose_acp_runtime_build_failed");
                        return;
                    }
                };
                runtime.block_on(async move {
                    if let Err(error) = run_event_loop(worker_spec, rx).await {
                        tracing::error!(%error, "goose_acp_event_loop_exited_with_error");
                    }
                });
            })
            .context("Failed to spawn goose ACP worker thread")?;
        Ok(Self { tx, spec })
    }

    fn send(&self, command: GooseCommand) -> crate::ai::reliability::AiAdapterResult<()> {
        self.tx
            .send_blocking(command)
            .context("goose ACP worker channel closed")
            .map_err(crate::ai::reliability::AiAdapterError::from)
    }
}

impl AgentChatConnection for GooseRuntime {
    fn start_turn(
        &self,
        request: AgentChatTurnRequest,
    ) -> crate::ai::reliability::AiAdapterResult<AgentChatEventRx> {
        let (event_tx, event_rx) = async_channel::bounded(256);
        self.send(GooseCommand::StartTurn { request, event_tx })?;
        Ok(event_rx)
    }

    /// Focused-text variations run concurrently, so each gets its own goose
    /// process and session instead of queueing on this connection.
    fn start_isolated_turn(
        &self,
        request: AgentChatTurnRequest,
    ) -> crate::ai::reliability::AiAdapterResult<IsolatedTurnHandle> {
        let mut spec = (*self.spec).clone();
        spec.surface = crate::ai::phase_trace::AiSurface::Mini;
        spec.owns_agent_policy = false;
        let runtime =
            GooseRuntime::spawn(spec).map_err(crate::ai::reliability::AiAdapterError::from)?;
        let ui_thread_id = request.ui_thread_id.clone();
        let inner_rx = runtime.start_turn(request)?;
        let (event_tx, event_rx) = async_channel::bounded(256);
        let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cancel_flag = cancel.clone();
        std::thread::Builder::new()
            .name("goose-acp-isolated-turn".to_string())
            .spawn(move || {
                let mut cancel_sent = false;
                loop {
                    if !cancel_sent && cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        cancel_sent = true;
                        let _ = runtime.cancel_turn(ui_thread_id.clone());
                    }
                    match inner_rx.recv_blocking_timeout(std::time::Duration::from_millis(100)) {
                        RecvStep::Event(event) => {
                            let terminal = matches!(
                                event,
                                AgentChatEvent::TurnCompleted { .. }
                                    | AgentChatEvent::TurnFailed { .. }
                            );
                            if event_tx.send_blocking(event).is_err() || terminal {
                                break;
                            }
                        }
                        RecvStep::Idle => {}
                        RecvStep::Closed => break,
                    }
                }
                // Dropping the runtime closes its command channel, which ends
                // the worker loop and kills the isolated goose process.
                drop(runtime);
            })
            .context("Failed to spawn goose isolated turn relay")
            .map_err(crate::ai::reliability::AiAdapterError::from)?;
        Ok(IsolatedTurnHandle {
            rx: event_rx,
            cancel: Some(cancel),
        })
    }

    fn cancel_turn(&self, ui_thread_id: String) -> crate::ai::reliability::AiAdapterResult<()> {
        self.send(GooseCommand::CancelTurn { ui_thread_id })
    }

    fn prepare_session(
        &self,
        _ui_thread_id: String,
        _cwd: std::path::PathBuf,
    ) -> crate::ai::reliability::AiAdapterResult<AgentChatEventRx> {
        let (event_tx, event_rx) = async_channel::bounded(4);
        self.send(GooseCommand::PrepareSession { event_tx })?;
        Ok(event_rx)
    }
}

impl Drop for GooseRuntime {
    fn drop(&mut self) {
        if let Some(policy_id) = self
            .spec
            .agent_policy_id
            .as_deref()
            .filter(|_| self.spec.owns_agent_policy)
        {
            crate::mcp_agent_tools::unregister_policy(policy_id);
        }
    }
}

enum RecvStep {
    Event(AgentChatEvent),
    Idle,
    Closed,
}

trait RecvBlockingTimeout {
    fn recv_blocking_timeout(&self, timeout: std::time::Duration) -> RecvStep;
}

impl RecvBlockingTimeout for AgentChatEventRx {
    fn recv_blocking_timeout(&self, timeout: std::time::Duration) -> RecvStep {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            match self.try_recv() {
                Ok(event) => return RecvStep::Event(event),
                Err(async_channel::TryRecvError::Closed) => return RecvStep::Closed,
                Err(async_channel::TryRecvError::Empty) => {
                    if std::time::Instant::now() >= deadline {
                        return RecvStep::Idle;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    }
}

#[derive(Clone)]
struct ActiveTurn {
    ui_thread_id: String,
    prompt_request_id: u64,
    event_tx: AgentChatEventTx,
    trace: PhaseTrace,
    tool_policy: AgentChatToolPolicy,
    /// A provider failure goose reported as assistant text; the turn ends as
    /// this failure even though goose then stops with `end_turn`.
    provider_failure: Option<crate::ai::reliability::AppFailureRecord>,
}

#[derive(Default)]
struct Shared {
    pending: HashMap<u64, oneshot::Sender<Result<Value, AcpError>>>,
    active_turn: Option<ActiveTurn>,
    session_id: Option<String>,
    /// A credential hint goose printed on stderr, kept so an exit it explains
    /// classifies as a sign-in problem rather than a dropped connection.
    stderr_auth_hint: Option<String>,
}

type SharedState = Arc<Mutex<Shared>>;

/// A classified session failure, boxed because the record is large.
type SessionFailure = Box<crate::ai::reliability::AppFailureRecord>;

/// Why an ACP call did not produce a result.
enum CallError {
    Acp(AcpError),
    Closed,
    Timeout,
}

impl CallError {
    fn failure(&self, method: &str) -> crate::ai::reliability::AppFailureRecord {
        match self {
            CallError::Acp(error) => failure_for_acp_error(error),
            CallError::Closed => crate::ai::reliability::runtime_closed_failure(
                ProtocolComponent::Goose,
                &format!("goose exited before answering {method}"),
            ),
            CallError::Timeout => crate::ai::reliability::provider_failure(
                ProtocolComponent::Goose,
                format!("goose did not answer {method} in time"),
            ),
        }
    }
}

struct Connection {
    shared: SharedState,
    out_tx: mpsc::UnboundedSender<Value>,
    next_id: u64,
}

impl Connection {
    fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    async fn call(
        &mut self,
        method: &str,
        params: Value,
        timeout: std::time::Duration,
    ) -> std::result::Result<Value, CallError> {
        let id = self.next_id();
        let (tx, rx) = oneshot::channel();
        self.shared.lock().pending.insert(id, tx);
        if self.out_tx.send(request(id, method, params)).is_err() {
            self.shared.lock().pending.remove(&id);
            return Err(CallError::Closed);
        }
        let outcome = tokio::time::timeout(timeout, rx).await;
        self.shared.lock().pending.remove(&id);
        match outcome {
            Ok(Ok(Ok(result))) => Ok(result),
            Ok(Ok(Err(error))) => Err(CallError::Acp(error)),
            Ok(Err(_)) => Err(CallError::Closed),
            Err(_) => Err(CallError::Timeout),
        }
    }
}

/// Session facts learned from the handshake.
struct SessionState {
    session_id: String,
    config_options: Value,
}

impl SessionState {
    fn current_value(&self, config_id: &str) -> Option<&str> {
        self.config_options
            .as_array()?
            .iter()
            .find(|option| option.get("id").and_then(Value::as_str) == Some(config_id))?
            .get("currentValue")?
            .as_str()
    }

    fn has_value(&self, config_id: &str, value: &str) -> bool {
        self.config_options
            .as_array()
            .into_iter()
            .flatten()
            .filter(|option| option.get("id").and_then(Value::as_str) == Some(config_id))
            .flat_map(super::events::select_values)
            .any(|(candidate, _)| candidate == value)
    }
}

async fn run_event_loop(
    spec: Arc<GooseLaunchSpec>,
    rx: async_channel::Receiver<GooseCommand>,
) -> Result<()> {
    let mut cmd = Command::new(&spec.command);
    cmd.args(&spec.args)
        .envs(&spec.env)
        .current_dir(&spec.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Own process group so orphan cleanup can reap goose and any tool
        // processes it spawned; kill_on_drop covers early returns.
        .process_group(0)
        .kill_on_drop(true);

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(error) => {
            let failure = crate::ai::reliability::spawn_failure(
                ProtocolComponent::Goose,
                &format!("failed to spawn goose: {error}"),
            );
            fail_commands_until_closed(rx, failure).await;
            return Ok(());
        }
    };
    let _registration = child.id().map(|pid| {
        crate::process_manager::ChildRegistration::register(pid, &spec.command.to_string_lossy())
    });
    let mut stdin = child.stdin.take().context("goose stdin unavailable")?;
    let stdout = child.stdout.take().context("goose stdout unavailable")?;
    let stderr = child.stderr.take().context("goose stderr unavailable")?;

    let shared: SharedState = Arc::new(Mutex::new(Shared::default()));
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Value>();

    tokio::spawn(async move {
        while let Some(message) = out_rx.recv().await {
            let mut line = message.to_string();
            line.push('\n');
            if stdin.write_all(line.as_bytes()).await.is_err() || stdin.flush().await.is_err() {
                break;
            }
        }
    });

    tokio::spawn(read_stdout(stdout, shared.clone(), out_tx.clone()));

    let stderr_shared = shared.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(hint) = stderr_auth_hint(&line) {
                stderr_shared.lock().stderr_auth_hint = Some(hint);
            }
            tracing::debug!(
                target: "script_kit::agent_chat",
                event = "goose_acp_stderr",
                line_chars = line.chars().count(),
                "goose stderr line suppressed"
            );
        }
    });

    let mut connection = Connection {
        shared: shared.clone(),
        out_tx,
        next_id: 0,
    };
    let mut session: Option<SessionState> = None;
    let mut counter = 0_u64;

    while let Ok(command) = rx.recv().await {
        counter += 1;
        match command {
            GooseCommand::PrepareSession { event_tx } => {
                let event = match ensure_session(&mut connection, &spec, &mut session).await {
                    Ok(state) => models_available(&state.config_options),
                    Err(failure) => AgentChatEvent::TurnFailed { failure: *failure },
                };
                let _ = tokio::time::timeout(SEND_TIMEOUT, event_tx.send(event)).await;
            }
            GooseCommand::StartTurn { request, event_tx } => {
                let trace = PhaseTrace::begin(
                    spec.surface,
                    AiTransport::GooseAcp,
                    format!("goose-{counter}"),
                );
                trace.turn_start(json!({ "blockCount": request.blocks.len() }));

                let state = match ensure_session(&mut connection, &spec, &mut session).await {
                    Ok(state) => state,
                    Err(failure) => {
                        send_traced(
                            &event_tx,
                            &trace,
                            AgentChatEvent::TurnFailed { failure: *failure },
                        )
                        .await;
                        continue;
                    }
                };
                if let Some(model_id) = request.model_id.as_deref() {
                    if let Err(failure) =
                        apply_model_selection(&mut connection, state, model_id).await
                    {
                        send_traced(
                            &event_tx,
                            &trace,
                            AgentChatEvent::TurnFailed { failure: *failure },
                        )
                        .await;
                        continue;
                    }
                }
                // A subscription provider with no stored sign-in: goose runs
                // its browser login inside the next request. The person who
                // started the turn is asked first; one login runs at a time
                // across every goose process; background work never leads one.
                let mut sign_in_leader = None;
                if let Some(provider) = state.current_value("provider").map(str::to_string) {
                    let root = super::launch_spec::goose_root_from_env(&spec.env);
                    match gate_browser_sign_in(
                        root,
                        provider,
                        request.allow_interactive_sign_in,
                        &event_tx,
                        &trace,
                    )
                    .await
                    {
                        SignInGate::Proceed(leader) => sign_in_leader = leader,
                        SignInGate::Stopped => continue,
                    }
                }
                let params = match prompt_params(&state.session_id, &request.blocks) {
                    Ok(params) => params,
                    Err(error) => {
                        send_traced(
                            &event_tx,
                            &trace,
                            AgentChatEvent::failed(ProtocolComponent::Goose, error.to_string()),
                        )
                        .await;
                        continue;
                    }
                };
                if let Some(policy_id) = spec.agent_policy_id.as_deref() {
                    crate::mcp_agent_tools::begin_turn(policy_id);
                }
                let prompt_request_id = connection.next_id();
                shared.lock().active_turn = Some(ActiveTurn {
                    ui_thread_id: request.ui_thread_id.clone(),
                    prompt_request_id,
                    event_tx: event_tx.clone(),
                    trace: trace.clone(),
                    tool_policy: request.tool_policy,
                    provider_failure: None,
                });
                if connection
                    .out_tx
                    .send(json!({
                        "jsonrpc": "2.0",
                        "id": prompt_request_id,
                        "method": "session/prompt",
                        "params": params,
                    }))
                    .is_err()
                {
                    let active = shared.lock().active_turn.take();
                    if let Some(active) = active {
                        deliver_terminal(
                            active,
                            AgentChatEvent::TurnFailed {
                                failure: crate::ai::reliability::runtime_closed_failure(
                                    ProtocolComponent::Goose,
                                    "goose input closed before the prompt was sent",
                                ),
                            },
                        )
                        .await;
                    }
                }
                if let Some((root, provider, lease)) = sign_in_leader {
                    tokio::spawn(watch_sign_in(
                        root,
                        provider,
                        lease,
                        shared.clone(),
                        prompt_request_id,
                        event_tx.clone(),
                    ));
                }
            }
            GooseCommand::CancelTurn { ui_thread_id } => {
                let active = shared.lock().active_turn.clone();
                let session_id = shared.lock().session_id.clone();
                match (active, session_id) {
                    (Some(active), Some(session_id)) if active.ui_thread_id == ui_thread_id => {
                        // A user Stop is cancellation, not an error: record it
                        // before goose answers so it never reads as a failure.
                        active.trace.terminal(TurnOutcome::Cancelled, None);
                        let _ = connection
                            .out_tx
                            .send(notification("session/cancel", cancel_params(&session_id)));
                    }
                    _ => {
                        tracing::debug!(
                            target: "script_kit::agent_chat",
                            event = "goose_acp_cancel_ignored_no_active_turn",
                        );
                    }
                }
            }
        }
    }

    stop_child(&mut child).await;
    Ok(())
}

/// Run `initialize` and `session/new` once per process.
async fn ensure_session<'a>(
    connection: &mut Connection,
    spec: &GooseLaunchSpec,
    session: &'a mut Option<SessionState>,
) -> std::result::Result<&'a mut SessionState, SessionFailure> {
    if session.is_none() {
        let initialized = connection
            .call("initialize", initialize_params(), HANDSHAKE_TIMEOUT)
            .await
            .map_err(|error| Box::new(error.failure("initialize")))?;
        if initialized.get("protocolVersion").and_then(Value::as_u64)
            != Some(super::protocol::ACP_PROTOCOL_VERSION)
        {
            return Err(Box::new(
                crate::ai::reliability::protocol_failure_with_detail(
                    ProtocolComponent::Goose,
                    crate::ai::reliability::ProtocolFailureFacts::MalformedResponse,
                    "goose negotiated an unsupported ACP protocol version",
                ),
            ));
        }
        let created = connection
            .call(
                "session/new",
                new_session_params(spec, &recipe_deeplink(&spec.recipe)),
                HANDSHAKE_TIMEOUT,
            )
            .await
            .map_err(|error| Box::new(error.failure("session/new")))?;
        let Some(session_id) = created.get("sessionId").and_then(Value::as_str) else {
            return Err(Box::new(
                crate::ai::reliability::protocol_failure_with_detail(
                    ProtocolComponent::Goose,
                    crate::ai::reliability::ProtocolFailureFacts::MalformedResponse,
                    "goose session/new returned no sessionId",
                ),
            ));
        };
        connection.shared.lock().session_id = Some(session_id.to_string());
        let mut state = SessionState {
            session_id: session_id.to_string(),
            config_options: created.get("configOptions").cloned().unwrap_or(Value::Null),
        };
        if let Some(thinking) = spec.thinking.as_deref() {
            // Models without adjustable reasoning offer only "off"; asking
            // them for another level would fail, so only set offered values.
            if state.has_value("thinking_effort", thinking) {
                set_config_option(connection, &mut state, "thinking_effort", thinking)
                    .await
                    .ok();
            }
        }
        tracing::info!(
            target: "script_kit::agent_chat",
            event = "goose_acp_session_ready",
            provider = ?state.current_value("provider"),
            model = ?state.current_value("model"),
        );
        *session = Some(state);
    }
    session.as_mut().ok_or_else(|| {
        Box::new(crate::ai::reliability::runtime_closed_failure(
            ProtocolComponent::Goose,
            "goose session was not established",
        ))
    })
}

async fn set_config_option(
    connection: &mut Connection,
    state: &mut SessionState,
    config_id: &str,
    value: &str,
) -> std::result::Result<(), SessionFailure> {
    let result = connection
        .call(
            "session/set_config_option",
            set_config_option_params(&state.session_id, config_id, value),
            CONFIG_TIMEOUT,
        )
        .await
        .map_err(|error| Box::new(error.failure("session/set_config_option")))?;
    if let Some(options) = result.get("configOptions") {
        state.config_options = options.clone();
    }
    Ok(())
}

/// Switch provider and model when the turn asks for a different pair.
async fn apply_model_selection(
    connection: &mut Connection,
    state: &mut SessionState,
    model_id: &str,
) -> std::result::Result<(), SessionFailure> {
    let (provider, model) = match split_model_selection(model_id) {
        Some((provider, model)) => (provider.to_string(), model.to_string()),
        // A bare model id keeps the session's provider.
        None => (
            state
                .current_value("provider")
                .unwrap_or_default()
                .to_string(),
            model_id.trim().to_string(),
        ),
    };
    let (provider, model) = (provider.as_str(), model.as_str());
    if model.is_empty() {
        return Ok(());
    }
    if !provider.is_empty() && state.current_value("provider") != Some(provider) {
        set_config_option(connection, state, "provider", provider).await?;
    }
    if state.current_value("model") != Some(model) {
        set_config_option(connection, state, "model", model).await?;
    }
    Ok(())
}

async fn read_stdout(
    stdout: tokio::process::ChildStdout,
    shared: SharedState,
    out_tx: mpsc::UnboundedSender<Value>,
) {
    let mut lines = BufReader::new(stdout).lines();
    // Citation markers can span chunks; track them per turn.
    let mut in_citation = false;
    let mut citation_turn = 0_u64;
    while let Ok(Some(line)) = lines.next_line().await {
        let parsed = match parse_line(&line) {
            Ok(parsed) => parsed,
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::agent_chat",
                    event = "goose_acp_unparseable_line",
                    %error,
                    line_chars = line.chars().count(),
                );
                continue;
            }
        };
        match parsed {
            AcpLine::Response { id, outcome } => {
                let active = {
                    let mut shared = shared.lock();
                    if shared
                        .active_turn
                        .as_ref()
                        .is_some_and(|turn| turn.prompt_request_id == id)
                    {
                        shared.active_turn.take()
                    } else {
                        if let Some(waiter) = shared.pending.remove(&id) {
                            let _ = waiter.send(outcome);
                        }
                        continue;
                    }
                };
                if let Some(mut active) = active {
                    let terminal = match active.provider_failure.take() {
                        // goose ends the turn normally after reporting the
                        // failure as text; the failure is the real outcome.
                        Some(failure) if outcome.is_ok() => AgentChatEvent::TurnFailed { failure },
                        _ => prompt_outcome(outcome.as_ref()),
                    };
                    deliver_terminal(active, terminal).await;
                }
            }
            AcpLine::Notification { method, params } => {
                if method != "session/update" {
                    continue;
                }
                let active = {
                    let shared = shared.lock();
                    let same_session = shared.session_id.as_deref()
                        == params.get("sessionId").and_then(Value::as_str);
                    same_session.then(|| shared.active_turn.clone()).flatten()
                };
                let Some(active) = active else {
                    continue;
                };
                let Some(update) = params.get("update") else {
                    continue;
                };
                if let Some(failure) = provider_error_in_update(update) {
                    let mut shared = shared.lock();
                    if let Some(turn) = shared
                        .active_turn
                        .as_mut()
                        .filter(|turn| turn.prompt_request_id == active.prompt_request_id)
                    {
                        turn.provider_failure = Some(failure);
                    }
                    continue;
                }
                if citation_turn != active.prompt_request_id {
                    citation_turn = active.prompt_request_id;
                    in_citation = false;
                }
                let cleaned = strip_citation_markers(update, &mut in_citation);
                let update = cleaned.as_ref().unwrap_or(update);
                send_events_traced(&active.event_tx, &active.trace, map_session_update(update))
                    .await;
            }
            AcpLine::Request { id, method, params } => {
                let reply = match method.as_str() {
                    "session/request_permission" => {
                        let policy = shared
                            .lock()
                            .active_turn
                            .as_ref()
                            .map(|turn| turn.tool_policy);
                        response(&id, permission_outcome(&params, policy))
                    }
                    _ => error_response(&id, -32601, "Method not supported by Script Kit"),
                };
                let _ = out_tx.send(reply);
            }
        }
    }

    // stdout closed: goose exited. Whatever was in flight will never finish.
    // The exit is a fact (RuntimeClosed) unless stderr said why: a missing
    // credential must keep its Sign In action (rules/AI_RELIABILITY.md).
    tracing::warn!(target: "script_kit::agent_chat", event = "goose_acp_stdout_closed");
    let failure = match await_stderr_auth_hint(&shared).await {
        Some(hint) => crate::ai::reliability::provider_failure(
            ProtocolComponent::Goose,
            format!("goose exited: {hint}"),
        ),
        None => crate::ai::reliability::runtime_closed_failure(
            ProtocolComponent::Goose,
            "goose exited while a request was in flight",
        ),
    };
    let (active, pending) = {
        let mut shared = shared.lock();
        (
            shared.active_turn.take(),
            shared.pending.drain().collect::<Vec<_>>(),
        )
    };
    if let Some(active) = active {
        deliver_terminal(active, AgentChatEvent::TurnFailed { failure }).await;
    }
    drop(pending);
}

/// How long to let the stderr reader catch up after stdout closes. stdout EOF
/// and the stderr line that explains it arrive on independent readers, and a
/// lost race would turn a Sign In card into a Reconnect card.
const STDERR_HINT_GRACE: std::time::Duration = std::time::Duration::from_millis(250);

async fn await_stderr_auth_hint(shared: &SharedState) -> Option<String> {
    let deadline = std::time::Instant::now() + STDERR_HINT_GRACE;
    loop {
        if let Some(hint) = shared.lock().stderr_auth_hint.clone() {
            return Some(hint);
        }
        if std::time::Instant::now() >= deadline {
            return None;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

/// A stderr line that names a missing or rejected credential. Only these
/// lines are kept; everything else goose prints stays suppressed.
fn stderr_auth_hint(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let lower = trimmed.to_ascii_lowercase();
    let is_auth = lower.contains("no api key")
        || lower.contains("missing api key")
        || lower.contains("api key not found")
        || lower.contains("not authenticated")
        || lower.contains("authentication required");
    (is_auth && !trimmed.is_empty()).then(|| trimmed.chars().take(300).collect())
}

/// Answer goose's permission request under the turn's tool policy.
///
/// Sessions run in goose's `auto` mode, so goose normally never asks. When it
/// does, a tool the policy admits gets a one-time allow; anything else, or a
/// request with no live turn, is rejected.
fn permission_outcome(params: &Value, policy: Option<AgentChatToolPolicy>) -> Value {
    let tool_name = params
        .get("toolCall")
        .and_then(super::events::tool_name)
        .unwrap_or_default();
    let allowed = policy.is_some_and(|policy| policy.allows_tool(&tool_name));
    let wanted = if allowed {
        ["allow_once", "allow_always"]
    } else {
        ["reject_once", "reject_always"]
    };
    let option_id = params
        .get("options")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|option| {
            option
                .get("kind")
                .and_then(Value::as_str)
                .is_some_and(|kind| wanted.contains(&kind))
        })
        .and_then(|option| option.get("optionId"))
        .and_then(Value::as_str);
    match option_id {
        Some(option_id) => json!({ "outcome": { "outcome": "selected", "optionId": option_id } }),
        None => json!({ "outcome": { "outcome": "cancelled" } }),
    }
}

/// Trace one event through the shared phase vocabulary. This is the single
/// classification point for the goose transport.
fn trace_event(trace: &PhaseTrace, event: &AgentChatEvent) {
    trace.observe_provider_event();
    match event {
        AgentChatEvent::AgentMessageDelta(text) => trace.observe_visible_output(text),
        AgentChatEvent::AgentThoughtDelta(text) => trace.observe_thought(text),
        AgentChatEvent::ToolCallStarted { .. } => trace.observe_tool_call(),
        AgentChatEvent::TurnCompleted {
            outcome: crate::ai::reliability::AiTurnRuntimeOutcome::Cancelled { .. },
        } => trace.terminal(TurnOutcome::Cancelled, None),
        AgentChatEvent::TurnCompleted { .. } => trace.terminal(TurnOutcome::Completed, None),
        AgentChatEvent::TurnFailed { failure } => {
            // Carry the already-classified code; raw provider text never
            // enters the trace (rules/AI_RELIABILITY.md).
            let code = serde_json::to_value(failure.failure.code)
                .ok()
                .and_then(|value| value.as_str().map(str::to_string));
            trace.terminal(TurnOutcome::Failed, code.as_deref());
        }
        _ => {}
    }
}

/// Deliver an event that is not provider output (sign-in progress), so it
/// never counts as the turn's first provider event in the phase trace.
async fn send_untraced(event_tx: &AgentChatEventTx, event: AgentChatEvent) {
    let _ = tokio::time::timeout(SEND_TIMEOUT, event_tx.send(event)).await;
}

/// How long a turn that may not lead a sign-in waits for another turn's
/// login to appear (Text variations start alongside the turn that asks).
const FOLLOW_SIGN_IN_GRACE: std::time::Duration = std::time::Duration::from_secs(3);

enum SignInGate {
    /// Send the prompt. `Some` = this turn leads the login goose is about to
    /// start, so the lease stays held until the sign-in is stored.
    Proceed(Option<(std::path::PathBuf, String, LeaderLease)>),
    /// The turn already ended (declined, cancelled, or sign-in failed).
    Stopped,
}

async fn gate_browser_sign_in(
    root: std::path::PathBuf,
    provider: String,
    interactive: bool,
    event_tx: &AgentChatEventTx,
    trace: &PhaseTrace,
) -> SignInGate {
    if !sign_in::is_browser_sign_in_provider(&provider) || sign_in::refresh_at(&root, &provider) {
        return SignInGate::Proceed(None);
    }
    let role = if interactive {
        sign_in::begin(&root, &provider)
    } else {
        let deadline = std::time::Instant::now() + FOLLOW_SIGN_IN_GRACE;
        loop {
            if sign_in::refresh_at(&root, &provider) {
                return SignInGate::Proceed(None);
            }
            if sign_in::in_flight(&root, &provider) {
                break SignInRole::Follower;
            }
            if std::time::Instant::now() >= deadline {
                send_sign_in_failure(event_tx, trace, &provider, "sign-in required").await;
                return SignInGate::Stopped;
            }
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        }
    };
    match role {
        SignInRole::Leader(lease) => {
            let (consent_tx, consent_rx) = async_channel::bounded(1);
            send_untraced(
                event_tx,
                AgentChatEvent::SignInRequired {
                    provider: provider.clone(),
                    consent: consent_tx,
                },
            )
            .await;
            // A dropped sender (the chat closed or the user stopped) is a no.
            if consent_rx.recv().await != Ok(true) {
                drop(lease);
                tracing::info!(
                    target: "script_kit::agent_chat",
                    event = "goose_sign_in_declined",
                    provider = %provider,
                );
                send_traced(
                    event_tx,
                    trace,
                    AgentChatEvent::TurnCompleted {
                        outcome: crate::ai::reliability::AiTurnRuntimeOutcome::Cancelled {
                            kind: sk_protocol::ai_reliability::CancellationKind::UserCancelled,
                            partial: sk_protocol::ai_reliability::PartialOutputState::None,
                        },
                    },
                )
                .await;
                return SignInGate::Stopped;
            }
            send_untraced(
                event_tx,
                AgentChatEvent::SignInStarted {
                    provider: provider.clone(),
                },
            )
            .await;
            SignInGate::Proceed(Some((root, provider, lease)))
        }
        SignInRole::Follower => {
            send_untraced(
                event_tx,
                AgentChatEvent::SignInStarted {
                    provider: provider.clone(),
                },
            )
            .await;
            if sign_in::wait_for_sign_in(&root, &provider, sign_in::SIGN_IN_WAIT).await {
                send_untraced(event_tx, AgentChatEvent::SignInCompleted { provider }).await;
                SignInGate::Proceed(None)
            } else {
                send_sign_in_failure(event_tx, trace, &provider, "sign-in did not finish").await;
                SignInGate::Stopped
            }
        }
    }
}

async fn send_sign_in_failure(
    event_tx: &AgentChatEventTx,
    trace: &PhaseTrace,
    provider: &str,
    reason: &str,
) {
    send_traced(
        event_tx,
        trace,
        AgentChatEvent::TurnFailed {
            failure: crate::ai::reliability::setup_required_failure(
                ProtocolComponent::Goose,
                &format!("{provider} {reason}"),
                &["browser".to_string()],
            ),
        },
    )
    .await;
}

/// While goose waits for the browser login, report the moment the sign-in
/// is stored. Holding `lease` keeps other turns from starting a second login;
/// it is released once this turn signs in, ends, or the wait runs out.
async fn watch_sign_in(
    root: std::path::PathBuf,
    provider: String,
    lease: LeaderLease,
    shared: SharedState,
    prompt_request_id: u64,
    event_tx: AgentChatEventTx,
) {
    let _lease = lease;
    let deadline = std::time::Instant::now() + sign_in::SIGN_IN_WAIT;
    loop {
        if sign_in::refresh_at(&root, &provider) {
            send_untraced(&event_tx, AgentChatEvent::SignInCompleted { provider }).await;
            return;
        }
        let turn_running = shared
            .lock()
            .active_turn
            .as_ref()
            .is_some_and(|turn| turn.prompt_request_id == prompt_request_id);
        if !turn_running || std::time::Instant::now() >= deadline {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    }
}

async fn send_traced(event_tx: &AgentChatEventTx, trace: &PhaseTrace, event: AgentChatEvent) {
    trace_event(trace, &event);
    let _ = tokio::time::timeout(SEND_TIMEOUT, event_tx.send(event)).await;
    trace.teardown();
}

async fn send_events_traced(
    event_tx: &AgentChatEventTx,
    trace: &PhaseTrace,
    events: Vec<AgentChatEvent>,
) {
    let reveal_count = events.iter().filter(|event| is_reveal_chunk(event)).count();
    let mut reveal_index = 0usize;
    for event in events {
        let pause_after = is_reveal_chunk(&event) && {
            reveal_index += 1;
            reveal_index < reveal_count
        };
        trace_event(trace, &event);
        if tokio::time::timeout(SEND_TIMEOUT, event_tx.send(event))
            .await
            .is_err()
        {
            break;
        }
        if pause_after {
            tokio::time::sleep(std::time::Duration::from_millis(REVEAL_CHUNK_DELAY_MS)).await;
        }
    }
}

fn is_reveal_chunk(event: &AgentChatEvent) -> bool {
    matches!(
        event,
        AgentChatEvent::AgentMessageDelta(_) | AgentChatEvent::AgentThoughtDelta(_)
    )
}

async fn deliver_terminal(active: ActiveTurn, event: AgentChatEvent) {
    trace_event(&active.trace, &event);
    // A blocked consumer must not hold transport teardown forever.
    let _ = tokio::time::timeout(SEND_TIMEOUT, active.event_tx.send(event)).await;
    active.trace.teardown();
}

/// goose never started: answer every command with the spawn failure until the
/// connection is dropped, so warm-up and turns both surface the real cause.
async fn fail_commands_until_closed(
    rx: async_channel::Receiver<GooseCommand>,
    failure: crate::ai::reliability::AppFailureRecord,
) {
    while let Ok(command) = rx.recv().await {
        let event_tx = match command {
            GooseCommand::PrepareSession { event_tx }
            | GooseCommand::StartTurn { event_tx, .. } => event_tx,
            GooseCommand::CancelTurn { .. } => continue,
        };
        let _ = tokio::time::timeout(
            SEND_TIMEOUT,
            event_tx.send(AgentChatEvent::TurnFailed {
                failure: failure.clone(),
            }),
        )
        .await;
    }
}

async fn stop_child(child: &mut tokio::process::Child) {
    if matches!(child.try_wait(), Ok(Some(_))) {
        return;
    }
    let _ = child.start_kill();
    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), child.wait()).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goose_runtime_is_an_agent_chat_connection() {
        fn accepts(_: &dyn AgentChatConnection) {}
        let (tx, _rx) = async_channel::bounded::<GooseCommand>(1);
        let runtime = GooseRuntime {
            tx,
            spec: Arc::new(GooseLaunchSpec::new("/bin/false", "/tmp")),
        };
        accepts(&runtime);
    }

    #[test]
    fn permission_requests_follow_the_turn_tool_policy() {
        let params = json!({
            "toolCall": {"toolCallId": "t1", "_meta": {"goose": {"toolCall": {"toolName": "developer__shell"}}}},
            "options": [
                {"optionId": "yes", "name": "Allow", "kind": "allow_once"},
                {"optionId": "no", "name": "Reject", "kind": "reject_once"}
            ]
        });
        assert_eq!(
            permission_outcome(&params, Some(AgentChatToolPolicy::Full))["outcome"]["optionId"],
            "yes"
        );
        assert_eq!(
            permission_outcome(&params, Some(AgentChatToolPolicy::WebSearchOnly))["outcome"]
                ["optionId"],
            "no"
        );
        assert_eq!(
            permission_outcome(&params, None)["outcome"]["optionId"],
            "no"
        );
        let search = json!({
            "toolCall": {"toolCallId": "t2", "_meta": {"goose": {"toolCall": {"toolName": "scriptkit__web_search"}}}},
            "options": [
                {"optionId": "yes", "name": "Allow", "kind": "allow_once"},
                {"optionId": "no", "name": "Reject", "kind": "reject_once"}
            ]
        });
        assert_eq!(
            permission_outcome(&search, Some(AgentChatToolPolicy::WebSearchOnly))["outcome"]
                ["optionId"],
            "yes"
        );
    }
}
