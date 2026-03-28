//! Client-side ACP handler.
//!
//! Implements the `agent_client_protocol::Client` trait so that the ACP
//! runtime can serve agent requests for file reads, permissions, and
//! session notifications.

use std::sync::Arc;

use agent_client_protocol::{
    Client, CreateTerminalRequest, CreateTerminalResponse, Error, ExtNotification, ExtRequest,
    ExtResponse, KillTerminalRequest, KillTerminalResponse, ReadTextFileRequest,
    ReadTextFileResponse, ReleaseTerminalRequest, ReleaseTerminalResponse,
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse, Result,
    SessionNotification, SessionUpdate, TerminalOutputRequest,
    TerminalOutputResponse, WaitForTerminalExitRequest, WaitForTerminalExitResponse,
    WriteTextFileRequest, WriteTextFileResponse,
};
use serde_json::value::RawValue;

use crate::ai::providers::StreamCallback;

/// Script Kit's ACP client handler.
///
/// Lives on the dedicated Tokio worker thread and is **not Send** (the ACP
/// crate's `Client` trait is `?Send`). Streaming chunks are forwarded to
/// the GPUI thread through the `on_chunk` callback which IS `Send + Sync`.
pub(crate) struct ScriptKitAcpClient {
    /// Current streaming callback. Set before each `session/prompt` call and
    /// cleared when the prompt completes. Wrapped in a parking_lot Mutex so
    /// that the async `session_notification` handler can borrow it.
    pub(crate) on_chunk: Arc<parking_lot::Mutex<Option<StreamCallback>>>,
}

impl ScriptKitAcpClient {
    pub(crate) fn new() -> Self {
        Self {
            on_chunk: Arc::new(parking_lot::Mutex::new(None)),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Client for ScriptKitAcpClient {
    // ── Required: permissions ───────────────────────────────────────────

    async fn request_permission(
        &self,
        args: RequestPermissionRequest,
    ) -> Result<RequestPermissionResponse> {
        tracing::warn!(
            tool_call_id = ?args.tool_call.tool_call_id,
            option_count = args.options.len(),
            "acp_permission_rejected_until_ui_bridge_lands"
        );
        Ok(RequestPermissionResponse::new(
            RequestPermissionOutcome::Cancelled,
        ))
    }

    // ── Required: session notifications ─────────────────────────────────

    async fn session_notification(&self, args: SessionNotification) -> Result<()> {
        match &args.update {
            SessionUpdate::AgentMessageChunk(chunk) => {
                if let agent_client_protocol::ContentBlock::Text(text) = &chunk.content {
                    let guard = self.on_chunk.lock();
                    if let Some(cb) = guard.as_ref() {
                        let _continue = cb(text.text.clone());
                        // If the callback returns false the UI wants to stop,
                        // but ACP cancellation is done via session/cancel which
                        // the caller handles — we just stop forwarding.
                    }
                }
            }
            SessionUpdate::AgentThoughtChunk(chunk) => {
                if let agent_client_protocol::ContentBlock::Text(text) = &chunk.content {
                    tracing::debug!(
                        session_id = %args.session_id.0,
                        thought_len = text.text.len(),
                        "acp_agent_thought"
                    );
                }
            }
            SessionUpdate::ToolCall(tc) => {
                tracing::info!(
                    session_id = %args.session_id.0,
                    tool_call_id = ?tc.tool_call_id,
                    title = %tc.title,
                    status = ?tc.status,
                    "acp_tool_call"
                );
            }
            SessionUpdate::ToolCallUpdate(tcu) => {
                tracing::debug!(
                    session_id = %args.session_id.0,
                    tool_call_id = ?tcu.tool_call_id,
                    "acp_tool_call_update"
                );
            }
            SessionUpdate::Plan(plan) => {
                tracing::info!(
                    session_id = %args.session_id.0,
                    entries = plan.entries.len(),
                    "acp_plan_received"
                );
            }
            SessionUpdate::CurrentModeUpdate(mode) => {
                tracing::info!(
                    session_id = %args.session_id.0,
                    mode = %mode.current_mode_id.0,
                    "acp_mode_change"
                );
            }
            SessionUpdate::AvailableCommandsUpdate(cmds) => {
                tracing::debug!(
                    session_id = %args.session_id.0,
                    count = cmds.available_commands.len(),
                    "acp_commands_update"
                );
            }
            _ => {
                tracing::trace!(
                    session_id = %args.session_id.0,
                    "acp_session_update_unhandled"
                );
            }
        }
        Ok(())
    }

    // ── Advertised: fs/read_text_file ──────────────────────────────────

    async fn read_text_file(&self, args: ReadTextFileRequest) -> Result<ReadTextFileResponse> {
        let path = &args.path;
        tracing::debug!(path = %path.display(), "acp_read_text_file");

        let content = std::fs::read_to_string(path).map_err(|e| {
            tracing::warn!(
                path = %path.display(),
                error = %e,
                "acp_read_text_file_failed"
            );
            Error::internal_error()
        })?;

        // Apply line/limit if specified
        let content = match (args.line, args.limit) {
            (Some(start_line), Some(limit)) => {
                let start = (start_line.saturating_sub(1)) as usize;
                content
                    .lines()
                    .skip(start)
                    .take(limit as usize)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            (Some(start_line), None) => {
                let start = (start_line.saturating_sub(1)) as usize;
                content
                    .lines()
                    .skip(start)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            (None, Some(limit)) => content
                .lines()
                .take(limit as usize)
                .collect::<Vec<_>>()
                .join("\n"),
            (None, None) => content,
        };

        Ok(ReadTextFileResponse::new(content))
    }

    // ── Not advertised: write/terminal methods return method_not_found ──

    async fn write_text_file(&self, _args: WriteTextFileRequest) -> Result<WriteTextFileResponse> {
        tracing::warn!("acp_write_text_file_rejected: not advertised in this cycle");
        Err(Error::method_not_found())
    }

    async fn create_terminal(
        &self,
        _args: CreateTerminalRequest,
    ) -> Result<CreateTerminalResponse> {
        Err(Error::method_not_found())
    }

    async fn terminal_output(
        &self,
        _args: TerminalOutputRequest,
    ) -> Result<TerminalOutputResponse> {
        Err(Error::method_not_found())
    }

    async fn release_terminal(
        &self,
        _args: ReleaseTerminalRequest,
    ) -> Result<ReleaseTerminalResponse> {
        Err(Error::method_not_found())
    }

    async fn wait_for_terminal_exit(
        &self,
        _args: WaitForTerminalExitRequest,
    ) -> Result<WaitForTerminalExitResponse> {
        Err(Error::method_not_found())
    }

    async fn kill_terminal(&self, _args: KillTerminalRequest) -> Result<KillTerminalResponse> {
        Err(Error::method_not_found())
    }

    async fn ext_method(&self, _args: ExtRequest) -> Result<ExtResponse> {
        let raw = RawValue::from_string("null".to_string()).map_err(|_| Error::internal_error())?;
        Ok(ExtResponse::new(Arc::from(raw)))
    }

    async fn ext_notification(&self, _args: ExtNotification) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_new_has_no_callback() {
        let client = ScriptKitAcpClient::new();
        assert!(client.on_chunk.lock().is_none());
    }
}
