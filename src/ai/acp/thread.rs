//! ACP thread entity.
//!
//! Owns one ACP conversation thread: messages, composer input, staged context
//! blocks, pending permission state, and the streaming event pump.
//!
//! Plain Tab stages context without sending a turn; the context blocks are
//! prepended to the first real user submit only. Quick-submit (Shift+Tab with
//! intent) populates `initial_input` and calls `submit_input()` after deferred
//! capture resolves.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use agent_client_protocol::{ContentBlock, TextContent};
use gpui::{Context, SharedString, Task};

use crate::components::text_input::TextInputState;

use super::{
    build_tab_ai_acp_context_blocks, AcpApprovalRequest, AcpConnection, AcpEvent, AcpEventRx,
    AcpPromptTurnRequest,
};

/// Bootstrap state for deferred context capture.
///
/// Tracks whether the Tab AI context has been assembled and staged on the
/// thread, so that the first user submit can be gated behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpContextBootstrapState {
    /// Context capture is still running in the background.
    Preparing,
    /// Context has been staged successfully.
    Ready,
    /// Context staging failed; partial or no context is available.
    Failed,
}

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
    /// Index into `AcpThread::messages` for O(1) message lookup.
    message_index: usize,
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
    /// Display name for the agent (shown in toolbar, e.g. "Claude Code").
    pub display_name: SharedString,
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
    /// Display name for the agent (shown in toolbar).
    display_name: SharedString,

    /// Thread message history (durable across turns).
    pub(crate) messages: Vec<AcpThreadMessage>,
    /// Current composer input state (with cursor, selection, undo/redo).
    pub(crate) input: TextInputState,
    /// Current thread status.
    pub(crate) status: AcpThreadStatus,
    /// Pending permission request awaiting user decision.
    pub(crate) pending_permission: Option<AcpApprovalRequest>,

    /// Staged context blocks, prepended to the first user submit only.
    pending_context_blocks: Vec<ContentBlock>,
    /// Whether staged context has already been consumed.
    pending_context_consumed: bool,

    /// Whether the deferred context capture has completed.
    context_bootstrap_state: AcpContextBootstrapState,
    /// Whether a submit was attempted while context was still `Preparing`.
    queued_submit_while_bootstrapping: bool,
    /// Human-readable status note for the bootstrap phase.
    context_bootstrap_note: Option<SharedString>,

    // ── Structured session state (readable by the view) ──────────
    /// Current plan entries from the latest `PlanUpdated` event.
    active_plan_entries: Vec<String>,
    /// Current agent mode from the latest `ModeChanged` event.
    active_mode_id: Option<String>,
    /// Current available commands from the latest `AvailableCommandsUpdated`.
    available_commands: Vec<String>,
    /// Tracked tool calls keyed by their ACP tool_call_id.
    active_tool_calls: Vec<AcpToolCallState>,
    /// O(1) lookup from tool_call_id to index in `active_tool_calls`.
    tool_call_lookup: HashMap<String, usize>,

    /// When the current streaming turn started (for elapsed time display).
    stream_started_at: Option<std::time::Instant>,

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
            display_name: init.display_name,
            messages: Vec::new(),
            input: match init.initial_input {
                Some(text) if !text.is_empty() => TextInputState::with_text(text),
                _ => TextInputState::new(),
            },
            status: AcpThreadStatus::Idle,
            pending_permission: None,
            pending_context_blocks: Vec::new(),
            pending_context_consumed: false,
            context_bootstrap_state: AcpContextBootstrapState::Preparing,
            queued_submit_while_bootstrapping: false,
            context_bootstrap_note: Some(
                "Attaching selection, window, and clipboard context\u{2026}".into(),
            ),
            active_plan_entries: Vec::new(),
            active_mode_id: None,
            available_commands: Vec::new(),
            active_tool_calls: Vec::new(),
            tool_call_lookup: HashMap::new(),
            stream_started_at: None,
            stream_task: None,
            permission_task: None,
            next_message_id: 1,
        };
        this.bind_permission_listener(cx);
        this
    }

    /// Stage context blocks from a `TabAiContextBlob`.
    ///
    /// Marks bootstrap as `Ready` and auto-submits any queued first turn.
    /// Calling this again before a submit replaces the staged blocks.
    pub(crate) fn stage_context(
        &mut self,
        context: &crate::ai::TabAiContextBlob,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        self.pending_context_blocks = build_tab_ai_acp_context_blocks(context)?;
        self.pending_context_consumed = false;
        self.context_bootstrap_state = AcpContextBootstrapState::Ready;
        self.context_bootstrap_note = Some("Context attached".into());

        let should_auto_submit = self.queued_submit_while_bootstrapping
            && !self.input.text().trim().is_empty()
            && !matches!(
                self.status,
                AcpThreadStatus::Streaming | AcpThreadStatus::WaitingForPermission
            );
        self.queued_submit_while_bootstrapping = false;

        if should_auto_submit {
            return self.submit_input(cx);
        }

        cx.notify();
        Ok(())
    }

    /// Mark the context bootstrap as failed with a human-readable note.
    ///
    /// If a submit was queued, it proceeds anyway (partial context is better
    /// than dropping user input).
    pub(crate) fn mark_context_bootstrap_failed(
        &mut self,
        note: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.context_bootstrap_state = AcpContextBootstrapState::Failed;
        self.context_bootstrap_note = Some(note.into());

        let should_auto_submit = self.queued_submit_while_bootstrapping
            && !self.input.text().trim().is_empty()
            && !matches!(
                self.status,
                AcpThreadStatus::Streaming | AcpThreadStatus::WaitingForPermission
            );
        self.queued_submit_while_bootstrapping = false;

        if should_auto_submit {
            let _ = self.submit_input(cx);
        } else {
            cx.notify();
        }
    }

    /// Update the composer input text (replaces entire content, cursor at end).
    pub(crate) fn set_input(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        self.input.set_text(value);
        cx.notify();
    }

    /// Submit the current input as a new user turn.
    ///
    /// If context is still bootstrapping (`Preparing`), the submit is queued
    /// and will fire automatically when `stage_context()` or
    /// `mark_context_bootstrap_failed()` completes.
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

        let input = self.input.text().to_string();
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        // Gate on bootstrap: queue instead of sending while context is still preparing.
        if matches!(
            self.context_bootstrap_state,
            AcpContextBootstrapState::Preparing
        ) {
            self.queued_submit_while_bootstrapping = true;
            self.context_bootstrap_note =
                Some("Queued \u{00b7} sending when context is attached\u{2026}".into());
            cx.notify();
            return Ok(());
        }

        let blocks = self.prepare_turn_blocks(trimmed);

        let msg_id = self.alloc_id();
        self.messages.push(AcpThreadMessage::new(
            msg_id,
            AcpThreadMessageRole::User,
            trimmed.to_string(),
        ));
        self.input.clear();
        self.context_bootstrap_note = None;
        self.stream_started_at = Some(std::time::Instant::now());
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
        let mut had_request = false;
        let mut changed = false;

        if let Some(request) = self.pending_permission.take() {
            let note = Self::permission_resolution_message(&request, selected_option_id.as_deref());
            let _ = request.reply_tx.send_blocking(selected_option_id);
            changed |= self.push_message(AcpThreadMessageRole::System, note);
            had_request = true;
        }

        // Stay in Streaming so submit_input() remains blocked until
        // TurnFinished or Failed arrives — prevents mid-turn double-submit.
        if had_request {
            changed |= self.set_status(AcpThreadStatus::Streaming);
        }

        if changed {
            cx.notify();
        }
    }

    /// Build a human-readable audit message for a permission resolution.
    fn permission_resolution_message(
        request: &AcpApprovalRequest,
        selected_option_id: Option<&str>,
    ) -> String {
        let tool_title = request
            .preview
            .as_ref()
            .map(|p| p.tool_title.clone())
            .unwrap_or_else(|| request.title.clone());

        match selected_option_id
            .and_then(|id| request.options.iter().find(|opt| opt.option_id == id))
        {
            Some(option) => format!(
                "Permission granted \u{00b7} {} \u{00b7} {}",
                tool_title,
                option.summary_label()
            ),
            None => format!("Permission cancelled \u{00b7} {}", tool_title),
        }
    }

    // ── Private helpers ────────────────────────────────────────────

    /// Build the content blocks for a turn, consuming staged context on first use.
    ///
    /// When context is present, the user's text is wrapped with a clear
    /// `--- USER REQUEST ---` marker so the agent distinguishes ambient context
    /// from the actual user intent.
    #[cfg_attr(test, allow(dead_code))]
    pub(super) fn prepare_turn_blocks(&mut self, input: &str) -> Vec<ContentBlock> {
        let mut blocks = Vec::new();

        let has_context = !self.pending_context_consumed;
        if has_context {
            blocks.extend(self.pending_context_blocks.clone());
            // No automatic artifact authoring guidance — users should
            // explicitly invoke /script-authoring when they want to
            // create scripts. Default is exploration mode.
            self.pending_context_consumed = true;
        }

        if has_context {
            // Wrap user text with a clear marker so the agent knows
            // everything above is ambient context and this is the request.
            blocks.push(ContentBlock::Text(TextContent::new(format!(
                "--- USER REQUEST ---\n{input}"
            ))));
        } else {
            blocks.push(ContentBlock::Text(TextContent::new(input)));
        }
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
    ///
    /// Only calls `cx.notify()` when state actually changes, avoiding redundant
    /// repaints for duplicate plan, mode, command, or tool-call updates.
    fn apply_event(&mut self, event: AcpEvent, cx: &mut Context<Self>) {
        let mut changed = false;

        match event {
            AcpEvent::UserMessageDelta(chunk) => {
                changed |= self.append_chunk(AcpThreadMessageRole::System, chunk);
                changed |= self.set_status(AcpThreadStatus::Streaming);
            }
            AcpEvent::AgentMessageDelta(chunk) => {
                changed |= self.append_chunk(AcpThreadMessageRole::Assistant, chunk);
                changed |= self.set_status(AcpThreadStatus::Streaming);
            }
            AcpEvent::AgentThoughtDelta(chunk) => {
                changed |= self.append_chunk(AcpThreadMessageRole::Thought, chunk);
                changed |= self.set_status(AcpThreadStatus::Streaming);
            }
            AcpEvent::ToolCallStarted {
                tool_call_id,
                title,
                status,
            } => {
                changed |= self.upsert_tool_call_start(tool_call_id, title, status);
                changed |= self.set_status(AcpThreadStatus::Streaming);
            }
            AcpEvent::ToolCallUpdated {
                tool_call_id,
                title,
                status,
                body,
            } => {
                changed |= self.apply_tool_call_update(tool_call_id, title, status, body);
                changed |= self.set_status(AcpThreadStatus::Streaming);
            }
            AcpEvent::PlanUpdated { entries } => {
                if self.active_plan_entries != entries {
                    self.active_plan_entries = entries;
                    changed = true;
                }
                changed |= self.set_status(AcpThreadStatus::Streaming);
            }
            AcpEvent::AvailableCommandsUpdated { command_names } => {
                if self.available_commands != command_names {
                    self.available_commands = command_names;
                    changed = true;
                }
            }
            AcpEvent::ModeChanged { mode_id } => {
                if self.active_mode_id.as_deref() != Some(mode_id.as_str()) {
                    self.active_mode_id = Some(mode_id);
                    changed = true;
                }
            }
            AcpEvent::TurnFinished { .. } => {
                if self.pending_permission.take().is_some() {
                    changed = true;
                }
                changed |= self.set_status(AcpThreadStatus::Idle);
            }
            AcpEvent::Failed { error } => {
                if self.pending_permission.take().is_some() {
                    changed = true;
                }
                changed |= self.push_message(AcpThreadMessageRole::Error, error);
                changed |= self.set_status(AcpThreadStatus::Error);
            }
        }

        if changed {
            cx.notify();
        }
    }

    /// Set the thread status, returning `true` if it actually changed.
    fn set_status(&mut self, next: AcpThreadStatus) -> bool {
        if self.status == next {
            return false;
        }
        // Track streaming start time.
        if matches!(next, AcpThreadStatus::Streaming)
            && !matches!(self.status, AcpThreadStatus::Streaming)
        {
            self.stream_started_at = Some(std::time::Instant::now());
        } else if !matches!(next, AcpThreadStatus::Streaming) {
            self.stream_started_at = None;
        }
        self.status = next;
        true
    }

    /// Push a new message with an auto-allocated ID. Returns `true` always.
    fn push_message(&mut self, role: AcpThreadMessageRole, body: impl Into<SharedString>) -> bool {
        let id = self.alloc_id();
        self.messages.push(AcpThreadMessage::new(id, role, body));
        true
    }

    /// Append a text chunk to the last message if it has the same role,
    /// otherwise create a new message. This coalesces streaming deltas.
    /// Returns `true` if state changed (i.e. chunk was non-empty).
    fn append_chunk(&mut self, role: AcpThreadMessageRole, chunk: String) -> bool {
        if chunk.is_empty() {
            return false;
        }
        if let Some(last) = self.messages.last_mut() {
            if last.role == role {
                let mut text = last.body.to_string();
                text.push_str(&chunk);
                last.body = text.into();
                return true;
            }
        }
        let id = self.alloc_id();
        self.messages.push(AcpThreadMessage::new(id, role, chunk));
        true
    }

    /// Insert or update a tool call from a `ToolCallStarted` event.
    /// Uses `tool_call_lookup` for O(1) access. Returns `true` if state changed.
    fn upsert_tool_call_start(
        &mut self,
        tool_call_id: String,
        title: String,
        status: String,
    ) -> bool {
        if let Some(&slot) = self.tool_call_lookup.get(&tool_call_id) {
            let existing = &mut self.active_tool_calls[slot];
            let mut changed = false;
            if existing.title != title {
                existing.title = title;
                changed = true;
            }
            if existing.status != status {
                existing.status = status;
                changed = true;
            }
            if changed {
                let new_body =
                    Self::format_tool_call_body(&existing.title, &existing.status, &existing.body);
                if let Some(msg) = self.messages.get_mut(existing.message_index) {
                    msg.body = new_body.into();
                }
            }
            return changed;
        }

        let message_id = self.alloc_id();
        let message_index = self.messages.len();
        let message_body = format!("{title}\n{status}");
        self.messages.push(AcpThreadMessage::with_tool_call_id(
            message_id,
            AcpThreadMessageRole::Tool,
            message_body,
            tool_call_id.clone(),
        ));

        let slot = self.active_tool_calls.len();
        self.active_tool_calls.push(AcpToolCallState {
            tool_call_id: tool_call_id.clone(),
            title,
            status,
            body: None,
            message_id,
            message_index,
        });
        self.tool_call_lookup.insert(tool_call_id, slot);
        true
    }

    /// Apply a `ToolCallUpdated` event, updating tracked state and message in-place.
    /// Uses `tool_call_lookup` for O(1) access. Returns `true` if state changed.
    fn apply_tool_call_update(
        &mut self,
        tool_call_id: String,
        title: Option<String>,
        status: Option<String>,
        body: Option<String>,
    ) -> bool {
        if let Some(&slot) = self.tool_call_lookup.get(&tool_call_id) {
            let tool_call = &mut self.active_tool_calls[slot];
            let mut changed = false;

            if let Some(title) = title {
                if tool_call.title != title {
                    tool_call.title = title;
                    changed = true;
                }
            }
            if let Some(status) = status {
                if tool_call.status != status {
                    tool_call.status = status;
                    changed = true;
                }
            }
            if let Some(body) = body {
                if tool_call.body.as_deref() != Some(body.as_str()) {
                    tool_call.body = Some(body);
                    changed = true;
                }
            }

            if changed {
                let new_body = Self::format_tool_call_body(
                    &tool_call.title,
                    &tool_call.status,
                    &tool_call.body,
                );
                if let Some(msg) = self.messages.get_mut(tool_call.message_index) {
                    msg.body = new_body.into();
                }
            }
            return changed;
        }

        // Orphan update — create a standalone tool call entry.
        let title = title.unwrap_or_else(|| "Tool".to_string());
        let status = status.unwrap_or_else(|| "running".to_string());
        let message_id = self.alloc_id();
        let message_index = self.messages.len();
        let message_body = Self::format_tool_call_body(&title, &status, &body);
        self.messages.push(AcpThreadMessage::with_tool_call_id(
            message_id,
            AcpThreadMessageRole::Tool,
            message_body,
            tool_call_id.clone(),
        ));

        let slot = self.active_tool_calls.len();
        self.active_tool_calls.push(AcpToolCallState {
            tool_call_id: tool_call_id.clone(),
            title,
            status,
            body,
            message_id,
            message_index,
        });
        self.tool_call_lookup.insert(tool_call_id, slot);
        true
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

    /// Display name for the agent (e.g. "Claude Code").
    pub(crate) fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Elapsed seconds since streaming started, or `None` if not streaming.
    pub(crate) fn stream_elapsed_secs(&self) -> Option<u64> {
        self.stream_started_at.map(|t| t.elapsed().as_secs())
    }

    /// Cancel the active streaming turn. Drops the pump task and resets to Idle.
    pub(crate) fn cancel_streaming(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.status, AcpThreadStatus::Streaming) {
            return;
        }
        self.stream_task = None;
        self.stream_started_at = None;
        self.status = AcpThreadStatus::Idle;
        cx.notify();
    }

    /// Tracked tool calls, ordered by creation.
    pub(crate) fn active_tool_calls(&self) -> &[AcpToolCallState] {
        &self.active_tool_calls
    }

    /// Current bootstrap state for deferred context capture.
    pub(crate) fn context_bootstrap_state(&self) -> AcpContextBootstrapState {
        self.context_bootstrap_state
    }

    /// Whether a submit is queued waiting for context bootstrap.
    pub(crate) fn queued_submit_while_bootstrapping(&self) -> bool {
        self.queued_submit_while_bootstrapping
    }

    /// Human-readable bootstrap status note, if any.
    pub(crate) fn context_bootstrap_note(&self) -> Option<&str> {
        self.context_bootstrap_note.as_ref().map(|s| s.as_ref())
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
            display_name: "Test Agent".into(),
            messages: Vec::new(),
            input: match initial_input {
                Some(text) if !text.is_empty() => TextInputState::with_text(text),
                _ => TextInputState::new(),
            },
            status: AcpThreadStatus::Idle,
            pending_permission: None,
            pending_context_blocks: context_blocks,
            pending_context_consumed: false,
            context_bootstrap_state: AcpContextBootstrapState::Ready,
            queued_submit_while_bootstrapping: false,
            context_bootstrap_note: None,
            active_plan_entries: Vec::new(),
            active_mode_id: None,
            available_commands: Vec::new(),
            active_tool_calls: Vec::new(),
            tool_call_lookup: HashMap::new(),
            stream_started_at: None,
            stream_task: None,
            permission_task: None,
            next_message_id: 1,
        }
    }

    /// Apply an event without a GPUI context (for testing pure logic).
    /// Reuses the same helper methods as `apply_event` but skips `cx.notify()`.
    pub(super) fn apply_event_test(&mut self, event: super::AcpEvent) {
        match event {
            super::AcpEvent::UserMessageDelta(chunk) => {
                self.append_chunk(AcpThreadMessageRole::System, chunk);
                self.set_status(AcpThreadStatus::Streaming);
            }
            super::AcpEvent::AgentMessageDelta(chunk) => {
                self.append_chunk(AcpThreadMessageRole::Assistant, chunk);
                self.set_status(AcpThreadStatus::Streaming);
            }
            super::AcpEvent::AgentThoughtDelta(chunk) => {
                self.append_chunk(AcpThreadMessageRole::Thought, chunk);
                self.set_status(AcpThreadStatus::Streaming);
            }
            super::AcpEvent::ToolCallStarted {
                tool_call_id,
                title,
                status,
            } => {
                self.upsert_tool_call_start(tool_call_id, title, status);
                self.set_status(AcpThreadStatus::Streaming);
            }
            super::AcpEvent::ToolCallUpdated {
                tool_call_id,
                title,
                status,
                body,
            } => {
                self.apply_tool_call_update(tool_call_id, title, status, body);
                self.set_status(AcpThreadStatus::Streaming);
            }
            super::AcpEvent::PlanUpdated { entries } => {
                self.active_plan_entries = entries;
                self.set_status(AcpThreadStatus::Streaming);
            }
            super::AcpEvent::AvailableCommandsUpdated { command_names } => {
                self.available_commands = command_names;
            }
            super::AcpEvent::ModeChanged { mode_id } => {
                self.active_mode_id = Some(mode_id);
            }
            super::AcpEvent::TurnFinished { .. } => {
                self.set_status(AcpThreadStatus::Idle);
            }
            super::AcpEvent::Failed { error } => {
                self.push_message(AcpThreadMessageRole::Error, error);
                self.set_status(AcpThreadStatus::Error);
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
            display_name: "Test Agent".into(),
            messages: Vec::new(),
            input: TextInputState::new(),
            status: AcpThreadStatus::Idle,
            pending_permission: None,
            pending_context_blocks,
            pending_context_consumed,
            context_bootstrap_state: AcpContextBootstrapState::Ready,
            queued_submit_while_bootstrapping: false,
            context_bootstrap_note: None,
            active_plan_entries: Vec::new(),
            active_mode_id: None,
            available_commands: Vec::new(),
            active_tool_calls: Vec::new(),
            tool_call_lookup: HashMap::new(),
            stream_started_at: None,
            stream_task: None,
            permission_task: None,
            next_message_id: 1,
        }
    }

    #[test]
    fn pending_context_is_only_consumed_once() {
        let mut thread = test_thread(vec![ContentBlock::Text(TextContent::new("context"))], false);

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
        let mut thread = test_thread(vec![ContentBlock::Text(TextContent::new("context"))], false);

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
        let mut thread = test_thread(vec![ContentBlock::Text(TextContent::new("context"))], false);

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
    /// Delegates to the instance method `apply_event_test` on `AcpThread`.
    fn apply_event_test(thread: &mut AcpThread, event: AcpEvent) {
        thread.apply_event_test(event);
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
        assert_eq!(thread.messages[0].tool_call_id.as_deref(), Some("tc-1"));
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
                title: None,
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
                title: None,
                status: Some("done".into()),
                body: None,
            },
        );

        assert_eq!(
            thread.messages.len(),
            1,
            "orphan update should create a standalone message"
        );
        // Orphan update now creates a full tool call entry with default title + provided status.
        assert!(thread.messages[0].body.contains("done"));
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
                title: None,
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
