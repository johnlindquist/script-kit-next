use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result};
use parking_lot::Mutex;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::oneshot;

use crate::ai::acp::events::AcpEventTx;
use crate::ai::agent_chat::events::{AgentChatEvent, AgentChatEventRx};
use crate::ai::agent_chat::runtime::{AgentChatConnection, AgentChatTurnRequest};

use super::events::map_rpc_line_to_events;
use super::protocol::{
    build_abort_command, build_get_available_models_command, build_prompt_command,
    build_prompt_payload, build_set_model_command, encode_json_line, parse_rpc_line,
    PiRpcLaunchSpec, PiRpcModelSelection, PiRpcResponse,
};

type PendingResponses = Arc<Mutex<HashMap<String, PendingResponse>>>;
type ActiveTurn = Arc<Mutex<Option<ActiveTurnState>>>;

enum PendingResponse {
    Events(AcpEventTx),
    Rpc(oneshot::Sender<PiRpcResponse>),
}

#[derive(Clone)]
struct ActiveTurnState {
    ui_thread_id: String,
    prompt_id: String,
    event_tx: AcpEventTx,
}

pub(crate) enum PiRpcRuntimeCommand {
    StartTurn {
        request: AgentChatTurnRequest,
        event_tx: AcpEventTx,
    },
    PrepareSession {
        ui_thread_id: String,
        cwd: std::path::PathBuf,
        event_tx: AcpEventTx,
    },
    CancelTurn {
        ui_thread_id: String,
    },
}

pub(crate) struct PiRpcRuntime {
    tx: async_channel::Sender<PiRpcRuntimeCommand>,
}

impl PiRpcRuntime {
    pub(crate) fn spawn(spec: PiRpcLaunchSpec) -> Result<Self> {
        let (tx, rx) = async_channel::bounded::<PiRpcRuntimeCommand>(8);

        std::thread::Builder::new()
            .name("pi-rpc-agent-chat".to_string())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        tracing::error!(%error, "pi_rpc_runtime_build_failed");
                        return;
                    }
                };

                runtime.block_on(async move {
                    if let Err(error) = run_pi_rpc_event_loop(spec, rx).await {
                        tracing::error!(%error, "pi_rpc_event_loop_exited_with_error");
                    }
                });
            })
            .context("Failed to spawn Pi RPC worker thread")?;

        Ok(Self { tx })
    }

    #[cfg(test)]
    pub(crate) fn from_sender(tx: async_channel::Sender<PiRpcRuntimeCommand>) -> Self {
        Self { tx }
    }
}

impl AgentChatConnection for PiRpcRuntime {
    fn start_turn(&self, request: AgentChatTurnRequest) -> Result<AgentChatEventRx> {
        let (event_tx, event_rx) = async_channel::bounded(256);
        self.tx
            .send_blocking(PiRpcRuntimeCommand::StartTurn { request, event_tx })
            .context("Pi RPC worker channel closed")?;
        Ok(event_rx)
    }

    fn cancel_turn(&self, ui_thread_id: String) -> Result<()> {
        self.tx
            .send_blocking(PiRpcRuntimeCommand::CancelTurn { ui_thread_id })
            .context("Pi RPC worker channel closed")
    }

    fn prepare_session(
        &self,
        ui_thread_id: String,
        cwd: std::path::PathBuf,
    ) -> Result<AgentChatEventRx> {
        let (event_tx, event_rx) = async_channel::bounded(8);
        self.tx
            .send_blocking(PiRpcRuntimeCommand::PrepareSession {
                ui_thread_id,
                cwd,
                event_tx,
            })
            .context("Pi RPC worker channel closed")?;
        Ok(event_rx)
    }
}

async fn run_pi_rpc_event_loop(
    spec: PiRpcLaunchSpec,
    rx: async_channel::Receiver<PiRpcRuntimeCommand>,
) -> Result<()> {
    let mut cmd = Command::new(&spec.command);
    cmd.args(&spec.args)
        .envs(&spec.env)
        .current_dir(&spec.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().context("Failed to spawn Pi RPC process")?;
    let mut stdin = child.stdin.take().context("Pi RPC stdin unavailable")?;
    let stdout = child.stdout.take().context("Pi RPC stdout unavailable")?;
    let stderr = child.stderr.take().context("Pi RPC stderr unavailable")?;

    let pending: PendingResponses = Arc::new(Mutex::new(HashMap::new()));
    let active_turn: ActiveTurn = Arc::new(Mutex::new(None));
    let stdout_pending = pending.clone();
    let stdout_active = active_turn.clone();

    tokio::spawn(async move {
        read_stdout(stdout, stdout_pending, stdout_active).await;
    });

    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            log_pi_rpc_stderr_line(&line);
        }
    });

    let mut counter = 0_u64;
    while let Ok(command) = rx.recv().await {
        counter += 1;
        match command {
            PiRpcRuntimeCommand::PrepareSession {
                ui_thread_id,
                cwd,
                event_tx,
            } => {
                tracing::debug!(
                    target: "script_kit::tab_ai",
                    event = "pi_rpc_prepare_session",
                    ui_thread_id = %ui_thread_id,
                    cwd = %cwd.display()
                );
                let id = format!("models-{counter}");
                pending
                    .lock()
                    .insert(id.clone(), PendingResponse::Events(event_tx));
                write_json(&mut stdin, &build_get_available_models_command(id)).await?;
            }
            PiRpcRuntimeCommand::StartTurn { request, event_tx } => {
                if let Some(model_id) = request.model_id.as_deref() {
                    let selection = match PiRpcModelSelection::parse(model_id) {
                        Ok(selection) => selection,
                        Err(error) => {
                            let _ = event_tx
                                .send(AgentChatEvent::Failed {
                                    error: format!("Invalid Pi model selection: {error}"),
                                })
                                .await;
                            continue;
                        }
                    };

                    let id = format!("set-model-{counter}");
                    match send_set_model_and_wait(&mut stdin, &pending, id, &selection).await {
                        Ok(()) => {}
                        Err(error) => {
                            let _ = event_tx
                                .send(AgentChatEvent::Failed {
                                    error: error.to_string(),
                                })
                                .await;
                            continue;
                        }
                    }
                }

                if request.cwd != spec.cwd {
                    tracing::debug!(
                        target: "script_kit::tab_ai",
                        event = "pi_rpc_cwd_mismatch",
                        requested_cwd = %request.cwd.display(),
                        launch_cwd = %spec.cwd.display(),
                        "Pi RPC runtime uses launch cwd for this connection"
                    );
                }

                let prompt_id = format!("prompt-{counter}");
                match build_prompt_payload(&request.blocks) {
                    Ok(payload) => {
                        active_turn.lock().replace(ActiveTurnState {
                            ui_thread_id: request.ui_thread_id,
                            prompt_id: prompt_id.clone(),
                            event_tx,
                        });
                        write_json(&mut stdin, &build_prompt_command(prompt_id, payload)).await?;
                    }
                    Err(error) => {
                        let _ = event_tx
                            .send(AgentChatEvent::Failed {
                                error: error.to_string(),
                            })
                            .await;
                    }
                }
            }
            PiRpcRuntimeCommand::CancelTurn { ui_thread_id } => {
                let active = active_turn.lock().clone();
                if let Some(active) = active.filter(|active| active.ui_thread_id == ui_thread_id) {
                    let id = format!("abort-{counter}");
                    write_json(&mut stdin, &build_abort_command(id)).await?;
                    tracing::debug!(
                        target: "script_kit::tab_ai",
                        event = "pi_rpc_abort_sent",
                        ui_thread_id = %ui_thread_id,
                        prompt_id = %active.prompt_id
                    );
                } else {
                    tracing::debug!(
                        target: "script_kit::tab_ai",
                        event = "pi_rpc_abort_ignored_no_active_turn",
                        ui_thread_id = %ui_thread_id
                    );
                }
            }
        }
    }

    let _ = child.kill().await;
    Ok(())
}

async fn write_json<W>(writer: &mut W, value: &serde_json::Value) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    writer
        .write_all(encode_json_line(value).as_bytes())
        .await
        .context("Failed to write Pi RPC command")?;
    writer
        .flush()
        .await
        .context("Failed to flush Pi RPC command")
}

async fn send_set_model_and_wait<W>(
    writer: &mut W,
    pending: &PendingResponses,
    id: String,
    selection: &PiRpcModelSelection,
) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    let (response_tx, response_rx) = oneshot::channel();
    pending
        .lock()
        .insert(id.clone(), PendingResponse::Rpc(response_tx));

    if let Err(error) = write_json(writer, &build_set_model_command(id.clone(), selection)).await {
        pending.lock().remove(&id);
        return Err(error);
    }

    let response = tokio::time::timeout(std::time::Duration::from_secs(10), response_rx)
        .await
        .context("Pi RPC set_model timed out")?
        .context("Pi RPC set_model response channel closed")?;

    if response.success {
        return Ok(());
    }

    anyhow::bail!(
        "{}",
        response
            .error
            .unwrap_or_else(|| "Pi RPC set_model failed".to_string())
    )
}

fn log_pi_rpc_stderr_line(line: &str) {
    tracing::debug!(
        target: "script_kit::tab_ai",
        event = "pi_rpc_stderr",
        line_chars = line.chars().count(),
        line_bytes = line.len(),
        "Pi RPC stderr line suppressed"
    );
}

async fn read_stdout<R>(stdout: R, pending: PendingResponses, active_turn: ActiveTurn)
where
    R: AsyncRead + Unpin,
{
    let mut lines = BufReader::new(stdout).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let parsed = match parse_rpc_line(&line) {
            Ok(parsed) => parsed,
            Err(error) => {
                send_to_active(
                    &active_turn,
                    AgentChatEvent::Failed {
                        error: format!("Invalid Pi RPC output: {error}"),
                    },
                )
                .await;
                continue;
            }
        };

        if let super::protocol::PiRpcLine::Response(response) = &parsed {
            if let Some(id) = response.id.as_ref() {
                let pending_response = pending.lock().remove(id);
                if let Some(pending_response) = pending_response {
                    match pending_response {
                        PendingResponse::Events(event_tx) => {
                            send_events(&event_tx, map_rpc_line_to_events(parsed)).await;
                        }
                        PendingResponse::Rpc(response_tx) => {
                            let _ = response_tx.send(response.clone());
                        }
                    }
                    continue;
                }
            }

            if response.command.as_deref() == Some("prompt") && !response.success {
                send_to_active(
                    &active_turn,
                    AgentChatEvent::Failed {
                        error: response
                            .error
                            .clone()
                            .unwrap_or_else(|| "Pi RPC prompt failed".to_string()),
                    },
                )
                .await;
            }
            continue;
        }

        let events = map_rpc_line_to_events(parsed);
        let closes_turn = events.iter().any(|event| {
            matches!(
                event,
                AgentChatEvent::TurnFinished { .. } | AgentChatEvent::Failed { .. }
            )
        });
        let event_tx = active_turn
            .lock()
            .as_ref()
            .map(|active| active.event_tx.clone());
        if let Some(event_tx) = event_tx {
            send_events(&event_tx, events).await;
        }
        if closes_turn {
            active_turn.lock().take();
        }
    }
}

async fn send_events(event_tx: &AcpEventTx, events: Vec<AgentChatEvent>) {
    let event_count = events.len();
    for event in events {
        let reveal_chunk = matches!(
            event,
            AgentChatEvent::AgentMessageDelta(_) | AgentChatEvent::AgentThoughtDelta(_)
        );
        let _ = event_tx.send(event).await;
        if reveal_chunk && event_count > 1 {
            tokio::time::sleep(std::time::Duration::from_millis(16)).await;
        }
    }
}

async fn send_to_active(active_turn: &ActiveTurn, event: AgentChatEvent) {
    let event_tx = active_turn
        .lock()
        .as_ref()
        .map(|active| active.event_tx.clone());
    if let Some(event_tx) = event_tx {
        let _ = event_tx.send(event).await;
    }
    active_turn.lock().take();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::pin::Pin;
    use std::task::{Context as TaskContext, Poll};

    struct RespondingWriter {
        bytes: Vec<u8>,
        pending: PendingResponses,
        id: String,
        success: bool,
        error: Option<String>,
        responded: bool,
    }

    impl RespondingWriter {
        fn new(pending: PendingResponses, success: bool, error: Option<String>) -> Self {
            Self {
                bytes: Vec::new(),
                pending,
                id: "set-model-test".to_string(),
                success,
                error,
                responded: false,
            }
        }

        fn written(&self) -> &[u8] {
            &self.bytes
        }
    }

    impl AsyncWrite for RespondingWriter {
        fn poll_write(
            mut self: Pin<&mut Self>,
            _cx: &mut TaskContext<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            self.bytes.extend_from_slice(buf);
            if !self.responded {
                self.responded = true;
                let pending_response = {
                    let id = self.id.clone();
                    self.pending.lock().remove(&id)
                };
                let Some(PendingResponse::Rpc(response_tx)) = pending_response else {
                    panic!("expected pending RPC response waiter");
                };
                response_tx
                    .send(PiRpcResponse {
                        id: Some(self.id.clone()),
                        command: Some("set_model".to_string()),
                        success: self.success,
                        data: None,
                        error: self.error.clone(),
                        raw: json!({
                            "type": "response",
                            "id": self.id.clone(),
                            "command": "set_model",
                            "success": self.success,
                        }),
                    })
                    .unwrap();
            }
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _cx: &mut TaskContext<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            _cx: &mut TaskContext<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    #[test]
    fn pi_rpc_runtime_implements_agent_chat_connection_trait() {
        fn accepts_connection(_: &dyn AgentChatConnection) {}
        let (tx, _rx) = async_channel::bounded::<PiRpcRuntimeCommand>(1);
        let runtime = PiRpcRuntime::from_sender(tx);
        accepts_connection(&runtime);
    }

    #[test]
    fn agent_chat_trait_start_turn_enqueues_pi_start_turn_command() {
        let (tx, rx) = async_channel::bounded::<PiRpcRuntimeCommand>(1);
        let runtime = PiRpcRuntime::from_sender(tx);

        let event_rx = runtime
            .start_turn(AgentChatTurnRequest {
                ui_thread_id: "thread-1".to_string(),
                cwd: std::path::PathBuf::from("/tmp"),
                blocks: Vec::new(),
                model_id: None,
            })
            .unwrap();
        drop(event_rx);

        let command = rx.recv_blocking().unwrap();
        assert!(matches!(command, PiRpcRuntimeCommand::StartTurn { .. }));
    }

    #[test]
    fn agent_chat_trait_prepare_session_enqueues_pi_prepare_session_command() {
        let (tx, rx) = async_channel::bounded::<PiRpcRuntimeCommand>(1);
        let runtime = PiRpcRuntime::from_sender(tx);

        let event_rx = runtime
            .prepare_session("thread-1".to_string(), std::path::PathBuf::from("/tmp"))
            .unwrap();
        drop(event_rx);

        let command = rx.recv_blocking().unwrap();
        assert!(matches!(
            command,
            PiRpcRuntimeCommand::PrepareSession { ui_thread_id, .. } if ui_thread_id == "thread-1"
        ));
    }

    #[test]
    fn agent_chat_trait_cancel_turn_enqueues_pi_cancel_command() {
        let (tx, rx) = async_channel::bounded::<PiRpcRuntimeCommand>(1);
        let runtime = PiRpcRuntime::from_sender(tx);

        runtime.cancel_turn("thread-1".to_string()).unwrap();

        let command = rx.recv_blocking().unwrap();
        assert!(matches!(
            command,
            PiRpcRuntimeCommand::CancelTurn { ui_thread_id } if ui_thread_id == "thread-1"
        ));
    }

    #[test]
    fn pi_rpc_stderr_logging_suppresses_raw_line_content() {
        let source = include_str!("runtime.rs");
        assert!(source.contains("fn log_pi_rpc_stderr_line"));
        assert!(source.contains("line_chars = line.chars().count()"));
        assert!(source.contains("line_bytes = line.len()"));
        assert!(!source.contains(&format!("{}{}", "line = %", "line")));
        assert!(!source.contains(&format!("{}{}", "line = ?", "line")));
    }

    #[test]
    fn set_model_wait_succeeds_only_after_pi_response() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let pending: PendingResponses = Arc::new(Mutex::new(HashMap::new()));
            let mut output = RespondingWriter::new(pending.clone(), true, None);
            let selection = PiRpcModelSelection {
                provider: "openai".to_string(),
                model_id: "gpt-5.4".to_string(),
            };

            send_set_model_and_wait(
                &mut output,
                &pending,
                "set-model-test".to_string(),
                &selection,
            )
            .await
            .unwrap();
            let written = String::from_utf8(output.written().to_vec()).unwrap();
            assert!(written.contains(r#""type":"set_model""#));
            assert!(written.contains(r#""provider":"openai""#));
            assert!(written.contains(r#""modelId":"gpt-5.4""#));
        });
    }

    #[test]
    fn set_model_wait_surfaces_pi_response_failure() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let pending: PendingResponses = Arc::new(Mutex::new(HashMap::new()));
            let mut output = RespondingWriter::new(
                pending.clone(),
                false,
                Some("model unavailable".to_string()),
            );
            let selection = PiRpcModelSelection {
                provider: "openai".to_string(),
                model_id: "missing-model".to_string(),
            };

            let result = send_set_model_and_wait(
                &mut output,
                &pending,
                "set-model-test".to_string(),
                &selection,
            )
            .await;
            let error = result.unwrap_err().to_string();
            assert!(error.contains("model unavailable"));
        });
    }
}
