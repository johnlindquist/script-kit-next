/// Messages sent from the prompt poller back to the main app
#[derive(Debug, Clone)]
enum PromptMessage {
    ShowArg {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        actions: Option<Vec<ProtocolAction>>,
    },
    ShowDiv {
        id: String,
        html: String,
        /// Tailwind classes for the content container
        container_classes: Option<String>,
        actions: Option<Vec<ProtocolAction>>,
        /// Placeholder text (header)
        placeholder: Option<String>,
        /// Hint text
        hint: Option<String>,
        /// Footer text
        footer: Option<String>,
        /// Container background color
        container_bg: Option<String>,
        /// Container padding (number or "none")
        container_padding: Option<serde_json::Value>,
        /// Container opacity (0-100)
        opacity: Option<u8>,
    },
    ShowForm {
        id: String,
        html: String,
        actions: Option<Vec<ProtocolAction>>,
    },
    ShowTerm {
        id: String,
        command: Option<String>,
        actions: Option<Vec<ProtocolAction>>,
    },
    ShowEditor {
        id: String,
        content: Option<String>,
        language: Option<String>,
        template: Option<String>,
        actions: Option<Vec<ProtocolAction>>,
    },
    /// Path picker prompt for file/folder selection
    ShowPath {
        id: String,
        start_path: Option<String>,
        hint: Option<String>,
    },
    /// Environment variable prompt with optional secret handling
    ShowEnv {
        id: String,
        key: String,
        prompt: Option<String>,
        secret: bool,
    },
    /// Drag and drop prompt for file uploads
    ShowDrop {
        id: String,
        placeholder: Option<String>,
        hint: Option<String>,
    },
    /// Template prompt for tab-through string templates
    ShowTemplate {
        id: String,
        template: String,
    },
    /// Multi-select prompt from choices
    ShowSelect {
        id: String,
        placeholder: Option<String>,
        choices: Vec<Choice>,
        multiple: bool,
    },
    /// Confirmation dialog with yes/no choice
    ShowConfirm {
        id: String,
        message: String,
        confirm_text: Option<String>,
        cancel_text: Option<String>,
    },
    /// Chat prompt for conversational interfaces (Raycast-style)
    ShowChat {
        id: String,
        placeholder: Option<String>,
        messages: Vec<protocol::ChatPromptMessage>,
        hint: Option<String>,
        footer: Option<String>,
        actions: Option<Vec<ProtocolAction>>,
        model: Option<String>,
        models: Vec<String>,
        save_history: bool,
        use_builtin_ai: bool,
    },
    /// Add a message to an active chat prompt
    ChatAddMessage {
        id: String,
        message: protocol::ChatPromptMessage,
    },
    /// Start streaming a message in chat
    ChatStreamStart {
        id: String,
        message_id: String,
        position: protocol::ChatMessagePosition,
    },
    /// Append chunk to streaming message
    ChatStreamChunk {
        id: String,
        message_id: String,
        chunk: String,
    },
    /// Complete streaming for a message
    ChatStreamComplete {
        id: String,
        message_id: String,
    },
    /// Clear all messages in chat
    ChatClear {
        id: String,
    },
    /// Set error on a message
    ChatSetError {
        id: String,
        message_id: String,
        error: String,
    },
    /// Clear error from a message
    ChatClearError {
        id: String,
        message_id: String,
    },
    /// Open AI window and start a new chat with a message
    AiStartChat {
        request_id: String,
        message: String,
        system_prompt: Option<String>,
        image: Option<String>,
        model_id: Option<String>,
        no_response: bool,
    },
    /// Focus the AI window (opens if not already open)
    AiFocus {
        request_id: String,
    },
    HideWindow,
    OpenBrowser {
        url: String,
    },
    ScriptExit,
    /// External command to run a script by path
    RunScript {
        path: String,
    },
    /// Script error with detailed information for toast display
    ScriptError {
        error_message: String,
        stderr_output: Option<String>,
        exit_code: Option<i32>,
        stack_trace: Option<String>,
        script_path: String,
        suggestions: Vec<String>,
    },
    /// Protocol parsing error reported from script stdout
    ProtocolError {
        correlation_id: String,
        summary: String,
        details: Option<String>,
        severity: ErrorSeverity,
        script_path: String,
    },
    /// Unhandled message type from script - shows warning toast
    UnhandledMessage {
        message_type: String,
    },
    /// Request to get current UI state - triggers StateResult response
    GetState {
        request_id: String,
    },
    /// Request to get layout info with component tree and computed styles
    GetLayoutInfo {
        request_id: String,
    },
    /// Force submit the current prompt with a value (from SDK's submit() function)
    ForceSubmit {
        value: serde_json::Value,
    },
    /// Set the current prompt input text
    SetInput {
        text: String,
    },
    /// Show HUD overlay message
    ShowHud {
        text: String,
        duration_ms: Option<u64>,
    },
    /// Set SDK actions for the ActionsDialog
    SetActions {
        actions: Vec<protocol::ProtocolAction>,
    },
    /// Show the debug grid overlay
    ShowGrid {
        options: protocol::GridOptions,
    },
    /// Hide the debug grid overlay
    HideGrid,
}
