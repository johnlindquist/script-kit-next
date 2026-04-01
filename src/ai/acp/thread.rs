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
}

impl AcpThreadMessage {
    fn new(id: u64, role: AcpThreadMessageRole, body: impl Into<SharedString>) -> Self {
        Self {
            id,
            role,
            body: body.into(),
        }
    }
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
    fn prepare_turn_blocks(&mut self, input: &str) -> Vec<ContentBlock> {
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
            AcpEvent::ToolCallStarted { title, status, .. } => {
                self.push_message(AcpThreadMessageRole::Tool, format!("{title}\n{status}"));
                self.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::ToolCallUpdated { status, body, .. } => {
                let text = match (status, body) {
                    (Some(s), Some(b)) => format!("{s}\n{b}"),
                    (Some(s), None) => s,
                    (None, Some(b)) => b,
                    (None, None) => String::new(),
                };
                if !text.is_empty() {
                    self.push_message(AcpThreadMessageRole::Tool, text);
                }
                self.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::PlanUpdated { entries } => {
                self.push_message(AcpThreadMessageRole::System, entries.join("\n"));
                self.status = AcpThreadStatus::Streaming;
            }
            AcpEvent::AvailableCommandsUpdated { command_names } => {
                self.push_message(
                    AcpThreadMessageRole::System,
                    format!("Commands: {}", command_names.join(", ")),
                );
            }
            AcpEvent::ModeChanged { mode_id } => {
                self.push_message(AcpThreadMessageRole::System, format!("Mode: {mode_id}"));
            }
            AcpEvent::TurnFinished { stop_reason } => {
                self.push_message(
                    AcpThreadMessageRole::System,
                    format!("Stop reason: {stop_reason}"),
                );
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
        let dummy_connection = Arc::new(AcpConnection {
            tx: conn_tx,
        });

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
}
