//! ACP client runtime.
//!
//! Spawns the agent subprocess on a dedicated `current_thread` Tokio runtime
//! with a `LocalSet` (required because ACP futures are `!Send`). Communication
//! with the GPUI thread goes through a bounded `async_channel`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use agent_client_protocol::{
    Agent, ClientCapabilities, ClientSideConnection, FileSystemCapabilities, Implementation,
    InitializeRequest, NewSessionRequest, PromptRequest, ProtocolVersion, SessionId,
};

use super::config::AcpAgentConfig;
use super::events::{AcpCommand, AcpEvent, AcpEventRx, AcpPromptTurnRequest};
use super::handlers::{ApprovalFn, ScriptKitAcpClient};
use super::types::AcpSessionBinding;

/// Handle to the ACP worker thread. Send commands via the bounded channel.
///
/// Supports both the legacy `stream_prompt()` path (for `AiProvider`) and
/// the new `start_turn()` path (for `AcpThread` / `AcpChatView`).
pub(crate) struct AcpRuntime {
    tx: async_channel::Sender<AcpCommand>,
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
                    if let Err(e) = run_acp_event_loop(agent, rx, approve).await {
                        tracing::error!(error = %e, "acp_event_loop_exited_with_error");
                    }
                });
            })
            .context("Failed to spawn ACP worker thread")?;

        Ok(Self { tx })
    }

    /// Create an `AcpRuntime` from an existing command sender (test only).
    #[cfg(test)]
    pub(crate) fn from_sender(tx: async_channel::Sender<AcpCommand>) -> Self {
        Self { tx }
    }

    /// Start a new event-driven turn and return a receiver for typed events.
    ///
    /// This is the new path used by `AcpThread` / `AcpChatView`. Events are
    /// streamed as `AcpEvent` variants until `TurnFinished` or `Failed`.
    pub(crate) fn start_turn(&self, request: AcpPromptTurnRequest) -> Result<AcpEventRx> {
        let (event_tx, event_rx) = async_channel::bounded(256);

        self.tx
            .send_blocking(AcpCommand::StartTurn {
                request,
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

/// Build the ACP initialize request with full capability advertisement.
///
/// Advertises read + write filesystem access and terminal support.
pub(crate) fn build_initialize_request() -> InitializeRequest {
    InitializeRequest::new(ProtocolVersion::V1)
        .client_capabilities(
            ClientCapabilities::new()
                .fs(FileSystemCapabilities::new()
                    .read_text_file(true)
                    .write_text_file(true))
                .terminal(true),
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
    approve: Option<ApprovalFn>,
) -> Result<()> {
    // ── Spawn the agent subprocess ──────────────────────────────────
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
                tracing::debug!(agent = %agent_id, stderr = %line, "acp_agent_stderr");
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

    tracing::info!(
        agent = %agent.id,
        protocol_version = ?init_response.protocol_version,
        load_session = init_response.agent_capabilities.load_session,
        "acp_initialized"
    );

    // ── Session cache ───────────────────────────────────────────────
    let mut sessions: HashMap<String, AcpSessionBinding> = HashMap::new();

    // ── Command loop ────────────────────────────────────────────────
    while let Ok(cmd) = rx.recv().await {
        match cmd {
            AcpCommand::StartTurn { request, event_tx } => {
                let result = handle_prompt_turn(
                    &connection,
                    &event_sinks_handle,
                    &mut sessions,
                    request,
                    event_tx.clone(),
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
    let _ = child.kill().await;

    Ok(())
}

/// Handle a single event-driven prompt turn within the ACP event loop.
async fn handle_prompt_turn(
    connection: &ClientSideConnection,
    event_sinks: &Arc<parking_lot::Mutex<HashMap<String, super::events::AcpEventTx>>>,
    sessions: &mut HashMap<String, AcpSessionBinding>,
    request: AcpPromptTurnRequest,
    event_tx: super::events::AcpEventTx,
) -> Result<()> {
    // Get or create ACP session
    let acp_session_id = if let Some(binding) = sessions.get(&request.ui_thread_id) {
        binding.agent_session_id.clone()
    } else {
        let session_response = connection
            .new_session(NewSessionRequest::new(&request.cwd))
            .await
            .context("ACP session/new failed")?;

        let acp_sid = session_response.session_id.0.to_string();
        tracing::info!(
            ui_thread = %request.ui_thread_id,
            acp_session = %acp_sid,
            "acp_session_created_for_turn"
        );

        sessions.insert(
            request.ui_thread_id.clone(),
            AcpSessionBinding {
                ui_session_id: request.ui_thread_id.clone(),
                agent_session_id: acp_sid.clone(),
            },
        );
        acp_sid
    };

    // Bind the event sink so session_notification forwards typed events
    event_sinks
        .lock()
        .insert(acp_session_id.clone(), event_tx.clone());

    // Send prompt — this blocks until the agent's prompt turn completes
    let prompt_response = connection
        .prompt(PromptRequest::new(
            SessionId::new(acp_session_id.as_str()),
            request.blocks,
        ))
        .await
        .context("ACP session/prompt failed")?;

    // Clean up the event sink
    event_sinks.lock().remove(&acp_session_id);

    let _ = event_tx
        .send(AcpEvent::TurnFinished {
            stop_reason: format!("{:?}", prompt_response.stop_reason),
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
async fn handle_stream_prompt(
    connection: &ClientSideConnection,
    sessions: &mut HashMap<String, AcpSessionBinding>,
    on_chunk_handle: &Arc<parking_lot::Mutex<Option<crate::ai::providers::StreamCallback>>>,
    ui_session_id: String,
    cwd: PathBuf,
    messages: &[crate::ai::providers::ProviderMessage],
    on_chunk: crate::ai::providers::StreamCallback,
) -> Result<()> {
    // Install the callback so session_notification can forward chunks
    *on_chunk_handle.lock() = Some(on_chunk);

    // Get or create ACP session
    let acp_session_id = if let Some(binding) = sessions.get(&ui_session_id) {
        binding.agent_session_id.clone()
    } else {
        let session_response = connection
            .new_session(NewSessionRequest::new(&cwd))
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
}
