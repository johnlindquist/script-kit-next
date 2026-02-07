macro_rules! protocol_message_variants_prompts_media {
    ($callback:ident) => {
        $callback! {
    // ============================================================
    // PROTOCOL HANDSHAKE
    // ============================================================
    /// Protocol version negotiation request (SDK → App)
    ///
    /// Optional handshake message sent at session start.
    /// If not sent, app assumes legacy protocol with default capabilities.
    ///
    /// # Example
    /// ```json
    /// {"type":"hello","protocol":1,"sdkVersion":"1.0.0","capabilities":["submitJson","semanticIdV2"]}
    /// ```
    #[serde(rename = "hello")]
    Hello {
        /// Protocol version number (starts at 1)
        protocol: u32,
        /// SDK version string (e.g., "1.0.0")
        #[serde(rename = "sdkVersion")]
        sdk_version: String,
        /// List of capability flags the SDK supports
        #[serde(default)]
        capabilities: Vec<String>,
    },

    /// Protocol version negotiation response (App → SDK)
    ///
    /// Sent in response to Hello, confirms negotiated capabilities.
    ///
    /// # Example
    /// ```json
    /// {"type":"helloAck","protocol":1,"capabilities":["submitJson"]}
    /// ```
    #[serde(rename = "helloAck")]
    HelloAck {
        /// Protocol version number the app supports
        protocol: u32,
        /// List of capability flags the app confirms it supports
        #[serde(default)]
        capabilities: Vec<String>,
    },

    // ============================================================
    // CORE PROMPTS (existing)
    // ============================================================
    /// Script sends arg prompt with choices and optional actions
    #[serde(rename = "arg")]
    Arg {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        /// Optional actions for the actions panel (Cmd+K to open)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<ProtocolAction>>,
    },

    /// Script sends div (HTML display)
    #[serde(rename = "div")]
    Div {
        id: String,
        html: String,
        /// Tailwind classes for the content container
        #[serde(rename = "containerClasses", skip_serializing_if = "Option::is_none")]
        container_classes: Option<String>,
        /// Optional actions for the actions panel (Cmd+K to open)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<ProtocolAction>>,
        /// Placeholder text (shown in header)
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        /// Hint text
        #[serde(skip_serializing_if = "Option::is_none")]
        hint: Option<String>,
        /// Footer text
        #[serde(skip_serializing_if = "Option::is_none")]
        footer: Option<String>,
        /// Container background color: "transparent", "#RRGGBB", "#RRGGBBAA", or Tailwind color name
        #[serde(rename = "containerBg", skip_serializing_if = "Option::is_none")]
        container_bg: Option<String>,
        /// Container padding in pixels, or "none" to disable
        #[serde(rename = "containerPadding", skip_serializing_if = "Option::is_none")]
        container_padding: Option<serde_json::Value>,
        /// Container opacity (0-100)
        #[serde(skip_serializing_if = "Option::is_none")]
        opacity: Option<u8>,
    },

    /// App responds with submission (selected value or null)
    #[serde(rename = "submit")]
    Submit { id: String, value: Option<String> },

    /// App sends live update
    #[serde(rename = "update")]
    Update {
        id: String,
        #[serde(flatten)]
        data: serde_json::Value,
    },

    /// Signal termination
    #[serde(rename = "exit")]
    Exit {
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },

    /// Force submit the current prompt with a value (from SDK's submit() function)
    #[serde(rename = "forceSubmit")]
    ForceSubmit { value: serde_json::Value },

    /// Set the current prompt's input text
    #[serde(rename = "setInput")]
    SetInput { text: String },

    // ============================================================
    // TEXT INPUT PROMPTS
    // ============================================================
    /// Code/text editor with syntax highlighting
    #[serde(rename = "editor")]
    Editor {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
        /// VSCode-style snippet template with tabstops (e.g., "Hello ${1:name}!")
        #[serde(skip_serializing_if = "Option::is_none")]
        template: Option<String>,
        #[serde(rename = "onInit", skip_serializing_if = "Option::is_none")]
        on_init: Option<String>,
        #[serde(rename = "onSubmit", skip_serializing_if = "Option::is_none")]
        on_submit: Option<String>,
        /// Optional actions for the actions panel (Cmd+K to open)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<ProtocolAction>>,
    },

    /// Compact arg prompt (same as Arg but compact display)
    #[serde(rename = "mini")]
    Mini {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
    },

    /// Tiny arg prompt (same as Arg but tiny display)
    #[serde(rename = "micro")]
    Micro {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
    },

    // ============================================================
    // SELECTION PROMPTS
    // ============================================================
    /// Select from choices with optional multiple selection
    #[serde(rename = "select")]
    Select {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        #[serde(skip_serializing_if = "Option::is_none")]
        multiple: Option<bool>,
    },

    /// Confirmation dialog with yes/no choice
    ///
    /// Shows a modal dialog with a message and two buttons (confirm/cancel).
    /// Returns true if confirmed, false if cancelled.
    ///
    /// # Example
    /// ```json
    /// {"type":"confirm","id":"123","message":"Delete this file?","confirmText":"Delete","cancelText":"Keep"}
    /// ```
    #[serde(rename = "confirm")]
    Confirm {
        id: String,
        message: String,
        /// Text for the confirm button (default: "OK")
        #[serde(rename = "confirmText", skip_serializing_if = "Option::is_none")]
        confirm_text: Option<String>,
        /// Text for the cancel button (default: "Cancel")
        #[serde(rename = "cancelText", skip_serializing_if = "Option::is_none")]
        cancel_text: Option<String>,
    },

    // ============================================================
    // FORM PROMPTS
    // ============================================================
    /// Multiple input fields
    #[serde(rename = "fields")]
    Fields {
        id: String,
        fields: Vec<Field>,
        /// Optional actions for the actions panel (Cmd+K to open)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<ProtocolAction>>,
    },

    /// Custom HTML form
    #[serde(rename = "form")]
    Form {
        id: String,
        html: String,
        /// Optional actions for the actions panel (Cmd+K to open)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<ProtocolAction>>,
    },

    // ============================================================
    // FILE/PATH PROMPTS
    // ============================================================
    /// File/folder path picker
    #[serde(rename = "path")]
    Path {
        id: String,
        #[serde(rename = "startPath", skip_serializing_if = "Option::is_none")]
        start_path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        hint: Option<String>,
    },

    /// File drop zone
    #[serde(rename = "drop")]
    Drop { id: String },

    // ============================================================
    // INPUT CAPTURE PROMPTS
    // ============================================================
    /// Hotkey capture
    #[serde(rename = "hotkey")]
    Hotkey {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
    },

    // ============================================================
    // TEMPLATE/TEXT PROMPTS
    // ============================================================
    /// Template string with placeholders
    #[serde(rename = "template")]
    Template { id: String, template: String },

    /// Environment variable prompt
    #[serde(rename = "env")]
    Env {
        id: String,
        key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        secret: Option<bool>,
    },

    // ============================================================
    // MEDIA PROMPTS
    // ============================================================
    /// Chat interface with message history and streaming support
    ///
    /// Displays a Raycast-style chat interface where users can send messages
    /// and receive responses (potentially streamed). Supports markdown rendering.
    #[serde(rename = "chat")]
    Chat {
        id: String,
        /// Placeholder text for the input field
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        /// Initial messages to display
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        messages: Vec<ChatPromptMessage>,
        /// Hint text (shown in header)
        #[serde(skip_serializing_if = "Option::is_none")]
        hint: Option<String>,
        /// Footer text
        #[serde(skip_serializing_if = "Option::is_none")]
        footer: Option<String>,
        /// Optional actions for the actions panel (Cmd+K to open)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<ProtocolAction>>,
        /// Default model to use
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        /// Available models in actions menu
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        models: Vec<String>,
        /// Save conversation to database (default: true)
        #[serde(rename = "saveHistory", default)]
        save_history: bool,
        /// Use built-in AI mode (app handles AI calls instead of SDK)
        /// When true, the app will auto-stream AI responses using configured providers
        #[serde(rename = "useBuiltinAi", default)]
        use_builtin_ai: bool,
    },

    /// Add a message to the chat (SDK → App)
    ///
    /// Sent by the SDK to add a new message to the chat interface.
    /// Use for both user-generated messages and assistant responses.
    #[serde(rename = "chatMessage")]
    ChatMessage {
        /// Chat prompt ID this message belongs to
        id: String,
        /// The message to add
        message: ChatPromptMessage,
    },

    /// Start streaming a message (SDK → App)
    ///
    /// Creates a new streaming message in the chat. Subsequent
    /// chatStreamChunk messages will append to this message.
    #[serde(rename = "chatStreamStart")]
    ChatStreamStart {
        /// Chat prompt ID
        id: String,
        /// Unique message ID for this stream
        #[serde(rename = "messageId")]
        message_id: String,
        /// Position: "left" (assistant) or "right" (user)
        #[serde(default)]
        position: ChatMessagePosition,
    },

    /// Stream a chunk of text to an active message (SDK → App)
    ///
    /// Appends text to the currently streaming message.
    #[serde(rename = "chatStreamChunk")]
    ChatStreamChunk {
        /// Chat prompt ID
        id: String,
        /// Message ID being streamed to
        #[serde(rename = "messageId")]
        message_id: String,
        /// Text chunk to append
        chunk: String,
    },

    /// Complete streaming for a message (SDK → App)
    ///
    /// Marks the streaming message as complete.
    #[serde(rename = "chatStreamComplete")]
    ChatStreamComplete {
        /// Chat prompt ID
        id: String,
        /// Message ID that completed
        #[serde(rename = "messageId")]
        message_id: String,
    },

    /// Clear all messages in the chat (SDK → App)
    #[serde(rename = "chatClear")]
    ChatClear {
        /// Chat prompt ID to clear
        id: String,
    },

    /// Set error on a message (SDK → App)
    #[serde(rename = "chatSetError")]
    ChatSetError {
        /// Chat prompt ID
        id: String,
        /// Message ID to set error on
        #[serde(rename = "messageId")]
        message_id: String,
        /// Error message
        error: String,
    },

    /// Clear error from a message (SDK → App)
    #[serde(rename = "chatClearError")]
    ChatClearError {
        /// Chat prompt ID
        id: String,
        /// Message ID to clear error from
        #[serde(rename = "messageId")]
        message_id: String,
    },

    /// User submitted a message in chat (App → SDK)
    ///
    /// Sent when the user presses Enter in the chat input.
    /// The SDK should handle this and potentially respond with
    /// chatMessage or chatStreamStart/chatStreamChunk/chatStreamComplete.
    #[serde(rename = "chatSubmit")]
    ChatSubmit {
        /// Chat prompt ID
        id: String,
        /// The text the user submitted
        text: String,
    },

    /// Terminal emulator
    #[serde(rename = "term")]
    Term {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        command: Option<String>,
        /// Optional actions for the actions panel (Cmd+K to open)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<Vec<ProtocolAction>>,
    },

    /// Custom widget with HTML
    #[serde(rename = "widget")]
    Widget {
        id: String,
        html: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        options: Option<serde_json::Value>,
    },

    /// Webcam capture
    #[serde(rename = "webcam")]
    Webcam { id: String },

    /// Microphone recording
    #[serde(rename = "mic")]
    Mic { id: String },

    // ============================================================
    // NOTIFICATION/FEEDBACK MESSAGES
    // ============================================================
    /// System notification
    #[serde(rename = "notify")]
    Notify {
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<String>,
    },

    /// System beep sound
    #[serde(rename = "beep")]
    Beep {},

    /// Text-to-speech
    #[serde(rename = "say")]
    Say {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        voice: Option<String>,
    },

    /// Status bar update
    #[serde(rename = "setStatus")]
    SetStatus {
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },

    /// HUD (heads-up display) overlay message
    #[serde(rename = "hud")]
    Hud {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
    },

        }
    };
}
