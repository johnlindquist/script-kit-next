//! Client-side ACP handler.
//!
//! Implements the `agent_client_protocol::Client` trait so that the ACP
//! runtime can serve agent requests for file reads/writes, terminal
//! operations, permissions, and session notifications.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use agent_client_protocol::{
    Client, CreateTerminalRequest, CreateTerminalResponse, Error, ExtNotification, ExtRequest,
    ExtResponse, KillTerminalRequest, KillTerminalResponse, PermissionOptionKind,
    ReadTextFileRequest, ReadTextFileResponse, ReleaseTerminalRequest, ReleaseTerminalResponse,
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse, Result,
    SelectedPermissionOutcome, SessionNotification, SessionUpdate, TerminalExitStatus, TerminalId,
    TerminalOutputRequest, TerminalOutputResponse, WaitForTerminalExitRequest,
    WaitForTerminalExitResponse, WriteTextFileRequest, WriteTextFileResponse,
};
use serde_json::value::RawValue;

use crate::ai::providers::StreamCallback;

// ── Approval seam ──────────────────────────────────────────────────────

/// Callback type for permission/approval decisions.
///
/// Parameters: `(title, body)`. Returns `Ok(true)` to approve, `Ok(false)`
/// to deny. In production this shows a GPUI confirm dialog; in tests it can
/// be a simple closure.
pub(crate) type ApprovalFn = Arc<dyn Fn(&str, &str) -> anyhow::Result<bool> + Send + Sync>;

/// Returns a default approval function that denies everything.
fn default_deny() -> ApprovalFn {
    Arc::new(|_title, _body| Ok(false))
}

// ── Terminal entry ─────────────────────────────────────────────────────

/// Tracks a spawned terminal process.
struct TerminalEntry {
    child: std::process::Child,
    output_buf: String,
    /// `None` while running, `Some(code)` once exited.
    exit_code: Option<Option<i32>>,
    /// Byte limit for output truncation. `None` means unlimited.
    output_byte_limit: Option<u64>,
}

impl TerminalEntry {
    /// Drain available stdout/stderr into `output_buf` without blocking.
    fn drain_output(&mut self) {
        use std::io::Read;

        // Read from stdout
        if let Some(stdout) = self.child.stdout.as_mut() {
            let mut buf = [0u8; 4096];
            loop {
                match stdout.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        self.output_buf
                            .push_str(&String::from_utf8_lossy(&buf[..n]));
                    }
                }
            }
        }

        // Read from stderr (interleaved into same buffer)
        if let Some(stderr) = self.child.stderr.as_mut() {
            let mut buf = [0u8; 4096];
            loop {
                match stderr.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        self.output_buf
                            .push_str(&String::from_utf8_lossy(&buf[..n]));
                    }
                }
            }
        }

        // Apply byte limit truncation
        if let Some(limit) = self.output_byte_limit {
            let limit = limit as usize;
            if self.output_buf.len() > limit {
                // Truncate from the beginning, keeping the tail
                let excess = self.output_buf.len() - limit;
                // Find character boundary at or after `excess`
                let boundary = self.output_buf.ceil_char_boundary(excess);
                self.output_buf.drain(..boundary);
            }
        }

        // Check if process exited
        if self.exit_code.is_none() {
            if let Ok(Some(status)) = self.child.try_wait() {
                self.exit_code = Some(status.code());
            }
        }
    }

    fn truncated(&self) -> bool {
        if let Some(limit) = self.output_byte_limit {
            // If we ever applied truncation, this is approximate
            self.output_buf.len() >= limit as usize
        } else {
            false
        }
    }
}

// ── Client handler ─────────────────────────────────────────────────────

/// Script Kit's ACP client handler.
///
/// Lives on the dedicated Tokio worker thread and is **not Send** (the ACP
/// crate's `Client` trait is `?Send`). Streaming chunks are forwarded to
/// the GPUI thread through the `on_chunk` callback which IS `Send + Sync`.
pub(crate) struct ScriptKitAcpClient {
    /// Current streaming callback.
    pub(crate) on_chunk: Arc<parking_lot::Mutex<Option<StreamCallback>>>,
    /// Injectable approval function for permission/write/terminal gates.
    approve: ApprovalFn,
    /// Registry of active terminal processes.
    terminals: std::cell::RefCell<HashMap<String, TerminalEntry>>,
    /// Counter for generating unique terminal IDs.
    next_terminal_id: std::cell::Cell<u64>,
}

impl ScriptKitAcpClient {
    /// Create a new client with default-deny approval behavior.
    pub(crate) fn new() -> Self {
        Self::with_approval(default_deny())
    }

    /// Create a new client with a custom approval function.
    pub(crate) fn with_approval(approve: ApprovalFn) -> Self {
        Self {
            on_chunk: Arc::new(parking_lot::Mutex::new(None)),
            approve,
            terminals: std::cell::RefCell::new(HashMap::new()),
            next_terminal_id: std::cell::Cell::new(1),
        }
    }

    /// Ask the approval seam for a decision.
    fn approve_or_deny(&self, title: &str, body: &str) -> anyhow::Result<bool> {
        (self.approve)(title, body)
    }

    /// Generate a unique terminal ID.
    fn next_id(&self) -> String {
        let id = self.next_terminal_id.get();
        self.next_terminal_id.set(id + 1);
        format!("term-{id}")
    }
}

#[async_trait::async_trait(?Send)]
impl Client for ScriptKitAcpClient {
    // ── Permissions ────────────────────────────────────────────────────

    async fn request_permission(
        &self,
        args: RequestPermissionRequest,
    ) -> Result<RequestPermissionResponse> {
        let tool_title = args
            .tool_call
            .fields
            .title
            .as_deref()
            .unwrap_or("unknown tool");
        let tool_id = args.tool_call.tool_call_id.0.as_ref();

        let body = format!(
            "Tool: {tool_title}\nTool call ID: {tool_id}\nOptions: {}",
            args.options
                .iter()
                .map(|o| o.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let approved = self
            .approve_or_deny("ACP permission request", &body)
            .map_err(|_| Error::internal_error())?;

        if approved {
            // Pick the first allow-style option, falling back to first option
            let selected = args
                .options
                .iter()
                .find(|o| {
                    matches!(
                        o.kind,
                        PermissionOptionKind::AllowOnce | PermissionOptionKind::AllowAlways
                    )
                })
                .or(args.options.first());

            match selected {
                Some(opt) => Ok(RequestPermissionResponse::new(
                    RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
                        opt.option_id.clone(),
                    )),
                )),
                None => Ok(RequestPermissionResponse::new(
                    RequestPermissionOutcome::Cancelled,
                )),
            }
        } else {
            tracing::info!(
                tool_call_id = tool_id,
                "acp_permission_denied_by_approval_seam"
            );
            Ok(RequestPermissionResponse::new(
                RequestPermissionOutcome::Cancelled,
            ))
        }
    }

    // ── Session notifications ──────────────────────────────────────────

    async fn session_notification(&self, args: SessionNotification) -> Result<()> {
        match &args.update {
            SessionUpdate::AgentMessageChunk(chunk) => {
                if let agent_client_protocol::ContentBlock::Text(text) = &chunk.content {
                    let guard = self.on_chunk.lock();
                    if let Some(cb) = guard.as_ref() {
                        let _continue = cb(text.text.clone());
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

    // ── fs/read_text_file ──────────────────────────────────────────────

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
                content.lines().skip(start).collect::<Vec<_>>().join("\n")
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

    // ── fs/write_text_file ─────────────────────────────────────────────

    async fn write_text_file(&self, args: WriteTextFileRequest) -> Result<WriteTextFileResponse> {
        let path_display = args.path.display().to_string();
        tracing::info!(path = %path_display, content_len = args.content.len(), "acp_write_text_file_request");

        let approved = self
            .approve_or_deny(
                "ACP file write request",
                &format!("Write {} bytes to {}", args.content.len(), path_display),
            )
            .map_err(|_| Error::internal_error())?;

        if !approved {
            tracing::info!(path = %path_display, "acp_write_text_file_denied");
            return Err(Error::internal_error());
        }

        // Ensure parent directory exists
        if let Some(parent) = args.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                tracing::warn!(path = %path_display, error = %e, "acp_write_mkdir_failed");
                Error::internal_error()
            })?;
        }

        std::fs::write(&args.path, &args.content).map_err(|e| {
            tracing::warn!(path = %path_display, error = %e, "acp_write_text_file_failed");
            Error::internal_error()
        })?;

        tracing::info!(path = %path_display, "acp_write_text_file_ok");
        Ok(WriteTextFileResponse::new())
    }

    // ── terminal/create ────────────────────────────────────────────────

    async fn create_terminal(&self, args: CreateTerminalRequest) -> Result<CreateTerminalResponse> {
        let cmd_display = format!("{} {}", args.command, args.args.join(" "));
        tracing::info!(command = %cmd_display, "acp_create_terminal_request");

        let approved = self
            .approve_or_deny(
                "ACP terminal request",
                &format!("terminal/create: {cmd_display}"),
            )
            .map_err(|_| Error::internal_error())?;

        if !approved {
            tracing::info!(command = %cmd_display, "acp_create_terminal_denied");
            return Err(Error::internal_error());
        }

        let mut cmd = std::process::Command::new(&args.command);
        cmd.args(&args.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(cwd) = &args.cwd {
            cmd.current_dir(cwd);
        }

        for env_var in &args.env {
            cmd.env(&env_var.name, &env_var.value);
        }

        let child = cmd.spawn().map_err(|e| {
            tracing::warn!(command = %cmd_display, error = %e, "acp_terminal_spawn_failed");
            Error::internal_error()
        })?;

        let tid = self.next_id();
        tracing::info!(terminal_id = %tid, command = %cmd_display, "acp_terminal_created");

        self.terminals.borrow_mut().insert(
            tid.clone(),
            TerminalEntry {
                child,
                output_buf: String::new(),
                exit_code: None,
                output_byte_limit: args.output_byte_limit,
            },
        );

        Ok(CreateTerminalResponse::new(TerminalId::new(tid)))
    }

    // ── terminal/output ────────────────────────────────────────────────

    async fn terminal_output(&self, args: TerminalOutputRequest) -> Result<TerminalOutputResponse> {
        let tid = args.terminal_id.0.as_ref();

        let mut terminals = self.terminals.borrow_mut();
        let entry = terminals.get_mut(tid).ok_or_else(|| {
            tracing::warn!(terminal_id = %tid, "acp_terminal_not_found");
            Error::internal_error()
        })?;

        entry.drain_output();

        let exit_status = entry
            .exit_code
            .map(|code| TerminalExitStatus::new().exit_code(code.map(|c| c as u32)));

        let mut resp = TerminalOutputResponse::new(&entry.output_buf, entry.truncated());
        if let Some(status) = exit_status {
            resp = resp.exit_status(status);
        }

        Ok(resp)
    }

    // ── terminal/wait_for_exit ─────────────────────────────────────────

    async fn wait_for_terminal_exit(
        &self,
        args: WaitForTerminalExitRequest,
    ) -> Result<WaitForTerminalExitResponse> {
        let tid = args.terminal_id.0.as_ref();

        let mut terminals = self.terminals.borrow_mut();
        let entry = terminals.get_mut(tid).ok_or_else(|| {
            tracing::warn!(terminal_id = %tid, "acp_terminal_not_found");
            Error::internal_error()
        })?;

        // Block until process exits
        let status = entry.child.wait().map_err(|e| {
            tracing::warn!(terminal_id = %tid, error = %e, "acp_terminal_wait_failed");
            Error::internal_error()
        })?;

        let code = status.code();
        entry.exit_code = Some(code);

        // Drain final output
        entry.drain_output();

        Ok(WaitForTerminalExitResponse::new(
            TerminalExitStatus::new().exit_code(code.map(|c| c as u32)),
        ))
    }

    // ── terminal/kill ──────────────────────────────────────────────────

    async fn kill_terminal(&self, args: KillTerminalRequest) -> Result<KillTerminalResponse> {
        let tid = args.terminal_id.0.as_ref();

        let mut terminals = self.terminals.borrow_mut();
        let entry = terminals.get_mut(tid).ok_or_else(|| {
            tracing::warn!(terminal_id = %tid, "acp_terminal_not_found");
            Error::internal_error()
        })?;

        if entry.exit_code.is_none() {
            let _ = entry.child.kill();
            let _ = entry.child.wait(); // Reap zombie
            entry.exit_code = Some(None); // Killed by signal
        }

        entry.drain_output();
        tracing::info!(terminal_id = %tid, "acp_terminal_killed");

        Ok(KillTerminalResponse::new())
    }

    // ── terminal/release ───────────────────────────────────────────────

    async fn release_terminal(
        &self,
        args: ReleaseTerminalRequest,
    ) -> Result<ReleaseTerminalResponse> {
        let tid = args.terminal_id.0.as_ref();

        let mut terminals = self.terminals.borrow_mut();
        let mut entry = terminals.remove(tid).ok_or_else(|| {
            tracing::warn!(terminal_id = %tid, "acp_terminal_not_found");
            Error::internal_error()
        })?;

        // Kill if still running
        if entry.exit_code.is_none() {
            let _ = entry.child.kill();
            let _ = entry.child.wait();
        }

        tracing::info!(terminal_id = %tid, "acp_terminal_released");
        Ok(ReleaseTerminalResponse::new())
    }

    // ── Extension points ───────────────────────────────────────────────

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

    /// Run an async test on a local-set tokio runtime (needed because
    /// the ACP Client trait is `!Send`).
    fn run_local<F: std::future::Future<Output = ()>>(f: F) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, f);
    }

    // ── Approval seam tests ────────────────────────────────────────────

    #[test]
    fn approval_hook_defaults_to_deny() {
        let client = ScriptKitAcpClient::new();
        let allowed = client
            .approve_or_deny("ACP permission request", "write /tmp/file.txt")
            .expect("approval hook should not error");
        assert!(!allowed);
    }

    #[test]
    fn approval_hook_can_be_overridden_for_verification() {
        let client = ScriptKitAcpClient::with_approval(Arc::new(|title, body| {
            assert_eq!(title, "ACP permission request");
            assert!(body.contains("terminal/create"));
            Ok(true)
        }));
        let allowed = client
            .approve_or_deny("ACP permission request", "terminal/create: git status")
            .expect("approval hook should not error");
        assert!(allowed);
    }

    #[test]
    fn client_new_has_no_callback() {
        let client = ScriptKitAcpClient::new();
        assert!(client.on_chunk.lock().is_none());
    }

    // ── Permission handler tests ───────────────────────────────────────

    #[test]
    fn request_permission_denied_by_default() {
        run_local(async {
            use agent_client_protocol::{
                PermissionOption, PermissionOptionId, PermissionOptionKind, SessionId, ToolCallId,
                ToolCallUpdate, ToolCallUpdateFields,
            };

            let client = ScriptKitAcpClient::new();

            let request = RequestPermissionRequest::new(
                SessionId::new("sess-1"),
                ToolCallUpdate::new(
                    ToolCallId::new("tc-1"),
                    ToolCallUpdateFields::new().title("Edit file".to_string()),
                ),
                vec![PermissionOption::new(
                    PermissionOptionId::new("allow-once"),
                    "Allow once",
                    PermissionOptionKind::AllowOnce,
                )],
            );

            let response = client.request_permission(request).await.expect("ok");
            assert_eq!(response.outcome, RequestPermissionOutcome::Cancelled);
        });
    }

    #[test]
    fn request_permission_approved_selects_allow_option() {
        run_local(async {
            use agent_client_protocol::{
                PermissionOption, PermissionOptionId, PermissionOptionKind, SessionId, ToolCallId,
                ToolCallUpdate, ToolCallUpdateFields,
            };

            let client = ScriptKitAcpClient::with_approval(Arc::new(|_, _| Ok(true)));

            let request = RequestPermissionRequest::new(
                SessionId::new("sess-1"),
                ToolCallUpdate::new(
                    ToolCallId::new("tc-2"),
                    ToolCallUpdateFields::new().title("Run command".to_string()),
                ),
                vec![
                    PermissionOption::new(
                        PermissionOptionId::new("reject"),
                        "Deny",
                        PermissionOptionKind::RejectOnce,
                    ),
                    PermissionOption::new(
                        PermissionOptionId::new("allow-once"),
                        "Allow once",
                        PermissionOptionKind::AllowOnce,
                    ),
                ],
            );

            let response = client.request_permission(request).await.expect("ok");
            match &response.outcome {
                RequestPermissionOutcome::Selected(selected) => {
                    assert_eq!(selected.option_id.0.as_ref(), "allow-once");
                }
                other => panic!("expected Selected, got {:?}", other),
            }
        });
    }

    // ── Write handler tests ────────────────────────────────────────────

    #[test]
    fn write_text_file_denied_by_default() {
        run_local(async {
            use agent_client_protocol::SessionId;

            let client = ScriptKitAcpClient::new();
            let request = WriteTextFileRequest::new(
                SessionId::new("sess-1"),
                "/tmp/acp-test-denied.txt",
                "hello",
            );
            let result = client.write_text_file(request).await;
            assert!(result.is_err(), "write should be denied by default");
        });
    }

    #[test]
    fn write_text_file_succeeds_when_approved() {
        run_local(async {
            use agent_client_protocol::SessionId;

            let client = ScriptKitAcpClient::with_approval(Arc::new(|_, _| Ok(true)));

            let dir = std::env::temp_dir().join("acp-handler-test");
            let _ = std::fs::create_dir_all(&dir);
            let path = dir.join("write-test.txt");

            let request =
                WriteTextFileRequest::new(SessionId::new("sess-1"), &path, "hello from ACP");
            let result = client.write_text_file(request).await;
            assert!(result.is_ok(), "write should succeed when approved");

            let content = std::fs::read_to_string(&path).expect("read back");
            assert_eq!(content, "hello from ACP");

            let _ = std::fs::remove_dir_all(&dir);
        });
    }

    // ── Terminal lifecycle tests ────────────────────────────────────────

    #[test]
    fn terminal_create_denied_by_default() {
        run_local(async {
            use agent_client_protocol::SessionId;

            let client = ScriptKitAcpClient::new();
            let request = CreateTerminalRequest::new(SessionId::new("sess-1"), "echo");
            let result = client.create_terminal(request).await;
            assert!(
                result.is_err(),
                "terminal create should be denied by default"
            );
        });
    }

    #[test]
    fn terminal_full_lifecycle() {
        run_local(async {
            use agent_client_protocol::SessionId;

            let client = ScriptKitAcpClient::with_approval(Arc::new(|_, _| Ok(true)));

            // Create terminal running `echo hello`
            let request = CreateTerminalRequest::new(SessionId::new("sess-1"), "echo")
                .args(vec!["hello-from-acp".to_string()]);
            let create_resp = client.create_terminal(request).await.expect("create ok");
            let tid = create_resp.terminal_id;

            // Wait for exit
            let wait_req = WaitForTerminalExitRequest::new(SessionId::new("sess-1"), tid.clone());
            let wait_resp = client
                .wait_for_terminal_exit(wait_req)
                .await
                .expect("wait ok");
            assert_eq!(wait_resp.exit_status.exit_code, Some(0));

            // Get output
            let output_req = TerminalOutputRequest::new(SessionId::new("sess-1"), tid.clone());
            let output_resp = client.terminal_output(output_req).await.expect("output ok");
            assert!(
                output_resp.output.contains("hello-from-acp"),
                "expected output to contain 'hello-from-acp', got: {}",
                output_resp.output
            );

            // Release
            let release_req = ReleaseTerminalRequest::new(SessionId::new("sess-1"), tid.clone());
            let release_resp = client.release_terminal(release_req).await;
            assert!(release_resp.is_ok());

            // After release, operations should fail
            let output_req2 = TerminalOutputRequest::new(SessionId::new("sess-1"), tid);
            let result = client.terminal_output(output_req2).await;
            assert!(result.is_err(), "output after release should fail");
        });
    }

    #[test]
    fn terminal_kill_then_output() {
        run_local(async {
            use agent_client_protocol::SessionId;

            let client = ScriptKitAcpClient::with_approval(Arc::new(|_, _| Ok(true)));

            // Create a long-running process
            let request = CreateTerminalRequest::new(SessionId::new("sess-1"), "sleep")
                .args(vec!["60".to_string()]);
            let create_resp = client.create_terminal(request).await.expect("create ok");
            let tid = create_resp.terminal_id;

            // Kill it
            let kill_req = KillTerminalRequest::new(SessionId::new("sess-1"), tid.clone());
            let kill_resp = client.kill_terminal(kill_req).await;
            assert!(kill_resp.is_ok());

            // Can still get output after kill (terminal not released)
            let output_req = TerminalOutputRequest::new(SessionId::new("sess-1"), tid.clone());
            let output_resp = client.terminal_output(output_req).await;
            assert!(output_resp.is_ok(), "output after kill should work");

            // Release
            let release_req = ReleaseTerminalRequest::new(SessionId::new("sess-1"), tid);
            let _ = client.release_terminal(release_req).await;
        });
    }

    #[test]
    fn terminal_not_found_returns_error() {
        run_local(async {
            use agent_client_protocol::SessionId;

            let client = ScriptKitAcpClient::new();

            let output_req = TerminalOutputRequest::new(
                SessionId::new("sess-1"),
                TerminalId::new("nonexistent-term"),
            );
            let result = client.terminal_output(output_req).await;
            assert!(result.is_err(), "nonexistent terminal should error");
        });
    }
}
