use super::*;

pub(super) const SP_1: gpui::Pixels = px(2.);
pub(super) const SP_2: gpui::Pixels = px(4.);
pub(super) const SP_3: gpui::Pixels = px(6.);
pub(super) const SP_4: gpui::Pixels = px(8.);
pub(super) const SP_5: gpui::Pixels = px(10.);
pub(super) const SP_6: gpui::Pixels = px(12.);
pub(super) const SP_7: gpui::Pixels = px(14.);
pub(super) const SP_8: gpui::Pixels = px(16.);
pub(super) const SP_9: gpui::Pixels = px(20.);
pub(super) const SP_10: gpui::Pixels = px(24.);

// -- Border radii --
pub(super) const RADIUS_XS: gpui::Pixels = px(3.);
pub(super) const RADIUS_SM: gpui::Pixels = px(4.);
pub(super) const RADIUS_MD: gpui::Pixels = px(6.);
pub(super) const RADIUS_LG: gpui::Pixels = px(10.);

// -- Icon sizes --
pub(super) const ICON_XS: gpui::Pixels = px(12.);
pub(super) const ICON_SM: gpui::Pixels = px(14.);
pub(super) const ICON_MD: gpui::Pixels = px(16.);

// -- Layout constants --
pub(super) const SIDEBAR_W: gpui::Pixels = px(240.);
pub(super) const TITLEBAR_H: gpui::Pixels = px(36.);

// -- Message bubble tokens --
pub(super) const MSG_PX: gpui::Pixels = SP_9;
pub(super) const MSG_PY: gpui::Pixels = SP_7;
pub(super) const MSG_RADIUS: gpui::Pixels = px(10.);
pub(super) const MSG_GAP: gpui::Pixels = SP_9;
pub(super) const MSG_GAP_CONTINUATION: gpui::Pixels = SP_3;

#[cfg(test)]
mod message_spacing_tests {
    use super::*;

    #[test]
    fn test_message_spacing_constants_preserve_transition_separation_when_using_scale_tokens() {
        assert_eq!(MSG_PX, SP_9);
        assert_eq!(MSG_PY, SP_7);
        assert_eq!(MSG_GAP, SP_9);
        assert_eq!(MSG_GAP_CONTINUATION, SP_3);
        assert!(MSG_GAP / MSG_GAP_CONTINUATION > 1.0);
    }
}

// -- Semantic opacity levels --
// Use named constants so the same semantic intent always gets the same value.
pub(super) const OP_SUBTLE: f32 = 0.15;
pub(super) const OP_MUTED: f32 = 0.3;
pub(super) const OP_MEDIUM: f32 = 0.5;
pub(super) const OP_STRONG: f32 = 0.7;
pub(super) const OP_NEAR_FULL: f32 = 0.85;

// -- Message-specific opacities (contrast-tuned) --
pub(super) const OP_USER_MSG_BG: f32 = 0.12; // user bubble tint (accent)
pub(super) const OP_ASSISTANT_MSG_BG: f32 = 0.10; // assistant bubble tint (muted)
pub(super) const OP_MSG_BORDER: f32 = 0.45; // left-border on user bubbles

// -- Dot separator --
pub(super) const DOT_SIZE: gpui::Pixels = px(3.);

/// Events from the streaming thread
pub(super) enum StreamingEvent {
    /// A chunk of text received
    Chunk(String),
    /// Streaming completed successfully
    Done,
    /// An error occurred
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct StreamingSessionKey {
    pub(super) chat_id: ChatId,
    pub(super) generation: u64,
}

pub(super) fn should_retry_existing_user_turn(messages: &[Message]) -> bool {
    messages
        .last()
        .map(|message| message.role == MessageRole::User)
        .unwrap_or(false)
}

pub(super) fn ai_window_can_submit_message(content: &str, has_pending_image: bool) -> bool {
    !content.trim().is_empty() || has_pending_image
}

pub(super) fn ai_window_prune_deleted_message_ui_state(
    collapsed_messages: &mut std::collections::HashSet<String>,
    expanded_messages: &mut std::collections::HashSet<String>,
    deleted_message_ids: &[String],
) {
    for message_id in deleted_message_ids {
        collapsed_messages.remove(message_id);
        expanded_messages.remove(message_id);
    }
}

pub(super) fn ai_window_queue_command_if_open(
    pending_commands: &mut Vec<AiCommand>,
    window_is_open: bool,
    command: AiCommand,
) -> bool {
    if !window_is_open {
        return false;
    }

    pending_commands.push(command);
    true
}

pub(super) fn should_persist_stale_completion(
    suppressed_sessions: &mut std::collections::HashSet<StreamingSessionKey>,
    session_key: StreamingSessionKey,
) -> bool {
    !suppressed_sessions.remove(&session_key)
}

/// A preset configuration for starting new chats
#[derive(Clone)]
pub(super) struct AiPreset {
    /// Unique identifier
    pub(super) id: &'static str,
    /// Display name
    pub(super) name: &'static str,
    /// Description shown in dropdown
    pub(super) description: &'static str,
    /// System prompt to use
    pub(super) system_prompt: &'static str,
    /// Icon name
    pub(super) icon: LocalIconName,
    /// Preferred model ID (if any)
    pub(super) preferred_model: Option<&'static str>,
}

/// A recently used model+provider configuration (for "Last Used Settings" in dropdown)
#[derive(Clone, Debug, PartialEq)]
pub(super) struct LastUsedSetting {
    /// Model ID (e.g., "claude-3-5-sonnet-20241022")
    pub(super) model_id: String,
    /// Provider name (e.g., "anthropic")
    pub(super) provider: String,
    /// Display name for the model
    pub(super) display_name: String,
    /// Provider display name
    pub(super) provider_display_name: String,
}

/// Internal enum for handling new chat dropdown selection
pub(super) enum NewChatAction {
    Model { model_id: String, provider: String },
    Preset { index: usize },
}

impl AiPreset {
    /// Get default presets
    pub(super) fn default_presets() -> Vec<AiPreset> {
        vec![
            AiPreset {
                id: "general",
                name: "General Assistant",
                description: "Helpful AI assistant for any task",
                system_prompt: "You are a helpful AI assistant.",
                icon: LocalIconName::Star,
                preferred_model: None,
            },
            AiPreset {
                id: "coder",
                name: "Code Assistant",
                description: "Expert programmer and debugger",
                system_prompt: "You are an expert programmer. Write clean, efficient, well-documented code. Explain your reasoning.",
                icon: LocalIconName::Code,
                preferred_model: None,
            },
            AiPreset {
                id: "writer",
                name: "Writing Assistant",
                description: "Help with writing and editing",
                system_prompt: "You are a skilled writer and editor. Help improve writing clarity, grammar, and style.",
                icon: LocalIconName::FileCode,
                preferred_model: None,
            },
            AiPreset {
                id: "researcher",
                name: "Research Assistant",
                description: "Deep analysis and research",
                system_prompt: "You are a thorough researcher. Analyze topics deeply, cite sources when possible, and provide comprehensive answers.",
                icon: LocalIconName::MagnifyingGlass,
                preferred_model: None,
            },
            AiPreset {
                id: "creative",
                name: "Creative Partner",
                description: "Brainstorming and creative ideas",
                system_prompt: "You are a creative partner. Help brainstorm ideas, think outside the box, and explore possibilities.",
                icon: LocalIconName::BoltFilled,
                preferred_model: None,
            },
        ]
    }
}

/// Date group categories for sidebar organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DateGroup {
    Today,
    Yesterday,
    ThisWeek,
    Older,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SidebarRow {
    Header { group: DateGroup, is_first: bool },
    Chat { chat_id: ChatId },
}

impl DateGroup {
    /// Get the display label for this group
    pub(super) fn label(&self) -> &'static str {
        match self {
            DateGroup::Today => "Today",
            DateGroup::Yesterday => "Yesterday",
            DateGroup::ThisWeek => "This Week",
            DateGroup::Older => "Older",
        }
    }
}

/// Determine which date group a date belongs to
pub(super) fn get_date_group(date: NaiveDate, today: NaiveDate) -> DateGroup {
    let days_ago = today.signed_duration_since(date).num_days();

    if days_ago == 0 {
        DateGroup::Today
    } else if days_ago == 1 {
        DateGroup::Yesterday
    } else if days_ago < 7
        && date.weekday().num_days_from_monday() < today.weekday().num_days_from_monday()
    {
        // Same week (and not earlier in a previous week)
        DateGroup::ThisWeek
    } else if days_ago < 7 {
        DateGroup::ThisWeek
    } else {
        DateGroup::Older
    }
}

/// Group chats by date categories
pub(super) fn group_chats_by_date(chats: &[Chat]) -> Vec<(DateGroup, Vec<&Chat>)> {
    let today = Utc::now().date_naive();

    let mut today_chats: Vec<&Chat> = Vec::new();
    let mut yesterday_chats: Vec<&Chat> = Vec::new();
    let mut this_week_chats: Vec<&Chat> = Vec::new();
    let mut older_chats: Vec<&Chat> = Vec::new();

    for chat in chats {
        let chat_date = chat.updated_at.date_naive();
        match get_date_group(chat_date, today) {
            DateGroup::Today => today_chats.push(chat),
            DateGroup::Yesterday => yesterday_chats.push(chat),
            DateGroup::ThisWeek => this_week_chats.push(chat),
            DateGroup::Older => older_chats.push(chat),
        }
    }

    let mut groups = Vec::new();
    if !today_chats.is_empty() {
        groups.push((DateGroup::Today, today_chats));
    }
    if !yesterday_chats.is_empty() {
        groups.push((DateGroup::Yesterday, yesterday_chats));
    }
    if !this_week_chats.is_empty() {
        groups.push((DateGroup::ThisWeek, this_week_chats));
    }
    if !older_chats.is_empty() {
        groups.push((DateGroup::Older, older_chats));
    }

    groups
}

pub(super) fn build_sidebar_rows_for_chats(chats: &[Chat]) -> Vec<SidebarRow> {
    let date_groups = group_chats_by_date(chats);
    let mut rows = Vec::with_capacity(chats.len() + date_groups.len());

    for (group_index, (group, group_chats)) in date_groups.into_iter().enumerate() {
        rows.push(SidebarRow::Header {
            group,
            is_first: group_index == 0,
        });
        rows.extend(
            group_chats
                .into_iter()
                .map(|chat| SidebarRow::Chat { chat_id: chat.id }),
        );
    }

    rows
}

/// Generate a contextual mock AI response based on the user's message
/// Used for demo/testing when no AI providers are configured
pub(super) fn generate_mock_response(user_message: &str) -> String {
    let msg_lower = user_message.to_lowercase();

    // Contextual responses based on common patterns
    if msg_lower.contains("hello") || msg_lower.contains("hi") || msg_lower.starts_with("hey") {
        return "Hello! I'm Script Kit's AI assistant running in demo mode. Since no API key is configured, I'm providing mock responses. To enable real AI, set `SCRIPT_KIT_ANTHROPIC_API_KEY` or `SCRIPT_KIT_OPENAI_API_KEY` in your environment.".to_string();
    }

    if msg_lower.contains("script") || msg_lower.contains("automation") {
        return "Script Kit is a powerful automation tool! Here are some things you can do:\n\n1. **Create scripts** - Write TypeScript/JavaScript to automate tasks\n2. **Use prompts** - `arg()`, `editor()`, `div()` for interactive UIs\n3. **Hotkeys** - Bind scripts to global keyboard shortcuts\n4. **Snippets** - Text expansion with dynamic content\n\nTry running a script with `Cmd+;` to see it in action!".to_string();
    }

    if msg_lower.contains("help") || msg_lower.contains("how") {
        return "I'm here to help! In demo mode, I can explain Script Kit concepts:\n\n• **Scripts** live in `~/.scriptkit/scripts/`\n• **SDK** provides `arg()`, `div()`, `editor()`, and more\n• **Hotkeys** are configured in script metadata\n• **This AI chat** works with Claude or GPT when you add an API key\n\nWhat would you like to know more about?".to_string();
    }

    if msg_lower.contains("code") || msg_lower.contains("example") {
        return "Here's a simple Script Kit example:\n\n```typescript\n// Name: Hello World\n// Shortcut: cmd+shift+h\n\nconst name = await arg(\"What's your name?\");\nawait div(`<h1>Hello, ${name}!</h1>`);\n```\n\nThis creates a script that:\n1. Asks for your name via a prompt\n2. Displays a greeting in an HTML view\n\nSave this to `~/.scriptkit/scripts/hello.ts` and run it!".to_string();
    }

    if msg_lower.contains("api") || msg_lower.contains("key") || msg_lower.contains("configure") {
        return "To enable real AI responses, configure an API key:\n\n**For Claude (Anthropic):**\n```bash\nexport SCRIPT_KIT_ANTHROPIC_API_KEY=\"sk-ant-...\"\n```\n\n**For GPT (OpenAI):**\n```bash\nexport SCRIPT_KIT_OPENAI_API_KEY=\"sk-...\"\n```\n\nAdd these to your `~/.zshrc` or `~/.scriptkit/.env` file, then restart Script Kit.".to_string();
    }

    // Default response for unrecognized queries
    format!(
        "I received your message: \"{}\"\n\n\
        I'm running in **demo mode** because no AI API key is configured. \
        My responses are pre-written examples.\n\n\
        To get real AI responses:\n\
        1. Get an API key from Anthropic or OpenAI\n\
        2. Set `SCRIPT_KIT_ANTHROPIC_API_KEY` or `SCRIPT_KIT_OPENAI_API_KEY`\n\
        3. Restart Script Kit\n\n\
        Try asking about \"scripts\", \"help\", or \"code examples\" to see more demo responses!",
        user_message.chars().take(50).collect::<String>()
    )
}

/// Global handle to the AI window
pub(super) static AI_WINDOW: std::sync::OnceLock<
    std::sync::Mutex<Option<gpui::WindowHandle<Root>>>,
> = std::sync::OnceLock::new();

/// Global flag to request input focus in the AI window.
/// This replaces the problematic AI_APP_ENTITY which caused memory leaks.
/// The flag is checked in AiApp::render() and cleared after use.
pub(super) static AI_FOCUS_REQUESTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Pending commands for the AI window (for testing via stdin).
/// These are processed in AiApp::render() to avoid needing a global entity reference.
pub(super) static AI_PENDING_COMMANDS: std::sync::OnceLock<std::sync::Mutex<Vec<AiCommand>>> =
    std::sync::OnceLock::new();

/// Commands that can be sent to the AI window (for testing)
#[derive(Clone)]
#[allow(clippy::enum_variant_names)]
pub(super) enum AiCommand {
    SetSearch(String),
    SetInput {
        text: String,
        submit: bool,
    },
    /// Set input with an attached image (base64 encoded PNG)
    SetInputWithImage {
        text: String,
        image_base64: String,
        submit: bool,
    },
    /// Initialize the chat with pending messages from open_ai_window_with_chat
    InitializeWithPendingChat,
    /// Show the command bar overlay (Cmd+K menu)
    ShowCommandBar,
    /// Simulate a key press (for testing)
    SimulateKey {
        key: String,
        modifiers: Vec<KeyModifier>,
    },
}

pub(super) fn get_pending_commands() -> &'static std::sync::Mutex<Vec<AiCommand>> {
    AI_PENDING_COMMANDS.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

pub(super) fn push_ai_command(cmd: AiCommand) {
    if let Ok(mut cmds) = get_pending_commands().lock() {
        cmds.push(cmd);
    }
}

pub(super) fn take_ai_commands() -> Vec<AiCommand> {
    get_pending_commands()
        .lock()
        .ok()
        .map(|mut cmds| std::mem::take(&mut *cmds))
        .unwrap_or_default()
}

// NOTE: AI_APP_ENTITY was removed to prevent memory leaks.
// The entity was being kept alive by this global reference and by theme watcher tasks,
// causing the AiApp to never be dropped even after the window closed.
// Instead, we use AI_FOCUS_REQUESTED (AtomicBool) which AiApp checks in render().
