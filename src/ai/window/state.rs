use super::*;

/// Input mode for AI window navigation.
/// Keyboard mode suppresses hover highlights to prevent dual-highlight states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum InputMode {
    #[default]
    Mouse,
    Keyboard,
}

/// The main AI chat application view
pub struct AiApp {
    /// Current presentation mode for the AI window.
    pub(super) window_mode: AiWindowMode,

    /// All chats (cached from storage)
    pub(super) chats: Vec<Chat>,

    /// Currently selected chat ID
    pub(super) selected_chat_id: Option<ChatId>,

    /// Cache of last message preview per chat (ChatId -> preview text)
    pub(super) message_previews: std::collections::HashMap<ChatId, String>,

    /// Cache of message counts per chat (for sidebar badges)
    pub(super) message_counts: std::collections::HashMap<ChatId, usize>,

    /// Chat input state (using gpui-component's Input)
    pub(super) input_state: Entity<InputState>,

    /// Search input state for sidebar
    pub(super) search_state: Entity<InputState>,

    /// Current search query
    pub(super) search_query: String,

    /// Generation counter for search (guards against stale async results)
    pub(super) search_generation: u64,

    /// Search result match snippets (ChatId -> snippet text)
    /// Populated when search returns ChatSearchResult with match context.
    pub(super) search_snippets: std::collections::HashMap<ChatId, String>,

    /// Whether the search match was in the title (ChatId -> matched_title)
    pub(super) search_matched_title: std::collections::HashMap<ChatId, bool>,

    /// Whether the sidebar is collapsed
    pub(super) sidebar_collapsed: bool,

    /// Whether the mini-mode history overlay is visible.
    pub(super) showing_mini_history_overlay: bool,

    /// Provider registry with available AI providers
    pub(super) provider_registry: ProviderRegistry,

    /// Available models from all providers
    pub(super) available_models: Vec<ModelInfo>,

    /// Currently selected model for new chats
    pub(super) selected_model: Option<ModelInfo>,

    /// Focus handle for keyboard navigation
    pub(super) focus_handle: FocusHandle,

    /// Subscriptions to keep alive
    pub(super) _subscriptions: Vec<Subscription>,

    // === Streaming State ===
    /// Whether we're currently streaming a response
    pub(super) is_streaming: bool,

    /// Content accumulated during streaming
    pub(super) streaming_content: String,

    /// The chat ID that is currently streaming (guards against chat-switch corruption)
    /// When user switches chats mid-stream, updates for this chat_id are ignored
    /// if selected_chat_id differs
    pub(super) streaming_chat_id: Option<ChatId>,

    /// Generation counter for streaming sessions (guards against stale updates)
    /// Incremented each time streaming starts. Old streaming updates become no-ops.
    pub(super) streaming_generation: u64,

    /// Cancellation flag for the currently active provider stream.
    /// Set by stop_streaming and checked by chunk callbacks.
    pub(crate) streaming_cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,

    /// Streaming sessions where stale completion persistence should be skipped.
    /// Used when a stream is explicitly stopped or the source chat is deleted.
    pub(super) suppressed_orphan_sessions: std::collections::HashSet<StreamingSessionKey>,

    /// Messages for the currently selected chat (cached for display)
    pub(super) current_messages: Vec<Message>,

    /// Virtualized list state for messages (only renders visible messages during scroll)
    pub(super) messages_list_state: ListState,

    /// Virtualized list state for sidebar rows (date headers + chats)
    pub(super) sidebar_list_state: ListState,

    /// Cached box shadows from theme (avoid reloading theme on every render)
    pub(super) cached_box_shadows: Vec<BoxShadow>,

    /// Flag to request input focus on next render.
    /// This replaces the need for a global AI_APP_ENTITY reference.
    /// Set this flag via window.update() and AiApp will process it on render.
    pub(super) needs_focus_input: bool,

    /// Flag to request main focus_handle focus on next render (for command bar keyboard routing).
    /// When true, the render function will focus the main focus_handle instead of the input,
    /// ensuring keyboard events route to the window's key handler for command bar navigation.
    pub(super) needs_command_bar_focus: bool,

    /// Track last persisted bounds for debounced save on close paths
    /// (traffic light, Cmd+W) that don't go through close_ai_window
    pub(super) last_persisted_bounds: Option<gpui::WindowBounds>,

    /// Last time we saved bounds (debounce to avoid too-frequent saves)
    pub(super) last_bounds_save: std::time::Instant,

    /// Theme revision seen - used to detect theme changes and recompute cached values
    pub(super) theme_rev_seen: u64,

    /// Pending image attachment (base64 encoded PNG) to include with next message
    pub(super) pending_image: Option<String>,

    /// Cache of decoded images: base64 hash -> Arc<RenderImage>
    /// Avoids re-decoding images on every render frame.
    pub(super) image_cache: std::collections::HashMap<String, std::sync::Arc<RenderImage>>,

    /// Timestamp when setup command was last copied (for showing "Copied!" feedback)
    pub(super) setup_copied_at: Option<std::time::Instant>,

    /// Claude Code setup feedback message (shown after clicking "Connect to Claude Code")
    /// None = no feedback, Some(msg) = show message (e.g., "Claude CLI not found")
    pub(super) claude_code_setup_feedback: Option<String>,

    /// Whether we're showing the API key input field (configure mode)
    pub(super) showing_api_key_input: bool,

    /// Focused setup button index (0=Configure Vercel AI Gateway, 1=Connect to Claude Code)
    pub(super) setup_button_focus_index: usize,

    /// API key input state (for configure flow)
    pub(super) api_key_input_state: Entity<InputState>,

    // === Command Bar State ===
    /// Command bar component (Cmd+K) - uses the unified CommandBar wrapper
    pub(super) command_bar: CommandBar,

    /// New chat dropdown (Raycast-style + ▼ button in titlebar)
    /// Uses CommandBar for consistent UI with Cmd+K actions
    pub(super) new_chat_command_bar: CommandBar,

    // === Presets State ===
    /// Whether the new chat dropdown (presets) is visible
    pub(super) showing_presets_dropdown: bool,

    /// Available presets
    pub(super) presets: Vec<AiPreset>,

    /// Selected preset index
    pub(super) presets_selected_index: usize,

    // === New Chat Dropdown State (Raycast-style) ===
    // DEPRECATED: `showing_new_chat_dropdown` and related fields are superseded
    // by `show_canonical_new_chat_surface` / `show_new_chat_command_bar`.
    // Retained for backwards compatibility; do not use in new code.
    /// Whether the new chat dropdown is visible (header dropdown)
    /// DEPRECATED: prefer `show_canonical_new_chat_surface`
    pub(super) showing_new_chat_dropdown: bool,

    /// Filter text for new chat dropdown search
    pub(super) new_chat_dropdown_filter: String,

    /// Input state for new chat dropdown search
    pub(super) new_chat_dropdown_input: Entity<InputState>,

    /// Selected section and index in the dropdown (section: 0=last_used, 1=presets, 2=models)
    pub(super) new_chat_dropdown_section: usize,

    /// Selected index within the current section
    pub(super) new_chat_dropdown_index: usize,

    /// Last used settings (derived from recent chats)
    pub(super) last_used_settings: Vec<LastUsedSetting>,

    // === Context Picker State ===
    /// Inline `@` context picker state. `Some` when the picker overlay is open.
    pub(super) context_picker: Option<super::context_picker::types::ContextPickerState>,

    // === Attachments State ===
    /// Pending context parts (file paths and resource URIs) that will be resolved
    /// into prompt blocks at submit time.
    pub(super) pending_context_parts: Vec<crate::ai::message_parts::AiContextPart>,

    /// Index of the pending context part whose preview panel is currently open.
    /// `None` means no preview is shown. Toggled by clicking the info icon on a chip.
    pub(super) context_preview_index: Option<usize>,

    /// Whether the mouse cursor is currently hidden (hidden on keyboard, shown on mouse move)
    pub(super) mouse_cursor_hidden: bool,

    /// Tracks whether the user is navigating with keyboard or mouse.
    /// Mouse mode enables hover highlights; keyboard mode suppresses them.
    pub(super) input_mode: InputMode,

    /// ID of the message whose content was just copied (for showing checkmark feedback)
    pub(super) copied_message_id: Option<String>,

    /// When the copy feedback started (resets after 2 seconds)
    pub(super) copied_at: Option<std::time::Instant>,

    /// When the current streaming session started (for elapsed time display)
    pub(super) streaming_started_at: Option<std::time::Instant>,

    /// Whether the user has manually scrolled up during streaming.
    /// When true, auto-scroll is suppressed so the user can read earlier messages.
    /// Reset when the user scrolls back to the bottom or sends a new message.
    pub(super) user_has_scrolled_up: bool,

    /// Duration of the last completed streaming response (for "Generated in Xs" feedback)
    pub(super) last_streaming_duration: Option<std::time::Duration>,

    /// When the last streaming response completed (for timed "Generated in Xs" display)
    pub(super) last_streaming_completed_at: Option<std::time::Instant>,

    /// Last streaming error message (displayed as a retry-able row below messages)
    pub(super) streaming_error: Option<String>,

    /// Per-chat input drafts preserved across chat switches
    pub(super) chat_drafts: std::collections::HashMap<ChatId, String>,

    /// Message ID currently being edited (inline edit mode)
    pub(super) editing_message_id: Option<String>,

    /// Chat currently being renamed in the sidebar
    pub(super) renaming_chat_id: Option<ChatId>,

    /// Chat pending deletion confirmation (two-step delete: first click shows "Confirm?",
    /// second click actually deletes)
    pub(super) pending_delete_chat_id: Option<ChatId>,

    /// Input state for the sidebar rename field
    pub(super) rename_input_state: Entity<InputState>,

    // === UX Batch 5 State ===
    /// Whether the keyboard shortcuts overlay is visible (Cmd+/)
    pub(super) showing_shortcuts_overlay: bool,

    /// Set of message IDs that the user has explicitly collapsed
    pub(super) collapsed_messages: std::collections::HashSet<String>,

    /// Set of message IDs that the user has explicitly expanded (overrides auto-collapse)
    pub(super) expanded_messages: std::collections::HashSet<String>,

    /// Feedback timestamp for "Exported!" clipboard feedback
    pub(super) export_copied_at: Option<std::time::Instant>,

    /// Feedback timestamp for "Copied!" transcript action state
    pub(super) chat_transcript_copied_at: Option<std::time::Instant>,

    /// Debounce task for search input — cancelled and replaced on each keystroke.
    /// When the user types, we delay the DB query by 150ms; if another keystroke
    /// arrives before the timer fires, the old task is dropped (cancelled).
    pub(super) search_debounce_task: Option<gpui::Task<()>>,

    /// The last message-preparation receipt, persisted for UI/debug/agent inspection.
    /// Updated immediately after preparation in both `submit_message` and `handle_start_chat`.
    pub(super) last_prepared_message_receipt:
        Option<crate::ai::message_parts::PreparedMessageReceipt>,

    /// Full machine-readable preflight audit for the most recent composer attempt.
    /// This is the canonical payload behind "Inspect Context".
    pub(super) last_preflight_audit: Option<crate::ai::AiPreflightAudit>,

    /// The last context-resolution receipt, persisted for compact UI summary after submit.
    /// Cleared when submitting with no pending context parts; set when parts are resolved.
    pub(super) last_context_receipt: Option<crate::ai::message_parts::ContextResolutionReceipt>,

    /// Whether the full prepared-message inspector is visible (toggled via ⌥⌘I).
    pub(super) show_context_inspector: bool,

    /// Whether the context drawer is open (shows per-part provenance rows).
    /// Toggled by clicking the context bar summary line.
    pub(super) show_context_drawer: bool,

    /// Pre-submit context preflight state.
    /// Updated whenever pending context parts change. Reuses the same
    /// `prepare_outbound_user_message` compiler as the submit path so
    /// the preview can never drift from what is actually sent.
    pub(super) context_preflight: super::context_preflight::ContextPreflightState,
}

/// Machine-readable snapshot of AI window state for agentic tests and debugging.
///
/// Serializable to JSON — callers can assert individual fields without
/// reaching into `AiApp` internals. Every dismissible overlay and modal
/// is represented so the full Esc-chain can be verified deterministically.
///
/// **Privacy:** This struct contains structural metadata (booleans, counts,
/// mode strings, UUIDs) plus `search_query` (user-typed sidebar filter text).
/// The telemetry helper `log_ai_state` redacts `search_query` to length-only
/// so no user input text reaches the telemetry sink. It never captures
/// conversation content, API keys, or other PII.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiMiniDebugSnapshot {
    pub window_mode: String,
    pub history_overlay_visible: bool,
    pub command_bar_open: bool,
    pub new_chat_menu_open: bool,
    pub presets_dropdown_open: bool,
    pub api_key_input_visible: bool,
    pub context_picker_open: bool,
    pub selected_model: Option<String>,
    pub selected_chat_id: Option<String>,
    pub pending_context_parts: usize,
    pub has_pending_image: bool,
    pub is_streaming: bool,
    pub streaming_error_present: bool,
    pub pending_delete_chat_present: bool,
    pub chat_count: usize,
    pub current_message_count: usize,
    pub sidebar_collapsed: bool,
    pub show_context_inspector: bool,
    pub show_context_drawer: bool,
    pub search_query: String,
    pub shortcuts_overlay_visible: bool,
    pub editing_message_present: bool,
    pub renaming_chat_present: bool,
}

impl AiApp {
    /// Build a serializable debug snapshot of the current AI window state.
    ///
    /// Used by agentic tests and future automation to assert state without
    /// reaching into struct internals.
    pub(crate) fn debug_snapshot(&self) -> AiMiniDebugSnapshot {
        AiMiniDebugSnapshot {
            window_mode: if self.window_mode.is_mini() {
                "mini".to_string()
            } else {
                "full".to_string()
            },
            history_overlay_visible: self.showing_mini_history_overlay,
            command_bar_open: self.command_bar.is_open(),
            new_chat_menu_open: self.new_chat_command_bar.is_open(),
            presets_dropdown_open: self.showing_presets_dropdown,
            api_key_input_visible: self.showing_api_key_input,
            context_picker_open: self.is_context_picker_open(),
            selected_model: self.selected_model.as_ref().map(|m| m.display_name.clone()),
            selected_chat_id: self.selected_chat_id.map(|id| id.to_string()),
            pending_context_parts: self.pending_context_parts.len(),
            has_pending_image: self.pending_image.is_some(),
            is_streaming: self.is_streaming,
            streaming_error_present: self.streaming_error.is_some(),
            pending_delete_chat_present: self.pending_delete_chat_id.is_some(),
            chat_count: self.chats.len(),
            current_message_count: self.current_messages.len(),
            sidebar_collapsed: self.sidebar_collapsed,
            show_context_inspector: self.show_context_inspector,
            show_context_drawer: self.show_context_drawer,
            search_query: self.search_query.clone(),
            shortcuts_overlay_visible: self.showing_shortcuts_overlay,
            editing_message_present: self.editing_message_id.is_some(),
            renaming_chat_present: self.renaming_chat_id.is_some(),
        }
    }
}

impl AiMiniDebugSnapshot {
    /// Remove user-entered text before exposing the snapshot outside the UI layer.
    pub(crate) fn redact_for_external_use(mut self) -> Self {
        self.search_query.clear();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{AiMiniDebugSnapshot, InputMode};

    #[test]
    fn test_ai_window_input_mode_defaults_to_mouse() {
        assert_eq!(InputMode::default(), InputMode::Mouse);
    }

    #[test]
    fn debug_snapshot_serde_roundtrip_uses_camel_case() {
        let snapshot = AiMiniDebugSnapshot {
            window_mode: "mini".to_string(),
            history_overlay_visible: true,
            command_bar_open: false,
            new_chat_menu_open: false,
            presets_dropdown_open: false,
            api_key_input_visible: false,
            context_picker_open: false,
            selected_model: Some("Claude 3.7 Sonnet".to_string()),
            selected_chat_id: Some("abc-123".to_string()),
            pending_context_parts: 2,
            has_pending_image: false,
            is_streaming: true,
            streaming_error_present: false,
            pending_delete_chat_present: false,
            chat_count: 5,
            current_message_count: 12,
            sidebar_collapsed: false,
            show_context_inspector: false,
            show_context_drawer: false,
            search_query: String::new(),
            shortcuts_overlay_visible: false,
            editing_message_present: false,
            renaming_chat_present: false,
        };

        let json = serde_json::to_string(&snapshot).expect("serialize");
        // Verify camelCase field names in output
        assert!(
            json.contains("\"windowMode\""),
            "must use camelCase: windowMode"
        );
        assert!(
            json.contains("\"historyOverlayVisible\""),
            "must use camelCase: historyOverlayVisible"
        );
        assert!(
            json.contains("\"isStreaming\""),
            "must use camelCase: isStreaming"
        );
        assert!(
            json.contains("\"selectedModel\""),
            "must use camelCase: selectedModel"
        );

        // Roundtrip
        let deserialized: AiMiniDebugSnapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(snapshot, deserialized);
    }

    #[test]
    fn debug_snapshot_redacts_search_query_for_external_use() {
        let snapshot = AiMiniDebugSnapshot {
            window_mode: "mini".to_string(),
            history_overlay_visible: true,
            command_bar_open: false,
            new_chat_menu_open: false,
            presets_dropdown_open: false,
            api_key_input_visible: false,
            context_picker_open: false,
            selected_model: None,
            selected_chat_id: None,
            pending_context_parts: 0,
            has_pending_image: false,
            is_streaming: false,
            streaming_error_present: false,
            pending_delete_chat_present: false,
            chat_count: 0,
            current_message_count: 0,
            sidebar_collapsed: false,
            show_context_inspector: false,
            show_context_drawer: false,
            search_query: "sensitive search".to_string(),
            shortcuts_overlay_visible: false,
            editing_message_present: false,
            renaming_chat_present: false,
        };

        let redacted = snapshot.redact_for_external_use();
        assert!(redacted.search_query.is_empty());
    }
}
