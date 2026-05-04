#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FileSearchSelectionMode {
    AutoFirst,
    UserLockedPath,
}

/// Render and filter-performance diagnostics owned by the main script list surface.
#[derive(Debug)]
struct MainMenuRenderDiagnosticsState {
    /// Last filter value that produced render diagnostics.
    last_render_log_filter: String,
    /// Last selection index that produced render diagnostics.
    last_render_log_selection: usize,
    /// Last item count that produced render diagnostics.
    last_render_log_item_count: usize,
    /// True when the current render changed enough to log preview diagnostics.
    log_this_render: bool,
    /// Start time for the current input-to-grouped-results performance sample.
    filter_perf_start: Option<std::time::Instant>,
}

impl Default for MainMenuRenderDiagnosticsState {
    fn default() -> Self {
        Self {
            last_render_log_filter: String::new(),
            last_render_log_selection: usize::MAX,
            last_render_log_item_count: usize::MAX,
            log_this_render: true,
            filter_perf_start: None,
        }
    }
}

/// Fallback commands shown when the main-menu search has no direct matches.
#[derive(Default)]
struct MainMenuFallbackState {
    active: bool,
    selected_index: usize,
    cached_items: Vec<crate::fallbacks::FallbackItem>,
}

impl MainMenuFallbackState {
    fn is_active(&self) -> bool {
        self.active && !self.cached_items.is_empty()
    }

    fn clear(&mut self) {
        self.active = false;
        self.selected_index = 0;
        self.cached_items.clear();
    }

    fn replace_items(&mut self, items: Vec<crate::fallbacks::FallbackItem>) {
        self.active = !items.is_empty();
        self.selected_index = 0;
        self.cached_items = items;
    }

    fn selected_item(&self) -> Option<&crate::fallbacks::FallbackItem> {
        if !self.is_active() {
            return None;
        }
        self.cached_items.get(self.selected_index)
    }

    fn move_up(&mut self) -> bool {
        if !self.is_active() || self.selected_index == 0 {
            return false;
        }
        self.selected_index -= 1;
        true
    }

    fn move_down(&mut self) -> bool {
        if !self.is_active() {
            return false;
        }
        if self.selected_index >= self.cached_items.len().saturating_sub(1) {
            return false;
        }
        self.selected_index += 1;
        true
    }
}

const MAIN_MENU_RESULT_CACHE_UNINITIALIZED_KEY: &str = "\0_UNINITIALIZED_\0";
const MAIN_MENU_RESULT_CACHE_INVALIDATED_KEY: &str = "\0_INVALIDATED_\0";
const MAIN_MENU_RESULT_CACHE_APPS_LOADED_KEY: &str = "\0_APPS_LOADED_\0";
const QUICK_TERMINAL_INITIAL_COLS: u16 = 80;
const QUICK_TERMINAL_INITIAL_ROWS: u16 = 24;
const QUICK_TERMINAL_WARM_TTL: std::time::Duration = std::time::Duration::from_secs(600);

/// Search and grouped-result caches owned by the main script-list surface.
struct MainMenuResultCacheState {
    cached_filtered_results: Vec<scripts::SearchResult>,
    filter_cache_key: String,
    cached_grouped_items: Arc<[GroupedListItem]>,
    cached_grouped_flat_results: Arc<[scripts::SearchResult]>,
    cached_grouped_first_selectable_index: Option<usize>,
    cached_grouped_last_selectable_index: Option<usize>,
    grouped_cache_key: String,
}

impl Default for MainMenuResultCacheState {
    fn default() -> Self {
        Self {
            cached_filtered_results: Vec::new(),
            filter_cache_key: String::from(MAIN_MENU_RESULT_CACHE_UNINITIALIZED_KEY),
            cached_grouped_items: Arc::from([]),
            cached_grouped_flat_results: Arc::from([]),
            cached_grouped_first_selectable_index: None,
            cached_grouped_last_selectable_index: None,
            grouped_cache_key: String::from(MAIN_MENU_RESULT_CACHE_UNINITIALIZED_KEY),
        }
    }
}

impl MainMenuResultCacheState {
    fn has_filtered_results_for(&self, filter_text: &str) -> bool {
        self.filter_cache_key == filter_text
    }

    fn filtered_cache_key(&self) -> &str {
        &self.filter_cache_key
    }

    fn clone_filtered_results(&self) -> Vec<scripts::SearchResult> {
        self.cached_filtered_results.clone()
    }

    fn filtered_results(&self) -> &Vec<scripts::SearchResult> {
        &self.cached_filtered_results
    }

    fn store_filtered_results(&mut self, filter_text: String, results: Vec<scripts::SearchResult>) {
        self.cached_filtered_results = results;
        self.filter_cache_key = filter_text;
    }

    fn has_grouped_results_for(&self, computed_filter_text: &str) -> bool {
        self.grouped_cache_key == computed_filter_text
    }

    fn grouped_cache_key(&self) -> &str {
        &self.grouped_cache_key
    }

    fn clone_grouped_results(&self) -> (Arc<[GroupedListItem]>, Arc<[scripts::SearchResult]>) {
        (
            self.cached_grouped_items.clone(),
            self.cached_grouped_flat_results.clone(),
        )
    }

    fn grouped_items(&self) -> &[GroupedListItem] {
        &self.cached_grouped_items
    }

    fn grouped_flat_results(&self) -> &[scripts::SearchResult] {
        &self.cached_grouped_flat_results
    }

    fn grouped_flat_result_count(&self) -> usize {
        self.cached_grouped_flat_results.len()
    }

    fn flat_result_index_for_grouped_item(&self, grouped_index: usize) -> Option<usize> {
        match self.cached_grouped_items.get(grouped_index) {
            Some(GroupedListItem::Item(result_idx)) => Some(*result_idx),
            _ => None,
        }
    }

    fn flat_result_index_for_coerced_grouped_selection(
        &self,
        selected_index: usize,
    ) -> Option<(usize, usize)> {
        let grouped_index =
            crate::list_item::coerce_selection(self.grouped_items(), selected_index)?;
        let result_idx = self.flat_result_index_for_grouped_item(grouped_index)?;
        Some((grouped_index, result_idx))
    }

    fn search_result_for_flat_index(
        &self,
        flat_result_index: usize,
    ) -> Option<&scripts::SearchResult> {
        self.cached_grouped_flat_results.get(flat_result_index)
    }

    fn cloned_search_result_for_flat_index(
        &self,
        flat_result_index: usize,
    ) -> Option<scripts::SearchResult> {
        self.search_result_for_flat_index(flat_result_index)
            .cloned()
    }

    fn search_result_for_grouped_item(
        &self,
        grouped_index: usize,
    ) -> Option<&scripts::SearchResult> {
        let result_idx = self.flat_result_index_for_grouped_item(grouped_index)?;
        self.search_result_for_flat_index(result_idx)
    }

    fn cloned_search_result_for_grouped_item(
        &self,
        grouped_index: usize,
    ) -> Option<scripts::SearchResult> {
        self.search_result_for_grouped_item(grouped_index).cloned()
    }

    fn first_search_result_at_or_after_grouped_item(
        &self,
        grouped_index: usize,
    ) -> Option<&scripts::SearchResult> {
        (grouped_index..self.cached_grouped_items.len())
            .find_map(|index| self.search_result_for_grouped_item(index))
    }

    fn cloned_first_search_result_at_or_after_grouped_item(
        &self,
        grouped_index: usize,
    ) -> Option<scripts::SearchResult> {
        self.first_search_result_at_or_after_grouped_item(grouped_index)
            .cloned()
    }

    fn grouped_search_results(&self) -> impl Iterator<Item = &scripts::SearchResult> {
        self.cached_grouped_items
            .iter()
            .filter_map(|item| match item {
                GroupedListItem::Item(result_idx) => self.search_result_for_flat_index(*result_idx),
                GroupedListItem::SectionHeader(..) => None,
            })
    }

    fn selectable_bounds(&self) -> (Option<usize>, Option<usize>) {
        (
            self.cached_grouped_first_selectable_index,
            self.cached_grouped_last_selectable_index,
        )
    }

    fn first_selectable_index(&self) -> Option<usize> {
        self.cached_grouped_first_selectable_index
    }

    fn last_selectable_index(&self) -> Option<usize> {
        self.cached_grouped_last_selectable_index
    }

    fn has_selectable_grouped_item(&self) -> bool {
        self.cached_grouped_first_selectable_index.is_some()
    }

    fn store_grouped_results(
        &mut self,
        computed_filter_text: String,
        grouped_items: Vec<GroupedListItem>,
        flat_results: Vec<scripts::SearchResult>,
        first_selectable_index: Option<usize>,
        last_selectable_index: Option<usize>,
    ) {
        self.cached_grouped_first_selectable_index = first_selectable_index;
        self.cached_grouped_last_selectable_index = last_selectable_index;
        self.cached_grouped_items = grouped_items.into();
        self.cached_grouped_flat_results = flat_results.into();
        self.grouped_cache_key = computed_filter_text;
    }

    fn mark_apps_loaded(&mut self) {
        self.filter_cache_key = String::from(MAIN_MENU_RESULT_CACHE_APPS_LOADED_KEY);
        self.grouped_cache_key = String::from(MAIN_MENU_RESULT_CACHE_APPS_LOADED_KEY);
    }

    fn invalidate_filtered_results(&mut self) {
        self.filter_cache_key = String::from(MAIN_MENU_RESULT_CACHE_INVALIDATED_KEY);
    }

    fn invalidate_grouped_results(&mut self) {
        self.cached_grouped_first_selectable_index = None;
        self.cached_grouped_last_selectable_index = None;
        self.grouped_cache_key = String::from(MAIN_MENU_RESULT_CACHE_INVALIDATED_KEY);
    }
}

struct ScriptListApp {
    /// H1 Optimization: Arc-wrapped scripts for cheap cloning during filter operations
    scripts: Vec<std::sync::Arc<scripts::Script>>,
    /// H1 Optimization: Arc-wrapped scriptlets for cheap cloning during filter operations
    scriptlets: Vec<std::sync::Arc<scripts::Scriptlet>>,
    /// Plugin-owned skills for main-menu search and ACP skill launch
    skills: Vec<std::sync::Arc<crate::plugins::PluginSkill>>,
    /// Latest validation report describing scripts that were excluded from the
    /// catalog (e.g., binding collisions). Used to surface the launcher
    /// "Script Issues" row and feed the diagnostic view.
    script_validation_report: Option<std::sync::Arc<crate::scripts::ValidationReport>>,
    builtin_entries: Vec<builtins::BuiltInEntry>,
    /// Cached list of installed applications for main search and AppLauncherView
    apps: Vec<app_launcher::AppInfo>,
    /// P0 FIX: Cached clipboard entries for ClipboardHistoryView (avoids cloning per frame)
    cached_clipboard_entries: Vec<clipboard_history::ClipboardEntryMeta>,
    /// Sequential paste state machine (Raycast-style paste-one-at-a-time)
    #[allow(dead_code)]
    paste_sequential_state: Option<clipboard_history::PasteSequentialState>,
    /// Focused clipboard entry ID for action handling in ClipboardHistoryView
    #[allow(dead_code)]
    focused_clipboard_entry_id: Option<String>,
    /// P0 FIX: Cached windows for WindowSwitcherView (avoids cloning per frame)
    cached_windows: Vec<window_control::WindowInfo>,
    /// Cached browser tabs for BrowserTabsView (avoids repeated AppleScript calls while open)
    cached_browser_tabs: Vec<browser_tabs::BrowserTabInfo>,
    /// Cached browser history entries for BrowserHistoryView.
    cached_browser_history: Vec<browser_history::BrowserHistoryEntry>,
    /// Cached file results for FileSearchView (avoids cloning per frame)
    cached_file_results: Vec<file_search::FileResult>,
    /// Cached process list for ProcessManagerView (avoids cloning per frame)
    cached_processes: Vec<crate::process_manager::ProcessInfo>,
    /// Background refresh task for ProcessManagerView (dropped on view change)
    process_manager_refresh_task: Option<gpui::Task<()>>,
    /// Cached menu bar entries for CurrentAppCommandsView (populated on open)
    cached_current_app_entries: Vec<builtins::BuiltInEntry>,
    /// Session metadata for CurrentAppCommandsView, including the source app identity.
    current_app_commands_session:
        Option<crate::menu_bar::current_app_commands::CurrentAppCommandsSession>,
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
    /// Whether the focused-info panel is visible (toggled via Cmd+I / "Show Info" action)
    show_info_panel: bool,
    /// Single warm PTY reserved for the next launcher Quick Terminal open.
    quick_terminal_warm_pty: Option<crate::terminal::TerminalHandle>,
    /// True while the one-slot Quick Terminal warm PTY pool is being refilled.
    quick_terminal_warm_inflight: bool,
    /// Creation timestamp for TTL validation before consuming the warm PTY.
    quick_terminal_warm_created_at: Option<std::time::Instant>,
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
    /// Selected index most recently chosen by a selection-owned builtin wheel scroll.
    builtin_wheel_owned_selected_index: Option<usize>,
    // Interactive script state
    current_view: AppView,
    pub(crate) main_window_mode: MainWindowMode,
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
    // Default stdout-backed response channel for stdin/session RPCs when no script owns stdin.
    default_response_sender: Option<mpsc::SyncSender<Message>>,
    // List state for variable-height list (supports section headers at 24px + items at 48px)
    main_list_state: ListState,
    // Scroll handle for uniform_list (still used for backward compat in some views)
    list_scroll_handle: UniformListScrollHandle,
    // P0: Scroll handle for virtualized arg prompt choices
    arg_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for clipboard history list
    clipboard_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for emoji picker grid (uniform_list virtualized rows)
    emoji_scroll_handle: UniformListScrollHandle,
    // Frozen frequent-emoji snapshot for the currently open EmojiPickerView.
    // Rebuilt from ~/.kenv/emoji-usage.json when the picker opens; render +
    // navigation + Enter all consume the same Vec so selection indices stay
    // stable while the view is open. See Oracle-Session
    // `emoji-picker-frecency-recency` — "freeze ranking at view-open time".
    emoji_frequent_snapshot: Vec<String>,
    // Scroll handle for window switcher list
    window_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for browser tabs list
    browser_tabs_scroll_handle: UniformListScrollHandle,
    // Scroll handle for process manager list
    process_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for current app commands list
    current_app_commands_scroll_handle: UniformListScrollHandle,
    // Scroll handle for ACP history list
    acp_history_scroll_handle: ScrollHandle,
    // Scroll handle for browser history list
    browser_history_scroll_handle: ScrollHandle,
    // Scroll handle for dictation history list
    dictation_history_scroll_handle: ScrollHandle,
    // Scroll handle for notes browse portal list
    notes_browse_scroll_handle: ScrollHandle,
    // Scroll handle for design gallery list
    design_gallery_scroll_handle: UniformListScrollHandle,
    // Scroll handle for file search list
    file_search_scroll_handle: UniformListScrollHandle,
    // Variable-height list state for the theme chooser
    theme_chooser_list_state: ListState,
    // File search loading state (true while mdfind is running)
    file_search_loading: bool,
    // Debounce task for file search (cancelled when new input arrives)
    file_search_debounce_task: Option<gpui::Task<()>>,
    // Current directory being listed (for instant filter mode)
    file_search_current_dir: Option<String>,
    // Whether the current directory cache includes dot-prefixed entries
    file_search_current_dir_show_hidden: bool,
    // Frozen filter during directory transitions (prevents wrong results flash)
    // When Some, use this filter instead of deriving from query
    // Outer Option: None = use query filter, Some = use frozen filter
    // Inner Option: None = no filter, Some(s) = filter by s
    file_search_frozen_filter: Option<Option<String>>,
    // Path of the file selected for actions (for file search actions handling)
    file_search_actions_path: Option<String>,
    // Active sort mode for directory-browse file search views (session-local)
    file_search_sort_mode: crate::actions::FileSearchSortMode,
    // Generation counter for ignoring stale search results
    // Incremented on each new search, results with old gen are discarded
    file_search_gen: u64,
    // Cancel token for in-flight search (set to true to stop mdfind)
    file_search_cancel: Option<file_search::CancelToken>,
    // Pre-computed display indices after Nucleo filtering/sorting
    // This is computed once when results change or filter changes (not in render)
    // Vec of indices into cached_file_results, sorted by match quality
    file_search_display_indices: Vec<usize>,
    // Whether file-search selection should stay pinned to the first visible row
    // or follow the user's explicit selection across streamed updates.
    file_search_selection_mode: FileSearchSelectionMode,
    // Right-side preview panel thumbnail state for selected image files.
    file_search_preview_thumbnail: FileSearchThumbnailPreviewState,
    // Actions popup overlay
    show_actions_popup: bool,
    /// Timestamp of the last actions popup close, used to debounce reopen from
    /// activation-triggered close racing with the footer button click handler.
    actions_closed_at: Option<std::time::Instant>,
    // ActionsDialog entity for focus management
    actions_dialog: Option<Entity<ActionsDialog>>,
    // Cursor blink state and focus tracking
    cursor_visible: bool,
    /// Which input currently has focus (for cursor display)
    focused_input: FocusedInput,
    // Current script process PID for explicit cleanup (belt-and-suspenders)
    current_script_pid: Option<u32>,
    /// Main menu filtered and grouped result caches.
    main_menu_result_caches: MainMenuResultCacheState,
    // P3: Two-stage filter - display vs search separation with coalescing
    /// What the search cache is built from (may lag behind filter_text during rapid typing)
    computed_filter_text: String,
    /// Coalesces filter updates and keeps only the latest value per tick
    filter_coalescer: FilterCoalescer,
    /// Raw-guarded parse of the current filter text for power syntax
    /// (`:` advanced query, `;target` / `target:` capture, `;` hint rows).
    /// Parsed at input-change time in `handle_filter_input_change` and
    /// `set_filter_text_immediate` so result grouping and execution can consume
    /// a stable snapshot without racing the filter coalescer.
    menu_syntax_mode: crate::menu_syntax::MenuSyntaxMode,
    /// Cached state for the menu-syntax trigger popup. `filter_input_change` runs
    /// `plan_trigger_popup_transition` on every filter update and keeps this
    /// field in sync while the detached popup window renders from the snapshot
    /// plus selected row id.
    menu_syntax_trigger_popup_state: crate::menu_syntax_trigger_popup::MenuSyntaxTriggerPopupState,
    /// Run 12 Pass 11 — pending Cmd+Enter inline AI proposal for
    /// `cmd-enter-inline-ai-proposal`. Set by the Cmd+Enter handler when the
    /// user is composing power syntax; threaded into the snapshot so the hint
    /// card can render the proposal title + accept-label inline. Cleared on
    /// filter change or Esc/Tab dismissal. Pass 11 ships a deterministic stub
    /// proposal so the receipt is observable without an LLM round-trip; the
    /// real ACP/LLM call wiring is a follow-up.
    pub(crate) pending_menu_syntax_ai_proposal: Option<crate::menu_syntax_ai::MenuSyntaxAiProposal>,
    /// When `Some(filter)`, the menu-syntax trigger popup must NOT
    /// automatically re-open for that exact filter text. Set by the
    /// keyboard-apply dispatcher after an Accept (Enter) outcome so the
    /// user does not see the popup "flicker" back open immediately after
    /// they committed a target selection — e.g. typing `+`, pressing
    /// Enter on `Todo inbox` sets the filter to `;todo ` which would
    /// otherwise re-trigger `plan_trigger_popup_transition` → `Open`
    /// with the handler snapshot. The suppression is single-use: any
    /// filter change that produces a DIFFERENT raw text clears it so the
    /// popup can open again when the user keeps typing or deletes back
    /// to a partial trigger.
    menu_syntax_trigger_popup_suppressed_filter: Option<String>,
    // Scroll stabilization: track last scrolled-to index to avoid redundant scroll_to_item calls
    last_scrolled_index: Option<usize>,
    // Preview cache: avoid re-reading file and re-highlighting on every render
    preview_cache_path: Option<String>,
    preview_cache_match_signature: Option<(usize, usize, usize)>,
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
    /// Main-menu fallback commands for no-match script searches.
    main_menu_fallback_state: MainMenuFallbackState,
    // Theme before chooser was opened (for cancel/restore)
    theme_before_chooser: Option<std::sync::Arc<theme::Theme>>,
    /// Main script-list render diagnostics and input-to-render timing receipts.
    main_menu_render_diagnostics: MainMenuRenderDiagnosticsState,
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
    /// Most recent Tab AI execution waiting for success/failure accounting.
    pending_tab_ai_execution: Option<crate::ai::TabAiExecutionRecord>,
    /// Tab AI save-offer overlay state — when Some, prompts to persist the last successful ephemeral script.
    tab_ai_save_offer_state: Option<TabAiSaveOfferState>,
    /// Persistent harness terminal session — reused across Tab presses.
    pub(crate) tab_ai_harness: Option<crate::ai::TabAiHarnessSessionState>,
    /// Generation counter for cancelling stale deferred capture results.
    /// Incremented on each new `begin_tab_ai_harness_entry()` call.
    pub(crate) tab_ai_harness_capture_generation: u64,
    /// Previous surface to restore when leaving Tab AI quick terminal.
    pub(crate) tab_ai_harness_return_view: Option<AppView>,
    /// Previous focus target to restore when leaving Tab AI quick terminal.
    pub(crate) tab_ai_harness_return_focus_target: Option<FocusTarget>,
    /// Main-menu trigger that launched the current ACP session, if any.
    pub(crate) tab_ai_harness_script_list_trigger: Option<char>,
    /// Pending explicit apply-back route for the active Tab AI harness session.
    pub(crate) tab_ai_harness_apply_back_route: Option<crate::ai::TabAiApplyBackRoute>,
    /// Persistent embedded ACP chat entity so repeated Tab opens can reuse
    /// the same live ACP connection instead of cold-starting a new one.
    pub(crate) embedded_acp_chat: Option<Entity<crate::ai::acp::view::AcpChatView>>,
    /// Hidden, never-shown ACP chat entity warmed at startup. It is consumed
    /// by the first compatible ACP open so prompt submit avoids initialization.
    pub(crate) prewarmed_acp_chat: Option<Entity<crate::ai::acp::view::AcpChatView>>,
    /// Cached ready script path from the ACP `SCRIPT_READY` receipt. Updated
    /// whenever the ACP observer fires. Used by footer button resolution
    /// (which only has `&self`) without needing a `cx` to read the ACP entity.
    pub(crate) acp_ready_script_path: Option<std::path::PathBuf>,
    /// Cached ACP footer dot status so child ACP notifications only repaint
    /// the parent-owned native footer when the visible footer state changes.
    pub(crate) acp_footer_dot_status: Option<crate::footer_popup::FooterDotStatus>,
    /// Cached ACP model label paired with `acp_footer_dot_status`.
    pub(crate) acp_footer_model_display: Option<String>,
    /// Snapshot of shared launcher host state while an attachment portal owns
    /// the main window.
    pub(crate) attachment_portal_host_snapshot: Option<AttachmentPortalHostSnapshot>,
    /// Previous surface to restore when leaving an attachment portal (file search / clipboard).
    pub(crate) attachment_portal_return_view: Option<AppView>,
    /// Previous focus target to restore when leaving an attachment portal.
    pub(crate) attachment_portal_return_focus_target: Option<FocusTarget>,
    /// Window width to restore when leaving an attachment portal.
    pub(crate) attachment_portal_return_width: Option<f32>,
    /// Which attachment portal is currently active, when any.
    pub(crate) active_attachment_portal_kind:
        Option<crate::ai::window::context_picker::types::PortalKind>,
    /// App-owned placement machine for the ACP surface. Source of
    /// truth for the `blocks_launcher_ai_entry` and
    /// `is_attachment_portal` predicates. Written only through
    /// `transition_acp_surface`; do not mutate directly.
    pub(crate) acp_surface_state: crate::ai::acp::surface_state::AcpSurfaceState,
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
    /// When Some, the script list is filtered to only show scripts whose names
    /// match one of these IDs. Set by the Favorites builtin, cleared on reset.
    active_favorites: Option<Vec<String>>,
    /// Sender for inline chat escape signals
    /// The ChatPrompt escape callback uses this to signal when ESC is pressed
    inline_chat_escape_sender: mpsc::SyncSender<()>,
    /// Receiver for inline chat escape signals
    /// Checked by timer to trigger view reset
    inline_chat_escape_receiver: mpsc::Receiver<()>,
    /// Sender for inline chat continue signals
    /// The ChatPrompt continue callback uses this to signal "Continue in Harness Terminal"
    #[allow(dead_code)]
    inline_chat_continue_sender: mpsc::SyncSender<()>,
    /// Receiver for inline chat continue signals
    /// Checked by timer to hide main window when transferring to AI window
    inline_chat_continue_receiver: mpsc::Receiver<()>,
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
    /// Cached preflight receipt for the main-window Execution Contract rail.
    /// Rebuilt on selection/filter changes; consumed read-only in render().
    cached_main_window_preflight: Option<crate::main_window_preflight::MainWindowPreflightReceipt>,
    /// Cache key for preflight receipt (filter_text + selected_index + view).
    /// Cleared by `invalidate_main_window_preflight()`.
    main_window_preflight_cache_key: String,
    /// Window orchestrator — pure state machine for window visibility and focus.
    window_orchestrator: crate::window_orchestrator::OrchestratorState,
}

#[derive(Clone, Debug)]
struct AttachmentPortalHostSnapshot {
    filter_text: String,
    computed_filter_text: String,
    pending_filter_sync: bool,
    pending_placeholder: Option<String>,
    hovered_index: Option<usize>,
    selected_index: usize,
    opened_from_main_menu: bool,
    focused_input: FocusedInput,
    pending_focus: Option<FocusTarget>,
    width_before_portal: Option<f32>,
    width_after_portal_open: Option<f32>,
}

/// Result of alias matching - either a Script or Scriptlet
#[derive(Clone, Debug)]
enum AliasMatch {
    Script(Arc<scripts::Script>),
    Scriptlet(Arc<scripts::Scriptlet>),
    BuiltIn(Arc<builtins::BuiltInEntry>),
    App(Arc<app_launcher::AppInfo>),
}
