struct ScriptListApp {
    /// H1 Optimization: Arc-wrapped scripts for cheap cloning during filter operations
    scripts: Vec<std::sync::Arc<scripts::Script>>,
    /// H1 Optimization: Arc-wrapped scriptlets for cheap cloning during filter operations
    scriptlets: Vec<std::sync::Arc<scripts::Scriptlet>>,
    builtin_entries: Vec<builtins::BuiltInEntry>,
    /// Cached list of installed applications for main search and AppLauncherView
    apps: Vec<app_launcher::AppInfo>,
    /// P0 FIX: Cached clipboard entries for ClipboardHistoryView (avoids cloning per frame)
    cached_clipboard_entries: Vec<clipboard_history::ClipboardEntryMeta>,
    /// Focused clipboard entry ID for action handling in ClipboardHistoryView
    #[allow(dead_code)]
    focused_clipboard_entry_id: Option<String>,
    /// P0 FIX: Cached windows for WindowSwitcherView (avoids cloning per frame)
    cached_windows: Vec<window_control::WindowInfo>,
    /// Cached file results for FileSearchView (avoids cloning per frame)
    cached_file_results: Vec<file_search::FileResult>,
    selected_index: usize,
    /// Main menu filter text (mirrors gpui-component input state)
    filter_text: String,
    /// Inline calculator result derived from filter_text when expression is math-like
    pub inline_calculator: Option<crate::calculator::CalculatorInlineResult>,
    /// gpui-component input state for the main filter
    gpui_input_state: Entity<InputState>,
    gpui_input_focused: bool,
    #[allow(dead_code)]
    gpui_input_subscriptions: Vec<Subscription>,
    /// Subscription for window bounds changes (saves position on drag)
    #[allow(dead_code)]
    bounds_subscription: Option<Subscription>,
    /// Subscription for window appearance changes (light/dark mode)
    #[allow(dead_code)]
    appearance_subscription: Option<Subscription>,
    /// Suppress handling of programmatic InputEvent::Change updates.
    suppress_filter_events: bool,
    /// Sync gpui input text on next render when window access is available.
    pending_filter_sync: bool,
    /// Pending placeholder text to set on next render (needs Window access).
    pending_placeholder: Option<String>,
    last_output: Option<SharedString>,
    focus_handle: FocusHandle,
    show_logs: bool,
    /// Theme wrapped in Arc for cheap cloning when passing to prompts/dialogs
    theme: std::sync::Arc<theme::Theme>,
    #[allow(dead_code)]
    config: config::Config,
    // Scroll activity tracking for scrollbar fade
    /// Current animated visibility for scrollbar fade (0.0 invisible .. 1.0 visible)
    scrollbar_visibility: crate::transitions::Opacity,
    /// Generation counter for cancelling stale fade-out tasks when new scroll activity occurs
    scrollbar_fade_gen: u64,
    /// Timestamp of last scroll activity (for fade-out timer)
    last_scroll_time: Option<std::time::Instant>,
    // Interactive script state
    current_view: AppView,
    script_session: SharedSession,
    // Prompt-specific state (used when view is ArgPrompt or DivPrompt)
    // Uses TextInputState for selection and clipboard support
    arg_input: TextInputState,
    arg_selected_index: usize,
    // Channel for receiving prompt messages from script thread (async_channel for event-driven)
    prompt_receiver: Option<async_channel::Receiver<PromptMessage>>,
    // Channel for sending responses back to script
    // FIX: Use SyncSender (bounded channel) to prevent OOM from slow scripts
    response_sender: Option<mpsc::SyncSender<Message>>,
    // List state for variable-height list (supports section headers at 24px + items at 48px)
    main_list_state: ListState,
    // Scroll handle for uniform_list (still used for backward compat in some views)
    list_scroll_handle: UniformListScrollHandle,
    // P0: Scroll handle for virtualized arg prompt choices
    arg_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for clipboard history list
    clipboard_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for emoji picker grid rows
    emoji_scroll_handle: UniformListScrollHandle,
    // Scroll handle for window switcher list
    window_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for design gallery list
    design_gallery_scroll_handle: UniformListScrollHandle,
    // Scroll handle for file search list
    file_search_scroll_handle: UniformListScrollHandle,
    // Scroll handle for theme chooser list
    #[allow(dead_code)]
    theme_chooser_scroll_handle: UniformListScrollHandle,
    // File search loading state (true while mdfind is running)
    file_search_loading: bool,
    // Debounce task for file search (cancelled when new input arrives)
    file_search_debounce_task: Option<gpui::Task<()>>,
    // Current directory being listed (for instant filter mode)
    file_search_current_dir: Option<String>,
    // Frozen filter during directory transitions (prevents wrong results flash)
    // When Some, use this filter instead of deriving from query
    // Outer Option: None = use query filter, Some = use frozen filter
    // Inner Option: None = no filter, Some(s) = filter by s
    file_search_frozen_filter: Option<Option<String>>,
    // Path of the file selected for actions (for file search actions handling)
    file_search_actions_path: Option<String>,
    // Generation counter for ignoring stale search results
    // Incremented on each new search, results with old gen are discarded
    file_search_gen: u64,
    // Cancel token for in-flight search (set to true to stop mdfind)
    file_search_cancel: Option<file_search::CancelToken>,
    // Pre-computed display indices after Nucleo filtering/sorting
    // This is computed once when results change or filter changes (not in render)
    // Vec of indices into cached_file_results, sorted by match quality
    file_search_display_indices: Vec<usize>,
    // Actions popup overlay
    show_actions_popup: bool,
    // ActionsDialog entity for focus management
    actions_dialog: Option<Entity<ActionsDialog>>,
    // Cursor blink state and focus tracking
    cursor_visible: bool,
    /// Which input currently has focus (for cursor display)
    focused_input: FocusedInput,
    // Current script process PID for explicit cleanup (belt-and-suspenders)
    current_script_pid: Option<u32>,
    // P1: Cache for filtered_results() - invalidate on filter_text change only
    cached_filtered_results: Vec<scripts::SearchResult>,
    filter_cache_key: String,
    // P1: Cache for get_grouped_results() - invalidate on filter_text change only
    // This avoids recomputing grouped results 9+ times per keystroke
    // P1-Arc: Use Arc<[T]> for cheap clone in render closures
    cached_grouped_items: Arc<[GroupedListItem]>,
    cached_grouped_flat_results: Arc<[scripts::SearchResult]>,
    #[allow(dead_code)]
    cached_grouped_first_selectable_index: Option<usize>,
    #[allow(dead_code)]
    cached_grouped_last_selectable_index: Option<usize>,
    grouped_cache_key: String,
    // P3: Two-stage filter - display vs search separation with coalescing
    /// What the search cache is built from (may lag behind filter_text during rapid typing)
    computed_filter_text: String,
    /// Coalesces filter updates and keeps only the latest value per tick
    filter_coalescer: FilterCoalescer,
    // Scroll stabilization: track last scrolled-to index to avoid redundant scroll_to_item calls
    last_scrolled_index: Option<usize>,
    // Preview cache: avoid re-reading file and re-highlighting on every render
    preview_cache_path: Option<String>,
    preview_cache_lines: Vec<syntax::HighlightedLine>,
    // Scriptlet preview cache: avoid re-highlighting scriptlet code on every render
    // Key is scriptlet name (unique within session), value is highlighted lines
    scriptlet_preview_cache_key: Option<String>,
    scriptlet_preview_cache_lines: Vec<syntax::HighlightedLine>,
    // Current design variant for hot-swappable UI designs
    current_design: DesignVariant,
    // Toast manager for notification queue
    toast_manager: ToastManager,
    // Cache for decoded clipboard images (entry_id -> RenderImage)
    clipboard_image_cache: std::collections::HashMap<String, Arc<gpui::RenderImage>>,
    // Frecency store for tracking script usage
    frecency_store: FrecencyStore,
    // Mouse hover tracking - independent from selected_index (keyboard focus)
    // hovered_index shows subtle visual feedback, selected_index shows full focus styling
    hovered_index: Option<usize>,
    // Input mode: Mouse vs Keyboard - when Keyboard, hover effects are disabled
    // to prevent dual-highlight. Mouse movement switches back to Mouse mode.
    input_mode: InputMode,
    // Fallback mode: when true, we're showing fallback commands instead of scripts
    // This happens when filter_text doesn't match any scripts
    fallback_mode: bool,
    // Selected index within the fallback list (0-based)
    fallback_selected_index: usize,
    // Cached fallback items for the current filter_text
    cached_fallbacks: Vec<crate::fallbacks::FallbackItem>,
    // Theme before chooser was opened (for cancel/restore)
    theme_before_chooser: Option<std::sync::Arc<theme::Theme>>,
    // P0-2: Debounce hover notify calls (16ms window to reduce 50% unnecessary re-renders)
    last_hover_notify: std::time::Instant,
    // Render log deduplication: only log when meaningful state changes (not cursor blink)
    last_render_log_filter: String,
    last_render_log_selection: usize,
    last_render_log_item_count: usize,
    /// Transient flag: true if current render has state changes worth logging
    /// Set at start of render_script_list, read by render_preview_panel
    log_this_render: bool,
    // Filter performance tracking: start time of filter change event
    filter_perf_start: Option<std::time::Instant>,
    // Pending path action - when set, show ActionsDialog for this path
    // Uses Arc<Mutex<>> so callbacks can write to it
    pending_path_action: Arc<Mutex<Option<PathInfo>>>,
    // Signal to close path actions dialog (set by callback on Escape/__cancel__)
    close_path_actions: Arc<Mutex<bool>>,
    // Shared state: whether path actions dialog is currently showing
    // Used by PathPrompt to implement toggle behavior for Cmd+K
    path_actions_showing: Arc<Mutex<bool>>,
    // Shared state: current search text in path actions dialog
    // Used by PathPrompt to display search in header (like main menu does)
    path_actions_search_text: Arc<Mutex<String>>,
    // DEPRECATED: These mutexes were used for polling in render before event-based refactor.
    // Kept for reset_to_script_list cleanup. Will be removed in future cleanup pass.
    #[allow(dead_code)]
    pending_path_action_result: Arc<Mutex<Option<(String, PathInfo)>>>,
    /// Alias registry: lowercase_alias -> script_path (for O(1) lookup)
    /// Conflict rule: first-registered wins
    alias_registry: std::collections::HashMap<String, String>,
    /// Shortcut registry: shortcut -> script_path (for O(1) lookup)
    /// Conflict rule: first-registered wins
    shortcut_registry: std::collections::HashMap<String, String>,
    /// SDK actions set via setActions() - stored for trigger_action_by_name lookup
    sdk_actions: Option<Vec<protocol::ProtocolAction>>,
    /// SDK action shortcuts: normalized_shortcut -> action_name (for O(1) lookup)
    action_shortcuts: std::collections::HashMap<String, String>,
    /// Debug grid overlay configuration (None = hidden)
    grid_config: Option<debug_grid::GridConfig>,
    // Navigation coalescing for rapid arrow key events (20ms window)
    // NOTE: Currently unused - arrow keys handled in interceptor without coalescing
    #[allow(dead_code)]
    nav_coalescer: NavCoalescer,
    // Wheel scroll accumulator for smooth trackpad scrolling
    // Accumulates fractional deltas until they cross 1.0, then converts to item steps
    wheel_accum: f32,
    // Window focus tracking - for detecting focus lost and auto-dismissing prompts
    // When window loses focus while in a dismissable prompt, close and reset
    was_window_focused: bool,
    /// Pin state - when true, window stays open on blur (only closes via ESC/Cmd+W)
    /// Toggle with Cmd+Shift+P
    is_pinned: bool,
    /// Pending focus target - when set, focus will be applied once on next render
    /// then cleared. This avoids the "perpetually enforce focus in render()" anti-pattern.
    /// DEPRECATED: Use focus_coordinator instead. This remains for gradual migration.
    pending_focus: Option<FocusTarget>,
    /// Focus coordinator - centralized focus management with push/pop overlay semantics.
    /// This is the new unified focus system that replaces focused_input + pending_focus.
    focus_coordinator: focus_coordinator::FocusCoordinator,
    // Show warning banner when bun is not available
    show_bun_warning: bool,
    // Builtin confirmation channel - for modal callback to signal completion
    // When a dangerous builtin requires confirmation, we open a modal and the callback
    // sends (entry_id, confirmed) through this channel
    builtin_confirm_sender: async_channel::Sender<(String, bool)>,
    builtin_confirm_receiver: async_channel::Receiver<(String, bool)>,
    // Scroll stabilization: track last scrolled-to index for each scroll handle
    #[allow(dead_code)]
    last_scrolled_main: Option<usize>,
    #[allow(dead_code)]
    last_scrolled_arg: Option<usize>,
    #[allow(dead_code)]
    last_scrolled_clipboard: Option<usize>,
    #[allow(dead_code)]
    last_scrolled_window: Option<usize>,
    #[allow(dead_code)]
    last_scrolled_design_gallery: Option<usize>,
    // Menu bar integration: Now handled by frontmost_app_tracker module
    // which pre-fetches menu items in background when apps activate
    /// Shortcut recorder state - when Some, shows the inline recorder overlay
    shortcut_recorder_state: Option<ShortcutRecorderState>,
    /// The shortcut recorder entity (persisted to maintain focus)
    shortcut_recorder_entity:
        Option<Entity<crate::components::shortcut_recorder::ShortcutRecorder>>,
    /// Alias input state - when Some, shows the alias input modal
    alias_input_state: Option<AliasInputState>,
    /// The alias input entity (persisted to maintain focus)
    alias_input_entity: Option<Entity<crate::components::alias_input::AliasInput>>,
    /// Input history for shell-like up/down navigation through previous inputs
    input_history: input_history::InputHistory,
    /// Pending API key configuration - tracks which provider is being configured
    /// Used to show success toast after EnvPrompt completes
    pending_api_key_config: Option<String>,
    /// Sender for API key configuration completion signals
    /// The EnvPrompt callback uses this to signal when done
    api_key_completion_sender: mpsc::SyncSender<(String, bool)>,
    /// Receiver for API key configuration completion signals
    /// Checked by timer to trigger toast and view reset
    api_key_completion_receiver: mpsc::Receiver<(String, bool)>,
    /// Whether the current built-in view was opened from the main menu.
    /// When true, ESC returns to main menu. When false (opened via hotkey/protocol), ESC closes window.
    opened_from_main_menu: bool,
    /// Sender for inline chat escape signals
    /// The ChatPrompt escape callback uses this to signal when ESC is pressed
    inline_chat_escape_sender: mpsc::SyncSender<()>,
    /// Receiver for inline chat escape signals
    /// Checked by timer to trigger view reset
    inline_chat_escape_receiver: mpsc::Receiver<()>,
    /// Sender for inline chat configure signals
    /// The ChatPrompt configure callback uses this to signal when user wants to configure API key
    inline_chat_configure_sender: mpsc::SyncSender<()>,
    /// Receiver for inline chat configure signals
    /// Checked by timer to trigger API key configuration prompt
    inline_chat_configure_receiver: mpsc::Receiver<()>,
    /// Sender for inline chat Claude Code signals
    /// The ChatPrompt Claude Code callback uses this to signal when user wants to enable Claude Code
    inline_chat_claude_code_sender: mpsc::SyncSender<()>,
    /// Receiver for inline chat Claude Code signals
    /// Checked by timer to trigger Claude Code enablement
    inline_chat_claude_code_receiver: mpsc::Receiver<()>,
    /// Sender for naming dialog completion signals
    /// Some(json_payload) = user submitted a name, None = user cancelled
    naming_submit_sender: mpsc::SyncSender<Option<String>>,
    /// Receiver for naming dialog completion signals
    /// Checked in render loop to handle naming dialog submit/cancel
    naming_submit_receiver: mpsc::Receiver<Option<String>>,
    /// Opacity offset for light theme adjustment
    /// Use Cmd+Shift+[ to decrease and Cmd+Shift+] to increase
    /// Range: -0.5 to +0.5 (added to base opacity values)
    light_opacity_offset: f32,
    /// Whether the mouse cursor is currently hidden (hidden while typing, shown on mouse move)
    mouse_cursor_hidden: bool,
    /// Cached provider registry built in background at startup.
    /// Avoids blocking the UI thread when opening inline AI chat.
    cached_provider_registry: Option<crate::ai::ProviderRegistry>,
}

/// Result of alias matching - either a Script or Scriptlet
#[derive(Clone, Debug)]
enum AliasMatch {
    Script(Arc<scripts::Script>),
    Scriptlet(Arc<scripts::Scriptlet>),
    BuiltIn(Arc<builtins::BuiltInEntry>),
    App(Arc<app_launcher::AppInfo>),
}
