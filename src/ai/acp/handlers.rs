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

use super::events::{AcpEvent, AcpEventTx};
use super::permission_broker::{
    approval_request_input, AcpApprovalOption, AcpApprovalPreview, AcpApprovalRequestInput,
};

// ── Tool call content summarization ───────────────────────────────────

fn summarize_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        other => serde_json::to_string_pretty(other).unwrap_or_else(|_| other.to_string()),
    }
}

fn summarize_tool_call_content(
    content: &[agent_client_protocol::ToolCallContent],
) -> Option<String> {
    let mut parts = Vec::new();
    for item in content {
        match item {
            agent_client_protocol::ToolCallContent::Content(content) => match &content.content {
                agent_client_protocol::ContentBlock::Text(text) => {
                    let text = text.text.trim();
                    if !text.is_empty() {
                        parts.push(text.to_string());
                    }
                }
                other => parts.push(format!("{other:?}")),
            },
            agent_client_protocol::ToolCallContent::Diff(diff) => {
                parts.push(format!("Diff: {}", diff.path.display()));
            }
            agent_client_protocol::ToolCallContent::Terminal(term) => {
                parts.push(format!("Terminal: {}", term.terminal_id.0.as_ref()));
            }
            _ => {}
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n\n"))
    }
}

fn summarize_tool_call_update(
    update: &agent_client_protocol::ToolCallUpdate,
) -> (Option<String>, Option<String>, Option<String>) {
    let mut body_parts = Vec::new();
    if let Some(content) = update.fields.content.as_ref() {
        if let Some(text) = summarize_tool_call_content(content) {
            body_parts.push(text);
        }
    }
    if let Some(raw_output) = update.fields.raw_output.as_ref() {
        body_parts.push(summarize_json_value(raw_output));
    }
    if let Some(raw_input) = update.fields.raw_input.as_ref() {
        body_parts.push(format!("Input:\n{}", summarize_json_value(raw_input)));
    }

    let body = if body_parts.is_empty() {
        None
    } else {
        Some(body_parts.join("\n\n"))
    };

    let title = update.fields.title.clone();
    let status = update
        .fields
        .status
        .as_ref()
        .map(|status| format!("{status:?}"));

    (title, status, body)
}

// ── Preview construction helpers ──────────────────────────────────────

fn truncate_for_overlay(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut truncated: String = text.chars().take(max_chars).collect();
    truncated.push_str("\n\u{2026}");
    truncated
}

fn guess_subject(raw_input: &serde_json::Value) -> Option<String> {
    let serde_json::Value::Object(map) = raw_input else {
        return None;
    };
    for key in ["path", "file_path", "cwd", "target", "command"] {
        if let Some(value) = map.get(key).and_then(|v| v.as_str()) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn build_permission_preview(
    args: &RequestPermissionRequest,
    options: &[AcpApprovalOption],
) -> AcpApprovalPreview {
    AcpApprovalPreview::new(
        args.tool_call
            .fields
            .title
            .as_deref()
            .unwrap_or("unknown tool"),
        args.tool_call.tool_call_id.0.as_ref(),
    )
    .with_subject(
        args.tool_call
            .fields
            .raw_input
            .as_ref()
            .and_then(guess_subject),
    )
    .with_summary(
        args.tool_call
            .fields
            .content
            .as_ref()
            .and_then(|content| summarize_tool_call_content(content))
            .map(|text| truncate_for_overlay(&text, 400)),
    )
    .with_input_preview(
        args.tool_call
            .fields
            .raw_input
            .as_ref()
            .map(|value| truncate_for_overlay(&summarize_json_value(value), 1200)),
    )
    .with_output_preview(
        args.tool_call
            .fields
            .raw_output
            .as_ref()
            .map(|value| truncate_for_overlay(&summarize_json_value(value), 1200)),
    )
    .with_options(options)
    .infer_kind()
}

// ── Shared option helpers ─────────────────────────────────────────────

fn allow_deny_once_options() -> Vec<AcpApprovalOption> {
    vec![
        AcpApprovalOption {
            option_id: "allow".to_string(),
            name: "Allow".to_string(),
            kind: "AllowOnce".to_string(),
        },
        AcpApprovalOption {
            option_id: "deny".to_string(),
            name: "Deny".to_string(),
            kind: "RejectOnce".to_string(),
        },
    ]
}

fn build_permission_request_input(
    args: &RequestPermissionRequest,
    options: Vec<AcpApprovalOption>,
) -> AcpApprovalRequestInput {
    let preview = build_permission_preview(args, &options);
    approval_request_input("ACP permission request", preview, options)
}

// ── Approval seam ──────────────────────────────────────────────────────

/// Callback type for permission/approval decisions.
///
/// Receives the full set of ACP permission options and returns
/// `Ok(Some(option_id))` for the selected option, or `Ok(None)` to cancel.
pub(crate) type ApprovalFn =
    Arc<dyn Fn(AcpApprovalRequestInput) -> anyhow::Result<Option<String>> + Send + Sync>;

/// Returns a default approval function that cancels everything.
fn default_deny() -> ApprovalFn {
    Arc::new(|_request| Ok(None))
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
///
/// Event sinks provide typed event streaming for the ACP chat view path.
pub(crate) struct ScriptKitAcpClient {
    /// Current streaming callback (legacy AiProvider path).
    pub(crate) on_chunk: Arc<parking_lot::Mutex<Option<StreamCallback>>>,
    /// Per-session event sinks for typed ACP event streaming.
    pub(crate) event_sinks: Arc<parking_lot::Mutex<HashMap<String, AcpEventTx>>>,
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
            event_sinks: Arc::new(parking_lot::Mutex::new(HashMap::new())),
            approve,
            terminals: std::cell::RefCell::new(HashMap::new()),
            next_terminal_id: std::cell::Cell::new(1),
        }
    }

    /// Bind an event sink for a specific ACP session.
    pub(crate) fn bind_event_sink(&self, session_id: &str, tx: AcpEventTx) {
        self.event_sinks.lock().insert(session_id.to_string(), tx);
    }

    /// Auto-approve operations that are safe without user confirmation.
    ///
    /// Approved automatically:
    /// - Any operation targeting paths within `~/.scriptkit/`
    /// - Read-only file operations (reading files, listing directories)
    /// - Read-only commands (ls, cat, head, tail, find, grep, wc, file, stat)
    fn auto_approve_operation(
        &self,
        args: &RequestPermissionRequest,
    ) -> Option<agent_client_protocol::PermissionOptionId> {
        let title = args
            .tool_call
            .fields
            .title
            .as_deref()
            .unwrap_or("")
            .to_lowercase();
        let raw_input = args.tool_call.fields.raw_input.as_ref();
        let input_str = raw_input.map(|v| v.to_string()).unwrap_or_default();

        let should_approve = 'check: {
            // 1. Operations on ~/.scriptkit paths
            let scriptkit_dir = dirs::home_dir()
                .map(|h| h.join(".scriptkit").to_string_lossy().to_string())
                .unwrap_or_default();
            if !scriptkit_dir.is_empty() && input_str.contains(&scriptkit_dir) {
                break 'check true;
            }

            // 2. Read-only file operations (title-based)
            if title.starts_with("read")
                || title.starts_with("view")
                || title.starts_with("list")
                || title.starts_with("search")
                || title.starts_with("find")
                || title.starts_with("cat ")
                || title.contains("read file")
                || title.contains("list dir")
                || title.contains("list files")
            {
                break 'check true;
            }

            // 3. Safe shell commands (read-only + common safe operations)
            if let Some(raw) = raw_input {
                let cmd_str = raw.to_string();
                let safe_prefixes = [
                    // Read-only commands
                    "\"ls ",
                    "\"ls\"",
                    "\"cat ",
                    "\"head ",
                    "\"tail ",
                    "\"find ",
                    "\"grep ",
                    "\"rg ",
                    "\"wc ",
                    "\"file ",
                    "\"stat ",
                    "\"which ",
                    "\"echo ",
                    "\"pwd\"",
                    "\"env\"",
                    "\"printenv",
                    // Safe creation commands
                    "\"mkdir ",
                    "\"touch ",
                    "\"date\"",
                    "\"uname",
                    // Development tools (safe in context)
                    "\"node ",
                    "\"bun ",
                    "\"npx ",
                    "\"npm ",
                    // Git read-only
                    "\"git status",
                    "\"git log",
                    "\"git diff",
                    "\"git branch",
                    "\"git show",
                ];
                for prefix in &safe_prefixes {
                    if cmd_str.contains(prefix) {
                        break 'check true;
                    }
                }
            }

            false
        };

        if !should_approve {
            return None;
        }

        // Pick the first "allow"-ish option (not reject/deny)
        args.options.iter().find_map(|option| {
            let id_str = option.option_id.0.as_ref();
            if id_str.contains("allow") || id_str.contains("approve") {
                Some(option.option_id.clone())
            } else {
                None
            }
        })
    }

    /// Remove the event sink for a specific ACP session.
    pub(crate) fn clear_event_sink(&self, session_id: &str) {
        self.event_sinks.lock().remove(session_id);
    }

    /// Emit a typed event to the sink bound to the given session.
    fn emit_event(&self, session_id: &agent_client_protocol::SessionId, event: AcpEvent) {
        if let Some(tx) = self.event_sinks.lock().get(session_id.0.as_ref()).cloned() {
            let _ = tx.send_blocking(event);
        }
    }

    /// Route a permission request through the approval function.
    fn choose_permission(
        &self,
        request: AcpApprovalRequestInput,
    ) -> anyhow::Result<Option<String>> {
        (self.approve)(request)
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
        let tool_id = args.tool_call.tool_call_id.0.as_ref();

        // Auto-allow safe operations (read-only, ~/.scriptkit paths)
        if let Some(auto_option) = self.auto_approve_operation(&args) {
            tracing::info!(
                tool_call_id = tool_id,
                option = auto_option.0.as_ref(),
                "acp_permission_auto_approved"
            );
            return Ok(RequestPermissionResponse::new(
                RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(auto_option)),
            ));
        }

        let options: Vec<AcpApprovalOption> = args
            .options
            .iter()
            .map(|option| AcpApprovalOption {
                option_id: option.option_id.0.as_ref().to_string(),
                name: option.name.to_string(),
                kind: format!("{:?}", option.kind),
            })
            .collect();

        let selected_id = self
            .choose_permission(build_permission_request_input(&args, options))
            .map_err(|_| Error::internal_error())?;

        let selected = selected_id.and_then(|id| {
            args.options
                .iter()
                .find(|option| option.option_id.0.as_ref() == id)
                .cloned()
        });

        match selected {
            Some(option) => Ok(RequestPermissionResponse::new(
                RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
                    option.option_id,
                )),
            )),
            None => {
                tracing::info!(
                    tool_call_id = tool_id,
                    "acp_permission_denied_by_approval_seam"
                );
                Ok(RequestPermissionResponse::new(
                    RequestPermissionOutcome::Cancelled,
                ))
            }
        }
    }

    // ── Session notifications ──────────────────────────────────────────

    async fn session_notification(&self, args: SessionNotification) -> Result<()> {
        match &args.update {
            SessionUpdate::UserMessageChunk(chunk) => {
                if let agent_client_protocol::ContentBlock::Text(text) = &chunk.content {
                    self.emit_event(
                        &args.session_id,
                        AcpEvent::UserMessageDelta(text.text.clone()),
                    );
                }
            }
            SessionUpdate::AgentMessageChunk(chunk) => {
                if let agent_client_protocol::ContentBlock::Text(text) = &chunk.content {
                    // Forward to legacy callback path
                    let guard = self.on_chunk.lock();
                    if let Some(cb) = guard.as_ref() {
                        let _continue = cb(text.text.clone());
                    }
                    // Forward to typed event sink
                    self.emit_event(
                        &args.session_id,
                        AcpEvent::AgentMessageDelta(text.text.clone()),
                    );
                }
            }
            SessionUpdate::AgentThoughtChunk(chunk) => {
                if let agent_client_protocol::ContentBlock::Text(text) = &chunk.content {
                    tracing::debug!(
                        session_id = %args.session_id.0,
                        thought_len = text.text.len(),
                        "acp_agent_thought"
                    );
                    self.emit_event(
                        &args.session_id,
                        AcpEvent::AgentThoughtDelta(text.text.clone()),
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
                self.emit_event(
                    &args.session_id,
                    AcpEvent::ToolCallStarted {
                        tool_call_id: tc.tool_call_id.0.as_ref().to_string(),
                        title: tc.title.to_string(),
                        status: format!("{:?}", tc.status),
                    },
                );
            }
            SessionUpdate::ToolCallUpdate(tcu) => {
                tracing::debug!(
                    session_id = %args.session_id.0,
                    tool_call_id = ?tcu.tool_call_id,
                    "acp_tool_call_update"
                );
                let (title, status, body) = summarize_tool_call_update(tcu);
                self.emit_event(
                    &args.session_id,
                    AcpEvent::ToolCallUpdated {
                        tool_call_id: tcu.tool_call_id.0.as_ref().to_string(),
                        title,
                        status,
                        body,
                    },
                );
            }
            SessionUpdate::Plan(plan) => {
                tracing::info!(
                    session_id = %args.session_id.0,
                    entries = plan.entries.len(),
                    "acp_plan_received"
                );
                self.emit_event(
                    &args.session_id,
                    AcpEvent::PlanUpdated {
                        entries: plan
                            .entries
                            .iter()
                            .map(|entry| entry.content.clone())
                            .collect(),
                    },
                );
            }
            SessionUpdate::CurrentModeUpdate(mode) => {
                tracing::info!(
                    session_id = %args.session_id.0,
                    mode = %mode.current_mode_id.0,
                    "acp_mode_change"
                );
                self.emit_event(
                    &args.session_id,
                    AcpEvent::ModeChanged {
                        mode_id: mode.current_mode_id.0.as_ref().to_string(),
                    },
                );
            }
            SessionUpdate::AvailableCommandsUpdate(cmds) => {
                tracing::debug!(
                    session_id = %args.session_id.0,
                    count = cmds.available_commands.len(),
                    "acp_commands_update"
                );
                self.emit_event(
                    &args.session_id,
                    AcpEvent::AvailableCommandsUpdated {
                        command_names: cmds
                            .available_commands
                            .iter()
                            .map(|command| command.name.to_string())
                            .collect(),
                    },
                );
            }
            SessionUpdate::UsageUpdate(usage) => {
                let cost_usd = usage.cost.as_ref().map(|c| c.amount);
                tracing::debug!(
                    session_id = %args.session_id.0,
                    used = usage.used,
                    size = usage.size,
                    cost_usd = ?cost_usd,
                    "acp_usage_update"
                );
                self.emit_event(
                    &args.session_id,
                    AcpEvent::UsageUpdated {
                        used_tokens: usage.used,
                        context_size: usage.size,
                        cost_usd,
                    },
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

        let write_options = allow_deny_once_options();
        let preview = AcpApprovalPreview::new("write_text_file", "client-fs-write")
            .with_kind(super::permission_broker::AcpApprovalPreviewKind::Write)
            .with_subject(Some(path_display.clone()))
            .with_summary(Some(format!("Write {} bytes", args.content.len())))
            .with_input_preview(Some(truncate_for_overlay(&args.content, 1200)))
            .with_options(&write_options);
        let selected = self
            .choose_permission(approval_request_input(
                "ACP file write request",
                preview,
                write_options,
            ))
            .map_err(|_| Error::internal_error())?;
        let approved = selected.is_some();

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

        let term_options = allow_deny_once_options();
        let preview = AcpApprovalPreview::new("terminal/create", "client-terminal-create")
            .with_kind(super::permission_broker::AcpApprovalPreviewKind::Execute)
            .with_subject(Some(cmd_display.clone()))
            .with_summary(Some(
                "Spawn a subprocess owned by the ACP client".to_string(),
            ))
            .with_options(&term_options);
        let selected = self
            .choose_permission(approval_request_input(
                "ACP terminal request",
                preview,
                term_options,
            ))
            .map_err(|_| Error::internal_error())?;
        let approved = selected.is_some();

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
    fn approval_hook_defaults_to_cancel() {
        let client = ScriptKitAcpClient::new();
        let result = client
            .choose_permission(AcpApprovalRequestInput {
                title: "ACP permission request".to_string(),
                body: "write /tmp/file.txt".to_string(),
                preview: None,
                options: vec![AcpApprovalOption {
                    option_id: "allow".to_string(),
                    name: "Allow".to_string(),
                    kind: "AllowOnce".to_string(),
                }],
            })
            .expect("approval hook should not error");
        assert!(result.is_none());
    }

    #[test]
    fn approval_hook_can_be_overridden_for_verification() {
        let client = ScriptKitAcpClient::with_approval(Arc::new(|input| {
            assert_eq!(input.title, "ACP permission request");
            assert!(input.body.contains("terminal/create"));
            // Return the first option ID
            Ok(input.options.first().map(|o| o.option_id.clone()))
        }));
        let result = client
            .choose_permission(AcpApprovalRequestInput {
                title: "ACP permission request".to_string(),
                body: "terminal/create: git status".to_string(),
                preview: None,
                options: vec![AcpApprovalOption {
                    option_id: "allow-once".to_string(),
                    name: "Allow once".to_string(),
                    kind: "AllowOnce".to_string(),
                }],
            })
            .expect("approval hook should not error");
        assert_eq!(result, Some("allow-once".to_string()));
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
    fn request_permission_approved_selects_exact_option() {
        run_local(async {
            use agent_client_protocol::{
                PermissionOption, PermissionOptionId, PermissionOptionKind, SessionId, ToolCallId,
                ToolCallUpdate, ToolCallUpdateFields,
            };

            // Approval function that picks the "allow-once" option by ID
            let client = ScriptKitAcpClient::with_approval(Arc::new(|input| {
                Ok(input
                    .options
                    .iter()
                    .find(|o| o.option_id == "allow-once")
                    .map(|o| o.option_id.clone()))
            }));

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

            let client = ScriptKitAcpClient::with_approval(Arc::new(|input| {
                Ok(input.options.first().map(|o| o.option_id.clone()))
            }));

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

            let client = ScriptKitAcpClient::with_approval(Arc::new(|input| {
                Ok(input.options.first().map(|o| o.option_id.clone()))
            }));

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

            let client = ScriptKitAcpClient::with_approval(Arc::new(|input| {
                Ok(input.options.first().map(|o| o.option_id.clone()))
            }));

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
