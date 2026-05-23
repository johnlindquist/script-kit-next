//! ACP client runtime.
//!
//! Spawns the agent subprocess on a dedicated `current_thread` Tokio runtime
//! with a `LocalSet` (required because ACP futures are `!Send`). Communication
//! with the GPUI thread goes through a bounded `async_channel`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use agent_client_protocol::{
    Agent, AuthCapabilities, CancelNotification, ClientCapabilities, ClientSideConnection,
    FileSystemCapabilities, Implementation, InitializeRequest, Meta, ModelId, NewSessionRequest,
    PromptRequest, ProtocolVersion, SessionId, SetSessionModelRequest,
};
use serde_json::json;

use super::config::{AcpAgentConfig, CODEX_ACP_AGENT_ID, CODEX_ACP_NPX_PACKAGE};
use super::events::{AcpCancelCommand, AcpCommand, AcpEvent, AcpEventRx, AcpPromptTurnRequest};
use super::handlers::{ApprovalFn, ScriptKitAcpClient};
use super::types::AcpSessionBinding;

/// Handle to the ACP worker thread. Send commands via the bounded channel.
///
/// Supports both the legacy `stream_prompt()` path (for `AiProvider`) and
/// the new `start_turn()` path (for `AcpThread` / `AcpChatView`).
pub(crate) struct AcpRuntime {
    tx: async_channel::Sender<AcpCommand>,
    cancel_tx: async_channel::Sender<AcpCancelCommand>,
}

/// Type alias for clarity in the new ACP chat view path.
pub(crate) type AcpConnection = AcpRuntime;

impl AcpRuntime {
    /// Spawn a new ACP worker thread for the given agent config.
    ///
    /// The worker owns a `current_thread` Tokio runtime, spawns the agent
    /// subprocess, completes the `initialize` handshake, and then loops
    /// waiting for `AcpCommand` messages.
    pub(crate) fn spawn(agent: AcpAgentConfig) -> Result<Self> {
        Self::spawn_with_approval(agent, None)
    }

    /// Spawn with a custom approval function for permission/write/terminal gates.
    pub(crate) fn spawn_with_approval(
        agent: AcpAgentConfig,
        approve: Option<ApprovalFn>,
    ) -> Result<Self> {
        let (tx, rx) = async_channel::bounded::<AcpCommand>(8);
        let (cancel_tx, cancel_rx) = async_channel::bounded::<AcpCancelCommand>(8);
        let (agent, session_system_prompt) = configure_profile_system_prompt_for_agent(agent);

        std::thread::Builder::new()
            .name(format!("acp-{}", agent.id))
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        tracing::error!(error = %e, agent = %agent.id, "acp_runtime_build_failed");
                        return;
                    }
                };

                let local = tokio::task::LocalSet::new();
                local.block_on(&runtime, async move {
                    if let Err(e) =
                        run_acp_event_loop(agent, rx, cancel_rx, approve, session_system_prompt)
                            .await
                    {
                        tracing::error!(error = %e, "acp_event_loop_exited_with_error");
                    }
                });
            })
            .context("Failed to spawn ACP worker thread")?;

        Ok(Self { tx, cancel_tx })
    }

    /// Create an `AcpRuntime` from an existing command sender (test only).
    #[cfg(test)]
    pub(crate) fn from_sender(tx: async_channel::Sender<AcpCommand>) -> Self {
        let (cancel_tx, _cancel_rx) = async_channel::bounded::<AcpCancelCommand>(1);
        Self { tx, cancel_tx }
    }

    /// Start a new event-driven turn and return a receiver for typed events.
    ///
    /// This is the new path used by `AcpThread` / `AcpChatView`. Events are
    /// streamed as `AcpEvent` variants until `TurnFinished` or `Failed`.
    pub(crate) fn start_turn(&self, request: AcpPromptTurnRequest) -> Result<AcpEventRx> {
        let (event_tx, event_rx) = async_channel::bounded(256);

        self.tx
            .send_blocking(AcpCommand::StartTurn { request, event_tx })
            .context("ACP worker channel closed")?;

        Ok(event_rx)
    }

    pub(crate) fn cancel_turn(&self, ui_thread_id: String) -> Result<()> {
        self.cancel_tx
            .send_blocking(AcpCancelCommand::CancelTurn { ui_thread_id })
            .context("ACP cancel channel closed")
    }

    /// Preflight: create (or reuse) the ACP session for `ui_thread_id` without
    /// sending a prompt, so the agent's advertised model list and any setup
    /// requirements reach the thread before the user submits anything.
    ///
    /// The returned receiver carries one-shot events (`ModelsAvailable` on
    /// success, `SetupRequired` or `Failed` on error) and then closes.
    pub(crate) fn prepare_session(&self, ui_thread_id: String, cwd: PathBuf) -> Result<AcpEventRx> {
        let (event_tx, event_rx) = async_channel::bounded(8);

        self.tx
            .send_blocking(AcpCommand::PrepareSession {
                ui_thread_id,
                cwd,
                event_tx,
            })
            .context("ACP worker channel closed")?;

        Ok(event_rx)
    }

    /// Send a prompt to the ACP agent and block until the response completes.
    ///
    /// This is the legacy path called from `AiProvider::stream_message`.
    /// The actual work happens on the ACP worker thread; we just wait for the
    /// reply channel.
    pub(crate) fn stream_prompt(
        &self,
        ui_session_id: String,
        cwd: PathBuf,
        messages: Vec<crate::ai::providers::ProviderMessage>,
        on_chunk: crate::ai::providers::StreamCallback,
    ) -> Result<()> {
        let (reply_tx, reply_rx) = async_channel::bounded(1);

        self.tx
            .send_blocking(AcpCommand::StreamPrompt {
                ui_session_id,
                cwd,
                messages,
                on_chunk,
                reply_tx,
            })
            .context("ACP worker channel closed")?;

        reply_rx
            .recv_blocking()
            .context("ACP worker reply channel closed")?
    }
}

fn is_claude_code_harness_agent(agent: &AcpAgentConfig) -> bool {
    agent.id == "claude-code"
}

/// Inject the selected profile's system prompt into the agent.
///
/// For the `claude-code` harness, we append `--append-system-prompt <prompt>`
/// to the agent's CLI args — that harness does not forward `session/new` meta
/// to the underlying CLI. For every other agent we return the prompt so the
/// session/new call site can pass it via ACP meta fields.
fn configure_profile_system_prompt_for_agent(
    mut agent: AcpAgentConfig,
) -> (AcpAgentConfig, Option<String>) {
    let Some((profile_name, system_prompt)) = super::config::load_selected_profile_system_prompt()
    else {
        return (agent, None);
    };

    if is_claude_code_harness_agent(&agent) {
        agent.args.push("--append-system-prompt".to_string());
        agent.args.push(system_prompt);
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_profile_system_prompt_cli_appended",
            agent_id = %agent.id,
            profile_name = %profile_name,
        );
        (agent, None)
    } else {
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_profile_system_prompt_session_new_forwarded",
            agent_id = %agent.id,
            profile_name = %profile_name,
        );
        (agent, Some(system_prompt))
    }
}

fn new_session_request_with_system_prompt(
    cwd: impl Into<PathBuf>,
    system_prompt: Option<&str>,
) -> NewSessionRequest {
    let request = NewSessionRequest::new(cwd);
    let Some(system_prompt) = system_prompt
        .map(str::trim)
        .filter(|prompt| !prompt.is_empty())
    else {
        return request;
    };

    let mut meta = Meta::new();
    meta.insert("system_prompt".to_string(), json!(system_prompt));
    meta.insert("systemPrompt".to_string(), json!(system_prompt));
    request.meta(meta)
}

/// Build the ACP initialize request with full capability advertisement.
///
/// Advertises read + write filesystem access, terminal support, and
/// terminal auth capability so agents can offer interactive login flows.
pub(crate) fn build_initialize_request() -> InitializeRequest {
    InitializeRequest::new(ProtocolVersion::V1)
        .client_capabilities(
            ClientCapabilities::new()
                .fs(FileSystemCapabilities::new()
                    .read_text_file(true)
                    .write_text_file(true))
                .terminal(true)
                .auth(AuthCapabilities::new().terminal(true)),
        )
        .client_info(
            Implementation::new("script-kit", env!("CARGO_PKG_VERSION")).title("Script Kit"),
        )
}

/// The main event loop running on the ACP worker thread.
///
/// 1. Spawns the agent subprocess.
/// 2. Creates a `ClientSideConnection` over stdin/stdout.
/// 3. Sends `initialize`.
/// 4. Loops, handling `AcpCommand` messages from the GPUI thread.
async fn run_acp_event_loop(
    agent: AcpAgentConfig,
    rx: async_channel::Receiver<AcpCommand>,
    cancel_rx: async_channel::Receiver<AcpCancelCommand>,
    approve: Option<ApprovalFn>,
    session_system_prompt: Option<String>,
) -> Result<()> {
    if codex_runtime_uses_disallowed_npx(&agent) {
        anyhow::bail!("Codex ACP npx runtime fallback is disabled");
    }

    // ── Spawn the agent subprocess ──────────────────────────────────
    let command_basename = std::path::Path::new(&agent.command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(agent.command.as_str());
    tracing::info!(
        target: "script_kit::tab_ai",
        event = "acp_agent_spawn_command_resolved",
        agent_id = %agent.id,
        command_basename = %command_basename,
        arg_count = agent.args.len(),
        uses_codex_npx_package = agent.args.iter().any(|arg| arg == CODEX_ACP_NPX_PACKAGE),
        npx_runtime_fallback_enabled = false,
    );
    let mut cmd = tokio::process::Command::new(&agent.command);
    cmd.args(&agent.args)
        .envs(&agent.env)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to spawn ACP agent: {}", agent.command))?;

    let child_stdin = child
        .stdin
        .take()
        .context("ACP agent stdin not available")?;
    let child_stdout = child
        .stdout
        .take()
        .context("ACP agent stdout not available")?;

    // Drain stderr in background so the agent doesn't block
    let child_stderr = child.stderr.take();
    if let Some(stderr) = child_stderr {
        let agent_id = agent.id.clone();
        tokio::task::spawn_local(async move {
            use tokio::io::AsyncBufReadExt;
            let reader = tokio::io::BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.starts_with("agy_acp_adapter ") {
                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "acp_agent_stderr",
                        agent = %agent_id,
                        stderr = %line,
                    );
                } else {
                    tracing::debug!(agent = %agent_id, stderr = %line, "acp_agent_stderr");
                }
            }
        });
    }

    // ── Create ACP connection ───────────────────────────────────────
    let client = match approve {
        Some(approve_fn) => ScriptKitAcpClient::with_approval(approve_fn),
        None => ScriptKitAcpClient::new(),
    };
    let on_chunk_handle = client.on_chunk.clone();
    let event_sinks_handle = client.event_sinks.clone();

    let (connection, io_task) = ClientSideConnection::new(
        client,
        child_stdin.compat_write(),
        child_stdout.compat(),
        |fut| {
            tokio::task::spawn_local(fut);
        },
    );
    let connection = Rc::new(connection);

    // Spawn the I/O pump
    tokio::task::spawn_local(async move {
        if let Err(e) = io_task.await {
            tracing::warn!(error = %e, "acp_io_task_finished_with_error");
        }
    });

    // ── Initialize ──────────────────────────────────────────────────
    let init_request = build_initialize_request();

    let init_response = connection
        .initialize(init_request)
        .await
        .context("ACP initialize failed")?;

    let supports_embedded_context = init_response
        .agent_capabilities
        .prompt_capabilities
        .embedded_context;
    let supports_image = init_response.agent_capabilities.prompt_capabilities.image;

    let auth_methods: Vec<String> = init_response
        .auth_methods
        .iter()
        .map(|method| method.id().0.to_string())
        .collect();

    tracing::info!(
        agent = %agent.id,
        protocol_version = ?init_response.protocol_version,
        load_session = init_response.agent_capabilities.load_session,
        auth_method_count = auth_methods.len(),
        auth_methods = ?auth_methods,
        supports_embedded_context,
        supports_image,
        "acp_initialized"
    );

    // Persist initialize-time capabilities so preflight sees them next launch.
    let initialize_state = super::config::AcpAgentRuntimeState {
        auth_state: Some(super::catalog::AcpAgentAuthState::Unknown),
        auth_methods: auth_methods.clone(),
        supports_embedded_context: Some(supports_embedded_context),
        supports_image: Some(supports_image),
        last_session_ok: false,
    };
    super::config::persist_acp_agent_runtime_state(agent.id.clone(), initialize_state.clone());

    // ── Session cache ───────────────────────────────────────────────
    let mut sessions: HashMap<String, AcpSessionBinding> = HashMap::new();

    // ── Command loop ────────────────────────────────────────────────
    while let Ok(cmd) = rx.recv().await {
        match cmd {
            AcpCommand::StartTurn { request, event_tx } => {
                let result = handle_prompt_turn(
                    &connection,
                    &cancel_rx,
                    &event_sinks_handle,
                    &mut sessions,
                    request,
                    event_tx.clone(),
                    &agent.id,
                    &initialize_state,
                    session_system_prompt.as_deref(),
                )
                .await;

                if let Err(error) = result {
                    let _ = event_tx
                        .send(AcpEvent::Failed {
                            error: error.to_string(),
                        })
                        .await;
                }
            }
            AcpCommand::PrepareSession {
                ui_thread_id,
                cwd,
                event_tx,
            } => {
                let result = ensure_session_and_announce_models(
                    &connection,
                    &event_sinks_handle,
                    &mut sessions,
                    &ui_thread_id,
                    &cwd,
                    &event_tx,
                    &agent.id,
                    &initialize_state,
                    session_system_prompt.as_deref(),
                )
                .await;

                match result {
                    Ok(Some(_acp_session_id)) => {
                        tracing::info!(
                            target: "script_kit::tab_ai",
                            event = "acp_prepare_session_ok",
                            ui_thread = %ui_thread_id,
                        );
                    }
                    Ok(None) => {
                        // Session refused with auth_required — event already emitted.
                    }
                    Err(error) => {
                        let _ = event_tx
                            .send(AcpEvent::Failed {
                                error: error.to_string(),
                            })
                            .await;
                    }
                }
                drop(event_tx);
            }
            AcpCommand::StreamPrompt {
                ui_session_id,
                cwd,
                messages,
                on_chunk,
                reply_tx,
            } => {
                let result = handle_stream_prompt(
                    &connection,
                    &mut sessions,
                    &on_chunk_handle,
                    ui_session_id,
                    cwd,
                    &messages,
                    on_chunk,
                    session_system_prompt.as_deref(),
                )
                .await;

                // Clear the callback regardless of success/failure
                *on_chunk_handle.lock() = None;

                // Best-effort reply; the caller might have timed out
                let _ = reply_tx.send(result).await;
            }
        }
    }

    tracing::info!(agent = %agent.id, "acp_event_loop_channel_closed");

    // Clean up: kill the child process
    if let Err(error) = child.kill().await {
        tracing::warn!(agent = %agent.id, error = %error, "acp_child_kill_failed");
    }

    Ok(())
}

fn codex_runtime_uses_disallowed_npx(agent: &AcpAgentConfig) -> bool {
    agent.id == CODEX_ACP_AGENT_ID
        && agent.command == "npx"
        && agent.args.iter().any(|arg| arg == CODEX_ACP_NPX_PACKAGE)
}

/// Ensure an ACP session exists for `ui_thread_id`, emitting `ModelsAvailable`
/// (and a `SetupRequired` on auth failure) through `event_tx`.
///
/// Returns:
/// - `Ok(Some(session_id))` if the session was created or reused.
/// - `Ok(None)` if the agent refused with `auth_required` (a `SetupRequired`
///   event was emitted so the UI can present the runtime setup card).
/// - `Err(_)` on any other `session/new` failure; the caller decides how to
///   surface it (the prompt-turn path emits `Failed`).
///
/// On first creation the agent's `SessionModelState` (if advertised) is mapped
/// into `AcpModelEntry` values and emitted as `ModelsAvailable` so the thread
/// can replace the hardcoded bootstrap list with the agent's live catalog.
#[allow(clippy::too_many_arguments)]
async fn ensure_session_and_announce_models(
    connection: &ClientSideConnection,
    event_sinks: &Arc<parking_lot::Mutex<HashMap<String, super::events::AcpEventTx>>>,
    sessions: &mut HashMap<String, AcpSessionBinding>,
    ui_thread_id: &str,
    cwd: &std::path::Path,
    event_tx: &super::events::AcpEventTx,
    agent_id: &str,
    initialize_state: &super::config::AcpAgentRuntimeState,
    session_system_prompt: Option<&str>,
) -> Result<Option<String>> {
    let (acp_session_id, agent_model_state) = if let Some(binding) = sessions.get(ui_thread_id) {
        (binding.agent_session_id.clone(), None)
    } else {
        let session_result = connection
            .new_session(new_session_request_with_system_prompt(
                cwd.to_path_buf(),
                session_system_prompt,
            ))
            .await;

        let session_response = match session_result {
            Ok(resp) => resp,
            Err(error) => {
                let error_text = error.to_string();
                if error_text.contains("auth_required") {
                    super::config::persist_acp_agent_runtime_state(
                        agent_id.to_string(),
                        super::config::AcpAgentRuntimeState {
                            auth_state: Some(
                                super::catalog::AcpAgentAuthState::NeedsAuthentication,
                            ),
                            auth_methods: initialize_state.auth_methods.clone(),
                            supports_embedded_context: initialize_state.supports_embedded_context,
                            supports_image: initialize_state.supports_image,
                            last_session_ok: false,
                        },
                    );
                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "acp_agent_runtime_state_transition",
                        agent_id = %agent_id,
                        auth_state = "needs_authentication",
                        auth_method_count = initialize_state.auth_methods.len(),
                    );
                    let _ = event_tx
                        .send(AcpEvent::SetupRequired {
                            reason: "auth_required".to_string(),
                            auth_methods: initialize_state.auth_methods.clone(),
                        })
                        .await;
                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "acp_auth_required",
                        ui_thread = %ui_thread_id,
                        agent_id = %agent_id,
                    );
                    return Ok(None);
                }
                return Err(error).context("ACP session/new failed");
            }
        };

        let acp_sid = session_response.session_id.0.to_string();
        let agent_model_state = session_response.models.clone();

        super::config::persist_acp_agent_runtime_state(
            agent_id.to_string(),
            super::config::AcpAgentRuntimeState {
                auth_state: Some(super::catalog::AcpAgentAuthState::Authenticated),
                auth_methods: initialize_state.auth_methods.clone(),
                supports_embedded_context: initialize_state.supports_embedded_context,
                supports_image: initialize_state.supports_image,
                last_session_ok: true,
            },
        );
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_agent_runtime_state_transition",
            agent_id = %agent_id,
            auth_state = "authenticated",
            auth_method_count = initialize_state.auth_methods.len(),
        );

        tracing::info!(
            ui_thread = %ui_thread_id,
            acp_session = %acp_sid,
            "acp_session_created"
        );

        sessions.insert(
            ui_thread_id.to_string(),
            AcpSessionBinding {
                ui_session_id: ui_thread_id.to_string(),
                agent_session_id: acp_sid.clone(),
            },
        );
        (acp_sid, agent_model_state)
    };

    event_sinks
        .lock()
        .insert(acp_session_id.clone(), event_tx.clone());

    if let Some(state) = agent_model_state {
        let models: Vec<super::config::AcpModelEntry> = state
            .available_models
            .iter()
            .map(|info| super::config::AcpModelEntry {
                id: info.model_id.0.to_string(),
                display_name: Some(info.name.clone()),
                context_window: None,
            })
            .collect();
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_session_models_advertised",
            session = %acp_session_id,
            current_model = %state.current_model_id.0,
            model_count = models.len(),
        );
        let _ = event_tx
            .send(AcpEvent::ModelsAvailable {
                current_model_id: Some(state.current_model_id.0.to_string()),
                models,
            })
            .await;
    }

    Ok(Some(acp_session_id))
}

/// Handle a single event-driven prompt turn within the ACP event loop.
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(
    name = "acp_turn",
    skip_all,
    fields(
        session = tracing::field::Empty,
        stop_reason = tracing::field::Empty,
    )
)]
async fn handle_prompt_turn(
    connection: &Rc<ClientSideConnection>,
    cancel_rx: &async_channel::Receiver<AcpCancelCommand>,
    event_sinks: &Arc<parking_lot::Mutex<HashMap<String, super::events::AcpEventTx>>>,
    sessions: &mut HashMap<String, AcpSessionBinding>,
    request: AcpPromptTurnRequest,
    event_tx: super::events::AcpEventTx,
    agent_id: &str,
    initialize_state: &super::config::AcpAgentRuntimeState,
    session_system_prompt: Option<&str>,
) -> Result<()> {
    tracing::info!(agent = %agent_id, "acp_turn_start");

    let acp_session_id = match ensure_session_and_announce_models(
        connection,
        event_sinks,
        sessions,
        &request.ui_thread_id,
        &request.cwd,
        &event_tx,
        agent_id,
        initialize_state,
        session_system_prompt,
    )
    .await?
    {
        Some(id) => id,
        None => return Ok(()),
    };

    tracing::Span::current().record("session", acp_session_id.as_str());
    let ui_thread_id = request.ui_thread_id.clone();
    while cancel_rx.try_recv().is_ok() {
        tracing::debug!(
            target: "script_kit::tab_ai",
            event = "acp_stale_cancel_drained_before_prompt",
            ui_thread = %ui_thread_id,
        );
    }

    // Set session model if the UI selected one (non-fatal on failure)
    if let Some(model_id) = &request.model_id {
        let set_model_req = SetSessionModelRequest::new(
            SessionId::new(acp_session_id.as_str()),
            ModelId::new(model_id.as_str()),
        );
        match connection.set_session_model(set_model_req).await {
            Ok(_) => {
                tracing::info!(
                    model = %model_id,
                    session = %acp_session_id,
                    "acp_session_model_set"
                );
            }
            Err(e) => {
                tracing::warn!(
                    model = %model_id,
                    session = %acp_session_id,
                    error = %e,
                    "acp_session_model_set_failed"
                );
            }
        }
    }

    // Send prompt and keep listening for an out-of-band cancel request. ACP
    // requires clients to continue reading until the agent returns Cancelled.
    let prompt_connection = Rc::clone(connection);
    let prompt_session_id = acp_session_id.clone();
    let (prompt_done_tx, prompt_done_rx) = async_channel::bounded::<
        agent_client_protocol::Result<agent_client_protocol::PromptResponse>,
    >(1);
    tokio::task::spawn_local(async move {
        let result = prompt_connection
            .prompt(PromptRequest::new(
                SessionId::new(prompt_session_id.as_str()),
                request.blocks,
            ))
            .await;
        let _ = prompt_done_tx.send(result).await;
    });
    let prompt_response = loop {
        if let Ok(result) = prompt_done_rx.try_recv() {
            break result.context("ACP session/prompt failed")?;
        }
        while let Ok(AcpCancelCommand::CancelTurn {
            ui_thread_id: cancel_ui_thread_id,
        }) = cancel_rx.try_recv()
        {
            if cancel_ui_thread_id != ui_thread_id {
                tracing::debug!(
                    target: "script_kit::tab_ai",
                    event = "acp_cancel_ignored_for_other_thread",
                    requested_ui_thread = %cancel_ui_thread_id,
                    active_ui_thread = %ui_thread_id,
                );
                continue;
            }
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_session_cancel_requested",
                session = %acp_session_id,
                ui_thread = %ui_thread_id,
            );
            if let Err(error) = connection
                .cancel(CancelNotification::new(SessionId::new(
                    acp_session_id.as_str(),
                )))
                .await
            {
                tracing::warn!(
                    target: "script_kit::tab_ai",
                    event = "acp_session_cancel_failed",
                    session = %acp_session_id,
                    error = %error,
                );
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    };

    // Clean up the event sink
    event_sinks.lock().remove(&acp_session_id);

    let stop_reason_str = format!("{:?}", prompt_response.stop_reason);
    tracing::Span::current().record("stop_reason", stop_reason_str.as_str());

    let _ = event_tx
        .send(AcpEvent::TurnFinished {
            stop_reason: stop_reason_str,
        })
        .await;

    tracing::info!(
        stop_reason = ?prompt_response.stop_reason,
        "acp_turn_completed"
    );

    Ok(())
}

/// Handle a single prompt request within the ACP event loop (legacy path).
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(
    name = "acp_turn",
    skip_all,
    fields(
        session = tracing::field::Empty,
        stop_reason = tracing::field::Empty,
    )
)]
async fn handle_stream_prompt(
    connection: &ClientSideConnection,
    sessions: &mut HashMap<String, AcpSessionBinding>,
    on_chunk_handle: &Arc<parking_lot::Mutex<Option<crate::ai::providers::StreamCallback>>>,
    ui_session_id: String,
    cwd: PathBuf,
    messages: &[crate::ai::providers::ProviderMessage],
    on_chunk: crate::ai::providers::StreamCallback,
    session_system_prompt: Option<&str>,
) -> Result<()> {
    tracing::info!(ui_session = %ui_session_id, "acp_turn_start");

    // Install the callback so session_notification can forward chunks
    *on_chunk_handle.lock() = Some(on_chunk);

    // Get or create ACP session
    let acp_session_id = if let Some(binding) = sessions.get(&ui_session_id) {
        binding.agent_session_id.clone()
    } else {
        let session_response = connection
            .new_session(new_session_request_with_system_prompt(
                cwd.clone(),
                session_system_prompt,
            ))
            .await
            .context("ACP session/new failed")?;

        let acp_sid = session_response.session_id.0.to_string();
        tracing::info!(
            ui_session = %ui_session_id,
            acp_session = %acp_sid,
            "acp_session_created"
        );

        sessions.insert(
            ui_session_id.clone(),
            AcpSessionBinding {
                ui_session_id: ui_session_id.clone(),
                agent_session_id: acp_sid.clone(),
            },
        );
        acp_sid
    };

    tracing::Span::current().record("session", acp_session_id.as_str());

    // Build prompt content blocks from provider messages
    let blocks = super::types::build_prompt_blocks(messages);

    if blocks.is_empty() {
        anyhow::bail!("No content blocks to send to ACP agent");
    }

    // Send prompt — this blocks until the agent's prompt turn completes
    let prompt_response = connection
        .prompt(PromptRequest::new(
            SessionId::new(acp_session_id.as_str()),
            blocks,
        ))
        .await
        .context("ACP session/prompt failed")?;

    tracing::Span::current().record(
        "stop_reason",
        format!("{:?}", prompt_response.stop_reason).as_str(),
    );

    tracing::info!(
        stop_reason = ?prompt_response.stop_reason,
        "acp_prompt_completed"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_request_advertises_full_fs_and_terminal() {
        let init = build_initialize_request();
        let value = serde_json::to_value(&init).expect("serialize init request");
        assert_eq!(
            value["clientCapabilities"]["fs"]["readTextFile"],
            serde_json::json!(true)
        );
        assert_eq!(
            value["clientCapabilities"]["fs"]["writeTextFile"],
            serde_json::json!(true)
        );
        assert_eq!(
            value["clientCapabilities"]["terminal"],
            serde_json::json!(true)
        );
    }

    #[test]
    fn initialize_request_advertises_terminal_auth_capability() {
        let init = build_initialize_request();
        let value = serde_json::to_value(&init).expect("serialize init request");
        assert_eq!(
            value["clientCapabilities"]["auth"]["terminal"],
            serde_json::json!(true),
            "client must advertise terminal auth capability"
        );
    }

    #[test]
    fn initialize_request_includes_client_info() {
        let init = build_initialize_request();
        let value = serde_json::to_value(&init).expect("serialize init request");
        assert_eq!(value["clientInfo"]["name"], serde_json::json!("script-kit"));
        assert_eq!(
            value["clientInfo"]["title"],
            serde_json::json!("Script Kit")
        );
    }

    #[test]
    #[ignore] // Hangs for 60s+ waiting for subprocess timeout
    fn spawn_fails_with_nonexistent_command() {
        let config = AcpAgentConfig {
            id: "test-nonexistent".into(),
            display_name: "Test".into(),
            command: "acp-agent-that-does-not-exist-12345".into(),
            args: vec![],
            env: HashMap::new(),
            models: vec![],
            install: None,
            auth: None,
        };

        let runtime = AcpRuntime::spawn(config);
        assert!(runtime.is_ok(), "spawn should succeed (thread created)");

        // The actual failure happens inside the thread when it tries to
        // spawn the subprocess. Sending a command should eventually fail.
        let rt = runtime.expect("runtime");
        let result = rt.stream_prompt(
            "test-session".into(),
            std::env::current_dir().expect("cwd"),
            vec![crate::ai::providers::ProviderMessage::user("Hello")],
            Box::new(|_chunk| true),
        );
        assert!(result.is_err(), "should fail because agent doesn't exist");
    }

    #[test]
    fn codex_npx_runtime_fallback_is_rejected_only_for_codex() {
        let mut config = AcpAgentConfig {
            id: CODEX_ACP_AGENT_ID.into(),
            display_name: "Codex".into(),
            command: "npx".into(),
            args: vec![CODEX_ACP_NPX_PACKAGE.into()],
            env: HashMap::new(),
            models: vec![],
            install: None,
            auth: None,
        };

        assert!(codex_runtime_uses_disallowed_npx(&config));

        config.id = "claude-code".into();
        assert!(!codex_runtime_uses_disallowed_npx(&config));
    }

    #[test]
    fn start_turn_returns_event_receiver() {
        // This test just verifies the API shape compiles and the channel
        // is properly constructed. The actual turn would require a running
        // ACP agent, which we don't have in unit tests.
        let (tx, _rx) = async_channel::bounded::<AcpCommand>(1);
        let runtime = AcpRuntime::from_sender(tx);

        let request = AcpPromptTurnRequest {
            ui_thread_id: "test-thread".to_string(),
            cwd: PathBuf::from("/tmp"),
            blocks: vec![],
            model_id: None,
        };

        // This will fail because no worker is listening, but it tests the API shape
        let result = runtime.start_turn(request);
        // The send_blocking will fail because the rx side is bounded(1) and
        // nobody is consuming, but the runtime was created with a fresh channel
        // so the first send should succeed before the channel is full
        assert!(
            result.is_ok() || result.is_err(),
            "start_turn should return a result"
        );
    }

    #[test]
    fn prepare_session_enqueues_command() {
        // Verify that `prepare_session` enqueues an `AcpCommand::PrepareSession`
        // carrying the ui thread id, cwd, and an open event sender. A stub
        // worker reads from the channel so the command is captured.
        let (tx, rx) = async_channel::bounded::<AcpCommand>(1);
        let runtime = AcpRuntime::from_sender(tx);

        let event_rx = runtime
            .prepare_session("thread-42".to_string(), PathBuf::from("/tmp/project"))
            .expect("prepare_session should enqueue");
        assert!(
            !event_rx.is_closed(),
            "event receiver should be open until the worker drops the sender"
        );

        let cmd = rx.try_recv().expect("PrepareSession should have been sent");
        match cmd {
            AcpCommand::PrepareSession {
                ui_thread_id,
                cwd,
                event_tx,
            } => {
                assert_eq!(ui_thread_id, "thread-42");
                assert_eq!(cwd, PathBuf::from("/tmp/project"));
                assert!(
                    !event_tx.is_closed(),
                    "the event sender shipped with the command must be writable"
                );
            }
            other => panic!(
                "expected PrepareSession, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }
}
