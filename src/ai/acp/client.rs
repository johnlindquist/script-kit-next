//! ACP client runtime.
//!
//! Spawns the agent subprocess on a dedicated `current_thread` Tokio runtime
//! with a `LocalSet` (required because ACP futures are `!Send`). Communication
//! with the GPUI thread goes through a bounded `async_channel`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use agent_client_protocol::{
    Agent, ClientCapabilities, ClientSideConnection, FileSystemCapabilities, Implementation,
    InitializeRequest, NewSessionRequest, PromptRequest, ProtocolVersion, SessionId,
};

use super::config::AcpAgentConfig;
use super::handlers::ScriptKitAcpClient;
use super::types::{build_prompt_blocks, AcpCommand, AcpSessionBinding};

/// Handle to the ACP worker thread. Send commands via the bounded channel.
pub(crate) struct AcpRuntime {
    tx: async_channel::Sender<AcpCommand>,
}

impl AcpRuntime {
    /// Spawn a new ACP worker thread for the given agent config.
    ///
    /// The worker owns a `current_thread` Tokio runtime, spawns the agent
    /// subprocess, completes the `initialize` handshake, and then loops
    /// waiting for `AcpCommand` messages.
    pub(crate) fn spawn(agent: AcpAgentConfig) -> Result<Self> {
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
                    if let Err(e) = run_acp_event_loop(agent, rx).await {
                        tracing::error!(error = %e, "acp_event_loop_exited_with_error");
                    }
                });
            })
            .context("Failed to spawn ACP worker thread")?;

        Ok(Self { tx })
    }

    /// Send a prompt to the ACP agent and block until the response completes.
    ///
    /// This is called from the `AiProvider::stream_message` synchronous path.
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

/// The main event loop running on the ACP worker thread.
///
/// 1. Spawns the agent subprocess.
/// 2. Creates a `ClientSideConnection` over stdin/stdout.
/// 3. Sends `initialize`.
/// 4. Loops, handling `AcpCommand` messages from the GPUI thread.
async fn run_acp_event_loop(
    agent: AcpAgentConfig,
    rx: async_channel::Receiver<AcpCommand>,
) -> Result<()> {
    // ── Spawn the agent subprocess ──────────────────────────────────
    let mut cmd = tokio::process::Command::new(&agent.command);
    cmd.args(&agent.args)
        .envs(&agent.env)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

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
    let client = ScriptKitAcpClient::new();
    let on_chunk_handle = client.on_chunk.clone();

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
    let init_request = InitializeRequest::new(ProtocolVersion::V1)
        .client_capabilities(
            ClientCapabilities::new()
                .fs(
                    FileSystemCapabilities::new()
                        .read_text_file(true)
                        .write_text_file(false),
                )
                .terminal(false),
        )
        .client_info(
            Implementation::new("script-kit", env!("CARGO_PKG_VERSION")).title("Script Kit"),
        );

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

/// Handle a single prompt request within the ACP event loop.
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
    let blocks = build_prompt_blocks(messages);

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
}
