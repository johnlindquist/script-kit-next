macro_rules! protocol_message_variants_query_ops {
    ($callback:ident, $($variants:tt)*) => {
        $callback! {
            $($variants)*
    // ============================================================
    // FILE SEARCH
    // ============================================================
    /// Request file search
    #[serde(rename = "fileSearch")]
    FileSearch {
        #[serde(rename = "requestId")]
        request_id: String,
        query: String,
        #[serde(rename = "onlyin", skip_serializing_if = "Option::is_none")]
        only_in: Option<String>,
    },

    /// Response with file search results
    #[serde(rename = "fileSearchResult")]
    FileSearchResult {
        #[serde(rename = "requestId")]
        request_id: String,
        files: Vec<FileSearchResultEntry>,
    },

    // ============================================================
    // SCREENSHOT CAPTURE
    // ============================================================
    /// Request to capture app window screenshot
    #[serde(rename = "captureScreenshot")]
    CaptureScreenshot {
        #[serde(rename = "requestId")]
        request_id: String,
        /// If true, return full retina resolution (2x). If false (default), scale down to 1x.
        #[serde(rename = "hiDpi", skip_serializing_if = "Option::is_none")]
        hi_dpi: Option<bool>,
    },

    /// Response with screenshot data as base64 PNG
    #[serde(rename = "screenshotResult")]
    ScreenshotResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Base64-encoded PNG data
        data: String,
        width: u32,
        height: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    // ============================================================
    // STATE QUERY
    // ============================================================
    /// Request current UI state without modifying it
    #[serde(rename = "getState")]
    GetState {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Response with current UI state
    #[serde(rename = "stateResult")]
    StateResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Current prompt type
        #[serde(rename = "promptType")]
        prompt_type: String,
        /// Prompt ID if active
        #[serde(rename = "promptId", skip_serializing_if = "Option::is_none")]
        prompt_id: Option<String>,
        /// Placeholder text if applicable
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        /// Current input/filter value
        #[serde(rename = "inputValue")]
        input_value: String,
        /// Total number of choices
        #[serde(rename = "choiceCount")]
        choice_count: usize,
        /// Number of visible/filtered choices
        #[serde(rename = "visibleChoiceCount")]
        visible_choice_count: usize,
        /// Currently selected index (-1 if none)
        #[serde(rename = "selectedIndex")]
        selected_index: i32,
        /// Value of the selected choice
        #[serde(rename = "selectedValue", skip_serializing_if = "Option::is_none")]
        selected_value: Option<String>,
        /// Whether the window has focus
        #[serde(rename = "isFocused")]
        is_focused: bool,
        /// Whether the window is visible
        #[serde(rename = "windowVisible")]
        window_visible: bool,
    },

    // ============================================================
    // ELEMENT QUERY (AI-driven UX)
    // ============================================================
    /// Request visible UI elements with semantic IDs
    #[serde(rename = "getElements")]
    GetElements {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Maximum number of elements to return (default: 50)
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<usize>,
    },

    /// Response with list of visible UI elements
    #[serde(rename = "elementsResult")]
    ElementsResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// List of visible UI elements
        elements: Vec<ElementInfo>,
        /// Total number of elements (may be larger than returned if limit applied)
        #[serde(rename = "totalCount")]
        total_count: usize,
        /// True when limit caused the response to omit elements
        #[serde(default)]
        truncated: bool,
        /// Semantic ID of the currently focused element, if any
        #[serde(rename = "focusedSemanticId", default, skip_serializing_if = "Option::is_none")]
        focused_semantic_id: Option<String>,
        /// Semantic ID of the currently selected element, if any
        #[serde(rename = "selectedSemanticId", default, skip_serializing_if = "Option::is_none")]
        selected_semantic_id: Option<String>,
        /// Machine-readable collection warnings
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        warnings: Vec<String>,
    },

    // ============================================================
    // LAYOUT INFO (AI Agent Debugging)
    // ============================================================
    /// Request layout information with component tree and computed styles
    ///
    /// Returns detailed information about every component's position,
    /// size, padding, margin, gap, and flex properties. Designed to
    /// help AI agents understand "why" components are positioned/sized.
    #[serde(rename = "getLayoutInfo")]
    GetLayoutInfo {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Response with full layout information
    #[serde(rename = "layoutInfoResult")]
    LayoutInfoResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Full layout information including component tree
        #[serde(flatten)]
        info: LayoutInfo,
    },

    // ============================================================
    // ERROR REPORTING
    // ============================================================
    /// Script error with structured error information
    #[serde(rename = "setError")]
    SetError {
        /// User-friendly error message
        #[serde(rename = "errorMessage")]
        error_message: String,
        /// Raw stderr output if available
        #[serde(rename = "stderrOutput", skip_serializing_if = "Option::is_none")]
        stderr_output: Option<String>,
        /// Process exit code if available
        #[serde(rename = "exitCode", skip_serializing_if = "Option::is_none")]
        exit_code: Option<i32>,
        /// Parsed stack trace if available
        #[serde(rename = "stackTrace", skip_serializing_if = "Option::is_none")]
        stack_trace: Option<String>,
        /// Path to the script that failed
        #[serde(rename = "scriptPath")]
        script_path: String,
        /// Actionable fix suggestions
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        suggestions: Vec<String>,
        /// When the error occurred (ISO 8601 format)
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<String>,
    },

    // ============================================================
    // SCRIPTLET OPERATIONS
    // ============================================================
    /// Run a scriptlet with variable substitution
    #[serde(rename = "runScriptlet")]
    RunScriptlet {
        #[serde(rename = "requestId")]
        request_id: String,
        /// The scriptlet data to execute
        scriptlet: ScriptletData,
        /// Named input values for {{variable}} substitution
        #[serde(default, skip_serializing_if = "Option::is_none")]
        inputs: Option<serde_json::Value>,
        /// Positional arguments for $1, $2, etc.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
    },

    /// Request list of available scriptlets
    #[serde(rename = "getScriptlets")]
    GetScriptlets {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Optional kit to filter by
        #[serde(skip_serializing_if = "Option::is_none")]
        kit: Option<String>,
        /// Optional group to filter by
        #[serde(skip_serializing_if = "Option::is_none")]
        group: Option<String>,
    },

    /// Response with list of scriptlets
    #[serde(rename = "scriptletList")]
    ScriptletList {
        #[serde(rename = "requestId")]
        request_id: String,
        /// List of scriptlets
        scriptlets: Vec<ScriptletData>,
    },

    /// Result of scriptlet execution
    #[serde(rename = "scriptletResult")]
    ScriptletResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Whether execution succeeded
        success: bool,
        /// Output from the scriptlet (stdout)
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<String>,
        /// Error message if failed
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        /// Exit code if available
        #[serde(rename = "exitCode", skip_serializing_if = "Option::is_none")]
        exit_code: Option<i32>,
    },

    // ============================================================
    // TEST INFRASTRUCTURE
    // ============================================================
    /// Simulate a mouse click at specific coordinates (for testing)
    ///
    /// This message is used by test infrastructure to simulate mouse clicks
    /// at specified window-relative coordinates. It enables automated visual
    /// testing of click behaviors without requiring actual user interaction.
    #[serde(rename = "simulateClick")]
    SimulateClick {
        #[serde(rename = "requestId")]
        request_id: String,
        /// X coordinate relative to the window
        x: f64,
        /// Y coordinate relative to the window
        y: f64,
        /// Optional button: "left" (default), "right", or "middle"
        #[serde(skip_serializing_if = "Option::is_none")]
        button: Option<String>,
    },

    /// Response after simulating a click
    #[serde(rename = "simulateClickResult")]
    SimulateClickResult {
        #[serde(rename = "requestId")]
        request_id: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    // ============================================================
    // DEBUG/VISUAL TESTING
    // ============================================================
    /// Show the debug grid overlay with options
    ///
    /// Displays a grid overlay for visual debugging and layout verification.
    /// The grid shows alignment lines, component bounds, and box model visualization.
    #[serde(rename = "showGrid")]
    ShowGrid {
        /// Grid configuration options (flattened into the message)
        #[serde(flatten)]
        options: GridOptions,
    },

    /// Hide the debug grid overlay
    #[serde(rename = "hideGrid")]
    HideGrid,

    // ============================================================
    // ACTIONS API
    // ============================================================
    /// Set actions to display in the ActionsDialog (incoming from SDK)
    ///
    /// Scripts define actions with optional onAction handlers. The `has_action`
    /// field on each action determines routing:
    /// - `has_action=true`: Send ActionTriggered back to SDK
    /// - `has_action=false`: Submit value directly
    #[serde(rename = "setActions")]
    SetActions {
        /// List of actions to display
        actions: Vec<ProtocolAction>,
    },

    /// Notify SDK that an action was triggered (outgoing to SDK)
    ///
    /// Sent when an action with `has_action=true` is triggered.
    /// The SDK's onAction handler will receive this.
    #[serde(rename = "actionTriggered")]
    ActionTriggered {
        /// Name of the triggered action
        action: String,
        /// Value associated with the action (if any)
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
        /// Current input/filter text at time of trigger
        input: String,
    },

    // ============================================================
    // MENU BAR INTEGRATION
    // ============================================================
    /// Request menu bar items from the frontmost app or a specific app
    ///
    /// SDK sends this to get the menu bar hierarchy from an application.
    /// If bundle_id is None, uses the frontmost application.
    #[serde(rename = "getMenuBar")]
    GetMenuBar {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Optional bundle ID to get menu bar from a specific app
        #[serde(rename = "bundleId", skip_serializing_if = "Option::is_none")]
        bundle_id: Option<String>,
    },

    /// Response with menu bar items
    ///
    /// App sends this back to SDK with the menu bar hierarchy.
    #[serde(rename = "menuBarResult")]
    MenuBarResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// The menu bar items (hierarchical)
        items: Vec<super::types::MenuBarItemData>,
    },

    /// Execute a menu action by path
    ///
    /// SDK sends this to click a menu item in a specific application.
    #[serde(rename = "executeMenuAction")]
    ExecuteMenuAction {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Bundle ID of the target application
        #[serde(rename = "bundleId")]
        bundle_id: String,
        /// Path of menu titles to the target item (e.g., ["File", "New", "Window"])
        path: Vec<String>,
    },

    /// Result of a menu action execution
    ///
    /// App sends this back to SDK after attempting to execute a menu action.
    #[serde(rename = "menuActionResult")]
    MenuActionResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Whether the action succeeded
        success: bool,
        /// Error message if failed
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    // ============================================================
    // ACP STATE QUERY
    // ============================================================
    /// Request machine-readable ACP chat view state for agentic testing.
    ///
    /// Returns input text, cursor position, picker state, accepted item
    /// metadata, thread status, and layout stability metrics.
    #[serde(rename = "getAcpState")]
    GetAcpState {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Response with ACP chat view state snapshot.
    #[serde(rename = "acpStateResult")]
    AcpStateResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Machine-readable ACP state snapshot.
        #[serde(flatten)]
        state: AcpStateSnapshot,
    },

    // ============================================================
    // WAIT / BATCH — Deterministic Transaction Layer
    // ============================================================
    /// Poll until a UI condition is satisfied or timeout expires.
    ///
    /// The app will check the condition at `pollInterval` (default 25 ms)
    /// and reply with `waitForResult` when satisfied or after `timeout` ms.
    #[serde(rename = "waitFor")]
    WaitFor {
        #[serde(rename = "requestId")]
        request_id: String,
        /// The condition to wait for
        condition: WaitCondition,
        /// Timeout in milliseconds (default: 5000)
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<u64>,
        /// Poll interval in milliseconds (default: 25)
        #[serde(rename = "pollInterval", skip_serializing_if = "Option::is_none")]
        poll_interval: Option<u64>,
        /// Trace mode: off (default), on, or onFailure
        #[serde(default, skip_serializing_if = "super::types::batch_wait::is_trace_off")]
        trace: TransactionTraceMode,
    },

    /// Result of a waitFor request.
    #[serde(rename = "waitForResult")]
    WaitForResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Whether the condition was satisfied before timeout
        success: bool,
        /// Wall-clock time elapsed in milliseconds
        elapsed: u64,
        /// Structured error if the wait failed (e.g., timeout)
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<TransactionError>,
        /// Embedded trace receipt (present when trace mode is on or onFailure+failed)
        #[serde(skip_serializing_if = "Option::is_none")]
        trace: Option<TransactionTrace>,
    },

    /// Execute a sequence of atomic UI commands as a transaction.
    ///
    /// Commands run in order. If `options.stop_on_error` is true (default),
    /// execution halts on the first failure and `failed_at` is set.
    #[serde(rename = "batch")]
    Batch {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Ordered list of commands to execute
        commands: Vec<BatchCommand>,
        /// Execution options (stop-on-error, timeout)
        #[serde(skip_serializing_if = "Option::is_none")]
        options: Option<BatchOptions>,
        /// Trace mode: off (default), on, or onFailure
        #[serde(default, skip_serializing_if = "super::types::batch_wait::is_trace_off")]
        trace: TransactionTraceMode,
    },

    /// Result of a batch execution.
    #[serde(rename = "batchResult")]
    BatchResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// True if all commands succeeded
        success: bool,
        /// Per-command results
        results: Vec<BatchResultEntry>,
        /// Index of the first failed command, if any
        #[serde(rename = "failedAt", skip_serializing_if = "Option::is_none")]
        failed_at: Option<usize>,
        /// Total wall-clock time for the entire batch, in milliseconds
        #[serde(rename = "totalElapsed")]
        total_elapsed: u64,
        /// Embedded trace receipt (present when trace mode is on or onFailure+failed)
        #[serde(skip_serializing_if = "Option::is_none")]
        trace: Option<TransactionTrace>,
    },

        }
    };
}
