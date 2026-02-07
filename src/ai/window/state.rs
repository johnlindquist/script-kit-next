use super::*;

/// The main AI chat application view
pub struct AiApp {
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

    /// Whether the sidebar is collapsed
    pub(super) sidebar_collapsed: bool,

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
    /// Whether the new chat dropdown is visible (header dropdown)
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

    // === Attachments State ===
    /// Whether the attachments picker is visible
    pub(super) showing_attachments_picker: bool,

    /// List of pending attachments (file paths)
    pub(super) pending_attachments: Vec<String>,

    /// Whether the mouse cursor is currently hidden (hidden on keyboard, shown on mouse move)
    pub(super) mouse_cursor_hidden: bool,

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
}
