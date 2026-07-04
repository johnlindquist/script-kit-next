//! Agent Chat thread entity.
//!
//! Owns one Agent Chat conversation thread: messages, composer input, staged context
//! blocks, pending permission state, and the streaming event pump.
//!
//! Plain Tab stages context without sending a turn; the context blocks are
//! prepended to the first real user submit only. Quick-submit (Shift+Tab with
//! intent) populates `initial_input` and calls `submit_input()` after deferred
//! capture resolves.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

use crate::ai::agent_chat::content::{ContentBlock, ImageContent, TextContent};
use gpui::{Context, SharedString, Task};
use smol::Timer;

use crate::components::text_input::TextInputState;
use crate::protocol::AiMessageInfo;

use crate::ai::agent_chat::runtime::{AgentChatConnection, AgentChatTurnRequest};

use super::notifications::{
    dispatch_agent_chat_notification, should_notify_agent_chat_event, truncate_notification_body,
    AgentChatNotificationDebounce, AgentChatNotificationEvent, AgentChatNotificationVisibility,
};
use super::streaming_buffer::StreamingTextBuffer;
use super::{
    build_tab_ai_agent_chat_context_blocks, AgentChatApprovalRequest, AgentChatEvent,
    AgentChatEventRx,
};

fn truncate_chars_for_title_prompt(value: &str, max_chars: usize) -> String {
    let mut out: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        out.push('\u{2026}');
    }
    out
}

#[cfg(test)]
struct TestAgentChatConnection;

#[cfg(test)]
impl AgentChatConnection for TestAgentChatConnection {
    fn start_turn(
        &self,
        _request: AgentChatTurnRequest,
    ) -> anyhow::Result<crate::ai::agent_chat::events::AgentChatEventRx> {
        anyhow::bail!("test connection does not start turns")
    }

    fn cancel_turn(&self, _ui_thread_id: String) -> anyhow::Result<()> {
        Ok(())
    }

    fn prepare_session(
        &self,
        _ui_thread_id: String,
        _cwd: PathBuf,
    ) -> anyhow::Result<crate::ai::agent_chat::events::AgentChatEventRx> {
        anyhow::bail!("test connection does not prepare sessions")
    }
}

/// Bootstrap state for deferred context capture.
///
/// Tracks whether the Tab AI context has been assembled and staged on the
/// thread, so that the first user submit can be gated behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatContextBootstrapState {
    /// Context capture is still running in the background.
    Preparing,
    /// Context has been staged successfully.
    Ready,
    /// Context staging failed; partial or no context is available.
    Failed,
}

/// Current status of the Agent Chat thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatThreadStatus {
    /// No turn in progress; ready for input.
    Idle,
    /// Streaming events from the agent.
    Streaming,
    /// Blocked on a permission decision from the user.
    WaitingForPermission,
    /// The last turn failed.
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatHostWindowKind {
    Main,
    Detached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AgentChatHostWindowState {
    pub(crate) kind: AgentChatHostWindowKind,
    pub(crate) key: bool,
}

/// Severity for the active composer callout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatCalloutSeverity {
    Error,
}

/// Dismissable, actionable callout rendered above the composer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentChatCallout {
    pub(crate) severity: AgentChatCalloutSeverity,
    pub(crate) title: SharedString,
    pub(crate) detail: Option<SharedString>,
    pub(crate) can_retry: bool,
}

impl AgentChatCallout {
    fn failed(error: impl Into<SharedString>, can_retry: bool) -> Self {
        Self {
            severity: AgentChatCalloutSeverity::Error,
            title: "Turn failed".into(),
            detail: Some(error.into()),
            can_retry,
        }
    }
}

/// Role for a message in the thread history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatThreadMessageRole {
    User,
    Assistant,
    Thought,
    Tool,
    System,
    Error,
}

/// A single message in the thread history.
#[derive(Debug, Clone)]
pub(crate) struct AgentChatThreadMessage {
    pub id: u64,
    pub role: AgentChatThreadMessageRole,
    pub body: SharedString,
    /// Optional tool call ID linking this message to an `AgentChatToolCallState`.
    pub tool_call_id: Option<String>,
    /// Structured card metadata for Tool messages (kind, status, subject, diff).
    pub tool_meta: Option<super::tool_card::AgentChatToolCardMeta>,
    /// Context attached to this user message at submit (label + snippet),
    /// rendered in the transcript so the user can see WHAT text was sent and
    /// where it was grabbed from (e.g. `Selection — Safari`).
    pub attachments: Vec<AgentChatMessageAttachment>,
}

/// Visible receipt of one context part sent with a user message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentChatMessageAttachment {
    pub label: SharedString,
    /// Short excerpt of the attached text, when the part carries text.
    pub snippet: Option<SharedString>,
}

impl AgentChatMessageAttachment {
    /// Max chars of attached text shown in the transcript receipt.
    const SNIPPET_MAX_CHARS: usize = 220;

    fn from_part(part: &crate::ai::message_parts::AiContextPart) -> Self {
        use crate::ai::message_parts::AiContextPart;
        let snippet = match part {
            AiContextPart::TextBlock { text, .. } => {
                let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
                let mut snippet: String = collapsed.chars().take(Self::SNIPPET_MAX_CHARS).collect();
                if collapsed.chars().count() > Self::SNIPPET_MAX_CHARS {
                    snippet.push('\u{2026}');
                }
                (!snippet.is_empty()).then(|| SharedString::from(snippet))
            }
            AiContextPart::FilePath { path, .. } => Some(SharedString::from(path.clone())),
            _ => None,
        };
        Self {
            label: SharedString::from(part.label().to_string()),
            snippet,
        }
    }
}

impl AgentChatThreadMessage {
    fn new(id: u64, role: AgentChatThreadMessageRole, body: impl Into<SharedString>) -> Self {
        Self {
            id,
            role,
            body: body.into(),
            tool_call_id: None,
            tool_meta: None,
            attachments: Vec::new(),
        }
    }

    fn with_tool_call_id(
        id: u64,
        role: AgentChatThreadMessageRole,
        body: impl Into<SharedString>,
        tool_call_id: String,
    ) -> Self {
        Self {
            id,
            role,
            body: body.into(),
            tool_call_id: Some(tool_call_id),
            tool_meta: None,
            attachments: Vec::new(),
        }
    }
}

/// Tracked state for a single tool call, kept in sync across start/update events.
#[derive(Debug, Clone)]
pub(crate) struct AgentChatToolCallState {
    /// Agent Chat tool call identifier.
    pub tool_call_id: String,
    /// Display title (e.g. "Read file").
    pub title: String,
    /// Latest status text (e.g. "running", "completed").
    pub status: String,
    /// Latest body text (e.g. file contents, command output).
    pub body: Option<String>,
    /// Raw Pi tool name (e.g. "bash", "edit") when the event carried one.
    pub tool_name: Option<String>,
    /// One-line subject extracted from the tool args (path, command, query).
    pub subject: Option<String>,
    /// Pre-rendered diff from `result.details.diff` for edit/write tools.
    pub diff: Option<String>,
    /// Whether the tool reported an error result.
    pub is_error: bool,
    /// ID of the corresponding `AgentChatThreadMessage` so the view can correlate.
    pub message_id: u64,
    /// Index into `AgentChatThread::messages` for O(1) message lookup.
    message_index: usize,
}

impl AgentChatToolCallState {
    fn card_meta(&self) -> super::tool_card::AgentChatToolCardMeta {
        use super::tool_card::{AgentChatToolCardMeta, AgentChatToolKind, AgentChatToolStatus};
        let tool_name = self.tool_name.clone().unwrap_or_else(|| self.title.clone());
        AgentChatToolCardMeta {
            kind: AgentChatToolKind::from_tool_name(&tool_name),
            tool_name,
            status: if self.is_error {
                AgentChatToolStatus::Failed
            } else {
                AgentChatToolStatus::from_status_str(&self.status)
            },
            subject: self.subject.clone(),
            diff: self.diff.clone(),
            is_error: self.is_error,
        }
    }
}

/// Snapshot of composer draft state that can survive Agent Chat view relaunch.
#[derive(Debug, Clone, Default)]
pub(crate) struct AgentChatThreadDraftSnapshot {
    pub input: String,
    pub input_cursor: usize,
    pub pending_context_parts: Vec<crate::ai::message_parts::AiContextPart>,
    pub pending_context_consumed: bool,
}

/// Initialization parameters for creating an `AgentChatThread`.
#[derive(Debug, Clone)]
pub(crate) struct AgentChatThreadInit {
    /// Unique identifier for this UI thread (used to map to Agent Chat sessions).
    pub ui_thread_id: String,
    /// Working directory for the Agent Chat session.
    pub cwd: PathBuf,
    /// Optional initial input text (populated by Shift+Tab quick-submit).
    pub initial_input: Option<String>,
    /// Optional typed context parts staged before the first submit.
    pub initial_context_parts: Vec<crate::ai::message_parts::AiContextPart>,
    /// Display name for the agent (shown in toolbar, e.g. "Claude Code").
    pub display_name: SharedString,
    /// Stable identifier for the selected Agent Chat profile.
    pub profile_id: String,
    /// Display name for the selected Agent Chat profile (shown beside model).
    pub profile_display_name: Option<SharedString>,
    /// Icon name for the selected Agent Chat profile.
    pub profile_icon_name: Option<String>,
    /// Available models for this agent.
    pub available_models: Vec<super::config::AgentChatModelEntry>,
    /// Initially selected model ID (e.g. "claude-sonnet-4-6").
    pub selected_model_id: Option<String>,
    /// The resolved catalog entry for the selected agent (used for runtime
    /// setup recovery — preserves agent context when `SetupRequired` fires).
    pub selected_agent: Option<super::catalog::AgentChatAgentCatalogEntry>,
    /// Full agent catalog carried through for runtime recovery picker.
    pub available_agents: Vec<super::catalog::AgentChatAgentCatalogEntry>,
    /// Capability requirements derived when the chat was opened.
    /// Preserved through runtime recovery so agent switching stays
    /// capability-driven.
    pub launch_requirements: super::preflight::AgentChatLaunchRequirements,
}

/// One-shot context payload consumed by `prepare_turn_blocks()`.
///
/// Holds the resolved hidden blocks and the resolution receipt from typed
/// context parts. Produced by `take_pending_context_for_turn()` and consumed
/// exactly once per submission.
struct PendingContextTurn {
    blocks: Vec<ContentBlock>,
    receipt: crate::ai::message_parts::ContextResolutionReceipt,
    /// Visible label+snippet receipts for the consumed parts, attached to the
    /// transcript user message.
    attachments: Vec<AgentChatMessageAttachment>,
}

/// Resolved turn-scoped context blocks plus the receipt describing
/// resolution outcomes for the current submit.
struct ResolvedPendingContext {
    blocks: Vec<ContentBlock>,
    receipt: crate::ai::message_parts::ContextResolutionReceipt,
}

/// Return value from `prepare_turn_blocks_with_receipt()`.
///
/// Carries the content blocks for the turn AND the optional resolution
/// receipt so callers can surface partial-failure feedback.
struct PreparedTurnBlocks {
    blocks: Vec<ContentBlock>,
    receipt: Option<crate::ai::message_parts::ContextResolutionReceipt>,
    /// Visible label+snippet receipts for the transcript user message.
    attachments: Vec<AgentChatMessageAttachment>,
}

/// A user submit captured while the current turn is locked.
#[derive(Debug, Clone)]
pub(crate) struct AgentChatQueuedMessage {
    pub(crate) text: String,
    pub(crate) context_parts: Vec<crate::ai::message_parts::AiContextPart>,
}

impl AgentChatQueuedMessage {
    fn new(text: String, context_parts: Vec<crate::ai::message_parts::AiContextPart>) -> Self {
        Self {
            text,
            context_parts,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompletedChatTurnIngest {
    thread_id: String,
    turn_index: usize,
    user_text: String,
    assistant_text: String,
    trace_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum SkillContextStagedBy {
    MainMenu,
    SlashPicker,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SkillContextIdentity {
    pub(crate) thread_id: String,
    pub(crate) skill_id: String,
    pub(crate) skill_file_hash: String,
    pub(crate) staged_by: SkillContextStagedBy,
}

/// GPUI entity that owns one Agent Chat conversation thread.
///
/// Holds durable message history, staged context blocks (consumed once on
/// first submit), composer input, streaming status, and pending permission
/// requests. Binds stream and permission listeners via `cx.spawn(...)`.
pub(crate) struct AgentChatThread {
    connection: Arc<dyn AgentChatConnection>,
    permission_rx: async_channel::Receiver<AgentChatApprovalRequest>,

    ui_thread_id: String,
    cwd: PathBuf,
    /// Display name for the agent (shown in toolbar).
    display_name: SharedString,

    /// Thread message history (durable across turns).
    pub(crate) messages: Vec<AgentChatThreadMessage>,
    /// Current composer input state (with cursor, selection, undo/redo).
    pub(crate) input: TextInputState,
    /// Current thread status.
    pub(crate) status: AgentChatThreadStatus,
    /// Active composer callout, rendered above the composer until dismissed or superseded.
    active_callout: Option<AgentChatCallout>,
    /// Pending permission request awaiting user decision.
    pub(crate) pending_permission: Option<AgentChatApprovalRequest>,

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
    context_bootstrap_state: AgentChatContextBootstrapState,
    /// Whether a submit was attempted while context was still `Preparing`.
    queued_submit_while_bootstrapping: bool,
    /// Human-readable status note for the bootstrap phase.
    context_bootstrap_note: Option<SharedString>,

    /// User messages submitted while the current turn is locked.
    queued_messages: VecDeque<AgentChatQueuedMessage>,
    /// User cancellation pauses queued auto-send without discarding drafts.
    queue_paused: bool,

    // ── Structured session state (readable by the view) ──────────
    /// Current plan entries from the latest `PlanUpdated` event.
    active_plan_entries: Vec<String>,
    /// Current agent mode from the latest `ModeChanged` event.
    active_mode_id: Option<String>,
    /// Current available commands from the latest `AvailableCommandsUpdated`.
    available_commands: Vec<String>,
    /// Tracked tool calls keyed by their Agent Chat tool_call_id.
    active_tool_calls: Vec<AgentChatToolCallState>,
    /// O(1) lookup from tool_call_id to index in `active_tool_calls`.
    tool_call_lookup: HashMap<String, usize>,
    /// Session-scoped "Allow always" grants the user has made, in grant
    /// order. Pi owns the live approval cache; this mirror keeps the grants
    /// visible and reviewable in the UI. Survives `clear_messages` (same Pi
    /// session keeps its cache) and resets with the session.
    standing_approvals: Vec<super::permission_broker::AgentChatStandingApproval>,

    /// User messages the live Pi session can rewind to, in conversation
    /// order (refreshed after every finished turn via `get_fork_messages`).
    fork_points: Vec<super::events::AgentChatForkPoint>,
    /// Ordinal (index into `fork_points`) of an in-flight rewind request.
    /// Consumed when the matching `ForkCompleted` event arrives.
    pending_fork_ordinal: Option<usize>,

    /// The resolved catalog entry for the selected agent. Retained so
    /// runtime `SetupRequired` events can build recovery cards with
    /// agent-specific context.
    selected_agent: Option<super::catalog::AgentChatAgentCatalogEntry>,

    /// Full agent catalog for runtime recovery picker.
    available_agents: Vec<super::catalog::AgentChatAgentCatalogEntry>,

    /// Capability requirements from the original launch context.
    /// Used during runtime recovery to ensure agent switching stays
    /// capability-driven.
    launch_requirements: super::preflight::AgentChatLaunchRequirements,

    /// Inline setup state armed by a runtime `SetupRequired` event.
    /// When `Some`, the view renders the setup recovery card instead of
    /// the normal chat transcript.
    setup_state: Option<super::setup_state::AgentChatInlineSetupState>,

    /// Session usage: tokens used / context window size.
    pub(crate) usage_tokens: Option<(u64, u64)>,
    /// Session cost in USD (cumulative).
    pub(crate) usage_cost_usd: Option<f64>,

    /// When the current streaming turn started (for elapsed time display).
    stream_started_at: Option<std::time::Instant>,
    /// True between submit and the first assistant/thought delta of the turn;
    /// gates the one-shot time-to-first-token log.
    ttft_pending: bool,

    /// Handle to the active stream pump task.
    stream_task: Option<Task<()>>,
    /// Handle to the permission listener task.
    permission_task: Option<Task<()>>,
    /// Buffered assistant deltas waiting for typewriter-paced reveal.
    streaming_text_buffer: StreamingTextBuffer,
    /// Handle to the active assistant text drain task.
    streaming_text_drain_task: Option<Task<()>>,

    /// Generation guard for async stream delivery into the transcript.
    transcript_generation: u64,

    /// Monotonically increasing message ID counter.
    next_message_id: u64,

    /// Host-window state last observed by the AgentChatView render/activation hooks.
    host_window_state: Option<AgentChatHostWindowState>,
    notification_debounce: AgentChatNotificationDebounce,
    current_turn_id: u64,
    llm_title_attempted: bool,

    // ── Model selection ──────────────────────────────────────
    /// Available models for this agent.
    available_models: Vec<super::config::AgentChatModelEntry>,
    /// Currently selected model ID.
    selected_model_id: Option<String>,
    /// Display name for the selected model.
    selected_model_display_name: Option<SharedString>,
    /// Stable identifier for the selected Agent Chat profile.
    profile_id: String,
    /// Display name for the selected Agent Chat profile.
    profile_display_name: Option<SharedString>,
    /// Icon name for the selected Agent Chat profile.
    profile_icon_name: Option<String>,
}

impl AgentChatThread {
    pub(crate) fn set_cwd(&mut self, cwd: PathBuf) {
        self.cwd = cwd;
    }

    pub(crate) fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    /// Create a new thread entity with optional initial input.
    ///
    /// Immediately binds the permission listener. Does NOT send an Agent Chat turn —
    /// context is staged and only consumed on the first `submit_input()`.
    pub(crate) fn new(
        connection: Arc<dyn AgentChatConnection>,
        permission_rx: async_channel::Receiver<AgentChatApprovalRequest>,
        init: AgentChatThreadInit,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut this = Self {
            connection,
            permission_rx,
            ui_thread_id: init.ui_thread_id,
            cwd: init.cwd,
            display_name: init.display_name,
            profile_id: init.profile_id,
            messages: Vec::new(),
            input: match init.initial_input {
                Some(text) if !text.is_empty() => TextInputState::with_text(text),
                _ => TextInputState::new(),
            },
            status: AgentChatThreadStatus::Idle,
            active_callout: None,
            pending_permission: None,
            pending_context_blocks: Vec::new(),
            pending_context_consumed: false,
            pending_context_parts: init.initial_context_parts,
            pending_ambient_context_enabled: false,
            context_bootstrap_state: AgentChatContextBootstrapState::Preparing,
            queued_submit_while_bootstrapping: false,
            context_bootstrap_note: None,
            queued_messages: VecDeque::new(),
            queue_paused: false,
            active_plan_entries: Vec::new(),
            active_mode_id: None,
            available_commands: Vec::new(),
            active_tool_calls: Vec::new(),
            tool_call_lookup: HashMap::new(),
            standing_approvals: Vec::new(),
            fork_points: Vec::new(),
            pending_fork_ordinal: None,
            selected_agent: init.selected_agent,
            available_agents: init.available_agents,
            launch_requirements: init.launch_requirements,
            profile_display_name: init.profile_display_name,
            profile_icon_name: init.profile_icon_name,
            setup_state: None,
            usage_tokens: None,
            usage_cost_usd: None,
            stream_started_at: None,
            ttft_pending: false,
            stream_task: None,
            permission_task: None,
            streaming_text_buffer: StreamingTextBuffer::default(),
            streaming_text_drain_task: None,
            transcript_generation: 0,
            next_message_id: 1,
            host_window_state: None,
            notification_debounce: AgentChatNotificationDebounce::default(),
            current_turn_id: 0,
            llm_title_attempted: false,
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

    fn maybe_spawn_auto_title(&mut self, conversation: &super::history::SavedConversation) {
        if self.llm_title_attempted || conversation.custom_title.is_some() {
            return;
        }
        if !conversation
            .messages
            .iter()
            .any(|message| message.role.eq_ignore_ascii_case("assistant"))
        {
            return;
        }

        let Some(first_user) = conversation
            .messages
            .iter()
            .find(|message| message.role.eq_ignore_ascii_case("user"))
            .map(|message| message.body.clone())
        else {
            return;
        };
        let Some(first_assistant) = conversation
            .messages
            .iter()
            .find(|message| message.role.eq_ignore_ascii_case("assistant"))
            .map(|message| message.body.clone())
        else {
            return;
        };

        self.llm_title_attempted = true;
        let session_id = conversation.session_id.clone();
        let user_excerpt = truncate_chars_for_title_prompt(&first_user, 400);
        let assistant_excerpt = truncate_chars_for_title_prompt(&first_assistant, 400);

        let spawn_result = std::thread::Builder::new()
            .name("agent_chat-auto-title".to_string())
            .spawn(move || {
                let registry =
                    crate::ai::providers::ProviderRegistry::from_environment_with_config(None);
                if !registry.has_any_provider() {
                    return;
                }

                let result = (|| -> anyhow::Result<()> {
                    let (model, provider) =
                        crate::ai::script_generation::select_generation_model(&registry)?;
                    let messages = vec![
                        crate::ai::providers::ProviderMessage::system(
                            "You title chat conversations. Reply with ONLY a concise 3-6 word title. No quotes, no punctuation at the end.",
                        ),
                        crate::ai::providers::ProviderMessage::user(format!(
                            "User: {user_excerpt}\nAssistant: {assistant_excerpt}"
                        )),
                    ];
                    let raw = provider.send_message(&messages, &model.id)?;
                    let title = super::history::sanitize_conversation_title(&raw);
                    if title.is_empty() {
                        return Ok(());
                    }
                    super::history::rename_conversation(&session_id, &title)?;
                    Ok(())
                })();

                if let Err(error) = result {
                    tracing::debug!(
                        target: "script_kit::tab_ai",
                        event = "agent_chat_auto_title_failed",
                        session_id = %session_id,
                        error = %error,
                    );
                }
            });

        if let Err(error) = spawn_result {
            tracing::debug!(
                target: "script_kit::tab_ai",
                event = "agent_chat_auto_title_spawn_failed",
                session_id = %conversation.session_id,
                error = %error,
            );
        }
    }

    pub(crate) fn set_host_window_state(
        &mut self,
        state: AgentChatHostWindowState,
        cx: &mut Context<Self>,
    ) {
        if self.host_window_state == Some(state) {
            return;
        }
        self.host_window_state = Some(state);
        cx.notify();
    }

    fn notification_visibility(&self) -> AgentChatNotificationVisibility {
        let Some(state) = self.host_window_state else {
            return AgentChatNotificationVisibility::Unknown;
        };
        let visible = match state.kind {
            AgentChatHostWindowKind::Main => crate::is_main_window_visible(),
            AgentChatHostWindowKind::Detached => {
                crate::ai::agent_chat::ui::chat_window::is_chat_window_open()
            }
        };
        if visible && state.key {
            AgentChatNotificationVisibility::VisibleAndKey
        } else {
            AgentChatNotificationVisibility::HiddenOrNotKey
        }
    }

    fn notifications_enabled() -> bool {
        crate::config::load_user_preferences()
            .ai
            .agent_chat_notify_when_hidden
            .unwrap_or(true)
    }

    fn maybe_notify_agent_chat_event(
        &mut self,
        event: AgentChatNotificationEvent,
        title: &'static str,
        body: String,
    ) {
        if should_notify_agent_chat_event(
            event,
            self.notification_visibility(),
            Self::notifications_enabled(),
            self.current_turn_id,
            &mut self.notification_debounce,
        ) {
            dispatch_agent_chat_notification(title, truncate_notification_body(&body));
        }
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
        self.pending_context_blocks = build_tab_ai_agent_chat_context_blocks(context)?;
        self.pending_ambient_context_enabled = false;
        self.arm_pending_context("stage_context");
        self.finish_bootstrap(
            AgentChatContextBootstrapState::Ready,
            "Context attached",
            cx,
        );
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
                AgentChatContextBootstrapState::Ready,
                Self::ambient_capture_removed_note(&ambient_label),
                cx,
            );
            return Ok(());
        }

        self.pending_context_blocks = build_tab_ai_agent_chat_context_blocks(context)?;
        self.promote_ask_anything_chip_to_ambient();
        self.arm_pending_context("stage_ask_anything_context");

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_ask_anything_promoted_to_ambient_chip",
            block_count = self.pending_context_blocks.len(),
            chip_label = %ambient_label,
        );

        self.finish_bootstrap(
            AgentChatContextBootstrapState::Ready,
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
        self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
        self.context_bootstrap_note = None;
        if let Err(error) = self.flush_bootstrap_queue(cx) {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "agent_chat_bootstrap_flush_failed",
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
        self.finish_bootstrap(AgentChatContextBootstrapState::Failed, note, cx);
    }

    /// Update the composer input text (replaces entire content, cursor at end).
    pub(crate) fn set_input(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        let value = value.into();
        let cursor = value.chars().count();
        self.input.set_text(value);
        self.input.set_cursor(cursor);
        cx.notify();
    }

    /// Recall the latest user-authored turn into the composer.
    ///
    /// Mirrors common agent prompt-history behavior: plain Up on an empty,
    /// idle composer brings back the previous user prompt with the caret at
    /// the beginning so another Up-like navigation gesture stays natural.
    /// Replace assistant transcript rows after the latest user turn.
    pub(crate) fn replace_assistant_messages_after_last_user(
        &mut self,
        bodies: Vec<String>,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(last_user_index) = self
            .messages
            .iter()
            .rposition(|message| message.role == AgentChatThreadMessageRole::User)
        else {
            return false;
        };

        self.messages.truncate(last_user_index + 1);
        for body in bodies {
            if body.trim().is_empty() {
                continue;
            }
            self.push_message(AgentChatThreadMessageRole::Assistant, body);
        }
        cx.notify();
        true
    }

    /// Add a local assistant guidance message without starting a provider turn.
    ///
    /// Host-owned staging flows use this to explain what context was attached
    /// while still requiring the user to explicitly submit the next turn.
    pub(crate) fn push_local_assistant_message(
        &mut self,
        body: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.push_message(AgentChatThreadMessageRole::Assistant, body);
        cx.notify();
    }

    pub(crate) fn recall_last_user_message(&mut self, cx: &mut Context<Self>) -> bool {
        if !matches!(
            self.status,
            AgentChatThreadStatus::Idle | AgentChatThreadStatus::Error
        ) || !self.input.is_empty()
        {
            return false;
        }

        let Some(body) = self
            .messages
            .iter()
            .rev()
            .find(|message| message.role == AgentChatThreadMessageRole::User)
            .map(|message| message.body.to_string())
        else {
            return false;
        };

        self.input.set_text(body);
        self.input.set_cursor(0);
        cx.notify();
        true
    }

    /// True while a user turn is streaming but no assistant text has landed yet.
    pub(crate) fn awaiting_first_assistant_text(&self) -> bool {
        if !matches!(self.status, AgentChatThreadStatus::Streaming) {
            return false;
        }

        let Some(last_user_index) = self
            .messages
            .iter()
            .rposition(|message| matches!(message.role, AgentChatThreadMessageRole::User))
        else {
            return false;
        };

        !self.messages[last_user_index + 1..].iter().any(|message| {
            matches!(message.role, AgentChatThreadMessageRole::Assistant)
                && !message.body.trim().is_empty()
        })
    }

    /// Submit the current input as a new user turn.
    ///
    /// If context is still bootstrapping (`Preparing`), the submit is queued
    /// and will fire automatically when `stage_context()` or
    /// `mark_context_bootstrap_failed()` completes.
    ///
    /// Prepends staged context blocks on the first submit, then clears them.
    /// Starts streaming events from the Agent Chat agent.
    pub(crate) fn submit_input(&mut self, cx: &mut Context<Self>) -> Result<(), String> {
        let input = self.input.text().to_string();
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        self.resume_queue_for_manual_submit();

        if matches!(
            self.status,
            AgentChatThreadStatus::Streaming | AgentChatThreadStatus::WaitingForPermission
        ) {
            self.queue_current_composer(trimmed.to_string());
            cx.notify();
            return Ok(());
        }

        // Gate on bootstrap: queue instead of sending while context is still preparing.
        if matches!(
            self.context_bootstrap_state,
            AgentChatContextBootstrapState::Preparing
        ) {
            self.queued_submit_while_bootstrapping = true;
            self.context_bootstrap_note =
                Some("Queued \u{00b7} sending when context is attached\u{2026}".into());
            cx.notify();
            return Ok(());
        }

        let prepared = self.prepare_turn_blocks_with_receipt(trimmed);
        self.set_context_resolution_note(prepared.receipt.as_ref());
        self.start_prepared_turn(
            trimmed.to_string(),
            prepared.blocks,
            prepared.attachments,
            true,
            true,
            cx,
        )
    }

    fn resume_queue_for_manual_submit(&mut self) {
        self.queue_paused = false;
    }

    fn queue_current_composer(&mut self, text: String) {
        self.queued_messages.push_back(AgentChatQueuedMessage::new(
            text,
            std::mem::take(&mut self.pending_context_parts),
        ));
        self.input.clear();
    }

    fn start_prepared_turn(
        &mut self,
        display_text: String,
        blocks: Vec<ContentBlock>,
        attachments: Vec<AgentChatMessageAttachment>,
        clear_composer: bool,
        push_user_message: bool,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let rx = self
            .connection
            .start_turn(AgentChatTurnRequest {
                ui_thread_id: self.ui_thread_id.clone(),
                cwd: self.cwd.clone(),
                blocks,
                model_id: self.selected_model_id.clone(),
            })
            .map_err(|error| error.to_string())?;

        if push_user_message {
            let msg_id = self.alloc_id();
            let mut message = AgentChatThreadMessage::new(
                msg_id,
                AgentChatThreadMessageRole::User,
                display_text.clone(),
            );
            message.attachments = attachments;
            self.messages.push(message);
            self.publish_sdk_new_message(msg_id, AgentChatThreadMessageRole::User, display_text);
        }
        if clear_composer {
            self.input.clear();
        }
        self.stream_started_at = Some(std::time::Instant::now());
        self.ttft_pending = true;
        self.status = AgentChatThreadStatus::Streaming;
        self.active_callout = None;
        self.current_turn_id = self.current_turn_id.wrapping_add(1);

        self.setup_state = None;
        self.bind_stream(rx, cx);
        cx.notify();
        Ok(())
    }

    fn submit_queued_message(
        &mut self,
        message: AgentChatQueuedMessage,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let saved_blocks = std::mem::take(&mut self.pending_context_blocks);
        let saved_consumed = self.pending_context_consumed;
        let saved_parts = std::mem::replace(&mut self.pending_context_parts, message.context_parts);
        let saved_ambient = self.pending_ambient_context_enabled;

        self.pending_context_consumed = false;
        self.pending_ambient_context_enabled = false;
        let prepared = self.prepare_turn_blocks_with_receipt(&message.text);
        self.set_context_resolution_note(prepared.receipt.as_ref());

        self.pending_context_blocks = saved_blocks;
        self.pending_context_consumed = saved_consumed;
        self.pending_context_parts = saved_parts;
        self.pending_ambient_context_enabled = saved_ambient;

        self.start_prepared_turn(
            message.text,
            prepared.blocks,
            prepared.attachments,
            false,
            true,
            cx,
        )
    }

    fn submit_next_queued_if_ready(&mut self, cx: &mut Context<Self>) -> Result<(), String> {
        if self.queue_paused
            || self.queued_messages.is_empty()
            || !matches!(self.status, AgentChatThreadStatus::Idle)
        {
            return Ok(());
        }
        if let Some(message) = self.queued_messages.pop_front() {
            self.submit_queued_message(message, cx)?;
        }
        Ok(())
    }

    /// Submit an already-built provider prompt while displaying a separate
    /// user-facing transcript string. Focused-text mini uses this so captured
    /// field contents stay hidden provider context instead of visible composer
    /// text.
    pub(crate) fn submit_blocks(
        &mut self,
        blocks: Vec<ContentBlock>,
        display_user_text: impl Into<String>,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if matches!(
            self.status,
            AgentChatThreadStatus::Streaming | AgentChatThreadStatus::WaitingForPermission
        ) {
            return Ok(());
        }

        let display_user_text = display_user_text.into();
        let trimmed_display = display_user_text.trim();
        if trimmed_display.is_empty() {
            return Ok(());
        }

        if matches!(
            self.context_bootstrap_state,
            AgentChatContextBootstrapState::Preparing
        ) {
            return Err("context_bootstrap_pending".to_string());
        }

        self.clear_all_pending_context("submit_blocks");

        let msg_id = self.alloc_id();
        self.messages.push(AgentChatThreadMessage::new(
            msg_id,
            AgentChatThreadMessageRole::User,
            trimmed_display.to_string(),
        ));
        self.publish_sdk_new_message(
            msg_id,
            AgentChatThreadMessageRole::User,
            trimmed_display.to_string(),
        );
        self.input.clear();
        self.stream_started_at = Some(std::time::Instant::now());
        self.ttft_pending = true;
        self.status = AgentChatThreadStatus::Streaming;

        let rx = self
            .connection
            .start_turn(AgentChatTurnRequest {
                ui_thread_id: self.ui_thread_id.clone(),
                cwd: self.cwd.clone(),
                blocks,
                model_id: self.selected_model_id.clone(),
            })
            .map_err(|error| error.to_string())?;

        self.active_callout = None;
        self.setup_state = None;
        self.bind_stream(rx, cx);
        cx.notify();
        Ok(())
    }

    pub(crate) fn active_callout(&self) -> Option<&AgentChatCallout> {
        self.active_callout.as_ref()
    }

    pub(crate) fn dismiss_active_callout(&mut self, cx: &mut Context<Self>) {
        if self.active_callout.take().is_some() {
            cx.notify();
        }
    }

    fn last_user_turn_text(&self) -> Option<String> {
        self.messages
            .iter()
            .rev()
            .find(|message| matches!(message.role, AgentChatThreadMessageRole::User))
            .map(|message| message.body.to_string())
    }

    pub(crate) fn retry_last_user_turn(&mut self, cx: &mut Context<Self>) -> Result<(), String> {
        if !matches!(
            self.status,
            AgentChatThreadStatus::Error | AgentChatThreadStatus::Idle
        ) {
            return Ok(());
        }

        let Some(display_text) = self.last_user_turn_text() else {
            return Err("no_user_turn_to_retry".to_string());
        };
        let prepared = self.prepare_turn_blocks_with_receipt(display_text.trim());
        self.set_context_resolution_note(prepared.receipt.as_ref());
        self.start_prepared_turn(
            display_text,
            prepared.blocks,
            prepared.attachments,
            false,
            false,
            cx,
        )
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
            self.record_standing_approval(&request, selected_option_id.as_deref());
            let _ = request.reply_tx.send_blocking(selected_option_id);
            changed |= self.push_message(AgentChatThreadMessageRole::System, note);
            had_request = true;
        }

        // Stay in Streaming so submit_input() remains blocked until
        // TurnFinished or Failed arrives — prevents mid-turn double-submit.
        if had_request {
            changed |= self.set_status(AgentChatThreadStatus::Streaming);
        }

        if changed {
            cx.notify();
        }
    }

    /// Record a session-scoped grant when the chosen option is a persistent
    /// "Allow always". Deduped by (tool, subject) so repeated grants for the
    /// same tool do not stack.
    fn record_standing_approval(
        &mut self,
        request: &AgentChatApprovalRequest,
        selected_option_id: Option<&str>,
    ) {
        let Some(option) = selected_option_id
            .and_then(|id| request.options.iter().find(|opt| opt.option_id == id))
        else {
            return;
        };
        if !option.is_persistent_allow() {
            return;
        }

        let (tool_title, subject, kind_badge) = match request.preview.as_ref() {
            Some(preview) => (
                preview.tool_title.clone(),
                preview.subject.clone(),
                preview.kind.badge_label(),
            ),
            None => (
                request.title.clone(),
                None,
                super::permission_broker::AgentChatApprovalPreviewKind::Generic.badge_label(),
            ),
        };

        let already_recorded = self
            .standing_approvals
            .iter()
            .any(|grant| grant.tool_title == tool_title && grant.subject == subject);
        if already_recorded {
            return;
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_standing_approval_recorded",
            ui_thread = %self.ui_thread_id,
            tool_title = %tool_title,
            has_subject = subject.is_some(),
            total = self.standing_approvals.len() + 1,
        );
        self.standing_approvals
            .push(super::permission_broker::AgentChatStandingApproval {
                tool_title,
                subject,
                kind_badge,
                option_label: option.summary_label(),
            });
    }

    /// Session-scoped "Allow always" grants recorded so far, in grant order.
    pub(crate) fn standing_approvals(
        &self,
    ) -> &[super::permission_broker::AgentChatStandingApproval] {
        &self.standing_approvals
    }

    /// Push a System transcript message listing every standing approval, so
    /// the user can review what the session will no longer ask about.
    pub(crate) fn review_standing_approvals(&mut self, cx: &mut Context<Self>) {
        let body = if self.standing_approvals.is_empty() {
            "**Auto-approvals** \u{00b7} none granted this session.".to_string()
        } else {
            let mut lines = vec![format!(
                "**Auto-approvals** \u{00b7} {} standing grant{} this session:",
                self.standing_approvals.len(),
                if self.standing_approvals.len() == 1 {
                    ""
                } else {
                    "s"
                },
            )];
            for grant in &self.standing_approvals {
                let subject = grant
                    .subject
                    .as_deref()
                    .map(|subject| format!(" \u{00b7} `{subject}`"))
                    .unwrap_or_default();
                lines.push(format!(
                    "- {} \u{00b7} {}{subject} \u{00b7} {}",
                    grant.tool_title, grant.kind_badge, grant.option_label,
                ));
            }
            lines.push(
                "Grants live in the Pi session approval cache; starting a new session resets them."
                    .to_string(),
            );
            lines.join("\n")
        };
        self.push_message(AgentChatThreadMessageRole::System, body);
        cx.notify();
    }

    fn permission_notification_body(&self, request: &AgentChatApprovalRequest) -> String {
        request
            .preview
            .as_ref()
            .map(|preview| {
                preview
                    .subject
                    .as_ref()
                    .map(|subject| format!("{} · {subject}", preview.tool_title))
                    .unwrap_or_else(|| preview.tool_title.clone())
            })
            .unwrap_or_else(|| {
                if request.body.trim().is_empty() {
                    request.title.clone()
                } else {
                    format!("{} · {}", request.title, request.body)
                }
            })
    }

    /// Build a human-readable audit message for a permission resolution.
    fn permission_resolution_message(
        request: &AgentChatApprovalRequest,
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
            event = "agent_chat_pending_context_armed",
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
            event = "agent_chat_pending_ambient_context_cleared",
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
            event = "agent_chat_pending_context_cleared",
            reason,
            cleared_part_count,
            cleared_block_count,
        );
    }

    /// Returns `true` for the explicit `@screenshot` resource chip.
    ///
    /// Agent Chat follow-up submits should attach this as a real image block instead
    /// of only resolving the text-only `kit://context?...` snapshot JSON.
    fn is_explicit_screenshot_part(part: &crate::ai::message_parts::AiContextPart) -> bool {
        matches!(
            part,
            crate::ai::message_parts::AiContextPart::ResourceUri { uri, label }
                if label == "Screenshot" && uri.contains("screenshot=1")
        )
    }

    /// Capture the explicit screenshot chip as an Agent Chat image block.
    ///
    /// `@screenshot` captures the active desktop — the display the Script Kit
    /// panel is on — with Script Kit's own windows excluded so the chat panel
    /// does not cover the content being asked about.
    ///
    /// Returns `Ok(None)` for non-screenshot parts so the normal prompt-block
    /// resolver can handle them. On capture failure the caller falls back to
    /// the canonical `kit://context?...` resource path.
    fn capture_special_context_block_for_part(
        part: &crate::ai::message_parts::AiContextPart,
    ) -> Result<Option<ContentBlock>, String> {
        if !Self::is_explicit_screenshot_part(part) {
            return Ok(None);
        }

        let (png_data, width, height) =
            crate::platform::capture_screen_screenshot().map_err(|error| error.to_string())?;
        if png_data.is_empty() {
            return Err("Active desktop screenshot was empty".to_string());
        }

        use base64::Engine as _;

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_inline_screenshot_attachment_captured",
            width,
            height,
            title = "Active desktop",
            bytes = png_data.len(),
        );

        let base64_png = base64::engine::general_purpose::STANDARD.encode(&png_data);
        Ok(Some(ContentBlock::Image(ImageContent::new(
            base64_png,
            "image/png",
        ))))
    }

    /// Resolve pending context parts into Agent Chat blocks plus a standard receipt.
    ///
    /// Most parts resolve into text prompt blocks. Explicit screenshot chips
    /// are upgraded to real Agent Chat attachment blocks first, with the canonical
    /// resource resolver kept as a fallback if image capture fails.
    fn resolve_pending_context_parts_with<F>(
        parts: &[crate::ai::message_parts::AiContextPart],
        mut special_block_resolver: F,
    ) -> ResolvedPendingContext
    where
        F: FnMut(&crate::ai::message_parts::AiContextPart) -> Result<Option<ContentBlock>, String>,
    {
        let mut blocks = Vec::new();
        let mut prompt_blocks = Vec::new();
        let mut failures = Vec::new();

        for part in parts {
            let mut resolved_as_special_block = false;

            match special_block_resolver(part) {
                Ok(Some(block)) => {
                    tracing::info!(
                        target: "script_kit::tab_ai",
                        event = "agent_chat_context_part_resolved_to_special_block",
                        source = %part.source(),
                        label = %part.label(),
                    );
                    blocks.push(block);
                    resolved_as_special_block = true;
                }
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(
                        target: "script_kit::tab_ai",
                        event = "agent_chat_context_special_block_capture_failed",
                        source = %part.source(),
                        label = %part.label(),
                        error = %error,
                    );
                }
            }

            if resolved_as_special_block {
                continue;
            }

            match crate::ai::message_parts::resolve_context_part_to_prompt_block(part, &[], &[]) {
                Ok(block) => {
                    if block.trim().is_empty() {
                        tracing::info!(
                            target: "script_kit::tab_ai",
                            event = "agent_chat_context_part_prompt_block_empty",
                            source = %part.source(),
                            label = %part.label(),
                        );
                        continue;
                    }
                    prompt_blocks.push(block);
                }
                Err(err) => {
                    tracing::warn!(
                        target: "script_kit::tab_ai",
                        event = "agent_chat_context_part_prompt_resolution_failed",
                        source = %part.source(),
                        label = %part.label(),
                        error = %err,
                    );
                    failures.push(crate::ai::message_parts::ContextResolutionFailure {
                        label: part.label().to_string(),
                        source: part.source().to_string(),
                        error: format!("{err:#}"),
                    });
                }
            }
        }

        let resolved = blocks.len() + prompt_blocks.len();
        let prompt_prefix = prompt_blocks.join("\n\n");

        ResolvedPendingContext {
            blocks,
            receipt: crate::ai::message_parts::ContextResolutionReceipt {
                attempted: parts.len(),
                resolved,
                failures,
                prompt_prefix,
            },
        }
    }

    /// Flush a queued submit if conditions allow, otherwise just notify.
    fn flush_bootstrap_queue(&mut self, cx: &mut Context<Self>) -> Result<(), String> {
        let submit_now = self.queued_submit_while_bootstrapping
            && !self.input.text().trim().is_empty()
            && !matches!(
                self.status,
                AgentChatThreadStatus::Streaming | AgentChatThreadStatus::WaitingForPermission
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
        state: AgentChatContextBootstrapState,
        note: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.context_bootstrap_state = state;
        self.context_bootstrap_note = Some(note.into());
        if let Err(error) = self.flush_bootstrap_queue(cx) {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "agent_chat_bootstrap_flush_failed",
                error = %error,
            );
        }
    }

    /// Consume pending context for a single turn. Returns `None` if already
    /// consumed or nothing is staged. Drains both hidden blocks and typed
    /// parts, resolves parts into prompt blocks, and marks context consumed.
    fn take_pending_context_for_turn(&mut self) -> Option<PendingContextTurn> {
        self.take_pending_context_for_turn_with(Self::capture_special_context_block_for_part)
    }

    /// Variant of `take_pending_context_for_turn` that lets tests inject a
    /// deterministic special-block resolver.
    fn take_pending_context_for_turn_with<F>(
        &mut self,
        mut special_block_resolver: F,
    ) -> Option<PendingContextTurn>
    where
        F: FnMut(&crate::ai::message_parts::AiContextPart) -> Result<Option<ContentBlock>, String>,
    {
        let has_pending_parts = !self.pending_context_parts.is_empty();
        let has_pending_blocks = !self.pending_context_blocks.is_empty();

        if self.pending_context_consumed || (!has_pending_parts && !has_pending_blocks) {
            return None;
        }

        let blocks = std::mem::take(&mut self.pending_context_blocks);
        // Clone parts so the chip remains visible after submit.
        // The `pending_context_consumed` flag prevents re-resolution.
        let pending_parts = self.pending_context_parts.clone();
        let consumed_hidden_block_count = blocks.len();
        let consumed_part_count = pending_parts.len();

        let resolved_pending_context = if pending_parts.is_empty() {
            ResolvedPendingContext {
                blocks: Vec::new(),
                receipt: crate::ai::message_parts::ContextResolutionReceipt {
                    attempted: 0,
                    resolved: 0,
                    failures: Vec::new(),
                    prompt_prefix: String::new(),
                },
            }
        } else {
            Self::resolve_pending_context_parts_with(&pending_parts, |part| {
                special_block_resolver(part)
            })
        };
        let consumed_special_block_count = resolved_pending_context.blocks.len();
        let receipt = resolved_pending_context.receipt;
        let mut blocks = blocks;
        blocks.extend(resolved_pending_context.blocks);

        self.pending_context_consumed = true;
        self.pending_ambient_context_enabled = false;

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_pending_context_consumed",
            consumed_part_count,
            consumed_hidden_block_count,
            consumed_special_block_count,
            resolved_part_count = receipt.resolved,
            failed_part_count = receipt.failures.len(),
        );

        let attachments = pending_parts
            .iter()
            .map(AgentChatMessageAttachment::from_part)
            .collect();

        Some(PendingContextTurn {
            blocks,
            receipt,
            attachments,
        })
    }

    /// Build the content blocks for a turn, consuming staged context on first use.
    ///
    /// Delegates to `take_pending_context_for_turn()` for one-shot consumption.
    /// When context is present, the user's text is wrapped with a clear
    /// `--- USER REQUEST ---` marker so the agent distinguishes ambient context
    /// from the actual user intent.
    #[cfg_attr(test, allow(dead_code))]
    pub(super) fn prepare_turn_blocks(&mut self, input: &str) -> Vec<ContentBlock> {
        self.prepare_turn_blocks_with_receipt(input).blocks
    }

    fn should_stage_brain_recall(&self) -> bool {
        self.profile_id == crate::ai::agent_chat::profiles::BUILTIN_BRAIN_PROFILE_ID
    }

    /// Build the content blocks for a turn AND return the resolution receipt
    /// so callers can surface partial-failure feedback.
    fn prepare_turn_blocks_with_receipt(&mut self, input: &str) -> PreparedTurnBlocks {
        self.prepare_turn_blocks_with_receipt_using(
            input,
            |query| crate::brain::recall_context_block(query).ok().flatten(),
            crate::brain::record_ask_signals,
        )
    }

    fn prepare_turn_blocks_with_receipt_using<R, S>(
        &mut self,
        input: &str,
        recall_context: R,
        record_ask_signals: S,
    ) -> PreparedTurnBlocks
    where
        R: FnOnce(&str) -> Option<String>,
        S: FnOnce(&str),
    {
        let mut blocks = Vec::new();

        // --- Brain recall: stage relevant local memory for the Brain profile ---
        // Lexical + attention-signal retrieval over the brain store (notes,
        // past chat turns). Milliseconds on sqlite; hard-capped output so it
        // can never crowd a prompt. Empty recall stages nothing.
        if self.should_stage_brain_recall() {
            let brain_block = recall_context(input);
            if let Some(recall) = brain_block {
                tracing::info!(
                    target: "script_kit::brain",
                    event = "agent_chat_brain_recall_staged",
                    profile_id = %self.profile_id,
                    chars = recall.len(),
                );
                blocks.push(ContentBlock::Text(TextContent::new(recall)));
            }
            record_ask_signals(input);
        }

        if let Some(turn) = self.take_pending_context_for_turn() {
            let receipt = turn.receipt;
            let attachments = turn.attachments;
            blocks.extend(turn.blocks);

            if receipt.attempted > 0 {
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "agent_chat_submit_resolved_context_parts",
                    attempted = receipt.attempted,
                    resolved = receipt.resolved,
                    failures = receipt.failures.len(),
                );
            }

            if !receipt.prompt_prefix.is_empty() {
                blocks.push(ContentBlock::Text(TextContent::new(
                    receipt.prompt_prefix.clone(),
                )));
            }

            blocks.push(ContentBlock::Text(TextContent::new(format!(
                "--- USER REQUEST ---\n{input}"
            ))));
            return PreparedTurnBlocks {
                blocks,
                receipt: Some(receipt),
                attachments,
            };
        }

        if blocks.is_empty() {
            blocks.push(ContentBlock::Text(TextContent::new(input)));
        } else {
            blocks.push(ContentBlock::Text(TextContent::new(format!(
                "--- USER REQUEST ---\n{input}"
            ))));
        }
        PreparedTurnBlocks {
            blocks,
            receipt: None,
            attachments: Vec::new(),
        }
    }

    fn completed_chat_turn_ingest(
        &self,
        history_trace_label: Option<String>,
    ) -> Option<CompletedChatTurnIngest> {
        let user_text = self
            .messages
            .iter()
            .rev()
            .find(|m| matches!(m.role, AgentChatThreadMessageRole::User))
            .map(|m| m.body.to_string())?;
        let assistant_text = self
            .messages
            .iter()
            .rev()
            .find(|m| matches!(m.role, AgentChatThreadMessageRole::Assistant))
            .map(|m| m.body.to_string())
            .unwrap_or_default();
        let trace_label = history_trace_label.unwrap_or_else(|| {
            self.messages
                .iter()
                .find(|m| matches!(m.role, AgentChatThreadMessageRole::User))
                .map(|m| m.body.to_string())
                .unwrap_or_default()
        });
        let turn_index = self
            .messages
            .iter()
            .filter(|m| matches!(m.role, AgentChatThreadMessageRole::User))
            .count()
            .saturating_sub(1);

        Some(CompletedChatTurnIngest {
            thread_id: self.ui_thread_id.clone(),
            turn_index,
            user_text,
            assistant_text,
            trace_label,
        })
    }

    /// Update `context_bootstrap_note` with a partial-failure summary when
    /// some provider-backed mentions failed to resolve.
    fn set_context_resolution_note(
        &mut self,
        receipt: Option<&crate::ai::message_parts::ContextResolutionReceipt>,
    ) {
        let Some(receipt) = receipt else {
            self.context_bootstrap_note = None;
            return;
        };
        if receipt.failures.is_empty() {
            self.context_bootstrap_note = None;
            return;
        }

        let labels: Vec<&str> = receipt
            .failures
            .iter()
            .map(|failure| failure.label.as_str())
            .collect();
        let sources: Vec<&str> = receipt
            .failures
            .iter()
            .map(|failure| failure.source.as_str())
            .collect();

        self.context_bootstrap_note = Some(
            format!(
                "{} context attachment{} unavailable \u{00b7} {}",
                receipt.failures.len(),
                if receipt.failures.len() == 1 { "" } else { "s" },
                labels.join(", "),
            )
            .into(),
        );

        tracing::warn!(
            target: "script_kit::tab_ai",
            event = "agent_chat_context_resolution_partial_failure",
            failure_count = receipt.failures.len(),
            labels = ?labels,
            sources = ?sources,
        );
    }

    /// Spawn a task that pumps events from the Agent Chat worker into thread state.
    fn bind_stream(&mut self, rx: AgentChatEventRx, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        let generation = self.transcript_generation;
        self.stream_task = Some(cx.spawn(async move |_this, cx| {
            let mut terminal_event_seen = false;
            while let Ok(event) = rx.recv().await {
                let should_stop = matches!(
                    event,
                    AgentChatEvent::TurnFinished { .. } | AgentChatEvent::Failed { .. }
                );

                let entity_alive = entity.upgrade().is_some();
                if !entity_alive {
                    break;
                }

                let entity_ref = entity.clone();
                cx.update(|cx| {
                    if let Some(entity) = entity_ref.upgrade() {
                        entity.update(cx, |this, cx| {
                            if this.transcript_generation != generation {
                                tracing::debug!(
                                    target: "script_kit::tab_ai",
                                    event = "agent_chat_stream_event_discarded_stale_generation",
                                    expected_generation = generation,
                                    actual_generation = this.transcript_generation,
                                );
                                return;
                            }
                            this.apply_event(event, cx);
                        });
                    }
                });

                if should_stop {
                    terminal_event_seen = true;
                    break;
                }
            }

            if !terminal_event_seen {
                let entity_ref = entity.clone();
                cx.update(|cx| {
                    if let Some(entity) = entity_ref.upgrade() {
                        entity.update(cx, |this, cx| {
                            if this.transcript_generation != generation {
                                return;
                            }
                            if this.finish_stream_closed_without_terminal() {
                                cx.notify();
                            }
                        });
                    }
                });
            }
        }));
    }

    fn bump_transcript_generation(&mut self, reason: &'static str) {
        self.transcript_generation = self.transcript_generation.wrapping_add(1);
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_thread_transcript_generation_bumped",
            reason,
            generation = self.transcript_generation,
        );
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
                            this.status = AgentChatThreadStatus::WaitingForPermission;
                            let body = this.permission_notification_body(
                                this.pending_permission
                                    .as_ref()
                                    .expect("request was just set"),
                            );
                            let request_id = this
                                .pending_permission
                                .as_ref()
                                .map(|request| request.id)
                                .unwrap_or_default();
                            this.maybe_notify_agent_chat_event(
                                AgentChatNotificationEvent::WaitingForPermission { request_id },
                                "Agent Chat — approval needed",
                                body,
                            );
                            cx.notify();
                        });
                    }
                });
            }
        }));
    }

    fn start_streaming_text_drain_if_needed(&mut self, cx: &mut Context<Self>) {
        if self.streaming_text_buffer.is_empty() || self.streaming_text_drain_task.is_some() {
            return;
        }

        let generation = self.transcript_generation;
        self.streaming_text_drain_task = Some(cx.spawn(async move |this, cx| loop {
            Timer::after(std::time::Duration::from_millis(16)).await;

            let should_continue = this
                .update(cx, |this, cx| {
                    if this.transcript_generation != generation {
                        this.streaming_text_buffer.flush_all();
                        this.streaming_text_drain_task = None;
                        return false;
                    }

                    let changed = this.drain_streaming_text_once();
                    if changed {
                        cx.notify();
                    }

                    if this.streaming_text_buffer.is_empty() {
                        this.streaming_text_drain_task = None;
                        false
                    } else {
                        true
                    }
                })
                .unwrap_or(false);

            if !should_continue {
                break;
            }
        }));
    }

    fn drain_streaming_text_once(&mut self) -> bool {
        let budget = self.streaming_text_buffer.drain_budget_for_tick();
        let Some(delta) = self.streaming_text_buffer.drain_next(budget) else {
            return false;
        };
        self.append_assistant_stream_delta(delta)
    }

    fn append_assistant_stream_delta(&mut self, delta: String) -> bool {
        let accumulated =
            self.accumulated_text_after_append(AgentChatThreadMessageRole::Assistant, &delta);
        let changed = self.append_chunk(AgentChatThreadMessageRole::Assistant, delta.clone());
        if changed {
            crate::ai::subscriptions::publish_stream_chunk(&self.ui_thread_id, delta, accumulated);
        }
        changed
    }

    fn flush_streaming_text_buffer(&mut self) -> bool {
        let delta = self.streaming_text_buffer.flush_all();
        self.streaming_text_drain_task = None;
        if delta.is_empty() {
            return false;
        }
        self.append_assistant_stream_delta(delta)
    }

    /// Apply a single Agent Chat event to thread state.
    ///
    /// Streaming text deltas coalesce into stable messages via `append_chunk`.
    /// Plan, mode, and command updates are persisted in dedicated fields so the
    /// view can render them as first-class UI strips without reparsing text.
    /// Tool calls are tracked by ID and updated in-place.
    ///
    /// Only calls `cx.notify()` when state actually changes, avoiding redundant
    /// repaints for duplicate plan, mode, command, or tool-call updates.
    fn apply_event(&mut self, event: AgentChatEvent, cx: &mut Context<Self>) {
        let mut changed = false;

        match event {
            AgentChatEvent::UserMessageDelta(chunk) => {
                changed |= self.append_chunk(AgentChatThreadMessageRole::System, chunk);
                changed |= self.set_status(AgentChatThreadStatus::Streaming);
            }
            AgentChatEvent::AgentMessageDelta(chunk) => {
                self.log_time_to_first_token("assistant_message");
                if !chunk.is_empty() {
                    self.streaming_text_buffer.push_chunk(chunk);
                    self.start_streaming_text_drain_if_needed(cx);
                }
                changed |= self.set_status(AgentChatThreadStatus::Streaming);
            }
            AgentChatEvent::AgentThoughtDelta(chunk) => {
                self.log_time_to_first_token("agent_thought");
                changed |= self.append_chunk(AgentChatThreadMessageRole::Thought, chunk);
                changed |= self.set_status(AgentChatThreadStatus::Streaming);
            }
            AgentChatEvent::ToolCallStarted {
                tool_call_id,
                title,
                status,
                tool_name,
                raw_input,
            } => {
                changed |=
                    self.upsert_tool_call_start(tool_call_id, title, status, tool_name, raw_input);
                changed |= self.set_status(AgentChatThreadStatus::Streaming);
            }
            AgentChatEvent::ToolCallUpdated {
                tool_call_id,
                title,
                status,
                body,
                raw_input,
                diff,
                is_error,
            } => {
                changed |= self.apply_tool_call_update(
                    tool_call_id,
                    title,
                    status,
                    body,
                    raw_input,
                    diff,
                    is_error,
                );
                changed |= self.set_status(AgentChatThreadStatus::Streaming);
            }
            AgentChatEvent::PlanUpdated { entries } => {
                if self.active_plan_entries != entries {
                    self.active_plan_entries = entries;
                    changed = true;
                }
                changed |= self.set_status(AgentChatThreadStatus::Streaming);
            }
            AgentChatEvent::AvailableCommandsUpdated { command_names } => {
                if self.available_commands != command_names {
                    self.available_commands = command_names;
                    changed = true;
                }
            }
            AgentChatEvent::ModeChanged { mode_id } => {
                if self.active_mode_id.as_deref() != Some(mode_id.as_str()) {
                    self.active_mode_id = Some(mode_id);
                    changed = true;
                }
            }
            AgentChatEvent::UsageUpdated {
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
            AgentChatEvent::ModelsAvailable {
                current_model_id,
                models,
            } => {
                changed |= self.apply_agent_models(current_model_id, models);
            }
            AgentChatEvent::ForkPointsAvailable { entries } => {
                if self.fork_points != entries {
                    self.fork_points = entries;
                    changed = true;
                }
            }
            AgentChatEvent::ForkCompleted { text } => {
                changed |= self.flush_streaming_text_buffer();
                changed |= self.apply_fork_completed(text, cx);
            }
            AgentChatEvent::TurnFinished { .. } => {
                changed |= self.flush_streaming_text_buffer();
                if self.pending_permission.take().is_some() {
                    changed = true;
                }
                changed |= self.set_status(AgentChatThreadStatus::Idle);
                if let Some(message) =
                    self.latest_message_with_role(AgentChatThreadMessageRole::Assistant)
                {
                    let message_id = message.id;
                    let full_content = message.body.to_string();
                    if !full_content.is_empty() {
                        self.maybe_notify_agent_chat_event(
                            AgentChatNotificationEvent::TurnFinished,
                            "Agent Chat — response ready",
                            full_content.clone(),
                        );
                        crate::ai::subscriptions::publish_stream_complete(
                            &self.ui_thread_id,
                            message_id.to_string(),
                            full_content.clone(),
                            None,
                        );
                        self.publish_sdk_new_message(
                            message_id,
                            AgentChatThreadMessageRole::Assistant,
                            full_content,
                        );
                    }
                }

                // Save conversation summary + full messages to history.
                // Build a rich index entry from the full conversation so
                // search_history() can match on later transcript content.
                let history_trace_label = if self
                    .messages
                    .iter()
                    .any(|m| matches!(m.role, AgentChatThreadMessageRole::User))
                {
                    let timestamp = chrono::Utc::now().to_rfc3339();
                    let existing_custom_title =
                        super::history::load_conversation(&self.ui_thread_id)
                            .and_then(|conversation| conversation.custom_title);
                    let conversation = super::history::SavedConversation {
                        session_id: self.ui_thread_id.clone(),
                        timestamp,
                        custom_title: existing_custom_title.clone(),
                        messages: self
                            .messages
                            .iter()
                            .map(|m| super::history::SavedMessage {
                                role: format!("{:?}", m.role),
                                body: m.body.to_string(),
                            })
                            .collect(),
                    };
                    super::history::save_conversation(&conversation);
                    self.maybe_spawn_auto_title(&conversation);

                    super::history::build_history_entry(&conversation).map(|entry| {
                        tracing::info!(
                            target: "script_kit::tab_ai",
                            event = "agent_chat_history_index_entry_built",
                            session_id = %entry.session_id,
                            title = %entry.title_display(),
                            preview_len = entry.preview.len(),
                            message_count = entry.message_count,
                        );
                        super::history::save_history_entry(&entry);
                        entry.title_display().to_string()
                    })
                } else {
                    None
                };

                // The finished turn appended a user message; refresh the
                // rewind checkpoints so Cmd+K can offer it for editing.
                self.refresh_fork_points(cx);

                // --- Brain ingestion: every finished turn becomes memory ---
                // The last user/assistant exchange is written into the brain
                // store (hash-guarded, idempotent) on a background thread so
                // future turns — in any thread — can recall it.
                if let Some(payload) = self.completed_chat_turn_ingest(history_trace_label) {
                    let _ = std::thread::Builder::new()
                        .name("script-kit-brain-ingest".to_string())
                        .spawn(move || {
                            crate::brain::day_trace::maybe_append_agent_chat_trace(
                                &payload.thread_id,
                                &payload.trace_label,
                            );
                            if let Err(error) = crate::brain::ingest_chat_turn(
                                &payload.thread_id,
                                payload.turn_index,
                                &payload.user_text,
                                &payload.assistant_text,
                            ) {
                                tracing::debug!(
                                    target: "script_kit::brain",
                                    error = %error,
                                    "brain chat ingestion failed"
                                );
                            }
                        });
                }
                if let Err(error) = self.submit_next_queued_if_ready(cx) {
                    tracing::warn!(
                        target: "script_kit::tab_ai",
                        event = "agent_chat_queue_auto_submit_failed",
                        error = %error,
                    );
                }
            }
            AgentChatEvent::SetupRequired {
                reason,
                auth_methods,
            } => {
                let current_requirements = self.current_setup_requirements();
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "agent_chat_runtime_setup_session_armed",
                    reason = %reason,
                    auth_method_count = auth_methods.len(),
                    selected_agent_id = self.selected_agent.as_ref().map(|a| a.id.as_ref()),
                    available_agent_count = self.available_agents.len(),
                    needs_embedded_context = current_requirements.needs_embedded_context,
                    needs_image = current_requirements.needs_image,
                );
                tracing::info!(
                    target: "script_kit::tab_ai",
                    event = "agent_chat_runtime_setup_requirements_preserved",
                    original_needs_embedded_context = self.launch_requirements.needs_embedded_context,
                    original_needs_image = self.launch_requirements.needs_image,
                    current_needs_embedded_context = current_requirements.needs_embedded_context,
                    current_needs_image = current_requirements.needs_image,
                );
                self.setup_state = Some(
                    super::setup_state::AgentChatInlineSetupState::from_runtime_setup_required(
                        self.selected_agent.clone(),
                        self.available_agents.clone(),
                        current_requirements,
                        &reason,
                        &auth_methods,
                    ),
                );
                changed |= self.set_status(AgentChatThreadStatus::Error);
            }
            AgentChatEvent::Failed { error } => {
                let _ = self.flush_streaming_text_buffer();
                self.maybe_notify_agent_chat_event(
                    AgentChatNotificationEvent::Failed,
                    "Agent Chat — turn failed",
                    error.clone(),
                );
                crate::ai::subscriptions::publish_error(
                    Some(&self.ui_thread_id),
                    "AGENT_CHAT_TURN_FAILED".to_string(),
                    error.clone(),
                );
                let _ = self.pending_permission.take();
                let can_retry = self.last_user_turn_text().is_some();
                self.active_callout = Some(AgentChatCallout::failed(error.clone(), can_retry));
                changed = true;
                changed |= self.push_message(AgentChatThreadMessageRole::Error, error);
                changed |= self.set_status(AgentChatThreadStatus::Error);
            }
        }

        if changed {
            cx.notify();
        }
    }

    fn finish_stream_closed_without_terminal(&mut self) -> bool {
        if !matches!(
            self.status,
            AgentChatThreadStatus::Streaming | AgentChatThreadStatus::WaitingForPermission
        ) {
            return false;
        }

        self.flush_streaming_text_buffer();
        let had_pending_permission = self.pending_permission.take().is_some();
        let assistant = self.latest_message_with_role(AgentChatThreadMessageRole::Assistant);
        let assistant_id = assistant.map(|message| message.id);
        let assistant_text = assistant
            .map(|message| message.body.to_string())
            .unwrap_or_default();
        let had_assistant_text = !assistant_text.trim().is_empty();

        tracing::warn!(
            target: "script_kit::tab_ai",
            event = "agent_chat_stream_closed_without_terminal_event",
            ui_thread = %self.ui_thread_id,
            had_pending_permission,
            had_assistant_text,
            message_count = self.messages.len(),
        );

        if had_assistant_text {
            self.status = AgentChatThreadStatus::Idle;
            if let Some(message_id) = assistant_id {
                crate::ai::subscriptions::publish_stream_complete(
                    &self.ui_thread_id,
                    message_id.to_string(),
                    assistant_text.clone(),
                    None,
                );
                self.publish_sdk_new_message(
                    message_id,
                    AgentChatThreadMessageRole::Assistant,
                    assistant_text,
                );
            }
        } else {
            let message = "Agent stream ended before sending a completion event.".to_string();
            self.push_message(AgentChatThreadMessageRole::Error, message);
            self.status = AgentChatThreadStatus::Error;
        }

        true
    }

    /// One-shot time-to-first-token log for the current turn. Makes warm/cold
    /// launch latency measurable from logs (`agent_chat_time_to_first_token`).
    fn log_time_to_first_token(&mut self, first_event: &'static str) {
        if !self.ttft_pending {
            return;
        }
        self.ttft_pending = false;
        let Some(started_at) = self.stream_started_at else {
            return;
        };
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_time_to_first_token",
            ui_thread_id = %self.ui_thread_id,
            first_event,
            elapsed_ms = started_at.elapsed().as_millis() as u64,
        );
    }

    /// Set the thread status, returning `true` if it actually changed.
    fn set_status(&mut self, next: AgentChatThreadStatus) -> bool {
        if self.status == next {
            return false;
        }
        // Track streaming start time.
        if matches!(next, AgentChatThreadStatus::Streaming)
            && !matches!(self.status, AgentChatThreadStatus::Streaming)
        {
            self.stream_started_at = Some(std::time::Instant::now());
        } else if !matches!(next, AgentChatThreadStatus::Streaming) {
            self.stream_started_at = None;
        }
        self.status = next;
        true
    }

    /// Push a new message with an auto-allocated ID. Returns `true` always.
    fn push_message(
        &mut self,
        role: AgentChatThreadMessageRole,
        body: impl Into<SharedString>,
    ) -> bool {
        let id = self.alloc_id();
        self.messages
            .push(AgentChatThreadMessage::new(id, role, body));
        true
    }

    fn sdk_role(role: AgentChatThreadMessageRole) -> Option<&'static str> {
        match role {
            AgentChatThreadMessageRole::User => Some("user"),
            AgentChatThreadMessageRole::Assistant => Some("assistant"),
            AgentChatThreadMessageRole::System => Some("system"),
            AgentChatThreadMessageRole::Error => Some("system"),
            AgentChatThreadMessageRole::Thought | AgentChatThreadMessageRole::Tool => None,
        }
    }

    fn publish_sdk_new_message(&self, id: u64, role: AgentChatThreadMessageRole, content: String) {
        let Some(role) = Self::sdk_role(role) else {
            return;
        };
        crate::ai::subscriptions::publish_new_message(
            &self.ui_thread_id,
            AiMessageInfo {
                id: id.to_string().into(),
                role: role.to_string().into(),
                content: content.into(),
                created_at: chrono::Utc::now().to_rfc3339(),
                tokens_used: None,
            },
        );
    }

    fn latest_message_with_role(
        &self,
        role: AgentChatThreadMessageRole,
    ) -> Option<&AgentChatThreadMessage> {
        self.messages
            .iter()
            .rev()
            .find(|message| message.role == role)
    }

    fn accumulated_text_after_append(
        &self,
        role: AgentChatThreadMessageRole,
        chunk: &str,
    ) -> String {
        let mut accumulated = self
            .messages
            .last()
            .filter(|message| message.role == role)
            .map(|message| message.body.to_string())
            .unwrap_or_default();
        accumulated.push_str(chunk);
        accumulated
    }

    /// Append a text chunk to the last message if it has the same role,
    /// otherwise create a new message. This coalesces streaming deltas.
    /// Returns `true` if state changed (i.e. chunk was non-empty).
    fn append_chunk(&mut self, role: AgentChatThreadMessageRole, chunk: String) -> bool {
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
        self.messages
            .push(AgentChatThreadMessage::new(id, role, chunk));
        true
    }

    /// Re-derive the formatted body and structured card metadata for the
    /// message backing a tool call slot.
    fn sync_tool_call_message(&mut self, slot: usize) {
        let tool_call = &self.active_tool_calls[slot];
        let new_body =
            Self::format_tool_call_body(&tool_call.title, &tool_call.status, &tool_call.body);
        let meta = tool_call.card_meta();
        let message_index = tool_call.message_index;
        if let Some(msg) = self.messages.get_mut(message_index) {
            msg.body = new_body.into();
            msg.tool_meta = Some(meta);
        }
    }

    /// Insert or update a tool call from a `ToolCallStarted` event.
    /// Uses `tool_call_lookup` for O(1) access. Returns `true` if state changed.
    fn upsert_tool_call_start(
        &mut self,
        tool_call_id: String,
        title: String,
        status: String,
        tool_name: Option<String>,
        raw_input: Option<serde_json::Value>,
    ) -> bool {
        let subject = raw_input
            .as_ref()
            .and_then(super::tool_card::subject_from_args);
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
            if tool_name.is_some() && existing.tool_name != tool_name {
                existing.tool_name = tool_name;
                changed = true;
            }
            if subject.is_some() && existing.subject != subject {
                existing.subject = subject;
                changed = true;
            }
            if changed {
                self.sync_tool_call_message(slot);
            }
            return changed;
        }

        let message_id = self.alloc_id();
        let message_index = self.messages.len();
        let message_body = format!("{title}\n{status}");
        self.messages
            .push(AgentChatThreadMessage::with_tool_call_id(
                message_id,
                AgentChatThreadMessageRole::Tool,
                message_body,
                tool_call_id.clone(),
            ));

        let slot = self.active_tool_calls.len();
        self.active_tool_calls.push(AgentChatToolCallState {
            tool_call_id: tool_call_id.clone(),
            title,
            status,
            body: None,
            tool_name,
            subject,
            diff: None,
            is_error: false,
            message_id,
            message_index,
        });
        self.tool_call_lookup.insert(tool_call_id, slot);
        self.sync_tool_call_message(slot);
        true
    }

    /// Apply a `ToolCallUpdated` event, updating tracked state and message in-place.
    /// Uses `tool_call_lookup` for O(1) access. Returns `true` if state changed.
    #[allow(clippy::too_many_arguments)]
    fn apply_tool_call_update(
        &mut self,
        tool_call_id: String,
        title: Option<String>,
        status: Option<String>,
        body: Option<String>,
        raw_input: Option<serde_json::Value>,
        diff: Option<String>,
        is_error: bool,
    ) -> bool {
        let subject = raw_input
            .as_ref()
            .and_then(super::tool_card::subject_from_args);
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
            if subject.is_some() && tool_call.subject != subject {
                tool_call.subject = subject;
                changed = true;
            }
            if diff.is_some() && tool_call.diff != diff {
                tool_call.diff = diff;
                changed = true;
            }
            if is_error && !tool_call.is_error {
                tool_call.is_error = true;
                changed = true;
            }

            if changed {
                self.sync_tool_call_message(slot);
            }
            return changed;
        }

        // Orphan update — create a standalone tool call entry.
        let title = title.unwrap_or_else(|| "Tool".to_string());
        let status = status.unwrap_or_else(|| "running".to_string());
        let message_id = self.alloc_id();
        let message_index = self.messages.len();
        let message_body = Self::format_tool_call_body(&title, &status, &body);
        self.messages
            .push(AgentChatThreadMessage::with_tool_call_id(
                message_id,
                AgentChatThreadMessageRole::Tool,
                message_body,
                tool_call_id.clone(),
            ));

        let slot = self.active_tool_calls.len();
        self.active_tool_calls.push(AgentChatToolCallState {
            tool_call_id: tool_call_id.clone(),
            title,
            status,
            body,
            tool_name: None,
            subject,
            diff,
            is_error,
            message_id,
            message_index,
        });
        self.tool_call_lookup.insert(tool_call_id, slot);
        self.sync_tool_call_message(slot);
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

    /// Capability requirements preserved from the original launch context.
    pub(crate) fn launch_requirements(&self) -> super::preflight::AgentChatLaunchRequirements {
        self.launch_requirements
    }

    /// Currently selected Agent Chat agent ID for this live thread.
    pub(crate) fn selected_agent_id(&self) -> Option<&str> {
        self.selected_agent.as_ref().map(|agent| agent.id.as_ref())
    }

    /// Replace the selected agent on the live thread (used during runtime
    /// recovery when the user picks a different agent in the setup card).
    pub(crate) fn replace_selected_agent(
        &mut self,
        next: Option<super::catalog::AgentChatAgentCatalogEntry>,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_thread_selected_agent_replaced",
            selected_agent_id = next.as_ref().map(|a| a.id.as_ref()),
            needs_embedded_context = self.launch_requirements.needs_embedded_context,
            needs_image = self.launch_requirements.needs_image,
        );
        self.selected_agent = next;
        cx.notify();
    }

    /// Runtime setup state armed by `AgentChatEvent::SetupRequired`.
    /// When `Some`, the view should render the inline setup recovery card.
    pub(crate) fn setup_state(&self) -> Option<&super::setup_state::AgentChatInlineSetupState> {
        self.setup_state.as_ref()
    }

    /// Derive runtime setup requirements from the live thread state.
    ///
    /// Unions the original `launch_requirements` with the current pending
    /// context parts and blocks so that later-added `@screenshot` or context
    /// chips are reflected when the thread re-enters `SetupRequired`.
    pub(crate) fn current_setup_requirements(
        &self,
    ) -> super::preflight::AgentChatLaunchRequirements {
        let needs_embedded_context = self.launch_requirements.needs_embedded_context
            || !self.pending_context_parts.is_empty()
            || !self.pending_context_blocks.is_empty();

        let needs_image = self.launch_requirements.needs_image
            || self
                .pending_context_parts
                .iter()
                .any(Self::is_explicit_screenshot_part);

        super::preflight::AgentChatLaunchRequirements {
            needs_embedded_context,
            needs_image,
        }
    }

    /// Replace the runtime setup state (used by the view after agent re-selection).
    pub(crate) fn replace_setup_state(
        &mut self,
        next: super::setup_state::AgentChatInlineSetupState,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_thread_setup_state_replaced",
            title = %next.title,
            selected_agent_id = next.selected_agent.as_ref().map(|a| a.id.as_ref()),
            catalog_count = next.catalog_entries.len(),
        );
        self.setup_state = Some(next);
        cx.notify();
    }

    /// Full agent catalog for runtime recovery.
    pub(crate) fn available_agents(&self) -> &[super::catalog::AgentChatAgentCatalogEntry] {
        &self.available_agents
    }

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

    /// Unique UI thread identifier (used for automation window targeting).
    pub(crate) fn ui_thread_id(&self) -> &str {
        &self.ui_thread_id
    }

    /// Display name for the agent (e.g. "Claude Code").
    pub(crate) fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Display name for the selected Agent Chat profile, or the agent name.
    pub(crate) fn profile_display(&self) -> &str {
        self.profile_display_name
            .as_deref()
            .unwrap_or(&self.display_name)
    }

    pub(crate) fn profile_icon_name(&self) -> Option<&str> {
        self.profile_icon_name.as_deref()
    }

    pub(crate) fn set_profile_display(
        &mut self,
        profile_id: String,
        profile_display_name: SharedString,
        profile_icon_name: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.profile_id = profile_id;
        self.profile_display_name = Some(profile_display_name);
        self.profile_icon_name = profile_icon_name;
        cx.notify();
    }

    /// Short display name for the currently selected model, or the agent name if none selected.
    pub(crate) fn selected_model_display(&self) -> &str {
        self.selected_model_display_name
            .as_deref()
            .unwrap_or(&self.display_name)
    }

    /// Available models for this agent.
    pub(crate) fn available_models(&self) -> &[super::config::AgentChatModelEntry] {
        &self.available_models
    }

    /// Replace the available model list with the agent's live advertisement
    /// from `session/new`. Preserves the user's current selection if it is
    /// still in the new list; otherwise falls back to the agent's declared
    /// current model, or the first entry. Returns `true` if anything changed.
    fn apply_agent_models(
        &mut self,
        current_model_id: Option<String>,
        models: Vec<super::config::AgentChatModelEntry>,
    ) -> bool {
        if models.is_empty() {
            return false;
        }

        let mut changed = self.available_models != models;
        self.available_models = models;

        let selection_still_valid = self
            .selected_model_id
            .as_deref()
            .map(|sel| self.available_models.iter().any(|m| m.id == sel))
            .unwrap_or(false);

        if !selection_still_valid {
            let fallback = current_model_id
                .as_deref()
                .and_then(|id| self.available_models.iter().find(|m| m.id == id))
                .or_else(|| self.available_models.first());
            if let Some(entry) = fallback {
                self.selected_model_id = Some(entry.id.clone());
                self.selected_model_display_name = Some(SharedString::from(
                    entry
                        .display_name
                        .clone()
                        .unwrap_or_else(|| entry.id.clone()),
                ));
                changed = true;
            }
        }

        changed
    }

    /// Currently selected model ID, if any.
    pub(crate) fn selected_model_id(&self) -> Option<&str> {
        self.selected_model_id.as_deref()
    }

    /// Start a view-owned auxiliary turn that must not mutate this thread's
    /// transcript, status, permissions, or stream_task.
    ///
    /// Focused-text variation generation uses this for conservative/creative
    /// candidates while the balanced candidate remains the canonical thread turn.
    pub(crate) fn start_auxiliary_turn(
        &self,
        ui_thread_id: String,
        blocks: Vec<ContentBlock>,
    ) -> Result<crate::ai::agent_chat::runtime::IsolatedTurnHandle, String> {
        self.connection
            .start_isolated_turn(AgentChatTurnRequest {
                ui_thread_id,
                cwd: self.cwd.clone(),
                blocks,
                model_id: self.selected_model_id.clone(),
            })
            .map_err(|error| error.to_string())
    }

    /// Fire-and-forget: ask the Agent Chat worker to create-or-reuse the session for
    /// this thread and emit a fresh `ModelsAvailable` event. Called when the
    /// user invokes the actions dialog so the Change Model picker reflects the
    /// agent's live catalog (including models released after the hardcoded
    /// fallback was written).
    ///
    /// If the worker is unreachable the call is a no-op; the picker will fall
    /// back to whatever `available_models` already held.
    /// User messages the live session can rewind to, in conversation order.
    pub(crate) fn fork_points(&self) -> &[super::events::AgentChatForkPoint] {
        &self.fork_points
    }

    /// Resolve the Pi fork point for a transcript user message. Primary mapping
    /// is conversation ordinal: nth visible user message maps to nth Pi fork
    /// point. If the local transcript and Pi fork list are out of sync, fall
    /// back to exact user-message text matching.
    pub(crate) fn fork_point_for_message_id<'a>(
        messages: &[AgentChatThreadMessage],
        fork_points: &'a [super::events::AgentChatForkPoint],
        message_id: u64,
    ) -> Option<&'a super::events::AgentChatForkPoint> {
        let user_messages: Vec<&AgentChatThreadMessage> = messages
            .iter()
            .filter(|message| matches!(message.role, AgentChatThreadMessageRole::User))
            .collect();
        let user_ordinal = user_messages
            .iter()
            .position(|message| message.id == message_id)?;

        if user_messages.len() == fork_points.len() {
            return fork_points.get(user_ordinal);
        }

        let user_text = user_messages[user_ordinal].body.as_ref();
        fork_points.iter().find(|point| point.text == user_text)
    }

    /// Refresh the rewindable user-message list from the agent session.
    /// No-op (with a debug log) for connections without rewind support.
    pub(crate) fn refresh_fork_points(&mut self, cx: &mut Context<Self>) {
        let rx = match self.connection.fork_points() {
            Ok(rx) => rx,
            Err(error) => {
                tracing::debug!(
                    target: "script_kit::tab_ai",
                    event = "agent_chat_fork_points_unsupported",
                    error = %error,
                );
                return;
            }
        };
        self.spawn_fork_event_task(rx, "fork_points", cx);
    }

    /// Rewind the session to just before the given user message. On
    /// completion the transcript truncates at that message and the composer
    /// is prefilled with its text for editing. Rejected while a turn is
    /// streaming or another rewind is in flight.
    pub(crate) fn fork_to_message(&mut self, entry_id: &str, cx: &mut Context<Self>) -> bool {
        if matches!(
            self.status,
            AgentChatThreadStatus::Streaming | AgentChatThreadStatus::WaitingForPermission
        ) {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "agent_chat_fork_rejected_busy",
                status = ?self.status,
            );
            return false;
        }
        if self.pending_fork_ordinal.is_some() {
            return false;
        }
        let Some(ordinal) = self
            .fork_points
            .iter()
            .position(|point| point.entry_id == entry_id)
        else {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "agent_chat_fork_unknown_entry",
                entry_id,
            );
            return false;
        };
        let rx = match self.connection.fork_to_entry(entry_id.to_string()) {
            Ok(rx) => rx,
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::tab_ai",
                    event = "agent_chat_fork_request_failed",
                    error = %error,
                );
                return false;
            }
        };
        self.pending_fork_ordinal = Some(ordinal);
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_fork_requested",
            entry_id,
            ordinal,
        );
        self.spawn_fork_event_task(rx, "fork", cx);
        cx.notify();
        true
    }

    /// Pump fork RPC responses into `apply_event`, downgrading failures to a
    /// system note: a failed background refresh or rewind must not flip the
    /// thread into the turn-failure error state.
    fn spawn_fork_event_task(
        &self,
        rx: AgentChatEventRx,
        context_label: &'static str,
        cx: &mut Context<Self>,
    ) {
        let entity = cx.entity().downgrade();
        cx.spawn(async move |_this, cx| {
            while let Ok(event) = rx.recv().await {
                let Some(weak) = entity.upgrade() else {
                    break;
                };
                cx.update(|cx| {
                    weak.update(cx, |this, cx| {
                        let is_fork_event = matches!(
                            event,
                            AgentChatEvent::ForkPointsAvailable { .. }
                                | AgentChatEvent::ForkCompleted { .. }
                        );
                        if is_fork_event {
                            this.apply_event(event, cx);
                        } else if let AgentChatEvent::Failed { error } = event {
                            tracing::warn!(
                                target: "script_kit::tab_ai",
                                event = "agent_chat_fork_rpc_failed",
                                context = context_label,
                                error = %error,
                            );
                            if this.pending_fork_ordinal.take().is_some() {
                                this.push_system_message(format!("Rewind failed: {error}"), cx);
                            }
                        }
                    });
                });
            }
        })
        .detach();
    }

    /// Apply a completed session rewind: truncate the transcript at the
    /// forked user message and stage its text in the composer for editing.
    fn apply_fork_completed(&mut self, text: String, cx: &mut Context<Self>) -> bool {
        let Some(ordinal) = self.pending_fork_ordinal.take() else {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "agent_chat_fork_completed_without_request",
            );
            return false;
        };
        Self::truncate_messages_at_user_ordinal(&mut self.messages, ordinal);
        self.active_tool_calls.clear();
        self.tool_call_lookup.clear();
        self.transcript_generation = self.transcript_generation.wrapping_add(1);
        self.input.set_text(text.clone());
        self.input.set_cursor(text.chars().count());
        self.set_status(AgentChatThreadStatus::Idle);
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_fork_completed",
            ordinal,
            message_count = self.messages.len(),
            prefill_chars = text.chars().count(),
        );
        // Pi rebuilt the session with fresh entry ids; refetch the list.
        self.fork_points.clear();
        self.refresh_fork_points(cx);
        true
    }

    /// Drop the `ordinal`-th user message and everything after it.
    fn truncate_messages_at_user_ordinal(
        messages: &mut Vec<AgentChatThreadMessage>,
        ordinal: usize,
    ) {
        let mut seen = 0usize;
        for index in 0..messages.len() {
            if matches!(messages[index].role, AgentChatThreadMessageRole::User) {
                if seen == ordinal {
                    messages.truncate(index);
                    return;
                }
                seen += 1;
            }
        }
    }

    pub(crate) fn refresh_models(&mut self, cx: &mut Context<Self>) {
        let rx = match self
            .connection
            .prepare_session(self.ui_thread_id.clone(), self.cwd.clone())
        {
            Ok(rx) => rx,
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::tab_ai",
                    event = "agent_chat_refresh_models_channel_closed",
                    ui_thread = %self.ui_thread_id,
                    error = %error,
                );
                return;
            }
        };
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_refresh_models_requested",
            ui_thread = %self.ui_thread_id,
        );

        let entity = cx.entity().downgrade();
        cx.spawn(async move |_this, cx| {
            while let Ok(event) = rx.recv().await {
                let Some(weak) = entity.upgrade() else {
                    break;
                };
                cx.update(|cx| {
                    weak.update(cx, |this, cx| {
                        this.apply_event(event, cx);
                    });
                });
            }
        })
        .detach();
    }

    /// Select a model by ID. Updates the display name, persists to config, and notifies.
    pub(crate) fn select_model(&mut self, model_id: &str, cx: &mut Context<Self>) {
        if let Some(entry) = self.available_models.iter().find(|m| m.id == model_id) {
            self.selected_model_id = Some(entry.id.clone());
            self.selected_model_display_name = Some(SharedString::from(
                entry
                    .display_name
                    .clone()
                    .unwrap_or_else(|| entry.id.clone()),
            ));

            // Persist selection to config.ts (non-fatal).
            let id = entry.id.clone();
            std::thread::Builder::new()
                .name("agent_chat-save-model".into())
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

    pub(crate) fn toggle_favorite_model(&mut self, model_id: &str, cx: &mut Context<Self>) {
        super::favorite_models::toggle_favorite_model_id(model_id);
        cx.notify();
    }

    pub(crate) fn cycle_favorite_model(&mut self, cx: &mut Context<Self>) {
        let favorites = super::favorite_models::load_favorite_model_ids();
        if let Some(model_id) = super::favorite_models::next_favorite_model_id(
            self.selected_model_id(),
            &favorites,
            self.available_models(),
        ) {
            self.select_model(&model_id, cx);
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
        self.push_message(AgentChatThreadMessageRole::System, body);
        cx.notify();
    }

    /// Clear all messages for a fresh conversation within the same session.
    /// Also clears all pending context state so no stale chips or hidden
    /// blocks leak into the next conversation.
    pub(crate) fn clear_messages(&mut self, cx: &mut Context<Self>) {
        self.flush_streaming_text_buffer();
        self.messages.clear();
        self.active_plan_entries.clear();
        self.active_tool_calls.clear();
        self.tool_call_lookup.clear();
        self.clear_all_pending_context("clear_messages");
        self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
        self.context_bootstrap_note = None;

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_thread_cleared",
        );
        cx.notify();
    }

    /// Install a synthetic transcript state for no-token Agent Chat UI proof.
    pub(crate) fn apply_test_fixture(
        &mut self,
        phase: &str,
        user_text: Option<String>,
        assistant_text: Option<String>,
        message_count: Option<usize>,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        self.flush_streaming_text_buffer();
        let user_text = user_text.unwrap_or_else(|| "No-token activity fixture".to_string());
        let user_text = user_text.trim();
        if user_text.is_empty() {
            return Err("setAgentChatTestFixture requires non-empty userText".to_string());
        }

        self.stream_task = None;
        self.pending_permission = None;
        self.messages.clear();
        self.active_plan_entries.clear();
        self.active_tool_calls.clear();
        self.tool_call_lookup.clear();
        self.standing_approvals.clear();
        self.active_mode_id = None;
        self.available_commands.clear();
        self.usage_tokens = None;
        self.usage_cost_usd = None;
        self.input.clear();
        self.clear_all_pending_context("set_agent_chat_test_fixture");
        self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
        self.context_bootstrap_note = None;
        self.queued_submit_while_bootstrapping = false;

        if let Some(message_count) = message_count {
            let message_count = message_count.clamp(1, 2_000);
            let assistant_text = assistant_text
                .as_deref()
                .unwrap_or("Fixture assistant text with enough markdown to exercise layout.");
            for index in 0..message_count {
                let role = if index % 2 == 0 {
                    AgentChatThreadMessageRole::User
                } else {
                    AgentChatThreadMessageRole::Assistant
                };
                let seed = if matches!(role, AgentChatThreadMessageRole::User) {
                    user_text
                } else {
                    assistant_text
                };
                let body = format!(
                    "{seed}\n\n### Fixture turn {turn}\n\n- row: {row}\n- purpose: rapid transcript scroll performance\n- repeated detail: alpha beta gamma delta epsilon zeta eta theta\n\n```text\nfixture block {row}\nline one with enough width to require text layout\nline two with stable markdown parsing\n```\n\nThis paragraph intentionally gives TextViewState markdown a non-trivial body while keeping the fixture deterministic.",
                    turn = index / 2 + 1,
                    row = index + 1,
                );
                self.push_message(role, body);
            }
            self.set_status(AgentChatThreadStatus::Idle);
        } else {
            self.push_message(AgentChatThreadMessageRole::User, user_text.to_string());
            match phase {
                "awaitingFirstAssistantText" | "awaiting-first-assistant-text" | "awaiting" => {
                    self.set_status(AgentChatThreadStatus::Streaming);
                }
                "assistantText" | "assistant-text" | "text" => {
                    self.push_message(
                        AgentChatThreadMessageRole::Assistant,
                        assistant_text.unwrap_or_else(|| "Fixture assistant text.".to_string()),
                    );
                    self.set_status(AgentChatThreadStatus::Streaming);
                }
                "idle" => {
                    if let Some(text) = assistant_text {
                        self.push_message(AgentChatThreadMessageRole::Assistant, text);
                    }
                    self.set_status(AgentChatThreadStatus::Idle);
                }
                other => {
                    return Err(format!(
                        "unknown setAgentChatTestFixture phase {other:?}; expected awaitingFirstAssistantText, assistantText, or idle"
                    ));
                }
            }
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_test_fixture_applied",
            phase,
            requested_message_count = message_count.unwrap_or(0),
            message_count = self.messages.len(),
            awaiting_first_assistant_text = self.awaiting_first_assistant_text(),
        );
        cx.notify();
        Ok(())
    }

    pub(crate) fn load_kitchen_sink_fixture(&mut self, cx: &mut Context<Self>) {
        use crate::ai::agent_chat::ui::kitchen_sink_fixture::{
            agent_chat_kitchen_sink_fixture, AgentChatKitchenSinkFixtureRole,
        };

        self.flush_streaming_text_buffer();
        let fixture = agent_chat_kitchen_sink_fixture();
        self.stream_task = None;
        self.pending_permission = None;
        self.messages.clear();
        self.active_plan_entries.clear();
        self.active_tool_calls.clear();
        self.tool_call_lookup.clear();
        self.standing_approvals.clear();
        self.active_mode_id = None;
        self.available_commands.clear();
        self.usage_tokens = None;
        self.usage_cost_usd = None;
        self.input.clear();
        self.next_message_id = 1;
        self.clear_all_pending_context("load_kitchen_sink_fixture");
        self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
        self.context_bootstrap_note = None;
        self.queued_submit_while_bootstrapping = false;

        for message in fixture.messages {
            let role = match message.role {
                AgentChatKitchenSinkFixtureRole::User => AgentChatThreadMessageRole::User,
                AgentChatKitchenSinkFixtureRole::Assistant => AgentChatThreadMessageRole::Assistant,
                AgentChatKitchenSinkFixtureRole::Thought => AgentChatThreadMessageRole::Thought,
                AgentChatKitchenSinkFixtureRole::Tool => AgentChatThreadMessageRole::Tool,
                AgentChatKitchenSinkFixtureRole::System => AgentChatThreadMessageRole::System,
                AgentChatKitchenSinkFixtureRole::Error => AgentChatThreadMessageRole::Error,
            };
            let body = message.body.to_string();
            if let Some(tool_call_id) = message.tool_call_id {
                // Tool rows with structured fixture meta go through the real
                // tool-call event path so the kitchen sink exercises the
                // production card pipeline (kind, status, subject, diff).
                if let Some(meta) =
                    crate::ai::agent_chat::ui::kitchen_sink_fixture::kitchen_sink_tool_meta(
                        tool_call_id,
                    )
                {
                    let mut lines = body.lines();
                    let title = lines.next().unwrap_or("Tool").to_string();
                    let status = lines.next().unwrap_or("running").to_string();
                    let output = lines.collect::<Vec<_>>().join("\n").trim().to_string();
                    self.upsert_tool_call_start(
                        tool_call_id.to_string(),
                        title,
                        status.clone(),
                        Some(meta.tool_name.to_string()),
                        serde_json::from_str(meta.args_json).ok(),
                    );
                    self.apply_tool_call_update(
                        tool_call_id.to_string(),
                        None,
                        Some(status),
                        (!output.is_empty()).then_some(output),
                        None,
                        meta.diff.map(str::to_string),
                        meta.is_error,
                    );
                    continue;
                }
                let id = self.alloc_id();
                self.messages
                    .push(AgentChatThreadMessage::with_tool_call_id(
                        id,
                        role,
                        body,
                        tool_call_id.to_string(),
                    ));
            } else {
                let id = self.alloc_id();
                self.messages
                    .push(AgentChatThreadMessage::new(id, role, body));
            }
        }
        self.set_status(AgentChatThreadStatus::Idle);
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_kitchen_sink_fixture_loaded",
            fixture_id = fixture.id,
            message_count = self.messages.len(),
        );
        cx.notify();
    }

    /// Clear composer-attached context state before a fresh external entry
    /// intent reuses this thread.
    ///
    /// Preserves transcript history, but removes stale chips, hidden context
    /// blocks, and queued bootstrap state so a launcher-initiated submit
    /// starts from the new intent alone.
    pub(crate) fn clear_pending_context_for_new_entry_intent(&mut self, cx: &mut Context<Self>) {
        self.reset_pending_context_for_new_entry_intent();
        cx.notify();
    }

    pub(crate) fn cancel_streaming(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.status, AgentChatThreadStatus::Streaming) {
            return;
        }
        self.flush_streaming_text_buffer();
        self.queue_paused = true;
        if let Err(error) = self.connection.cancel_turn(self.ui_thread_id.clone()) {
            tracing::warn!(
                target: "script_kit::tab_ai",
                event = "agent_chat_cancel_turn_enqueue_failed",
                error = %error,
            );
        }
        self.stream_task = None;
        self.stream_started_at = None;
        self.status = AgentChatThreadStatus::Idle;
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
        self.bump_transcript_generation("load_saved_messages");
        self.flush_streaming_text_buffer();
        self.stream_task = None;
        self.stream_started_at = None;
        self.pending_permission = None;
        self.status = AgentChatThreadStatus::Idle;
        self.active_plan_entries.clear();
        self.active_tool_calls.clear();
        self.tool_call_lookup.clear();
        self.standing_approvals.clear();
        self.active_mode_id = None;
        self.available_commands.clear();
        self.usage_tokens = None;
        self.usage_cost_usd = None;
        self.next_message_id = 1;
        self.clear_all_pending_context("load_saved_messages");
        self.messages.clear();
        for msg in saved {
            let role = match msg.role.as_str() {
                "User" => AgentChatThreadMessageRole::User,
                "Assistant" => AgentChatThreadMessageRole::Assistant,
                "Thought" => AgentChatThreadMessageRole::Thought,
                "Tool" => AgentChatThreadMessageRole::Tool,
                "System" => AgentChatThreadMessageRole::System,
                "Error" => AgentChatThreadMessageRole::Error,
                _ => AgentChatThreadMessageRole::System,
            };
            let id = self.alloc_id();
            self.messages
                .push(AgentChatThreadMessage::new(id, role, msg.body.clone()));
        }
        self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
        self.context_bootstrap_note = None;
        cx.notify();
    }

    pub(crate) fn draft_snapshot(&self) -> AgentChatThreadDraftSnapshot {
        AgentChatThreadDraftSnapshot {
            input: self.input.text().to_string(),
            input_cursor: self.input.cursor(),
            pending_context_parts: self.pending_context_parts.clone(),
            pending_context_consumed: self.pending_context_consumed,
        }
    }

    pub(crate) fn restore_draft_snapshot(
        &mut self,
        snapshot: AgentChatThreadDraftSnapshot,
        cx: &mut Context<Self>,
    ) {
        let input_len = snapshot.input.chars().count();
        let cursor = snapshot.input_cursor.min(input_len);
        self.input.set_text(snapshot.input);
        self.input.set_cursor(cursor);
        self.pending_context_parts = snapshot.pending_context_parts;
        self.pending_context_consumed = snapshot.pending_context_consumed;
        self.pending_context_blocks.clear();
        self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
        self.context_bootstrap_note = None;
        self.queued_submit_while_bootstrapping = false;
        cx.notify();
    }

    fn reset_pending_context_for_new_entry_intent(&mut self) {
        self.clear_all_pending_context("new_entry_intent");
        self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
        self.context_bootstrap_note = None;
        self.queued_submit_while_bootstrapping = false;
    }

    /// Tracked tool calls, ordered by creation.
    pub(crate) fn active_tool_calls(&self) -> &[AgentChatToolCallState] {
        &self.active_tool_calls
    }

    /// Current bootstrap state for deferred context capture.
    pub(crate) fn context_bootstrap_state(&self) -> AgentChatContextBootstrapState {
        self.context_bootstrap_state
    }

    /// Whether deferred ambient context capture is still expected.
    pub(crate) fn pending_ambient_context_enabled(&self) -> bool {
        self.pending_ambient_context_enabled
    }

    /// Whether a submit is queued waiting for context bootstrap.
    pub(crate) fn queued_submit_while_bootstrapping(&self) -> bool {
        self.queued_submit_while_bootstrapping
    }

    pub(crate) fn queued_messages(&self) -> &VecDeque<AgentChatQueuedMessage> {
        &self.queued_messages
    }

    pub(crate) fn queue_paused(&self) -> bool {
        self.queue_paused
    }

    pub(crate) fn remove_queued_message(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.queued_messages.len() {
            self.queued_messages.remove(index);
            cx.notify();
        }
    }

    pub(crate) fn clear_queued_messages(&mut self, cx: &mut Context<Self>) {
        if !self.queued_messages.is_empty() {
            self.queued_messages.clear();
            cx.notify();
        }
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
                event = "agent_chat_context_part_add_skipped_duplicate",
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
            self.context_bootstrap_state = AgentChatContextBootstrapState::Preparing;
            self.context_bootstrap_note = ambient_label
                .as_deref()
                .map(Self::ambient_capture_preparing_note);
        } else if !self.pending_ambient_context_enabled
            && matches!(
                self.context_bootstrap_state,
                AgentChatContextBootstrapState::Preparing
            )
        {
            self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
            self.context_bootstrap_note = None;
        }

        self.pending_context_parts.push(part);
        self.arm_pending_context(if is_ambient_bootstrap {
            "add_ambient_bootstrap_part"
        } else {
            "add_context_part"
        });

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_context_part_added",
            source = %source,
            label = %label,
            is_ambient_bootstrap,
            ambient_label = ?ambient_label,
            pending_part_count = self.pending_context_parts.len(),
            pending_block_count = self.pending_context_blocks.len(),
        );
        cx.notify();
    }

    pub(crate) fn add_or_replace_skill_context(
        &mut self,
        identity: SkillContextIdentity,
        part: crate::ai::message_parts::AiContextPart,
        cx: &mut Context<Self>,
    ) {
        let crate::ai::message_parts::AiContextPart::SkillFile {
            path, slash_name, ..
        } = &part
        else {
            self.add_context_part(part, cx);
            return;
        };

        self.pending_context_parts.retain(|existing| {
            !matches!(
                existing,
                crate::ai::message_parts::AiContextPart::SkillFile {
                    path: existing_path,
                    slash_name: existing_slash,
                    ..
                } if existing_path == path || existing_slash == slash_name
            )
        });

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_skill_context_bound_to_thread",
            thread_id = %identity.thread_id,
            skill_id = %identity.skill_id,
            skill_file_hash = %identity.skill_file_hash,
            staged_by = ?identity.staged_by,
        );
        self.add_context_part(part, cx);
    }

    /// Replace all pending typed context parts in one host-owned handoff.
    ///
    /// Used by host surfaces that stage a fresh context payload on an
    /// existing Agent Chat view and must not append onto stale chips from a prior
    /// entry path.
    pub(crate) fn replace_pending_context_parts(
        &mut self,
        parts: Vec<crate::ai::message_parts::AiContextPart>,
        reason: &'static str,
        cx: &mut Context<Self>,
    ) {
        self.replace_pending_context_parts_inner(parts, reason);

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_pending_context_parts_replaced",
            reason,
            part_count = self.pending_context_parts.len(),
            ambient_enabled = self.pending_ambient_context_enabled,
        );
        cx.notify();
    }

    fn replace_pending_context_parts_inner(
        &mut self,
        parts: Vec<crate::ai::message_parts::AiContextPart>,
        reason: &'static str,
    ) {
        self.clear_all_pending_context(reason);
        self.pending_context_parts = parts;
        self.pending_context_consumed = false;
        self.queued_submit_while_bootstrapping = false;

        let ambient_label = self
            .pending_context_parts
            .iter()
            .find_map(|part| part.ambient_chip_label().map(|value| value.to_string()));
        let has_ambient_bootstrap = self
            .pending_context_parts
            .iter()
            .any(|part| part.is_ambient_bootstrap_resource());
        let has_promoted_ambient_chip = self
            .pending_context_parts
            .iter()
            .any(|part| part.is_ambient_context_chip());

        self.pending_ambient_context_enabled = has_ambient_bootstrap || has_promoted_ambient_chip;

        if has_ambient_bootstrap {
            self.context_bootstrap_state = AgentChatContextBootstrapState::Preparing;
            self.context_bootstrap_note = ambient_label
                .as_deref()
                .map(Self::ambient_capture_preparing_note);
        } else if has_promoted_ambient_chip {
            self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
            self.context_bootstrap_note = ambient_label
                .as_deref()
                .map(Self::ambient_capture_ready_note);
        } else {
            self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
            self.context_bootstrap_note = None;
        }
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
                AgentChatContextBootstrapState::Ready,
                Self::ambient_capture_removed_note(ambient_label),
                cx,
            );
        } else {
            self.arm_pending_context("remove_context_part");
            cx.notify();
        }

        tracing::info!(
            target: "script_kit::tab_ai",
            event = "agent_chat_context_part_removed",
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

/// Test-only helpers exposed to sibling modules in `src/ai/agent_chat/ui/`.
#[cfg(test)]
impl AgentChatThread {
    /// Build a test thread without a real connection or GPUI context.
    pub(super) fn test_new(
        context_blocks: Vec<ContentBlock>,
        initial_input: Option<String>,
    ) -> Self {
        let (_perm_tx, perm_rx) = async_channel::bounded(1);
        let dummy_connection: Arc<dyn AgentChatConnection> = Arc::new(TestAgentChatConnection);

        Self {
            connection: dummy_connection,
            permission_rx: perm_rx,
            ui_thread_id: "test-thread".to_string(),
            cwd: PathBuf::from("/tmp/test"),
            display_name: "Test Agent".into(),
            profile_id: crate::ai::agent_chat::profiles::BUILTIN_GENERAL_PROFILE_ID.to_string(),
            messages: Vec::new(),
            input: match initial_input {
                Some(text) if !text.is_empty() => TextInputState::with_text(text),
                _ => TextInputState::new(),
            },
            status: AgentChatThreadStatus::Idle,
            active_callout: None,
            pending_permission: None,
            pending_context_blocks: context_blocks,
            pending_context_consumed: false,
            pending_context_parts: Vec::new(),
            pending_ambient_context_enabled: false,
            context_bootstrap_state: AgentChatContextBootstrapState::Ready,
            queued_submit_while_bootstrapping: false,
            context_bootstrap_note: None,
            queued_messages: VecDeque::new(),
            queue_paused: false,
            active_plan_entries: Vec::new(),
            active_mode_id: None,
            available_commands: Vec::new(),
            active_tool_calls: Vec::new(),
            tool_call_lookup: HashMap::new(),
            standing_approvals: Vec::new(),
            fork_points: Vec::new(),
            pending_fork_ordinal: None,
            selected_agent: None,
            available_agents: Vec::new(),
            launch_requirements: crate::ai::agent_chat::ui::AgentChatLaunchRequirements::default(),
            setup_state: None,
            usage_tokens: None,
            usage_cost_usd: None,
            stream_started_at: None,
            ttft_pending: false,
            stream_task: None,
            permission_task: None,
            streaming_text_buffer: StreamingTextBuffer::default(),
            streaming_text_drain_task: None,
            transcript_generation: 0,
            next_message_id: 1,
            host_window_state: None,
            notification_debounce: AgentChatNotificationDebounce::default(),
            current_turn_id: 0,
            llm_title_attempted: false,
            available_models: Vec::new(),
            selected_model_id: None,
            selected_model_display_name: None,
            profile_display_name: None,
            profile_icon_name: None,
        }
    }

    pub(super) fn dismiss_active_callout_test(&mut self) {
        self.active_callout = None;
    }

    pub(super) fn retry_last_user_turn_test(&mut self) -> Result<(), String> {
        if !matches!(
            self.status,
            AgentChatThreadStatus::Error | AgentChatThreadStatus::Idle
        ) {
            return Ok(());
        }

        let Some(display_text) = self.last_user_turn_text() else {
            return Err("no_user_turn_to_retry".to_string());
        };
        let prepared = self.prepare_turn_blocks_with_receipt(display_text.trim());
        self.set_context_resolution_note(prepared.receipt.as_ref());
        let _request = AgentChatTurnRequest {
            ui_thread_id: self.ui_thread_id.clone(),
            cwd: self.cwd.clone(),
            blocks: prepared.blocks,
            model_id: self.selected_model_id.clone(),
        };
        self.stream_started_at = Some(std::time::Instant::now());
        self.ttft_pending = true;
        self.status = AgentChatThreadStatus::Streaming;
        self.active_callout = None;
        self.setup_state = None;
        Ok(())
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
            self.context_bootstrap_state = AgentChatContextBootstrapState::Preparing;
            self.context_bootstrap_note = part
                .ambient_chip_label()
                .map(Self::ambient_capture_preparing_note);
        } else if !self.pending_ambient_context_enabled
            && matches!(
                self.context_bootstrap_state,
                AgentChatContextBootstrapState::Preparing
            )
        {
            self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
            self.context_bootstrap_note = None;
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
            self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
            self.context_bootstrap_note = Some(Self::ambient_capture_removed_note(ambient_label));
        }
    }

    pub(super) fn replace_pending_context_parts_test(
        &mut self,
        parts: Vec<crate::ai::message_parts::AiContextPart>,
        reason: &'static str,
    ) {
        self.replace_pending_context_parts_inner(parts, reason);
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
            self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
            self.context_bootstrap_note = Some(Self::ambient_capture_removed_note(&ambient_label));
            return Ok(());
        }

        self.pending_context_blocks = build_tab_ai_agent_chat_context_blocks(context)?;
        self.pending_context_consumed = false;
        self.promote_ask_anything_chip_to_ambient();
        self.context_bootstrap_state = AgentChatContextBootstrapState::Ready;
        self.context_bootstrap_note = Some(Self::ambient_capture_ready_note(&ambient_label));
        Ok(())
    }

    /// Apply an event without a GPUI context (for testing pure logic).
    /// Reuses the same helper methods as `apply_event` but skips `cx.notify()`.
    pub(super) fn apply_event_test(&mut self, event: super::AgentChatEvent) {
        match event {
            super::AgentChatEvent::UserMessageDelta(chunk) => {
                self.append_chunk(AgentChatThreadMessageRole::System, chunk);
                self.set_status(AgentChatThreadStatus::Streaming);
            }
            super::AgentChatEvent::AgentMessageDelta(chunk) => {
                self.append_chunk(AgentChatThreadMessageRole::Assistant, chunk);
                self.set_status(AgentChatThreadStatus::Streaming);
            }
            super::AgentChatEvent::AgentThoughtDelta(chunk) => {
                self.append_chunk(AgentChatThreadMessageRole::Thought, chunk);
                self.set_status(AgentChatThreadStatus::Streaming);
            }
            super::AgentChatEvent::ToolCallStarted {
                tool_call_id,
                title,
                status,
                tool_name,
                raw_input,
            } => {
                self.upsert_tool_call_start(tool_call_id, title, status, tool_name, raw_input);
                self.set_status(AgentChatThreadStatus::Streaming);
            }
            super::AgentChatEvent::ToolCallUpdated {
                tool_call_id,
                title,
                status,
                body,
                raw_input,
                diff,
                is_error,
            } => {
                self.apply_tool_call_update(
                    tool_call_id,
                    title,
                    status,
                    body,
                    raw_input,
                    diff,
                    is_error,
                );
                self.set_status(AgentChatThreadStatus::Streaming);
            }
            super::AgentChatEvent::PlanUpdated { entries } => {
                self.active_plan_entries = entries;
                self.set_status(AgentChatThreadStatus::Streaming);
            }
            super::AgentChatEvent::AvailableCommandsUpdated { command_names } => {
                self.available_commands = command_names;
            }
            super::AgentChatEvent::ModeChanged { mode_id } => {
                self.active_mode_id = Some(mode_id);
            }
            super::AgentChatEvent::UsageUpdated {
                used_tokens,
                context_size,
                cost_usd,
            } => {
                self.usage_tokens = Some((used_tokens, context_size));
                if let Some(cost) = cost_usd {
                    self.usage_cost_usd = Some(cost);
                }
            }
            super::AgentChatEvent::ModelsAvailable {
                current_model_id,
                models,
            } => {
                self.apply_agent_models(current_model_id, models);
            }
            super::AgentChatEvent::ForkPointsAvailable { entries } => {
                self.fork_points = entries;
            }
            super::AgentChatEvent::ForkCompleted { text } => {
                // Test path mirrors `apply_fork_completed` minus the GPUI
                // refresh: truncate locally and stage the text for editing.
                if let Some(ordinal) = self.pending_fork_ordinal.take() {
                    Self::truncate_messages_at_user_ordinal(&mut self.messages, ordinal);
                    self.active_tool_calls.clear();
                    self.tool_call_lookup.clear();
                    self.input.set_text(text.clone());
                    self.input.set_cursor(text.chars().count());
                    self.set_status(AgentChatThreadStatus::Idle);
                }
            }
            super::AgentChatEvent::TurnFinished { .. } => {
                self.set_status(AgentChatThreadStatus::Idle);
                if !self.queue_paused {
                    if let Some(message) = self.queued_messages.pop_front() {
                        self.push_message(AgentChatThreadMessageRole::User, message.text);
                        self.set_status(AgentChatThreadStatus::Streaming);
                    }
                }
            }
            super::AgentChatEvent::SetupRequired {
                reason,
                auth_methods,
            } => {
                let current_requirements = self.current_setup_requirements();
                self.setup_state = Some(
                    super::setup_state::AgentChatInlineSetupState::from_runtime_setup_required(
                        self.selected_agent.clone(),
                        self.available_agents.clone(),
                        current_requirements,
                        &reason,
                        &auth_methods,
                    ),
                );
                self.set_status(AgentChatThreadStatus::Error);
            }
            super::AgentChatEvent::Failed { error } => {
                let can_retry = self.last_user_turn_text().is_some();
                self.active_callout = Some(AgentChatCallout::failed(error.clone(), can_retry));
                self.push_message(AgentChatThreadMessageRole::Error, error);
                self.set_status(AgentChatThreadStatus::Error);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to build an `AgentChatThread` without a real connection or GPUI context.
    /// Only for testing pure logic methods that don't need cx or connection.
    fn fork_point(entry_id: &str, text: &str) -> super::super::events::AgentChatForkPoint {
        super::super::events::AgentChatForkPoint {
            entry_id: entry_id.to_string(),
            text: text.to_string(),
        }
    }

    #[test]
    fn fork_points_event_replaces_rewind_list() {
        let mut thread = test_thread(Vec::new(), false);
        thread.apply_event_test(AgentChatEvent::ForkPointsAvailable {
            entries: vec![
                fork_point("e0", "first ask"),
                fork_point("e1", "second ask"),
            ],
        });
        assert_eq!(thread.fork_points().len(), 2);
        assert_eq!(thread.fork_points()[0].entry_id, "e0");

        thread.apply_event_test(AgentChatEvent::ForkPointsAvailable {
            entries: vec![fork_point("e0", "first ask")],
        });
        assert_eq!(
            thread.fork_points().len(),
            1,
            "list is replaced, not appended"
        );
    }

    #[test]
    fn fork_completed_truncates_at_user_ordinal_and_prefills_composer() {
        let mut thread = test_thread(Vec::new(), false);
        thread.push_message(AgentChatThreadMessageRole::User, "first ask");
        thread.push_message(AgentChatThreadMessageRole::Assistant, "first answer");
        thread.push_message(AgentChatThreadMessageRole::User, "second ask");
        thread.push_message(AgentChatThreadMessageRole::Assistant, "second answer");
        thread.fork_points = vec![
            fork_point("e0", "first ask"),
            fork_point("e1", "second ask"),
        ];
        thread.pending_fork_ordinal = Some(1);

        thread.apply_event_test(AgentChatEvent::ForkCompleted {
            text: "second ask".to_string(),
        });

        assert_eq!(
            thread.messages.len(),
            2,
            "second user message and its answer are dropped"
        );
        assert_eq!(thread.messages[0].body.as_ref(), "first ask");
        assert_eq!(thread.messages[1].body.as_ref(), "first answer");
        assert_eq!(thread.input.text(), "second ask");
        assert_eq!(thread.status, AgentChatThreadStatus::Idle);
        assert!(thread.pending_fork_ordinal.is_none());
    }

    #[test]
    fn fork_completed_without_pending_request_is_ignored() {
        let mut thread = test_thread(Vec::new(), false);
        thread.push_message(AgentChatThreadMessageRole::User, "only ask");

        thread.apply_event_test(AgentChatEvent::ForkCompleted {
            text: "stray".to_string(),
        });

        assert_eq!(thread.messages.len(), 1, "transcript untouched");
        assert!(thread.input.text().is_empty(), "composer untouched");
    }

    #[test]
    fn fork_point_for_message_id_maps_by_user_ordinal() {
        let mut thread = test_thread(Vec::new(), false);
        thread.push_message(AgentChatThreadMessageRole::User, "first ask");
        thread.push_message(AgentChatThreadMessageRole::Assistant, "first answer");
        thread.push_message(AgentChatThreadMessageRole::User, "second ask");
        let second_user_id = thread.messages[2].id;
        let fork_points = vec![
            fork_point("entry-0", "stale first text from pi"),
            fork_point("entry-1", "stale second text from pi"),
        ];

        let point = AgentChatThread::fork_point_for_message_id(
            &thread.messages,
            &fork_points,
            second_user_id,
        )
        .expect("second user message should resolve by ordinal");

        assert_eq!(point.entry_id, "entry-1");
    }

    #[test]
    fn fork_point_for_message_id_falls_back_to_text_when_lengths_mismatch() {
        let mut thread = test_thread(Vec::new(), false);
        thread.push_message(AgentChatThreadMessageRole::User, "first ask");
        thread.push_message(AgentChatThreadMessageRole::Assistant, "first answer");
        thread.push_message(AgentChatThreadMessageRole::User, "second ask");
        let second_user_id = thread.messages[2].id;
        let fork_points = vec![fork_point("entry-second", "second ask")];

        let point = AgentChatThread::fork_point_for_message_id(
            &thread.messages,
            &fork_points,
            second_user_id,
        )
        .expect("mismatched fork list should resolve by exact text");

        assert_eq!(point.entry_id, "entry-second");
    }

    #[test]
    fn fork_point_for_message_id_returns_none_when_unresolvable() {
        let mut thread = test_thread(Vec::new(), false);
        thread.push_message(AgentChatThreadMessageRole::User, "first ask");
        let first_user_id = thread.messages[0].id;
        let fork_points = Vec::new();

        assert!(AgentChatThread::fork_point_for_message_id(
            &thread.messages,
            &fork_points,
            first_user_id,
        )
        .is_none());
        assert!(AgentChatThread::fork_point_for_message_id(
            &thread.messages,
            &fork_points,
            first_user_id + 999,
        )
        .is_none());
    }

    #[test]
    fn truncate_at_user_ordinal_zero_clears_from_first_user_message() {
        let mut thread = test_thread(Vec::new(), false);
        thread.push_message(AgentChatThreadMessageRole::System, "context note");
        thread.push_message(AgentChatThreadMessageRole::User, "first ask");
        thread.push_message(AgentChatThreadMessageRole::Assistant, "answer");

        AgentChatThread::truncate_messages_at_user_ordinal(&mut thread.messages, 0);

        assert_eq!(thread.messages.len(), 1);
        assert_eq!(thread.messages[0].body.as_ref(), "context note");
    }

    fn test_thread(
        pending_context_blocks: Vec<ContentBlock>,
        pending_context_consumed: bool,
    ) -> AgentChatThread {
        test_thread_with_profile(
            crate::ai::agent_chat::profiles::BUILTIN_GENERAL_PROFILE_ID,
            pending_context_blocks,
            pending_context_consumed,
        )
    }

    fn test_thread_with_profile(
        profile_id: &str,
        pending_context_blocks: Vec<ContentBlock>,
        pending_context_consumed: bool,
    ) -> AgentChatThread {
        let (_perm_tx, perm_rx) = async_channel::bounded(1);
        // We create a dummy connection channel — tests that call prepare_turn_blocks
        // and append_chunk don't need a live connection.
        let dummy_connection: Arc<dyn AgentChatConnection> =
            Arc::new(super::TestAgentChatConnection);

        AgentChatThread {
            connection: dummy_connection,
            permission_rx: perm_rx,
            ui_thread_id: "test-thread".to_string(),
            cwd: PathBuf::from("."),
            display_name: "Test Agent".into(),
            profile_id: profile_id.to_string(),
            messages: Vec::new(),
            input: TextInputState::new(),
            status: AgentChatThreadStatus::Idle,
            active_callout: None,
            pending_permission: None,
            pending_context_blocks,
            pending_context_consumed,
            pending_context_parts: Vec::new(),
            pending_ambient_context_enabled: false,
            context_bootstrap_state: AgentChatContextBootstrapState::Ready,
            queued_submit_while_bootstrapping: false,
            context_bootstrap_note: None,
            queued_messages: VecDeque::new(),
            queue_paused: false,
            active_plan_entries: Vec::new(),
            active_mode_id: None,
            available_commands: Vec::new(),
            active_tool_calls: Vec::new(),
            tool_call_lookup: HashMap::new(),
            standing_approvals: Vec::new(),
            fork_points: Vec::new(),
            pending_fork_ordinal: None,
            selected_agent: None,
            available_agents: Vec::new(),
            launch_requirements: crate::ai::agent_chat::ui::AgentChatLaunchRequirements::default(),
            setup_state: None,
            usage_tokens: None,
            usage_cost_usd: None,
            stream_started_at: None,
            ttft_pending: false,
            stream_task: None,
            permission_task: None,
            streaming_text_buffer: StreamingTextBuffer::default(),
            streaming_text_drain_task: None,
            transcript_generation: 0,
            next_message_id: 1,
            host_window_state: None,
            notification_debounce: AgentChatNotificationDebounce::default(),
            current_turn_id: 0,
            llm_title_attempted: false,
            available_models: Vec::new(),
            selected_model_id: None,
            selected_model_display_name: None,
            profile_display_name: None,
            profile_icon_name: None,
        }
    }

    fn block_text(block: &ContentBlock) -> &str {
        match block {
            ContentBlock::Text(text) => text.text.as_str(),
            other => panic!("expected text block, got {other:?}"),
        }
    }

    #[test]
    fn brain_profile_prepends_recall_and_records_ask_signal() {
        let mut thread = test_thread_with_profile(
            crate::ai::agent_chat::profiles::BUILTIN_BRAIN_PROFILE_ID,
            Vec::new(),
            false,
        );
        let signal_calls = std::cell::Cell::new(0);

        let prepared = thread.prepare_turn_blocks_with_receipt_using(
            "What is the handoff port?",
            |_| Some("Brain recall\n- [Note] The handoff port is 49217.".to_string()),
            |_| signal_calls.set(signal_calls.get() + 1),
        );

        assert_eq!(signal_calls.get(), 1);
        assert_eq!(prepared.blocks.len(), 2);
        assert!(block_text(&prepared.blocks[0]).contains("Brain recall"));
        assert_eq!(
            block_text(&prepared.blocks[1]),
            "--- USER REQUEST ---\nWhat is the handoff port?"
        );
    }

    #[test]
    fn non_brain_profile_does_not_call_recall_or_record_ask_signal() {
        let mut thread = test_thread_with_profile(
            crate::ai::agent_chat::profiles::BUILTIN_GENERAL_PROFILE_ID,
            Vec::new(),
            false,
        );

        let prepared = thread.prepare_turn_blocks_with_receipt_using(
            "What is the handoff port?",
            |_| panic!("non-Brain profile must not read brain recall"),
            |_| panic!("non-Brain profile must not record brain ask signals"),
        );

        assert_eq!(prepared.blocks.len(), 1);
        assert_eq!(block_text(&prepared.blocks[0]), "What is the handoff port?");
    }

    #[test]
    fn brain_recall_sits_before_pending_context_and_user_request() {
        let mut thread = test_thread_with_profile(
            crate::ai::agent_chat::profiles::BUILTIN_BRAIN_PROFILE_ID,
            vec![ContentBlock::Text(TextContent::new("staged context"))],
            false,
        );

        let prepared = thread.prepare_turn_blocks_with_receipt_using(
            "Summarize this",
            |_| Some("Brain recall\n- [Day page] remembered context".to_string()),
            |_| {},
        );

        assert_eq!(prepared.blocks.len(), 3);
        assert!(block_text(&prepared.blocks[0]).starts_with("Brain recall"));
        assert_eq!(block_text(&prepared.blocks[1]), "staged context");
        assert_eq!(
            block_text(&prepared.blocks[2]),
            "--- USER REQUEST ---\nSummarize this"
        );
    }

    #[test]
    fn completed_turn_ingest_payload_uses_latest_turn_and_stable_index() {
        let mut thread = test_thread(Vec::new(), false);
        thread.push_message(AgentChatThreadMessageRole::User, "first ask");
        thread.push_message(AgentChatThreadMessageRole::Assistant, "first answer");
        thread.push_message(AgentChatThreadMessageRole::User, "second ask");
        thread.push_message(AgentChatThreadMessageRole::Assistant, "second answer");

        let payload = thread
            .completed_chat_turn_ingest(Some("History Title".to_string()))
            .expect("completed turn should produce ingest payload");

        assert_eq!(payload.thread_id, "test-thread");
        assert_eq!(payload.turn_index, 1);
        assert_eq!(payload.user_text, "second ask");
        assert_eq!(payload.assistant_text, "second answer");
        assert_eq!(payload.trace_label, "History Title");

        let fallback = thread
            .completed_chat_turn_ingest(None)
            .expect("completed turn should produce fallback ingest payload");
        assert_eq!(fallback.trace_label, "first ask");
    }

    #[test]
    fn completed_turn_ingest_payload_is_not_brain_profile_gated() {
        let mut thread = test_thread_with_profile(
            crate::ai::agent_chat::profiles::BUILTIN_GENERAL_PROFILE_ID,
            Vec::new(),
            false,
        );
        thread.push_message(AgentChatThreadMessageRole::User, "general profile ask");
        thread.push_message(
            AgentChatThreadMessageRole::Assistant,
            "general profile answer",
        );

        let payload = thread
            .completed_chat_turn_ingest(None)
            .expect("all completed Agent Chat turns should become memory");

        assert_eq!(payload.turn_index, 0);
        assert_eq!(payload.user_text, "general profile ask");
        assert_eq!(payload.assistant_text, "general profile answer");
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
    fn awaiting_first_assistant_text_tracks_pre_text_streaming_gap() {
        let mut thread = test_thread(Vec::new(), true);

        thread.push_message(AgentChatThreadMessageRole::User, "Follow up");
        thread.set_status(AgentChatThreadStatus::Streaming);

        assert!(thread.awaiting_first_assistant_text());

        thread.push_message(AgentChatThreadMessageRole::Thought, "Inspecting files");
        thread.push_message(AgentChatThreadMessageRole::Tool, "Read file completed");

        assert!(
            thread.awaiting_first_assistant_text(),
            "thought/tool events before text should keep the activity row visible"
        );

        thread.push_message(AgentChatThreadMessageRole::Assistant, "I found the issue.");

        assert!(!thread.awaiting_first_assistant_text());
    }

    #[test]
    fn awaiting_first_assistant_text_is_false_without_streaming_user_turn() {
        let mut thread = test_thread(Vec::new(), true);

        assert!(!thread.awaiting_first_assistant_text());

        thread.push_message(AgentChatThreadMessageRole::User, "Follow up");
        assert!(!thread.awaiting_first_assistant_text());

        thread.set_status(AgentChatThreadStatus::Streaming);
        assert!(thread.awaiting_first_assistant_text());

        thread.set_status(AgentChatThreadStatus::Idle);
        assert!(!thread.awaiting_first_assistant_text());
    }

    #[test]
    fn assistant_chunks_append_to_last_assistant_message() {
        let mut thread = test_thread(Vec::new(), true);

        thread.append_chunk(AgentChatThreadMessageRole::Assistant, "Hello".to_string());
        thread.append_chunk(AgentChatThreadMessageRole::Assistant, " world".to_string());

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

        thread.append_chunk(AgentChatThreadMessageRole::Assistant, "Hello".to_string());
        thread.append_chunk(
            AgentChatThreadMessageRole::Thought,
            "thinking...".to_string(),
        );
        thread.append_chunk(AgentChatThreadMessageRole::Assistant, "world".to_string());

        assert_eq!(
            thread.messages.len(),
            3,
            "different roles should create separate messages"
        );
    }

    #[test]
    fn prepare_turn_blocks_no_guidance_in_exploration_mode() {
        let mut thread = test_thread(vec![ContentBlock::Text(TextContent::new("context"))], false);

        // Even authoring-like intents get no guidance — users invoke /new-script explicitly
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
    /// Delegates to the instance method `apply_event_test` on `AgentChatThread`.
    fn apply_event_test(thread: &mut AgentChatThread, event: AgentChatEvent) {
        thread.apply_event_test(event);
    }

    #[test]
    fn plan_updated_stores_in_dedicated_field() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AgentChatEvent::PlanUpdated {
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
            AgentChatEvent::ModeChanged {
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
    fn models_available_replaces_list_and_surfaces_new_models() {
        use super::super::config::AgentChatModelEntry;

        let mut thread = test_thread(Vec::new(), true);
        // Seed the thread with the old hardcoded fallback list so we can
        // prove that ModelsAvailable actually replaces it.
        thread.available_models = vec![
            AgentChatModelEntry {
                id: "claude-sonnet-4-6".into(),
                display_name: Some("Sonnet 4.6".into()),
                context_window: Some(200_000),
            },
            AgentChatModelEntry {
                id: "claude-opus-4-6".into(),
                display_name: Some("Opus 4.6".into()),
                context_window: Some(200_000),
            },
        ];

        // Simulate what the Agent Chat client produces when claude-code-agent_chat advertises
        // Opus 4.7 in its session/new response.
        let agent_list = vec![
            AgentChatModelEntry {
                id: "claude-opus-4-7".into(),
                display_name: Some("Opus 4.7".into()),
                context_window: None,
            },
            AgentChatModelEntry {
                id: "claude-sonnet-4-6".into(),
                display_name: Some("Sonnet 4.6".into()),
                context_window: None,
            },
            AgentChatModelEntry {
                id: "claude-haiku-4-5".into(),
                display_name: Some("Haiku 4.5".into()),
                context_window: None,
            },
        ];

        apply_event_test(
            &mut thread,
            AgentChatEvent::ModelsAvailable {
                current_model_id: Some("claude-opus-4-7".into()),
                models: agent_list.clone(),
            },
        );

        let ids: Vec<&str> = thread
            .available_models()
            .iter()
            .map(|m| m.id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec!["claude-opus-4-7", "claude-sonnet-4-6", "claude-haiku-4-5"],
            "agent-advertised list should replace the hardcoded fallback"
        );
        assert!(
            ids.contains(&"claude-opus-4-7"),
            "Opus 4.7 must surface when the agent advertises it"
        );
        // The stale fallback-only entry must be gone.
        assert!(
            !ids.contains(&"claude-opus-4-6"),
            "old fallback entries should not leak through"
        );
    }

    #[test]
    fn models_available_preserves_user_selection_when_still_valid() {
        use super::super::config::AgentChatModelEntry;

        let mut thread = test_thread(Vec::new(), true);
        thread.selected_model_id = Some("claude-sonnet-4-6".into());
        thread.selected_model_display_name = Some(SharedString::from("Sonnet 4.6"));

        apply_event_test(
            &mut thread,
            AgentChatEvent::ModelsAvailable {
                current_model_id: Some("claude-opus-4-7".into()),
                models: vec![
                    AgentChatModelEntry {
                        id: "claude-opus-4-7".into(),
                        display_name: Some("Opus 4.7".into()),
                        context_window: None,
                    },
                    AgentChatModelEntry {
                        id: "claude-sonnet-4-6".into(),
                        display_name: Some("Sonnet 4.6".into()),
                        context_window: None,
                    },
                ],
            },
        );

        assert_eq!(
            thread.selected_model_id(),
            Some("claude-sonnet-4-6"),
            "user's persisted selection must be preserved when still in the new list"
        );
    }

    #[test]
    fn models_available_falls_back_to_current_when_selection_dropped() {
        use super::super::config::AgentChatModelEntry;

        let mut thread = test_thread(Vec::new(), true);
        // User had a selection that the agent no longer lists.
        thread.selected_model_id = Some("claude-retired-model".into());

        apply_event_test(
            &mut thread,
            AgentChatEvent::ModelsAvailable {
                current_model_id: Some("claude-opus-4-7".into()),
                models: vec![
                    AgentChatModelEntry {
                        id: "claude-opus-4-7".into(),
                        display_name: Some("Opus 4.7".into()),
                        context_window: None,
                    },
                    AgentChatModelEntry {
                        id: "claude-sonnet-4-6".into(),
                        display_name: Some("Sonnet 4.6".into()),
                        context_window: None,
                    },
                ],
            },
        );

        assert_eq!(
            thread.selected_model_id(),
            Some("claude-opus-4-7"),
            "selection should fall back to the agent's declared current model"
        );
    }

    #[test]
    fn available_commands_stores_in_dedicated_field() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AgentChatEvent::AvailableCommandsUpdated {
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
            AgentChatEvent::ToolCallStarted {
                tool_call_id: "tc-1".into(),
                title: "Read file".into(),
                status: "running".into(),
                tool_name: None,
                raw_input: None,
            },
        );

        assert_eq!(thread.active_tool_calls().len(), 1);
        assert_eq!(thread.active_tool_calls()[0].tool_call_id, "tc-1");
        assert_eq!(thread.active_tool_calls()[0].title, "Read file");
        assert_eq!(thread.active_tool_calls()[0].status, "running");

        assert_eq!(thread.messages.len(), 1);
        assert_eq!(thread.messages[0].role, AgentChatThreadMessageRole::Tool);
        assert_eq!(thread.messages[0].tool_call_id.as_deref(), Some("tc-1"));
    }

    #[test]
    fn tool_call_updated_modifies_existing_message_in_place() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AgentChatEvent::ToolCallStarted {
                tool_call_id: "tc-1".into(),
                title: "Read file".into(),
                status: "running".into(),
                tool_name: None,
                raw_input: None,
            },
        );

        apply_event_test(
            &mut thread,
            AgentChatEvent::ToolCallUpdated {
                tool_call_id: "tc-1".into(),
                title: None,
                status: Some("completed".into()),
                body: Some("file contents here".into()),
                raw_input: None,
                diff: None,
                is_error: false,
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
            AgentChatEvent::ToolCallUpdated {
                tool_call_id: "unknown".into(),
                title: None,
                status: Some("done".into()),
                body: None,
                raw_input: None,
                diff: None,
                is_error: false,
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
            AgentChatEvent::TurnFinished {
                stop_reason: "end_turn".into(),
            },
        );

        assert!(
            thread.messages.is_empty(),
            "turn finished should not produce a message"
        );
        assert_eq!(thread.status, AgentChatThreadStatus::Idle);
    }

    #[test]
    fn submit_while_streaming_queues_and_clears_composer() {
        let mut thread = test_thread(Vec::new(), true);
        thread.status = AgentChatThreadStatus::Streaming;
        thread.input.set_text("follow up".to_string());
        thread
            .pending_context_parts
            .push(crate::ai::message_parts::AiContextPart::TextBlock {
                label: "ctx".to_string(),
                source: "test".to_string(),
                text: "ctx".to_string(),
                mime_type: None,
            });

        let text = thread.input.text().trim().to_string();
        thread.resume_queue_for_manual_submit();
        thread.queue_current_composer(text);

        assert_eq!(thread.queued_messages().len(), 1);
        assert_eq!(thread.queued_messages()[0].text, "follow up");
        assert_eq!(thread.queued_messages()[0].context_parts.len(), 1);
        assert!(thread.input.text().is_empty());
        assert!(thread.pending_context_parts().is_empty());
        assert_eq!(thread.status, AgentChatThreadStatus::Streaming);
    }

    #[test]
    fn turn_finished_auto_sends_front_of_queue() {
        let mut thread = test_thread(Vec::new(), true);
        thread.status = AgentChatThreadStatus::Streaming;
        thread
            .queued_messages
            .push_back(AgentChatQueuedMessage::new(
                "first queued".to_string(),
                Vec::new(),
            ));
        thread
            .queued_messages
            .push_back(AgentChatQueuedMessage::new(
                "second queued".to_string(),
                Vec::new(),
            ));

        thread.apply_event_test(AgentChatEvent::TurnFinished {
            stop_reason: "end_turn".into(),
        });

        assert_eq!(thread.status, AgentChatThreadStatus::Streaming);
        assert_eq!(
            thread.messages.last().unwrap().body.as_ref(),
            "first queued"
        );
        assert_eq!(thread.queued_messages().len(), 1);
        assert_eq!(thread.queued_messages()[0].text, "second queued");
    }

    #[test]
    fn paused_queue_does_not_auto_send_on_turn_finished() {
        let mut thread = test_thread(Vec::new(), true);
        thread.status = AgentChatThreadStatus::Streaming;
        thread.queue_paused = true;
        thread
            .queued_messages
            .push_back(AgentChatQueuedMessage::new(
                "held queued".to_string(),
                Vec::new(),
            ));

        thread.apply_event_test(AgentChatEvent::TurnFinished {
            stop_reason: "cancelled".into(),
        });

        assert_eq!(thread.status, AgentChatThreadStatus::Idle);
        assert!(thread.messages.is_empty());
        assert_eq!(thread.queued_messages().len(), 1);
    }

    #[test]
    fn manual_submit_clears_queue_pause() {
        let mut thread = test_thread(Vec::new(), true);
        thread.queue_paused = true;

        thread.resume_queue_for_manual_submit();

        assert!(!thread.queue_paused());
    }

    #[test]
    fn closed_stream_without_terminal_unlocks_after_assistant_text() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AgentChatEvent::AgentMessageDelta("done".into()),
        );
        assert_eq!(thread.status, AgentChatThreadStatus::Streaming);

        assert!(thread.finish_stream_closed_without_terminal());

        assert_eq!(
            thread.status,
            AgentChatThreadStatus::Idle,
            "missing terminal event must not leave composer blocked"
        );
        assert_eq!(thread.messages.len(), 1);
        assert_eq!(
            thread.messages[0].role,
            AgentChatThreadMessageRole::Assistant
        );
    }

    #[test]
    fn closed_stream_without_terminal_errors_without_assistant_text() {
        let mut thread = test_thread(Vec::new(), true);
        thread.status = AgentChatThreadStatus::Streaming;

        assert!(thread.finish_stream_closed_without_terminal());

        assert_eq!(
            thread.status,
            AgentChatThreadStatus::Error,
            "missing terminal event without content should still unlock follow-up"
        );
        assert_eq!(thread.messages.len(), 1);
        assert_eq!(thread.messages[0].role, AgentChatThreadMessageRole::Error);
    }

    #[test]
    fn failed_event_creates_error_message_and_retryable_callout() {
        let mut thread = test_thread(Vec::new(), true);
        thread.push_message(AgentChatThreadMessageRole::User, "please try");

        apply_event_test(
            &mut thread,
            AgentChatEvent::Failed {
                error: "connection lost".into(),
            },
        );

        assert_eq!(thread.messages.len(), 2);
        assert_eq!(thread.messages[1].role, AgentChatThreadMessageRole::Error);
        assert_eq!(thread.messages[1].body.as_ref(), "connection lost");
        assert_eq!(thread.status, AgentChatThreadStatus::Error);
        let callout = thread.active_callout().expect("failed turn arms callout");
        assert_eq!(callout.severity, AgentChatCalloutSeverity::Error);
        assert_eq!(callout.title.as_ref(), "Turn failed");
        assert_eq!(callout.detail.as_ref().unwrap().as_ref(), "connection lost");
        assert!(callout.can_retry);
    }

    #[test]
    fn retry_from_error_reenters_streaming_without_duplicate_user_message() {
        let mut thread = test_thread(Vec::new(), true);
        thread.push_message(AgentChatThreadMessageRole::User, "please try");
        thread.apply_event_test(AgentChatEvent::Failed {
            error: "connection lost".into(),
        });
        let before = thread.messages.len();

        thread.retry_last_user_turn_test().unwrap();

        assert_eq!(thread.status, AgentChatThreadStatus::Streaming);
        assert_eq!(thread.messages.len(), before);
        assert_eq!(
            thread
                .messages
                .iter()
                .filter(|message| matches!(message.role, AgentChatThreadMessageRole::User))
                .count(),
            1
        );
        assert!(thread.active_callout().is_none());
    }

    #[test]
    fn dismiss_clears_failed_turn_callout() {
        let mut thread = test_thread(Vec::new(), true);
        thread.push_message(AgentChatThreadMessageRole::User, "please try");
        thread.apply_event_test(AgentChatEvent::Failed {
            error: "connection lost".into(),
        });

        thread.dismiss_active_callout_test();

        assert!(thread.active_callout().is_none());
    }

    #[test]
    fn starting_new_turn_clears_failed_turn_callout() {
        let mut thread = test_thread(Vec::new(), true);
        thread.push_message(AgentChatThreadMessageRole::User, "please try");
        thread.apply_event_test(AgentChatEvent::Failed {
            error: "connection lost".into(),
        });

        thread.retry_last_user_turn_test().unwrap();

        assert!(thread.active_callout().is_none());
    }

    #[test]
    fn multiple_tool_calls_tracked_independently() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AgentChatEvent::ToolCallStarted {
                tool_call_id: "tc-1".into(),
                title: "Read file".into(),
                status: "running".into(),
                tool_name: None,
                raw_input: None,
            },
        );
        apply_event_test(
            &mut thread,
            AgentChatEvent::ToolCallStarted {
                tool_call_id: "tc-2".into(),
                title: "Write file".into(),
                status: "running".into(),
                tool_name: None,
                raw_input: None,
            },
        );

        // Update only tc-1.
        apply_event_test(
            &mut thread,
            AgentChatEvent::ToolCallUpdated {
                tool_call_id: "tc-1".into(),
                title: None,
                status: Some("completed".into()),
                body: None,
                raw_input: None,
                diff: None,
                is_error: false,
            },
        );

        assert_eq!(thread.active_tool_calls().len(), 2);
        assert_eq!(thread.active_tool_calls()[0].status, "completed");
        assert_eq!(thread.active_tool_calls()[1].status, "running");

        // Two messages, one per tool call.
        assert_eq!(thread.messages.len(), 2);
    }

    fn approval_request_with_options(
        reply_tx: async_channel::Sender<Option<String>>,
    ) -> AgentChatApprovalRequest {
        use super::super::permission_broker::AgentChatApprovalOption;
        AgentChatApprovalRequest {
            id: 1,
            title: "Run command".into(),
            body: "Agent wants to run a command".into(),
            preview: Some(
                super::super::permission_broker::AgentChatApprovalPreview::new("bash", "tc-1")
                    .with_subject(Some("cargo test".to_string())),
            ),
            options: vec![
                AgentChatApprovalOption {
                    option_id: "allow-once".into(),
                    name: "Allow".into(),
                    kind: "AllowOnce".into(),
                },
                AgentChatApprovalOption {
                    option_id: "allow-always".into(),
                    name: "Allow always".into(),
                    kind: "AllowAlways".into(),
                },
                AgentChatApprovalOption {
                    option_id: "deny".into(),
                    name: "Deny".into(),
                    kind: "RejectOnce".into(),
                },
            ],
            reply_tx,
        }
    }

    #[test]
    fn persistent_allow_records_standing_approval_once() {
        let mut thread = test_thread(Vec::new(), true);
        let (reply_tx, _reply_rx) = async_channel::bounded(1);
        let request = approval_request_with_options(reply_tx);

        // One-shot allow must NOT record a standing grant.
        thread.record_standing_approval(&request, Some("allow-once"));
        assert!(thread.standing_approvals().is_empty());

        // Denial must not record either.
        thread.record_standing_approval(&request, Some("deny"));
        assert!(thread.standing_approvals().is_empty());

        // Persistent allow records the grant with tool/subject context.
        thread.record_standing_approval(&request, Some("allow-always"));
        assert_eq!(thread.standing_approvals().len(), 1);
        let grant = &thread.standing_approvals()[0];
        assert_eq!(grant.tool_title, "bash");
        assert_eq!(grant.subject.as_deref(), Some("cargo test"));
        assert_eq!(grant.option_label, "Allow always (AllowAlways)");

        // Repeating the same grant dedupes by (tool, subject).
        thread.record_standing_approval(&request, Some("allow-always"));
        assert_eq!(thread.standing_approvals().len(), 1);
    }

    #[test]
    fn plan_updated_replaces_previous_plan() {
        let mut thread = test_thread(Vec::new(), true);

        apply_event_test(
            &mut thread,
            AgentChatEvent::PlanUpdated {
                entries: vec!["Step 1".into()],
            },
        );
        apply_event_test(
            &mut thread,
            AgentChatEvent::PlanUpdated {
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

    /// Helper: build the explicit screenshot resource part.
    fn screenshot_part() -> crate::ai::message_parts::AiContextPart {
        crate::ai::context_contract::ContextAttachmentKind::Screenshot.part()
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
            AgentChatContextBootstrapState::Preparing
        );
        assert_eq!(thread.pending_context_parts.len(), 1);

        // 2. User removes the chip before capture finishes.
        thread.remove_context_part_test(0);

        // 3. Assert: ambient disabled, no blocks, bootstrap ready, chip gone.
        assert!(!thread.pending_ambient_context_enabled);
        assert!(thread.pending_context_blocks.is_empty());
        assert_eq!(
            thread.context_bootstrap_state,
            AgentChatContextBootstrapState::Ready
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
        thread.context_bootstrap_state = AgentChatContextBootstrapState::Ready;

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

    #[test]
    fn follow_up_screenshot_chip_emits_special_attachment_block() {
        let mut thread = test_thread(Vec::new(), false);

        // First turn consumes the existing focused target context.
        thread.add_context_part_test(focused_target_part("choose-theme"));
        let first_blocks = thread.prepare_turn_blocks("summarize this command");
        assert!(
            first_blocks.len() >= 2,
            "first turn should include focused target context"
        );
        assert!(thread.pending_context_consumed);

        // Follow-up: user explicitly types @screenshot.
        thread.add_context_part_test(screenshot_part());
        assert!(
            !thread.pending_context_consumed,
            "new explicit screenshot chip must re-arm pending context"
        );

        let turn = thread
            .take_pending_context_for_turn_with(|part| {
                if AgentChatThread::is_explicit_screenshot_part(part) {
                    return Ok(Some(ContentBlock::Text(TextContent::new(
                        "__test_screenshot_block__",
                    ))));
                }
                Ok(None)
            })
            .expect("follow-up screenshot turn should resolve");

        assert_eq!(
            turn.receipt.attempted, 2,
            "follow-up submit should resolve both the focused target and the explicit screenshot"
        );
        assert_eq!(
            turn.receipt.resolved, 2,
            "both follow-up context parts should resolve"
        );
        assert!(
            turn.receipt.failures.is_empty(),
            "follow-up screenshot should not fail: {:?}",
            turn.receipt.failures
        );
        assert!(
            !turn
                .receipt
                .prompt_prefix
                .contains("kit://context?screenshot=1"),
            "explicit screenshot should not fall back to the text-only MCP resource when the attachment block succeeds"
        );
        assert!(
            turn.receipt.prompt_prefix.contains("focusedTarget"),
            "focused target should still resolve through the normal prompt-prefix path"
        );
        assert_eq!(
            turn.blocks.len(),
            1,
            "only the explicit screenshot should become a special attachment block"
        );
        match &turn.blocks[0] {
            ContentBlock::Text(text) => assert_eq!(text.text, "__test_screenshot_block__"),
            other => panic!("expected test screenshot block, got {other:?}"),
        }
        assert!(
            thread.pending_context_consumed,
            "follow-up screenshot submit should mark pending context consumed"
        );
    }

    #[test]
    fn non_ambient_part_marks_bootstrap_ready_when_no_ambient_capture_is_pending() {
        let mut thread = test_thread(Vec::new(), false);
        thread.context_bootstrap_state = AgentChatContextBootstrapState::Preparing;
        thread.context_bootstrap_note = Some("Queued · sending when context is attached…".into());

        thread.add_context_part_test(focused_target_part("my-script"));

        assert_eq!(
            thread.context_bootstrap_state,
            AgentChatContextBootstrapState::Ready,
            "typed context attachments should not leave the composer stuck in Preparing"
        );
        assert_eq!(
            thread.context_bootstrap_note, None,
            "manual non-ambient attachments should clear the queued bootstrap note"
        );
        assert_eq!(thread.pending_context_parts.len(), 1);
    }

    #[test]
    fn current_context_selector_part_marks_bootstrap_ready_instead_of_waiting_for_ambient_capture()
    {
        let mut thread = test_thread(Vec::new(), false);
        thread.context_bootstrap_state = AgentChatContextBootstrapState::Preparing;
        thread.context_bootstrap_note = Some("Capturing Current Context…".into());

        thread.add_context_part_test(crate::ai::message_parts::AiContextPart::ResourceUri {
            uri: crate::ai::message_parts::ASK_ANYTHING_RESOURCE_URI.to_string(),
            label: "Current Context".to_string(),
        });

        assert_eq!(
            thread.context_bootstrap_state,
            AgentChatContextBootstrapState::Ready
        );
        assert_eq!(thread.context_bootstrap_note, None);
        assert!(!thread.pending_ambient_context_enabled);
    }

    #[test]
    fn successful_context_resolution_clears_prior_failure_note() {
        let mut thread = test_thread(Vec::new(), false);

        thread.add_context_part_test(crate::ai::message_parts::AiContextPart::FilePath {
            path: "/tmp/script-kit-gpui-missing-context.txt".to_string(),
            label: "Missing Context".to_string(),
        });

        let failed = thread.prepare_turn_blocks_with_receipt("first");
        assert!(
            failed
                .receipt
                .as_ref()
                .is_some_and(|receipt| !receipt.failures.is_empty()),
            "missing file should surface as a context resolution failure"
        );
        thread.set_context_resolution_note(failed.receipt.as_ref());
        assert_eq!(
            thread
                .context_bootstrap_note
                .as_ref()
                .map(|note| note.as_ref()),
            Some("1 context attachment unavailable · Missing Context")
        );

        thread.remove_context_part_test(0);
        thread.add_context_part_test(focused_target_part("my-script"));

        let successful = thread.prepare_turn_blocks_with_receipt("second");
        assert!(
            successful
                .receipt
                .as_ref()
                .is_some_and(|receipt| receipt.failures.is_empty()),
            "focused target should resolve cleanly"
        );
        thread.set_context_resolution_note(successful.receipt.as_ref());

        assert_eq!(
            thread.context_bootstrap_note, None,
            "a clean follow-up submit should clear stale failure messaging"
        );
    }

    /// The submitted user message must carry a visible receipt of what text
    /// was attached and where it came from (e.g. `Draft — TextEdit` plus a
    /// snippet), so a rewrite never sends invisible context.
    #[test]
    fn prepared_turn_carries_attachment_receipts_for_transcript() {
        let mut thread = test_thread(Vec::new(), false);
        thread.add_context_part_test(crate::ai::message_parts::AiContextPart::TextBlock {
            label: "Draft \u{2014} TextEdit".to_string(),
            source: "frontmost-app#selection=full".to_string(),
            text: "This  draft\nspans   whitespace and should collapse.".to_string(),
            mime_type: None,
        });

        let prepared = thread.prepare_turn_blocks_with_receipt("rewrite this");

        assert_eq!(prepared.attachments.len(), 1);
        let attachment = &prepared.attachments[0];
        assert_eq!(attachment.label.as_ref(), "Draft \u{2014} TextEdit");
        assert_eq!(
            attachment.snippet.as_ref().map(|s| s.as_ref()),
            Some("This draft spans whitespace and should collapse."),
            "snippet must be whitespace-collapsed attached text"
        );

        // No pending context → no receipts.
        let mut clean = test_thread(Vec::new(), false);
        let empty = clean.prepare_turn_blocks_with_receipt("hello");
        assert!(empty.attachments.is_empty());
    }

    // ── current_setup_requirements tests ─────────────────────

    #[test]
    fn current_setup_requirements_default_when_empty() {
        let thread = test_thread(Vec::new(), false);
        let reqs = thread.current_setup_requirements();
        assert!(
            !reqs.needs_embedded_context,
            "no pending parts/blocks → no embedded context"
        );
        assert!(!reqs.needs_image, "no screenshot parts → no image");
    }

    #[test]
    fn current_setup_requirements_reflects_pending_blocks() {
        let thread = test_thread(
            vec![ContentBlock::Text(TextContent::new("some context"))],
            false,
        );
        let reqs = thread.current_setup_requirements();
        assert!(
            reqs.needs_embedded_context,
            "pending_context_blocks should set needs_embedded_context"
        );
        assert!(!reqs.needs_image, "text block should not set needs_image");
    }

    #[test]
    fn current_setup_requirements_reflects_pending_parts() {
        let mut thread = test_thread(Vec::new(), false);
        thread.add_context_part_test(crate::ai::message_parts::AiContextPart::ResourceUri {
            uri: "kit://context?profile=minimal".to_string(),
            label: "Current Context".to_string(),
        });
        let reqs = thread.current_setup_requirements();
        assert!(
            reqs.needs_embedded_context,
            "pending_context_parts should set needs_embedded_context"
        );
        assert!(
            !reqs.needs_image,
            "non-screenshot part should not set needs_image"
        );
    }

    #[test]
    fn current_setup_requirements_detects_screenshot_part() {
        let mut thread = test_thread(Vec::new(), false);
        thread.add_context_part_test(crate::ai::message_parts::AiContextPart::ResourceUri {
            uri: "kit://context?screenshot=1".to_string(),
            label: "Screenshot".to_string(),
        });
        let reqs = thread.current_setup_requirements();
        assert!(
            reqs.needs_embedded_context,
            "screenshot part implies embedded context"
        );
        assert!(reqs.needs_image, "screenshot part should set needs_image");
    }

    #[test]
    fn current_setup_requirements_unions_with_launch_requirements() {
        let mut thread = test_thread(Vec::new(), false);
        thread.launch_requirements = crate::ai::agent_chat::ui::AgentChatLaunchRequirements {
            needs_embedded_context: true,
            needs_image: false,
        };
        // No pending parts/blocks — should still reflect launch_requirements.
        let reqs = thread.current_setup_requirements();
        assert!(
            reqs.needs_embedded_context,
            "should preserve launch needs_embedded_context"
        );
        assert!(!reqs.needs_image, "no screenshot added → false");

        // Now add screenshot part — should union to true.
        thread.add_context_part_test(crate::ai::message_parts::AiContextPart::ResourceUri {
            uri: "kit://context?screenshot=1".to_string(),
            label: "Screenshot".to_string(),
        });
        let reqs = thread.current_setup_requirements();
        assert!(reqs.needs_embedded_context, "still true from launch");
        assert!(reqs.needs_image, "screenshot part added after open → true");
    }

    #[test]
    fn reset_pending_context_for_new_entry_intent_preserves_messages_but_clears_context_state() {
        let mut thread = test_thread(vec![ContentBlock::Text(TextContent::new("context"))], false);
        thread.messages.push(AgentChatThreadMessage::new(
            1,
            AgentChatThreadMessageRole::Assistant,
            "existing response",
        ));
        thread.add_context_part_test(focused_target_part("existing-chip"));
        thread.context_bootstrap_state = AgentChatContextBootstrapState::Preparing;
        thread.context_bootstrap_note = Some("Capturing Current Context…".into());
        thread.queued_submit_while_bootstrapping = true;

        thread.reset_pending_context_for_new_entry_intent();

        assert_eq!(thread.messages.len(), 1, "transcript history should remain");
        assert!(
            thread.pending_context_parts.is_empty(),
            "stale composer chips must be cleared before reusing the thread"
        );
        assert!(
            thread.pending_context_blocks.is_empty(),
            "hidden staged context must be cleared before reusing the thread"
        );
        assert_eq!(
            thread.context_bootstrap_state,
            AgentChatContextBootstrapState::Ready,
            "reused entry intents must not stay stuck behind old bootstrap work"
        );
        assert_eq!(
            thread.context_bootstrap_note, None,
            "stale bootstrap messaging should be cleared"
        );
        assert!(
            !thread.queued_submit_while_bootstrapping,
            "reused entry intents should not inherit an old queued submit"
        );
    }

    #[test]
    fn replace_pending_context_parts_clears_previous_parts_and_resets_consumption() {
        let mut thread = test_thread(vec![ContentBlock::Text(TextContent::new("hidden"))], false);
        thread.add_context_part_test(focused_target_part("old-chip"));
        thread.pending_context_consumed = true;
        thread.pending_ambient_context_enabled = true;
        thread.context_bootstrap_state = AgentChatContextBootstrapState::Preparing;
        thread.context_bootstrap_note = Some("Capturing Current Context…".into());
        thread.queued_submit_while_bootstrapping = true;

        let replacement = vec![crate::ai::message_parts::AiContextPart::TextBlock {
            label: "Selected Text".to_string(),
            source: "notes://123#selection=0-5".to_string(),
            text: "hello".to_string(),
            mime_type: None,
        }];

        thread.replace_pending_context_parts_test(replacement.clone(), "test_replace");

        assert_eq!(thread.pending_context_parts, replacement);
        assert!(
            thread.pending_context_blocks.is_empty(),
            "replacing pending parts should clear hidden staged blocks"
        );
        assert!(
            !thread.pending_context_consumed,
            "replacing pending parts should re-arm first-submit consumption"
        );
        assert!(
            !thread.pending_ambient_context_enabled,
            "non-ambient replacement should disable stale ambient state"
        );
        assert_eq!(
            thread.context_bootstrap_state,
            AgentChatContextBootstrapState::Ready,
            "non-ambient replacement should clear stale bootstrap state"
        );
        assert_eq!(
            thread.context_bootstrap_note, None,
            "non-ambient replacement should clear stale bootstrap note"
        );
        assert!(
            !thread.queued_submit_while_bootstrapping,
            "replacement should clear stale queued submit state"
        );
    }
}
