use super::*;
/// Available AI models for the chat
#[derive(Clone, Debug, PartialEq)]
pub struct ChatModel {
    pub id: String,
    pub name: String,
    pub provider: String,
}

impl ChatModel {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            provider: provider.into(),
        }
    }
}

/// Default models available in the chat
/// NOTE: First model in list is the default
pub fn default_models() -> Vec<ChatModel> {
    vec![
        // Default: Claude Haiku 4.5 (fast, good quality)
        ChatModel::new("claude-haiku-4-5-20250514", "Claude Haiku 4.5", "Anthropic"),
        ChatModel::new("claude-3-5-haiku-20241022", "Claude 3.5 Haiku", "Anthropic"),
        ChatModel::new(
            "claude-3-5-sonnet-20241022",
            "Claude 3.5 Sonnet",
            "Anthropic",
        ),
        ChatModel::new("gpt-4o-mini", "GPT-4o mini", "OpenAI"),
        ChatModel::new("gpt-4o", "GPT-4o", "OpenAI"),
    ]
}

/// Callback type for when user submits a message: (prompt_id, message_text)
pub type ChatSubmitCallback = Arc<dyn Fn(String, String) + Send + Sync>;

/// Callback type for when user presses Escape: (prompt_id)
pub type ChatEscapeCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Callback type for "Continue in Chat": (prompt_id)
pub type ChatContinueCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Callback type for retry: (prompt_id, message_id)
pub type ChatRetryCallback = Arc<dyn Fn(String, String) + Send + Sync>;

/// Callback type for "Configure API" action: () -> triggers API key setup
pub type ChatConfigureCallback = Arc<dyn Fn() + Send + Sync>;

/// Callback type for "Connect to Claude Code" action: () -> enables Claude Code in config
pub type ChatClaudeCodeCallback = Arc<dyn Fn() + Send + Sync>;

/// Callback type for showing actions menu: (prompt_id) -> triggers ActionsDialog
pub type ChatShowActionsCallback = Arc<dyn Fn(String) + Send + Sync>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SetupCardAction {
    None,
    ActivateConfigure,
    ActivateClaudeCode,
    Escape,
}

pub(crate) fn resolve_setup_card_key(
    key: &str,
    shift: bool,
    current_index: usize,
) -> (usize, SetupCardAction, bool) {
    let current_index = current_index % 2;

    if key.eq_ignore_ascii_case("tab") {
        let next_index = if shift {
            if current_index == 0 {
                1
            } else {
                current_index - 1
            }
        } else {
            (current_index + 1) % 2
        };
        return (next_index, SetupCardAction::None, true);
    }

    if key.eq_ignore_ascii_case("up") || key.eq_ignore_ascii_case("arrowup") {
        let next_index = if current_index == 0 {
            1
        } else {
            current_index - 1
        };
        return (next_index, SetupCardAction::None, true);
    }

    if key.eq_ignore_ascii_case("down") || key.eq_ignore_ascii_case("arrowdown") {
        let next_index = (current_index + 1) % 2;
        return (next_index, SetupCardAction::None, true);
    }

    if key.eq_ignore_ascii_case("enter") || key.eq_ignore_ascii_case("return") || key == " " {
        let action = if current_index == 0 {
            SetupCardAction::ActivateConfigure
        } else {
            SetupCardAction::ActivateClaudeCode
        };
        return (current_index, action, false);
    }

    if key.eq_ignore_ascii_case("escape") {
        return (current_index, SetupCardAction::Escape, false);
    }

    (current_index, SetupCardAction::None, false)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChatInputKeyAction {
    Escape,
    StopStreaming,
    ToggleActions,
    ContinueInChat,
    Submit,
    InsertNewline,
    CopyLastResponse,
    ClearConversation,
    Paste,
    DelegateToInput,
    Ignore,
}

pub(crate) fn resolve_chat_input_key_action(
    key: &str,
    cmd_pressed: bool,
    shift_pressed: bool,
) -> ChatInputKeyAction {
    let key_lower = key.to_ascii_lowercase();

    match key_lower.as_str() {
        "escape" | "esc" => ChatInputKeyAction::Escape,
        "." if cmd_pressed => ChatInputKeyAction::StopStreaming,
        "k" if cmd_pressed => ChatInputKeyAction::ToggleActions,
        "enter" | "return" if cmd_pressed => ChatInputKeyAction::ContinueInChat,
        "enter" | "return" if shift_pressed => ChatInputKeyAction::InsertNewline,
        "enter" | "return" => ChatInputKeyAction::Submit,
        "c" if cmd_pressed => ChatInputKeyAction::CopyLastResponse,
        "backspace" if cmd_pressed => ChatInputKeyAction::ClearConversation,
        "v" if cmd_pressed => ChatInputKeyAction::Paste,
        _ if cmd_pressed => ChatInputKeyAction::Ignore,
        _ => ChatInputKeyAction::DelegateToInput,
    }
}

pub(super) fn should_ignore_stream_reveal_update(
    active_stream_message_id: Option<&str>,
    streaming_message_id: &str,
) -> bool {
    active_stream_message_id != Some(streaming_message_id)
}

const CHAT_SCROLL_BOTTOM_REJOIN_BUFFER_ITEMS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChatScrollDirection {
    Up,
    Down,
    None,
}

pub(crate) fn next_chat_scroll_follow_state(
    user_has_scrolled_up: bool,
    direction: ChatScrollDirection,
    scroll_top_item_ix: usize,
    total_items: usize,
) -> bool {
    match direction {
        // Upward intent means "stop following streaming output".
        ChatScrollDirection::Up => true,
        // For multi-row transcripts, allow scroll-down near the end to rejoin auto-follow.
        // Single-row transcripts can be a single giant markdown item; item index alone is
        // not enough to infer bottom there, so keep manual mode until explicit rejoin.
        ChatScrollDirection::Down
            if user_has_scrolled_up
                && total_items > 1
                && scroll_top_item_ix.saturating_add(CHAT_SCROLL_BOTTOM_REJOIN_BUFFER_ITEMS)
                    >= total_items =>
        {
            false
        }
        ChatScrollDirection::Down | ChatScrollDirection::None => user_has_scrolled_up,
    }
}

/// A conversation turn: user prompt + optional AI response
#[derive(Clone, Debug)]
pub struct ConversationTurn {
    pub user_prompt: String,
    pub assistant_response: Option<String>,
    pub model: Option<String>,
    pub streaming: bool,
    pub error: Option<String>,
    pub message_id: Option<String>,
    pub user_image: Option<Arc<RenderImage>>,
}

/// Conversation starter suggestion
#[derive(Clone, Debug)]
pub struct ConversationStarter {
    pub id: String,
    pub label: String,
    pub prompt: String,
}

impl ConversationStarter {
    pub fn new(id: impl Into<String>, label: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            prompt: prompt.into(),
        }
    }
}

/// Default conversation starters
pub(super) fn default_conversation_starters() -> Vec<ConversationStarter> {
    vec![
        ConversationStarter::new("explain", "Explain this code", "Explain this code: "),
        ConversationStarter::new("debug", "Debug an error", "Help me debug this error: "),
        ConversationStarter::new("tests", "Write tests", "Write tests for: "),
        ConversationStarter::new("improve", "Improve code", "Improve this code: "),
    ]
}

pub(super) fn build_conversation_turns(
    messages: &[ChatPromptMessage],
    image_render_cache: &HashMap<String, Arc<RenderImage>>,
) -> Vec<ConversationTurn> {
    let mut turns = Vec::new();
    let mut i = 0;

    while i < messages.len() {
        let msg = &messages[i];

        if msg.is_user() {
            // Start a new turn with this user message
            let user_prompt = msg.get_content().to_string();
            let user_image = msg
                .id
                .as_ref()
                .and_then(|id| image_render_cache.get(id).cloned());
            let mut turn = ConversationTurn {
                user_prompt,
                assistant_response: None,
                model: None,
                streaming: false,
                error: None,
                message_id: msg.id.clone(),
                user_image,
            };

            // Look for the next assistant response
            if i + 1 < messages.len() {
                let next_msg = &messages[i + 1];
                if !next_msg.is_user() {
                    turn.assistant_response = Some(next_msg.get_content().to_string());
                    turn.model = next_msg.model.clone();
                    turn.streaming = next_msg.streaming;
                    turn.error = next_msg.error.clone();
                    turn.message_id = next_msg.id.clone().or(turn.message_id);
                    i += 1;
                }
            }

            turns.push(turn);
        } else {
            // Standalone assistant message (no user prompt before it)
            // This happens for system-initiated messages
            let turn = ConversationTurn {
                user_prompt: String::new(),
                assistant_response: Some(msg.get_content().to_string()),
                model: msg.model.clone(),
                streaming: msg.streaming,
                error: msg.error.clone(),
                message_id: msg.id.clone(),
                user_image: None,
            };
            turns.push(turn);
        }

        i += 1;
    }

    turns
}

/// Find the next reveal boundary after `offset` in `text`.
///
/// Reveals through the next newline so markdown structural elements (list markers,
/// headings) are always delivered as complete lines. Within a long line that has no
/// newline yet, falls back to word boundaries for smooth character-level pacing.
///
/// Returns `None` when only a partial token remains (no whitespace yet), signalling
/// the reveal loop to wait for more data. All returned offsets land on UTF-8
/// character boundaries.
pub(super) fn next_reveal_boundary(text: &str, offset: usize) -> Option<usize> {
    let remaining = &text[offset..];
    if remaining.is_empty() {
        return None;
    }

    // Strategy: reveal through the next newline (keeps markdown lines intact).
    // If no newline is found, fall back to next word boundary within the line.
    if let Some(nl_pos) = remaining.find('\n') {
        // Include the newline itself
        return Some(offset + nl_pos + 1);
    }

    // No newline — reveal next word within the current (incomplete) line.
    let mut found_non_ws = false;
    let mut word_end: Option<usize> = None;

    for (i, c) in remaining.char_indices() {
        if c.is_whitespace() {
            if found_non_ws && word_end.is_none() {
                word_end = Some(i);
            }
            if word_end.is_some() {
                continue;
            }
        } else {
            if word_end.is_some() {
                return Some(offset + i);
            }
            found_non_ws = true;
        }
    }

    if word_end.is_some() {
        Some(offset + remaining.len())
    } else if found_non_ws {
        // Partial word, no trailing whitespace — wait for more data
        None
    } else {
        Some(offset + remaining.len())
    }
}

/// Error types for chat operations
#[derive(Clone, Debug, PartialEq)]
pub enum ChatErrorType {
    NoApiKey,
    NetworkError,
    StreamInterrupted,
    RateLimited,
    InvalidModel,
    TokenLimit,
    Unknown,
}

impl ChatErrorType {
    pub fn from_error_string(s: &str) -> Self {
        let s_lower = s.to_lowercase();
        if s_lower.contains("api key")
            || s_lower.contains("unauthorized")
            || s_lower.contains("401")
        {
            ChatErrorType::NoApiKey
        } else if s_lower.contains("network")
            || s_lower.contains("connection")
            || s_lower.contains("timeout")
        {
            ChatErrorType::NetworkError
        } else if s_lower.contains("interrupt") || s_lower.contains("abort") {
            ChatErrorType::StreamInterrupted
        } else if s_lower.contains("rate limit") || s_lower.contains("429") {
            ChatErrorType::RateLimited
        } else if s_lower.contains("model")
            && (s_lower.contains("invalid")
                || s_lower.contains("not found")
                || s_lower.contains("unavailable")
                || s_lower.contains("does not exist")
                || s_lower.contains("not supported"))
        {
            ChatErrorType::InvalidModel
        } else if s_lower.contains("token")
            || s_lower.contains("too long")
            || s_lower.contains("length")
        {
            ChatErrorType::TokenLimit
        } else {
            ChatErrorType::Unknown
        }
    }

    pub fn display_message(&self) -> &'static str {
        match self {
            ChatErrorType::NoApiKey => "⚠ API key not configured. Set up your API key to continue.",
            ChatErrorType::NetworkError => "⚠ Network error. Check your connection and try again.",
            ChatErrorType::StreamInterrupted => "⚠ Response interrupted. Click retry to continue.",
            ChatErrorType::RateLimited => "⚠ Rate limited. Please wait a moment and try again.",
            ChatErrorType::InvalidModel => "⚠ Model unavailable. Using default model.",
            ChatErrorType::TokenLimit => "⚠ Message too long. Try a shorter prompt.",
            ChatErrorType::Unknown => "⚠ Something went wrong. Please try again.",
        }
    }

    pub fn can_retry(&self) -> bool {
        matches!(
            self,
            ChatErrorType::NetworkError
                | ChatErrorType::StreamInterrupted
                | ChatErrorType::RateLimited
                | ChatErrorType::Unknown
        )
    }
}
