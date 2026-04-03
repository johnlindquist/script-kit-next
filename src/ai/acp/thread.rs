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
    /// Available models for this agent.
    pub available_models: Vec<super::config::AcpModelEntry>,
    /// Initially selected model ID (e.g. "claude-sonnet-4-6").
    pub selected_model_id: Option<String>,
}

/// One-shot context payload consumed by `prepare_turn_blocks()`.
///
/// Holds the resolved hidden blocks and the resolution receipt from typed
/// context parts. Produced by `take_pending_context_for_turn()` and consumed
/// exactly once per submission.
struct PendingContextTurn {
    blocks: Vec<ContentBlock>,
    receipt: crate::ai::message_parts::ContextResolutionReceipt,
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

    /// Typed context parts visible as chips in the composer.
    /// Resolved into prompt blocks at submit time via
    /// `resolve_context_parts_with_receipt`. Supports add/remove/dedup.
    pending_context_parts: Vec<crate::ai::message_parts::AiContextPart>,

    /// Whether the Ask Anything ambient context path is still active.
    /// Set `true` when an Ask Anything chip is staged; cleared when the
    /// chip is removed. When `false`, deferred ambient capture is suppressed.
    pending_ambient_context_enabled: bool,

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

    /// Session usage: tokens used / context window size.
    pub(crate) usage_tokens: Option<(u64, u64)>,
    /// Session cost in USD (cumulative).
    pub(crate) usage_cost_usd: Option<f64>,

    /// When the current streaming turn started (for elapsed time display).
    stream_started_at: Option<std::time::Instant>,

    /// Handle to the active stream pump task.
    stream_task: Option<Task<()>>,
    /// Handle to the permission listener task.
    permission_task: Option<Task<()>>,

    /// Monotonically increasing message ID counter.
    next_message_id: u64,

    // ── Model selection ──────────────────────────────────────
    /// Available models for this agent.
    available_models: Vec<super::config::AcpModelEntry>,
    /// Currently selected model ID.
    selected_model_id: Option<String>,
    /// Display name for the selected model.
    selected_model_display_name: Option<SharedString>,
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
            pending_context_parts: Vec::new(),
            pending_ambient_context_enabled: false,
            context_bootstrap_state: AcpContextBootstrapState::Preparing,
            queued_submit_while_bootstrapping: false,
            context_bootstrap_note: None,
            active_plan_entries: Vec::new(),
            active_mode_id: None,
            available_commands: Vec::new(),
            active_tool_calls: Vec::new(),
            tool_call_lookup: HashMap::new(),
            usage_tokens: None,
            usage_cost_usd: None,
            stream_started_at: None,
            stream_task: None,
            permission_task: None,
            next_message_id: 1,
            selected_model_display_name: {
                let id = init.selected_model_id.as_deref();
                id.and_then(|sel| {
                    init.available_models.iter().find(|m| m.id == sel).map(|m| {
                        SharedString::from(m.display_name.clone().unwrap_or_else(|| m.id.clone()))
                    })
                })
            },
            selected_model_id: init.selected_model_id,
            available_models: init.available_models,
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
        self.pending_ambient_context_enabled = false;
        self.arm_pending_context("stage_context");
        self.finish_bootstrap(AcpContextBootstrapState::Ready, "Context attached", cx);
        Ok(())
    }

    /// Stage ambient context from a deferred Ask Anything capture.
    ///
    /// If the Ask Anything chip was removed before capture finished, this
    /// is a no-op that still marks bootstrap as ready.  Otherwise it stages
    /// the context blocks and promotes the visible chip from `ResourceUri`
    /// to `AmbientContext` (display-only).
    pub(crate) fn stage_ask_anything_context(
        &mut self,
        context: &crate::ai::TabAiContextBlob,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let ambient_label = self
            .current_ambient_chip_label()
            .unwrap_or_else(|| crate::ai::message_parts::ASK_ANYTHING_LABEL.to_string());

        if !self.pending_ambient_context_enabled {
            self.clear_pending_ambient_context("ask_anything_removed_before_stage");
            self.finish_bootstrap(
                AcpContextBootstrapState::Ready,
                Self::ambient_capture_removed_note(&ambient_label),
                cx,
            );
            return Ok(());
        }

        self.pending_context_blocks = build_tab_ai_acp_context_blocks(context)?;
        self.promote_ask_anything_chip_to_ambient();
        self.arm_pending_context("stage_ask_anything_context");

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_ask_anything_promoted_to_ambient_chip",
            block_count = self.pending_context_blocks.len(),
            chip_label = %ambient_label,
        );

        self.finish_bootstrap(
            AcpContextBootstrapState::Ready,
            Self::ambient_capture_ready_note(&ambient_label),
            cx,
        );
        Ok(())
    }

    fn ambient_capture_preparing_note(label: &str) -> SharedString {
        if label == crate::ai::message_parts::ASK_ANYTHING_LABEL {
            "Capturing desktop context\u{2026}".into()
        } else {
            format!("Capturing {label}\u{2026}").into()
        }
    }

    fn ambient_capture_ready_note(label: &str) -> SharedString {
        if label == crate::ai::message_parts::ASK_ANYTHING_LABEL {
            "Ask Anything ready".into()
        } else {
            format!("{label} ready").into()
        }
    }

    fn ambient_capture_removed_note(label: &str) -> SharedString {
        if label == crate::ai::message_parts::ASK_ANYTHING_LABEL {
            "Ask Anything removed".into()
        } else {
            format!("{label} removed").into()
        }
    }

    fn current_ambient_chip_label(&self) -> Option<String> {
        self.pending_context_parts
            .iter()
            .find_map(|part| part.ambient_chip_label().map(|value| value.to_string()))
    }

    /// Replace the initial ambient bootstrap `ResourceUri` chip with a
    /// display-only `AmbientContext` chip, preserving the original label.
    /// If the resource chip was already removed, pushes a new ambient chip.
    fn promote_ask_anything_chip_to_ambient(&mut self) {
        if let Some(part) = self
            .pending_context_parts
            .iter_mut()
            .find(|part| part.is_ambient_bootstrap_resource())
        {
            let label = part.label().to_string();
            *part = crate::ai::message_parts::AiContextPart::AmbientContext { label };
            return;
        }
        self.pending_context_parts
            .push(crate::ai::message_parts::AiContextPart::AmbientContext {
                label: crate::ai::message_parts::ASK_ANYTHING_LABEL.to_string(),
            });
    }

    /// Mark the context bootstrap as ready without staging ambient context
    /// blocks. Used by the focused-target Tab path where only typed context
    /// parts (chips) are staged — no hidden `TabAiContextBlob`.
    pub(crate) fn mark_context_bootstrap_ready(&mut self, cx: &mut Context<Self>) {
        self.context_bootstrap_state = AcpContextBootstrapState::Ready;
        self.context_bootstrap_note = None;
        if let Err(error) = self.flush_bootstrap_queue(cx) {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "acp_bootstrap_flush_failed",
                error = %error,
            );
        }
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
        self.finish_bootstrap(AcpContextBootstrapState::Failed, note, cx);
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
                model_id: self.selected_model_id.clone(),
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

    /// Mark pending context as ready for the next submit.
    fn arm_pending_context(&mut self, reason: &'static str) {
        self.pending_context_consumed = false;
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_pending_context_armed",
            reason,
            pending_part_count = self.pending_context_parts.len(),
            pending_block_count = self.pending_context_blocks.len(),
            ambient_enabled = self.pending_ambient_context_enabled,
        );
    }

    /// Clear hidden ambient context blocks and disable the ambient flag.
    fn clear_pending_ambient_context(&mut self, reason: &'static str) {
        let cleared_block_count = self.pending_context_blocks.len();
        self.pending_context_blocks.clear();
        self.pending_ambient_context_enabled = false;
        self.pending_context_consumed = false;
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_pending_ambient_context_cleared",
            reason,
            cleared_block_count,
            pending_part_count = self.pending_context_parts.len(),
        );
    }

    /// Clear all pending context state (parts, blocks, flags).
    fn clear_all_pending_context(&mut self, reason: &'static str) {
        let cleared_part_count = self.pending_context_parts.len();
        let cleared_block_count = self.pending_context_blocks.len();
        self.pending_context_parts.clear();
        self.pending_context_blocks.clear();
        self.pending_context_consumed = false;
        self.pending_ambient_context_enabled = false;
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_pending_context_cleared",
            reason,
            cleared_part_count,
            cleared_block_count,
        );
    }

    /// Flush a queued submit if conditions allow, otherwise just notify.
    fn flush_bootstrap_queue(&mut self, cx: &mut Context<Self>) -> Result<(), String> {
        let submit_now = self.queued_submit_while_bootstrapping
            && !self.input.text().trim().is_empty()
            && !matches!(
                self.status,
                AcpThreadStatus::Streaming | AcpThreadStatus::WaitingForPermission
            );
        self.queued_submit_while_bootstrapping = false;

        if submit_now {
            return self.submit_input(cx);
        }
        cx.notify();
        Ok(())
    }

    /// Finalize bootstrap state and flush any queued submit.
    fn finish_bootstrap(
        &mut self,
        state: AcpContextBootstrapState,
        note: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.context_bootstrap_state = state;
        self.context_bootstrap_note = Some(note.into());
        if let Err(error) = self.flush_bootstrap_queue(cx) {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "acp_bootstrap_flush_failed",
                error = %error,
            );
        }
    }

    /// Consume pending context for a single turn. Returns `None` if already
    /// consumed or nothing is staged. Drains both hidden blocks and typed
    /// parts, resolves parts into prompt blocks, and marks context consumed.
    fn take_pending_context_for_turn(&mut self) -> Option<PendingContextTurn> {
        let has_pending_parts = !self.pending_context_parts.is_empty();
        let has_pending_blocks = !self.pending_context_blocks.is_empty();

        if self.pending_context_consumed || (!has_pending_parts && !has_pending_blocks) {
            return None;
        }

        let blocks = std::mem::take(&mut self.pending_context_blocks);
        // Clone parts so the chip remains visible after submit.
        // The `pending_context_consumed` flag prevents re-resolution.
        let pending_parts = self.pending_context_parts.clone();
        let consumed_block_count = blocks.len();
        let consumed_part_count = pending_parts.len();

        let receipt = if pending_parts.is_empty() {
            crate::ai::message_parts::ContextResolutionReceipt {
                attempted: 0,
                resolved: 0,
                failures: Vec::new(),
                prompt_prefix: String::new(),
            }
        } else {
            crate::ai::message_parts::resolve_context_parts_with_receipt(&pending_parts, &[], &[])
        };

        self.pending_context_consumed = true;
        self.pending_ambient_context_enabled = false;

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_pending_context_consumed",
            consumed_part_count,
            consumed_block_count,
            resolved_part_count = receipt.resolved,
            failed_part_count = receipt.failures.len(),
        );

        Some(PendingContextTurn { blocks, receipt })
    }

    /// Build the content blocks for a turn, consuming staged context on first use.
    ///
    /// Delegates to `take_pending_context_for_turn()` for one-shot consumption.
    /// When context is present, the user's text is wrapped with a clear
    /// `--- USER REQUEST ---` marker so the agent distinguishes ambient context
    /// from the actual user intent.
    #[cfg_attr(test, allow(dead_code))]
    pub(super) fn prepare_turn_blocks(&mut self, input: &str) -> Vec<ContentBlock> {
        let mut blocks = Vec::new();

        if let Some(turn) = self.take_pending_context_for_turn() {
            blocks.extend(turn.blocks);

            if turn.receipt.attempted > 0 {
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "acp_submit_resolved_context_parts",
                    attempted = turn.receipt.attempted,
                    resolved = turn.receipt.resolved,
                    failures = turn.receipt.failures.len(),
                );
            }

            if !turn.receipt.prompt_prefix.is_empty() {
                blocks.push(ContentBlock::Text(TextContent::new(
                    turn.receipt.prompt_prefix,
                )));
            }

            blocks.push(ContentBlock::Text(TextContent::new(format!(
                "--- USER REQUEST ---\n{input}"
            ))));
            return blocks;
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
            AcpEvent::UsageUpdated {
                used_tokens,
                context_size,
                cost_usd,
            } => {
                self.usage_tokens = Some((used_tokens, context_size));
                if let Some(cost) = cost_usd {
                    self.usage_cost_usd = Some(cost);
                }
                changed = true;
            }
            AcpEvent::TurnFinished { .. } => {
                if self.pending_permission.take().is_some() {
                    changed = true;
                }
                changed |= self.set_status(AcpThreadStatus::Idle);

                // Save conversation summary + full messages to history
                if let Some(first_user_msg) = self
                    .messages
                    .iter()
                    .find(|m| matches!(m.role, AcpThreadMessageRole::User))
                {
                    let timestamp = chrono::Utc::now().to_rfc3339();
                    super::history::save_history_entry(&super::history::AcpHistoryEntry {
                        timestamp: timestamp.clone(),
                        first_message: first_user_msg.body.chars().take(100).collect(),
                        message_count: self.messages.len(),
                        session_id: self.ui_thread_id.clone(),
                    });
                    super::history::save_conversation(&super::history::SavedConversation {
                        session_id: self.ui_thread_id.clone(),
                        timestamp,
                        messages: self
                            .messages
                            .iter()
                            .map(|m| super::history::SavedMessage {
                                role: format!("{:?}", m.role),
                                body: m.body.to_string(),
                            })
                            .collect(),
                    });
                }
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

    /// Short display name for the currently selected model, or the agent name if none selected.
    pub(crate) fn selected_model_display(&self) -> &str {
        self.selected_model_display_name
            .as_deref()
            .unwrap_or(&self.display_name)
    }

    /// Available models for this agent.
    pub(crate) fn available_models(&self) -> &[super::config::AcpModelEntry] {
        &self.available_models
    }

    /// Currently selected model ID, if any.
    pub(crate) fn selected_model_id(&self) -> Option<&str> {
        self.selected_model_id.as_deref()
    }

    /// Select a model by ID. Updates the display name, persists to settings, and notifies.
    pub(crate) fn select_model(&mut self, model_id: &str, cx: &mut Context<Self>) {
        if let Some(entry) = self.available_models.iter().find(|m| m.id == model_id) {
            self.selected_model_id = Some(entry.id.clone());
            self.selected_model_display_name = Some(SharedString::from(
                entry
                    .display_name
                    .clone()
                    .unwrap_or_else(|| entry.id.clone()),
            ));

            // Persist selection to settings.json (non-fatal).
            let id = entry.id.clone();
            std::thread::Builder::new()
                .name("acp-save-model".into())
                .spawn(move || {
                    let mut prefs = crate::config::load_user_preferences();
                    prefs.ai.selected_model_id = Some(id.clone());
                    if let Err(e) = crate::config::save_user_preferences(&prefs) {
                        tracing::warn!(error = %e, "failed_to_persist_model_selection");
                    } else {
                        tracing::info!(model = %id, "model_selection_persisted");
                    }
                })
                .ok();

            cx.notify();
        }
    }

    /// Elapsed seconds since streaming started, or `None` if not streaming.
    pub(crate) fn stream_elapsed_secs(&self) -> Option<u64> {
        self.stream_started_at.map(|t| t.elapsed().as_secs())
    }

    /// Add a system message to the thread (visible in the chat, not sent to the agent).
    pub(crate) fn push_system_message(
        &mut self,
        body: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.push_message(AcpThreadMessageRole::System, body);
        cx.notify();
    }

    /// Clear all messages for a fresh conversation within the same session.
    /// Also clears all pending context state so no stale chips or hidden
    /// blocks leak into the next conversation.
    pub(crate) fn clear_messages(&mut self, cx: &mut Context<Self>) {
        self.messages.clear();
        self.active_plan_entries.clear();
        self.active_tool_calls.clear();
        self.tool_call_lookup.clear();
        self.clear_all_pending_context("clear_messages");
        self.context_bootstrap_state = AcpContextBootstrapState::Ready;
        self.context_bootstrap_note = None;

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_thread_cleared",
        );
        cx.notify();
    }

    pub(crate) fn cancel_streaming(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.status, AcpThreadStatus::Streaming) {
            return;
        }
        self.stream_task = None;
        self.stream_started_at = None;
        self.status = AcpThreadStatus::Idle;
        cx.notify();
    }

    /// Load saved messages from a conversation history file.
    /// Replaces current messages with the saved ones (read-only view).
    /// Clears all pending context state so loaded history does not inherit
    /// stale chips from the previous conversation.
    pub(crate) fn load_saved_messages(
        &mut self,
        saved: &[super::history::SavedMessage],
        cx: &mut Context<Self>,
    ) {
        self.clear_all_pending_context("load_saved_messages");
        self.messages.clear();
        for msg in saved {
            let role = match msg.role.as_str() {
                "User" => AcpThreadMessageRole::User,
                "Assistant" => AcpThreadMessageRole::Assistant,
                "Thought" => AcpThreadMessageRole::Thought,
                "Tool" => AcpThreadMessageRole::Tool,
                "System" => AcpThreadMessageRole::System,
                "Error" => AcpThreadMessageRole::Error,
                _ => AcpThreadMessageRole::System,
            };
            let id = self.alloc_id();
            self.messages
                .push(AcpThreadMessage::new(id, role, msg.body.clone()));
        }
        self.context_bootstrap_state = AcpContextBootstrapState::Ready;
        self.context_bootstrap_note = None;
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

    // ── Typed context parts (composer chips) ─────────────────────

    /// Read the pending context parts (visible as chips in the composer).
    pub(crate) fn pending_context_parts(&self) -> &[crate::ai::message_parts::AiContextPart] {
        &self.pending_context_parts
    }

    /// Add a typed context part. Deduplicates by value equality — if an
    /// identical part already exists the call is a no-op.
    ///
    /// When an Ask Anything part is added, stale hidden ambient blocks are
    /// cleared and the bootstrap state is set to `Preparing` so the deferred
    /// capture path knows to arm.
    pub(crate) fn add_context_part(
        &mut self,
        part: crate::ai::message_parts::AiContextPart,
        cx: &mut Context<Self>,
    ) {
        let already_present = self
            .pending_context_parts
            .iter()
            .any(|existing| existing == &part);

        if already_present {
            tracing::info!(
                target: "script_kit::tab_ai",
                event = "acp_context_part_add_skipped_duplicate",
                source = %part.source(),
                label = %part.label(),
            );
            return;
        }

        let is_ambient_bootstrap = part.is_ambient_bootstrap_resource();
        let ambient_label = part.ambient_chip_label().map(|value| value.to_string());
        let label = part.label().to_string();
        let source = part.source().to_string();

        if is_ambient_bootstrap {
            self.clear_pending_ambient_context("add_ambient_bootstrap_resource");
            self.pending_ambient_context_enabled = true;
            self.context_bootstrap_state = AcpContextBootstrapState::Preparing;
            self.context_bootstrap_note = ambient_label
                .as_deref()
                .map(Self::ambient_capture_preparing_note);
        }

        self.pending_context_parts.push(part);
        self.arm_pending_context(if is_ambient_bootstrap {
            "add_ambient_bootstrap_part"
        } else {
            "add_context_part"
        });

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_context_part_added",
            source = %source,
            label = %label,
            is_ambient_bootstrap,
            ambient_label = ?ambient_label,
            pending_part_count = self.pending_context_parts.len(),
            pending_block_count = self.pending_context_blocks.len(),
        );
        cx.notify();
    }

    /// Remove a typed context part by index.
    ///
    /// When an Ask Anything or AmbientContext chip is removed, clears the
    /// staged ambient blocks, disables ambient staging, updates the bootstrap
    /// state/note, and prevents deferred ambient context from being submitted.
    /// If a submit was queued while bootstrapping and the chip is removed,
    /// re-evaluates whether to submit now (without ambient context).
    pub(crate) fn remove_context_part(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.pending_context_parts.len() {
            return;
        }
        let removed = self.pending_context_parts.remove(index);
        let removed_ambient_label = removed.ambient_chip_label().map(|value| value.to_string());

        if let Some(ref ambient_label) = removed_ambient_label {
            self.clear_pending_ambient_context("remove_ambient_context_part");
            self.finish_bootstrap(
                AcpContextBootstrapState::Ready,
                Self::ambient_capture_removed_note(ambient_label),
                cx,
            );
        } else {
            self.arm_pending_context("remove_context_part");
            cx.notify();
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "acp_context_part_removed",
            index,
            source = %removed.source(),
            label = %removed.label(),
            removed_ambient = removed_ambient_label.is_some(),
            ambient_label = ?removed_ambient_label,
            pending_part_count = self.pending_context_parts.len(),
            pending_block_count = self.pending_context_blocks.len(),
        );
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
            pending_context_parts: Vec::new(),
            pending_ambient_context_enabled: false,
            context_bootstrap_state: AcpContextBootstrapState::Ready,
            queued_submit_while_bootstrapping: false,
            context_bootstrap_note: None,
            active_plan_entries: Vec::new(),
            active_mode_id: None,
            available_commands: Vec::new(),
            active_tool_calls: Vec::new(),
            tool_call_lookup: HashMap::new(),
            usage_tokens: None,
            usage_cost_usd: None,
            stream_started_at: None,
            stream_task: None,
            permission_task: None,
            next_message_id: 1,
            available_models: Vec::new(),
            selected_model_id: None,
            selected_model_display_name: None,
        }
    }

    /// Add a context part without a GPUI context (skips `cx.notify()`).
    pub(super) fn add_context_part_test(&mut self, part: crate::ai::message_parts::AiContextPart) {
        let already_present = self
            .pending_context_parts
            .iter()
            .any(|existing| existing == &part);
        if already_present {
            return;
        }

        let is_ambient_bootstrap = part.is_ambient_bootstrap_resource();
        self.pending_context_consumed = false;

        if is_ambient_bootstrap {
            self.pending_context_blocks.clear();
            self.pending_ambient_context_enabled = true;
            self.context_bootstrap_state = AcpContextBootstrapState::Preparing;
            self.context_bootstrap_note = part
                .ambient_chip_label()
                .map(Self::ambient_capture_preparing_note);
        }

        self.pending_context_parts.push(part);
    }

    /// Remove a context part by index without a GPUI context (skips `cx.notify()`).
    pub(super) fn remove_context_part_test(&mut self, index: usize) {
        if index >= self.pending_context_parts.len() {
            return;
        }
        let removed = self.pending_context_parts.remove(index);
        let removed_ambient_label = removed.ambient_chip_label().map(|value| value.to_string());

        if let Some(ref ambient_label) = removed_ambient_label {
            self.pending_ambient_context_enabled = false;
            self.pending_context_blocks.clear();
            self.pending_context_consumed = false;
            self.context_bootstrap_state = AcpContextBootstrapState::Ready;
            self.context_bootstrap_note = Some(Self::ambient_capture_removed_note(ambient_label));
        }
    }

    /// Stage Ask Anything context without GPUI context (skips `cx.notify()`).
    pub(super) fn stage_ask_anything_context_test(
        &mut self,
        context: &crate::ai::TabAiContextBlob,
    ) -> Result<(), String> {
        let ambient_label = self
            .current_ambient_chip_label()
            .unwrap_or_else(|| crate::ai::message_parts::ASK_ANYTHING_LABEL.to_string());

        if !self.pending_ambient_context_enabled {
            self.pending_context_blocks.clear();
            self.pending_context_consumed = false;
            self.context_bootstrap_state = AcpContextBootstrapState::Ready;
            self.context_bootstrap_note = Some(Self::ambient_capture_removed_note(&ambient_label));
            return Ok(());
        }

        self.pending_context_blocks = build_tab_ai_acp_context_blocks(context)?;
        self.pending_context_consumed = false;
        self.promote_ask_anything_chip_to_ambient();
        self.context_bootstrap_state = AcpContextBootstrapState::Ready;
        self.context_bootstrap_note = Some(Self::ambient_capture_ready_note(&ambient_label));
        Ok(())
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
            super::AcpEvent::UsageUpdated {
                used_tokens,
                context_size,
                cost_usd,
            } => {
                self.usage_tokens = Some((used_tokens, context_size));
                if let Some(cost) = cost_usd {
                    self.usage_cost_usd = Some(cost);
                }
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
            pending_context_parts: Vec::new(),
            pending_ambient_context_enabled: false,
            context_bootstrap_state: AcpContextBootstrapState::Ready,
            queued_submit_while_bootstrapping: false,
            context_bootstrap_note: None,
            active_plan_entries: Vec::new(),
            active_mode_id: None,
            available_commands: Vec::new(),
            active_tool_calls: Vec::new(),
            tool_call_lookup: HashMap::new(),
            usage_tokens: None,
            usage_cost_usd: None,
            stream_started_at: None,
            stream_task: None,
            permission_task: None,
            next_message_id: 1,
            available_models: Vec::new(),
            selected_model_id: None,
            selected_model_display_name: None,
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
    fn prepare_turn_blocks_no_guidance_in_exploration_mode() {
        let mut thread = test_thread(vec![ContentBlock::Text(TextContent::new("context"))], false);

        // Even authoring-like intents get no guidance — users invoke /script-authoring explicitly
        let blocks = thread.prepare_turn_blocks("build a clipboard cleanup script");

        // context + input = 2 blocks (no guidance, exploration mode)
        assert_eq!(
            blocks.len(),
            2,
            "exploration mode: context + input only, no guidance"
        );
    }

    #[test]
    fn prepare_turn_blocks_no_guidance_for_any_intent() {
        let mut thread = test_thread(vec![ContentBlock::Text(TextContent::new("context"))], false);

        let blocks = thread.prepare_turn_blocks("explain this selection");

        // context + input = 2 blocks
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

    // ── Chip lifecycle regression tests ───────────────────────────

    /// Helper: build a minimal `TabAiContextBlob` for testing stage operations.
    fn minimal_blob() -> crate::ai::TabAiContextBlob {
        crate::ai::TabAiContextBlob::from_parts(
            crate::ai::tab_context::TabAiUiSnapshot {
                prompt_type: "ScriptList".to_string(),
                input_text: None,
                focused_semantic_id: None,
                selected_semantic_id: None,
                visible_elements: Vec::new(),
            },
            crate::context_snapshot::AiContextSnapshot::default(),
            Vec::new(),
            None,
            Vec::new(),
            Vec::new(),
            "2026-01-01T00:00:00Z".to_string(),
        )
    }

    /// Helper: build an Ask Anything `ResourceUri` part.
    fn ask_anything_part() -> crate::ai::message_parts::AiContextPart {
        crate::ai::message_parts::AiContextPart::ResourceUri {
            uri: crate::ai::message_parts::ASK_ANYTHING_RESOURCE_URI.to_string(),
            label: crate::ai::message_parts::ASK_ANYTHING_LABEL.to_string(),
        }
    }

    /// Helper: build a focused-target part.
    fn focused_target_part(name: &str) -> crate::ai::message_parts::AiContextPart {
        crate::ai::message_parts::AiContextPart::FocusedTarget {
            target: crate::ai::tab_context::TabAiTargetContext {
                source: "ScriptList".to_string(),
                kind: "script".to_string(),
                semantic_id: format!("choice:0:{name}"),
                label: name.to_string(),
                metadata: None,
            },
            label: name.to_string(),
        }
    }

    /// Regression: Ask Anything chip removed before capture completes.
    ///
    /// When the user arms Ask Anything then removes the chip while the deferred
    /// capture is still running, the thread must disable ambient context so that
    /// `stage_ask_anything_context` becomes a no-op and no stale blocks are
    /// attached to the first submit.
    #[test]
    fn ask_anything_removed_before_capture_completes() {
        let mut thread = test_thread(Vec::new(), false);

        // 1. Arm the Ask Anything chip (simulates Tab from a fallback surface).
        thread.add_context_part_test(ask_anything_part());
        assert!(thread.pending_ambient_context_enabled);
        assert_eq!(
            thread.context_bootstrap_state,
            AcpContextBootstrapState::Preparing
        );
        assert_eq!(thread.pending_context_parts.len(), 1);

        // 2. User removes the chip before capture finishes.
        thread.remove_context_part_test(0);

        // 3. Assert: ambient disabled, no blocks, bootstrap ready, chip gone.
        assert!(!thread.pending_ambient_context_enabled);
        assert!(thread.pending_context_blocks.is_empty());
        assert_eq!(
            thread.context_bootstrap_state,
            AcpContextBootstrapState::Ready
        );
        assert_eq!(
            thread.context_bootstrap_note.as_ref().map(|s| s.as_ref()),
            Some("Ask Anything removed")
        );
        assert!(thread.pending_context_parts.is_empty());

        // 4. Deferred capture completes — should be a no-op.
        let blob = minimal_blob();
        thread
            .stage_ask_anything_context_test(&blob)
            .expect("stage should succeed");
        assert!(
            thread.pending_context_blocks.is_empty(),
            "blocks should remain empty after late capture"
        );

        // 5. First submit should carry no ambient context.
        thread.input.set_text("hello");
        let blocks = thread.prepare_turn_blocks("hello");
        assert_eq!(blocks.len(), 1, "only user input, no ambient context");
    }

    /// Regression: Ask Anything chip removed after ambient promotion.
    ///
    /// After capture completes and the chip is promoted from `ResourceUri` to
    /// `AmbientContext`, removing the promoted chip must clear the hidden
    /// `pending_context_blocks` so the first submit sends no ambient context.
    #[test]
    fn ask_anything_removed_after_ambient_promotion() {
        let mut thread = test_thread(Vec::new(), false);

        // 1. Arm the Ask Anything chip.
        thread.add_context_part_test(ask_anything_part());
        assert!(thread.pending_ambient_context_enabled);

        // 2. Capture completes — promotes chip to AmbientContext, stages blocks.
        let blob = minimal_blob();
        thread
            .stage_ask_anything_context_test(&blob)
            .expect("stage should succeed");

        // Verify promotion happened.
        assert_eq!(thread.pending_context_parts.len(), 1);
        assert!(
            thread.pending_context_parts[0].is_ambient_context_chip(),
            "chip should be promoted to AmbientContext"
        );
        assert!(
            !thread.pending_context_blocks.is_empty(),
            "blocks should be staged"
        );
        assert_eq!(
            thread.context_bootstrap_note.as_ref().map(|s| s.as_ref()),
            Some("Ask Anything ready")
        );

        // 3. User removes the promoted chip.
        thread.remove_context_part_test(0);

        // 4. Assert: ambient disabled, blocks cleared, chip gone.
        assert!(!thread.pending_ambient_context_enabled);
        assert!(
            thread.pending_context_blocks.is_empty(),
            "removing promoted chip must clear hidden blocks"
        );
        assert!(thread.pending_context_parts.is_empty());

        // 5. First submit should carry no ambient context.
        thread.input.set_text("hello");
        let blocks = thread.prepare_turn_blocks("hello");
        assert_eq!(blocks.len(), 1, "only user input, no ambient context");
    }

    /// Regression: Focused-target chip consumed on first submit.
    ///
    /// After a focused-target chip is staged and the first message is submitted,
    /// the chip must be consumed (removed from `pending_context_parts`) so the
    /// composer shows no stale chips on the second turn.
    #[test]
    fn focused_target_chip_consumed_on_first_submit() {
        let mut thread = test_thread(Vec::new(), false);

        // 1. Stage a focused-target chip (simulates Tab from a focused surface).
        thread.add_context_part_test(focused_target_part("my-script"));
        assert_eq!(thread.pending_context_parts.len(), 1);
        assert!(!thread.pending_context_consumed);

        // Mark bootstrap as ready (focused path doesn't use deferred capture).
        thread.context_bootstrap_state = AcpContextBootstrapState::Ready;

        // 2. First submit.
        let blocks = thread.prepare_turn_blocks("explain this script");

        // Should have: resolved context part block + USER REQUEST marker + input.
        assert!(
            blocks.len() >= 2,
            "first submit should include context + input, got {} blocks",
            blocks.len()
        );
        assert!(thread.pending_context_consumed);

        // 3. Chip stays visible after submit (not drained).
        assert_eq!(
            thread.pending_context_parts.len(),
            1,
            "chip must persist after submit so it remains visible in the composer"
        );

        // 4. Second submit should carry no context.
        let blocks2 = thread.prepare_turn_blocks("what else?");
        assert_eq!(
            blocks2.len(),
            1,
            "second turn should only have user input, no context"
        );
    }
}
