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

use crate::ai::agent_chat::approval::AgentApprovalMode;
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
    /// Switch how much the model may do before it asks.
    SetApprovalMode {
        mode: AgentApprovalMode,
    },
    /// Add a message to the turn that is running.
    Steer {
        blocks: Vec<crate::ai::agent_chat::content::ContentBlock>,
        reply: async_channel::Sender<std::result::Result<(), String>>,
    },
    /// List the user messages the session can go back to.
    ForkPoints {
        event_tx: AgentChatEventTx,
    },
    /// Continue in a copy of the session that ends just before a user message.
    ForkTo {
        entry_id: String,
        event_tx: AgentChatEventTx,
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

    fn fork_points(&self) -> crate::ai::reliability::AiAdapterResult<AgentChatEventRx> {
        let (event_tx, event_rx) = async_channel::bounded(4);
        self.send(GooseCommand::ForkPoints { event_tx })?;
        Ok(event_rx)
    }

    fn fork_to_entry(
        &self,
        entry_id: String,
    ) -> crate::ai::reliability::AiAdapterResult<AgentChatEventRx> {
        let (event_tx, event_rx) = async_channel::bounded(4);
        self.send(GooseCommand::ForkTo { entry_id, event_tx })?;
        Ok(event_rx)
    }

    fn steer(
        &self,
        blocks: Vec<crate::ai::agent_chat::content::ContentBlock>,
    ) -> crate::ai::reliability::AiAdapterResult<
        async_channel::Receiver<std::result::Result<(), String>>,
    > {
        let (reply, rx) = async_channel::bounded(1);
        self.send(GooseCommand::Steer { blocks, reply })?;
        Ok(rx)
    }

    fn set_approval_mode(
        &self,
        mode: AgentApprovalMode,
    ) -> crate::ai::reliability::AiAdapterResult<()> {
        self.send(GooseCommand::SetApprovalMode { mode })
    }

    fn tool_session_id(&self) -> Option<String> {
        self.spec.agent_policy_id.clone()
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
    /// The run goose is executing for this session (needed to steer it).
    active_run_id: Option<String>,
    approval_mode: AgentApprovalMode,
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

    let shared: SharedState = Arc::new(Mutex::new(Shared {
        approval_mode: spec.approval_mode,
        ..Shared::default()
    }));
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
            GooseCommand::SetApprovalMode { mode } => {
                shared.lock().approval_mode = mode;
                if let Some(state) = session.as_ref() {
                    let result = connection
                        .call(
                            "session/set_mode",
                            json!({ "sessionId": state.session_id, "modeId": mode.goose_mode() }),
                            CONFIG_TIMEOUT,
                        )
                        .await;
                    tracing::info!(
                        target: "script_kit::agent_chat",
                        event = "goose_approval_mode_set",
                        mode = mode.id(),
                        ok = result.is_ok(),
                    );
                }
            }
            GooseCommand::Steer { blocks, reply } => {
                let outcome = steer(&mut connection, &shared, &blocks).await;
                let _ = reply.send(outcome).await;
            }
            GooseCommand::ForkPoints { event_tx } => {
                let event = match session.as_ref() {
                    Some(state) => match export_session(&mut connection, &state.session_id).await {
                        Ok(exported) => AgentChatEvent::ForkPointsAvailable {
                            entries: user_fork_points(&exported),
                        },
                        Err(failure) => AgentChatEvent::TurnFailed { failure: *failure },
                    },
                    None => AgentChatEvent::ForkPointsAvailable {
                        entries: Vec::new(),
                    },
                };
                let _ = tokio::time::timeout(SEND_TIMEOUT, event_tx.send(event)).await;
            }
            GooseCommand::ForkTo { entry_id, event_tx } => {
                let event = match session.as_mut() {
                    Some(state) => {
                        match fork_before(&mut connection, &spec, state, &entry_id).await {
                            Ok(text) => AgentChatEvent::ForkCompleted { text },
                            Err(failure) => AgentChatEvent::TurnFailed { failure: *failure },
                        }
                    }
                    None => AgentChatEvent::failed(
                        ProtocolComponent::Goose,
                        "there is no conversation to go back in yet",
                    ),
                };
                let _ = tokio::time::timeout(SEND_TIMEOUT, event_tx.send(event)).await;
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
        let mode = connection.shared.lock().approval_mode;
        let _ = connection
            .call(
                "session/set_mode",
                json!({ "sessionId": state.session_id, "modeId": mode.goose_mode() }),
                CONFIG_TIMEOUT,
            )
            .await;
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
    let mut tool_names = ToolNames::new();
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
                if let Some(run) = active_run_update(&params) {
                    let mut shared = shared.lock();
                    if shared.session_id.as_deref()
                        == params.get("sessionId").and_then(Value::as_str)
                    {
                        shared.active_run_id = run;
                    }
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
                remember_tool_name(&mut tool_names, update);
                let cleaned = strip_citation_markers(update, &mut in_citation);
                let update = cleaned.as_ref().unwrap_or(update);
                send_events_traced(&active.event_tx, &active.trace, map_session_update(update))
                    .await;
            }
            AcpLine::Request { id, method, params } => {
                let reply = match method.as_str() {
                    "session/request_permission" => {
                        let (active, mode) = {
                            let shared = shared.lock();
                            (shared.active_turn.clone(), shared.approval_mode)
                        };
                        let named = with_known_tool_name(&params, &tool_names);
                        let params = named.unwrap_or(params);
                        match permission_route(
                            &params,
                            active.as_ref().map(|t| t.tool_policy),
                            mode,
                        ) {
                            PermissionRoute::Answer(outcome) => response(&id, outcome),
                            PermissionRoute::AskUser => match active {
                                Some(active) => {
                                    tokio::spawn(ask_user_permission(
                                        id,
                                        params,
                                        active.event_tx,
                                        out_tx.clone(),
                                    ));
                                    continue;
                                }
                                None => response(&id, permission_outcome(&params, false)),
                            },
                        }
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

enum PermissionRoute {
    /// Script Kit answers without asking.
    Answer(Value),
    /// The person decides.
    AskUser,
}

/// Decide who answers goose's permission request.
///
/// A tool the turn's policy forbids, or a request with no live turn, is
/// rejected. A tool the approval mode lets through is allowed once. Anything
/// else goes to the person.
fn permission_route(
    params: &Value,
    policy: Option<AgentChatToolPolicy>,
    mode: AgentApprovalMode,
) -> PermissionRoute {
    let tool_name = params
        .get("toolCall")
        .and_then(super::events::tool_name)
        .unwrap_or_default();
    let Some(policy) = policy else {
        return PermissionRoute::Answer(permission_outcome(params, false));
    };
    if !policy.allows_tool(&tool_name) {
        return PermissionRoute::Answer(permission_outcome(params, false));
    }
    if !policy.allows_permission_prompts()
        || mode.allows_without_asking(&tool_name)
        || is_bookkeeping_tool(&tool_name)
    {
        return PermissionRoute::Answer(permission_outcome(params, true));
    }
    PermissionRoute::AskUser
}

/// Tools that only change the session's own notes or what it has loaded:
/// the todo list, skills, and recipe sources. They never ask.
fn is_bookkeeping_tool(tool_name: &str) -> bool {
    matches!(tool_name, "todo__todo_write" | "load" | "load_skill")
}

/// Show goose's question in the chat and send back the person's answer. A
/// closed chat or Stop counts as no.
async fn ask_user_permission(
    id: Value,
    params: Value,
    event_tx: AgentChatEventTx,
    out_tx: mpsc::UnboundedSender<Value>,
) {
    let (reply_tx, reply_rx) = async_channel::bounded(1);
    let request = approval_request(&params, reply_tx);
    let sent = tokio::time::timeout(
        SEND_TIMEOUT,
        event_tx.send(AgentChatEvent::ApprovalRequested(Box::new(request))),
    )
    .await
    .is_ok_and(|sent| sent.is_ok());
    let choice = if sent {
        reply_rx.recv().await.ok().flatten()
    } else {
        None
    };
    let outcome = match choice {
        Some(option_id) => json!({ "outcome": { "outcome": "selected", "optionId": option_id } }),
        None => json!({ "outcome": { "outcome": "cancelled" } }),
    };
    let _ = out_tx.send(response(&id, outcome));
}

/// The question as the chat shows it: what the tool is and what it would do.
fn approval_request(
    params: &Value,
    reply_tx: async_channel::Sender<Option<String>>,
) -> crate::ai::agent_chat::ui::AgentChatApprovalRequest {
    static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let tool_call = params.get("toolCall").cloned().unwrap_or(Value::Null);
    let tool_name = super::events::tool_name(&tool_call).unwrap_or_else(|| "tool".into());
    let title = tool_call
        .get("title")
        .and_then(Value::as_str)
        .filter(|title| !title.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("Run {tool_name}"));
    let body = tool_call
        .get("rawInput")
        .filter(|input| !input.is_null())
        .map(|input| match input.get("command").and_then(Value::as_str) {
            Some(command) => command.to_string(),
            None => serde_json::to_string_pretty(input).unwrap_or_default(),
        })
        .unwrap_or_default();
    let options = params
        .get("options")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|option| {
            let option_id = option.get("optionId").and_then(Value::as_str)?;
            let kind = option
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or(option_id);
            Some(crate::ai::agent_chat::ui::AgentChatApprovalOption {
                option_id: option_id.to_string(),
                name: approval_option_name(kind).to_string(),
                kind: approval_option_kind(kind).to_string(),
            })
        })
        .collect();
    crate::ai::agent_chat::ui::AgentChatApprovalRequest {
        id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        title,
        body,
        preview: None,
        options,
        reply_tx,
    }
}

fn approval_option_name(kind: &str) -> &'static str {
    match kind {
        "allow_once" => "Allow",
        "allow_always" => "Always allow this tool",
        "reject_always" => "Never allow this tool",
        _ => "Don't allow",
    }
}

fn approval_option_kind(kind: &str) -> &'static str {
    match kind {
        "allow_once" => "AllowOnce",
        "allow_always" => "AllowAlways",
        "reject_always" => "RejectAlways",
        _ => "RejectOnce",
    }
}

/// `Some(run)` when a `session_info_update` reports the session's active run.
/// Tool names by tool-call id, newest kept. goose names the tool in its
/// `tool_call` update but its permission request carries only the id.
type ToolNames = std::collections::VecDeque<(String, String)>;
const TOOL_NAMES_KEPT: usize = 256;

fn remember_tool_name(names: &mut ToolNames, update: &Value) {
    let Some(id) = update.get("toolCallId").and_then(Value::as_str) else {
        return;
    };
    let Some(name) = update
        .get("_meta")
        .and_then(|meta| meta.get("goose"))
        .and_then(|goose| goose.get("toolCall"))
        .and_then(|tool| tool.get("toolName"))
        .and_then(Value::as_str)
    else {
        return;
    };
    if names.iter().any(|(known, _)| known == id) {
        return;
    }
    if names.len() >= TOOL_NAMES_KEPT {
        names.pop_front();
    }
    names.push_back((id.to_string(), name.to_string()));
}

/// The permission request with goose's tool name filled in from the earlier
/// `tool_call` update, when the request itself does not name the tool.
fn with_known_tool_name(params: &Value, names: &ToolNames) -> Option<Value> {
    let tool_call = params.get("toolCall")?;
    if super::events::tool_name(tool_call).is_some() {
        return None;
    }
    let id = tool_call.get("toolCallId").and_then(Value::as_str)?;
    let (_, name) = names.iter().rev().find(|(known, _)| known == id)?;
    let mut params = params.clone();
    params["toolCall"]["_meta"] = json!({"goose": {"toolCall": {"toolName": name}}});
    Some(params)
}

fn active_run_update(params: &Value) -> Option<Option<String>> {
    let update = params.get("update")?;
    if update.get("sessionUpdate").and_then(Value::as_str) != Some("session_info_update") {
        return None;
    }
    let run = update.get("_meta")?.get("goose")?.get("activeRunId")?;
    Some(run.as_str().map(str::to_string))
}

async fn steer(
    connection: &mut Connection,
    shared: &SharedState,
    blocks: &[crate::ai::agent_chat::content::ContentBlock],
) -> std::result::Result<(), String> {
    let (session_id, run_id) = {
        let shared = shared.lock();
        (shared.session_id.clone(), shared.active_run_id.clone())
    };
    let (Some(session_id), Some(run_id)) = (session_id, run_id) else {
        return Err("nothing is running to add to".into());
    };
    let mut params = prompt_params(&session_id, blocks).map_err(|error| error.to_string())?;
    params["expectedRunId"] = json!(run_id);
    connection
        .call("_goose/unstable/session/steer", params, CONFIG_TIMEOUT)
        .await
        .map(|_| ())
        .map_err(|error| match error {
            CallError::Acp(error) => error.detail(),
            CallError::Closed => "goose closed".into(),
            CallError::Timeout => "goose did not answer in time".into(),
        })
}

async fn export_session(
    connection: &mut Connection,
    session_id: &str,
) -> std::result::Result<Value, SessionFailure> {
    let exported = connection
        .call(
            "_goose/unstable/session/export",
            json!({ "sessionId": session_id, "format": "json" }),
            CONFIG_TIMEOUT,
        )
        .await
        .map_err(|error| Box::new(error.failure("_goose/unstable/session/export")))?;
    let data = exported.get("data").and_then(Value::as_str).unwrap_or("{}");
    serde_json::from_str(data).map_err(|error| {
        Box::new(crate::ai::reliability::protocol_failure_with_detail(
            ProtocolComponent::Goose,
            crate::ai::reliability::ProtocolFailureFacts::MalformedResponse,
            &format!("goose session export was not JSON: {error}"),
        ))
    })
}

/// The messages of an exported goose session.
fn exported_messages(exported: &Value) -> &[Value] {
    let conversation = exported.get("conversation");
    conversation
        .and_then(Value::as_array)
        .or_else(|| {
            conversation
                .and_then(|c| c.get("messages"))
                .and_then(Value::as_array)
        })
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

/// The text the person typed in one exported message, or `None` when it is
/// not a visible user message (tool results, hidden context).
fn user_text(message: &Value) -> Option<String> {
    if message.get("role").and_then(Value::as_str) != Some("user") {
        return None;
    }
    if message
        .get("metadata")
        .and_then(|metadata| metadata.get("userVisible"))
        .and_then(Value::as_bool)
        == Some(false)
    {
        return None;
    }
    let content = message.get("content").and_then(Value::as_array)?;
    let texts: Vec<&str> = content
        .iter()
        .filter(|block| block.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|block| block.get("text").and_then(Value::as_str))
        .collect();
    let only_text = content.iter().all(|block| {
        matches!(
            block.get("type").and_then(Value::as_str),
            Some("text") | Some("image")
        )
    });
    (only_text && !texts.is_empty()).then(|| texts.join("\n"))
}

/// Fork points for every visible user message. The entry id is the message's
/// creation time, which `session/fork` cuts before.
fn user_fork_points(exported: &Value) -> Vec<crate::ai::agent_chat::events::AgentChatForkPoint> {
    exported_messages(exported)
        .iter()
        .filter_map(|message| {
            let text = user_text(message)?;
            let created = message.get("created").and_then(Value::as_i64)?;
            Some(crate::ai::agent_chat::events::AgentChatForkPoint {
                entry_id: created.to_string(),
                text,
            })
        })
        .collect()
}

/// Continue in a copy of the session that ends just before the user message
/// created at `entry_id`. The original session stays in goose's history.
async fn fork_before(
    connection: &mut Connection,
    spec: &GooseLaunchSpec,
    state: &mut SessionState,
    entry_id: &str,
) -> std::result::Result<String, SessionFailure> {
    let before: i64 = entry_id.parse().map_err(|_| {
        Box::new(crate::ai::reliability::protocol_failure_with_detail(
            ProtocolComponent::Goose,
            crate::ai::reliability::ProtocolFailureFacts::MalformedResponse,
            "unknown rewind point",
        ))
    })?;
    let exported = export_session(connection, &state.session_id).await?;
    let text = exported_messages(&exported)
        .iter()
        .find(|message| message.get("created").and_then(Value::as_i64) == Some(before))
        .and_then(user_text)
        .unwrap_or_default();
    let forked = connection
        .call(
            "session/fork",
            json!({
                "sessionId": state.session_id,
                "cwd": spec.cwd,
                "mcpServers": [],
                "_meta": { "conversationBefore": before },
            }),
            HANDSHAKE_TIMEOUT,
        )
        .await
        .map_err(|error| Box::new(error.failure("session/fork")))?;
    let Some(session_id) = forked.get("sessionId").and_then(Value::as_str) else {
        return Err(Box::new(
            crate::ai::reliability::protocol_failure_with_detail(
                ProtocolComponent::Goose,
                crate::ai::reliability::ProtocolFailureFacts::MalformedResponse,
                "goose session/fork returned no sessionId",
            ),
        ));
    };
    {
        let mut shared = connection.shared.lock();
        shared.session_id = Some(session_id.to_string());
        shared.active_run_id = None;
    }
    state.session_id = session_id.to_string();
    if let Some(options) = forked.get("configOptions") {
        state.config_options = options.clone();
    }
    let mode = connection.shared.lock().approval_mode;
    let _ = connection
        .call(
            "session/set_mode",
            json!({ "sessionId": state.session_id, "modeId": mode.goose_mode() }),
            CONFIG_TIMEOUT,
        )
        .await;
    tracing::info!(
        target: "script_kit::agent_chat",
        event = "goose_session_forked",
        before,
    );
    Ok(text)
}

/// A fixed answer to goose's permission request.
fn permission_outcome(params: &Value, allowed: bool) -> Value {
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
            | GooseCommand::StartTurn { event_tx, .. }
            | GooseCommand::ForkPoints { event_tx }
            | GooseCommand::ForkTo { event_tx, .. } => event_tx,
            GooseCommand::Steer { reply, .. } => {
                let _ = reply.send(Err("goose is not running".into())).await;
                continue;
            }
            GooseCommand::CancelTurn { .. } | GooseCommand::SetApprovalMode { .. } => continue,
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
    fn permission_requests_follow_the_turn_tool_policy_and_approval_mode() {
        let request = |tool: &str| {
            json!({
                "toolCall": {"toolCallId": "t1", "_meta": {"goose": {"toolCall": {"toolName": tool}}}},
                "options": [
                    {"optionId": "yes", "name": "Allow", "kind": "allow_once"},
                    {"optionId": "no", "name": "Reject", "kind": "reject_once"}
                ]
            })
        };
        let answer = |route: PermissionRoute| match route {
            PermissionRoute::Answer(outcome) => {
                outcome["outcome"]["optionId"].as_str().map(str::to_string)
            }
            PermissionRoute::AskUser => None,
        };
        let ask = AgentApprovalMode::AskBeforeChanges;
        let shell = request("developer__shell");
        assert!(matches!(
            permission_route(&shell, Some(AgentChatToolPolicy::Full), ask),
            PermissionRoute::AskUser
        ));
        assert_eq!(
            answer(permission_route(
                &shell,
                Some(AgentChatToolPolicy::WebSearchOnly),
                ask
            ))
            .as_deref(),
            Some("no")
        );
        assert_eq!(
            answer(permission_route(&shell, None, ask)).as_deref(),
            Some("no")
        );
        assert_eq!(
            answer(permission_route(
                &shell,
                Some(AgentChatToolPolicy::Full),
                AgentApprovalMode::FullAccess
            ))
            .as_deref(),
            Some("yes")
        );
        let edit = request("scriptkit__edit");
        assert_eq!(
            answer(permission_route(
                &edit,
                Some(AgentChatToolPolicy::Full),
                AgentApprovalMode::AcceptEdits
            ))
            .as_deref(),
            Some("yes")
        );
        assert!(matches!(
            permission_route(
                &shell,
                Some(AgentChatToolPolicy::Full),
                AgentApprovalMode::AcceptEdits
            ),
            PermissionRoute::AskUser
        ));
        assert_eq!(
            answer(permission_route(
                &request("todo__todo_write"),
                Some(AgentChatToolPolicy::Full),
                ask
            ))
            .as_deref(),
            Some("yes")
        );
        assert!(matches!(
            permission_route(&request("delegate"), Some(AgentChatToolPolicy::Full), ask),
            PermissionRoute::AskUser
        ));
        let search = request("scriptkit__web_search");
        assert_eq!(
            answer(permission_route(
                &search,
                Some(AgentChatToolPolicy::WebSearchOnly),
                ask
            ))
            .as_deref(),
            Some("yes")
        );
    }

    #[test]
    fn approval_request_reads_goose_options_and_command() {
        let (tx, _rx) = async_channel::bounded(1);
        let request = approval_request(
            &json!({
                "toolCall": {"title": "Run a command", "rawInput": {"command": "rm -rf build"}},
                "options": [
                    {"optionId": "allow_always", "kind": "allow_always"},
                    {"optionId": "allow_once", "kind": "allow_once"},
                    {"optionId": "reject_once", "kind": "reject_once"}
                ]
            }),
            tx,
        );
        assert_eq!(request.title, "Run a command");
        assert_eq!(request.body, "rm -rf build");
        assert!(request.options[0].is_persistent_allow());
        assert!(request.options[2].is_reject());
    }

    #[test]
    fn fork_points_are_visible_user_messages_with_their_times() {
        let exported = json!({"conversation": [
            {"role": "user", "created": 100, "content": [{"type": "text", "text": "first"}], "metadata": {"userVisible": true}},
            {"role": "assistant", "created": 101, "content": [{"type": "text", "text": "hi"}]},
            {"role": "user", "created": 102, "content": [{"type": "toolResponse", "id": "x"}]},
            {"role": "user", "created": 103, "content": [{"type": "text", "text": "hidden"}], "metadata": {"userVisible": false}},
            {"role": "user", "created": 104, "content": [{"type": "text", "text": "second"}]}
        ]});
        let points = user_fork_points(&exported);
        let ids: Vec<_> = points
            .iter()
            .map(|p| (p.entry_id.as_str(), p.text.as_str()))
            .collect();
        assert_eq!(ids, vec![("100", "first"), ("104", "second")]);
    }

    #[test]
    fn permission_requests_learn_the_tool_name_from_the_earlier_tool_call() {
        let mut names = ToolNames::new();
        remember_tool_name(
            &mut names,
            &json!({"sessionUpdate": "tool_call", "toolCallId": "c1", "_meta": {"goose": {"toolCall": {"toolName": "todo__todo_write"}}}}),
        );
        // goose 1.52's permission request: a title and the call id, no name.
        let asked = json!({
            "toolCall": {"toolCallId": "c1", "title": "todo: todo write"},
            "options": [
                {"optionId": "allow_once", "name": "allow_once", "kind": "allow_once"},
                {"optionId": "reject_once", "name": "reject_once", "kind": "reject_once"}
            ]
        });
        let named = with_known_tool_name(&asked, &names).expect("name filled in");
        let answer = |route: PermissionRoute| match route {
            PermissionRoute::Answer(outcome) => {
                outcome["outcome"]["optionId"].as_str().map(str::to_string)
            }
            PermissionRoute::AskUser => None,
        };
        let ask = AgentApprovalMode::AskBeforeChanges;
        assert_eq!(
            answer(permission_route(
                &named,
                Some(AgentChatToolPolicy::Full),
                ask
            ))
            .as_deref(),
            Some("allow_once")
        );
        let unknown = json!({"toolCall": {"toolCallId": "c9", "title": "scriptkit: write"}});
        assert!(with_known_tool_name(&unknown, &names).is_none());
        assert!(matches!(
            permission_route(
                &unknown,
                Some(AgentChatToolPolicy::Full),
                AgentApprovalMode::AcceptEdits
            ),
            PermissionRoute::AskUser
        ));
    }

    #[test]
    fn active_run_updates_are_recognised() {
        let started = json!({"update": {"sessionUpdate": "session_info_update", "_meta": {"goose": {"activeRunId": "r1"}}}});
        assert_eq!(active_run_update(&started), Some(Some("r1".into())));
        let ended = json!({"update": {"sessionUpdate": "session_info_update", "_meta": {"goose": {"activeRunId": null}}}});
        assert_eq!(active_run_update(&ended), Some(None));
        let chunk = json!({"update": {"sessionUpdate": "agent_message_chunk"}});
        assert_eq!(active_run_update(&chunk), None);
    }
}
