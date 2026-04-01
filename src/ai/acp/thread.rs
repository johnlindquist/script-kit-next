//! ACP thread entity.
//!
//! Owns one ACP conversation thread: messages, composer input, staged context
//! blocks, pending permission state, and the streaming event pump.
//!
//! Plain Tab stages context without sending a turn; the context blocks are
//! prepended to the first real user submit only. Quick-submit (Shift+Tab with
//! intent) populates `initial_input` and calls `submit_input()` after deferred
//! capture resolves.

use std::path::PathBuf;
use std::sync::Arc;

use agent_client_protocol::{ContentBlock, TextContent};
use gpui::{Context, SharedString, Task};

use super::{
    build_tab_ai_acp_context_blocks, build_tab_ai_acp_guidance_blocks, AcpApprovalRequest,
    AcpConnection, AcpEvent, AcpEventRx, AcpPromptTurnRequest,
};

/// Current status of the ACP thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpThreadStatus {
    /// No turn in progress; ready for input.
    Idle,
    /// Streaming events from the agent.
    Streaming,
    /// Blocked on a permission decision from the user.
    WaitingForPermission,
    /// The last turn failed.
    Error,
}

/// Role for a message in the thread history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpThreadMessageRole {
    User,
    Assistant,
    Thought,
    Tool,
    System,
    Error,
}

/// A single message in the thread history.
#[derive(Debug, Clone)]
pub(crate) struct AcpThreadMessage {
    pub id: u64,
    pub role: AcpThreadMessageRole,
    pub body: SharedString,
    /// Optional tool call ID linking this message to an `AcpToolCallState`.
    pub tool_call_id: Option<String>,
}

impl AcpThreadMessage {
    fn new(id: u64, role: AcpThreadMessageRole, body: impl Into<SharedString>) -> Self {
        Self {
            id,
            role,
            body: body.into(),
            tool_call_id: None,
        }
    }

    fn with_tool_call_id(
        id: u64,
        role: AcpThreadMessageRole,
        body: impl Into<SharedString>,
        tool_call_id: String,
    ) -> Self {
        Self {
            id,
            role,
            body: body.into(),
            tool_call_id: Some(tool_call_id),
        }
    }
}

/// Tracked state for a single tool call, kept in sync across start/update events.
#[derive(Debug, Clone)]
pub(crate) struct AcpToolCallState {
    /// ACP tool call identifier.
    pub tool_call_id: String,
    /// Display title (e.g. "Read file").
    pub title: String,
    /// Latest status text (e.g. "running", "completed").
    pub status: String,
    /// Latest body text (e.g. file contents, command output).
    pub body: Option<String>,
    /// ID of the corresponding `AcpThreadMessage` so the view can correlate.
    pub message_id: u64,
}

/// Initialization parameters for creating an `AcpThread`.
#[derive(Debug, Clone)]
pub(crate) struct AcpThreadInit {
    /// Unique identifier for this UI thread (used to map to ACP sessions).
    pub ui_thread_id: String,
    /// Working directory for the ACP session.
    pub cwd: PathBuf,
    /// Optional initial input text (populated by Shift+Tab quick-submit).
    pub initial_input: Option<String>,
}

/// GPUI entity that owns one ACP conversation thread.
///
/// Holds durable message history, staged context blocks (consumed once on
/// first submit), composer input, streaming status, and pending permission
/// requests. Binds stream and permission listeners via `cx.spawn(...)`.
pub(crate) struct AcpThread {
    connection: Arc<AcpConnection>,
    permission_rx: async_channel::Receiver<AcpApprovalRequest>,

    ui_thread_id: String,
    cwd: PathBuf,

    /// Thread message history (durable across turns).
    pub(crate) messages: Vec<AcpThreadMessage>,
    /// Current composer input text.
    pub(crate) input: SharedString,
    /// Current thread status.
    pub(crate) status: AcpThreadStatus,
    /// Pending permission request awaiting user decision.
    pub(crate) pending_permission: Option<AcpApprovalRequest>,

    /// Staged context blocks, prepended to the first user submit only.
    pending_context_blocks: Vec<ContentBlock>,
    /// Whether staged context has already been consumed.
    pending_context_consumed: bool,

    // ── Structured session state (readable by the view) ──────────
    /// Current plan entries from the latest `PlanUpdated` event.
    active_plan_entries: Vec<String>,
    /// Current agent mode from the latest `ModeChanged` event.
    active_mode_id: Option<String>,
    /// Current available commands from the latest `AvailableCommandsUpdated`.
    available_commands: Vec<String>,
    /// Tracked tool calls keyed by their ACP tool_call_id.
    active_tool_calls: Vec<AcpToolCallState>,

    /// Handle to the active stream pump task.
    stream_task: Option<Task<()>>,
    /// Handle to the permission listener task.
    permission_task: Option<Task<()>>,

    /// Monotonically increasing message ID counter.
    next_message_id: u64,
}

impl AcpThread {
    /// Create a new thread entity with optional initial input.
    ///
    /// Immediately binds the permission listener. Does NOT send an ACP turn —
    /// context is staged and only consumed on the first `submit_input()`.
    pub(crate) fn new(
        connection: Arc<AcpConnection>,
        permission_rx: async_channel::Receiver<AcpApprovalRequest>,
        init: AcpThreadInit,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut this = Self {
            connection,
            permission_rx,
            ui_thread_id: init.ui_thread_id,
            cwd: init.cwd,
            messages: Vec::new(),
            input: init.initial_input.unwrap_or_default().into(),
            status: AcpThreadStatus::Idle,
            pending_permission: None,
            pending_context_blocks: Vec::new(),
            pending_context_consumed: false,
            active_plan_entries: Vec::new(),
            active_mode_id: None,
            available_commands: Vec::new(),
            active_tool_calls: Vec::new(),
            stream_task: None,
            permission_task: None,
            next_message_id: 1,
        };
        this.bind_permission_listener(cx);
        this
    }

    /// Stage context blocks from a `TabAiContextBlob`.
    ///
    /// These blocks will be prepended to the first user submit only.
    /// Calling this again before a submit replaces the staged blocks.
    pub(crate) fn stage_context(
        &mut self,
        context: &crate::ai::TabAiContextBlob,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        self.pending_context_blocks = build_tab_ai_acp_context_blocks(context)?;
        self.pending_context_consumed = false;
        cx.notify();
        Ok(())
    }

    /// Update the composer input text.
    pub(crate) fn set_input(&mut self, value: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.input = value.into();
        cx.notify();
    }

    /// Submit the current input as a new user turn.
    ///
    /// Prepends staged context blocks on the first submit, then clears them.
    /// Starts streaming events from the ACP agent.
    pub(crate) fn submit_input(&mut self, cx: &mut Context<Self>) -> Result<(), String> {
        if matches!(
            self.status,
            AcpThreadStatus::Streaming | AcpThreadStatus::WaitingForPermission
        ) {
            return Ok(());
        }

        let input = self.input.to_string();
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        let blocks = self.prepare_turn_blocks(trimmed);

        let msg_id = self.alloc_id();
        self.messages.push(AcpThreadMessage::new(
            msg_id,
            AcpThreadMessageRole::User,
            trimmed.to_string(),
        ));
        self.input = SharedString::from("");
        self.status = AcpThreadStatus::Streaming;

        let rx = self
            .connection
            .start_turn(AcpPromptTurnRequest {
                ui_thread_id: self.ui_thread_id.clone(),
                cwd: self.cwd.clone(),
                blocks,
            })
            .map_err(|error| error.to_string())?;

        self.bind_stream(rx, cx);
        cx.notify();
        Ok(())
    }

    /// Resolve a pending permission request with the user's selection.
    ///
    /// Pass `None` for cancellation, or `Some(option_id)` for a selection.
    pub(crate) fn approve_pending_permission(
        &mut self,
        selected_option_id: Option<String>,
        cx: &mut Context<Self>,
    ) {
        if let Some(request) = self.pending_permission.take() {
            let _ = request.reply_tx.send_blocking(selected_option_id);
        }
        self.status = AcpThreadStatus::Idle;
        cx.notify();
    }

    // ── Private helpers ────────────────────────────────────────────

    /// Build the content blocks for a turn, consuming staged context on first use.
    #[cfg_attr(test, allow(dead_code))]
    pub(super) fn prepare_turn_blocks(&mut self, input: &str) -> Vec<ContentBlock> {
        let mut blocks = Vec::new();

        if !self.pending_context_consumed {
            blocks.extend(self.pending_context_blocks.clone());
            blocks.extend(build_tab_ai_acp_guidance_blocks(Some(input)));
            self.pending_context_consumed = true;
        }

        blocks.push(ContentBlock::Text(TextContent::new(input)));
        blocks
    }

    /// Spawn a task that pumps events from the ACP worker into thread state.
    fn bind_stream(&mut self, rx: AcpEventRx, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        self.stream_task = Some(cx.spawn(async move |_this, cx| {
            while let Ok(event) = rx.recv().await {
                let should_stop = matches!(
                    event,
                    AcpEvent::TurnFinished { .. } | AcpEvent::Failed { .. }
                );

                let entity_alive = entity.upgrade().is_some();
                if !entity_alive {
                    break;
                }

                let entity_ref = entity.clone();
                cx.update(|cx| {
                    if let Some(entity) = entity_ref.upgrade() {
                        entity.update(cx, |this, cx| {
                            this.apply_event(event, cx);
                        });
                    }
                });

                if should_stop {
                    break;
                }
            }
        }));
    }

    /// Spawn a long-lived task that listens for permission requests.
    fn bind_permission_listener(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        let rx = self.permission_rx.clone();
        self.permission_task = Some(cx.spawn(async move |_this, cx| {
            while let Ok(request) = rx.recv().await {
                let entity_alive = entity.upgrade().is_some();
                if !entity_alive {
                    break;
                }

                let entity_ref = entity.clone();
                cx.update(|cx| {
                    if let Some(entity) = entity_ref.upgrade() {
                        entity.update(cx, |this, cx| {
                            this.pending_permission = Some(request);
                            this.status = AcpThreadStatus::WaitingForPermission;
                            cx.notify();
                        });
                    }
                });
            }
        }));
    }

    /// Apply a single ACP event to thread state.
    ///
    /// Streaming text deltas coalesce into stable messages via `append_chunk`.
    /// Plan, mode, and command updates are persisted in dedicated fields so the
    /// view can render them as first-class UI strips without reparsing text.
    /// Tool calls are tracked by ID and updated in-place.
    fn apply_event(&mut self, event: AcpEvent, cx: &mut Context<Self>) {
        match event {
            AcpEvent::UserMessageDelta(chunk) => {
                self.append_chunk(AcpThreadMessageRole::System, chunk);
                self.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::AgentMessageDelta(chunk) => {
                self.append_chunk(AcpThreadMessageRole::Assistant, chunk);
                self.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::AgentThoughtDelta(chunk) => {
                self.append_chunk(AcpThreadMessageRole::Thought, chunk);
                self.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::ToolCallStarted {
                tool_call_id,
                title,
                status,
            } => {
                let msg_id = self.alloc_id();
                let body_text = format!("{title}\n{status}");
                self.messages.push(AcpThreadMessage::with_tool_call_id(
                    msg_id,
                    AcpThreadMessageRole::Tool,
                    body_text,
                    tool_call_id.clone(),
                ));
                self.active_tool_calls.push(AcpToolCallState {
                    tool_call_id,
                    title,
                    status,
                    body: None,
                    message_id: msg_id,
                });
                self.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::ToolCallUpdated {
                tool_call_id,
                status,
                body,
            } => {
                // Update the tracked tool call state and its message in-place.
                if let Some(tc) = self
                    .active_tool_calls
                    .iter_mut()
                    .find(|tc| tc.tool_call_id == tool_call_id)
                {
                    if let Some(s) = &status {
                        tc.status = s.clone();
                    }
                    if body.is_some() {
                        tc.body = body.clone();
                    }
                    // Rebuild the message body from the tracked state.
                    let new_body = Self::format_tool_call_body(&tc.title, &tc.status, &tc.body);
                    let mid = tc.message_id;
                    if let Some(msg) = self.messages.iter_mut().find(|m| m.id == mid) {
                        msg.body = new_body.into();
                    }
                } else {
                    // Orphan update — create a standalone message.
                    let text = match (status, body) {
                        (Some(s), Some(b)) => format!("{s}\n{b}"),
                        (Some(s), None) => s,
                        (None, Some(b)) => b,
                        (None, None) => String::new(),
                    };
                    if !text.is_empty() {
                        self.push_message(AcpThreadMessageRole::Tool, text);
                    }
                }
                self.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::PlanUpdated { entries } => {
                self.active_plan_entries = entries;
                self.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::AvailableCommandsUpdated { command_names } => {
                self.available_commands = command_names;
            }
            AcpEvent::ModeChanged { mode_id } => {
                self.active_mode_id = Some(mode_id);
            }
            AcpEvent::TurnFinished { .. } => {
                self.status = AcpThreadStatus::Idle;
            }
            AcpEvent::Failed { error } => {
                self.push_message(AcpThreadMessageRole::Error, error);
                self.status = AcpThreadStatus::Error;
            }
        }
        cx.notify();
    }

    /// Push a new message with an auto-allocated ID.
    fn push_message(&mut self, role: AcpThreadMessageRole, body: impl Into<SharedString>) {
        let id = self.alloc_id();
        self.messages.push(AcpThreadMessage::new(id, role, body));
    }

    /// Append a text chunk to the last message if it has the same role,
    /// otherwise create a new message. This coalesces streaming deltas.
    fn append_chunk(&mut self, role: AcpThreadMessageRole, chunk: String) {
        if let Some(last) = self.messages.last_mut() {
            if last.role == role {
                let mut text = last.body.to_string();
                text.push_str(&chunk);
                last.body = text.into();
                return;
            }
        }
        let id = self.alloc_id();
        self.messages
            .push(AcpThreadMessage::new(id, role, chunk));
    }

    /// Allocate a unique message ID.
    fn alloc_id(&mut self) -> u64 {
        let id = self.next_message_id;
        self.next_message_id += 1;
        id
    }

    /// Format a tool call message body from tracked state.
    fn format_tool_call_body(title: &str, status: &str, body: &Option<String>) -> String {
        match body {
            Some(b) => format!("{title}\n{status}\n{b}"),
            None => format!("{title}\n{status}"),
        }
    }

    // ── Public accessors for structured session state ──────────────

    /// Current plan entries from the latest `PlanUpdated` event.
    pub(crate) fn active_plan_entries(&self) -> &[String] {
        &self.active_plan_entries
    }

    /// Current agent mode ID (e.g. "code", "architect").
    pub(crate) fn active_mode_id(&self) -> Option<&str> {
        self.active_mode_id.as_deref()
    }

    /// Current available commands from the agent.
    pub(crate) fn available_commands(&self) -> &[String] {
        &self.available_commands
    }

    /// Tracked tool calls, ordered by creation.
    pub(crate) fn active_tool_calls(&self) -> &[AcpToolCallState] {
        &self.active_tool_calls
    }
}

/// Test-only helpers exposed to sibling modules in `src/ai/acp/`.
#[cfg(test)]
impl AcpThread {
    /// Build a test thread without a real connection or GPUI context.
    pub(super) fn test_new(
        context_blocks: Vec<ContentBlock>,
        initial_input: Option<String>,
    ) -> Self {
        let (_perm_tx, perm_rx) = async_channel::bounded(1);
        let (conn_tx, _conn_rx) = async_channel::bounded::<super::AcpCommand>(1);
        let dummy_connection = Arc::new(AcpConnection::from_sender(conn_tx));

        Self {
            connection: dummy_connection,
            permission_rx: perm_rx,
            ui_thread_id: "test-thread".to_string(),
            cwd: PathBuf::from("/tmp/test"),
            messages: Vec::new(),
            input: initial_input.unwrap_or_default().into(),
            status: AcpThreadStatus::Idle,
            pending_permission: None,
            pending_context_blocks: context_blocks,
            pending_context_consumed: false,
            active_plan_entries: Vec::new(),
            active_mode_id: None,
            available_commands: Vec::new(),
            active_tool_calls: Vec::new(),
            stream_task: None,
            permission_task: None,
            next_message_id: 1,
        }
    }

    /// Apply an event without a GPUI context (for testing pure logic).
    pub(super) fn apply_event_test(&mut self, event: super::AcpEvent) {
        match event {
            super::AcpEvent::UserMessageDelta(chunk) => {
                self.append_chunk(AcpThreadMessageRole::System, chunk);
                self.status = AcpThreadStatus::Streaming;
            }
            super::AcpEvent::AgentMessageDelta(chunk) => {
                self.append_chunk(AcpThreadMessageRole::Assistant, chunk);
                self.status = AcpThreadStatus::Streaming;
            }
            super::AcpEvent::AgentThoughtDelta(chunk) => {
                self.append_chunk(AcpThreadMessageRole::Thought, chunk);
                self.status = AcpThreadStatus::Streaming;
            }
            super::AcpEvent::ToolCallStarted {
                tool_call_id,
                title,
                status,
            } => {
                let msg_id = self.alloc_id();
                let body_text = format!("{title}\n{status}");
                self.messages.push(AcpThreadMessage::with_tool_call_id(
                    msg_id,
                    AcpThreadMessageRole::Tool,
                    body_text,
                    tool_call_id.clone(),
                ));
                self.active_tool_calls.push(AcpToolCallState {
                    tool_call_id,
                    title,
                    status,
                    body: None,
                    message_id: msg_id,
                });
                self.status = AcpThreadStatus::Streaming;
            }
            super::AcpEvent::ToolCallUpdated {
                tool_call_id,
                status,
                body,
            } => {
                if let Some(tc) = self
                    .active_tool_calls
                    .iter_mut()
                    .find(|tc| tc.tool_call_id == tool_call_id)
                {
                    if let Some(s) = &status {
                        tc.status = s.clone();
                    }
                    if body.is_some() {
                        tc.body = body.clone();
                    }
                    let new_body =
                        Self::format_tool_call_body(&tc.title, &tc.status, &tc.body);
                    let mid = tc.message_id;
                    if let Some(msg) = self.messages.iter_mut().find(|m| m.id == mid) {
                        msg.body = new_body.into();
                    }
                } else {
                    let text = match (status, body) {
                        (Some(s), Some(b)) => format!("{s}\n{b}"),
                        (Some(s), None) => s,
                        (None, Some(b)) => b,
                        (None, None) => String::new(),
                    };
                    if !text.is_empty() {
                        self.push_message(AcpThreadMessageRole::Tool, text);
                    }
                }
                self.status = AcpThreadStatus::Streaming;
            }
            super::AcpEvent::PlanUpdated { entries } => {
                self.active_plan_entries = entries;
                self.status = AcpThreadStatus::Streaming;
            }
            super::AcpEvent::AvailableCommandsUpdated { command_names } => {
                self.available_commands = command_names;
            }
            super::AcpEvent::ModeChanged { mode_id } => {
                self.active_mode_id = Some(mode_id);
            }
            super::AcpEvent::TurnFinished { .. } => {
                self.status = AcpThreadStatus::Idle;
            }
            super::AcpEvent::Failed { error } => {
                self.push_message(AcpThreadMessageRole::Error, error);
                self.status = AcpThreadStatus::Error;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to build an `AcpThread` without a real connection or GPUI context.
    /// Only for testing pure logic methods that don't need cx or connection.
    fn test_thread(
        pending_context_blocks: Vec<ContentBlock>,
        pending_context_consumed: bool,
    ) -> AcpThread {
        let (_perm_tx, perm_rx) = async_channel::bounded(1);
        // We create a dummy connection channel — tests that call prepare_turn_blocks
        // and append_chunk don't need a live connection.
        let (conn_tx, _conn_rx) = async_channel::bounded::<super::super::AcpCommand>(1);
        let dummy_connection = Arc::new(AcpConnection::from_sender(conn_tx));

        AcpThread {
            connection: dummy_connection,
            permission_rx: perm_rx,
            ui_thread_id: "test-thread".to_string(),
            cwd: PathBuf::from("."),
            messages: Vec::new(),
            input: SharedString::from(""),
            status: AcpThreadStatus::Idle,
            pending_permission: None,
            pending_context_blocks,
            pending_context_consumed,
            active_plan_entries: Vec::new(),
            active_mode_id: None,
            available_commands: Vec::new(),
            active_tool_calls: Vec::new(),
            stream_task: None,
            permission_task: None,
            next_message_id: 1,
        }
    }

    #[test]
    fn pending_context_is_only_consumed_once() {
        let mut thread = test_thread(
            vec![ContentBlock::Text(TextContent::new("context"))],
            false,
        );

        let first = thread.prepare_turn_blocks("hello");
        let second = thread.prepare_turn_blocks("again");

        // First turn: context block + user input = 2 blocks
        assert_eq!(first.len(), 2, "first turn should include context + input");

        // Second turn: only user input = 1 block
        assert_eq!(second.len(), 1, "second turn should only include input");
    }

    #[test]
    fn assistant_chunks_append_to_last_assistant_message() {
        let mut thread = test_thread(Vec::new(), true);

        thread.append_chunk(AcpThreadMessageRole::Assistant, "Hello".to_string());
        thread.append_chunk(AcpThreadMessageRole::Assistant, " world".to_string());

        assert_eq!(thread.messages.len(), 1, "chunks should coalesce");
        assert_eq!(
            thread.messages[0].body.to_string(),
            "Hello world",
            "chunks should be concatenated"
        );
    }

    #[test]
    fn chunks_of_different_roles_create_separate_messages() {
        let mut thread = test_thread(Vec::new(), true);

        thread.append_chunk(AcpThreadMessageRole::Assistant, "Hello".to_string());
        thread.append_chunk(AcpThreadMessageRole::Thought, "thinking...".to_string());
        thread.append_chunk(AcpThreadMessageRole::Assistant, "world".to_string());

        assert_eq!(
            thread.messages.len(),
            3,
            "different roles should create separate messages"
        );
    }

    #[test]
    fn prepare_turn_blocks_includes_guidance_for_authoring_intents() {
        let mut thread = test_thread(
            vec![ContentBlock::Text(TextContent::new("context"))],
            false,
        );

        let blocks = thread.prepare_turn_blocks("build a clipboard cleanup script");

        // context + guidance + input = 3 blocks
        assert_eq!(
            blocks.len(),
            3,
            "authoring intent should include context + guidance + input"
        );
    }

    #[test]
    fn prepare_turn_blocks_skips_guidance_for_non_authoring_intents() {
        let mut thread = test_thread(
            vec![ContentBlock::Text(TextContent::new("context"))],
            false,
        );

        let blocks = thread.prepare_turn_blocks("explain this selection");

        // context + input = 2 blocks (no guidance for non-authoring)
        assert_eq!(
            blocks.len(),
            2,
            "non-authoring intent should include context + input only"
        );
    }

    #[test]
    fn alloc_id_is_monotonically_increasing() {
        let mut thread = test_thread(Vec::new(), true);

        let id1 = thread.alloc_id();
        let id2 = thread.alloc_id();
        let id3 = thread.alloc_id();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn context_already_consumed_skips_on_first_turn() {
        let mut thread = test_thread(
            vec![ContentBlock::Text(TextContent::new("context"))],
            true, // already consumed
        );

        let blocks = thread.prepare_turn_blocks("hello");
        assert_eq!(blocks.len(), 1, "consumed context should not be prepended");
    }

    // ── Structured state tests ────────────────────────────────────

    /// Helper that applies an event without a GPUI context (for pure logic tests).
    /// Mirrors apply_event but skips cx.notify().
    fn apply_event_test(thread: &mut AcpThread, event: AcpEvent) {
        match event {
            AcpEvent::UserMessageDelta(chunk) => {
                thread.append_chunk(AcpThreadMessageRole::System, chunk);
                thread.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::AgentMessageDelta(chunk) => {
                thread.append_chunk(AcpThreadMessageRole::Assistant, chunk);
                thread.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::AgentThoughtDelta(chunk) => {
                thread.append_chunk(AcpThreadMessageRole::Thought, chunk);
                thread.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::ToolCallStarted {
                tool_call_id,
                title,
                status,
            } => {
                let msg_id = thread.alloc_id();
                let body_text = format!("{title}\n{status}");
                thread.messages.push(AcpThreadMessage::with_tool_call_id(
                    msg_id,
                    AcpThreadMessageRole::Tool,
                    body_text,
                    tool_call_id.clone(),
                ));
                thread.active_tool_calls.push(AcpToolCallState {
                    tool_call_id,
                    title,
                    status,
                    body: None,
                    message_id: msg_id,
                });
                thread.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::ToolCallUpdated {
                tool_call_id,
                status,
                body,
            } => {
                if let Some(tc) = thread
                    .active_tool_calls
                    .iter_mut()
                    .find(|tc| tc.tool_call_id == tool_call_id)
                {
                    if let Some(s) = &status {
                        tc.status = s.clone();
                    }
                    if body.is_some() {
                        tc.body = body.clone();
                    }
                    let new_body =
                        AcpThread::format_tool_call_body(&tc.title, &tc.status, &tc.body);
                    let mid = tc.message_id;
                    if let Some(msg) = thread.messages.iter_mut().find(|m| m.id == mid) {
                        msg.body = new_body.into();
                    }
                } else {
                    let text = match (status, body) {
                        (Some(s), Some(b)) => format!("{s}\n{b}"),
                        (Some(s), None) => s,
                        (None, Some(b)) => b,
                        (None, None) => String::new(),
                    };
                    if !text.is_empty() {
                        thread.push_message(AcpThreadMessageRole::Tool, text);
                    }
                }
                thread.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::PlanUpdated { entries } => {
                thread.active_plan_entries = entries;
                thread.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::AvailableCommandsUpdated { command_names } => {
                thread.available_commands = command_names;
            }
            AcpEvent::ModeChanged { mode_id } => {
                thread.active_mode_id = Some(mode_id);
            }
            AcpEvent::TurnFinished { .. } => {
                thread.status = AcpThreadStatus::Idle;
            }
            AcpEvent::Failed { error } => {
                thread.push_message(AcpThreadMessageRole::Error, error);
                thread.status = AcpThreadStatus::Error;
            }
        }
    }

    #[test]
    fn plan_updated_stores_in_dedicated_field() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AcpEvent::PlanUpdated {
                entries: vec!["Step 1".into(), "Step 2".into()],
            },
        );

        assert_eq!(thread.active_plan_entries(), &["Step 1", "Step 2"]);
        // Plan updates should not create messages — the view reads the field.
        assert!(
            thread.messages.is_empty(),
            "plan updates should not produce messages"
        );
    }

    #[test]
    fn mode_changed_stores_in_dedicated_field() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AcpEvent::ModeChanged {
                mode_id: "architect".into(),
            },
        );

        assert_eq!(thread.active_mode_id(), Some("architect"));
        assert!(
            thread.messages.is_empty(),
            "mode changes should not produce messages"
        );
    }

    #[test]
    fn available_commands_stores_in_dedicated_field() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AcpEvent::AvailableCommandsUpdated {
                command_names: vec!["plan".into(), "compact".into()],
            },
        );

        assert_eq!(thread.available_commands(), &["plan", "compact"]);
        assert!(
            thread.messages.is_empty(),
            "command updates should not produce messages"
        );
    }

    #[test]
    fn tool_call_started_creates_tracked_state_and_message() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AcpEvent::ToolCallStarted {
                tool_call_id: "tc-1".into(),
                title: "Read file".into(),
                status: "running".into(),
            },
        );

        assert_eq!(thread.active_tool_calls().len(), 1);
        assert_eq!(thread.active_tool_calls()[0].tool_call_id, "tc-1");
        assert_eq!(thread.active_tool_calls()[0].title, "Read file");
        assert_eq!(thread.active_tool_calls()[0].status, "running");

        assert_eq!(thread.messages.len(), 1);
        assert_eq!(thread.messages[0].role, AcpThreadMessageRole::Tool);
        assert_eq!(
            thread.messages[0].tool_call_id.as_deref(),
            Some("tc-1")
        );
    }

    #[test]
    fn tool_call_updated_modifies_existing_message_in_place() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AcpEvent::ToolCallStarted {
                tool_call_id: "tc-1".into(),
                title: "Read file".into(),
                status: "running".into(),
            },
        );

        apply_event_test(
            &mut thread,
            AcpEvent::ToolCallUpdated {
                tool_call_id: "tc-1".into(),
                status: Some("completed".into()),
                body: Some("file contents here".into()),
            },
        );

        // Should still be 1 message, updated in-place.
        assert_eq!(
            thread.messages.len(),
            1,
            "tool update should modify existing message, not create a new one"
        );

        let msg = &thread.messages[0];
        assert!(
            msg.body.contains("completed"),
            "message body should reflect updated status"
        );
        assert!(
            msg.body.contains("file contents here"),
            "message body should include updated body"
        );

        // Tracked state should also be updated.
        let tc = &thread.active_tool_calls()[0];
        assert_eq!(tc.status, "completed");
        assert_eq!(tc.body.as_deref(), Some("file contents here"));
    }

    #[test]
    fn orphan_tool_update_creates_standalone_message() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AcpEvent::ToolCallUpdated {
                tool_call_id: "unknown".into(),
                status: Some("done".into()),
                body: None,
            },
        );

        assert_eq!(
            thread.messages.len(),
            1,
            "orphan update should create a standalone message"
        );
        assert_eq!(thread.messages[0].body.as_ref(), "done");
    }

    #[test]
    fn turn_finished_does_not_create_message() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AcpEvent::TurnFinished {
                stop_reason: "end_turn".into(),
            },
        );

        assert!(
            thread.messages.is_empty(),
            "turn finished should not produce a message"
        );
        assert_eq!(thread.status, AcpThreadStatus::Idle);
    }

    #[test]
    fn failed_event_creates_error_message() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AcpEvent::Failed {
                error: "connection lost".into(),
            },
        );

        assert_eq!(thread.messages.len(), 1);
        assert_eq!(thread.messages[0].role, AcpThreadMessageRole::Error);
        assert_eq!(thread.messages[0].body.as_ref(), "connection lost");
        assert_eq!(thread.status, AcpThreadStatus::Error);
    }

    #[test]
    fn multiple_tool_calls_tracked_independently() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AcpEvent::ToolCallStarted {
                tool_call_id: "tc-1".into(),
                title: "Read file".into(),
                status: "running".into(),
            },
        );
        apply_event_test(
            &mut thread,
            AcpEvent::ToolCallStarted {
                tool_call_id: "tc-2".into(),
                title: "Write file".into(),
                status: "running".into(),
            },
        );

        // Update only tc-1.
        apply_event_test(
            &mut thread,
            AcpEvent::ToolCallUpdated {
                tool_call_id: "tc-1".into(),
                status: Some("completed".into()),
                body: None,
            },
        );

        assert_eq!(thread.active_tool_calls().len(), 2);
        assert_eq!(thread.active_tool_calls()[0].status, "completed");
        assert_eq!(thread.active_tool_calls()[1].status, "running");

        // Two messages, one per tool call.
        assert_eq!(thread.messages.len(), 2);
    }

    #[test]
    fn plan_updated_replaces_previous_plan() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AcpEvent::PlanUpdated {
                entries: vec!["Step 1".into()],
            },
        );
        apply_event_test(
            &mut thread,
            AcpEvent::PlanUpdated {
                entries: vec!["Step A".into(), "Step B".into()],
            },
        );

        assert_eq!(
            thread.active_plan_entries(),
            &["Step A", "Step B"],
            "plan should be fully replaced, not appended"
        );
    }
}
