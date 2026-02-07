macro_rules! protocol_message_variants_ai {
    ($callback:ident, $($variants:tt)*) => {
        $callback! {
            $($variants)*
    // ============================================================
    // AI CHAT SDK API
    // ============================================================
    /// Check if AI window is open (SDK → App)
    #[serde(rename = "aiIsOpen")]
    AiIsOpen {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Response with AI window open status (App → SDK)
    #[serde(rename = "aiIsOpenResult")]
    AiIsOpenResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Whether the AI window is currently open
        #[serde(rename = "isOpen")]
        is_open: bool,
        /// Active chat ID if window is open
        #[serde(rename = "activeChatId", skip_serializing_if = "Option::is_none")]
        active_chat_id: Option<String>,
    },

    /// Get active chat metadata (SDK → App)
    #[serde(rename = "aiGetActiveChat")]
    AiGetActiveChat {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Response with active chat info (App → SDK)
    #[serde(rename = "aiActiveChatResult")]
    AiActiveChatResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Active chat info (null if no chat is active)
        #[serde(skip_serializing_if = "Option::is_none")]
        chat: Option<AiChatInfo>,
    },

    /// List all chats (SDK → App)
    #[serde(rename = "aiListChats")]
    AiListChats {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Max chats to return (default 50)
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<usize>,
        /// Include soft-deleted chats
        #[serde(rename = "includeDeleted", default)]
        include_deleted: bool,
    },

    /// Response with chat list (App → SDK)
    #[serde(rename = "aiChatListResult")]
    AiChatListResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// List of chats
        chats: Vec<AiChatInfo>,
        /// Total number of chats
        #[serde(rename = "totalCount")]
        total_count: usize,
    },

    /// Get messages from a conversation (SDK → App)
    #[serde(rename = "aiGetConversation")]
    AiGetConversation {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Chat ID (defaults to active chat if not provided)
        #[serde(rename = "chatId", skip_serializing_if = "Option::is_none")]
        chat_id: Option<String>,
        /// Max messages to return (default 100)
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<usize>,
    },

    /// Response with conversation messages (App → SDK)
    #[serde(rename = "aiConversationResult")]
    AiConversationResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Chat ID
        #[serde(rename = "chatId")]
        chat_id: String,
        /// Messages in the conversation
        messages: Vec<AiMessageInfo>,
        /// Whether there are more messages
        #[serde(rename = "hasMore")]
        has_more: bool,
    },

    /// Start a new AI conversation (SDK → App)
    #[serde(rename = "aiStartChat")]
    AiStartChat {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Initial user message
        message: String,
        /// Optional system prompt
        #[serde(rename = "systemPrompt", skip_serializing_if = "Option::is_none")]
        system_prompt: Option<String>,
        /// Optional base64-encoded image attachment
        #[serde(skip_serializing_if = "Option::is_none")]
        image: Option<String>,
        /// Optional model ID (e.g., "claude-3-5-sonnet-20241022")
        #[serde(rename = "modelId", skip_serializing_if = "Option::is_none")]
        model_id: Option<String>,
        /// If true, don't trigger AI response (just create chat with user message)
        #[serde(rename = "noResponse", default)]
        no_response: bool,
    },

    /// Response with created chat info (App → SDK)
    #[serde(rename = "aiChatCreated")]
    AiChatCreated {
        #[serde(rename = "requestId")]
        request_id: String,
        /// New chat ID
        #[serde(rename = "chatId")]
        chat_id: String,
        /// Chat title
        title: String,
        /// Model ID used
        #[serde(rename = "modelId")]
        model_id: String,
        /// Provider name
        provider: String,
        /// Whether AI response streaming started
        #[serde(rename = "streamingStarted")]
        streaming_started: bool,
    },

    /// Append a message without triggering AI response (SDK → App)
    #[serde(rename = "aiAppendMessage")]
    AiAppendMessage {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Chat ID
        #[serde(rename = "chatId")]
        chat_id: String,
        /// Message content
        content: String,
        /// Message role: "user", "assistant", or "system"
        role: String,
    },

    /// Response after appending message (App → SDK)
    #[serde(rename = "aiMessageAppended")]
    AiMessageAppended {
        #[serde(rename = "requestId")]
        request_id: String,
        /// New message ID
        #[serde(rename = "messageId")]
        message_id: String,
        /// Chat ID
        #[serde(rename = "chatId")]
        chat_id: String,
    },

    /// Send user message and trigger AI response (SDK → App)
    #[serde(rename = "aiSendMessage")]
    AiSendMessage {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Chat ID
        #[serde(rename = "chatId")]
        chat_id: String,
        /// Message content
        content: String,
        /// Optional base64-encoded image attachment
        #[serde(skip_serializing_if = "Option::is_none")]
        image: Option<String>,
    },

    /// Response after sending message (App → SDK)
    #[serde(rename = "aiMessageSent")]
    AiMessageSent {
        #[serde(rename = "requestId")]
        request_id: String,
        /// User message ID
        #[serde(rename = "userMessageId")]
        user_message_id: String,
        /// Chat ID
        #[serde(rename = "chatId")]
        chat_id: String,
        /// Whether AI response streaming started
        #[serde(rename = "streamingStarted")]
        streaming_started: bool,
    },

    /// Set/update system prompt for a chat (SDK → App)
    #[serde(rename = "aiSetSystemPrompt")]
    AiSetSystemPrompt {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Chat ID
        #[serde(rename = "chatId")]
        chat_id: String,
        /// System prompt content
        prompt: String,
    },

    /// Response after setting system prompt (App → SDK)
    #[serde(rename = "aiSystemPromptSet")]
    AiSystemPromptSet {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Whether the operation succeeded
        success: bool,
        /// Error message if failed
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    /// Focus the AI window (SDK → App)
    #[serde(rename = "aiFocus")]
    AiFocus {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Response after focusing AI window (App → SDK)
    #[serde(rename = "aiFocusResult")]
    AiFocusResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Whether the operation succeeded
        success: bool,
        /// Whether window was already open
        #[serde(rename = "wasOpen")]
        was_open: bool,
    },

    /// Get streaming status for AI window (SDK → App)
    #[serde(rename = "aiGetStreamingStatus")]
    AiGetStreamingStatus {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Optional chat ID (defaults to active chat)
        #[serde(rename = "chatId", skip_serializing_if = "Option::is_none")]
        chat_id: Option<String>,
    },

    /// Response with streaming status (App → SDK)
    #[serde(rename = "aiStreamingStatusResult")]
    AiStreamingStatusResult {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Whether AI is currently streaming
        #[serde(rename = "isStreaming")]
        is_streaming: bool,
        /// Chat ID being streamed to
        #[serde(rename = "chatId", skip_serializing_if = "Option::is_none")]
        chat_id: Option<String>,
        /// Accumulated content so far (if streaming)
        #[serde(rename = "partialContent", skip_serializing_if = "Option::is_none")]
        partial_content: Option<String>,
    },

    /// Delete a chat (SDK → App)
    #[serde(rename = "aiDeleteChat")]
    AiDeleteChat {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Chat ID to delete
        #[serde(rename = "chatId")]
        chat_id: String,
        /// If true, permanently delete (otherwise soft delete)
        #[serde(default)]
        permanent: bool,
    },

    /// Response after deleting chat (App → SDK)
    #[serde(rename = "aiChatDeleted")]
    AiChatDeleted {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Whether the operation succeeded
        success: bool,
        /// Error message if failed
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    /// Subscribe to AI events (SDK → App)
    #[serde(rename = "aiSubscribe")]
    AiSubscribe {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Event types to subscribe to: "message", "streamChunk", "streamComplete", "error"
        events: Vec<String>,
        /// Optional chat ID to filter events (None = all chats)
        #[serde(rename = "chatId", skip_serializing_if = "Option::is_none")]
        chat_id: Option<String>,
    },

    /// Subscription confirmation (App → SDK)
    #[serde(rename = "aiSubscribed")]
    AiSubscribed {
        #[serde(rename = "requestId")]
        request_id: String,
        /// Subscription ID for later unsubscribe
        #[serde(rename = "subscriptionId")]
        subscription_id: String,
        /// Confirmed event types
        events: Vec<String>,
    },

    /// Unsubscribe from AI events (SDK → App)
    #[serde(rename = "aiUnsubscribe")]
    AiUnsubscribe {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Unsubscription confirmation (App → SDK)
    #[serde(rename = "aiUnsubscribed")]
    AiUnsubscribed {
        #[serde(rename = "requestId")]
        request_id: String,
    },

    /// Streaming chunk event (pushed to subscribed scripts)
    #[serde(rename = "aiStreamChunk")]
    AiStreamChunk {
        /// Subscription ID
        #[serde(rename = "subscriptionId")]
        subscription_id: String,
        /// Chat ID
        #[serde(rename = "chatId")]
        chat_id: String,
        /// Delta chunk of text
        chunk: String,
        /// Accumulated content so far
        #[serde(rename = "accumulatedContent")]
        accumulated_content: String,
    },

    /// Stream complete event (pushed to subscribed scripts)
    #[serde(rename = "aiStreamComplete")]
    AiStreamComplete {
        /// Subscription ID
        #[serde(rename = "subscriptionId")]
        subscription_id: String,
        /// Chat ID
        #[serde(rename = "chatId")]
        chat_id: String,
        /// Assistant message ID
        #[serde(rename = "messageId")]
        message_id: String,
        /// Full response content
        #[serde(rename = "fullContent")]
        full_content: String,
        /// Tokens used (if available)
        #[serde(rename = "tokensUsed", skip_serializing_if = "Option::is_none")]
        tokens_used: Option<u32>,
    },

    /// New message event (pushed to subscribed scripts)
    #[serde(rename = "aiNewMessage")]
    AiNewMessage {
        /// Subscription ID
        #[serde(rename = "subscriptionId")]
        subscription_id: String,
        /// Chat ID
        #[serde(rename = "chatId")]
        chat_id: String,
        /// Message info
        message: AiMessageInfo,
    },

    /// AI error (for both request failures and subscription errors)
    #[serde(rename = "aiError")]
    AiError {
        /// Subscription ID (if this is a subscription error)
        #[serde(rename = "subscriptionId", skip_serializing_if = "Option::is_none")]
        subscription_id: Option<String>,
        /// Request ID (if this is a request error)
        #[serde(rename = "requestId", skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
        /// Error code (e.g., "INVALID_CHAT_ID", "NO_API_KEY")
        code: String,
        /// Human-readable error message
        message: String,
    },
        }
    };
}
